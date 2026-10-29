//! Read-only EAL runtime storage snapshots and explicit record exports.
use super::{ImportRecord, digest, read_bounded, required_str, valid_id};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags, backup::Backup};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path, time::Duration};

const MAX_RECORD_BYTES: usize = 64 * 1024 * 1024;

/// Make a consistent SQLite snapshot with SQLite's online backup API. The source
/// connection is read-only; WAL content participates in the snapshot. Destination
/// must be a fresh path owned by the caller. Store/hash these bytes as raw input.
pub fn snapshot_eal_sqlite(source: &Path, destination: &Path) -> Result<()> {
    ensure!(!destination.exists(), "snapshot destination must not exist");
    let input = Connection::open_with_flags(
        source,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    input.busy_timeout(Duration::from_secs(5))?;
    // Pin one read snapshot before stepping the backup. A continuously writing
    // producer otherwise can repeatedly restart a multi-step backup.
    input.execute_batch("BEGIN")?;
    input.query_row("PRAGMA schema_version", [], |row| row.get::<_, i64>(0))?;
    let mut output = Connection::open(destination)?;
    let backup = Backup::new(&input, &mut output)?;
    backup.run_to_completion(128, Duration::from_millis(10), None)?;
    drop(backup);
    output.close().map_err(|(_, error)| error)?;
    Ok(())
}

struct EalRow<'a> {
    id: &'a str,
    kind: &'a str,
    created_at: &'a str,
    payload: &'a Value,
    payload_hash: &'a str,
}

fn eal_record(
    namespace: &str,
    schema: &str,
    snapshot_hash: &str,
    row: EalRow<'_>,
) -> Result<ImportRecord> {
    let EalRow {
        id,
        kind,
        created_at,
        payload,
        payload_hash,
    } = row;
    ensure!(
        valid_id(id) && valid_id(kind),
        "invalid EAL record identity/kind"
    );
    let mut data = json!({"source_type":"eal","source_schema":schema,"source_revision":null,
        "purpose":"product_execution","record_type":"runtime_record","canonical_call_id":null,
        "namespace":namespace,"source_record_id":id,"runtime_kind":kind,"created_at":created_at,
        "snapshot_sha256":snapshot_hash,"payload_sha256":payload_hash,
        "payload_schema":payload.get("schema").or_else(||payload.get("schema_version")).and_then(Value::as_str)});
    for key in [
        "status",
        "complete",
        "valid",
        "claim_status",
        "evidence_id",
        "environment",
        "request_digest",
        "tool_binding_digest",
        "source_digest",
    ] {
        if let Some(value) = payload
            .get(key)
            .filter(|v| v.is_string() || v.is_boolean() || v.is_null())
        {
            data.as_object_mut()
                .unwrap()
                .insert(key.into(), value.clone());
        }
    }
    let observed = digest(serde_json::to_vec(&data)?);
    Ok(ImportRecord {
        id: format!("eal:{observed}"),
        kind: "source_record".into(),
        data,
    })
}

/// Import the real EAL runtime records table or an explicit records-export JSON
/// envelope. Payloads remain private raw artifacts; emitted rows retain hashes
/// and selected provenance/status fields, never command arguments or evidence text.
pub fn eal_import(path: &Path, namespace: &str) -> Result<Vec<ImportRecord>> {
    ensure!(
        valid_id(namespace),
        "EAL namespace must be a bounded stable identifier"
    );
    let mut header = [0_u8; 16];
    use std::io::Read;
    let n = fs::File::open(path)?.read(&mut header)?;
    if n == 16 && &header == b"SQLite format 3\0" {
        let directory = tempfile::tempdir()?;
        let snapshot = directory.path().join("records.sqlite");
        snapshot_eal_sqlite(path, &snapshot)?;
        let snapshot_hash = digest(read_bounded(&snapshot)?);
        let db = Connection::open_with_flags(&snapshot, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut columns = db.prepare("PRAGMA table_info(records)")?;
        let names = columns
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<BTreeSet<_>>>()?;
        ensure!(
            ["id", "kind", "created_at", "payload"]
                .iter()
                .all(|name| names.contains(*name)),
            "unsupported EAL SQLite records schema"
        );
        let mut stmt=db.prepare("SELECT id,kind,created_at,CASE WHEN length(CAST(payload AS BLOB)) <= 67108864 THEN payload ELSE NULL END FROM records ORDER BY id")?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let kind: String = row.get(1)?;
            let created: String = row.get(2)?;
            let encoded: Option<String> = row.get(3)?;
            let encoded = encoded.context("EAL payload exceeds 64 MiB record limit")?;
            let payload: Value =
                serde_json::from_str(&encoded).context("invalid EAL stored JSON payload")?;
            out.push(eal_record(
                namespace,
                "eal.records-sqlite/1",
                &snapshot_hash,
                EalRow {
                    id: &id,
                    kind: &kind,
                    created_at: &created,
                    payload: &payload,
                    payload_hash: &digest(encoded.as_bytes()),
                },
            )?);
        }
        return Ok(out);
    }
    let bytes = read_bounded(path)?;
    let snapshot_hash = digest(&bytes);
    let root: Value = serde_json::from_slice(&bytes).context("invalid EAL records export")?;
    ensure!(
        root.get("schema").and_then(Value::as_str) == Some("eal.records-export/1"),
        "unsupported EAL export schema; expected eal.records-export/1"
    );
    let records = root
        .get("records")
        .and_then(Value::as_array)
        .context("EAL export lacks records array")?;
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for row in records {
        let id = required_str(row, "id")?;
        ensure!(seen.insert(id), "duplicate EAL export record ID");
        let payload = row
            .get("payload")
            .context("EAL export record lacks payload")?;
        let encoded = serde_json::to_vec(payload)?;
        ensure!(
            encoded.len() <= MAX_RECORD_BYTES,
            "EAL export payload exceeds record limit"
        );
        out.push(eal_record(
            namespace,
            "eal.records-export/1",
            &snapshot_hash,
            EalRow {
                id,
                kind: required_str(row, "kind")?,
                created_at: required_str(row, "created_at")?,
                payload,
                payload_hash: &digest(encoded),
            },
        )?);
    }
    Ok(out)
}
