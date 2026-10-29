//! Synthetic fixtures exercise the producer's retained formats, never invoke an assistant.
use observatory::importers::{eal_import, engarg_import, snapshot_eal_sqlite};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

fn hash(s: &str) -> String {
    Sha256::digest(s.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn save(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn manifest(kind: &str) -> Value {
    json!({"schema":"observatory.engarg-import/1","provenance_chain_id":"chain-a","session_id":"session-a","attempt_id":"attempt-a","source_revision":"e8b21c929f2aa9e8b870131e534e354926354094","artifact_kind":kind})
}
fn config(provider: Option<&str>) -> Value {
    let mut value = json!({"api":"Responses","endpoint":"https://example.invalid/v1/responses","default_model":"selected","role_models":{"single":"role-selected"},"requested_reasoning_effort":null});
    if let Some(provider) = provider {
        value["api"] = json!("AssistantCommand");
        value["assistant"] = json!({"provider":provider,"configuration_sha256":null});
    }
    value
}
fn call(model: &str, input: Value) -> Value {
    json!({"role":"single","prompt":{"role":"single","system":"PRIVATE PROMPT","user":"SECRET_SOURCE_TEXT","schema":{}},"response":{"content":"{}","model":model,"input_tokens":input,"output_tokens":0,"request_id":null,"service_tier":null,"reasoning_effort":null},"response_content_bytes":2,"response_content_sha256":hash("{}"),"response_content_omitted":false,"elapsed_ms":42})
}
fn report(provider: Option<&str>, model: &str, input: Value) -> Value {
    json!({"schema_version":"engarg.run.v3","provider_config":config(provider),"transcript":[call(model,input)]})
}
fn imported(
    value: &Value,
    kind: &str,
    m: Option<Value>,
) -> Vec<observatory::importers::ImportRecord> {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("source.json");
    let mp = dir.path().join("manifest.json");
    save(&p, value);
    save(&mp, &m.unwrap_or_else(|| manifest(kind)));
    engarg_import(&p, &mp).unwrap()
}
fn model_calls(records: &[observatory::importers::ImportRecord]) -> Vec<&Value> {
    records
        .iter()
        .filter(|r| r.data["record_type"] == "model_call")
        .map(|r| &r.data)
        .collect()
}

#[test]
fn historical_http_unknown_usage_and_zero_are_distinct_and_private_text_is_omitted() {
    let records = imported(&report(None, "selected", Value::Null), "report", None);
    let call = model_calls(&records)[0];
    assert_eq!(call["usage"]["input_tokens"], Value::Null);
    assert_eq!(call["usage"]["input_tokens_status"], "unknown");
    assert_eq!(call["usage"]["output_tokens"], 0);
    assert_eq!(call["provider"]["configured_model"], "role-selected");
    assert_eq!(call["provider"]["observed_model"], Value::Null);
    assert_eq!(call["purpose"], "product_execution");
    let encoded = serde_json::to_string(&records).unwrap();
    for secret in [
        "PRIVATE PROMPT",
        "SECRET_SOURCE_TEXT",
        "https://example.invalid",
    ] {
        assert!(!encoded.contains(secret));
    }
}

#[test]
fn explicit_synthetic_execution_class_is_preserved_without_model_label_inference() {
    let mut fixture = report(
        Some("cursor"),
        "assistant:cursor:configured-model",
        Value::Null,
    );
    fixture["execution_mode"] = json!("synthetic_adapter_fixture");
    let records = imported(&fixture, "report", None);
    assert!(records.iter().all(|r| r.data["synthetic"] == true));
    assert!(
        records
            .iter()
            .all(|r| r.data["execution_mode"] == "synthetic_adapter_fixture")
    );
    assert!(
        records
            .iter()
            .all(|r| r.data["evidence_class"] == "synthetic")
    );
    fixture["execution_mode"] = json!("provider_calls");
    let records = imported(&fixture, "report", None);
    assert!(records.iter().all(|r| r.data["synthetic"].is_null()));
    assert!(
        records
            .iter()
            .all(|r| r.data["evidence_class"] == "unspecified")
    );
    fixture["evidence_class"] = json!("synthetic");
    let records = imported(&fixture, "report", None);
    assert!(records.iter().all(|r| r.data["synthetic"] == true));
    assert!(
        records
            .iter()
            .all(|r| r.data["source_evidence_class"] == "synthetic")
    );
}

#[test]
fn native_and_generic_assistant_identities_follow_actual_contract() {
    for (provider, label, basis, observed) in [
        (
            "cursor",
            "assistant:cursor:selected",
            "configured_selection",
            Value::Null,
        ),
        (
            "augment",
            "assistant:augment:selected",
            "configured_selection",
            Value::Null,
        ),
        (
            "cursor",
            "actual-model",
            "adapter_reported",
            json!("actual-model"),
        ),
        (
            "command",
            "adapter-model",
            "adapter_reported",
            json!("adapter-model"),
        ),
    ] {
        let records = imported(
            &report(
                Some(provider),
                label,
                if provider == "command" {
                    json!(7)
                } else {
                    Value::Null
                },
            ),
            "report",
            None,
        );
        let call = model_calls(&records)[0];
        assert_eq!(call["provider"]["kind"], provider);
        assert_eq!(call["provider"]["model_identity_basis"], basis);
        assert_eq!(call["provider"]["observed_model"], observed);
    }
}

#[test]
fn identical_completions_are_distinct_calls_but_representations_share_identity() {
    let c = call("m", json!(2));
    let r = json!({"schema_version":"engarg.run.v3","provider_config":config(None),"transcript":[c.clone(),c.clone()]});
    let records = imported(&r, "report", None);
    let calls = model_calls(&records);
    assert_ne!(calls[0]["canonical_call_id"], calls[1]["canonical_call_id"]);
    assert_eq!(
        calls[0]["call_record_sha256"],
        calls[1]["call_record_sha256"]
    );
    let checkpoint = json!({"schema":"engarg.authoring-checkpoint/1","provider_config":config(None),"transcript":[c.clone(),c]});
    let mut m = manifest("checkpoint");
    m["call_ordinals"] = json!([1, 0]);
    let other = imported(&checkpoint, "checkpoint", Some(m));
    let other = model_calls(&other);
    assert_eq!(other[0]["canonical_call_id"], calls[1]["canonical_call_id"]);
    assert_ne!(other[0]["source_schema"], calls[1]["source_schema"]);
}

#[test]
fn optional_metadata_enrichment_keeps_core_identity_and_missing_null_normalize() {
    let absent = report(None, "selected", Value::Null);
    let mut explicit = absent.clone();
    explicit["transcript"][0]["response"]
        .as_object_mut()
        .unwrap()
        .remove("service_tier");
    explicit["transcript"][0]["response"]
        .as_object_mut()
        .unwrap()
        .remove("reasoning_effort");
    let absent_records = imported(&absent, "report", None);
    let explicit_records = imported(&explicit, "report", None);
    let a = model_calls(&absent_records)[0];
    let b = model_calls(&explicit_records)[0];
    assert_eq!(a["canonical_call_id"], b["canonical_call_id"]);
    assert_eq!(a["core_call_sha256"], b["core_call_sha256"]);
    assert_eq!(a["canonical_fact_sha256"], b["canonical_fact_sha256"]);
    assert_ne!(a["source_record_sha256"], b["source_record_sha256"]);
    let mut enriched = absent.clone();
    enriched["transcript"][0]["response"]["input_tokens"] = json!(7);
    let enriched_records = imported(&enriched, "report", None);
    let c = model_calls(&enriched_records)[0];
    assert_eq!(a["core_call_sha256"], c["core_call_sha256"]);
    assert_eq!(a["canonical_call_id"], c["canonical_call_id"]);
    assert_ne!(a["canonical_fact_sha256"], c["canonical_fact_sha256"]);
}

#[test]
fn audit_failure_preserves_completed_call_without_promoting_run_success() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let mp = dir.path().join("manifest.json");
    let mut start = config(Some("augment"));
    start["event"] = json!("start");
    let lines = [
        start,
        json!({"event":"completion","record":call("assistant:augment:selected",Value::Null)}),
        json!({"event":"failure","error":"PRIVATE_ENDPOINT_AND_ERROR"}),
    ];
    fs::write(
        &path,
        lines
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n{\"event\":",
    )
    .unwrap();
    save(&mp, &manifest("audit"));
    let records = engarg_import(&path, &mp).unwrap();
    assert_eq!(model_calls(&records).len(), 1);
    assert!(records.iter().any(|r| r.data["event"] == "failure"));
    assert!(
        records
            .iter()
            .any(|r| r.data["record_type"] == "artifact" && r.data["complete"] == false)
    );
    assert!(
        !serde_json::to_string(&records)
            .unwrap()
            .contains("PRIVATE_ENDPOINT_AND_ERROR")
    );
}

#[test]
fn native_failure_without_completion_does_not_invent_tokens_or_calls() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let mp = dir.path().join("manifest.json");
    let mut start = config(Some("cursor"));
    start["event"] = json!("start");
    fs::write(
        &path,
        format!(
            "{}\n{}\n",
            start,
            json!({"event":"failure","error":"timeout"})
        ),
    )
    .unwrap();
    save(&mp, &manifest("audit"));
    let records = engarg_import(&path, &mp).unwrap();
    assert!(model_calls(&records).is_empty());
    assert_eq!(
        records.last().unwrap().data["attempted_logical_calls"],
        Value::Null
    );
}

#[test]
fn resumed_report_reuses_parent_ordinal_and_new_provider_is_scoped_to_suffix() {
    let mut r = report(None, "m", json!(2));
    r["transcript"]
        .as_array_mut()
        .unwrap()
        .push(call("assistant:cursor:new", Value::Null));
    r["authoring_recovery"] = json!({"schema":"engarg.authoring-recovery/1","recorded_calls":1,"new_model_calls":1,"previous_call_order":[0],"parent_checkpoint_sha256":"a".repeat(64),"provider_config":config(Some("cursor")),"previous_provider_config":config(None)});
    let mut m = manifest("report");
    m["attempt_id"] = json!("attempt-b");
    m["parent_attempt_id"] = json!("attempt-a");
    m["parent_checkpoint_sha256"] = json!("a".repeat(64));
    let parent = imported(&report(None, "m", json!(2)), "report", None);
    let resumed = imported(&r, "report", Some(m));
    let calls = model_calls(&resumed);
    assert_eq!(
        calls[0]["canonical_call_id"],
        model_calls(&parent)[0]["canonical_call_id"]
    );
    assert_eq!(
        calls[0]["provider"]["model_identity_basis"],
        "historical_provider_unresolved"
    );
    assert_eq!(calls[1]["provider"]["kind"], "cursor");
    assert_ne!(calls[1]["canonical_call_id"], calls[0]["canonical_call_id"]);
}

#[test]
fn unknown_schemas_bad_counts_and_ambiguous_order_are_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("source.json");
    let mp = dir.path().join("manifest.json");
    save(&mp, &manifest("report"));
    save(
        &path,
        &json!({"schema_version":"engarg.run.v999","transcript":[]}),
    );
    assert!(engarg_import(&path, &mp).is_err());
    let mut r = report(None, "m", Value::Null);
    r["transcript"][0]["response_content_sha256"] = json!("a".repeat(64));
    save(&path, &r);
    assert!(engarg_import(&path, &mp).is_err());
    let checkpoint = json!({"schema":"engarg.authoring-checkpoint/1","provider_config":config(None),"transcript":[call("m",Value::Null),call("m",Value::Null)]});
    save(&path, &checkpoint);
    save(&mp, &manifest("checkpoint"));
    assert!(engarg_import(&path, &mp).is_err());
    let mut r = report(None, "m", Value::Null);
    r["authoring_recovery"] =
        json!({"schema":"engarg.authoring-recovery/1","recorded_calls":2,"new_model_calls":2});
    save(&path, &r);
    save(&mp, &manifest("report"));
    assert!(engarg_import(&path, &mp).is_err());
    let mut m = manifest("report");
    m["purpose"] = json!("development_exposure");
    save(&mp, &m);
    assert!(engarg_import(&path, &mp).is_err());
}

#[test]
fn eal_sqlite_online_backup_includes_wal_and_never_exports_private_payload() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("run.sqlite");
    let snap = dir.path().join("snapshot.sqlite");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; CREATE TABLE records(id TEXT PRIMARY KEY,kind TEXT NOT NULL,created_at TEXT NOT NULL,payload TEXT NOT NULL);").unwrap();
    db.execute("INSERT INTO records VALUES (?1,?2,?3,?4)",rusqlite::params!["observation-a","observation","2026-10-29T10:00:00Z",json!({"status":"ok","evidence_id":"need-a","value":"PRIVATE_EVIDENCE","command":["SECRET"]}).to_string()]).unwrap();
    snapshot_eal_sqlite(&path, &snap).unwrap();
    assert!(snapshot_eal_sqlite(&path, &snap).is_err());
    let records = eal_import(&snap, "store-a").unwrap();
    assert_eq!(records.len(), 1);
    let snapshot_hash: String = Sha256::digest(fs::read(&snap).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(records[0].data["snapshot_sha256"], snapshot_hash);
    assert_eq!(records[0].data["status"], "ok");
    assert_eq!(records[0].data["source_record_id"], "observation-a");
    assert!(
        !serde_json::to_string(&records)
            .unwrap()
            .contains("PRIVATE_EVIDENCE")
    );
    assert!(!serde_json::to_string(&records).unwrap().contains("SECRET"));
    assert_eq!(
        db.query_row("SELECT count(*) FROM records", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn explicit_eal_json_export_accepts_negative_outcome_and_rejects_unknown_schema() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("export.json");
    save(
        &p,
        &json!({"schema":"eal.records-export/1","records":[{"id":"assessment-a","kind":"assessment","created_at":"2026-10-29T00:00:00Z","payload":{"complete":true,"claim_status":"unsupported"}}]}),
    );
    let records = eal_import(&p, "store-a").unwrap();
    assert_eq!(records[0].data["claim_status"], "unsupported");
    assert_eq!(records[0].data["complete"], true);
    save(&p, &json!({"schema":"unknown","records":[]}));
    assert!(eal_import(&p, "store-a").is_err());
}
