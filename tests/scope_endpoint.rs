use chrono::{DateTime, Utc};
use observatory::analysis::analyse;
use observatory::model::Evidence;
use serde_json::{Value, json};

fn at(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

fn record(id: &str, kind: &str, data: Value) -> Evidence {
    Evidence {
        id: id.into(),
        kind: kind.into(),
        data,
    }
}

fn completed_original_scope_with_later_change() -> Vec<Evidence> {
    vec![
        record(
            "protocol",
            "protocol",
            json!({"name":"Synthetic scope endpoint", "frozen_at":"2026-09-29T00:00:00Z", "acceptance_days":30, "follow_up_days":30}),
        ),
        record(
            "task",
            "task",
            json!({"protocol_id":"protocol", "requirement":"Preserve the original behaviour", "task_class":"maintenance", "scope_version":"1", "enrolled_at":"2026-10-01T00:00:00Z", "score_frozen_at":"2026-10-01T00:00:00Z", "frozen_score":1}),
        ),
        record(
            "acceptance",
            "lifecycle",
            json!({"task_id":"task", "event":"accepted", "occurred_at":"2026-10-03T00:00:00Z", "scope_version":"1"}),
        ),
        record(
            "follow-up",
            "assessment",
            json!({"task_id":"task", "phase":"follow_up", "assessed_at":"2026-11-02T00:00:00Z", "status":"observed", "retained_fraction":1, "rubric_version":"requirements-v1", "scope_version":"1"}),
        ),
        record(
            "effort",
            "effort",
            json!({"task_id":"task", "person_id":"participant", "role":"implementation", "start":"2026-10-01T09:00:00Z", "end":"2026-10-01T10:00:00Z", "allocation_fraction":1}),
        ),
        record(
            "effort-coverage",
            "coverage",
            json!({"task_id":"task", "domain":"effort", "start":"2026-10-01T00:00:00Z", "end":"2026-11-30T00:00:00Z", "status":"complete"}),
        ),
        record(
            "lifecycle-coverage",
            "coverage",
            json!({"task_id":"task", "domain":"lifecycle", "start":"2026-10-01T00:00:00Z", "end":"2026-11-30T00:00:00Z", "status":"complete"}),
        ),
        record(
            "later-scope-change",
            "task_scope_change",
            json!({"task_id":"task", "occurred_at":"2026-11-15T00:00:00Z", "previous_scope_version":"1", "new_scope_version":"2", "reason":"Additional requested behaviour after original-scope follow-up"}),
        ),
    ]
}

#[test]
fn late_scope_penalty_preserves_known_acceptance_and_qualifies_suppressed_endpoints() {
    let records = completed_original_scope_with_later_change();
    let interim = analyse(&records, at("2026-11-03T00:00:00Z")).unwrap();
    assert_eq!(interim.tasks[0].retained_output, Some(1.0));
    assert_eq!(interim.tasks[0].accepted_by_deadline, Some(true));
    assert!(interim.tasks[0].scope_change_ids.is_empty());
    assert_eq!(interim.tasks[0].person_hours, None);

    let final_report = analyse(&records, at("2026-12-01T00:00:00Z")).unwrap();
    let task = &final_report.tasks[0];
    assert_eq!(task.accepted_at, Some(at("2026-10-03T00:00:00Z")));
    assert_eq!(task.retention_due, Some(at("2026-11-02T00:00:00Z")));
    assert_eq!(task.labour_end, at("2026-11-30T00:00:00Z"));
    assert_eq!(task.person_hours, Some(1.0));
    assert_eq!(task.retained_output, Some(0.0));
    assert_eq!(task.accepted_by_deadline, None);
    assert_eq!(task.restricted_days_without_acceptance, None);
    assert_eq!(task.scope_treatment, "excluded_from_output");
    assert_eq!(task.lifecycle_status, "scope_changed_excluded_from_output");
    assert_eq!(task.scope_change_ids, vec!["later-scope-change"]);
    assert!(task.evidence_ids.iter().any(|id| id == "acceptance"));
    assert!(
        task.issues
            .iter()
            .any(|issue| issue.contains("frozen exclude_from_output rule"))
    );
}

#[test]
fn predeclared_baseline_rule_retains_explicit_original_scope_endpoints() {
    let mut records = completed_original_scope_with_later_change();
    records[0].data["scope_change_rule"] = json!("baseline_only");
    let report = analyse(&records, at("2026-12-01T00:00:00Z")).unwrap();
    let task = &report.tasks[0];
    assert_eq!(task.accepted_at, Some(at("2026-10-03T00:00:00Z")));
    assert_eq!(task.accepted_by_deadline, Some(true));
    assert_eq!(task.restricted_days_without_acceptance, Some(2.0));
    assert_eq!(task.retained_output, Some(1.0));
    assert_eq!(task.retained_fraction, Some(1.0));
    assert_eq!(task.scope_treatment, "baseline_only");
    assert_eq!(task.scope_change_ids, vec!["later-scope-change"]);
}

#[test]
fn explicitly_different_scope_never_establishes_the_enrolled_outcome() {
    for rule in ["exclude_from_output", "baseline_only"] {
        let mut records = completed_original_scope_with_later_change();
        records.retain(|row| row.kind != "task_scope_change");
        records[0].data["scope_change_rule"] = json!(rule);
        let cut = at("2026-12-01T00:00:00Z");
        records[3].data["scope_version"] = json!("2");

        let report = analyse(&records, cut).unwrap();
        let task = &report.tasks[0];
        assert_eq!(task.accepted_by_deadline, Some(true));
        assert_eq!(task.retained_fraction, None);
        assert_eq!(report.cohorts[0].total_retained_output, None);
        assert!(task.evidence_ids.iter().any(|id| id == "follow-up"));

        records[2].data["scope_version"] = json!("2");
        let report = analyse(&records, cut).unwrap();
        let task = &report.tasks[0];
        assert_eq!(task.accepted_at, Some(at("2026-10-03T00:00:00Z")));
        assert_eq!(task.lifecycle_status, "acceptance_scope_mismatch");
        assert_eq!(report.cohorts[0].accepted_by_deadline, None);
        assert_eq!(task.restricted_days_without_acceptance, None);
        assert_eq!(task.person_hours, Some(1.0));
    }
}

#[test]
fn a_change_in_assessment_tolerance_requires_an_original_scope_assessment() {
    let mut records = completed_original_scope_with_later_change();
    records[0].data["assessment_tolerance_days"] = json!(2);
    records[2].data["occurred_at"] = json!("2026-10-31T00:00:00Z");
    records[3].data["assessed_at"] = json!("2026-12-01T12:00:00Z");
    records[3]
        .data
        .as_object_mut()
        .unwrap()
        .remove("scope_version");
    let change = records
        .iter_mut()
        .find(|r| r.kind == "task_scope_change")
        .unwrap();
    change.data["occurred_at"] = json!("2026-12-01T00:00:00Z");
    let cut = at("2026-12-02T00:00:00Z");
    let report = analyse(&records, cut).unwrap();
    let task = &report.tasks[0];
    assert_eq!(task.accepted_by_deadline, Some(true));
    assert!(task.scope_change_ids.is_empty());
    assert_eq!(task.retained_output, None);
    assert!(
        task.evidence_ids
            .iter()
            .any(|id| id == "later-scope-change")
    );

    records[3].data["scope_version"] = json!("1");
    assert_eq!(
        analyse(&records, cut).unwrap().tasks[0].retained_output,
        Some(1.0)
    );
}
