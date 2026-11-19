//! Dispatch commands and acquire a store only for operations that use it.
mod files;
mod options;

use anyhow::{Result, ensure};
use chrono::Utc;
use clap::Parser;
use files::{parse_records, read_input, read_snapshot, save_new};
use observatory::{
    collectors, importers,
    ingestion::{self, CollectionRequirements},
    model::Evidence,
    report,
    snapshot::Snapshot,
    store::Store,
};
use options::{Cli, CollectionProfile, Command};
use serde::Deserialize;
use serde_json::json;
use std::{fs, path::Path};

pub(super) fn run() -> Result<()> {
    execute(Cli::parse())
}

fn persist_collection(
    store: &mut Store,
    collection: collectors::Collection,
    source: &str,
) -> Result<()> {
    let receipt =
        ingestion::persist_collection(store, collection, source, CollectionRequirements::Full)?;
    println!(
        "Retained {} new records; collection manifest {}",
        receipt.new_records, receipt.manifest_sha256
    );
    Ok(())
}

fn selected_repositories(repositories: Vec<String>) -> Vec<String> {
    if repositories.is_empty() {
        vec!["emmett08/eal-mcp".into(), "emmett08/engarg".into()]
    } else {
        repositories
    }
}

/// Wire repository acquisition to the shared retained-ingestion boundary.
fn collect_repositories(
    store: &mut Store,
    state: &Path,
    repositories: &[String],
    api_base: &str,
    token_env: &str,
    profile: CollectionProfile,
    show_receipts: bool,
) -> Result<()> {
    let token = std::env::var(token_env).ok();
    let mut failures = Vec::new();
    for repository in repositories {
        tracing::info!(repository, "collecting GitHub evidence");
        let collection = match profile {
            CollectionProfile::Full => collectors::collect_github_cached(
                repository,
                token.as_deref(),
                api_base,
                &state.join("cache"),
            )?,
            CollectionProfile::Throughput => collectors::collect_github_throughput_cached(
                repository,
                token.as_deref(),
                api_base,
                &state.join("cache"),
            )?,
        };
        let requirements = match profile {
            CollectionProfile::Full => CollectionRequirements::Full,
            CollectionProfile::Throughput => CollectionRequirements::Throughput,
        };
        match ingestion::persist_collection(
            store,
            collection,
            &format!("github:{repository}"),
            requirements,
        ) {
            Ok(receipt) => {
                if show_receipts {
                    println!(
                        "Retained {} new records; collection manifest {}",
                        receipt.new_records, receipt.manifest_sha256
                    );
                } else {
                    tracing::info!(new_records=receipt.new_records, manifest=%receipt.manifest_sha256, "retained source collection");
                }
            }
            Err(error) => failures.push(format!("{repository}: {error}")),
        }
    }
    ensure!(failures.is_empty(), "{}", failures.join("; "));
    Ok(())
}

fn execute(cli: Cli) -> Result<()> {
    let open_store = || Store::open(&cli.state);
    match cli.command {
        Command::Analyse { snapshot, output } => {
            let snapshot = read_snapshot(&snapshot)?;
            report::write_report(&snapshot, &output)?;
            println!("Report written to {}", output.display());
        }
        Command::Simulate { file } => {
            let scenario = serde_json::from_slice(&read_input(&file)?)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&observatory::simulations::retry(&scenario)?)?
            );
        }
        Command::EstimateLabour { file } => {
            let bytes = read_input(&file)?;
            #[derive(Deserialize)]
            struct Declaration {
                schema: String,
            }
            let declaration: Declaration = serde_json::from_slice(&bytes)?;
            let result = match declaration.schema.as_str() {
                "observatory.labour-estimate-input/1" => {
                    let frame = serde_json::from_slice(&bytes)?;
                    serde_json::to_string_pretty(&observatory::inference::estimate_labour(&frame)?)?
                }
                "observatory.labour-estimate-input/2" => {
                    let frame = serde_json::from_slice(&bytes)?;
                    serde_json::to_string_pretty(&observatory::inference::estimate_labour_v2(
                        &frame,
                    )?)?
                }
                _ => anyhow::bail!("unsupported labour estimate input schema"),
            };
            println!("{result}");
        }
        Command::BoundLabour { file } => {
            let input = serde_json::from_slice(&read_input(&file)?)?;
            let result = observatory::identification::bound_labour(&input)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Command::ForecastSnapshot {
            snapshot,
            repositories,
            horizon,
            backtest,
            history_bins,
            initial_training_bins,
            output,
        } => {
            let snapshot = read_snapshot(&snapshot)?;
            ensure!(
                backtest || history_bins.is_none(),
                "history window options require --backtest"
            );
            let history = observatory::forecast::history::prepare(&snapshot, &repositories)?;
            let result = if backtest {
                let backtest =
                    if let (Some(bins), Some(initial)) = (history_bins, initial_training_bins) {
                        observatory::forecast::reconstruct_backtest_window(
                            &history,
                            horizon.into(),
                            bins,
                            initial,
                        )?
                    } else {
                        observatory::forecast::reconstruct_backtest(&history, horizon.into())?
                    };
                serde_json::to_value(backtest)?
            } else {
                json!({"schema":"observatory.team-forecast-replay/1", "history":history, "forecast":observatory::forecast::forecast(&history, horizon.into())?})
            };
            if let Some(output) = output {
                save_new(&output, &serde_json::to_vec_pretty(&result)?)?;
            }
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Command::ScoreForecast {
            bundle,
            later_snapshot,
            output,
        } => {
            let prior = serde_json::from_slice(&read_input(&bundle)?)?;
            let later = read_snapshot(&later_snapshot)?;
            let score = observatory::forecast::archive::score_issued(&prior, &later)?;
            let bytes = serde_json::to_vec_pretty(&score)?;
            if let Some(output) = output {
                save_new(&output, &bytes)?;
            }
            println!("{}", std::str::from_utf8(&bytes)?);
        }
        Command::Init => {
            open_store()?;
            println!("Initialised {}", cli.state.display());
        }
        Command::Verify => println!("Verified {} retained records", open_store()?.verify()?),
        Command::Import { file, source } => {
            let mut store = open_store()?;
            let bytes = read_input(&file)?;
            let hash = store.retain_bytes(&bytes)?;
            let records = parse_records(&bytes)?;
            let count = store.insert_batch(&records, &source, Utc::now(), &hash)?;
            println!("Imported {count} new study records");
        }
        Command::CollectGithub {
            repositories,
            api_base,
            token_env,
            profile,
        } => {
            let mut store = open_store()?;
            let repositories = selected_repositories(repositories);
            collect_repositories(
                &mut store,
                &cli.state,
                &repositories,
                &api_base,
                &token_env,
                profile,
                true,
            )?;
        }
        Command::ForecastTeam {
            repositories,
            horizon,
            api_base,
            token_env,
            output,
        } => {
            let mut store = open_store()?;
            let repositories = selected_repositories(repositories);
            let archive_directory = cli.state.join("forecasts");
            let archive_path =
                archive_directory.join(format!("{}.json", Utc::now().format("%Y%m%dT%H%M%S%.9fZ")));
            ensure!(
                !archive_path.exists(),
                "refusing to overwrite an existing forecast"
            );
            if let Some(output) = &output {
                ensure!(
                    !output.exists(),
                    "refusing to overwrite an existing forecast export"
                );
            }
            collect_repositories(
                &mut store,
                &cli.state,
                &repositories,
                &api_base,
                &token_env,
                CollectionProfile::Throughput,
                false,
            )?;
            let snapshot = Snapshot::freeze(&store, Utc::now())?;
            let scores =
                observatory::forecast::archive::score_directory(&archive_directory, &snapshot)?;
            let bundle = observatory::forecast::archive::IssuedForecast::issue(
                snapshot,
                &repositories,
                horizon.into(),
            )?;
            let score_path = cli
                .state
                .join("forecast-scores")
                .join(format!("{}.json", bundle.snapshot.sha256));
            let bytes = serde_json::to_vec_pretty(&bundle)?;
            save_new(&archive_path, &bytes)?;
            save_new(&score_path, &serde_json::to_vec_pretty(&scores)?)?;
            if let Some(output) = output {
                save_new(&output, &bytes)?;
            }
            eprintln!(
                "Immutable forecast saved to {}; prior outcome scores saved to {}",
                archive_path.display(),
                score_path.display()
            );
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"forecast":bundle.forecast, "current_pressure":bundle.history.current_pressure, "previous_forecast_scores":scores})
                )?
            );
        }
        Command::CollectGit { path, repository } => {
            let mut store = open_store()?;
            let collection = collectors::collect_git(&path, &repository)?;
            persist_collection(&mut store, collection, &format!("git:{repository}"))?;
        }
        Command::ImportEngarg { file, manifest } => {
            let mut store = open_store()?;
            let bytes = read_input(&file)?;
            let artifact_hash = store.retain_bytes(&bytes)?;
            let manifest_bytes = read_input(&manifest)?;
            let manifest_hash = store.retain_bytes(&manifest_bytes)?;
            let records = importers::engarg_import(
                &store.object_path(&artifact_hash)?,
                &store.object_path(&manifest_hash)?,
            )?
            .into_iter()
            .map(Evidence::from)
            .collect::<Vec<_>>();
            let import_manifest = json!({"schema":"observatory.import/1","raw_objects":[artifact_hash,manifest_hash]});
            let hash = store.retain_bytes(&serde_json::to_vec(&import_manifest)?)?;
            let count = store.insert_batch(&records, "engarg-export", Utc::now(), &hash)?;
            println!("Imported {count} new EngArg evidence records");
        }
        Command::ImportExecutions { file } => {
            let mut store = open_store()?;
            let bytes = read_input(&file)?;
            let records = importers::executions_import(&bytes)?
                .into_iter()
                .map(Evidence::from)
                .collect::<Vec<_>>();
            let artifact_hash = store.retain_bytes(&bytes)?;
            let manifest = json!({
                "schema":"observatory.import/1",
                "source":"observatory.tool_receipt",
                "raw_objects":[artifact_hash]
            });
            let hash = store.retain_bytes(&serde_json::to_vec(&manifest)?)?;
            let count = store.insert_batch(&records, "execution-bundle", Utc::now(), &hash)?;
            println!("Imported {count} new execution evidence records");
        }
        Command::CollectGithubExecutions {
            repository,
            run_id,
            api_base,
            token_env,
        } => {
            let mut store = open_store()?;
            let token = std::env::var(token_env).ok();
            let collection = collectors::collect_github_executions(
                &repository,
                token.as_deref(),
                &api_base,
                run_id,
            )?;
            let receipt = ingestion::persist_execution_collection(
                &mut store,
                collection,
                &format!("github-executions:{repository}"),
            )?;
            println!(
                "Retained {} new records; collection manifest {}",
                receipt.new_records, receipt.manifest_sha256
            );
        }
        Command::ImportEal { file, namespace } => {
            let mut store = open_store()?;
            let scratch = tempfile::tempdir()?;
            let snapshot_path = scratch.path().join("eal.sqlite3");
            use std::io::Read;
            let mut header = [0u8; 16];
            let count = fs::File::open(&file)?.read(&mut header)?;
            let source = if count == 16 && &header == b"SQLite format 3\0" {
                importers::snapshot_eal_sqlite(&file, &snapshot_path)?;
                snapshot_path.as_path()
            } else {
                &file
            };
            let hash = store.retain_bytes(&read_input(source)?)?;
            let records = importers::eal_import(&store.object_path(&hash)?, &namespace)?
                .into_iter()
                .map(Evidence::from)
                .collect::<Vec<_>>();
            let count = store.insert_batch(&records, "eal-export", Utc::now(), &hash)?;
            println!("Imported {count} new EAL evidence records");
        }
        Command::Snapshot { as_of, output } => {
            let store = open_store()?;
            let snapshot = Snapshot::freeze(&store, as_of.unwrap_or_else(Utc::now))?;
            save_new(&output, &serde_json::to_vec_pretty(&snapshot)?)?;
            println!(
                "Snapshot {} written to {}",
                snapshot.sha256,
                output.display()
            );
        }
    }
    Ok(())
}
