//! Score an unchanged issued forecast only after its complete target is observed.
//!
//! Snapshot digests bind retained content; they are not certified issuance times.
use super::{
    EmpiricalDistribution, ForecastMethod, ForecastReport, ForecastStatus, MODEL_VERSION, calendar,
    empirical_rps, history::PreparedHistory,
};
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForecastScoreStatus {
    Pending,
    Available,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForecastScore {
    pub schema: String,
    pub model_version: String,
    pub method: ForecastMethod,
    pub prior_source_snapshot_sha256: String,
    pub later_source_snapshot_sha256: String,
    pub issued_at: DateTime<Utc>,
    pub data_cutoff: DateTime<Utc>,
    pub target_start: DateTime<Utc>,
    pub target_end: DateTime<Utc>,
    pub later_snapshot_cut: DateTime<Utc>,
    pub later_observed_cut: DateTime<Utc>,
    pub repository_ids: Vec<u64>,
    pub status: ForecastScoreStatus,
    pub reasons: Vec<String>,
    pub outcome: Option<u64>,
    pub rps: Option<f64>,
    pub median_absolute_error: Option<u64>,
    pub quantile_range_hit: Option<bool>,
    pub limitations: Vec<String>,
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Check the stored numerical forecast against the fixed model's own recipe.
/// The caller must additionally bind its training counts to the archived source
/// snapshot; an internally consistent fabricated count history is not detected here.
fn validate_prior(prior: &ForecastReport) -> Result<()> {
    ensure!(
        prior.schema == "observatory.merge-count-forecast/1",
        "unsupported forecast schema"
    );
    ensure!(
        prior.model_version == MODEL_VERSION,
        "unsupported forecast model version"
    );
    ensure!(
        valid_digest(&prior.source_snapshot_sha256),
        "invalid prior source snapshot digest"
    );
    let ids: BTreeSet<_> = prior.repository_ids.iter().copied().collect();
    ensure!(
        ids.len() == prior.repository_ids.len() && !ids.contains(&0),
        "invalid or duplicate forecast repository identity"
    );
    ensure!(
        prior.data_cutoff <= prior.issued_at,
        "forecast data cutoff exceeds issuance"
    );
    ensure!(
        prior.history_start <= prior.issued_at,
        "forecast history begins after issuance"
    );
    ensure!(
        prior.target_start >= prior.issued_at,
        "forecast target predates issuance"
    );
    ensure!(
        prior.target_start == calendar::target_start(prior.issued_at, prior.period)?,
        "forecast target does not match its declared calendar period"
    );
    let width = Duration::days(prior.period.days());
    ensure!(
        prior.target_start.checked_add_signed(width) == Some(prior.target_end),
        "forecast target width does not match its declared period"
    );
    ensure!(
        prior.required_training_bins == prior.period.required_bins(),
        "forecast training requirement does not match model version"
    );
    if !prior.coverage_complete {
        ensure!(
            prior.status == ForecastStatus::Unavailable
                && prior.available_complete_bins == 0
                && prior.training_bins.is_empty()
                && prior.distribution.is_none(),
            "incomplete forecast coverage has contradictory fitted output"
        );
        return Ok(());
    }
    ensure!(!ids.is_empty(), "complete forecast has no repository scope");
    let end = calendar::floor_boundary(prior.data_cutoff, prior.period)?;
    let available = if end > prior.history_start {
        usize::try_from((end - prior.history_start).num_seconds() / width.num_seconds())
            .context("forecast training support overflow")?
    } else {
        0
    };
    ensure!(
        available <= 100_000,
        "forecast training support exceeds model limit"
    );
    ensure!(
        prior.available_complete_bins == available,
        "forecast complete-bin count does not match its calendar frame"
    );
    let selected = prior.method.training_count(prior.period, available);
    ensure!(
        prior.training_bins.len() == selected,
        "forecast does not retain the model's complete training window"
    );
    for (index, bin) in prior.training_bins.iter().enumerate() {
        let periods = i32::try_from(selected - index).context("training window overflow")?;
        let expected_start = end
            .checked_sub_signed(width * periods)
            .context("training boundary underflow")?;
        ensure!(
            bin.start == expected_start
                && bin.start.checked_add_signed(width) == Some(bin.end)
                && bin.start >= prior.history_start
                && bin.end <= prior.data_cutoff,
            "forecast training bins are not complete consecutive model periods"
        );
    }
    if selected < prior.period.required_bins() {
        ensure!(
            prior.status == ForecastStatus::Unavailable && prior.distribution.is_none(),
            "forecast with insufficient history contains a predictive distribution"
        );
    } else {
        ensure!(
            prior.status == ForecastStatus::Available,
            "supported forecast incorrectly marked unavailable"
        );
        let counts: Vec<_> = prior.training_bins.iter().map(|bin| bin.count).collect();
        let expected = EmpiricalDistribution::from_counts(&counts)?;
        ensure!(
            prior.distribution.as_ref() == Some(&expected),
            "stored forecast distribution does not match its frozen training counts"
        );
    }
    Ok(())
}

/// Score all visible merges in the forecast's half-open target interval.
///
/// No outcome is emitted before the common conservative source exposure reaches
/// the entire target. New arrivals and reopened PRs count through `merged_at`;
/// current backlog and unmerged closures do not determine an outcome label.
pub fn score_forecast(prior: &ForecastReport, later: &PreparedHistory) -> Result<ForecastScore> {
    validate_prior(prior)?;
    ensure!(
        later.schema == "observatory.throughput-history/1",
        "unsupported later history schema"
    );
    ensure!(
        valid_digest(&later.source_snapshot_sha256),
        "invalid later source snapshot digest"
    );
    ensure!(
        later.observed_cut <= later.snapshot_cut,
        "later exposure exceeds knowledge cutoff"
    );
    let mut score = ForecastScore {
        schema: "observatory.merge-count-forecast-score/1".into(),
        model_version: prior.model_version.clone(),
        method: prior.method,
        prior_source_snapshot_sha256: prior.source_snapshot_sha256.clone(),
        later_source_snapshot_sha256: later.source_snapshot_sha256.clone(),
        issued_at: prior.issued_at,
        data_cutoff: prior.data_cutoff,
        target_start: prior.target_start,
        target_end: prior.target_end,
        later_snapshot_cut: later.snapshot_cut,
        later_observed_cut: later.observed_cut,
        repository_ids: prior.repository_ids.clone(),
        status: ForecastScoreStatus::Unavailable,
        reasons: Vec::new(),
        outcome: None,
        rps: None,
        median_absolute_error: None,
        quantile_range_hit: None,
        limitations: vec![
            "The score evaluates the unchanged issued distribution; it does not tune the model or establish predictive superiority.".into(),
            "Complete pagination describes currently visible source records, not proof that deleted or inaccessible historical objects never existed.".into(),
            "Source digests identify retained content. Receipt and issuance timestamps are not cryptographically certified evidence of when a forecast existed.".into(),
        ],
    };
    if prior.status != ForecastStatus::Available {
        score
            .reasons
            .push("The stored forecast was unavailable and has no distribution to score.".into());
        return Ok(score);
    }
    if !later.complete {
        score.reasons.push("Later required source coverage is incomplete; missing records cannot establish a zero outcome.".into());
        score.reasons.extend(later.reasons.iter().cloned());
        return Ok(score);
    }
    calendar::validate_history(later)?;
    let prior_ids: BTreeSet<_> = prior.repository_ids.iter().copied().collect();
    let later_ids: BTreeSet<_> = later
        .repositories
        .iter()
        .map(|repo| repo.repository_id)
        .collect();
    if prior_ids != later_ids {
        score.reasons.push(
            "The later numeric repository portfolio differs from the issued forecast.".into(),
        );
        return Ok(score);
    }
    if later.history_start > prior.target_start {
        score
            .reasons
            .push("The later source history begins after the forecast target starts.".into());
        return Ok(score);
    }
    if later.snapshot_cut < prior.issued_at {
        score
            .reasons
            .push("The supplied later snapshot predates forecast issuance.".into());
        return Ok(score);
    }
    if later.observed_cut < prior.target_end {
        score.status = ForecastScoreStatus::Pending;
        score.reasons.push("The common conservative source exposure has not reached the target end; no partial outcome is scored.".into());
        return Ok(score);
    }
    let outcome = later
        .pull_requests
        .iter()
        .filter(|pr| {
            pr.merged_at
                .is_some_and(|at| at >= prior.target_start && at < prior.target_end)
        })
        .try_fold(0_u64, |count, _| {
            count
                .checked_add(1)
                .context("forecast outcome count overflow")
        })?;
    let counts: Vec<_> = prior.training_bins.iter().map(|bin| bin.count).collect();
    // Recomputed above; use the checked distribution for all point/quantile summaries.
    let distribution = prior
        .distribution
        .as_ref()
        .context("available forecast has no distribution")?;
    score.status = ForecastScoreStatus::Available;
    score.outcome = Some(outcome);
    score.rps = Some(empirical_rps(&counts, outcome)?);
    score.median_absolute_error = Some(distribution.median.abs_diff(outcome));
    score.quantile_range_hit =
        Some(outcome >= distribution.quantile_10 && outcome <= distribution.quantile_90);
    Ok(score)
}
