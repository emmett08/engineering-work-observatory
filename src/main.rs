//! The local command interface; policies and external adapters live in library modules.
use anyhow::{Context, Result, bail, ensure};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use observatory::{
    collectors, importers,
    model::Evidence,
    report,
    snapshot::Snapshot,
    store::{Store, digest},
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
    let mut raw_objects = Vec::new();
    let mut response_metadata = Vec::new();
    for observation in &collection.observations {
        let hash = store.retain_bytes(&observation.body)?;
        raw_objects.push(hash.clone());
        let mut value = serde_json::to_value(observation)?;
        if let Some(map) = value.as_object_mut() {
            map.remove("body");
            map.insert("body_sha256".into(), json!(hash));
        }
        response_metadata.push(value);
    }
    let manifest = json!({"schema":"observatory.collection/1","source":source,"raw_objects":raw_objects,"responses":response_metadata,"coverage":collection.coverage});
    let bytes = serde_json::to_vec(&manifest)?;
    let hash = store.retain_bytes(&bytes)?;
    let mut records: Vec<Evidence> = collection
        .records
        .into_iter()
        .map(|r| Evidence {
            id: r.id,
            kind: r.kind,
            data: r.data,
        })
        .collect();
    for coverage in &collection.coverage {
        let data = json!({"record_type":"collection_coverage","coverage":coverage});
        records.push(Evidence {
            id: format!("coverage:{}", digest(&serde_json::to_vec(&data)?)),
            kind: "source_record".into(),
            data,
        });
    }
    let count = store.insert_batch(&records, source, Utc::now(), &hash)?;
    println!("Retained {count} new records; collection manifest {hash}");
    let incomplete: Vec<_> = collection
        .coverage
        .iter()
        .filter(|c| {
            c.state != "complete"
                && c.state != "not_applicable"
                && !(c.state == "not_collected"
                    && ["github.workflow_logs", "github.history"].contains(&c.source.as_str()))
        })
        .collect();
    if !incomplete.is_empty() {
        bail!(
            "collection retained with incomplete coverage: {}",
            serde_json::to_string(&incomplete)?
        );
    }
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
                    mut repositories,
                    api_base,
                    token_env,
                } => {
                    if repositories.is_empty() {
                        repositories = vec!["emmett08/eal-mcp".into(), "emmett08/engarg".into()];
                    }
                    let token = std::env::var(&token_env).ok();
                    let mut failures = Vec::new();
                    for repository in repositories {
                        tracing::info!(repository, "collecting GitHub evidence");
                        let collection = collectors::collect_github_cached(
                            &repository,
                            token.as_deref(),
                            &api_base,
                            &cli.state.join("cache"),
                        )?;
                        if let Err(e) = persist_collection(
                            &mut store,
                            collection,
                            &format!("github:{repository}"),
                        ) {
                            failures.push(format!("{repository}: {e}"));
                        }
                    }
                    ensure!(failures.is_empty(), "{}", failures.join("; "));
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
                Command::Analyse { .. } | Command::Simulate { .. } => unreachable!(),
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
