use chrono::{DateTime, Utc};
use observatory::{forecast::history::prepare, model::Evidence, snapshot::Snapshot, store::Store};
use serde_json::{Value, json};

fn at(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

fn source(id: &str, repository: u64, family: &str, payload: Value) -> Evidence {
    Evidence {
        id: id.into(),
        kind: "source_record".into(),
        data: json!({
            "source":family, "repository":format!("github:repository:{repository}"),
            "source_id":id,"payload":payload,
        }),
    }
}

fn pr(number: u64, created: &str, merged: Option<&str>) -> Value {
    json!({"id":1000+number,"number":number,"created_at":created,
        "state":if merged.is_some() {"closed"} else {"open"},
        "merged_at":merged,"closed_at":merged})
}

struct Acquisition<'a> {
    tag: &'a str,
    repository: u64,
    name: &'a str,
    created: &'a str,
    start: &'a str,
    end: &'a str,
    receipt: &'a str,
    complete: bool,
    prs: Vec<Value>,
    extras: Vec<Evidence>,
}

fn acquire(store: &mut Store, acquisition: Acquisition<'_>) {
    let Acquisition {
        tag,
        repository,
        name,
        created,
        start,
        end,
        receipt,
        complete,
        prs,
        extras,
    } = acquisition;
    let coverage = json!([
        {"source":"github.repository","scope":format!("https://api.github.com/repos/{name}"),
            "state":"complete","pages":1,"records":1},
        {"source":"github.pull_request","scope":format!("https://api.github.com/repos/{name}/pulls?state=all&per_page=100"),
            "state":if complete {"complete"} else {"incomplete"},"pages":1,"records":prs.len()}
    ]);
    let unique = prs
        .iter()
        .filter_map(|payload| payload.get("number").and_then(Value::as_u64))
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let mut records = vec![
        source(
            &format!("{tag}/metadata"),
            repository,
            "github.repository",
            json!({"id":repository,"full_name":name,"created_at":created}),
        ),
        source(
            &format!("{tag}/summary"),
            repository,
            "github.throughput_collection",
            json!({"schema":"observatory.github-throughput-collection/1","profile":"throughput",
                "started_at":start,"completed_at":end,"coverage":coverage,
                "unique_pull_requests":unique,"duplicate_pull_request_rows":prs.len()-unique}),
        ),
    ];
    records.extend(prs.into_iter().enumerate().map(|(index, payload)| {
        source(
            &format!("{tag}/pr/{index}"),
            repository,
            "github.pull_request",
            payload,
        )
    }));
    records.extend(extras);
    let raw = store
        .retain_bytes(&serde_json::to_vec(&records).unwrap())
        .unwrap();
    store
        .insert_batch(&records, "automatic-github-test", at(receipt), &raw)
        .unwrap();
}

fn fixture<'a>(tag: &'a str, prs: Vec<Value>) -> Acquisition<'a> {
    Acquisition {
        tag,
        repository: 1,
        name: "team/repo",
        created: "2026-10-01T12:00:00Z",
        start: "2026-10-10T12:00:00Z",
        end: "2026-10-10T12:01:00Z",
        receipt: "2026-10-10T12:02:00Z",
        complete: true,
        prs,
        extras: vec![],
    }
}

#[test]
fn projects_machine_identity_once_and_reports_only_observed_pressure() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    let merged = pr(1, "2026-10-03T00:00:00Z", Some("2026-10-04T00:00:00Z"));
    let mut acquisition = fixture(
        "one",
        vec![merged.clone(), pr(2, "2026-10-08T00:00:00Z", None)],
    );
    acquisition.extras.push(source(
        "one/detail",
        1,
        "github.pull_request_detail",
        merged,
    ));
    acquire(&mut store, acquisition);
    let snapshot = Snapshot::freeze(&store, at("2026-10-10T12:02:00Z")).unwrap();
    let history = prepare(&snapshot, &["TEAM/REPO".into()]).unwrap();
    assert!(history.complete, "{:?}", history.reasons);
    assert_eq!(history.pull_requests.len(), 2);
    assert_eq!(history.pull_requests[0].source_record_id, "one/detail");
    assert_eq!(history.current_pressure.open_pull_requests, Some(1));
    assert_eq!(history.current_pressure.open_draft_pull_requests, None);
    assert_eq!(
        history.current_pressure.open_review_requested_pull_requests,
        None
    );
    assert_eq!(history.current_pressure.review_status, "not_collected");
    assert_eq!(history.current_pressure.ci_status, "not_collected");
    assert_eq!(history.mode, "retrospective_visible_inventory");
}

#[test]
fn stale_snapshot_does_not_extend_exposure_or_age_old_state_as_current() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    acquire(
        &mut store,
        fixture("one", vec![pr(1, "2026-10-09T12:02:00Z", None)]),
    );
    let snapshot = Snapshot::freeze(&store, at("2026-10-29T00:00:00Z")).unwrap();
    let history = prepare(&snapshot, &[]).unwrap();
    assert!(history.complete, "{:?}", history.reasons);
    assert_eq!(history.snapshot_cut, at("2026-10-29T00:00:00Z"));
    assert_eq!(history.observed_cut, at("2026-10-10T12:00:00Z"));
    assert_eq!(history.current_pressure.open_age_days, vec![1.0]);
    assert!(
        history
            .current_pressure
            .oldest_collection_age_seconds
            .unwrap()
            > 18.0 * 86400.0
    );
}

#[test]
fn crossing_midnight_uses_common_scan_start_and_latest_repository_birth() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    let mut first = fixture(
        "one",
        vec![pr(1, "2026-10-03T00:00:00Z", Some("2026-10-09T23:59:30Z"))],
    );
    first.start = "2026-10-09T23:59:00Z";
    first.end = "2026-10-10T00:01:00Z";
    first.receipt = "2026-10-10T00:02:00Z";
    acquire(&mut store, first);
    let mut second = fixture("two", vec![]);
    second.repository = 2;
    second.name = "team/other";
    second.created = "2026-10-03T13:00:00Z";
    acquire(&mut store, second);
    let history = prepare(
        &Snapshot::freeze(&store, at("2026-10-11T00:00:00Z")).unwrap(),
        &[],
    )
    .unwrap();
    assert!(history.complete, "{:?}", history.reasons);
    assert_eq!(history.observed_cut, at("2026-10-09T23:59:00Z"));
    assert_eq!(history.history_start, at("2026-10-03T13:00:00Z"));
    // A valid later-page event is retained, but cannot enter bins after the common exposure cut.
    assert!(history.pull_requests[0].merged_at.unwrap() > history.observed_cut);
}

#[test]
fn incomplete_latest_scan_cannot_fall_back_to_older_success_or_zero() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    acquire(
        &mut store,
        fixture("one", vec![pr(1, "2026-10-03T00:00:00Z", None)]),
    );
    let mut failed = fixture("two", vec![]);
    failed.start = "2026-10-11T12:00:00Z";
    failed.end = "2026-10-11T12:01:00Z";
    failed.receipt = "2026-10-11T12:02:00Z";
    failed.complete = false;
    acquire(&mut store, failed);
    let history = prepare(
        &Snapshot::freeze(&store, at("2026-10-12T00:00:00Z")).unwrap(),
        &[],
    )
    .unwrap();
    assert!(!history.complete);
    assert!(
        history
            .reasons
            .iter()
            .any(|reason| reason.contains("retrieval incomplete"))
    );
    assert_eq!(history.current_pressure.open_pull_requests, None);
    assert!(history.pull_requests.is_empty());
}

#[test]
fn later_source_versions_do_not_change_an_earlier_knowledge_cut() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    acquire(
        &mut store,
        fixture("one", vec![pr(1, "2026-10-03T00:00:00Z", None)]),
    );
    let cut = at("2026-10-10T12:02:00Z");
    let before = Snapshot::freeze(&store, cut).unwrap();
    let mut later = fixture(
        "two",
        vec![pr(1, "2026-10-03T00:00:00Z", Some("2026-10-11T10:00:00Z"))],
    );
    later.start = "2026-10-11T12:00:00Z";
    later.end = "2026-10-11T12:01:00Z";
    later.receipt = "2026-10-11T12:02:00Z";
    acquire(&mut store, later);
    let after = Snapshot::freeze(&store, cut).unwrap();
    assert_eq!(before.sha256, after.sha256);
    assert_eq!(
        prepare(&after, &[]).unwrap().pull_requests[0].merged_at,
        None
    );
    let latest = prepare(
        &Snapshot::freeze(&store, at("2026-10-12T00:00:00Z")).unwrap(),
        &[],
    )
    .unwrap();
    assert!(latest.complete, "{:?}", latest.reasons);
    assert_eq!(latest.pull_requests.len(), 1);
    assert_eq!(
        latest.pull_requests[0].merged_at,
        Some(at("2026-10-11T10:00:00Z"))
    );
}

#[test]
fn a_manual_coverage_attestation_does_not_replace_automatic_inventory() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    let records = vec![
        source(
            "metadata",
            1,
            "github.repository",
            json!({"id":1,"full_name":"team/repo","created_at":"2026-10-01T00:00:00Z"}),
        ),
        Evidence {
            id: "manual-complete".into(),
            kind: "source_record".into(),
            data: json!({"record_type":"collection_coverage","coverage":{"source":"github.pull_request","state":"complete","records":0}}),
        },
    ];
    let raw = store.retain_bytes(b"fixture").unwrap();
    store
        .insert_batch(&records, "test", at("2026-10-10T00:00:00Z"), &raw)
        .unwrap();
    let history = prepare(
        &Snapshot::freeze(&store, at("2026-10-11T00:00:00Z")).unwrap(),
        &[],
    )
    .unwrap();
    assert!(!history.complete);
    assert!(
        history
            .reasons
            .iter()
            .any(|reason| reason.contains("missing automatic throughput-profile"))
    );
}

#[test]
fn conflicting_same_receipt_variants_and_post_collection_events_fail_closed() {
    for conflict in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let mut store = Store::open(directory.path()).unwrap();
        let mut acquisition = fixture("one", vec![pr(1, "2026-10-03T00:00:00Z", None)]);
        if conflict {
            acquisition.extras.push(source(
                "one/conflict",
                1,
                "github.pull_request",
                pr(1, "2026-10-03T00:00:00Z", Some("2026-10-09T00:00:00Z")),
            ));
        } else {
            acquisition.prs = vec![pr(1, "2026-10-03T00:00:00Z", Some("2026-10-11T00:00:00Z"))];
        }
        acquire(&mut store, acquisition);
        let history = prepare(
            &Snapshot::freeze(&store, at("2026-10-12T00:00:00Z")).unwrap(),
            &[],
        )
        .unwrap();
        assert!(!history.complete, "conflict={conflict}");
        assert_eq!(history.current_pressure.open_pull_requests, None);
    }
}

#[test]
fn missing_requested_repository_invalidates_portfolio_instead_of_reducing_scope() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    acquire(&mut store, fixture("one", vec![]));
    let history = prepare(
        &Snapshot::freeze(&store, at("2026-10-11T00:00:00Z")).unwrap(),
        &["team/repo".into(), "team/missing".into()],
    )
    .unwrap();
    assert!(!history.complete);
    assert_eq!(history.current_pressure.open_pull_requests, None);
}

#[test]
fn exact_duplicate_pages_are_idempotent_with_automatic_duplicate_accounting() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    let row = pr(1, "2026-10-03T00:00:00Z", Some("2026-10-04T00:00:00Z"));
    acquire(&mut store, fixture("one", vec![row.clone(), row]));
    let history = prepare(
        &Snapshot::freeze(&store, at("2026-10-11T00:00:00Z")).unwrap(),
        &[],
    )
    .unwrap();
    assert!(history.complete, "{:?}", history.reasons);
    assert_eq!(history.pull_requests.len(), 1);
}

#[test]
fn conflicts_in_an_unselected_repository_do_not_poison_the_selected_scope() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    let mut acquisition = fixture("one", vec![]);
    for (index, name) in ["other/first", "other/conflict"].into_iter().enumerate() {
        acquisition.extras.push(source(
            &format!("unselected/{index}"),
            2,
            "github.repository",
            json!({"id":2,"full_name":name,"created_at":"2026-10-01T00:00:00Z"}),
        ));
    }
    acquire(&mut store, acquisition);
    let history = prepare(
        &Snapshot::freeze(&store, at("2026-10-11T00:00:00Z")).unwrap(),
        &["team/repo".into()],
    )
    .unwrap();
    assert!(history.complete, "{:?}", history.reasons);
    assert_eq!(history.repositories.len(), 1);
}

#[test]
fn optional_current_predicates_overlap_and_do_not_become_historical_features() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    let mut row = pr(1, "2026-10-02T00:00:00Z", None);
    row["draft"] = json!(true);
    row["requested_reviewers"] = json!([]);
    row["requested_teams"] = json!([{"id":7}]);
    acquire(&mut store, fixture("one", vec![row]));
    let history = prepare(
        &Snapshot::freeze(&store, at("2026-10-11T00:00:00Z")).unwrap(),
        &[],
    )
    .unwrap();
    assert!(history.complete, "{:?}", history.reasons);
    let pressure = &history.current_pressure;
    assert_eq!(pressure.open_pull_requests, Some(1));
    assert_eq!(pressure.open_draft_pull_requests, Some(1));
    assert_eq!(pressure.open_review_requested_pull_requests, Some(1));
    assert_eq!(pressure.open_at_least_7_days_pull_requests, Some(1));
    assert_eq!(pressure.review_status, "not_collected");
    assert!(pressure.source_record_ids.contains(&"one/summary".into()));
    assert!(pressure.source_record_ids.contains(&"one/pr/0".into()));
    let count_row = serde_json::to_value(&history.pull_requests[0]).unwrap();
    assert!(count_row.get("draft").is_none());
    assert!(count_row.get("requested_teams").is_none());
}

#[test]
fn empty_complete_queue_has_observed_zero_optional_predicates() {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    acquire(&mut store, fixture("one", vec![]));
    let history = prepare(
        &Snapshot::freeze(&store, at("2026-10-11T00:00:00Z")).unwrap(),
        &[],
    )
    .unwrap();
    assert!(history.complete, "{:?}", history.reasons);
    let pressure = &history.current_pressure;
    assert_eq!(pressure.open_pull_requests, Some(0));
    assert_eq!(pressure.open_draft_pull_requests, Some(0));
    assert_eq!(pressure.open_review_requested_pull_requests, Some(0));
    assert_eq!(pressure.open_at_least_7_days_pull_requests, Some(0));
}

#[test]
fn missing_optional_request_field_is_unknown_unless_an_observed_request_is_positive() {
    for positive in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let mut store = Store::open(directory.path()).unwrap();
        let mut row = pr(1, "2026-10-09T00:00:00Z", None);
        row["draft"] = json!(false);
        row["requested_reviewers"] = if positive {
            json!([{"id":9}])
        } else {
            json!([])
        };
        acquire(&mut store, fixture("one", vec![row]));
        let history = prepare(
            &Snapshot::freeze(&store, at("2026-10-11T00:00:00Z")).unwrap(),
            &[],
        )
        .unwrap();
        assert_eq!(history.current_pressure.open_draft_pull_requests, Some(0));
        assert_eq!(
            history.current_pressure.open_review_requested_pull_requests,
            if positive { Some(1) } else { None }
        );
    }
}
