//! Freeze an inspectable, content-identified knowledge cut for reproducible analysis.
use crate::{
    model::Evidence,
    store::{RetainedRecord, Store},
};
use anyhow::{Result, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufWriter, Write};

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
        // Preserve the v1 digest's field order and empty hash without cloning
        // every retained record or allocating its complete JSON representation.
        #[derive(Serialize)]
        struct Content<'a> {
            schema: &'a str,
            calculation_version: &'a str,
            package_version: &'a str,
            implementation_revision: &'a str,
            implementation_source_sha256: &'a str,
            as_of: DateTime<Utc>,
            records: &'a [RetainedRecord],
            sha256: &'a str,
        }
        struct HashWriter<'a>(&'a mut Sha256);
        impl Write for HashWriter<'_> {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.update(bytes);
                Ok(bytes.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let content = Content {
            schema: &self.schema,
            calculation_version: &self.calculation_version,
            package_version: &self.package_version,
            implementation_revision: &self.implementation_revision,
            implementation_source_sha256: &self.implementation_source_sha256,
            as_of: self.as_of,
            records: &self.records,
            sha256: "",
        };
        let mut hash = Sha256::new();
        {
            let mut writer = BufWriter::new(HashWriter(&mut hash));
            serde_json::to_writer(&mut writer, &content)?;
            writer.flush()?;
        }
        Ok(hash
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect())
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
