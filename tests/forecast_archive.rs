use chrono::{DateTime, Utc};
use observatory::{
    forecast::{
        ForecastPeriod, ForecastScoreStatus,
        archive::{IssuedForecast, score_directory, score_issued},
        forecast,
        history::prepare,
    },
    model::Evidence,
    snapshot::Snapshot,
    store::Store,
};
use serde_json::{Value, json};
use std::fs;

fn at(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

// An automatic source fixture, independent of the forecasting library's count bins.
fn snapshot(start: &str, receipt: &str, merged: Option<&str>) -> Snapshot {
    snapshot_named(start, receipt, merged, "team/repo")
}

fn snapshot_named(start: &str, receipt: &str, merged: Option<&str>, name: &str) -> Snapshot {
    let root = tempfile::tempdir().unwrap();
    let mut store = Store::open(root.path()).unwrap();
    let mut values = vec![
        (
            "metadata",
            "github.repository",
            json!({"id":42,"full_name":name,"created_at":"2026-01-01T12:00:00Z"}),
        ),
        (
            "summary",
            "github.throughput_collection",
            json!({
                "schema":"observatory.github-throughput-collection/1","profile":"throughput",
                "started_at":start,"completed_at":receipt,"unique_pull_requests":usize::from(merged.is_some()),
                "duplicate_pull_request_rows":0,
                "coverage":[
                    {"source":"github.repository","scope":format!("https://api.github.com/repos/{name}"),"state":"complete","pages":1,"records":1},
                    {"source":"github.pull_request","scope":format!("https://api.github.com/repos/{name}/pulls?state=all&per_page=100"),"state":"complete","pages":1,"records":usize::from(merged.is_some())}
                ]
            }),
        ),
    ];
    if let Some(merged) = merged {
        values.push(("pr", "github.pull_request", json!({"id":100,"number":1,"state":"closed","created_at":merged,"closed_at":merged,"merged_at":merged})));
    }
    let records: Vec<_> = values.into_iter().map(|(id, family, payload)| Evidence {
        id:id.into(), kind:"source_record".into(),
        data:json!({"source":family,"repository":"github:repository:42","source_id":id,"payload":payload})
    }).collect();
    let raw = store
        .retain_bytes(&serde_json::to_vec(&records).unwrap())
        .unwrap();
    store
        .insert_batch(&records, "automatic-test", at(receipt), &raw)
        .unwrap();
    Snapshot::freeze(&store, at(receipt)).unwrap()
}

fn prior() -> IssuedForecast {
    let snapshot = snapshot("2026-01-10T12:00:00Z", "2026-01-10T12:01:00Z", None);
    let history = prepare(&snapshot, &[]).unwrap();
    assert!(history.complete, "{:?}", history.reasons);
    let forecast = forecast(&history, ForecastPeriod::Day).unwrap();
    let bundle = IssuedForecast {
        schema: "observatory.team-forecast-bundle/1".into(),
        created_at: at("2026-01-10T12:01:01Z"),
        snapshot,
        history,
        forecast,
    };
    bundle.verify().unwrap();
    bundle
}

#[test]
fn archived_source_bound_forecast_scores_only_after_exposure_and_counts_new_arrival() {
    let prior = prior();
    let roundtrip: IssuedForecast =
        serde_json::from_slice(&serde_json::to_vec(&prior).unwrap()).unwrap();
    roundtrip.verify().unwrap();
    let stale = snapshot(
        "2026-01-11T23:59:59Z",
        "2026-01-12T01:00:00Z",
        Some("2026-01-11T15:00:00Z"),
    );
    assert_eq!(
        score_issued(&roundtrip, &stale).unwrap().status,
        ForecastScoreStatus::Pending
    );
    let mature = snapshot(
        "2026-01-12T00:00:00Z",
        "2026-01-12T01:00:00Z",
        Some("2026-01-11T15:00:00Z"),
    );
    let score = score_issued(&roundtrip, &mature).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Available);
    assert_eq!(score.outcome, Some(1));
    assert_eq!(score.rps, Some(1.0));
}

#[test]
fn repository_rename_preserves_numeric_portfolio_when_scoring() {
    let receipt = prior();
    let later = snapshot_named(
        "2026-01-12T00:00:00Z",
        "2026-01-12T01:00:00Z",
        Some("2026-01-11T15:00:00Z"),
        "new-owner/new-name",
    );
    let score = score_issued(&receipt, &later).unwrap();
    assert_eq!(score.status, ForecastScoreStatus::Available);
    assert_eq!(score.repository_ids, vec![42]);
    assert_eq!(score.outcome, Some(1));
}

#[test]
fn receipt_rejects_fabricated_counts_even_with_internally_consistent_distribution() {
    let mut prior = prior();
    prior.forecast.training_bins[0].count = 100;
    let counts: Vec<_> = prior
        .forecast
        .training_bins
        .iter()
        .map(|b| b.count)
        .collect();
    prior.forecast.distribution =
        Some(observatory::forecast::EmpiricalDistribution::from_counts(&counts).unwrap());
    assert!(
        prior
            .verify()
            .unwrap_err()
            .to_string()
            .contains("frozen source")
    );
}

#[test]
fn receipts_reject_replays_modified_history_sources_and_post_target_creation() {
    let mut receipt = prior();
    receipt.schema = "observatory.team-forecast-replay/1".into();
    assert!(receipt.verify().is_err());
    receipt = prior();
    receipt.history.pull_requests.clear();
    receipt.history.current_pressure.open_pull_requests = Some(999);
    assert!(receipt.verify().is_err());
    receipt = prior();
    receipt.snapshot.records[0].record.data["payload"]["id"] = json!(999);
    assert!(receipt.verify().is_err());
    receipt = prior();
    receipt.created_at = receipt.forecast.target_start + chrono::Duration::seconds(1);
    assert!(receipt.verify().is_err());
}

#[test]
fn directory_scores_originals_and_reports_corrupt_replays_without_mutating_any_receipt() {
    let root = tempfile::tempdir().unwrap();
    let first = serde_json::to_vec(&prior()).unwrap();
    fs::write(root.path().join("original.json"), &first).unwrap();
    let mut replay: Value = serde_json::from_slice(&first).unwrap();
    replay["schema"] = json!("observatory.team-forecast-replay/1");
    fs::write(
        root.path().join("replay.json"),
        serde_json::to_vec(&replay).unwrap(),
    )
    .unwrap();
    fs::write(root.path().join("broken.json"), b"{").unwrap();
    fs::write(root.path().join("ignored.txt"), b"{").unwrap();
    let later = snapshot("2026-01-12T00:00:00Z", "2026-01-12T01:00:00Z", None);
    let scores = score_directory(root.path(), &later).unwrap();
    assert_eq!(scores.entries.len(), 3);
    assert_eq!(
        scores.entries.iter().filter(|e| e.score.is_some()).count(),
        1
    );
    assert_eq!(
        scores.entries.iter().filter(|e| e.error.is_some()).count(),
        2
    );
    assert_eq!(fs::read(root.path().join("original.json")).unwrap(), first);
    let absent = score_directory(&root.path().join("absent"), &later).unwrap();
    assert!(absent.entries.is_empty());
}
