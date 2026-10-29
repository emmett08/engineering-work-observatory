//! Direct task accounting from PDF equations (4), (6), (7), (9), and (22).
//! These descriptive results are not an estimate of a causal AI policy effect.
use crate::model::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub schema: String,
    pub as_of: DateTime<Utc>,
    pub interpretation: String,
    pub record_count: usize,
    pub task_count: usize,
    pub cohorts: Vec<CohortReport>,
    pub tasks: Vec<TaskReport>,
    pub limitations: Vec<String>,
    pub corrections: Vec<CorrectionSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CohortReport {
    pub protocol_id: String,
    pub enrolled_records: usize,
    pub counted_output_tasks: usize,
    pub accepted_by_deadline: Option<usize>,
    pub acceptance_fraction: Option<f64>,
    pub mean_restricted_days_without_acceptance: Option<f64>,
    pub total_person_hours: Option<f64>,
    pub mean_person_hours_per_counted_task: Option<f64>,
    pub total_retained_output: Option<f64>,
    pub mean_retained_output_per_counted_task: Option<f64>,
    pub retained_output_per_person_hour: Option<f64>,
    pub ratio_status: String,
    pub contains_sampled_effort_estimates: bool,
    pub uncertainty: String,
    pub incomplete_tasks: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskReport {
    pub task_id: String,
    pub protocol_id: String,
    pub requirement: String,
    pub task_class: String,
    pub assigned_policy: Option<String>,
    pub assignment_unit: Option<String>,
    pub scope_treatment: String,
    pub scope_change_ids: Vec<String>,
    pub task_link_ids: Vec<String>,
    pub defects: Vec<DefectReport>,
    pub count_output: bool,
    pub enrolled_at: DateTime<Utc>,
    pub acceptance_deadline: DateTime<Utc>,
    pub labour_end: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub lifecycle_status: String,
    pub accepted_by_deadline: Option<bool>,
    pub restricted_days_without_acceptance: Option<f64>,
    pub retention_due: Option<DateTime<Utc>>,
    pub retained_fraction: Option<f64>,
    pub frozen_score: f64,
    pub retained_output: Option<f64>,
    pub person_hours: Option<f64>,
    /// Available interval contributions, not an implied complete labour total.
    pub recorded_person_hours: f64,
    pub contains_sampled_effort_estimates: bool,
    pub sampled_intervals: usize,
    pub nonresponse_intervals: usize,
    pub evidence_ids: Vec<String>,
    pub issues: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DefectReport {
    pub id: String,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
    pub evidence_ids: Vec<String>,
    pub repair_effort_ids: Vec<String>,
}

mod cohort;
mod coverage;
mod labour;
mod outcome;

fn elapsed_hours(start: DateTime<Utc>, end: DateTime<Utc>) -> f64 {
    (end - start).as_seconds_f64() / 3_600.0
}

/// Analyse only observations available by `as_of`; unknown is never coerced to zero.
pub fn analyse(records: &[Evidence], as_of: DateTime<Utc>) -> Result<Report> {
    validate_records(records)?;
    let effective = effective_records(records, as_of)?;
    validate_records(&effective)?;
    let d = crate::model::dataset(&effective)?;
    let tasks = d
        .tasks
        .iter()
        .filter(|(_, task)| task.enrolled_at <= as_of)
        .map(|(id, task)| task_report(&d, id, task, as_of))
        .collect::<Result<Vec<_>>>()?;
    let record_count = records.iter().map(|r| &r.id).collect::<BTreeSet<_>>().len();
    Ok(Report {
        schema: "observatory.analysis/1".into(), as_of,
        interpretation: "Descriptive task accounting, not a causal estimate of AI effectiveness. Source records and product AI usage do not establish development labour or retained functionality.".into(),
        record_count, task_count: tasks.len(), cohorts: cohort::summarise(&tasks)?, tasks,
        corrections: correction_summaries(records, as_of)?,
        limitations: vec![
            "Coverage is an explicit researcher attestation; this program cannot verify omitted off-keyboard work or inaccurate allocations.".into(),
            "All-role person-hours, elapsed acceptance time, retained reference output, tokens and money remain distinct quantities.".into(),
            "Labour ends enrolment + acceptance horizon + follow-up; retention is assessed acceptance + follow-up. Early acceptance does not shorten labour accounting.".into(),
            "A complete sampled frame with observed responses permits an HT point estimate; nonresponse and partial frames remain unavailable rather than zero.".into(),
            "Independent quality sampling, feedback effects, assignment-weighted causal estimation and design variance are not implemented. Assess every counted accepted task or keep retained output unavailable.".into(),
            "Records dated after the requested as-of cut are excluded. A complete attestation whose end is after the cut is not used retrospectively.".into(),
            "Parent/group records with count_output=false contribute their measured labour but no task output, acceptance denominator or restricted-time denominator.".into(),
            "Implementation children must share the original requirement enrolment and protocol; implementation start does not restart the observation clock.".into(),
            "A material scope change within the fixed observation horizon follows the frozen protocol rule: exclude_from_output assigns zero output and retains labour; baseline_only requires explicit original-scope acceptance and assessment. Policy labels and links are metadata, not causal identification.".into(),
            "Retrospective corrections are explicit immutable records. Only corrections dated by this as-of cut affect calculations; the applied chain and reasons are reported. Corrections do not validate the underlying measurements or causal assumptions.".into(),
        ],
    })
}

/// Join independently calculated outcomes and labour with retained provenance.
fn task_report(
    d: &crate::model::Dataset,
    id: &str,
    task: &Task,
    as_of: DateTime<Utc>,
) -> Result<TaskReport> {
    let protocol = &d.protocols[&task.protocol_id];
    let deadline = crate::model::days_after(task.enrolled_at, protocol.acceptance_days)?;
    let labour_end = crate::model::days_after(deadline, protocol.follow_up_days)?;
    let outcome = outcome::account(d, id, task, protocol, deadline, labour_end, as_of)?;
    let labour = labour::account(d, id, task, labour_end, as_of)?;
    let task_link_ids: Vec<_> = d
        .task_links
        .iter()
        .filter(|(_, link)| link.task_id == id && link.observed_at <= as_of)
        .map(|(id, _)| id.clone())
        .collect();
    let defects: Vec<_> = d
        .defects
        .iter()
        .filter(|(_, defect)| defect.task_id == id && defect.detected_at <= as_of)
        .map(|(id, defect)| DefectReport {
            id: id.clone(),
            severity: defect.severity.clone(),
            detected_at: defect.detected_at,
            evidence_ids: defect.evidence_ids.clone(),
            repair_effort_ids: defect.repair_effort_ids.clone(),
        })
        .collect();
    let mut evidence_ids = vec![id.to_owned(), task.protocol_id.clone()];
    evidence_ids.extend(outcome.evidence_ids);
    evidence_ids.extend(labour.evidence_ids);
    evidence_ids.extend(task_link_ids.iter().cloned());
    evidence_ids.extend(defects.iter().map(|defect| defect.id.clone()));
    evidence_ids.extend(
        d.coverage
            .iter()
            .filter(|(_, c)| c.task_id == id && c.end <= as_of)
            .map(|(id, _)| id.clone()),
    );
    evidence_ids.sort();
    evidence_ids.dedup();
    let mut issues = outcome.issues;
    issues.extend(labour.issues);
    Ok(TaskReport {
        task_id: id.to_owned(),
        protocol_id: task.protocol_id.clone(),
        requirement: task.requirement.clone(),
        task_class: task.task_class.clone(),
        assigned_policy: task.assigned_policy.clone(),
        assignment_unit: task.assignment_unit.clone(),
        scope_treatment: outcome.scope_treatment,
        scope_change_ids: outcome.scope_change_ids,
        task_link_ids,
        defects,
        count_output: task.count_output,
        enrolled_at: task.enrolled_at,
        acceptance_deadline: deadline,
        labour_end,
        accepted_at: outcome.accepted_at,
        lifecycle_status: outcome.lifecycle_status,
        accepted_by_deadline: outcome.accepted_by_deadline,
        restricted_days_without_acceptance: outcome.restricted_days,
        retention_due: outcome.retention_due,
        retained_fraction: outcome.retained_fraction,
        frozen_score: task.frozen_score,
        retained_output: outcome.retained_output,
        person_hours: labour.person_hours,
        recorded_person_hours: labour.recorded_person_hours,
        contains_sampled_effort_estimates: labour.contains_sampled_estimates,
        sampled_intervals: labour.sampled_intervals,
        nonresponse_intervals: labour.nonresponse_intervals,
        evidence_ids,
        issues,
    })
}
