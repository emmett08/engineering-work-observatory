use chrono::{DateTime, Utc};
use observatory::analysis::analyse;
use observatory::model::{Evidence, effective_records, validate_records};
use serde_json::{Value, json};

fn at(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}
fn rec(id: &str, kind: &str, data: Value) -> Evidence {
    Evidence {
        id: id.into(),
        kind: kind.into(),
        data,
    }
}
fn fixture() -> Vec<Evidence> {
    vec![
        rec(
            "p",
            "protocol",
            json!({"name":"Synthetic fixture", "frozen_at":"2026-09-29T00:00:00Z", "acceptance_days":30,"follow_up_days":30,"assessment_tolerance_days":3}),
        ),
        rec(
            "t",
            "task",
            json!({"protocol_id":"p","requirement":"Preserve the requested behaviour", "task_class":"maintenance","scope_version":"1","enrolled_at":"2026-10-01T00:00:00Z","score_frozen_at":"2026-10-01T00:00:00Z","frozen_score":1}),
        ),
        rec(
            "accepted",
            "lifecycle",
            json!({"task_id":"t","event":"accepted","occurred_at":"2026-10-03T00:00:00Z"}),
        ),
        rec(
            "hours",
            "effort",
            json!({"task_id":"t","person_id":"person-a","role":"implementation","start":"2026-10-01T09:00:00Z","end":"2026-10-01T10:00:00Z","allocation_fraction":1}),
        ),
        rec(
            "lc",
            "coverage",
            json!({"task_id":"t","domain":"lifecycle","start":"2026-10-01T00:00:00Z","end":"2026-11-30T00:00:00Z","status":"complete"}),
        ),
        rec(
            "hc",
            "coverage",
            json!({"task_id":"t","domain":"effort","start":"2026-10-01T00:00:00Z","end":"2026-11-30T00:00:00Z","status":"complete"}),
        ),
        rec(
            "audit",
            "assessment",
            json!({"task_id":"t","phase":"follow_up","assessed_at":"2026-11-02T00:00:00Z","status":"observed","retained_fraction":1,"rubric_version":"requirements-v1"}),
        ),
    ]
}
fn final_cut() -> DateTime<Utc> {
    at("2026-12-01T00:00:00Z")
}

#[test]
fn early_retention_does_not_close_fixed_labour_horizon() {
    let report = analyse(&fixture(), at("2026-11-03T00:00:00Z")).unwrap();
    assert_eq!(report.tasks[0].retained_output, Some(1.0));
    assert_eq!(report.tasks[0].person_hours, None);
    assert_eq!(report.tasks[0].labour_end, at("2026-11-30T00:00:00Z"));
    assert_eq!(report.cohorts[0].retained_output_per_person_hour, None);
}

#[test]
fn late_acceptance_gets_no_deadline_output_but_keeps_labour() {
    let mut records = fixture();
    records.retain(|r| r.kind != "assessment");
    records
        .iter_mut()
        .find(|r| r.id == "accepted")
        .unwrap()
        .data["occurred_at"] = json!("2026-11-10T00:00:00Z");
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].retained_output, Some(0.0));
    assert_eq!(report.tasks[0].person_hours, Some(1.0));
    assert_eq!(
        report.tasks[0].restricted_days_without_acceptance,
        Some(30.0)
    );
}

#[test]
fn pair_work_adds_both_people_and_overallocation_is_rejected() {
    let mut records = fixture();
    let mut second = records.iter().find(|r| r.id == "hours").unwrap().clone();
    second.id = "reviewer".into();
    second.data["person_id"] = json!("person-b");
    records.push(second);
    assert_eq!(
        analyse(&records, final_cut()).unwrap().tasks[0].person_hours,
        Some(2.0)
    );
    records.last_mut().unwrap().data["person_id"] = json!("person-a");
    records.last_mut().unwrap().data["role"] = json!("review");
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("overallocated")
    );
}

#[test]
fn missing_effort_coverage_is_not_a_complete_zero_or_total() {
    let mut records = fixture();
    records.retain(|r| r.id != "hc");
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].recorded_person_hours, 1.0);
    assert_eq!(report.tasks[0].person_hours, None);
    assert_eq!(report.cohorts[0].retained_output_per_person_hour, None);
}

#[test]
fn missing_retention_is_not_perfect_or_zero_retention() {
    let mut records = fixture();
    records.retain(|r| r.id != "audit");
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].retained_output, None);
    assert_eq!(report.tasks[0].person_hours, Some(1.0));
}

#[test]
fn ht_sampled_effort_uses_inclusion_probability_without_claiming_variance() {
    let mut records = fixture();
    let e = &mut records.iter_mut().find(|r| r.id == "hours").unwrap().data;
    e["method"] = json!("sampled");
    e["inclusion_probability"] = json!(0.25);
    records.iter_mut().find(|r| r.id == "hc").unwrap().data["method"] = json!("sampled");
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].person_hours, Some(4.0));
    assert_eq!(
        report.cohorts[0].retained_output_per_person_hour,
        Some(0.25)
    );
    assert!(report.cohorts[0].contains_sampled_effort_estimates);
    assert!(
        report.cohorts[0]
            .uncertainty
            .contains("variance is not estimated")
    );
}

#[test]
fn sample_nonresponse_is_unavailable_not_zero() {
    let mut records = fixture();
    let e = &mut records.iter_mut().find(|r| r.id == "hours").unwrap().data;
    e["method"] = json!("sampled");
    e["inclusion_probability"] = json!(0.25);
    e["response_status"] = json!("missing");
    e["allocation_fraction"] = Value::Null;
    e["missing_reason"] = json!("No response");
    records.iter_mut().find(|r| r.id == "hc").unwrap().data["method"] = json!("sampled");
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].person_hours, None);
    assert_eq!(report.tasks[0].nonresponse_intervals, 1);
}

#[test]
fn abandonment_and_lost_observation_have_different_outcomes() {
    let mut records = fixture();
    records.retain(|r| r.kind != "assessment");
    records
        .iter_mut()
        .find(|r| r.id == "accepted")
        .unwrap()
        .data["event"] = json!("abandoned");
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].retained_output, Some(0.0));
    assert_eq!(
        report.tasks[0].restricted_days_without_acceptance,
        Some(30.0)
    );
    records
        .iter_mut()
        .find(|r| r.id == "accepted")
        .unwrap()
        .data["event"] = json!("lost_to_follow_up");
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].accepted_by_deadline, None);
    assert_eq!(report.tasks[0].retained_output, None);
}

#[test]
fn aggregate_productivity_is_ratio_of_totals() {
    let mut records = fixture();
    let mut second: Vec<_> = fixture()
        .into_iter()
        .filter(|r| r.kind != "protocol")
        .collect();
    for r in &mut second {
        r.id = format!("{}-two", r.id);
        if r.kind == "task" {
            r.id = "t-two".into();
        }
        if r.data.get("task_id").is_some() {
            r.data["task_id"] = json!("t-two");
        }
        if r.kind == "effort" {
            r.data["person_id"] = json!("person-b");
            r.data["end"] = json!("2026-10-01T12:00:00Z");
        }
    }
    records.extend(second);
    let cohort = &analyse(&records, final_cut()).unwrap().cohorts[0];
    assert_eq!(cohort.total_person_hours, Some(4.0));
    assert_eq!(cohort.total_retained_output, Some(2.0));
    assert_eq!(cohort.retained_output_per_person_hour, Some(0.5));
}

#[test]
fn zero_total_labour_has_no_ratio() {
    let mut records = fixture();
    records.iter_mut().find(|r| r.id == "hours").unwrap().data["allocation_fraction"] = json!(0.0);
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.cohorts[0].total_person_hours, Some(0.0));
    assert_eq!(report.cohorts[0].retained_output_per_person_hour, None);
    assert_eq!(
        report.cohorts[0].ratio_status,
        "unavailable_zero_labour_denominator"
    );
}

#[test]
fn boundary_crossing_average_allocation_requires_observed_splits() {
    for method in ["direct", "sampled"] {
        let mut records = fixture();
        if method == "sampled" {
            let hours = &mut records.iter_mut().find(|r| r.id == "hours").unwrap().data;
            hours["method"] = json!(method);
            hours["inclusion_probability"] = json!(1.0);
            records.iter_mut().find(|r| r.id == "hc").unwrap().data["method"] = json!(method);
        }
        let mut crossing = rec(
            "crossing",
            "effort",
            json!({"task_id":"t","person_id":"person-a","role":"repair",
                "start":"2026-11-29T23:30:00Z","end":"2026-11-30T00:30:00Z",
                "allocation_fraction":0.5,"method":method}),
        );
        if method == "sampled" {
            crossing.data["inclusion_probability"] = json!(1.0);
        }
        records.push(crossing);
        let report = analyse(&records, final_cut()).unwrap();
        assert_eq!(report.tasks[0].person_hours, None, "{method}");
        assert_eq!(report.tasks[0].recorded_person_hours, 1.0, "{method}");
        assert!(
            report.tasks[0].issues.iter().any(|issue| {
                issue.contains("effort interval crosses") && issue.contains("split")
            })
        );

        // Separately observed allocations identify the two parts; the whole-hour
        // average alone could not determine this half-hour of in-horizon work.
        let before = records.iter_mut().find(|r| r.id == "crossing").unwrap();
        before.data["end"] = json!("2026-11-30T00:00:00Z");
        before.data["allocation_fraction"] = json!(1.0);
        let mut after = before.clone();
        after.id = "after-boundary".into();
        after.data["start"] = json!("2026-11-30T00:00:00Z");
        after.data["end"] = json!("2026-11-30T00:30:00Z");
        after.data["allocation_fraction"] = json!(0.0);
        records.push(after);
        let report = analyse(&records, final_cut()).unwrap();
        assert_eq!(report.tasks[0].person_hours, Some(1.5), "{method}");
        assert_eq!(report.tasks[0].recorded_person_hours, 1.5, "{method}");
    }
}

#[test]
fn zero_direct_average_identifies_zero_work_across_the_boundary() {
    let mut records = fixture();
    let hours = &mut records.iter_mut().find(|r| r.id == "hours").unwrap().data;
    hours["start"] = json!("2026-11-29T23:30:00Z");
    hours["end"] = json!("2026-11-30T00:30:00Z");
    hours["allocation_fraction"] = json!(0.0);
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].person_hours, Some(0.0));
    assert_eq!(report.tasks[0].recorded_person_hours, 0.0);
}

#[test]
fn full_direct_average_identifies_work_across_the_boundary() {
    let mut records = fixture();
    let hours = &mut records.iter_mut().find(|r| r.id == "hours").unwrap().data;
    hours["start"] = json!("2026-11-29T23:30:00Z");
    hours["end"] = json!("2026-11-30T00:30:00Z");
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].person_hours, Some(0.5));
    assert_eq!(report.tasks[0].recorded_person_hours, 0.5);
}

#[test]
fn submillisecond_effort_remains_positive_when_subdivided() {
    let mut records = fixture();
    let hours = &mut records.iter_mut().find(|r| r.id == "hours").unwrap().data;
    hours["end"] = json!("2026-10-01T09:00:00.001Z");
    let whole = analyse(&records, final_cut()).unwrap().tasks[0]
        .person_hours
        .unwrap();
    assert!(whole > 0.0);

    let first = records.iter_mut().find(|r| r.id == "hours").unwrap();
    first.data["end"] = json!("2026-10-01T09:00:00.0005Z");
    let mut second = first.clone();
    second.id = "second-half".into();
    second.data["start"] = json!("2026-10-01T09:00:00.0005Z");
    second.data["end"] = json!("2026-10-01T09:00:00.001Z");
    records.push(second);
    let report = analyse(&records, final_cut()).unwrap();
    let split = report.tasks[0].person_hours.unwrap();
    assert!(split > 0.0);
    assert!((split - whole).abs() <= whole * 1e-12);
    assert!(report.cohorts[0].retained_output_per_person_hour.is_some());
}

#[test]
fn long_effort_duration_does_not_require_an_i64_nanosecond_count() {
    let mut records = fixture();
    for record in &mut records {
        match record.id.as_str() {
            "p" => {
                record.data["frozen_at"] = json!("1599-01-01T00:00:00Z");
                record.data["acceptance_days"] = json!(200_000);
            }
            "t" => {
                record.data["enrolled_at"] = json!("1600-01-01T00:00:00Z");
                record.data["score_frozen_at"] = json!("1600-01-01T00:00:00Z");
            }
            "accepted" => record.data["occurred_at"] = json!("1600-01-02T00:00:00Z"),
            "audit" => record.data["assessed_at"] = json!("1600-02-01T00:00:00Z"),
            "hours" => {
                record.data["start"] = json!("1600-01-01T00:00:00Z");
                record.data["end"] = json!("2000-01-01T00:00:00Z");
            }
            "lc" | "hc" => {
                record.data["start"] = json!("1600-01-01T00:00:00Z");
                record.data["end"] = json!("2300-01-01T00:00:00Z");
            }
            _ => {}
        }
    }
    let report = analyse(&records, at("2300-01-01T00:00:00Z")).unwrap();
    assert_eq!(report.tasks[0].person_hours, Some(146_097.0 * 24.0));
}

#[test]
fn parent_and_child_cannot_both_create_output() {
    let mut records = fixture();
    let mut child = records.iter().find(|r| r.kind == "task").unwrap().clone();
    child.id = "child".into();
    child.data["parent_task_id"] = json!("t");
    records.push(child);
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("both count output")
    );
    records.last_mut().unwrap().data["count_output"] = json!(false);
    validate_records(&records).unwrap();
}

#[test]
fn idempotent_records_are_allowed_but_conflicts_and_unknown_kinds_reject() {
    let mut records = fixture();
    records.push(records[0].clone());
    validate_records(&records).unwrap();
    records.last_mut().unwrap().data["name"] = json!("Conflicting protocol");
    assert!(validate_records(&records).is_err());
    assert!(validate_records(&[rec("x", "unknown", json!({}))]).is_err());
    assert!(validate_records(&[rec("x", "source_record", json!([]))]).is_err());
}

#[test]
fn future_assessment_is_not_used_before_as_of_cut() {
    let report = analyse(&fixture(), at("2026-10-15T00:00:00Z")).unwrap();
    assert_eq!(report.tasks[0].retained_output, None);
    assert_eq!(report.tasks[0].retained_fraction, None);
    assert!(!report.tasks[0].evidence_ids.contains(&"audit".into()));
}

#[test]
fn score_must_be_frozen_before_enrolment() {
    let mut records = fixture();
    records.iter_mut().find(|r| r.kind == "task").unwrap().data["score_frozen_at"] =
        json!("2026-10-02T00:00:00Z");
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("score frozen after")
    );
}

#[test]
fn implementation_children_share_the_requirement_clock() {
    let mut records = fixture();
    let mut child = records.iter().find(|r| r.kind == "task").unwrap().clone();
    child.id = "child".into();
    child.data["parent_task_id"] = json!("t");
    child.data["count_output"] = json!(false);
    child.data["enrolled_at"] = json!("2026-10-02T00:00:00Z");
    records.push(child);
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("share the original requirement enrolment")
    );
}

#[test]
fn uncounted_implementation_child_still_requires_explicit_labour_coverage() {
    let mut records = fixture();
    let mut child = records.iter().find(|r| r.kind == "task").unwrap().clone();
    child.id = "child".into();
    child.data["parent_task_id"] = json!("t");
    child.data["count_output"] = json!(false);
    records.push(child);
    records.push(rec("child-hours", "effort", json!({"task_id":"child","person_id":"person-b","role":"integration","start":"2026-10-02T09:00:00Z","end":"2026-10-02T09:30:00Z","allocation_fraction":1})));
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.cohorts[0].counted_output_tasks, 1);
    assert_eq!(report.cohorts[0].total_person_hours, None);
    assert_eq!(report.cohorts[0].total_retained_output, Some(1.0));
    records.push(rec("child-coverage", "coverage", json!({"task_id":"child","domain":"effort","start":"2026-10-01T00:00:00Z","end":"2026-11-30T00:00:00Z","status":"complete"})));
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.cohorts[0].total_person_hours, Some(1.5));
    assert_eq!(report.cohorts[0].acceptance_fraction, Some(1.0));
    assert_eq!(
        report.cohorts[0].retained_output_per_person_hour,
        Some(1.0 / 1.5)
    );
}

#[test]
fn policy_metadata_links_and_defects_preserve_independent_evidence() {
    let mut records = fixture();
    records.iter_mut().find(|r| r.kind == "task").unwrap().data["assigned_policy"] =
        json!("registered-ai-policy-v1");
    records
        .iter_mut()
        .find(|r| r.kind == "assessment")
        .unwrap()
        .data["reviewer_id"] = json!("reviewer-pseudonym");
    records.push(rec(
        "source",
        "source_record",
        json!({"provider":"synthetic", "note":"A reported defect"}),
    ));
    records.push(rec("link", "task_link", json!({"task_id":"t","relation":"observed_by","observed_at":"2026-10-01T08:00:00Z","source_record_id":"source","method":"human-confirmed","ambiguity":"None within this synthetic fixture"})));
    records.push(rec("defect", "defect", json!({"task_id":"t","detected_at":"2026-10-01T08:00:00Z","severity":"high","evidence_ids":["source"],"repair_effort_ids":["hours"]})));
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(
        report.tasks[0].assigned_policy.as_deref(),
        Some("registered-ai-policy-v1")
    );
    assert_eq!(report.tasks[0].task_link_ids, vec!["link"]);
    assert_eq!(report.tasks[0].defects[0].severity, "high");
    assert_eq!(report.tasks[0].person_hours, Some(1.0));
    records.iter_mut().find(|r| r.id == "link").unwrap().data["source_record_id"] =
        json!("unknown");
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("unknown source_record")
    );
}

#[test]
fn defect_repair_references_cannot_point_at_an_unrelated_task() {
    let mut records = fixture();
    records.push(rec("source", "source_record", json!({})));
    records.push(rec("defect", "defect", json!({"task_id":"t","detected_at":"2026-10-01T08:00:00Z","severity":"unclassified","evidence_ids":["source"],"repair_effort_ids":["nonexistent"]})));
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("unknown repair effort")
    );
}

#[test]
fn scope_change_default_excludes_output_and_keeps_labour() {
    let mut records = fixture();
    records.push(rec("scope-change", "task_scope_change", json!({"task_id":"t","occurred_at":"2026-10-02T00:00:00Z","previous_scope_version":"1","new_scope_version":"2","reason":"Materially expanded requested behaviour"})));
    let report = analyse(&records, final_cut()).unwrap();
    assert_eq!(report.tasks[0].retained_output, Some(0.0));
    assert_eq!(report.tasks[0].person_hours, Some(1.0));
    assert_eq!(report.tasks[0].accepted_by_deadline, None);
    assert_eq!(report.tasks[0].scope_treatment, "excluded_from_output");
}

#[test]
fn baseline_scope_rule_requires_explicit_original_scope_measurements() {
    let mut records = fixture();
    records[0].data["scope_change_rule"] = json!("baseline_only");
    records.push(rec("scope-change", "task_scope_change", json!({"task_id":"t","occurred_at":"2026-10-02T00:00:00Z","new_scope_version":"2","reason":"Added functionality"})));
    assert_eq!(
        analyse(&records, final_cut()).unwrap().tasks[0].retained_output,
        None
    );
    records
        .iter_mut()
        .find(|r| r.id == "accepted")
        .unwrap()
        .data["scope_version"] = json!("1");
    assert_eq!(
        analyse(&records, final_cut()).unwrap().tasks[0].retained_output,
        None
    );
    records.iter_mut().find(|r| r.id == "audit").unwrap().data["scope_version"] = json!("1");
    assert_eq!(
        analyse(&records, final_cut()).unwrap().tasks[0].retained_output,
        Some(1.0)
    );
}

fn correction(
    id: &str,
    target_id: &str,
    replacement: Value,
    at: &str,
    previous: Option<&str>,
) -> Evidence {
    rec(
        id,
        "correction",
        json!({"target_id":target_id,"replacement_data":replacement,"corrected_at":at,"reason":"Researcher resolved a documented missing or inaccurate record","supersedes_correction_id":previous}),
    )
}

#[test]
fn corrections_resolve_nonresponse_without_mutating_raw_history() {
    let mut records = fixture();
    let original_observed = records
        .iter()
        .find(|r| r.id == "hours")
        .unwrap()
        .data
        .clone();
    let effort = &mut records.iter_mut().find(|r| r.id == "hours").unwrap().data;
    effort["allocation_fraction"] = Value::Null;
    effort["response_status"] = json!("missing");
    effort["missing_reason"] = json!("Awaiting response");
    let raw_missing = records.iter().find(|r| r.id == "hours").unwrap().clone();
    records.push(correction(
        "fix-hours",
        "hours",
        original_observed,
        "2026-12-02T00:00:00Z",
        None,
    ));
    validate_records(&records).unwrap();
    assert_eq!(
        analyse(&records, final_cut()).unwrap().cohorts[0].total_person_hours,
        None
    );
    let report = analyse(&records, at("2026-12-03T00:00:00Z")).unwrap();
    assert_eq!(report.cohorts[0].total_person_hours, Some(1.0));
    assert_eq!(report.corrections[0].id, "fix-hours");
    assert_eq!(report.corrections[0].target_id, "hours");
    assert!(report.corrections[0].is_current);
    assert_eq!(
        records.iter().find(|r| r.id == "hours").unwrap(),
        &raw_missing
    );
    assert_eq!(report.record_count, records.len());
}

#[test]
fn coverage_correction_changes_partial_frame_to_complete() {
    let mut records = fixture();
    let complete = records.iter().find(|r| r.id == "hc").unwrap().data.clone();
    let partial = &mut records.iter_mut().find(|r| r.id == "hc").unwrap().data;
    partial["status"] = json!("partial");
    partial["reason"] = json!("Awaiting reconciliation");
    records.push(correction(
        "fix-coverage",
        "hc",
        complete,
        "2026-12-02T00:00:00Z",
        None,
    ));
    assert_eq!(
        analyse(&records, final_cut()).unwrap().tasks[0].person_hours,
        None
    );
    assert_eq!(
        analyse(&records, at("2026-12-03T00:00:00Z")).unwrap().tasks[0].person_hours,
        Some(1.0)
    );
}

#[test]
fn assessment_correction_chain_is_explicit_and_as_of_sensitive() {
    let mut records = fixture();
    let mut replacement = records
        .iter()
        .find(|r| r.id == "audit")
        .unwrap()
        .data
        .clone();
    replacement["retained_fraction"] = json!(0.75);
    records.push(correction(
        "audit-fix-1",
        "audit",
        replacement.clone(),
        "2026-12-02T00:00:00Z",
        None,
    ));
    replacement["retained_fraction"] = json!(0.5);
    records.push(correction(
        "audit-fix-2",
        "audit",
        replacement,
        "2026-12-04T00:00:00Z",
        Some("audit-fix-1"),
    ));
    assert_eq!(
        analyse(&records, final_cut()).unwrap().tasks[0].retained_output,
        Some(1.0)
    );
    assert_eq!(
        analyse(&records, at("2026-12-03T00:00:00Z")).unwrap().tasks[0].retained_output,
        Some(0.75)
    );
    let report = analyse(&records, at("2026-12-05T00:00:00Z")).unwrap();
    assert_eq!(report.tasks[0].retained_output, Some(0.5));
    assert_eq!(report.corrections.len(), 2);
    assert!(!report.corrections[0].is_current);
    assert!(report.corrections[1].is_current);
    let effective = effective_records(&records, at("2026-12-05T00:00:00Z")).unwrap();
    assert!(effective.iter().all(|r| r.kind != "correction"));
    assert_eq!(
        effective.iter().find(|r| r.id == "audit").unwrap().kind,
        "assessment"
    );
}

#[test]
fn correction_branches_cycles_and_nonstudy_targets_are_rejected() {
    let mut records = fixture();
    let replacement = records
        .iter()
        .find(|r| r.id == "hours")
        .unwrap()
        .data
        .clone();
    records.push(correction(
        "fix-1",
        "hours",
        replacement.clone(),
        "2026-12-02T00:00:00Z",
        None,
    ));
    records.push(correction(
        "fix-2",
        "hours",
        replacement.clone(),
        "2026-12-03T00:00:00Z",
        None,
    ));
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("branches")
    );
    records.last_mut().unwrap().data["supersedes_correction_id"] = json!("fix-1");
    validate_records(&records).unwrap();
    records.iter_mut().find(|r| r.id == "fix-1").unwrap().data["supersedes_correction_id"] =
        json!("fix-2");
    assert!(validate_records(&records).is_err());
    assert!(
        validate_records(&[
            rec("source", "source_record", json!({})),
            correction("invalid", "source", json!({}), "2026-12-02T00:00:00Z", None)
        ])
        .is_err()
    );
    assert!(
        validate_records(&[correction(
            "unknown",
            "does-not-exist",
            replacement,
            "2026-12-02T00:00:00Z",
            None
        )])
        .is_err()
    );
}

#[test]
fn corrected_effective_study_still_obeys_allocation_and_reference_constraints() {
    let mut records = fixture();
    let mut replacement = records
        .iter()
        .find(|r| r.id == "hours")
        .unwrap()
        .data
        .clone();
    replacement["allocation_fraction"] = json!(1.5);
    records.push(correction(
        "invalid-allocation",
        "hours",
        replacement,
        "2026-12-02T00:00:00Z",
        None,
    ));
    assert!(validate_records(&records).is_err());
}

#[test]
fn later_correction_cannot_hide_invalid_historical_replacement() {
    let mut records = fixture();
    let valid = records
        .iter()
        .find(|r| r.id == "hours")
        .unwrap()
        .data
        .clone();
    let mut invalid = valid.clone();
    invalid["allocation_fraction"] = json!(1.5);
    records.push(correction(
        "invalid-first",
        "hours",
        invalid,
        "2026-12-02T00:00:00Z",
        None,
    ));
    records.push(correction(
        "valid-second",
        "hours",
        valid.clone(),
        "2026-12-03T00:00:00Z",
        Some("invalid-first"),
    ));
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("invalid corrected study graph")
    );
    let first = records
        .iter_mut()
        .find(|r| r.id == "invalid-first")
        .unwrap();
    first.data["replacement_data"] = valid;
    first.data["replacement_data"]["misspelt_allocation"] = json!(1.0);
    assert!(validate_records(&records).is_err());
    assert!(
        serde_json::from_value::<Evidence>(
            json!({"id":"x","kind":"source_record","data":{},"misspelt_kind":"effort"})
        )
        .is_err()
    );
}

#[test]
fn stored_corrections_preserve_raw_payloads_and_actual_observation_cuts() {
    use observatory::{snapshot::Snapshot, store::Store};
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let mut records = fixture();
    let observed = records
        .iter()
        .find(|r| r.id == "hours")
        .unwrap()
        .data
        .clone();
    let missing = &mut records.iter_mut().find(|r| r.id == "hours").unwrap().data;
    missing["allocation_fraction"] = Value::Null;
    missing["response_status"] = json!("missing");
    missing["missing_reason"] = json!("Awaiting clarification");
    let original = records.iter().find(|r| r.id == "hours").unwrap().clone();
    let hash = store
        .retain_bytes(&serde_json::to_vec(&records).unwrap())
        .unwrap();
    store
        .insert_batch(&records, "synthetic-study", final_cut(), &hash)
        .unwrap();
    let amendment = correction(
        "later-response",
        "hours",
        observed,
        "2026-12-02T00:00:00Z",
        None,
    );
    let correction_hash = store
        .retain_bytes(&serde_json::to_vec(&amendment).unwrap())
        .unwrap();
    store
        .insert_batch(
            std::slice::from_ref(&amendment),
            "synthetic-study",
            at("2026-12-03T00:00:00Z"),
            &correction_hash,
        )
        .unwrap();
    let retained = store.records(None).unwrap();
    assert_eq!(
        retained.iter().find(|r| r.id == "hours").unwrap(),
        &original
    );
    assert_eq!(
        retained.iter().find(|r| r.id == "later-response").unwrap(),
        &amendment
    );
    assert_eq!(store.verify().unwrap(), records.len() + 1);
    let old_cut = Snapshot::freeze(&store, at("2026-12-02T12:00:00Z")).unwrap();
    assert_eq!(
        analyse(&old_cut.evidence(), old_cut.as_of).unwrap().cohorts[0].total_person_hours,
        None
    );
    let new_cut = Snapshot::freeze(&store, at("2026-12-04T00:00:00Z")).unwrap();
    assert_eq!(
        analyse(&new_cut.evidence(), new_cut.as_of).unwrap().cohorts[0].total_person_hours,
        Some(1.0)
    );
}
