//! Compute attainable labour ranges under exact interval and shared-capacity constraints.
//!
//! Cell masses exactly represent measurable fractional allocations for the declared
//! streams. Rational input strings preserve observations without floating-point
//! conversion. Infeasibility is a model result; resource exhaustion is an error.
use anyhow::{Result, ensure};
use num_traits::Zero;

mod exact;
mod model;
mod reduction;
mod simplex;

pub use exact::Exact;
pub use model::{
    BoundStatus, ExactProductivityBounds, Extremum, IntervalObservation, LabourBoundInput,
    LabourBoundReport, LabourStream, StreamAllocation, TargetWindow, TimeInterval,
};
use reduction::CellProblem;

/// Return both attainable endpoints and an allocation witnessing each one.
///
/// All times share a calendar-hour axis and all interval masses use person-hours.
/// Observations must supply exact totals or justified deterministic bounds. A
/// probability-sampled estimate is not an exact mass. This calculation establishes
/// compatibility under the declared model, not observation accuracy or causality.
pub fn bound_labour(input: &LabourBoundInput) -> Result<LabourBoundReport> {
    let problem = CellProblem::build(input)?;
    let mut report = LabourBoundReport {
        schema: "observatory.labour-bound/1".into(),
        problem_id: input.problem_id.clone(),
        input: input.clone(),
        status: BoundStatus::Inconsistent,
        numerical_semantics: "exact_rational",
        cells: problem.cells.clone(),
        minimum: None,
        maximum: None,
        productivity: None,
        limitations: vec![
            "Bounds are conditional on declared streams, person identities, interval totals or deterministic observation bounds, and measurable fractional allocations.".into(),
            "Frame completeness, observation accuracy and attribution are not established by input validation. Sampled labour estimates do not become exact interval observations.".into(),
            "No minimum session length, task-interruption rule, smoothness constraint, switching cost, or causal AI effect is inferred.".into(),
            "Supplying known_output requires unit-weight non-overlapping target windows; invalid targets are rejected. A zero lower labour bound leaves productivity unavailable.".into(),
        ],
    };
    let Some(maximum_masses) =
        simplex::maximise(problem.a.clone(), problem.b.clone(), problem.c.clone())?
    else {
        return Ok(report);
    };
    let minimum_masses = simplex::maximise(
        problem.a.clone(),
        problem.b.clone(),
        problem.c.iter().map(|value| -value).collect(),
    )?
    .ok_or_else(|| anyhow::anyhow!("inconsistent feasibility results for the same constraints"))?;
    let minimum_value = problem.verify_witness(input, &minimum_masses)?;
    let maximum_value = problem.verify_witness(input, &maximum_masses)?;
    ensure!(
        minimum_value <= maximum_value,
        "solver returned reversed extrema"
    );
    if let Some(output) = &input.known_output
        && minimum_value > reduction::Rational::zero()
    {
        report.productivity = Some(ExactProductivityBounds {
            lower_reference_units_per_person_hour: Exact(&output.0 / &maximum_value),
            upper_reference_units_per_person_hour: Exact(&output.0 / &minimum_value),
        });
    }
    let witness = |masses: &[reduction::Rational]| -> Vec<StreamAllocation> {
        input
            .streams
            .iter()
            .enumerate()
            .map(|(index, stream)| StreamAllocation {
                stream_id: stream.id.clone(),
                person_hours: masses
                    [index * problem.cells.len()..(index + 1) * problem.cells.len()]
                    .iter()
                    .cloned()
                    .map(Exact)
                    .collect(),
            })
            .collect()
    };
    report.minimum = Some(Extremum {
        value: Exact(minimum_value),
        allocations: witness(&minimum_masses),
    });
    report.maximum = Some(Extremum {
        value: Exact(maximum_value),
        allocations: witness(&maximum_masses),
    });
    report.status = BoundStatus::Feasible;
    Ok(report)
}

#[cfg(test)]
mod model_validation_tests {
    use super::*;
    use serde_json::{Value, json};

    fn valid_input() -> Value {
        json!({
            "schema":"observatory.labour-bound-input/1", "problem_id":"validation",
            "domain":{"start":"0","end":"2"},
            "streams":[{"id":"a","person_id":"p","task_id":"t","role":"review"}],
            "observations":[], "target":[{"stream_id":"a","start":"0","end":"1","weight":"1"}]
        })
    }

    #[test]
    fn retains_the_reproducible_input() {
        let input: LabourBoundInput = serde_json::from_value(valid_input()).unwrap();
        let result = bound_labour(&input).unwrap();
        assert_eq!(
            serde_json::to_value(result.input).unwrap(),
            serde_json::to_value(input).unwrap()
        );
    }

    #[test]
    fn rejects_malformed_domains_unknown_targets_and_missing_schema() {
        for defect in [
            "reversed_domain",
            "zero_domain",
            "outside_domain",
            "unknown_target",
            "missing_schema",
        ] {
            let mut bad = valid_input();
            match defect {
                "reversed_domain" => bad["domain"]["end"] = json!("-1"),
                "zero_domain" => bad["domain"]["end"] = json!("0"),
                "outside_domain" => bad["target"][0]["end"] = json!("3"),
                "unknown_target" => bad["target"][0]["stream_id"] = json!("absent"),
                "missing_schema" => {
                    bad.as_object_mut().unwrap().remove("schema");
                }
                _ => unreachable!(),
            }
            let rejected = match serde_json::from_value::<LabourBoundInput>(bad) {
                Err(_) => true,
                Ok(input) => bound_labour(&input).is_err(),
            };
            assert!(rejected, "accepted {defect}");
        }
    }
}
