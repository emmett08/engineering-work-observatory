use anyhow::{Context, Result, ensure};
use observatory::{model::Evidence, snapshot::Snapshot};
use serde_json::Value;
use std::{fs, path::Path};

pub(super) fn read_input(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    const LIMIT: u64 = 128 * 1024 * 1024;
    let file =
        fs::File::open(path).with_context(|| format!("cannot open input {}", path.display()))?;
    ensure!(
        file.metadata()?.len() <= LIMIT,
        "input exceeds 128 MiB limit"
    );
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= LIMIT, "input exceeds 128 MiB limit");
    Ok(bytes)
}

/// Aggregate snapshots can exceed the per-import limit as a study grows.
/// Parse through a buffer instead of allocating another copy of the whole file.
pub(super) fn read_snapshot(path: &Path) -> Result<Snapshot> {
    let file =
        fs::File::open(path).with_context(|| format!("cannot open snapshot {}", path.display()))?;
    serde_json::from_reader(std::io::BufReader::new(file))
        .with_context(|| format!("cannot parse snapshot {}", path.display()))
}

pub(super) fn parse_records(bytes: &[u8]) -> Result<Vec<Evidence>> {
    if let Ok(mut value) = serde_json::from_slice::<Value>(bytes) {
        if value.get("schema").is_some() {
            ensure!(
                value.get("schema").and_then(Value::as_str) == Some("observatory.bundle/1"),
                "unsupported study bundle schema"
            );
            return serde_json::from_value(
                value
                    .get_mut("records")
                    .context("bundle records missing")?
                    .take(),
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

pub(super) fn save_new(path: &Path, bytes: &[u8]) -> Result<()> {
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
