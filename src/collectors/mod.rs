//! Read-only source adapters. Coverage describes retrieval, never engineering success.
mod git;
mod github;

pub use git::collect_git;
pub use github::{
    collect_github, collect_github_cached, collect_github_throughput,
    collect_github_throughput_cached,
};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedRecord {
    pub id: String,
    pub kind: String,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawResponse {
    pub source: String,
    pub url: String,
    pub collected_at: String,
    pub status: Option<u16>,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
    pub sha256: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage {
    pub source: String,
    pub scope: String,
    /// complete, incomplete, request_failed, not_collected, or not_applicable.
    pub state: String,
    pub pages: usize,
    pub records: usize,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Collection {
    pub records: Vec<CollectedRecord>,
    pub observations: Vec<RawResponse>,
    pub coverage: Vec<Coverage>,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn push_record(out: &mut Collection, repository: &str, family: &str, key: &str, payload: Value) {
    // Collection time belongs to the evidence envelope, not the content identity.
    let bytes = serde_json::to_vec(&payload).expect("JSON values serialize");
    let hash = digest(&bytes);
    out.records.push(CollectedRecord {
        id: format!("{repository}/{family}/{key}/{hash}"),
        kind: "source_record".into(),
        data: json!({
            "schema_version": 1,
            "source": family,
            "repository": repository,
            "source_id": key,
            "content_sha256": hash,
            // Original timestamp fields in payload remain source event times.
            "payload": payload,
        }),
    });
}
