//! Cohort ratios retain all enrolled labour and count each declared output once.
use super::{CohortReport, TaskReport};
use anyhow::{Result, ensure};
use std::collections::BTreeMap;

pub(super) fn summarise(tasks: &[TaskReport]) -> Result<Vec<CohortReport>> {
    let mut grouped: BTreeMap<&str, Vec<&TaskReport>> = BTreeMap::new();
    for task in tasks {
        grouped.entry(&task.protocol_id).or_default().push(task);
    }
    let mut cohorts = Vec::new();
    for (protocol_id, rows) in grouped {
        let counted: Vec<_> = rows.iter().filter(|t| t.count_output).collect();
        let n = counted.len();
        let accepted = counted
            .iter()
            .map(|t| t.accepted_by_deadline.map(usize::from))
            .collect::<Option<Vec<_>>>()
            .map(|values| values.iter().sum::<usize>());
        let restricted = counted
            .iter()
            .map(|t| t.restricted_days_without_acceptance)
            .collect::<Option<Vec<_>>>()
            .filter(|_| n > 0)
            .map(|values| values.iter().sum::<f64>() / n as f64);
        let hours = rows
            .iter()
            .map(|t| t.person_hours)
            .collect::<Option<Vec<_>>>()
            .map(|values| values.iter().sum::<f64>());
        let output = counted
            .iter()
            .map(|t| t.retained_output)
            .collect::<Option<Vec<_>>>()
            .map(|values| values.iter().sum::<f64>());
        ensure!(
            hours.is_none_or(f64::is_finite) && output.is_none_or(f64::is_finite),
            "cohort totals exceed finite numerical range for {protocol_id}"
        );
        let sampled = rows.iter().any(|t| t.contains_sampled_effort_estimates);
        let ratio = hours
            .zip(output)
            .filter(|(h, _)| *h > 0.0)
            .map(|(h, q)| q / h);
        ensure!(
            ratio.is_none_or(f64::is_finite),
            "cohort ratio exceeds finite numerical range for {protocol_id}"
        );
        let ratio_status = if n == 0 {
            "unavailable_no_counted_tasks"
        } else if hours == Some(0.0) {
            "unavailable_zero_labour_denominator"
        } else if ratio.is_none() {
            "unavailable_incomplete_measurement"
        } else if sampled {
            "descriptive_ratio_using_sampled_labour_estimate"
        } else {
            "descriptive_direct_accounting"
        };
        cohorts.push(CohortReport {
            protocol_id: protocol_id.into(), enrolled_records: rows.len(), counted_output_tasks: n,
            accepted_by_deadline: accepted, acceptance_fraction: accepted.filter(|_| n > 0).map(|a| a as f64 / n as f64),
            mean_restricted_days_without_acceptance: restricted,
            total_person_hours: hours, mean_person_hours_per_counted_task: hours.filter(|_| n > 0).map(|h| h / n as f64),
            total_retained_output: output, mean_retained_output_per_counted_task: output.filter(|_| n > 0).map(|q| q / n as f64),
            retained_output_per_person_hour: if n > 0 { ratio } else { None }, ratio_status: ratio_status.into(),
            contains_sampled_effort_estimates: sampled,
            uncertainty: if sampled { "HT point estimate only; design variance is not estimated because joint inclusion/design information is not implemented. Ratio is not unbiased in general.".into() }
                else { "No population or causal uncertainty interval is inferred from direct accounting.".into() },
            incomplete_tasks: rows.iter().filter(|t| t.person_hours.is_none() || (t.count_output && t.retained_output.is_none())).map(|t| t.task_id.clone()).collect(),
        });
    }
    Ok(cohorts)
}
