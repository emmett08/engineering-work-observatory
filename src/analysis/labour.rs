//! All-role human labour and Horvitz–Thompson sample contributions.
//! Completeness is an explicit frame attestation, never inferred from event count.
use super::{BoundaryEffortBounds, LabourBounds, bounds};
use crate::model::*;
use anyhow::{Result, ensure};
use chrono::{DateTime, Utc};
use std::collections::BTreeSet;

pub(super) struct LabourAccounting {
    pub person_hours: Option<f64>,
    pub person_hours_bounds: Option<LabourBounds>,
    pub boundary_effort_bounds: Vec<BoundaryEffortBounds>,
    pub recorded_person_hours: f64,
    pub contains_sampled_estimates: bool,
    pub sampled_intervals: usize,
    pub nonresponse_intervals: usize,
    pub evidence_ids: Vec<String>,
    pub issues: Vec<String>,
}

pub(super) fn account(
    d: &Dataset,
    id: &str,
    task: &Task,
    labour_end: DateTime<Utc>,
    as_of: DateTime<Utc>,
) -> Result<LabourAccounting> {
    let mut issues = Vec::new();
    let effort: Vec<_> = d
        .effort
        .iter()
        .filter(|(_, e)| {
            e.task_id == *id && e.start < labour_end && e.end > task.enrolled_at && e.end <= as_of
        })
        .collect();
    let sampled_intervals = effort
        .iter()
        .filter(|(_, e)| e.method == EffortMethod::Sampled)
        .map(|(_, e)| (&e.person_id, e.start, e.end))
        .collect::<BTreeSet<_>>()
        .len();
    let nonresponse_intervals = effort
        .iter()
        .filter(|(_, e)| e.response_status == ObservationStatus::Missing)
        .map(|(_, e)| (&e.person_id, e.start, e.end))
        .collect::<BTreeSet<_>>()
        .len();
    let mut contribution = 0.0;
    let mut direct_bounds = Vec::new();
    let mut boundary_effort_bounds = Vec::new();
    let mut method_matches = true;
    let mut boundary_ok = true;
    for (effort_id, e) in &effort {
        let left = e.start.max(task.enrolled_at);
        let right = e.end.min(labour_end);
        let crosses_boundary = e.start < task.enrolled_at || e.end > labour_end;
        // Coverage qualifies a total, not the conditional bound from one datum.
        // Check this also for observations whose point contribution is unknown.
        if !super::coverage::effort_covered(d, id, &e.method, left, right, as_of) {
            method_matches = false;
        }
        if e.method == EffortMethod::Direct
            && let Some(allocation) = e.allocation_fraction
        {
            let bound =
                bounds::direct_interval(e.start, e.end, allocation, task.enrolled_at, labour_end);
            if crosses_boundary {
                boundary_effort_bounds.push(BoundaryEffortBounds {
                    effort_id: effort_id.clone(),
                    overlap_start: left,
                    overlap_end: right,
                    bounds: bound.clone(),
                });
            }
            direct_bounds.push(bound);
        }
        // An average of exactly zero or one identifies a direct partial integral
        // under the attention bound. Interior averages do not; sampled intervals
        // must also stay within the declared sampling frame.
        let identified_direct = e.method == EffortMethod::Direct
            && e.allocation_fraction
                .is_some_and(|allocation| allocation == 0.0 || allocation == 1.0);
        if crosses_boundary && !identified_direct {
            boundary_ok = false;
            continue;
        }
        if let Some(allocation) = e.allocation_fraction {
            let weight = if e.method == EffortMethod::Sampled {
                1.0 / e.inclusion_probability.expect("validated probability")
            } else {
                1.0
            };
            contribution += super::elapsed_hours(left, right) * allocation * weight;
        }
    }
    ensure!(contribution.is_finite(), "non-finite effort total for {id}");
    let frame_complete = super::coverage::covered(
        d,
        id,
        CoverageDomain::Effort,
        task.enrolled_at,
        labour_end,
        as_of,
    ) && nonresponse_intervals == 0
        && method_matches;
    let sampled_frame = d.coverage.iter().any(|(_, c)| {
        c.task_id == *id
            && c.domain == CoverageDomain::Effort
            && c.method == EffortMethod::Sampled
            && c.start < labour_end
            && c.end > task.enrolled_at
            && c.end <= as_of
    });
    // A sampled frame requires an actual sample, including explicitly observed zeros.
    let sample_present = !sampled_frame || sampled_intervals > 0;
    // An unfinished observation cannot be promoted to an observed zero even if
    // its frame attestation is already available at the requested evidence cut.
    let unfinished_interval = d.effort.iter().any(|(_, e)| {
        e.task_id == id && e.start < labour_end && e.end > task.enrolled_at && e.end > as_of
    });
    let person_hours = if frame_complete && boundary_ok && sample_present && !unfinished_interval {
        Some(contribution)
    } else {
        None
    };
    let person_hours_bounds =
        if frame_complete && !sampled_frame && sampled_intervals == 0 && !unfinished_interval {
            let total = LabourBounds::sum(direct_bounds.iter());
            ensure!(
                total.is_finite(),
                "non-finite direct effort bounds for {id}"
            );
            Some(total)
        } else {
            None
        };
    if as_of < labour_end {
        issues.push("fixed enrolment labour horizon is not complete".into());
    }
    if unfinished_interval {
        issues.push("an intersecting effort interval ends after the evidence cut; its unobserved contribution is not zero".into());
    }
    if person_hours.is_none() {
        issues.push("labour total unavailable: require full declared frame, method-matched observations, and responses".into());
    }
    if nonresponse_intervals > 0 {
        issues.push("sample/effort nonresponse is not zero allocation".into());
    }
    if !boundary_ok {
        issues.push("effort interval crosses the fixed enrolment labour boundary without an identified within-boundary contribution: supply separately observed split intervals under the declared measurement design".into());
    }

    Ok(LabourAccounting {
        person_hours,
        person_hours_bounds,
        boundary_effort_bounds,
        recorded_person_hours: contribution,
        contains_sampled_estimates: sampled_frame || sampled_intervals > 0,
        sampled_intervals,
        nonresponse_intervals,
        evidence_ids: effort.iter().map(|(id, _)| id.clone()).collect(),
        issues,
    })
}
