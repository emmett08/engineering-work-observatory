//! Receipt lifecycle and hostile input checks; fixtures do not execute an assistant.
use chrono::Utc;
use observatory::{
    importers::executions_import,
    model::Evidence,
    store::{Store, digest},
};
use serde_json::{Value, json};
use std::io::{Cursor, Write};

fn start() -> Value {
    serde_json::from_str(include_str!("fixtures/execution-start.json")).unwrap()
}
fn completion() -> Value {
    let mut r = start();
    r["phase"] = json!("completion");
    r["timestamp"] = json!("2026-11-01T12:00:01Z");
    r["result"] = json!({"status":"success","exit_code":0,"runtime_ms":1000,"observed_model":null,"observed_session_id":null,"input_tokens":null,"output_tokens":null,"cached_input_tokens":null,"charge_amount":null,"charge_currency":null,"usage_source":"unknown","checks":[]});
    r
}
fn bundle(receipts: &[Value]) -> Vec<u8> {
    let entries: Vec<_> = receipts
        .iter()
        .map(|r| {
            let receipt_json = serde_json::to_string(r).unwrap();
            json!({"sha256":digest(receipt_json.as_bytes()),"receipt_json":receipt_json})
        })
        .collect();
    serde_json::to_vec(&json!({"schema":"observatory.execution-bundle/1","platform":receipts.first().unwrap_or(&start())["platform"],"receipts":entries})).unwrap()
}
fn import(store: &mut Store, bytes: &[u8]) -> anyhow::Result<usize> {
    let records = executions_import(bytes)?
        .into_iter()
        .map(|r| Evidence {
            id: r.id,
            kind: r.kind,
            data: r.data,
        })
        .collect::<Vec<_>>();
    let raw = store.retain_bytes(bytes)?;
    let manifest = store.retain_bytes(&serde_json::to_vec(
        &json!({"schema":"observatory.import/1","raw_objects":[raw]}),
    )?)?;
    store.insert_batch(&records, "execution-test", Utc::now(), &manifest)
}
fn zip(name: &str, bytes: &[u8]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(name, zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(bytes).unwrap();
    writer.finish().unwrap().into_inner()
}
#[test]
fn imports_separate_immutable_phases_and_deduplicates_repeated_delivery() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let begin = bundle(&[start()]);
    let end = bundle(&[completion()]);
    assert_eq!(import(&mut store, &end).unwrap(), 2); // completion can arrive first
    assert_eq!(import(&mut store, &begin).unwrap(), 2);
    assert_eq!(import(&mut store, &end).unwrap(), 0);
    let combined = bundle(&[start(), completion()]);
    assert_eq!(import(&mut store, &combined).unwrap(), 1); // new bundle, same logical receipts
    assert_eq!(store.verify().unwrap(), 5);
    let records = store.records(None).unwrap();
    assert_eq!(
        records
            .iter()
            .filter(|r| r.data["source"] == "observatory.tool_receipt")
            .count(),
        2
    );
    let r = records.iter().find(|r| r.id.ends_with("/start")).unwrap();
    assert_eq!(r.data["payload"]["platform"]["actor"], "credential-owner");
    assert_eq!(r.data["payload"]["declared"]["controller"], "assistant");
    assert!(records.iter().all(|r| r.kind == "source_record"));
}
#[test]
fn cross_delivery_identity_conflicts_are_atomic_and_do_not_replace_original() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    import(&mut store, &bundle(&[start()])).unwrap();
    for pointer in [
        "/declared/controller",
        "/declared/session_id",
        "/tool/version",
        "/platform/triggering_actor",
        "/platform/job_name",
    ] {
        let mut r = completion();
        *r.pointer_mut(pointer).unwrap() = json!(if pointer == "/declared/controller" {
            "human"
        } else {
            "different"
        });
        assert!(import(&mut store, &bundle(&[r])).is_err(), "{pointer}");
        assert_eq!(store.records(None).unwrap().len(), 2);
    }
    let mut changed = start();
    changed["timestamp"] = json!("2026-11-01T12:00:00.1Z");
    assert!(import(&mut store, &bundle(&[changed])).is_err());
    assert_eq!(store.verify().unwrap(), 2);
}
#[test]
fn reruns_and_tool_retries_have_distinct_identity_and_matrix_dimensions_are_retained() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let a = start();
    let mut b = start();
    b["platform"]["run_attempt"] = json!(2);
    let mut retry = start();
    retry["invocation_id"] = json!("retry-b");
    retry["declared"]["parent_invocation_id"] = json!("invocation-a");
    let mut matrix = start();
    matrix["invocation_id"] = json!("matrix-c");
    matrix["platform"]["matrix"]["os"] = json!("macos");
    matrix["platform"]["job_name"] = json!("test (macos)");
    for r in [a, b, retry, matrix] {
        assert_eq!(import(&mut store, &bundle(&[r])).unwrap(), 2);
    }
    assert_eq!(store.verify().unwrap(), 8);
}
#[test]
fn start_only_cancelled_and_empty_bundles_do_not_infer_completion_or_labour() {
    let records = executions_import(&bundle(&[start()])).unwrap();
    assert_eq!(records.len(), 2);
    assert!(records[1].data["payload"]["result"].is_null());
    let empty = executions_import(&bundle(&[])).unwrap();
    assert_eq!(empty.len(), 1);
    assert_eq!(empty[0].data["payload"]["receipt_count"], 0);
    assert_eq!(empty[0].data["payload"]["platform"]["job_key"], "test");
    let mut cancelled = completion();
    cancelled["result"]["status"] = json!("cancelled");
    cancelled["result"]["exit_code"] = json!(-15);
    assert!(executions_import(&bundle(&[start(), cancelled])).is_ok());
}
#[test]
fn rejects_bad_hashes_schema_metadata_and_result_contradictions() {
    let mut bad: Value = serde_json::from_slice(&bundle(&[start()])).unwrap();
    bad["receipts"][0]["sha256"] = json!("0".repeat(64));
    assert!(executions_import(&serde_json::to_vec(&bad).unwrap()).is_err());
    for (pointer, value) in [
        ("/schema", json!("observatory.tool-receipt/999")),
        ("/declared/controller", json!("github-actor")),
        ("/platform/run_attempt", json!(0)),
        ("/result/status", json!("skipped")),
        ("/result/exit_code", json!(1)),
        ("/result/input_tokens", json!(-1)),
        ("/result/charge_amount", json!(0.01)),
        ("/result/observed_model", json!("unattributed-model")),
        ("/timestamp", json!("yesterday")),
    ] {
        let mut r = completion();
        *r.pointer_mut(pointer).unwrap() = value;
        assert!(executions_import(&bundle(&[r])).is_err(), "{pointer}");
    }
    let mut r = start();
    r["environment"] = json!({"TOKEN":"secret"});
    assert!(executions_import(&bundle(&[r])).is_err());
    let mut r = start();
    r["tool"].as_object_mut().unwrap().remove("version");
    assert!(executions_import(&bundle(&[r])).is_err());
    let mut r = start();
    r["platform"]["matrix"]["env"] = json!({"secret":"value"});
    assert!(executions_import(&bundle(&[r])).is_err());
    let mut r = completion();
    r["timestamp"] = json!("2026-11-01T11:59:59Z");
    assert!(executions_import(&bundle(&[start(), r])).is_err());
}
#[test]
fn validates_observed_assistant_usage_without_promoting_configured_identity() {
    let mut r = completion();
    r["tool"]["adapter"] = json!("codex_exec");
    r["tool"]["configured_model"] = json!("selected-model");
    r["result"]["usage_source"] = json!("tool_reported");
    r["result"]["observed_session_id"] = json!("thread-observed");
    r["result"]["input_tokens"] = json!(10);
    r["result"]["output_tokens"] = json!(0);
    r["result"]["cached_input_tokens"] = json!(5);
    let records = executions_import(&bundle(&[r])).unwrap();
    assert_eq!(records[1].data["payload"]["result"]["output_tokens"], 0);
    assert!(records[1].data["payload"]["result"]["observed_model"].is_null());
}
#[test]
fn validates_zip_member_and_preserves_same_bundle_identity() {
    let bytes = bundle(&[start()]);
    let json = executions_import(&bytes).unwrap();
    let archived = executions_import(&zip("bundle.json", &bytes)).unwrap();
    assert_eq!(json[0].id, archived[0].id);
    assert!(executions_import(&zip("../bundle.json", &bytes)).is_err());
    assert!(executions_import(&zip("stdout.log", b"secret log")).is_err());
    assert!(executions_import(b"PK invalid").is_err());
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for name in ["bundle.json", "environment.txt"] {
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    assert!(executions_import(&writer.finish().unwrap().into_inner()).is_err());
}
#[test]
fn rejects_bundle_platform_mismatch_and_duplicate_phase_conflicts() {
    let mut other = start();
    other["platform"]["job_name"] = json!("other job");
    assert!(executions_import(&bundle(&[start(), other])).is_err());
    let mut changed = start();
    changed["timestamp"] = json!("2026-11-01T12:00:00.001Z");
    assert!(executions_import(&bundle(&[start(), changed])).is_err());
}

#[test]
fn rejects_duplicate_json_keys_even_inside_matrix_and_bundle_platform() {
    let raw = serde_json::to_string(&start())
        .unwrap()
        .replace("\"os\":\"linux\"", "\"os\":\"linux\",\"os\":\"macos\"");
    let bytes=serde_json::to_vec(&json!({"schema":"observatory.execution-bundle/1","platform":start()["platform"],"receipts":[{"sha256":digest(raw.as_bytes()),"receipt_json":raw}]})).unwrap();
    assert!(
        executions_import(&bytes)
            .unwrap_err()
            .to_string()
            .contains("duplicate")
    );
    let bytes = String::from_utf8(bundle(&[]))
        .unwrap()
        .replace("\"os\":\"linux\"", "\"os\":\"linux\",\"os\":\"macos\"");
    assert!(
        executions_import(bytes.as_bytes())
            .unwrap_err()
            .to_string()
            .contains("duplicate")
    );
}

#[test]
fn rejects_oversized_expansion_symlink_and_invalid_artifact_labels() {
    let oversized = vec![b' '; 16 * 1024 * 1024 + 1];
    assert!(executions_import(&oversized).is_err());
    assert!(executions_import(&zip("bundle.json", &oversized)).is_err());
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .add_symlink(
            "bundle.json",
            "/etc/passwd",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    assert!(executions_import(&writer.finish().unwrap().into_inner()).is_err());
    let mut r = start();
    r["artifacts"] =
        json!([{"role":"input","name":"../../secret","sha256":"a".repeat(64),"size_bytes":1}]);
    assert!(executions_import(&bundle(&[r])).is_err());
}
