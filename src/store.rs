//! Persist immutable records and source bytes with atomic batch insertion.
use crate::model::{Evidence, validate_records};
use anyhow::{Context, Result, bail, ensure};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub struct Store {
    connection: Connection,
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainedRecord {
    pub record: Evidence,
    pub observed_at: DateTime<Utc>,
    pub source: String,
    pub raw_sha256: String,
}

impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        let is_new = !root.exists();
        fs::create_dir_all(root)?;
        #[cfg(unix)]
        if is_new {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
        }
        #[cfg(not(unix))]
        let _ = is_new;
        fs::create_dir_all(root.join("objects"))?;
        let connection = Connection::open(root.join("observatory.sqlite3"))?;
        connection.busy_timeout(std::time::Duration::from_secs(10))?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;
          CREATE TABLE IF NOT EXISTS schema_version(version INTEGER PRIMARY KEY);
          INSERT OR IGNORE INTO schema_version VALUES(1);
          CREATE TABLE IF NOT EXISTS records(
            id TEXT PRIMARY KEY, kind TEXT NOT NULL, payload TEXT NOT NULL, sha256 TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS observations(
            record_id TEXT NOT NULL REFERENCES records(id), observed_at TEXT NOT NULL,
            source TEXT NOT NULL, raw_sha256 TEXT NOT NULL,
            PRIMARY KEY(record_id,observed_at,source,raw_sha256));
          CREATE INDEX IF NOT EXISTS record_kind ON records(kind);
          CREATE INDEX IF NOT EXISTS observation_time ON observations(observed_at,record_id);
          CREATE VIEW IF NOT EXISTS tasks AS SELECT id,json_extract(payload,'$.data.protocol_id') AS protocol_id,
            json_extract(payload,'$.data.requirement') AS requirement,
            json_extract(payload,'$.data.task_class') AS task_class,
            json_extract(payload,'$.data.enrolled_at') AS enrolled_at,
            json_extract(payload,'$.data.frozen_score') AS frozen_score FROM records WHERE kind='task';
          CREATE VIEW IF NOT EXISTS effort_intervals AS SELECT id,json_extract(payload,'$.data.task_id') AS task_id,
            json_extract(payload,'$.data.person_id') AS person_id,json_extract(payload,'$.data.role') AS role,
            json_extract(payload,'$.data.start') AS start,json_extract(payload,'$.data.end') AS end,
            json_extract(payload,'$.data.allocation_fraction') AS allocation_fraction FROM records WHERE kind='effort';
          CREATE VIEW IF NOT EXISTS source_records AS SELECT id,json_extract(payload,'$.data') AS data FROM records WHERE kind='source_record';")?;
        let versions: Vec<i64> = connection
            .prepare("SELECT version FROM schema_version")?
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        ensure!(versions == [1], "unsupported database schema version");
        Ok(Self {
            connection,
            root: root.to_owned(),
        })
    }

    pub fn retain_bytes(&self, bytes: &[u8]) -> Result<String> {
        let hash = digest(bytes);
        let destination = self.object_path(&hash)?;
        if destination.exists() {
            ensure!(
                digest(&fs::read(&destination)?) == hash,
                "retained object integrity failure"
            );
            return Ok(hash);
        }
        let mut temporary = tempfile::NamedTempFile::new_in(self.root.join("objects"))?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        match temporary.persist_noclobber(&destination) {
            Ok(_) => (),
            Err(error) if destination.exists() => {
                ensure!(
                    digest(&fs::read(&destination)?) == hash,
                    "retained object integrity failure: {error}"
                );
            }
            Err(error) => return Err(error.into()),
        }
        Ok(hash)
    }

    pub fn object_path(&self, hash: &str) -> Result<PathBuf> {
        ensure!(
            hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid SHA-256"
        );
        Ok(self.root.join("objects").join(hash))
    }

    pub fn insert_batch(
        &mut self,
        records: &[Evidence],
        source: &str,
        observed_at: DateTime<Utc>,
        raw_sha256: &str,
    ) -> Result<usize> {
        ensure!(!source.is_empty(), "source must not be empty");
        self.verify_object(raw_sha256, &mut std::collections::BTreeSet::new())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut all = read_retained(&tx, None)?
            .into_iter()
            .map(|r| r.record)
            .collect::<Vec<_>>();
        all.extend_from_slice(records);
        validate_records(&all)?;
        validate_call_identities(&all)?;
        let mut inserted = 0;
        for record in records {
            let payload = serde_json::to_string(record)?;
            let hash = digest(payload.as_bytes());
            let existing: Option<String> = tx
                .query_row(
                    "SELECT sha256 FROM records WHERE id=?1",
                    [&record.id],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(existing) = existing {
                ensure!(
                    existing == hash,
                    "record {} already exists with different content; create a new version ID",
                    record.id
                );
            } else {
                tx.execute(
                    "INSERT INTO records VALUES(?1,?2,?3,?4)",
                    params![record.id, record.kind, payload, hash],
                )?;
                inserted += 1;
            }
            tx.execute(
                "INSERT OR IGNORE INTO observations VALUES(?1,?2,?3,?4)",
                params![record.id, observed_at.to_rfc3339(), source, raw_sha256],
            )?;
        }
        tx.commit()?;
        Ok(inserted)
    }

    pub fn records(&self, as_of: Option<DateTime<Utc>>) -> Result<Vec<Evidence>> {
        Ok(self
            .retained_records(as_of)?
            .into_iter()
            .map(|r| r.record)
            .collect())
    }

    pub fn retained_records(&self, as_of: Option<DateTime<Utc>>) -> Result<Vec<RetainedRecord>> {
        read_retained(&self.connection, as_of)
    }

    pub fn verify(&self) -> Result<usize> {
        let mut statement = self
            .connection
            .prepare("SELECT payload,sha256 FROM records")?;
        let mut count = 0;
        for row in
            statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        {
            let (payload, expected) = row?;
            ensure!(
                digest(payload.as_bytes()) == expected,
                "record integrity failure"
            );
            count += 1;
        }
        let mut statement = self
            .connection
            .prepare("SELECT DISTINCT raw_sha256 FROM observations")?;
        let mut visited = std::collections::BTreeSet::new();
        for hash in statement.query_map([], |r| r.get::<_, String>(0))? {
            self.verify_object(&hash?, &mut visited)?;
        }
        let integrity: String = self
            .connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        if integrity != "ok" {
            bail!("SQLite integrity check: {integrity}");
        }
        let records = self.records(None)?;
        validate_records(&records)?;
        validate_call_identities(&records)?;
        Ok(count)
    }

    fn verify_object(
        &self,
        hash: &str,
        visited: &mut std::collections::BTreeSet<String>,
    ) -> Result<()> {
        if !visited.insert(hash.to_owned()) {
            return Ok(());
        }
        let bytes = fs::read(self.object_path(hash)?).context("missing retained evidence")?;
        ensure!(digest(&bytes) == hash, "source evidence integrity failure");
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && matches!(
                value.get("schema").and_then(|v| v.as_str()),
                Some("observatory.collection/1" | "observatory.import/1")
            )
        {
            let children = value
                .get("raw_objects")
                .and_then(|v| v.as_array())
                .context("raw object manifest is incomplete")?;
            for child in children {
                self.verify_object(
                    child.as_str().context("invalid raw object reference")?,
                    visited,
                )?;
            }
        }
        Ok(())
    }
}

fn read_retained(
    connection: &Connection,
    as_of: Option<DateTime<Utc>>,
) -> Result<Vec<RetainedRecord>> {
    // Select the latest receipt available at the cut; every earlier receipt remains in SQLite.
    let mut statement = connection.prepare("SELECT r.payload,o.observed_at,o.source,o.raw_sha256 FROM records r JOIN observations o ON r.id=o.record_id ORDER BY r.id,o.observed_at DESC,o.source,o.raw_sha256")?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut result = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let (payload, time, source, raw_sha256) = row?;
        let observed_at: DateTime<Utc> = time.parse()?;
        if as_of.is_some_and(|cut| observed_at > cut) {
            continue;
        }
        let record: Evidence = serde_json::from_str(&payload)?;
        if seen.insert(record.id.clone()) {
            result.push(RetainedRecord {
                record,
                observed_at,
                source,
                raw_sha256,
            });
        }
    }
    Ok(result)
}

fn validate_call_identities(records: &[Evidence]) -> Result<()> {
    let mut calls = std::collections::BTreeMap::<(&str, &str), &serde_json::Value>::new();
    for record in records {
        if let Some(id) = record
            .data
            .get("canonical_call_id")
            .and_then(|v| v.as_str())
        {
            for field in [
                "/core_call_sha256",
                "/usage/input_tokens",
                "/usage/output_tokens",
                "/provider/observed_model",
            ] {
                if let Some(value) = record.data.pointer(field).filter(|v| !v.is_null())
                    && let Some(prior) = calls.insert((id, field), value)
                {
                    ensure!(
                        prior == value,
                        "conflicting representations of logical call {id} at {field}"
                    );
                }
            }
        }
    }
    Ok(())
}
