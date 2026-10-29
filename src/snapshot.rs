//! Freeze an inspectable, content-identified knowledge cut for reproducible analysis.
use crate::{
    model::Evidence,
    store::{RetainedRecord, Store, digest},
};
use anyhow::{Result, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CALCULATION_VERSION: &str = "observatory.task-accounting/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema: String,
    pub calculation_version: String,
    pub package_version: String,
    pub implementation_revision: String,
    pub implementation_source_sha256: String,
    pub as_of: DateTime<Utc>,
    pub records: Vec<RetainedRecord>,
    pub sha256: String,
}

impl Snapshot {
    pub fn freeze(store: &Store, as_of: DateTime<Utc>) -> Result<Self> {
        store.verify()?;
        let mut snapshot = Self {
            schema: "observatory.snapshot/1".into(),
            calculation_version: CALCULATION_VERSION.into(),
            package_version: env!("CARGO_PKG_VERSION").into(),
            implementation_revision: env!("OBSERVATORY_BUILD_REVISION").into(),
            implementation_source_sha256: env!("OBSERVATORY_SOURCE_SHA256").into(),
            as_of,
            records: store.retained_records(Some(as_of))?,
            sha256: String::new(),
        };
        snapshot.sha256 = snapshot.compute_digest()?;
        snapshot.verify()?;
        Ok(snapshot)
    }

    fn compute_digest(&self) -> Result<String> {
        let mut content = self.clone();
        content.sha256.clear();
        Ok(digest(&serde_json::to_vec(&content)?))
    }

    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.schema == "observatory.snapshot/1",
            "unsupported snapshot schema"
        );
        ensure!(
            self.calculation_version == CALCULATION_VERSION,
            "unsupported calculation version"
        );
        ensure!(
            self.compute_digest()? == self.sha256,
            "snapshot digest mismatch"
        );
        ensure!(
            self.records.iter().all(|r| r.observed_at <= self.as_of),
            "snapshot contains future observations"
        );
        ensure!(
            self.records
                .windows(2)
                .all(|r| r[0].record.id < r[1].record.id),
            "snapshot record IDs must be sorted and unique"
        );
        crate::model::validate_records(&self.evidence())?;
        Ok(())
    }

    pub fn evidence(&self) -> Vec<Evidence> {
        self.records.iter().map(|r| r.record.clone()).collect()
    }
}
