//! Fixed rolling-origin comparison reconstructed from currently visible source events.
use super::{
    CountBin, EmpiricalDistribution, ForecastPeriod, ForecastStatus, calendar, empirical_rps,
    history::PreparedHistory,
};
use anyhow::{Result, ensure};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BacktestRow {
    pub method: String,
    pub target_start: DateTime<Utc>,
    pub target_end: DateTime<Utc>,
    pub outcome: u64,
    pub training_bins: usize,
    pub mean: Option<f64>,
    pub median: Option<u64>,
    pub rps: Option<f64>,
    pub median_absolute_error: Option<f64>,
    pub quantile_10: Option<u64>,
    pub quantile_90: Option<u64>,
    pub within_quantile_range: Option<bool>,
    pub poisson_omitted_tail_bound: Option<f64>,
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodSummary {
    pub method: String,
    pub scored_origins: usize,
    pub mean_rps: Option<f64>,
    pub mean_median_absolute_error: Option<f64>,
    pub quantile_range_coverage: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BacktestReport {
    pub schema: String,
    pub source_snapshot_sha256: String,
    pub evidence_mode: String,
    pub status: ForecastStatus,
    pub period: ForecastPeriod,
    pub snapshot_cut: DateTime<Utc>,
    pub complete_through: DateTime<Utc>,
    pub history_start: DateTime<Utc>,
    pub available_complete_bins: usize,
    pub evaluation_origins: usize,
    pub study_selection: String,
    pub requested_history_bins: Option<usize>,
    pub selected_history_bins: usize,
    pub initial_training_bins: usize,
    pub rows: Vec<BacktestRow>,
    pub summaries: Vec<MethodSummary>,
    pub reasons: Vec<String>,
    pub limitations: Vec<String>,
}

fn empirical_row(method: &str, train: &[u64], target: &CountBin) -> Result<BacktestRow> {
    let distribution = EmpiricalDistribution::from_counts(train)?;
    Ok(BacktestRow {
        method: method.into(),
        target_start: target.start,
        target_end: target.end,
        outcome: target.count,
        training_bins: train.len(),
        mean: Some(distribution.mean),
        median: Some(distribution.median),
        rps: Some(empirical_rps(train, target.count)?),
        median_absolute_error: Some(distribution.median.abs_diff(target.count) as f64),
        quantile_10: Some(distribution.quantile_10),
        quantile_90: Some(distribution.quantile_90),
        within_quantile_range: Some(
            (distribution.quantile_10..=distribution.quantile_90).contains(&target.count),
        ),
        poisson_omitted_tail_bound: None,
        unavailable_reason: None,
    })
}

/// Reconstruct fixed chronological comparisons; these are not historical receipts
/// or evidence that source fields were available at the simulated issue time.
pub fn reconstruct_backtest(
    history: &PreparedHistory,
    period: ForecastPeriod,
) -> Result<BacktestReport> {
    reconstruct(history, period, None, period.required_bins())
}

/// Evaluate a predeclared trailing frame, preserving its initial training block.
/// For example, (78, 52) evaluates only the last 26 of 78 weekly bins.
pub fn reconstruct_backtest_window(
    history: &PreparedHistory,
    period: ForecastPeriod,
    history_bins: usize,
    initial_training_bins: usize,
) -> Result<BacktestReport> {
    ensure!(
        initial_training_bins >= period.required_bins(),
        "initial training block is below the model minimum"
    );
    ensure!(
        history_bins > initial_training_bins,
        "study window must include a held-out target"
    );
    reconstruct(history, period, Some(history_bins), initial_training_bins)
}

fn reconstruct(
    history: &PreparedHistory,
    period: ForecastPeriod,
    requested_history_bins: Option<usize>,
    initial_training_bins: usize,
) -> Result<BacktestReport> {
    let mut report = BacktestReport {
        schema: "observatory.merge-count-backtest/1".into(),
        source_snapshot_sha256: history.source_snapshot_sha256.clone(),
        evidence_mode: "retrospective_visible_inventory_reconstruction".into(),
        status: ForecastStatus::Unavailable, period, snapshot_cut: history.snapshot_cut,
        complete_through: history.observed_cut, history_start: history.history_start,
        available_complete_bins: 0, evaluation_origins: 0, rows: Vec::new(), summaries: Vec::new(),
        study_selection: if requested_history_bins.is_some() { "predeclared_trailing_window" } else { "all_supported_complete_bins" }.into(),
        requested_history_bins,
        selected_history_bins: 0,
        initial_training_bins,
        reasons: history.reasons.clone(),
        limitations: vec![
            "This reconstructs event-time count histories from the current visible inventory; it does not establish historical source availability.".into(),
            "Nonoverlapping targets can remain serially dependent; no confidence interval or superiority claim is inferred from these scores.".into(),
            "Windows and baselines are fixed; scores do not select or tune the operational empirical model.".into(),
            "Poisson scores use an explicitly normalized numerical PMF with a reported upper-tail truncation bound.".into(),
        ],
    };
    if !history.complete {
        report
            .reasons
            .push("Incomplete source coverage prevents historical zero counts.".into());
        return Ok(report);
    }
    calendar::validate_history(history)?;
    let mut end = calendar::floor_boundary(history.observed_cut, period)?;
    if period == ForecastPeriod::Fortnight {
        // Every other Monday, anchored permanently to 1970-01-05 UTC.
        let anchor = NaiveDate::from_ymd_opt(1970, 1, 5)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let offset = (end - anchor).num_days().rem_euclid(14);
        end -= Duration::days(offset);
    }
    let mut bins = calendar::complete_bins_ending(history, period, end)?;
    report.available_complete_bins = bins.len();
    if let Some(requested) = requested_history_bins {
        if bins.len() < requested {
            report.reasons.push(format!(
                "Predeclared study requires {requested} complete bins; only {} are supported.",
                bins.len()
            ));
            return Ok(report);
        }
        bins = bins.split_off(bins.len() - requested);
    }
    report.selected_history_bins = bins.len();
    if bins.len() <= initial_training_bins {
        report
            .reasons
            .push("No complete target period remains after the minimum training history.".into());
        return Ok(report);
    }
    for index in initial_training_bins..bins.len() {
        let target = &bins[index];
        let train: Vec<_> = bins[index.saturating_sub(period.window())..index]
            .iter()
            .map(|b| b.count)
            .collect();
        report
            .rows
            .push(empirical_row("empirical_recent", &train, target)?);
        let expanding: Vec<_> = bins[..index].iter().map(|b| b.count).collect();
        report
            .rows
            .push(empirical_row("empirical_expanding", &expanding, target)?);
        report
            .rows
            .push(empirical_row("zero_point_mass", &[0], target)?);
        report.rows.push(empirical_row(
            "previous_period_point_mass",
            &[bins[index - 1].count],
            target,
        )?);
        let poisson_train = &bins[index - 4..index];
        let mean = poisson_train.iter().map(|b| b.count as f64 / 4.0).sum();
        let mut row = BacktestRow {
            method: "poisson_prior_four_period_mean".into(),
            target_start: target.start,
            target_end: target.end,
            outcome: target.count,
            training_bins: 4,
            mean: Some(mean),
            median: None,
            rps: None,
            median_absolute_error: None,
            quantile_10: None,
            quantile_90: None,
            within_quantile_range: None,
            poisson_omitted_tail_bound: None,
            unavailable_reason: None,
        };
        match super::poisson::score(mean, target.count) {
            Ok(score) => {
                row.median = Some(score.median);
                row.rps = Some(score.rps);
                row.median_absolute_error = Some(score.median.abs_diff(target.count) as f64);
                row.quantile_10 = Some(score.lower);
                row.quantile_90 = Some(score.upper);
                row.within_quantile_range =
                    Some((score.lower..=score.upper).contains(&target.count));
                row.poisson_omitted_tail_bound = Some(score.omitted_tail_bound);
            }
            Err(error) => row.unavailable_reason = Some(error.to_string()),
        }
        report.rows.push(row);
        report.evaluation_origins += 1;
    }
    let mut groups: BTreeMap<&str, Vec<&BacktestRow>> = BTreeMap::new();
    for row in &report.rows {
        groups.entry(&row.method).or_default().push(row);
    }
    for (method, rows) in groups {
        let scored: Vec<_> = rows.into_iter().filter(|row| row.rps.is_some()).collect();
        let n = scored.len();
        report.summaries.push(MethodSummary {
            method: method.into(),
            scored_origins: n,
            mean_rps: (n > 0).then(|| scored.iter().map(|r| r.rps.unwrap() / n as f64).sum()),
            mean_median_absolute_error: (n > 0).then(|| {
                scored
                    .iter()
                    .map(|r| r.median_absolute_error.unwrap() / n as f64)
                    .sum()
            }),
            quantile_range_coverage: (n > 0).then(|| {
                scored
                    .iter()
                    .filter(|r| r.within_quantile_range == Some(true))
                    .count() as f64
                    / n as f64
            }),
        });
    }
    report.status = ForecastStatus::Available;
    Ok(report)
}
