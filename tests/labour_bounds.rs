use chrono::{DateTime, Utc};
use observatory::analysis::{LabourBounds, LabourBoundsMethod, ProductivityBoundsMethod, analyse};
use observatory::model::{Evidence, validate_records};
use serde_json::{Value, json};

fn at(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
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
            json!({"name":"Bounds fixture", "frozen_at":"2026-09-30T00:00:00Z", "acceptance_days":1,"follow_up_days":0}),
        ),
        rec(
            "t",
            "task",
            json!({"protocol_id":"p", "requirement":"Bound measured labour", "task_class":"test", "scope_version":"1", "enrolled_at":"2026-10-01T00:00:00Z", "score_frozen_at":"2026-10-01T00:00:00Z", "frozen_score":1}),
        ),
        rec(
            "frame",
            "coverage",
            json!({"task_id":"t", "domain":"effort", "start":"2026-10-01T00:00:00Z", "end":"2026-10-02T00:00:00Z", "status":"complete"}),
        ),
        rec(
            "crossing",
            "effort",
            json!({"task_id":"t", "person_id":"a", "role":"implementation", "start":"2026-10-01T23:00:00Z", "end":"2026-10-02T01:00:00Z", "allocation_fraction":0.75}),
        ),
    ]
}

fn fixture_with_output(retained_fraction: f64) -> Vec<Evidence> {
    let mut records = fixture();
    records.push(rec(
        "accepted",
        "lifecycle",
        json!({"task_id":"t", "event":"accepted", "occurred_at":"2026-10-01T12:00:00Z"}),
    ));
    records.push(rec("quality", "assessment", json!({"task_id":"t", "phase":"follow_up", "assessed_at":"2026-10-01T12:00:00Z", "status":"observed", "retained_fraction":retained_fraction, "rubric_version":"requirements-v1"})));
    records
}

fn data<'a>(records: &'a mut [Evidence], id: &str) -> &'a mut Value {
    &mut records
        .iter_mut()
        .find(|record| record.id == id)
        .unwrap()
        .data
}

fn cut() -> DateTime<Utc> {
    at("2026-10-03T00:00:00Z")
}

fn assert_range(actual: &LabourBounds, lower: f64, upper: f64) {
    assert_eq!(actual.lower_person_hours, lower);
    assert_eq!(actual.upper_person_hours, upper);
}

#[test]
fn boundary_mass_has_sharp_marginal_bounds_without_an_exact_total() {
    let report = analyse(&fixture(), cut()).unwrap();
    let task = &report.tasks[0];
    assert_eq!(task.person_hours, None);
    assert_eq!(task.recorded_person_hours, 0.0);
    let marginal = &task.boundary_effort_bounds[0];
    assert_eq!(marginal.effort_id, "crossing");
    assert_eq!(marginal.overlap_start, at("2026-10-01T23:00:00Z"));
    assert_eq!(marginal.overlap_end, at("2026-10-02T00:00:00Z"));
    assert_range(&marginal.bounds, 0.5, 1.0);
    assert_eq!(
        marginal.bounds.method,
        LabourBoundsMethod::SharpFromSingleDirectInterval
    );
    let total = task.person_hours_bounds.as_ref().unwrap();
    assert_range(total, 0.5, 1.0);
    assert_eq!(
        total.method,
        LabourBoundsMethod::ConservativeSumOfDirectIntervalBounds
    );
    assert_range(
        report.cohorts[0]
            .total_person_hours_bounds
            .as_ref()
            .unwrap(),
        0.5,
        1.0,
    );
    assert_eq!(report.cohorts[0].total_person_hours, None);
    assert_eq!(report.cohorts[0].retained_output_per_person_hour, None);
    let encoded = serde_json::to_value(report).unwrap();
    assert_eq!(
        encoded["tasks"][0]["boundary_effort_bounds"][0]["bounds"]["method"],
        "sharp_from_single_direct_interval"
    );
}

#[test]
fn boundary_bounds_combine_with_identified_mass_but_do_not_replace_it() {
    let mut records = fixture();
    records.push(rec("known", "effort", json!({"task_id":"t", "person_id":"a", "role":"review", "start":"2026-10-01T12:00:00Z", "end":"2026-10-01T14:00:00Z", "allocation_fraction":0.5})));
    let report = analyse(&records, cut()).unwrap();
    assert_eq!(report.tasks[0].recorded_person_hours, 1.0);
    assert_eq!(report.tasks[0].person_hours, None);
    assert_range(
        report.tasks[0].person_hours_bounds.as_ref().unwrap(),
        1.5,
        2.0,
    );
}

#[test]
fn exact_direct_endpoints_and_whole_observations_collapse_bounds() {
    for (allocation, expected) in [(0.0, 0.0), (1.0, 1.0)] {
        let mut records = fixture();
        data(&mut records, "crossing")["allocation_fraction"] = json!(allocation);
        let report = analyse(&records, cut()).unwrap();
        assert_eq!(report.tasks[0].person_hours, Some(expected));
        assert_range(
            report.tasks[0].person_hours_bounds.as_ref().unwrap(),
            expected,
            expected,
        );
    }
    let mut records = fixture();
    data(&mut records, "crossing")["end"] = json!("2026-10-02T00:00:00Z");
    let report = analyse(&records, cut()).unwrap();
    assert_eq!(report.tasks[0].person_hours, Some(0.75));
    assert_range(
        report.tasks[0].person_hours_bounds.as_ref().unwrap(),
        0.75,
        0.75,
    );
    assert!(report.tasks[0].boundary_effort_bounds.is_empty());
}

#[test]
fn a_complete_empty_direct_frame_is_zero_and_outside_work_is_excluded() {
    let mut records = fixture();
    data(&mut records, "crossing")["start"] = json!("2026-10-02T00:00:00Z");
    let report = analyse(&records, cut()).unwrap();
    assert_eq!(report.tasks[0].person_hours, Some(0.0));
    assert_range(
        report.tasks[0].person_hours_bounds.as_ref().unwrap(),
        0.0,
        0.0,
    );
    assert!(report.tasks[0].boundary_effort_bounds.is_empty());
}

#[test]
fn incomplete_frames_never_acquire_finite_complete_total_bounds() {
    for problem in [
        "missing",
        "partial",
        "gap",
        "mismatch",
        "future_attestation",
        "future_cut",
    ] {
        let mut records = fixture();
        match problem {
            "missing" => records.retain(|record| record.id != "frame"),
            "partial" => {
                data(&mut records, "frame")["status"] = json!("partial");
                data(&mut records, "frame")["reason"] = json!("Unobserved period");
            }
            "gap" => data(&mut records, "frame")["end"] = json!("2026-10-01T23:30:00Z"),
            "mismatch" => data(&mut records, "frame")["method"] = json!("sampled"),
            "future_attestation" => {
                data(&mut records, "frame")["end"] = json!("2026-10-04T00:00:00Z")
            }
            "future_cut" => (),
            _ => unreachable!(),
        }
        let as_of = if problem == "future_cut" {
            at("2026-10-01T23:30:00Z")
        } else {
            cut()
        };
        let report = analyse(&records, as_of).unwrap();
        assert!(report.tasks[0].person_hours_bounds.is_none(), "{problem}");
        assert!(
            report.cohorts[0].total_person_hours_bounds.is_none(),
            "{problem}"
        );
        let encoded = serde_json::to_value(&report).unwrap();
        assert!(encoded["tasks"][0].get("person_hours_bounds").is_none());
        if problem != "future_cut" {
            assert_range(&report.tasks[0].boundary_effort_bounds[0].bounds, 0.5, 1.0);
        }
    }
}

#[test]
fn nonresponse_and_unfinished_observations_do_not_bound_complete_labour() {
    let mut records = fixture();
    let missing = data(&mut records, "crossing");
    missing["allocation_fraction"] = Value::Null;
    missing["response_status"] = json!("missing");
    missing["missing_reason"] = json!("No diary response");
    let report = analyse(&records, cut()).unwrap();
    assert_eq!(report.tasks[0].person_hours, None);
    assert!(report.tasks[0].person_hours_bounds.is_none());
    assert!(report.tasks[0].boundary_effort_bounds.is_empty());

    let report = analyse(&fixture(), at("2026-10-02T00:30:00Z")).unwrap();
    assert_eq!(report.tasks[0].person_hours, None);
    assert_eq!(report.tasks[0].recorded_person_hours, 0.0);
    assert!(report.tasks[0].person_hours_bounds.is_none());
    assert!(report.tasks[0].boundary_effort_bounds.is_empty());
    assert!(
        report.tasks[0]
            .issues
            .iter()
            .any(|issue| issue.contains("ends after the evidence cut"))
    );
}

#[test]
fn sampled_points_and_mixed_frames_do_not_become_exact_population_mass() {
    for probability in [0.25, 1.0] {
        let mut records = fixture();
        let hours = data(&mut records, "crossing");
        hours["method"] = json!("sampled");
        hours["inclusion_probability"] = json!(probability);
        hours["end"] = json!("2026-10-02T00:00:00Z");
        data(&mut records, "frame")["method"] = json!("sampled");
        let report = analyse(&records, cut()).unwrap();
        assert_eq!(report.tasks[0].person_hours, Some(0.75 / probability));
        assert!(report.tasks[0].person_hours_bounds.is_none());
        assert!(report.tasks[0].boundary_effort_bounds.is_empty());

        // Directly observed boundary mass remains bounded, but cannot close a
        // population total whose other contributions are sample estimates.
        data(&mut records, "frame")["end"] = json!("2026-10-01T23:30:00Z");
        data(&mut records, "crossing")["end"] = json!("2026-10-01T23:30:00Z");
        records.push(rec("direct-frame", "coverage", json!({"task_id":"t", "domain":"effort", "start":"2026-10-01T23:30:00Z", "end":"2026-10-02T00:00:00Z", "status":"complete"})));
        records.push(rec("direct", "effort", json!({"task_id":"t", "person_id":"a", "role":"review", "start":"2026-10-01T23:30:00Z", "end":"2026-10-02T00:30:00Z", "allocation_fraction":0.5})));
        let report = analyse(&records, cut()).unwrap();
        assert_eq!(report.tasks[0].person_hours, None);
        assert!(report.tasks[0].person_hours_bounds.is_none());
        assert!(report.cohorts[0].total_person_hours_bounds.is_none());
        assert_range(&report.tasks[0].boundary_effort_bounds[0].bounds, 0.0, 0.5);
    }
}

#[test]
fn shared_person_capacity_makes_marginal_sum_conservative_not_jointly_sharp() {
    let mut records = fixture();
    data(&mut records, "crossing")["allocation_fraction"] = json!(0.5);
    let mut review = records
        .iter()
        .find(|record| record.id == "crossing")
        .unwrap()
        .clone();
    review.id = "review".into();
    review.data["role"] = json!("review");
    records.push(review);
    let report = analyse(&records, cut()).unwrap();
    // Each role can put its whole hour inside in isolation. Together they can
    // occupy only one person-hour there; the marginal sum is an outer enclosure.
    assert_range(
        report.tasks[0].person_hours_bounds.as_ref().unwrap(),
        0.0,
        2.0,
    );
    assert_eq!(
        report.tasks[0].person_hours_bounds.as_ref().unwrap().method,
        LabourBoundsMethod::ConservativeSumOfDirectIntervalBounds
    );
    data(&mut records, "review")["role"] = json!("implementation");
    data(&mut records, "review")["start"] = json!("2026-10-01T23:30:00Z");
    assert!(
        validate_records(&records)
            .unwrap_err()
            .to_string()
            .contains("same task/person/role")
    );
}

#[test]
fn subsecond_boundary_mass_and_near_endpoint_fractions_remain_distinct() {
    let mut records = fixture();
    let hours = data(&mut records, "crossing");
    hours["start"] = json!("2026-10-01T23:59:59.999999999Z");
    hours["end"] = json!("2026-10-02T00:00:00.000000001Z");
    hours["allocation_fraction"] = json!(0.5);
    let report = analyse(&records, cut()).unwrap();
    let bounds = report.tasks[0].person_hours_bounds.as_ref().unwrap();
    assert_eq!(bounds.lower_person_hours, 0.0);
    assert_eq!(bounds.upper_person_hours, 1.0e-9 / 3_600.0);
    for allocation in [f64::MIN_POSITIVE, 1.0_f64.next_down()] {
        let mut records = fixture();
        data(&mut records, "crossing")["allocation_fraction"] = json!(allocation);
        let report = analyse(&records, cut()).unwrap();
        let bounds = report.tasks[0].person_hours_bounds.as_ref().unwrap();
        assert_eq!(report.tasks[0].person_hours, None);
        assert!(bounds.upper_person_hours > bounds.lower_person_hours);
    }
}

#[test]
fn cohort_total_requires_every_task_to_have_a_direct_frame() {
    let mut records = fixture();
    let mut other = records
        .iter()
        .find(|record| record.id == "t")
        .unwrap()
        .clone();
    other.id = "other".into();
    records.push(other);
    let report = analyse(&records, cut()).unwrap();
    assert!(report.cohorts[0].total_person_hours_bounds.is_none());
    records.push(rec("other-frame", "coverage", json!({"task_id":"other", "domain":"effort", "start":"2026-10-01T00:00:00Z", "end":"2026-10-02T00:00:00Z", "status":"complete"})));
    let report = analyse(&records, cut()).unwrap();
    assert_range(
        report.cohorts[0]
            .total_person_hours_bounds
            .as_ref()
            .unwrap(),
        0.5,
        1.0,
    );
}

#[test]
fn exported_report_keeps_unknown_point_and_qualified_bounds_visible() {
    use observatory::{report::write_report, snapshot::Snapshot, store::Store};
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(&directory.path().join("state")).unwrap();
    let records = fixture();
    let hash = store
        .retain_bytes(&serde_json::to_vec(&records).unwrap())
        .unwrap();
    store
        .insert_batch(&records, "synthetic bounds test", cut(), &hash)
        .unwrap();
    let snapshot = Snapshot::freeze(&store, cut()).unwrap();
    let output = directory.path().join("report");
    write_report(&snapshot, &output).unwrap();
    let encoded: Value =
        serde_json::from_slice(&std::fs::read(output.join("report.json")).unwrap()).unwrap();
    assert!(encoded["analysis"]["tasks"][0]["person_hours"].is_null());
    let bounds = &encoded["analysis"]["tasks"][0]["person_hours_bounds"];
    assert_eq!(bounds["lower_person_hours"], 0.5);
    assert_eq!(bounds["upper_person_hours"], 1.0);
    assert_eq!(
        bounds["numerical_semantics"],
        "floating_point_approximation_without_outward_rounding"
    );
    let html = std::fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("<td>unknown</td><td>[0.5, 1.0]</td>"));
    assert!(html.contains("not outward rounded"));
    assert!(html.contains("not confidence intervals"));
}

#[test]
fn known_output_and_positive_direct_labour_produce_conditional_productivity_range() {
    let mut records = fixture_with_output(1.0);
    let report = analyse(&records, cut()).unwrap();
    let cohort = &report.cohorts[0];
    assert_eq!(cohort.total_retained_output, Some(1.0));
    assert_eq!(cohort.retained_output_per_person_hour, None);
    assert_eq!(cohort.ratio_status, "unavailable_incomplete_measurement");
    let bounds = cohort
        .retained_output_per_person_hour_bounds
        .as_ref()
        .unwrap();
    assert_eq!(bounds.lower_reference_units_per_person_hour, 1.0);
    assert_eq!(bounds.upper_reference_units_per_person_hour, 2.0);
    assert_eq!(
        bounds.method,
        ProductivityBoundsMethod::ConservativeRatioFromKnownOutputAndDirectLabourBounds
    );
    let encoded = serde_json::to_value(bounds).unwrap();
    assert!(encoded.get("lower_person_hours").is_none());
    assert_eq!(
        encoded["numerical_semantics"],
        "floating_point_approximation_without_outward_rounding"
    );

    data(&mut records, "crossing")["allocation_fraction"] = json!(1.0);
    let report = analyse(&records, cut()).unwrap();
    let cohort = &report.cohorts[0];
    assert_eq!(cohort.retained_output_per_person_hour, Some(1.0));
    assert_eq!(cohort.ratio_status, "descriptive_direct_accounting");
    let bounds = cohort
        .retained_output_per_person_hour_bounds
        .as_ref()
        .unwrap();
    assert_eq!(bounds.lower_reference_units_per_person_hour, 1.0);
    assert_eq!(bounds.upper_reference_units_per_person_hour, 1.0);
}

#[test]
fn known_zero_output_has_zero_productivity_bounds_only_with_positive_labour() {
    let mut records = fixture_with_output(0.0);
    let report = analyse(&records, cut()).unwrap();
    let cohort = &report.cohorts[0];
    assert_eq!(cohort.total_retained_output, Some(0.0));
    assert_eq!(cohort.retained_output_per_person_hour, None);
    let bounds = cohort
        .retained_output_per_person_hour_bounds
        .as_ref()
        .unwrap();
    assert_eq!(bounds.lower_reference_units_per_person_hour, 0.0);
    assert_eq!(bounds.upper_reference_units_per_person_hour, 0.0);

    data(&mut records, "crossing")["allocation_fraction"] = json!(0.5);
    let report = analyse(&records, cut()).unwrap();
    assert!(
        report.cohorts[0]
            .retained_output_per_person_hour_bounds
            .is_none()
    );
}

#[test]
fn productivity_bounds_require_quality_positive_direct_labour_and_counted_tasks() {
    for problem in [
        "zero_lower",
        "unknown_quality",
        "sampled",
        "uncounted",
        "overflow",
    ] {
        let mut records = fixture_with_output(1.0);
        match problem {
            "zero_lower" => data(&mut records, "crossing")["allocation_fraction"] = json!(0.5),
            "unknown_quality" => records.retain(|record| record.id != "quality"),
            "sampled" => {
                let effort = data(&mut records, "crossing");
                effort["method"] = json!("sampled");
                effort["inclusion_probability"] = json!(0.25);
                effort["end"] = json!("2026-10-02T00:00:00Z");
                data(&mut records, "frame")["method"] = json!("sampled");
            }
            "uncounted" => data(&mut records, "t")["count_output"] = json!(false),
            "overflow" => data(&mut records, "t")["frozen_score"] = json!(f64::MAX),
            _ => unreachable!(),
        }
        let report = analyse(&records, cut()).unwrap();
        assert!(
            report.cohorts[0]
                .retained_output_per_person_hour_bounds
                .is_none(),
            "{problem}"
        );
        let encoded = serde_json::to_value(&report.cohorts[0]).unwrap();
        assert!(
            encoded
                .get("retained_output_per_person_hour_bounds")
                .is_none(),
            "{problem}"
        );
        if problem == "sampled" {
            assert_eq!(
                report.cohorts[0].retained_output_per_person_hour,
                Some(1.0 / 3.0)
            );
            assert_eq!(
                report.cohorts[0].ratio_status,
                "descriptive_ratio_using_sampled_labour_estimate"
            );
        }
    }
}
