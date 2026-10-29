//! Immutable, baseline-first forecasts of observed portfolio PR merge counts.
pub mod archive;
mod backtest;
mod calendar;
mod distribution;
pub mod history;
mod poisson;
pub mod validation;

pub use backtest::{
    BacktestReport, BacktestRow, MethodSummary, reconstruct_backtest, reconstruct_backtest_window,
};
pub use distribution::{CountMass, EmpiricalDistribution, empirical_rps};
pub use validation::{ForecastScore, ForecastScoreStatus, score_forecast};

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use history::PreparedHistory;
use serde::{Deserialize, Serialize};

pub const MODEL_VERSION: &str = "observatory.empirical-merge-count/2";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForecastPeriod {
    Day,
    Week,
    Fortnight,
}

impl ForecastPeriod {
    pub fn days(self) -> i64 {
        match self {
            Self::Day => 1,
            Self::Week => 7,
            Self::Fortnight => 14,
        }
    }
    pub fn required_bins(self) -> usize {
        4
    }
    pub fn window(self) -> usize {
        if self == Self::Fortnight { 4 } else { 8 }
    }
}

/// An explicit forecast rule; historical scores never silently select a method.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForecastMethod {
    ExpandingEmpirical,
    RecentEmpirical,
}

impl ForecastMethod {
    pub fn training_count(self, period: ForecastPeriod, available: usize) -> usize {
        match self {
            Self::ExpandingEmpirical => available,
            Self::RecentEmpirical => available.min(period.window()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CountBin {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForecastStatus {
    Available,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForecastReport {
    pub schema: String,
    pub model_version: String,
    pub method: ForecastMethod,
    pub source_snapshot_sha256: String,
    pub evidence_mode: String,
    pub status: ForecastStatus,
    pub period: ForecastPeriod,
    pub issued_at: DateTime<Utc>,
    pub data_cutoff: DateTime<Utc>,
    pub history_start: DateTime<Utc>,
    pub target_start: DateTime<Utc>,
    pub target_end: DateTime<Utc>,
    pub repository_ids: Vec<u64>,
    pub coverage_complete: bool,
    pub available_complete_bins: usize,
    pub required_training_bins: usize,
    pub training_bins: Vec<CountBin>,
    pub distribution: Option<EmpiricalDistribution>,
    pub validation_status: String,
    pub reasons: Vec<String>,
    pub limitations: Vec<String>,
}

/// Forecast the next full UTC day or Monday-based period from a frozen source cut.
/// At an exact UTC boundary, the period beginning at that boundary is still future
/// and is used. Otherwise the target starts at the following midnight/Monday.
pub fn forecast(history: &PreparedHistory, period: ForecastPeriod) -> Result<ForecastReport> {
    forecast_with_method(history, period, ForecastMethod::ExpandingEmpirical)
}

/// Issue the declared method without selecting or tuning it on evaluation scores.
/// The default reference uses the full supported frame. The recent-window method
/// remains an explicit experimental challenger; both require four complete bins.
pub fn forecast_with_method(
    history: &PreparedHistory,
    period: ForecastPeriod,
    method: ForecastMethod,
) -> Result<ForecastReport> {
    let target_start = calendar::target_start(history.snapshot_cut, period)?;
    let target_end = target_start
        .checked_add_signed(Duration::days(period.days()))
        .context("forecast target overflow")?;
    let mut report = ForecastReport {
        schema: "observatory.merge-count-forecast/1".into(),
        model_version: MODEL_VERSION.into(),
        method,
        source_snapshot_sha256: history.source_snapshot_sha256.clone(),
        evidence_mode: history.mode.clone(),
        status: ForecastStatus::Unavailable,
        period,
        issued_at: history.snapshot_cut,
        data_cutoff: history.observed_cut,
        history_start: history.history_start,
        target_start,
        target_end,
        repository_ids: history.repositories.iter().map(|r| r.repository_id).collect(),
        coverage_complete: history.complete,
        available_complete_bins: 0,
        required_training_bins: period.required_bins(),
        training_bins: Vec::new(),
        distribution: None,
        validation_status: match method {
            ForecastMethod::ExpandingEmpirical => "unvalidated_operational_reference",
            ForecastMethod::RecentEmpirical => "unvalidated_experimental_candidate",
        }.into(),
        reasons: history.reasons.clone(),
        limitations: vec![
            "The outcome is a count of observed PR merges, not effort, delivered functionality or AI effect.".into(),
            "Historical complete calendar periods must be relevant to the future; drift and missing source objects can invalidate that expectation.".into(),
            "Empirical quantiles have coarse support and no guaranteed future coverage; historical maxima are not physical upper bounds.".into(),
            "Partial creation and current periods are excluded; a target may begin after the most recent completed training period.".into(),
        ],
    };
    report.limitations.push(match method {
        ForecastMethod::ExpandingEmpirical => "The operational reference uses all supported complete bins. This frame differs from a bounded public-reference study and does not establish predictive skill on these repositories.",
        ForecastMethod::RecentEmpirical => "The recent-window method is an explicit experimental challenger; it is not promoted or selected by these report scores.",
    }.into());
    if !history.complete {
        report.reasons.push(
            "Required source coverage is incomplete; missing intervals are not zero counts.".into(),
        );
        return Ok(report);
    }
    calendar::validate_history(history)?;
    let bins = calendar::complete_bins(history, period)?;
    report.available_complete_bins = bins.len();
    report.training_bins = bins
        .into_iter()
        .rev()
        .take(method.training_count(period, report.available_complete_bins))
        .collect();
    report.training_bins.reverse();
    if report.training_bins.len() < period.required_bins() {
        report.reasons.push(format!(
            "Need at least {} complete {}-day bins; only {} are supported. No horizon scaling or convolution is used.",
            period.required_bins(), period.days(), report.training_bins.len()
        ));
        return Ok(report);
    }
    let counts: Vec<_> = report.training_bins.iter().map(|b| b.count).collect();
    report.distribution = Some(EmpiricalDistribution::from_counts(&counts)?);
    report.status = ForecastStatus::Available;
    if period == ForecastPeriod::Day {
        report.limitations.push("This is a separate exploratory one-day forecast; it does not establish weekly forecasting skill or remove weekday and launch-period effects.".into());
    }
    Ok(report)
}
