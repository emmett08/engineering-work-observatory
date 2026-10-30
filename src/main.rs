//! The local command interface; policies and external adapters live in library modules.
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use observatory::{
    collectors, importers,
    ingestion::{self, CollectionRequirements},
    model::Evidence,
    report,
    snapshot::Snapshot,
    store::Store,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(
    version,
    about = "Collect evidence and measure effort, acceptance and retained output"
)]
struct Cli {
    #[arg(long, global = true, default_value = ".observatory")]
    state: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CollectionProfile {
    Full,
    Throughput,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ForecastHorizon {
    Day,
    Week,
    Fortnight,
}

impl From<ForecastHorizon> for observatory::forecast::ForecastPeriod {
    fn from(value: ForecastHorizon) -> Self {
        match value {
            ForecastHorizon::Day => Self::Day,
            ForecastHorizon::Week => Self::Week,
            ForecastHorizon::Fortnight => Self::Fortnight,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Create the local evidence store.
    Init,
    /// Import a complete JSONL study batch or an observatory.bundle/1 JSON document.
    Import {
        file: PathBuf,
        #[arg(long, default_value = "study-records")]
        source: String,
    },
    /// Collect both project repositories by default; retain partial-source coverage on errors.
    CollectGithub {
        #[arg(long = "repository")]
        repositories: Vec<String>,
        #[arg(long, default_value = "https://api.github.com")]
        api_base: String,
        #[arg(long, default_value = "GH_TOKEN")]
        token_env: String,
        #[arg(long, value_enum, default_value_t = CollectionProfile::Full)]
        profile: CollectionProfile,
    },
    /// Capture a local repository without checking out or executing its code.
    CollectGit {
        path: PathBuf,
        #[arg(long)]
        repository: String,
    },
    /// Import a report, audit or checkpoint with an explicit stable lineage manifest.
    ImportEngarg {
        file: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
    },
    /// Take a consistent read-only snapshot of an EAL SQLite store, or import an export.
    ImportEal {
        file: PathBuf,
        #[arg(long)]
        namespace: String,
    },
    /// Freeze a content-identified knowledge cut (default: now).
    Snapshot {
        #[arg(long)]
        as_of: Option<DateTime<Utc>>,
        #[arg(long)]
        output: PathBuf,
    },
    /// Recompute an offline report from a verified frozen snapshot.
    Analyse {
        snapshot: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Check retained source hashes, record hashes and SQLite integrity.
    Verify,
    /// Evaluate a synthetic conditional retry scenario, separate from empirical analysis.
    Simulate { file: PathBuf },
    /// Estimate frame labour from frozen predictions and probability-sampled observations.
    EstimateLabour { file: PathBuf },
    /// Compute exact attainable labour bounds under interval and shared-capacity constraints.
    BoundLabour { file: PathBuf },
    /// Collect automatic PR evidence, freeze a cut and issue an immutable team forecast.
    ForecastTeam {
        #[arg(long = "repository")]
        repositories: Vec<String>,
        #[arg(long, value_enum, default_value_t = ForecastHorizon::Day)]
        horizon: ForecastHorizon,
        #[arg(long, default_value = "https://api.github.com")]
        api_base: String,
        #[arg(long, default_value = "GH_TOKEN")]
        token_env: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Reproduce forecasts or event-time reconstruction scores from a retained source cut.
    ForecastSnapshot {
        snapshot: PathBuf,
        #[arg(long = "repository")]
        repositories: Vec<String>,
        #[arg(long, value_enum, default_value_t = ForecastHorizon::Day)]
        horizon: ForecastHorizon,
        #[arg(long)]
        backtest: bool,
        #[arg(long, requires = "initial_training_bins")]
        history_bins: Option<usize>,
        #[arg(long, requires = "history_bins")]
        initial_training_bins: Option<usize>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Score an original command-issued forecast against a later verified snapshot.
    ScoreForecast {
        bundle: PathBuf,
        later_snapshot: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn read_input(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    const LIMIT: u64 = 128 * 1024 * 1024;
    ensure!(
        fs::metadata(path)?.len() <= LIMIT,
        "input exceeds 128 MiB limit"
    );
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= LIMIT, "input exceeds 128 MiB limit");
    Ok(bytes)
}

fn parse_records(bytes: &[u8]) -> Result<Vec<Evidence>> {
    if let Ok(value) = serde_json::from_slice::<Value>(bytes) {
        if value.get("schema").is_some() {
            ensure!(
                value.get("schema").and_then(Value::as_str) == Some("observatory.bundle/1"),
                "unsupported study bundle schema"
            );
            return serde_json::from_value(
                value
                    .get("records")
                    .context("bundle records missing")?
                    .clone(),
            )
            .context("invalid study bundle");
        }
        if value.is_array() {
            return serde_json::from_value(value).context("invalid record array");
        }
        return Ok(vec![
            serde_json::from_value(value).context("invalid record")?,
        ]);
    }
    let text = std::str::from_utf8(bytes).context("study records must be UTF-8")?;
    let mut records = Vec::new();
    for (line_number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        records.push(
            serde_json::from_str(line)
                .with_context(|| format!("invalid JSONL line {}", line_number + 1))?,
        );
    }
    ensure!(!records.is_empty(), "empty study batch");
    Ok(records)
}

fn save_new(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path)
        .context("refusing to overwrite an existing result")?;
    Ok(())
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
    match cli.command {
        Command::Analyse { snapshot, output } => {
            let snapshot: Snapshot = serde_json::from_slice(&read_input(&snapshot)?)?;
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
            let declaration: Value = serde_json::from_slice(&bytes)?;
            let result = match declaration.get("schema").and_then(Value::as_str) {
                Some("observatory.labour-estimate-input/1") => {
                    let frame = serde_json::from_slice(&bytes)?;
                    serde_json::to_string_pretty(&observatory::inference::estimate_labour(&frame)?)?
                }
                Some("observatory.labour-estimate-input/2") => {
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
            let snapshot: Snapshot = serde_json::from_slice(&read_input(&snapshot)?)?;
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
            let later = serde_json::from_slice(&read_input(&later_snapshot)?)?;
            let score = observatory::forecast::archive::score_issued(&prior, &later)?;
            let bytes = serde_json::to_vec_pretty(&score)?;
            if let Some(output) = output {
                save_new(&output, &bytes)?;
            }
            println!("{}", std::str::from_utf8(&bytes)?);
        }
        command => {
            let mut store = Store::open(&cli.state)?;
            match command {
                Command::Init => println!("Initialised {}", cli.state.display()),
                Command::Verify => println!("Verified {} retained records", store.verify()?),
                Command::Import { file, source } => {
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
                    let repositories = selected_repositories(repositories);
                    let archive_directory = cli.state.join("forecasts");
                    let archive_path = archive_directory
                        .join(format!("{}.json", Utc::now().format("%Y%m%dT%H%M%S%.9fZ")));
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
                    let scores = observatory::forecast::archive::score_directory(
                        &archive_directory,
                        &snapshot,
                    )?;
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
                    let collection = collectors::collect_git(&path, &repository)?;
                    persist_collection(&mut store, collection, &format!("git:{repository}"))?;
                }
                Command::ImportEngarg { file, manifest } => {
                    let bytes = read_input(&file)?;
                    let artifact_hash = store.retain_bytes(&bytes)?;
                    let manifest_bytes = read_input(&manifest)?;
                    let manifest_hash = store.retain_bytes(&manifest_bytes)?;
                    let records = importers::engarg_import(
                        &store.object_path(&artifact_hash)?,
                        &store.object_path(&manifest_hash)?,
                    )?
                    .into_iter()
                    .map(|r| Evidence {
                        id: r.id,
                        kind: r.kind,
                        data: r.data,
                    })
                    .collect::<Vec<_>>();
                    let import_manifest = json!({"schema":"observatory.import/1","raw_objects":[artifact_hash,manifest_hash]});
                    let hash = store.retain_bytes(&serde_json::to_vec(&import_manifest)?)?;
                    let count = store.insert_batch(&records, "engarg-export", Utc::now(), &hash)?;
                    println!("Imported {count} new EngArg evidence records");
                }
                Command::ImportEal { file, namespace } => {
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
                        .map(|r| Evidence {
                            id: r.id,
                            kind: r.kind,
                            data: r.data,
                        })
                        .collect::<Vec<_>>();
                    let count = store.insert_batch(&records, "eal-export", Utc::now(), &hash)?;
                    println!("Imported {count} new EAL evidence records");
                }
                Command::Snapshot { as_of, output } => {
                    let snapshot = Snapshot::freeze(&store, as_of.unwrap_or_else(Utc::now))?;
                    save_new(&output, &serde_json::to_vec_pretty(&snapshot)?)?;
                    println!(
                        "Snapshot {} written to {}",
                        snapshot.sha256,
                        output.display()
                    );
                }
                Command::Analyse { .. }
                | Command::Simulate { .. }
                | Command::EstimateLabour { .. }
                | Command::BoundLabour { .. }
                | Command::ForecastSnapshot { .. }
                | Command::ScoreForecast { .. } => unreachable!(),
            }
        }
    }
    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    if let Err(error) = execute(Cli::parse()) {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_batch_does_not_accept_prefix() {
        assert!(
            parse_records(b"{\"id\":\"a\",\"kind\":\"source_record\",\"data\":{}}\n{broken")
                .is_err()
        );
    }
}
