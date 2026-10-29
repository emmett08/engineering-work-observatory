use observatory::collectors::{CollectedRecord, Collection, Coverage, RawResponse};
use observatory::ingestion::{CollectionRequirements, persist_collection};
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
