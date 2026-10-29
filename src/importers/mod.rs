//! Read-only adapters for retained EngArg and EAL artifacts.
//!
//! Imported calls describe product execution. They never establish development
//! exposure, human effort, invoiced cost, or successful engineering outcomes.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

mod eal;
mod engarg;

pub use eal::{eal_import, snapshot_eal_sqlite};
pub use engarg::{EngargManifest, engarg_import};

const MAX_ARTIFACT_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImportRecord {
    pub id: String,
    pub kind: String,
    pub data: Value,
}

fn digest(bytes: impl AsRef<[u8]>) -> String {
    crate::store::digest(bytes.as_ref())
}

fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    ensure!(
        fs::metadata(path)?.len() <= MAX_ARTIFACT_BYTES,
        "artifact exceeds 128 MiB import limit"
    );
    let bytes = fs::read(path)?;
    ensure!(
        bytes.len() as u64 <= MAX_ARTIFACT_BYTES,
        "artifact exceeds import limit"
    );
    Ok(bytes)
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .with_context(|| format!("missing or invalid {key}"))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:/-".contains(&b))
}
