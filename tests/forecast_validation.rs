use chrono::{DateTime, Duration, Utc};
use observatory::forecast::{
    CountBin, EmpiricalDistribution, ForecastMethod, ForecastPeriod, ForecastReport,
    ForecastScoreStatus, ForecastStatus, forecast, forecast_with_method,
    history::{PreparedHistory, PullRequestHistory},
    score_forecast,
};
use serde_json::json;

fn time(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

fn fixture() -> (ForecastReport, PreparedHistory) {
    let start = time("2026-01-05T00:00:00Z");
    let issued = time("2026-01-09T12:00:00Z");
    let mut prs = Vec::new();
    for (day, count) in [0, 1, 2, 3].into_iter().enumerate() {
        for _ in 0..count {
            let merged = start + Duration::days(day as i64) + Duration::hours(6);
            prs.push(json!({
                "repository_id":1,"number":prs.len()+1,
                "created_at":merged-Duration::hours(1),"merged_at":merged,
                "closed_at":merged,"state":"closed","observed_at":issued
            }));
        }
    }
    let mut source: PreparedHistory = serde_json::from_value(json!({
        "schema":"observatory.throughput-history/1","mode":"synthetic_visible_inventory",
        "snapshot_cut":issued,"observed_cut":issued,"history_start":start,
        "complete":true,"reasons":[],
        "repositories":[{"repository_id":1,"full_name":"fixture/repo","created_at":start}],
        "pull_requests":prs,"source_snapshot_sha256":"a".repeat(64)
    }))
    .unwrap();
    source.current_pressure.open_pull_requests = Some(0);
    let report = forecast(&source, ForecastPeriod::Day).unwrap();
    assert_eq!(report.status, ForecastStatus::Available);
    assert_eq!(report.target_start, time("2026-01-10T00:00:00Z"));
    let mut later = source;
    later.snapshot_cut = time("2026-01-11T01:00:00Z");
    later.observed_cut = time("2026-01-11T00:00:00Z");
    later.source_snapshot_sha256 = "b".repeat(64);
    (report, later)
}

fn merge(later: &mut PreparedHistory, number: u64, created: &str, merged: &str) {
    let at = time(merged);
    later.pull_requests.push(PullRequestHistory {
        repository_id: 1,
        number,
        created_at: time(created),
        merged_at: Some(at),
        closed_at: Some(at),
        state: "closed".into(),
        observed_at: later.snapshot_cut,
        source_record_id: format!("synthetic/pr/{number}"),
    });
}

#[test]
fn mature_half_open_target_counts_arrivals_and_reopened_merges_with_empty_origin_queue() {
    let (prior, mut later) = fixture();
    assert_eq!(later.current_pressure.open_pull_requests, Some(0));
    merge(
        &mut later,
        100,
        "2026-01-09T23:00:00Z",
        "2026-01-10T00:00:00Z",
    );
    // This PR did not exist at forecast issuance.
    merge(
        &mut later,
        101,
        "2026-01-10T10:00:00Z",
        "2026-01-10T12:00:00Z",
    );
    // The merge endpoint counts even if this old PR was previously closed/reopened.
    merge(
        &mut later,
        102,
        "2026-01-07T00:00:00Z",
        "2026-01-10T18:00:00Z",
    );
    merge(
        &mut later,
        103,
        "2026-01-10T10:00:00Z",
        "2026-01-11T00:00:00Z",
    );
    merge(
        &mut later,
        104,
        "2026-01-09T10:00:00Z",
        "2026-01-09T23:59:59Z",
    );
    let score = score_forecast(&prior, &later).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Available);
    assert_eq!(score.outcome, Some(3));
    assert_eq!(score.rps, Some(0.875));
    assert_eq!(score.median_absolute_error, Some(2));
    assert_eq!(score.quantile_range_hit, Some(true));
    assert_eq!(score.prior_source_snapshot_sha256, "a".repeat(64));
    assert_eq!(score.later_source_snapshot_sha256, "b".repeat(64));
}

#[test]
fn a_fully_observed_zero_outcome_is_scoreable() {
    let (prior, later) = fixture();
    let score = score_forecast(&prior, &later).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Available);
    assert_eq!(score.outcome, Some(0));
    assert_eq!(score.rps, Some(0.875));
    assert_eq!(score.median_absolute_error, Some(1));
}

#[test]
fn knowledge_time_does_not_mature_a_stale_source_or_score_early_positives() {
    let (prior, mut later) = fixture();
    merge(
        &mut later,
        100,
        "2026-01-10T10:00:00Z",
        "2026-01-10T12:00:00Z",
    );
    later.observed_cut = prior.target_end - Duration::milliseconds(1);
    later.snapshot_cut = prior.target_end + Duration::days(10);
    let score = score_forecast(&prior, &later).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Pending);
    assert!(score.outcome.is_none() && score.rps.is_none() && score.quantile_range_hit.is_none());
    later.observed_cut = prior.target_end;
    assert_eq!(score_forecast(&prior, &later).unwrap().outcome, Some(1));
}

#[test]
fn incomplete_inventory_is_unavailable_instead_of_a_zero_outcome() {
    let (prior, mut later) = fixture();
    later.complete = false;
    later.reasons.push("pagination failed".into());
    let score = score_forecast(&prior, &later).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Unavailable);
    assert!(score.outcome.is_none() && score.rps.is_none());
    assert!(
        score
            .reasons
            .iter()
            .any(|reason| reason == "pagination failed")
    );
}

#[test]
fn changed_numeric_portfolio_is_unavailable_even_if_the_name_is_unchanged() {
    let (prior, mut later) = fixture();
    later.repositories[0].repository_id = 2;
    for pr in &mut later.pull_requests {
        pr.repository_id = 2;
    }
    let score = score_forecast(&prior, &later).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Unavailable);
    assert!(
        score
            .reasons
            .iter()
            .any(|reason| reason.contains("portfolio differs"))
    );
}

#[test]
fn source_frame_excluding_target_and_snapshot_before_issue_cannot_score() {
    let (prior, mut later) = fixture();
    later.pull_requests.clear();
    later.history_start = prior.target_start + Duration::hours(1);
    later.repositories[0].created_at = later.history_start;
    let score = score_forecast(&prior, &later).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Unavailable);
    assert!(
        score
            .reasons
            .iter()
            .any(|reason| reason.contains("history begins"))
    );
    let (_, mut earlier) = fixture();
    earlier.snapshot_cut = prior.issued_at - Duration::seconds(1);
    earlier.observed_cut = earlier.snapshot_cut;
    for pr in &mut earlier.pull_requests {
        pr.observed_at = earlier.snapshot_cut;
    }
    let score = score_forecast(&prior, &earlier).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Unavailable);
    assert!(score.outcome.is_none());
}

#[test]
fn future_duplicate_and_out_of_scope_later_events_reject() {
    let (prior, later) = fixture();
    let mut changed = later.clone();
    changed.pull_requests[0].merged_at = Some(changed.snapshot_cut + Duration::seconds(1));
    assert!(score_forecast(&prior, &changed).is_err());
    let mut changed = later.clone();
    changed.pull_requests[0].observed_at = changed.snapshot_cut + Duration::seconds(1);
    assert!(score_forecast(&prior, &changed).is_err());
    let mut changed = later.clone();
    changed.pull_requests.push(changed.pull_requests[0].clone());
    assert!(score_forecast(&prior, &changed).is_err());
    let mut changed = later.clone();
    changed.pull_requests[0].repository_id = 999;
    assert!(score_forecast(&prior, &changed).is_err());
    let mut changed = later;
    changed.observed_cut = changed.snapshot_cut + Duration::seconds(1);
    assert!(score_forecast(&prior, &changed).is_err());
}

#[test]
fn malformed_prior_clocks_calendar_training_and_model_reject() {
    let (prior, later) = fixture();
    let alterations: &[fn(&mut ForecastReport)] = &[
        |p| p.issued_at = p.target_start + Duration::seconds(1),
        |p| p.data_cutoff = p.issued_at + Duration::seconds(1),
        |p| p.target_start += Duration::hours(1),
        |p| p.target_end += Duration::days(1),
        |p| p.training_bins[0].start += Duration::seconds(1),
        |p| p.training_bins[0].end += Duration::days(1),
        |p| p.training_bins.swap(0, 1),
        |p| {
            p.training_bins.pop();
        },
        |p| p.available_complete_bins += 1,
        |p| p.required_training_bins -= 1,
        |p| p.model_version = "future-model/999".into(),
        |p| p.repository_ids.push(1),
        |p| p.coverage_complete = false,
        |p| p.source_snapshot_sha256 = "not-a-digest".into(),
    ];
    for alter in alterations {
        let mut changed = prior.clone();
        alter(&mut changed);
        assert!(
            score_forecast(&changed, &later).is_err(),
            "accepted malformed prior: {changed:?}"
        );
    }
}

#[test]
fn every_stored_distribution_summary_is_bound_to_frozen_counts() {
    let (prior, later) = fixture();
    let original = serde_json::to_value(&prior).unwrap();
    for (field, forged) in [
        ("observations", json!(999)),
        ("mean", json!(99.0)),
        ("median", json!(99)),
        ("quantile_10", json!(99)),
        ("quantile_90", json!(99)),
        ("quantile_range_mass", json!(0.0)),
        ("historical_minimum", json!(99)),
        ("historical_maximum", json!(99)),
        ("mass", json!([{"count":0,"probability":1.0}])),
    ] {
        let mut changed = original.clone();
        changed["distribution"][field] = forged;
        let changed: ForecastReport = serde_json::from_value(changed).unwrap();
        assert!(
            score_forecast(&changed, &later).is_err(),
            "accepted forged {field}"
        );
    }
    let restored: ForecastReport =
        serde_json::from_str(&serde_json::to_string(&prior).unwrap()).unwrap();
    assert_eq!(
        score_forecast(&restored, &later).unwrap().status,
        ForecastScoreStatus::Available
    );
}

#[test]
fn legitimate_saved_distributions_with_fractional_masses_round_trip() {
    let (template, later) = fixture();
    let end = template.training_bins.last().unwrap().end;
    for size in 4..=64 {
        let mut prior = template.clone();
        prior.history_start = end - Duration::days(size as i64);
        prior.available_complete_bins = size;
        prior.training_bins = (0..size)
            .map(|index| {
                let start = prior.history_start + Duration::days(index as i64);
                CountBin {
                    start,
                    end: start + Duration::days(1),
                    count: (index * index + 1) as u64,
                }
            })
            .collect();
        let counts: Vec<_> = prior.training_bins.iter().map(|bin| bin.count).collect();
        prior.distribution = Some(EmpiricalDistribution::from_counts(&counts).unwrap());
        let restored = serde_json::from_str(&serde_json::to_string(&prior).unwrap()).unwrap();
        assert_eq!(
            score_forecast(&restored, &later).unwrap().status,
            ForecastScoreStatus::Available
        );
    }
}

#[test]
fn method_is_preserved_and_checked_without_reinterpreting_old_saved_forecasts() {
    let (template, mut source) = fixture();
    source.snapshot_cut = template.issued_at;
    source.observed_cut = template.data_cutoff;
    source.history_start = template.training_bins.last().unwrap().end - Duration::days(12);
    source.repositories[0].created_at = source.history_start;
    let expanding = forecast(&source, ForecastPeriod::Day).unwrap();
    let recent = forecast_with_method(
        &source,
        ForecastPeriod::Day,
        ForecastMethod::RecentEmpirical,
    )
    .unwrap();
    assert_eq!(expanding.method, ForecastMethod::ExpandingEmpirical);
    assert_eq!(expanding.training_bins.len(), 12);
    assert_eq!(recent.training_bins.len(), 8);
    source.observed_cut = expanding.target_end;
    source.snapshot_cut = expanding.target_end + Duration::hours(1);
    for prior in [&expanding, &recent] {
        let score = score_forecast(prior, &source).unwrap();
        assert_eq!(score.status, ForecastScoreStatus::Available);
        assert_eq!(score.method, prior.method);
    }
    let mut tampered = expanding.clone();
    tampered.method = ForecastMethod::RecentEmpirical;
    assert!(score_forecast(&tampered, &source).is_err());
    let mut tampered = recent;
    tampered.method = ForecastMethod::ExpandingEmpirical;
    assert!(score_forecast(&tampered, &source).is_err());
    let mut old = expanding.clone();
    old.model_version = "observatory.empirical-merge-count/1".into();
    assert!(score_forecast(&old, &source).is_err());
    let mut missing_method = serde_json::to_value(expanding).unwrap();
    missing_method.as_object_mut().unwrap().remove("method");
    assert!(serde_json::from_value::<ForecastReport>(missing_method).is_err());
}

#[test]
fn unavailable_weekly_forecast_is_not_warmed_up_or_scaled_from_daily_counts() {
    let (daily, later) = fixture();
    let mut source = later.clone();
    source.snapshot_cut = daily.issued_at;
    source.observed_cut = daily.data_cutoff;
    source.source_snapshot_sha256 = daily.source_snapshot_sha256.clone();
    let prior = forecast(&source, ForecastPeriod::Week).unwrap();
    assert_eq!(prior.status, ForecastStatus::Unavailable);
    let mut matured = later;
    matured.observed_cut = prior.target_end;
    matured.snapshot_cut = prior.target_end + Duration::hours(1);
    let score = score_forecast(&prior, &matured).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Unavailable);
    assert!(score.outcome.is_none() && score.rps.is_none());
}
