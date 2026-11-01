use observatory::collectors::{CollectedRecord, Collection, Coverage, RawResponse};
use observatory::ingestion::{
    CollectionRequirements, persist_collection, persist_execution_collection,
};
use observatory::store::{Store, digest};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn coverage(source: &str, state: &str) -> Coverage {
    Coverage {
        source: source.into(),
        scope: format!("https://api.github.com/repos/acme/demo/{source}"),
        state: state.into(),
        pages: usize::from(state == "complete"),
        records: usize::from(source == "github.pull_request"),
        detail: (state != "complete").then(|| "synthetic endpoint result".into()),
    }
}

fn collection(scopes: Vec<Coverage>) -> Collection {
    let body = br#"[{"id":101,"number":1}]"#.to_vec();
    Collection {
        records: vec![CollectedRecord {
            id: "fixture/pr/1".into(),
            kind: "source_record".into(),
            data: json!({"source":"github.pull_request","repository":"github:repository:42","payload":{"id":101,"number":1}}),
        }],
        observations: vec![RawResponse {
            source: "github.pull_request".into(),
            url: "https://api.github.com/repos/acme/demo/pulls?state=all".into(),
            collected_at: "2026-01-10T12:00:00Z".into(),
            status: Some(200),
            headers: BTreeMap::new(),
            sha256: digest(&body),
            body,
            error: None,
        }],
        coverage: scopes,
    }
}

fn required() -> Vec<Coverage> {
    vec![
        coverage("github.repository", "complete"),
        coverage("github.pull_request", "complete"),
    ]
}

fn retained_manifest(store: &Store) -> Value {
    let retained = store.retained_records(None).unwrap();
    let receipt = retained
        .iter()
        .find(|r| r.record.id == "fixture/pr/1")
        .unwrap();
    serde_json::from_slice(&std::fs::read(store.object_path(&receipt.raw_sha256).unwrap()).unwrap())
        .unwrap()
}

#[test]
fn full_policy_preserves_legacy_omissions_and_not_applicable_scopes() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let mut scopes = required();
    scopes.extend([
        coverage("github.workflow_logs", "not_collected"),
        coverage("github.history", "not_collected"),
        coverage("git.graph", "not_applicable"),
    ]);
    let receipt = persist_collection(
        &mut store,
        collection(scopes),
        "github:acme/demo",
        CollectionRequirements::Full,
    )
    .unwrap();
    assert_eq!(receipt.new_records, 6);
    assert_eq!(store.verify().unwrap(), receipt.new_records);
    let manifest = retained_manifest(&store);
    assert_eq!(manifest["schema"], "observatory.collection/1");
    assert_eq!(manifest["source"], "github:acme/demo");
    assert_eq!(manifest["coverage"].as_array().unwrap().len(), 5);
    assert_eq!(
        digest(&std::fs::read(store.object_path(&receipt.manifest_sha256).unwrap()).unwrap()),
        receipt.manifest_sha256
    );
}

#[test]
fn full_policy_does_not_invent_throughput_requirements_for_local_collection() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let input = collection(vec![coverage("git.graph", "complete")]);
    let receipt = persist_collection(
        &mut store,
        input,
        "git:acme/demo",
        CollectionRequirements::Full,
    )
    .unwrap();
    assert_eq!(receipt.new_records, 2);
    store.verify().unwrap();
}

#[test]
fn full_policy_rejects_other_omissions_and_any_incomplete_legacy_exception() {
    for (source, state) in [
        ("github.review", "not_collected"),
        ("github.workflow_logs", "request_failed"),
        ("github.history", "incomplete"),
        ("github.pull_request", "request_failed"),
        ("github.repository", "unknown"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut store = Store::open(root.path()).unwrap();
        let result = persist_collection(
            &mut store,
            collection(vec![coverage(source, state)]),
            "github:acme/demo",
            CollectionRequirements::Full,
        );
        assert!(result.is_err(), "{source}: {state}");
        assert_eq!(store.records(None).unwrap().len(), 2);
        store.verify().unwrap();
    }
}

#[test]
fn throughput_allows_deliberately_omitted_optional_families() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let mut scopes = required();
    scopes.extend([
        coverage("github.review", "not_collected"),
        coverage("github.workflow_run", "not_collected"),
        coverage("github.commit", "not_collected"),
        coverage("github.history", "not_collected"),
    ]);
    let receipt = persist_collection(
        &mut store,
        collection(scopes),
        "github:acme/demo",
        CollectionRequirements::Throughput,
    )
    .unwrap();
    assert_eq!(receipt.new_records, 7);
    assert_eq!(store.verify().unwrap(), 7);
}

#[test]
fn throughput_requires_both_required_sources_present_and_complete() {
    for required_source in ["github.repository", "github.pull_request"] {
        for state in [
            None,
            Some("not_collected"),
            Some("not_applicable"),
            Some("request_failed"),
            Some("incomplete"),
        ] {
            let root = tempfile::tempdir().unwrap();
            let mut store = Store::open(root.path()).unwrap();
            let mut scopes = required()
                .into_iter()
                .filter(|c| c.source != required_source)
                .collect::<Vec<_>>();
            if let Some(state) = state {
                scopes.push(coverage(required_source, state));
            }
            let error = persist_collection(
                &mut store,
                collection(scopes),
                "github:acme/demo",
                CollectionRequirements::Throughput,
            )
            .err()
            .expect("required scope must be complete");
            assert!(error.to_string().contains(required_source));
            assert!(
                store
                    .records(None)
                    .unwrap()
                    .iter()
                    .any(|r| r.id == "fixture/pr/1")
            );
            store.verify().unwrap();
        }
    }
}

#[test]
fn throughput_does_not_treat_failed_optional_requests_as_deliberate_omissions() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let mut scopes = required();
    scopes.push(coverage("github.cache", "request_failed"));
    let error = persist_collection(
        &mut store,
        collection(scopes),
        "github:acme/demo",
        CollectionRequirements::Throughput,
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("github.cache"));
    store.verify().unwrap();
}

#[test]
fn incomplete_collection_preserves_prefix_failure_bytes_and_verifiable_manifest() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let mut scopes = required();
    scopes[1].state = "incomplete".into();
    scopes[1].detail = Some("second page returned 403".into());
    let mut input = collection(scopes);
    let prefix = input.observations[0].body.clone();
    let failure = br#"{"message":"Forbidden"}"#.to_vec();
    input.observations.push(RawResponse {
        source: "github.pull_request".into(),
        url: "https://api.github.com/repos/acme/demo/pulls?page=2".into(),
        collected_at: "2026-01-10T12:00:01Z".into(),
        status: Some(403),
        headers: BTreeMap::from([("x-github-request-id".into(), "synthetic-request".into())]),
        sha256: digest(&failure),
        body: failure.clone(),
        error: Some("HTTP 403".into()),
    });
    assert!(
        persist_collection(
            &mut store,
            input,
            "github:acme/demo",
            CollectionRequirements::Throughput
        )
        .is_err()
    );
    assert_eq!(store.verify().unwrap(), 3);
    let manifest = retained_manifest(&store);
    assert_eq!(manifest["responses"].as_array().unwrap().len(), 2);
    assert_eq!(manifest["responses"][1]["status"], 403);
    assert_eq!(manifest["responses"][1]["error"], "HTTP 403");
    assert_eq!(manifest["coverage"][1]["state"], "incomplete");
    for (index, body) in [prefix, failure].into_iter().enumerate() {
        assert!(manifest["responses"][index].get("body").is_none());
        let hash = manifest["raw_objects"][index].as_str().unwrap();
        assert_eq!(manifest["responses"][index]["body_sha256"], hash);
        assert_eq!(
            std::fs::read(store.object_path(hash).unwrap()).unwrap(),
            body
        );
    }
}

#[test]
fn duplicate_rows_and_repeated_collection_are_idempotent() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let mut input = collection(required());
    input.records.push(input.records[0].clone());
    let first = persist_collection(
        &mut store,
        input.clone(),
        "github:acme/demo",
        CollectionRequirements::Throughput,
    )
    .unwrap();
    let second = persist_collection(
        &mut store,
        input,
        "github:acme/demo",
        CollectionRequirements::Throughput,
    )
    .unwrap();
    assert_eq!(first.new_records, 3);
    assert_eq!(second.new_records, 0);
    assert_eq!(first.manifest_sha256, second.manifest_sha256);
    assert_eq!(store.verify().unwrap(), 3);
    assert_eq!(
        store
            .records(None)
            .unwrap()
            .iter()
            .filter(|r| r.id == "fixture/pr/1")
            .count(),
        1
    );
}

#[test]
fn conflicting_source_identity_does_not_overwrite_previously_retained_evidence() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let original = collection(required());
    persist_collection(
        &mut store,
        original.clone(),
        "github:acme/demo",
        CollectionRequirements::Throughput,
    )
    .unwrap();
    let mut conflicting = original.clone();
    conflicting.records[0].data["payload"]["number"] = json!(2);
    assert!(
        persist_collection(
            &mut store,
            conflicting,
            "github:acme/demo",
            CollectionRequirements::Throughput
        )
        .is_err()
    );
    let retained = store.records(None).unwrap();
    assert_eq!(
        retained
            .iter()
            .find(|r| r.id == "fixture/pr/1")
            .unwrap()
            .data,
        original.records[0].data
    );
    assert_eq!(store.verify().unwrap(), 3);
}

#[test]
fn ambiguous_required_scope_cannot_be_promoted_by_another_complete_entry() {
    for extra_state in ["complete", "not_collected", "not_applicable"] {
        for required_source in ["github.repository", "github.pull_request"] {
            let root = tempfile::tempdir().unwrap();
            let mut store = Store::open(root.path()).unwrap();
            let mut scopes = required();
            let mut extra = coverage(required_source, extra_state);
            extra.scope = "https://api.github.com/repos/acme/other/pulls".into();
            scopes.push(extra);
            let result = persist_collection(
                &mut store,
                collection(scopes),
                "github:acme/demo",
                CollectionRequirements::Throughput,
            );
            assert!(
                result.is_err(),
                "ambiguous {required_source}: {extra_state}"
            );
            assert!(
                store
                    .records(None)
                    .unwrap()
                    .iter()
                    .any(|r| r.id == "fixture/pr/1")
            );
            store.verify().unwrap();
        }
    }
}

fn execution_receipt_collection(version: &str) -> Collection {
    let platform = json!({
        "source":"runner_environment", "job_context_source":"caller", "repository_id":"42", "repository":"acme/demo",
        "workflow_ref":"acme/demo/.github/workflows/test.yml@refs/heads/main",
        "workflow_sha":"a".repeat(40), "run_id":"7", "run_attempt":1,
        "job_key":"test", "job_name":"test (linux)", "job_id":null,
        "matrix":{"os":"linux"}, "actor":"original", "triggering_actor":"original",
        "event_name":"workflow_dispatch"
    });
    let receipt = json!({
        "schema":"observatory.tool-receipt/1", "invocation_id":"invocation-1",
        "phase":"start", "timestamp":"2026-11-01T10:00:00Z", "platform":platform,
        "declared":{
            "source":"caller", "controller":"unknown", "controller_provenance":[],
            "protocol_id":null, "task_id":null, "session_id":null, "logical_call_id":null,
            "parent_invocation_id":null
        },
        "tool":{
            "adapter":"command", "name":"cargo", "version":version, "purpose":"development",
            "configured_model":null, "configuration":{
                "reasoning_effort":null,"temperature":null,"seed":null,
                "max_output_tokens":null,"configuration_sha256":null
            }
        },
        "result":null,"artifacts":[],"missing_fields":[]
    });
    let receipt_json = serde_json::to_string(&receipt).unwrap();
    let bytes = serde_json::to_vec(&json!({
        "schema":"observatory.execution-bundle/1", "platform":platform,
        "receipts":[{"sha256":digest(receipt_json.as_bytes()),"receipt_json":receipt_json}]
    }))
    .unwrap();
    let records = observatory::importers::executions_import(&bytes)
        .unwrap()
        .into_iter()
        .map(|record| CollectedRecord {
            id: record.id,
            kind: record.kind,
            data: record.data,
        })
        .collect();
    Collection {
        records,
        observations: vec![RawResponse {
            source: "github.execution_artifact_archive".into(),
            url: "https://api.github.com/repos/acme/demo/actions/artifacts/1/zip".into(),
            collected_at: "2026-11-01T10:00:01Z".into(),
            status: Some(200),
            headers: BTreeMap::new(),
            sha256: digest(&bytes),
            body: bytes,
            error: None,
        }],
        coverage: vec![coverage("github.execution_artifact_bundle", "complete")],
    }
}

#[test]
fn conflicting_execution_delivery_retains_cancelled_job_and_raw_evidence() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    persist_execution_collection(&mut store, execution_receipt_collection("1"), "test").unwrap();
    let mut conflict = execution_receipt_collection("2");
    let original_conflicting_bytes = conflict.observations[0].body.clone();
    conflict.records.extend([
        CollectedRecord {
            id:"fixture/cancelled-job".into(),kind:"source_record".into(),
            data:json!({"source":"github.workflow_job","payload":{"id":17,"run_id":7,"run_attempt":1,"status":"completed","conclusion":"cancelled"}}),
        },
        CollectedRecord {
            id:"fixture/new-reconciliation".into(),kind:"source_record".into(),
            data:json!({"source":"github.execution_job_reconciliation","payload":{"job_id":17,"receipt_observation":"receipts_observed"}}),
        },
    ]);
    conflict
        .coverage
        .push(coverage("github.workflow_job", "complete"));
    let error = persist_execution_collection(&mut store, conflict, "test")
        .err()
        .expect("conflict must return failure after retaining independent observations");
    assert!(
        error
            .to_string()
            .contains("observatory.execution_ingestion")
    );
    let retained = store.retained_records(None).unwrap();
    let job = retained
        .iter()
        .find(|r| r.record.id == "fixture/cancelled-job")
        .unwrap();
    assert_eq!(job.record.data["payload"]["conclusion"], "cancelled");
    assert!(
        !retained
            .iter()
            .any(|r| r.record.id == "fixture/new-reconciliation")
    );
    let receipts: Vec<_> = retained
        .iter()
        .filter(|r| r.record.data["source"] == "observatory.tool_receipt")
        .collect();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].record.data["payload"]["tool"]["version"], "1");
    assert_eq!(
        retained
            .iter()
            .filter(|r| r.record.data["source"] == "observatory.execution_bundle")
            .count(),
        1
    );
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(store.object_path(&job.raw_sha256).unwrap()).unwrap(),
    )
    .unwrap();
    let archive_hash = manifest["raw_objects"][0].as_str().unwrap();
    assert_eq!(
        std::fs::read(store.object_path(archive_hash).unwrap()).unwrap(),
        original_conflicting_bytes
    );
    assert!(
        manifest["coverage"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |v| v["source"] == "github.execution_artifact_bundle" && v["state"] == "incomplete"
            )
    );
    store.verify().unwrap();
}

#[test]
fn identical_execution_delivery_remains_idempotent() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let first = persist_execution_collection(&mut store, execution_receipt_collection("1"), "test")
        .unwrap();
    assert!(first.new_records > 0);
    let second =
        persist_execution_collection(&mut store, execution_receipt_collection("1"), "test")
            .unwrap();
    assert_eq!(second.new_records, 0);
    store.verify().unwrap();
}
