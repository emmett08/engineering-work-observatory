use chrono::{DateTime, Utc};
use observatory::{model::Evidence, snapshot::Snapshot, store::Store};
use serde_json::json;

fn at(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}
fn record(id: &str, kind: &str, data: serde_json::Value) -> Evidence {
    Evidence {
        id: id.into(),
        kind: kind.into(),
        data,
    }
}
fn source(id: &str) -> Evidence {
    record(id, "source_record", json!({"source":"synthetic_test"}))
}

#[test]
fn idempotence_conflicts_and_source_corruption_are_checked() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let hash = store.retain_bytes(b"synthetic raw evidence").unwrap();
    let time = at("2026-01-01T00:00:00Z");
    assert_eq!(
        store
            .insert_batch(&[source("a")], "test", time, &hash)
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .insert_batch(&[source("a")], "test", time, &hash)
            .unwrap(),
        0
    );
    let conflict = record("a", "source_record", json!({"different":true}));
    assert!(
        store
            .insert_batch(&[source("b"), conflict], "test", time, &hash)
            .is_err()
    );
    assert_eq!(store.records(None).unwrap().len(), 1);
    std::fs::write(store.object_path(&hash).unwrap(), b"corrupt").unwrap();
    assert!(store.verify().is_err());
    assert!(
        store
            .insert_batch(&[source("c")], "test", time, &hash)
            .is_err()
    );
}

#[test]
fn nested_evidence_cannot_disappear_unnoticed() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let child = store.retain_bytes(b"HTTP response bytes").unwrap();
    let manifest = json!({"schema":"observatory.collection/1","raw_objects":[child]});
    let hash = store
        .retain_bytes(&serde_json::to_vec(&manifest).unwrap())
        .unwrap();
    let time = at("2026-01-01T00:00:00Z");
    store
        .insert_batch(&[source("a")], "test", time, &hash)
        .unwrap();
    store.verify().unwrap();
    std::fs::remove_file(store.object_path(&child).unwrap()).unwrap();
    assert!(store.verify().is_err());
    assert!(Snapshot::freeze(&store, time).is_err());
}

#[test]
fn historical_cut_excludes_later_observations_and_has_stable_identity() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let hash = store.retain_bytes(b"raw").unwrap();
    let cut = at("2026-01-02T00:00:00Z");
    store
        .insert_batch(&[source("a")], "test", at("2026-01-01T00:00:00Z"), &hash)
        .unwrap();
    store
        .insert_batch(&[source("b")], "test", at("2026-01-03T00:00:00Z"), &hash)
        .unwrap();
    let a = Snapshot::freeze(&store, cut).unwrap();
    let b = Snapshot::freeze(&store, cut).unwrap();
    assert_eq!(a.sha256, b.sha256);
    assert_eq!(a.records.len(), 1);
    let mut changed = a;
    changed.records[0].record.data = json!({"changed":true});
    assert!(changed.verify().is_err());
}

#[test]
fn frozen_facts_select_latest_receipt_available_at_each_cut() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let hash = store.retain_bytes(b"raw").unwrap();
    let early = at("2026-01-01T00:00:00Z");
    let late = at("2026-01-03T00:00:00Z");
    store
        .insert_batch(&[source("a")], "test", early, &hash)
        .unwrap();
    store
        .insert_batch(&[source("a")], "test", late, &hash)
        .unwrap();
    assert_eq!(
        Snapshot::freeze(&store, at("2026-01-02T00:00:00Z"))
            .unwrap()
            .records[0]
            .observed_at,
        early
    );
    assert_eq!(
        Snapshot::freeze(&store, late).unwrap().records[0].observed_at,
        late
    );
}

fn task_records() -> Vec<Evidence> {
    vec![
        record(
            "protocol",
            "protocol",
            json!({"name":"synthetic","frozen_at":"2026-01-01T00:00:00Z","acceptance_days":30,"follow_up_days":30}),
        ),
        record(
            "task",
            "task",
            json!({"protocol_id":"protocol","requirement":"test only","task_class":"maintenance","scope_version":"1","enrolled_at":"2026-01-01T00:00:00Z","score_frozen_at":"2026-01-01T00:00:00Z","frozen_score":1.0}),
        ),
    ]
}

#[test]
fn freeze_rejects_a_dependency_not_yet_observed_at_cut() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let hash = store.retain_bytes(b"raw").unwrap();
    let records = task_records();
    store
        .insert_batch(&records[..1], "test", at("2026-01-03T00:00:00Z"), &hash)
        .unwrap();
    store
        .insert_batch(&records[1..], "test", at("2026-01-01T00:00:00Z"), &hash)
        .unwrap();
    assert!(Snapshot::freeze(&store, at("2026-01-02T00:00:00Z")).is_err());
}

#[test]
fn concurrent_batches_validate_against_committed_person_allocation() {
    let root = tempfile::tempdir().unwrap();
    let mut first = Store::open(root.path()).unwrap();
    let hash = first.retain_bytes(b"raw").unwrap();
    let time = at("2026-01-01T00:00:00Z");
    first
        .insert_batch(&task_records(), "test", time, &hash)
        .unwrap();
    let second = Store::open(root.path()).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles=[first,second].into_iter().enumerate().map(|(i,mut store)|{
        let barrier=barrier.clone();let hash=hash.clone();
        std::thread::spawn(move||{
            let effort=record(&format!("effort{i}"),"effort",json!({"task_id":"task","person_id":"person","role":"review","start":"2026-01-01T09:00:00Z","end":"2026-01-01T10:00:00Z","allocation_fraction":0.6}));
            barrier.wait();store.insert_batch(&[effort],"test",time,&hash).is_ok()
        })
    }).collect::<Vec<_>>();
    let successes = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .filter(|ok| *ok)
        .count();
    assert_eq!(successes, 1);
    assert_eq!(
        Store::open(root.path())
            .unwrap()
            .records(None)
            .unwrap()
            .iter()
            .filter(|r| r.kind == "effort")
            .count(),
        1
    );
}

#[test]
fn conflicting_logical_calls_are_rejected_but_unknown_usage_can_be_enriched() {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let hash = store.retain_bytes(b"raw").unwrap();
    let time = at("2026-01-01T00:00:00Z");
    let a = record(
        "a",
        "source_record",
        json!({"canonical_call_id":"call1","core_call_sha256":"same","usage":{"input_tokens":null}}),
    );
    let b = record(
        "b",
        "source_record",
        json!({"canonical_call_id":"call1","core_call_sha256":"same","usage":{"input_tokens":10}}),
    );
    store.insert_batch(&[a, b], "test", time, &hash).unwrap();
    let c = record(
        "c",
        "source_record",
        json!({"canonical_call_id":"call1","core_call_sha256":"same","usage":{"input_tokens":11}}),
    );
    assert!(store.insert_batch(&[c], "test", time, &hash).is_err());
}
