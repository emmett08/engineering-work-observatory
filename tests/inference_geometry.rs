//! Check the physical frame premises independently of statistical estimator arithmetic.
use chrono::{DateTime, Duration, Utc};
use observatory::inference::{
    LabourEstimateInputV2, LabourObservation, SamplingDesign, VarianceStatus, estimate_labour,
    estimate_labour_v2,
};
use serde_json::json;

fn example() -> LabourEstimateInputV2 {
    serde_json::from_str(include_str!("../examples/model-assisted-labour-v2.json")).unwrap()
}

fn at(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

fn single(start: &str, end: &str, person_hours: f64) -> LabourEstimateInputV2 {
    let mut input = example();
    input.estimation.units.truncate(1);
    input.estimation.units[0].inclusion_probability = 1.0;
    input.estimation.units[0].observation = LabourObservation::Observed { person_hours };
    input.estimation.design = SamplingDesign::SimpleRandomWithoutReplacement { sample_size: 1 };
    input.frames.truncate(1);
    input.frames[0].start = at(start);
    input.frames[0].end = at(end);
    input.geometry.truncate(1);
    input.geometry[0].start = at(start);
    input.geometry[0].end = at(end);
    input
}

fn rejects(input: &LabourEstimateInputV2, message: &str) {
    let error = estimate_labour_v2(input).unwrap_err().to_string();
    assert!(error.contains(message), "{error}");
}

#[test]
fn adjacent_units_and_concurrent_different_people_preserve_the_estimator() {
    let mut input = example();
    input.geometry.reverse();
    let baseline = estimate_labour(&input.estimation).unwrap();
    let report = estimate_labour_v2(&input).unwrap();
    assert_eq!(report.schema, "observatory.labour-estimate/2");
    assert_eq!(report.frames.len(), 2);
    assert_eq!(report.geometry.len(), 4);
    assert_eq!(report.geometry[0].unit_id, input.geometry[0].unit_id);
    assert_eq!(
        serde_json::to_value(report.estimation).unwrap(),
        serde_json::to_value(baseline).unwrap()
    );
}

#[test]
fn overlap_is_rejected_even_with_different_unit_ids() {
    let mut input = example();
    input.geometry[1].start -= Duration::nanoseconds(1);
    rejects(&input, "overlapping");
    input = example();
    input.geometry[3].start = input.geometry[2].start;
    input.geometry[3].end = input.geometry[2].end;
    rejects(&input, "overlapping");
}

#[test]
fn interior_and_boundary_gaps_reject() {
    let mut input = example();
    input.geometry[3].start += Duration::nanoseconds(1);
    rejects(&input, "gap");
    input = example();
    input.frames[0].start -= Duration::nanoseconds(1);
    rejects(&input, "gap");
    input = example();
    input.frames[1].end += Duration::nanoseconds(1);
    rejects(&input, "gap");
}

#[test]
fn unknown_duplicate_or_missing_geometry_rejects() {
    let mut input = example();
    input.geometry[0].unit_id = "unknown-unit".into();
    rejects(&input, "unknown unit");
    input = example();
    input.geometry.push(input.geometry[0].clone());
    rejects(&input, "duplicate geometry");
    input = example();
    input.geometry.pop();
    rejects(&input, "missing");
}

#[test]
fn invalid_person_references_and_person_frames_reject() {
    let mut input = example();
    input.geometry[0].person_id = "unknown-person".into();
    rejects(&input, "unknown person");
    input = example();
    input.frames[0].person_id = " ".into();
    rejects(&input, "nonempty");
    input = example();
    input.frames.push(input.frames[0].clone());
    rejects(&input, "duplicate person frame");
    input = example();
    let mut unused = input.frames[0].clone();
    unused.person_id = "person-without-units".into();
    input.frames.push(unused);
    rejects(&input, "missing interval coverage");
    input = example();
    input.frames.clear();
    rejects(&input, "nonempty");
}

#[test]
fn zero_negative_and_out_of_frame_intervals_reject() {
    let mut input = example();
    input.geometry[0].end = input.geometry[0].start;
    rejects(&input, "positive duration");
    input = example();
    input.geometry[0].end = input.geometry[0].start - Duration::nanoseconds(1);
    rejects(&input, "positive duration");
    input = example();
    input.geometry[0].start -= Duration::nanoseconds(1);
    rejects(&input, "outside");
    input = example();
    input.geometry[3].end += Duration::nanoseconds(1);
    rejects(&input, "outside");
    input = example();
    input.frames[0].end = input.frames[0].start;
    rejects(&input, "positive duration");
}

#[test]
fn observed_capacity_is_checked_without_rounding_tolerance() {
    let mut input = example();
    input.estimation.units[0].observation = LabourObservation::Observed {
        person_hours: 1.0_f64.next_up(),
    };
    rejects(&input, "exceeds interval capacity");
    let fraction = single("2026-10-30T09:00:00Z", "2026-10-30T10:30:00Z", 1.5);
    assert!(estimate_labour_v2(&fraction).is_ok());

    // 1/3_600_000_000_000 hours rounds up in f64. Comparing against that
    // rounded duration would accept a value greater than one nanosecond.
    let rounded_up = 1.0 / 3_600_000_000_000_f64;
    let mut nanosecond = single(
        "2026-10-30T09:00:00Z",
        "2026-10-30T09:00:00.000000001Z",
        rounded_up,
    );
    rejects(&nanosecond, "exceeds interval capacity");
    nanosecond.estimation.units[0].observation = LabourObservation::Observed {
        person_hours: rounded_up.next_down(),
    };
    assert!(estimate_labour_v2(&nanosecond).is_ok());
}

#[test]
fn exact_elapsed_conversion_handles_long_and_fractional_windows() {
    let long = single("1700-01-01T00:00:00Z", "2300-01-01T00:00:00Z", 1.0);
    assert!(estimate_labour_v2(&long).is_ok());
    let fraction = single("2026-10-30T09:00:00Z", "2026-10-30T09:00:00.5Z", 0.0);
    let report = estimate_labour_v2(&fraction).unwrap();
    assert_eq!(
        report.estimation.variance.status,
        VarianceStatus::KnownZeroCensus
    );
}

#[test]
fn predictions_remain_unbounded_and_signed_corrections_are_preserved() {
    let mut input = example();
    input.estimation.design = SamplingDesign::General {};
    input.estimation.units[0].predicted_person_hours = 100.0;
    input.estimation.units[0].observation = LabourObservation::Observed { person_hours: 0.0 };
    let report = estimate_labour_v2(&input).unwrap();
    assert!(report.estimation.difference_total_person_hours < 0.0);
    input.estimation.units[0].observation = LabourObservation::Nonresponse {
        reason: "Selected participant did not respond".into(),
    };
    rejects(&input, "nonresponse");
}

#[test]
fn schema_and_unknown_fields_are_rejected_at_each_boundary() {
    let mut input = example();
    input.schema = "observatory.labour-estimate-input/3".into();
    rejects(&input, "schema");
    input = example();
    input.estimation.schema = "observatory.labour-estimate-input/2".into();
    rejects(&input, "schema");
    let value = serde_json::to_value(example()).unwrap();
    for (scope, field) in [
        ("frames", "person"),
        ("geometry", "interval"),
        ("estimation", "frames"),
    ] {
        let mut bad = value.clone();
        if scope == "estimation" {
            bad[scope][field] = json!([]);
        } else {
            bad[scope][0][field] = json!("unexpected");
        }
        assert!(serde_json::from_value::<LabourEstimateInputV2>(bad).is_err());
    }
}
