//! Bounded, immutable execution receipts. Validation establishes integrity, not provenance.
use super::{ImportRecord, digest};
use crate::model::Evidence;
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, FixedOffset};
use serde::{
    Deserialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

const MAX_BUNDLE: usize = 16 * 1024 * 1024;
const MAX_RECEIPT: usize = 256 * 1024;

// Reject duplicate keys before typed decoding, including within maps such as matrix.
// Different JSON parsers must not disagree about the evidence whose bytes were hashed.
struct UniqueValue(Value);
impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueValue;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("JSON without duplicate object keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Self::Value, E> {
                Ok(UniqueValue(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
                Ok(UniqueValue(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| UniqueValue(Value::Number(n)))
                    .ok_or_else(|| E::custom("nonfinite JSON number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(v.to_owned())))
            }
            fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(v)))
            }
            fn visit_unit<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
                Ok(UniqueValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(UniqueValue(value)) = seq.next_element()? {
                    values.push(value);
                }
                Ok(UniqueValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some((key, UniqueValue(value))) =
                    map.next_entry::<String, UniqueValue>()?
                {
                    if values.insert(key, value).is_some() {
                        return Err(de::Error::custom("duplicate JSON object key"));
                    }
                }
                Ok(UniqueValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}
fn parse_json(bytes: &[u8]) -> Result<Value> {
    Ok(serde_json::from_slice::<UniqueValue>(bytes)?.0)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bundle {
    schema: String,
    platform: Value,
    receipts: Vec<BundleEntry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleEntry {
    sha256: String,
    receipt_json: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: String,
    invocation_id: String,
    phase: String,
    timestamp: String,
    platform: Platform,
    declared: Declared,
    tool: Tool,
    result: Option<ExecutionResult>,
    artifacts: Vec<Artifact>,
    missing_fields: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Platform {
    source: String,
    job_context_source: String,
    repository_id: String,
    repository: String,
    workflow_ref: String,
    workflow_sha: String,
    run_id: String,
    run_attempt: Option<u64>,
    job_key: String,
    job_name: String,
    job_id: Option<String>,
    matrix: BTreeMap<String, Value>,
    actor: String,
    triggering_actor: String,
    event_name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declared {
    source: String,
    controller: String,
    controller_provenance: Vec<String>,
    protocol_id: Option<String>,
    task_id: Option<String>,
    session_id: Option<String>,
    logical_call_id: Option<String>,
    parent_invocation_id: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Tool {
    adapter: String,
    name: String,
    version: Option<String>,
    purpose: String,
    configured_model: Option<String>,
    configuration: Configuration,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    reasoning_effort: Option<String>,
    temperature: Option<f64>,
    seed: Option<i64>,
    max_output_tokens: Option<u64>,
    configuration_sha256: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionResult {
    status: String,
    exit_code: Option<i32>,
    runtime_ms: Option<u64>,
    observed_model: Option<String>,
    observed_session_id: Option<String>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cached_input_tokens: Option<u64>,
    charge_amount: Option<f64>,
    charge_currency: Option<String>,
    usage_source: String,
    checks: Vec<Check>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Check {
    name: String,
    status: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    role: String,
    name: String,
    sha256: String,
    size_bytes: u64,
}

fn text(value: &str, max: usize) -> Result<()> {
    ensure!(
        !value.is_empty() && value.chars().count() <= max && !value.chars().any(char::is_control),
        "empty, oversized or control-containing metadata string"
    );
    Ok(())
}
fn optional_text(value: &Option<String>, max: usize) -> Result<()> {
    if let Some(value) = value {
        text(value, max)?;
    }
    Ok(())
}
fn choice(value: &str, choices: &[&str]) -> Result<()> {
    ensure!(choices.contains(&value), "unsupported metadata enum value");
    Ok(())
}
fn hash(value: &str) -> Result<()> {
    ensure!(
        value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "invalid lowercase SHA-256"
    );
    Ok(())
}
fn safe_id(value: &str, max: usize) -> Result<()> {
    text(value, max)?;
    ensure!(
        value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)),
        "invalid identifier"
    );
    Ok(())
}
fn numeric_id(value: &str, unknown: bool) -> Result<()> {
    text(value, 32)?;
    ensure!(
        (unknown && value == "unknown")
            || (!value.starts_with('0') && value.bytes().all(|b| b.is_ascii_digit())),
        "invalid platform numeric identifier"
    );
    Ok(())
}
fn keys(value: &Value, expected: &[&str]) -> Result<()> {
    let object = value.as_object().context("expected metadata object")?;
    ensure!(
        object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key)),
        "missing or unexpected metadata fields; expected {}",
        expected.join(", ")
    );
    Ok(())
}
fn validate_platform_shape(value: &Value) -> Result<()> {
    keys(
        value,
        &[
            "source",
            "job_context_source",
            "repository_id",
            "repository",
            "workflow_ref",
            "workflow_sha",
            "run_id",
            "run_attempt",
            "job_key",
            "job_name",
            "job_id",
            "matrix",
            "actor",
            "triggering_actor",
            "event_name",
        ],
    )
}
fn validate_platform(p: &Platform) -> Result<()> {
    ensure!(
        p.source == "runner_environment",
        "unsupported platform source"
    );
    ensure!(
        p.job_context_source == "caller",
        "unsupported job context source"
    );
    numeric_id(&p.repository_id, true)?;
    numeric_id(&p.run_id, true)?;
    for v in [
        &p.repository,
        &p.job_key,
        &p.actor,
        &p.triggering_actor,
        &p.event_name,
    ] {
        text(v, 256)?;
    }
    text(&p.workflow_ref, 512)?;
    text(&p.job_name, 512)?;
    ensure!(
        p.workflow_sha == "unknown"
            || ([40, 64].contains(&p.workflow_sha.len())
                && p.workflow_sha.bytes().all(|b| b.is_ascii_hexdigit())),
        "invalid workflow revision"
    );
    ensure!(
        p.run_attempt.is_none_or(|v| v > 0),
        "run_attempt must be positive or null"
    );
    if let Some(v) = &p.job_id {
        numeric_id(v, false)?;
    }
    ensure!(p.matrix.len() <= 32, "too many matrix dimensions");
    for (key, v) in &p.matrix {
        safe_id(key, 64)?;
        ensure!(
            v.is_null() || v.is_boolean() || v.is_number() || v.is_string(),
            "matrix values must be scalar"
        );
        if let Some(v) = v.as_str() {
            ensure!(
                v.chars().count() <= 512 && !v.chars().any(char::is_control),
                "invalid matrix string"
            );
        }
    }
    Ok(())
}
fn validate_shape(value: &Value) -> Result<()> {
    keys(
        value,
        &[
            "schema",
            "invocation_id",
            "phase",
            "timestamp",
            "platform",
            "declared",
            "tool",
            "result",
            "artifacts",
            "missing_fields",
        ],
    )?;
    validate_platform_shape(&value["platform"])?;
    keys(
        &value["declared"],
        &[
            "source",
            "controller",
            "controller_provenance",
            "protocol_id",
            "task_id",
            "session_id",
            "logical_call_id",
            "parent_invocation_id",
        ],
    )?;
    keys(
        &value["tool"],
        &[
            "adapter",
            "name",
            "version",
            "purpose",
            "configured_model",
            "configuration",
        ],
    )?;
    keys(
        &value["tool"]["configuration"],
        &[
            "reasoning_effort",
            "temperature",
            "seed",
            "max_output_tokens",
            "configuration_sha256",
        ],
    )?;
    if !value["result"].is_null() {
        keys(
            &value["result"],
            &[
                "status",
                "exit_code",
                "runtime_ms",
                "observed_model",
                "observed_session_id",
                "input_tokens",
                "output_tokens",
                "cached_input_tokens",
                "charge_amount",
                "charge_currency",
                "usage_source",
                "checks",
            ],
        )?;
    }
    Ok(())
}
fn validate(receipt: &Receipt) -> Result<()> {
    ensure!(
        receipt.schema == "observatory.tool-receipt/1",
        "unsupported execution receipt schema"
    );
    safe_id(&receipt.invocation_id, 128)?;
    choice(&receipt.phase, &["start", "completion"])?;
    DateTime::<FixedOffset>::parse_from_rfc3339(&receipt.timestamp)
        .context("invalid receipt timestamp")?;
    validate_platform(&receipt.platform)?;
    let d = &receipt.declared;
    ensure!(d.source == "caller", "unsupported declaration source");
    choice(
        &d.controller,
        &["human", "assistant", "automation", "unknown"],
    )?;
    ensure!(
        d.controller_provenance.len() <= 32,
        "too many provenance references"
    );
    for v in &d.controller_provenance {
        text(v, 512)?;
    }
    for v in [
        &d.protocol_id,
        &d.task_id,
        &d.session_id,
        &d.logical_call_id,
        &d.parent_invocation_id,
    ] {
        optional_text(v, 256)?;
    }
    let t = &receipt.tool;
    choice(&t.adapter, &["command", "codex_exec"])?;
    text(&t.name, 256)?;
    optional_text(&t.version, 256)?;
    optional_text(&t.configured_model, 256)?;
    choice(&t.purpose, &["development", "product_execution", "unknown"])?;
    optional_text(&t.configuration.reasoning_effort, 64)?;
    ensure!(
        t.configuration
            .temperature
            .is_none_or(|v| v.is_finite() && v >= 0.0),
        "temperature must be nonnegative"
    );
    if let Some(v) = &t.configuration.configuration_sha256 {
        hash(v)?;
    }
    let _ = (t.configuration.seed, t.configuration.max_output_tokens);
    ensure!(
        (receipt.phase == "start") == receipt.result.is_none(),
        "start result must be null; completion result is required"
    );
    if let Some(r) = &receipt.result {
        choice(&r.status, &["success", "failure", "cancelled", "unknown"])?;
        if let Some(code) = r.exit_code {
            ensure!(
                !(r.status == "success" && code != 0 || r.status == "failure" && code == 0),
                "result status contradicts exit code"
            );
        }
        optional_text(&r.observed_model, 256)?;
        optional_text(&r.observed_session_id, 256)?;
        choice(&r.usage_source, &["tool_reported", "unknown"])?;
        ensure!(
            r.charge_amount.is_some() == r.charge_currency.is_some(),
            "charge amount and currency must be supplied together"
        );
        ensure!(
            r.charge_amount.is_none_or(|v| v.is_finite() && v >= 0.0),
            "charge must be nonnegative"
        );
        if let Some(v) = &r.charge_currency {
            ensure!(
                v.len() == 3 && v.bytes().all(|b| b.is_ascii_uppercase()),
                "invalid charge currency"
            );
        }
        if r.usage_source == "unknown" {
            ensure!(
                r.observed_model.is_none()
                    && r.observed_session_id.is_none()
                    && r.input_tokens.is_none()
                    && r.output_tokens.is_none()
                    && r.cached_input_tokens.is_none()
                    && r.charge_amount.is_none(),
                "reported usage/model/session requires tool_reported source"
            );
        }
        if let (Some(cached), Some(input)) = (r.cached_input_tokens, r.input_tokens) {
            ensure!(cached <= input, "cached input tokens exceed input tokens");
        }
        ensure!(r.checks.len() <= 1024, "too many checks");
        for c in &r.checks {
            text(&c.name, 256)?;
            choice(&c.status, &["passed", "failed", "skipped", "unknown"])?;
        }
        let _ = r.runtime_ms;
    }
    ensure!(
        receipt.artifacts.len() <= 1024,
        "too many artifact references"
    );
    for a in &receipt.artifacts {
        choice(&a.role, &["input", "output"])?;
        text(&a.name, 256)?;
        ensure!(
            !a.name.contains(['/', '\\']) && a.name != "." && a.name != "..",
            "artifact name must be a logical label, not a path"
        );
        hash(&a.sha256)?;
        let _ = a.size_bytes;
    }
    ensure!(
        receipt.missing_fields.len() <= 128,
        "too many missing fields"
    );
    let mut missing = std::collections::BTreeSet::new();
    for v in &receipt.missing_fields {
        text(v, 256)?;
        ensure!(
            v.as_bytes()[0].is_ascii_lowercase()
                && v.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.".contains(&b)),
            "invalid missing field path"
        );
        ensure!(missing.insert(v), "duplicate missing field");
    }
    Ok(())
}
fn record_id(receipt: &Receipt) -> String {
    format!(
        "execution/{}/{}/{}/{}/{}",
        receipt.platform.repository_id,
        receipt.platform.run_id,
        receipt
            .platform
            .run_attempt
            .map_or_else(|| "unknown".into(), |v| v.to_string()),
        receipt.invocation_id,
        receipt.phase
    )
}

/// Import a JSON bundle or a GitHub artifact ZIP containing only `bundle.json`.
/// Archive members are never extracted, and compressed/expanded input is bounded.
pub fn executions_import(bytes: &[u8]) -> Result<Vec<ImportRecord>> {
    ensure!(
        bytes.len() <= MAX_BUNDLE,
        "execution artifact exceeds 16 MiB limit"
    );
    let unzipped;
    let bytes = if bytes.starts_with(b"PK") {
        let mut archive =
            zip::ZipArchive::new(Cursor::new(bytes)).context("invalid execution artifact ZIP")?;
        ensure!(
            archive.len() == 1,
            "execution artifact ZIP must contain exactly bundle.json"
        );
        let mut file = archive.by_index(0)?;
        ensure!(
            file.name() == "bundle.json" && !file.is_dir() && !file.is_symlink(),
            "unexpected or unsafe execution artifact ZIP member"
        );
        ensure!(
            file.size() <= MAX_BUNDLE as u64,
            "expanded execution artifact exceeds limit"
        );
        let mut output = Vec::new();
        (&mut file)
            .take(MAX_BUNDLE as u64 + 1)
            .read_to_end(&mut output)?;
        ensure!(
            output.len() <= MAX_BUNDLE,
            "expanded execution artifact exceeds limit"
        );
        unzipped = output;
        unzipped.as_slice()
    } else {
        bytes
    };
    let bundle: Bundle =
        serde_json::from_value(parse_json(bytes)?).context("invalid execution bundle")?;
    ensure!(
        bundle.schema == "observatory.execution-bundle/1",
        "unsupported execution bundle schema"
    );
    ensure!(
        bundle.receipts.len() <= 10000,
        "too many execution receipts"
    );
    validate_platform_shape(&bundle.platform)?;
    validate_platform(&serde_json::from_value(bundle.platform.clone())?)?;
    let mut records = Vec::new();
    let bundle_sha256 = digest(bytes);
    records.push(ImportRecord {
        id:format!("execution-bundle/{bundle_sha256}"), kind:"source_record".into(),
        data:json!({"schema_version":1,"source":"observatory.execution_bundle","record_type":"execution_bundle","payload":{"platform":bundle.platform,"receipt_count":bundle.receipts.len(),"bundle_sha256":bundle_sha256}}),
    });
    for entry in bundle.receipts {
        ensure!(
            entry.receipt_json.len() <= MAX_RECEIPT,
            "execution receipt exceeds 256 KiB limit"
        );
        hash(&entry.sha256)?;
        ensure!(
            digest(entry.receipt_json.as_bytes()) == entry.sha256,
            "execution receipt SHA-256 mismatch"
        );
        let payload = parse_json(entry.receipt_json.as_bytes())?;
        let receipt: Receipt =
            serde_json::from_value(payload.clone()).context("invalid execution receipt")?;
        validate_shape(&payload)?;
        validate(&receipt)?;
        ensure!(
            payload["platform"] == bundle.platform,
            "receipt platform differs from bundle platform"
        );
        records.push(ImportRecord { id:record_id(&receipt), kind:"source_record".into(), data:json!({"schema_version":1,"source":"observatory.tool_receipt","record_type":"tool_execution_receipt","payload":payload}) });
    }
    let evidence: Vec<_> = records
        .iter()
        .map(|r| Evidence {
            id: r.id.clone(),
            kind: r.kind.clone(),
            data: r.data.clone(),
        })
        .collect();
    validate_execution_records(&evidence)?;
    Ok(records)
}

/// Enforce receipt invariants across separate deliveries inside the store transaction.
/// A missing start or completion remains incomplete evidence and is permitted.
pub fn validate_execution_records(records: &[Evidence]) -> Result<()> {
    let mut phases = BTreeMap::<String, &Value>::new();
    let mut identities = BTreeMap::<String, &Value>::new();
    let mut times = BTreeMap::<String, BTreeMap<String, DateTime<FixedOffset>>>::new();
    for record in records {
        if record.data["source"] != "observatory.tool_receipt" {
            continue;
        }
        ensure!(
            record.kind == "source_record",
            "tool receipt must be a source record"
        );
        let payload = &record.data["payload"];
        validate_shape(payload)?;
        let receipt: Receipt = serde_json::from_value(payload.clone())?;
        validate(&receipt)?;
        ensure!(
            record.id == record_id(&receipt),
            "incorrect immutable receipt record ID"
        );
        if let Some(prior) = phases.insert(record.id.clone(), payload) {
            ensure!(
                prior == payload,
                "conflicting receipt phase for {}",
                record.id
            );
        }
        let scope = record
            .id
            .rsplit_once('/')
            .context("invalid receipt ID")?
            .0
            .to_owned();
        if let Some(prior) = identities.insert(scope.clone(), payload) {
            for field in ["invocation_id", "platform", "declared", "tool"] {
                ensure!(
                    prior[field] == payload[field],
                    "conflicting invocation identity at {field}: {scope}"
                );
            }
        }
        times.entry(scope).or_default().insert(
            receipt.phase,
            DateTime::parse_from_rfc3339(&receipt.timestamp)?,
        );
    }
    for (scope, times) in times {
        if let (Some(start), Some(end)) = (times.get("start"), times.get("completion")) {
            ensure!(end >= start, "completion precedes start: {scope}");
        }
    }
    Ok(())
}
