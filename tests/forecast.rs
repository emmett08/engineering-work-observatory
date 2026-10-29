use chrono::{DateTime, Duration, Utc};
use observatory::forecast::{
    EmpiricalDistribution, ForecastMethod, ForecastPeriod, ForecastStatus, MODEL_VERSION,
    empirical_rps, forecast, forecast_with_method, history::PreparedHistory, reconstruct_backtest,
};
use serde_json::json;

fn time(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

fn history(counts: &[u64]) -> PreparedHistory {
    let start = time("2026-01-05T00:00:00Z"); // Monday.
    let cut = start + Duration::days(counts.len() as i64) + Duration::hours(12);
    let mut prs = Vec::new();
    for (day, &count) in counts.iter().enumerate() {
        for _ in 0..count {
            let merged = start + Duration::days(day as i64) + Duration::hours(12);
            prs.push(json!({
                "repository_id":1,"number":prs.len()+1,"created_at":merged-Duration::hours(1),
                "merged_at":merged,"state":"closed","observed_at":cut
            }));
        }
    }
    serde_json::from_value(json!({
        "schema":"observatory.throughput-history/1","mode":"synthetic_visible_inventory",
        "snapshot_cut":cut,"observed_cut":cut,"history_start":start,"complete":true,"reasons":[],
        "repositories":[{"repository_id":1,"full_name":"fixture/repo","created_at":start}],
        "pull_requests":prs,"source_snapshot_sha256":"synthetic"
    }))
    .unwrap()
}

#[test]
fn sparse_daily_forecast_does_not_invent_weekly_support() {
    let source = history(&[0, 1, 2, 7, 3]);
    let daily = forecast(&source, ForecastPeriod::Day).unwrap();
    assert_eq!(daily.status, ForecastStatus::Available);
    assert_eq!(daily.training_bins.len(), 5);
    assert_eq!(daily.target_start, time("2026-01-11T00:00:00Z"));
    assert_eq!(daily.target_end, time("2026-01-12T00:00:00Z"));
    let distribution = daily.distribution.unwrap();
    assert_eq!(distribution.mean, 2.6);
    assert_eq!(distribution.median, 2);
    assert_eq!(distribution.quantile_range_mass, 1.0);
    for period in [ForecastPeriod::Week, ForecastPeriod::Fortnight] {
        let unavailable = forecast(&source, period).unwrap();
        assert_eq!(unavailable.status, ForecastStatus::Unavailable);
        assert!(unavailable.distribution.is_none());
    }
}

#[test]
fn incomplete_coverage_abstains_and_is_not_zero() {
    let mut source = history(&[0; 9]);
    source.complete = false;
    source.reasons.push("request failed".into());
    let report = forecast(&source, ForecastPeriod::Day).unwrap();
    assert_eq!(report.status, ForecastStatus::Unavailable);
    assert!(report.distribution.is_none());
    assert!(report.reasons.iter().any(|s| s == "request failed"));
    assert_eq!(
        reconstruct_backtest(&source, ForecastPeriod::Day)
            .unwrap()
            .evaluation_origins,
        0
    );
}

#[test]
fn clock_cutoffs_exclude_current_and_stale_periods() {
    let mut source = history(&[1, 2, 3, 4, 5, 6]);
    source.observed_cut = time("2026-01-09T23:59:00Z");
    source.snapshot_cut = time("2026-02-10T15:00:00Z");
    for pr in &mut source.pull_requests {
        pr.observed_at = source.snapshot_cut;
    }
    let report = forecast(&source, ForecastPeriod::Day).unwrap();
    assert_eq!(report.available_complete_bins, 4);
    assert_eq!(
        report
            .training_bins
            .iter()
            .map(|b| b.count)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(report.issued_at, source.snapshot_cut);
    assert_eq!(report.data_cutoff, source.observed_cut);
    assert_eq!(report.target_start, time("2026-02-11T00:00:00Z"));
    // A later source row is valid retained evidence, but cannot extend the common frame.
    assert!(
        source
            .pull_requests
            .iter()
            .any(|pr| pr.merged_at.unwrap() > source.observed_cut)
    );
}

#[test]
fn exact_midnight_target_can_begin_at_issue_time() {
    let mut source = history(&[1, 2, 3, 4]);
    source.snapshot_cut = time("2026-01-09T00:00:00Z");
    source.observed_cut = source.snapshot_cut;
    for pr in &mut source.pull_requests {
        pr.observed_at = source.snapshot_cut;
    }
    let report = forecast(&source, ForecastPeriod::Day).unwrap();
    assert_eq!(report.target_start, source.snapshot_cut);
    assert_eq!(
        report.training_bins.last().unwrap().end,
        source.snapshot_cut
    );
}

#[test]
fn partial_first_day_and_nonexistent_repository_time_are_excluded() {
    let mut source = history(&[1; 12]);
    source.repositories.push(
        serde_json::from_value(json!({
            "repository_id":2,"full_name":"fixture/new","created_at":"2026-01-10T12:00:00Z"
        }))
        .unwrap(),
    );
    source.history_start = time("2026-01-10T12:00:00Z");
    let report = forecast(&source, ForecastPeriod::Day).unwrap();
    assert_eq!(
        report.training_bins.first().unwrap().start,
        time("2026-01-11T00:00:00Z")
    );
    assert_eq!(report.available_complete_bins, 6);
    assert_eq!(report.distribution.unwrap().mean, 1.0);
}

#[test]
fn portfolio_is_aggregated_before_empirical_forecasting() {
    let mut source = history(&[0, 0, 4, 4]);
    let mut repository = source.repositories[0].clone();
    repository.repository_id = 2;
    repository.full_name = "fixture/second".into();
    source.repositories.push(repository);
    let copies: Vec<_> = source
        .pull_requests
        .iter()
        .cloned()
        .map(|mut pr| {
            pr.repository_id = 2;
            pr
        })
        .collect();
    source.pull_requests.extend(copies);
    let distribution = forecast(&source, ForecastPeriod::Day)
        .unwrap()
        .distribution
        .unwrap();
    assert_eq!(
        distribution
            .mass
            .iter()
            .map(|p| p.count)
            .collect::<Vec<_>>(),
        [0, 8]
    );
    assert_eq!(
        distribution
            .mass
            .iter()
            .map(|p| p.probability)
            .collect::<Vec<_>>(),
        [0.5, 0.5]
    );
}

#[test]
fn fourteen_days_uses_four_observed_disjoint_pairs() {
    let mut source = history(&[1; 56]);
    source.observed_cut = time("2026-03-02T00:00:00Z");
    source.snapshot_cut = source.observed_cut;
    for pr in &mut source.pull_requests {
        pr.observed_at = source.snapshot_cut;
    }
    let report = forecast(&source, ForecastPeriod::Fortnight).unwrap();
    assert_eq!(report.status, ForecastStatus::Available);
    assert_eq!(report.training_bins.len(), 4);
    assert!(report.training_bins.iter().all(|b| b.count == 14));
    assert_eq!(report.target_start, source.snapshot_cut);
    assert_eq!(report.target_end, time("2026-03-16T00:00:00Z"));
    assert_eq!(report.training_bins[0].end, report.training_bins[1].start);
}

#[test]
fn unsupported_or_contradictory_evidence_fails_closed() {
    let mut source = history(&[1; 5]);
    source.pull_requests.push(source.pull_requests[0].clone());
    assert!(forecast(&source, ForecastPeriod::Day).is_err());
    source.pull_requests.pop();
    source.pull_requests[0].merged_at =
        Some(source.pull_requests[0].created_at - Duration::seconds(1));
    assert!(forecast(&source, ForecastPeriod::Day).is_err());
    source.pull_requests[0].merged_at = Some(source.snapshot_cut + Duration::seconds(1));
    assert!(forecast(&source, ForecastPeriod::Day).is_err());
}

#[test]
fn rps_matches_independent_pairwise_oracle_including_unseen_tails() {
    for a in 0..=3_u64 {
        for b in 0..=3_u64 {
            for c in 0..=3_u64 {
                let counts = [a, b, c];
                for y in [0, 1, 3, 10, 100_000] {
                    let first: f64 = counts.iter().map(|&v| v.abs_diff(y) as f64 / 3.0).sum();
                    let pair: f64 = counts
                        .iter()
                        .flat_map(|&x| counts.iter().map(move |&z| x.abs_diff(z) as f64 / 18.0))
                        .sum();
                    assert!((empirical_rps(&counts, y).unwrap() - (first - pair)).abs() < 1e-9);
                }
            }
        }
    }
    assert_eq!(empirical_rps(&[0; 8], 50).unwrap(), 50.0);
    assert!(empirical_rps(&[], 0).is_err());
}

#[test]
fn quantiles_median_and_empirical_mass_are_explicit() {
    let d = EmpiricalDistribution::from_counts(&[0, 1, 2, 3, 4, 5, 6, 70]).unwrap();
    assert_eq!(d.median, 3);
    assert_eq!(d.quantile_10, 0);
    assert_eq!(d.quantile_90, 70);
    assert_eq!(d.quantile_range_mass, 1.0);
    assert_ne!(d.mean, d.median as f64);
    assert!(EmpiricalDistribution::from_counts(&[u64::MAX]).is_err());
}

#[test]
fn fixed_backtest_uses_only_mature_preceding_bins() {
    let first = reconstruct_backtest(&history(&[0, 1, 4, 2, 3, 9]), ForecastPeriod::Day).unwrap();
    let changed_later =
        reconstruct_backtest(&history(&[0, 1, 4, 2, 3, 99]), ForecastPeriod::Day).unwrap();
    assert_eq!(first.evaluation_origins, 2);
    assert_eq!(first.rows.len(), 10);
    for (a, b) in first.rows.iter().take(5).zip(changed_later.rows.iter()) {
        assert_eq!(a.target_start, b.target_start);
        assert_eq!(a.mean, b.mean);
        assert_eq!(a.rps, b.rps);
    }
    let poisson = first
        .rows
        .iter()
        .find(|r| r.method.starts_with("poisson"))
        .unwrap();
    assert!(poisson.rps.unwrap().is_finite());
    assert!(poisson.poisson_omitted_tail_bound.unwrap() <= 1e-12);
    assert_eq!(first.summaries.len(), 5);
}

#[test]
fn predeclared_frame_reserves_training_then_scores_only_holdout() {
    let source = history(&[1; 100]);
    let report =
        observatory::forecast::reconstruct_backtest_window(&source, ForecastPeriod::Day, 78, 52)
            .unwrap();
    assert_eq!(report.status, ForecastStatus::Available);
    assert_eq!(report.available_complete_bins, 100);
    assert_eq!(report.requested_history_bins, Some(78));
    assert_eq!(report.selected_history_bins, 78);
    assert_eq!(report.initial_training_bins, 52);
    assert_eq!(report.evaluation_origins, 26);
    assert_eq!(report.rows.len(), 26 * 5);
    assert_eq!(
        report.rows[0].target_start,
        time("2026-01-05T00:00:00Z") + Duration::days(74)
    );
    let expanding = report
        .rows
        .iter()
        .find(|r| r.method == "empirical_expanding")
        .unwrap();
    assert_eq!(expanding.training_bins, 52);
    let short = observatory::forecast::reconstruct_backtest_window(
        &history(&[1; 12]),
        ForecastPeriod::Day,
        78,
        52,
    )
    .unwrap();
    assert_eq!(short.status, ForecastStatus::Unavailable);
    assert!(short.rows.is_empty());
}

#[test]
fn valid_new_repository_after_common_exposure_cut_has_no_full_bins() {
    let mut source = history(&[1; 5]);
    source.repositories[0].created_at = source.snapshot_cut - Duration::hours(1);
    source.history_start = source.repositories[0].created_at;
    source.observed_cut = source.snapshot_cut - Duration::hours(2);
    source.pull_requests.clear();
    let report = forecast(&source, ForecastPeriod::Day).unwrap();
    assert_eq!(report.status, ForecastStatus::Unavailable);
    assert_eq!(report.available_complete_bins, 0);
}

#[test]
fn expanding_reference_and_recent_challenger_have_explicit_distinct_rules() {
    let source = history(&[20, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let reference = forecast(&source, ForecastPeriod::Day).unwrap();
    let recent = forecast_with_method(
        &source,
        ForecastPeriod::Day,
        ForecastMethod::RecentEmpirical,
    )
    .unwrap();
    assert_eq!(reference.method, ForecastMethod::ExpandingEmpirical);
    assert_eq!(recent.method, ForecastMethod::RecentEmpirical);
    assert_eq!(reference.model_version, MODEL_VERSION);
    assert_eq!(MODEL_VERSION, "observatory.empirical-merge-count/2");
    assert_eq!(reference.training_bins.len(), 12);
    assert_eq!(recent.training_bins.len(), 8);
    assert_eq!(reference.distribution.unwrap().historical_maximum, 20);
    assert_eq!(recent.distribution.unwrap().historical_maximum, 0);
    assert_eq!(reference.target_start, recent.target_start);
    assert_eq!(reference.target_end, recent.target_end);
    assert_eq!(
        reference.validation_status,
        "unvalidated_operational_reference"
    );
    assert_eq!(
        recent.validation_status,
        "unvalidated_experimental_candidate"
    );
}

#[test]
fn expanding_fortnight_reference_keeps_whole_observed_pairs() {
    let source = history(&[1; 84]);
    let reference = forecast(&source, ForecastPeriod::Fortnight).unwrap();
    let recent = forecast_with_method(
        &source,
        ForecastPeriod::Fortnight,
        ForecastMethod::RecentEmpirical,
    )
    .unwrap();
    assert_eq!(reference.training_bins.len(), 6);
    assert_eq!(recent.training_bins.len(), 4);
    assert!(reference.training_bins.iter().all(|bin| bin.count == 14));
    assert_eq!(reference.training_bins.last(), recent.training_bins.last());
}
