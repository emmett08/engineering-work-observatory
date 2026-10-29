//! Frozen scope, first acceptance, missing follow-up and retained output.
use crate::model::*;
use anyhow::Result;
use chrono::{DateTime, Utc};

pub(super) struct OutcomeAccounting {
    pub accepted_at: Option<DateTime<Utc>>,
    pub lifecycle_status: String,
    pub accepted_by_deadline: Option<bool>,
    pub restricted_days: Option<f64>,
    pub retention_due: Option<DateTime<Utc>>,
    pub retained_fraction: Option<f64>,
    pub retained_output: Option<f64>,
    pub scope_treatment: String,
    pub scope_change_ids: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub issues: Vec<String>,
}

pub(super) fn account(
    d: &Dataset,
    id: &str,
    task: &Task,
    protocol: &Protocol,
    deadline: DateTime<Utc>,
    labour_end: DateTime<Utc>,
    as_of: DateTime<Utc>,
) -> Result<OutcomeAccounting> {
    let scope_changes: Vec<_> = d
        .scope_changes
        .iter()
        .filter(|(_, c)| c.task_id == *id && c.occurred_at <= as_of && c.occurred_at <= labour_end)
        .collect();
    let scope_changed = !scope_changes.is_empty();
    let scope_excluded =
        scope_changed && protocol.scope_change_rule == ScopeChangeRule::ExcludeFromOutput;
    let baseline_scope_required =
        scope_changed && protocol.scope_change_rule == ScopeChangeRule::BaselineOnly;
    let events: Vec<_> = d
        .lifecycle
        .iter()
        .filter(|(_, e)| e.task_id == *id && e.occurred_at <= as_of)
        .collect();
    let accepted_event = events
        .iter()
        .find(|(_, e)| e.event == LifecycleEvent::Accepted);
    let accepted_at = accepted_event.map(|(_, e)| e.occurred_at);
    let baseline_scope_unresolved = baseline_scope_required
        && !accepted_event
            .is_some_and(|(_, e)| e.scope_version.as_deref() == Some(task.scope_version.as_str()));
    let abandoned = events
        .iter()
        .any(|(_, e)| e.event == LifecycleEvent::Abandoned);
    let lost = events
        .iter()
        .any(|(_, e)| e.event == LifecycleEvent::LostToFollowUp && e.occurred_at <= deadline);
    let lifecycle_complete = super::coverage::covered(
        d,
        id,
        CoverageDomain::Lifecycle,
        task.enrolled_at,
        deadline,
        as_of,
    ) && !lost;
    let (status, accepted_by_deadline, restricted_days) = if scope_excluded {
        ("scope_changed_excluded_from_output", None, None)
    } else if baseline_scope_unresolved {
        ("scope_changed_baseline_unresolved", None, None)
    } else if let Some(at) = accepted_at {
        if at <= deadline {
            (
                "accepted_by_deadline",
                Some(true),
                Some(super::elapsed_hours(task.enrolled_at, at) / 24.0),
            )
        } else {
            (
                "accepted_late",
                Some(false),
                Some(f64::from(protocol.acceptance_days)),
            )
        }
    } else if abandoned {
        (
            "abandoned",
            Some(false),
            Some(f64::from(protocol.acceptance_days)),
        )
    } else if as_of < deadline {
        ("pending", None, None)
    } else if lost {
        ("lost_to_follow_up", None, None)
    } else if lifecycle_complete {
        (
            "observed_unaccepted",
            Some(false),
            Some(f64::from(protocol.acceptance_days)),
        )
    } else {
        ("coverage_missing", None, None)
    };
    let mut issues = Vec::new();
    if scope_excluded {
        issues.push("material scope change within the fixed observation horizon: frozen exclude_from_output rule assigns zero output while retaining labour; original-scope acceptance is unavailable".into());
    }
    if baseline_scope_unresolved {
        issues.push("baseline_only scope rule requires acceptance explicitly tied to the original scope_version".into());
    }
    if accepted_by_deadline.is_none() {
        issues.push(format!("acceptance outcome unavailable: {status}"));
    }
    let retention_due = accepted_at
        .map(|at| crate::model::days_after(at, protocol.follow_up_days))
        .transpose()?;
    let mut retained_fraction = None;
    let mut assessment_ids = Vec::new();
    if accepted_by_deadline == Some(true) && task.count_output {
        let due = retention_due.expect("accepted task has due date");
        let last = crate::model::days_after(due, protocol.assessment_tolerance_days)?;
        let assessments: Vec<_> = d
            .assessments
            .iter()
            .filter(|(_, a)| {
                a.task_id == *id
                    && a.phase == AssessmentPhase::FollowUp
                    && a.is_primary
                    && a.status == ObservationStatus::Observed
                    && a.assessed_at >= due
                    && a.assessed_at <= last
                    && a.assessed_at <= as_of
                    && (!baseline_scope_required
                        || a.scope_version.as_deref() == Some(task.scope_version.as_str()))
            })
            .collect();
        if assessments.len() == 1 {
            retained_fraction = assessments[0].1.retained_fraction;
            assessment_ids.push(assessments[0].0.clone());
        } else if assessments.len() > 1 {
            issues.push(
                "multiple primary follow-up assessments: designate an adjudicated primary".into(),
            );
        } else if as_of < due {
            issues.push("retention follow-up not yet due".into());
        } else {
            issues.push(
                "no observed primary retention assessment within the frozen date window".into(),
            );
        }
    }
    let retained_output =
        if !task.count_output || scope_excluded || accepted_by_deadline == Some(false) {
            Some(0.0)
        } else if accepted_by_deadline == Some(true) {
            retained_fraction.map(|r| task.frozen_score * r)
        } else {
            None
        };

    let scope_change_ids: Vec<_> = scope_changes.iter().map(|(id, _)| id.clone()).collect();
    let mut evidence_ids: Vec<_> = events.iter().map(|(id, _)| id.clone()).collect();
    evidence_ids.extend(assessment_ids);
    evidence_ids.extend(scope_change_ids.iter().cloned());
    Ok(OutcomeAccounting {
        accepted_at,
        lifecycle_status: status.into(),
        accepted_by_deadline,
        restricted_days,
        retention_due,
        retained_fraction,
        retained_output,
        scope_treatment: if scope_excluded {
            "excluded_from_output"
        } else if baseline_scope_required {
            "baseline_only"
        } else {
            "unchanged"
        }
        .into(),
        scope_change_ids,
        evidence_ids,
        issues,
    })
}
