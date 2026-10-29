//! All-role human labour and Horvitz–Thompson sample contributions.
//! Completeness is an explicit frame attestation, never inferred from event count.
use crate::model::*;
use anyhow::{Result, ensure};
use chrono::{DateTime, Utc};
use std::collections::BTreeSet;

pub(super) struct LabourAccounting {
    pub person_hours: Option<f64>,
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
    let mut method_matches = true;
    let mut boundary_ok = true;
    for (_, e) in &effort {
        // An average of exactly zero or one identifies a direct partial integral
        // under the attention bound. Interior averages do not; sampled intervals
        // must also stay within the declared sampling frame.
        let identified_direct = e.method == EffortMethod::Direct
            && e.allocation_fraction
                .is_some_and(|allocation| allocation == 0.0 || allocation == 1.0);
        if (e.start < task.enrolled_at || e.end > labour_end) && !identified_direct {
            boundary_ok = false;
            continue;
        }
        let left = e.start.max(task.enrolled_at);
        let right = e.end.min(labour_end);
        if let Some(allocation) = e.allocation_fraction {
            let weight = if e.method == EffortMethod::Sampled {
                1.0 / e.inclusion_probability.expect("validated probability")
            } else {
                1.0
            };
            contribution += super::elapsed_hours(left, right) * allocation * weight;
        }
        // Matching attestations may compose without splitting the observation.
        if !super::coverage::effort_covered(d, id, &e.method, left, right, as_of) {
            method_matches = false;
        }
    }
    ensure!(contribution.is_finite(), "non-finite effort total for {id}");
    let effort_complete = super::coverage::covered(
        d,
        id,
        CoverageDomain::Effort,
        task.enrolled_at,
        labour_end,
        as_of,
    ) && nonresponse_intervals == 0
        && method_matches
        && boundary_ok;
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
    let person_hours = if effort_complete && sample_present {
        Some(contribution)
    } else {
        None
    };
    if as_of < labour_end {
        issues.push("fixed enrolment labour horizon is not complete".into());
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
        recorded_person_hours: contribution,
        contains_sampled_estimates: sampled_frame || sampled_intervals > 0,
        sampled_intervals,
        nonresponse_intervals,
        evidence_ids: effort.iter().map(|(id, _)| id.clone()).collect(),
        issues,
    })
}
