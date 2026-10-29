//! Source-bound local forecast receipts and automatic, label-free outcome scoring.
use super::{
    ForecastPeriod, ForecastReport, ForecastScore, forecast_with_method,
    history::{self, PreparedHistory},
    score_forecast,
};
use crate::{snapshot::Snapshot, store::digest};
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::Path};

/// A command-issued local artifact. Times are declarations, not trusted timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssuedForecast {
    pub schema: String,
    pub created_at: DateTime<Utc>,
    pub snapshot: Snapshot,
    pub history: PreparedHistory,
    pub forecast: ForecastReport,
}

impl IssuedForecast {
    pub fn issue(
        snapshot: Snapshot,
        repositories: &[String],
        period: ForecastPeriod,
    ) -> Result<Self> {
        let history = history::prepare(&snapshot, repositories)?;
        let forecast = super::forecast(&history, period)?;
        let issued = Self {
            schema: "observatory.team-forecast-bundle/1".into(),
            created_at: Utc::now(),
            snapshot,
            history,
            forecast,
        };
        issued.verify()?;
        Ok(issued)
    }

    /// Recompute the exact retained recipe, rather than trusting a supplied PMF.
    pub fn verify(&self) -> Result<()> {
        ensure!(
            self.schema == "observatory.team-forecast-bundle/1",
            "expected an original command forecast bundle; offline replays are not issuance receipts"
        );
        self.snapshot.verify()?;
        ensure!(
            self.created_at >= self.snapshot.as_of,
            "receipt creation predates its source cut"
        );
        ensure!(
            self.created_at <= self.forecast.target_start,
            "receipt was created after its target began"
        );
        let names: Vec<_> = self
            .history
            .repositories
            .iter()
            .map(|r| r.repository_id.to_string())
            .collect();
        ensure!(!names.is_empty(), "issued receipt has no repository scope");
        let history = history::prepare(&self.snapshot, &names)?;
        ensure!(
            serde_json::to_value(&history)? == serde_json::to_value(&self.history)?,
            "stored history differs from its original source snapshot"
        );
        let expected = forecast_with_method(&history, self.forecast.period, self.forecast.method)?;
        ensure!(
            serde_json::to_value(&expected)? == serde_json::to_value(&self.forecast)?,
            "stored forecast differs from its frozen source and declared recipe"
        );
        Ok(())
    }
}

pub fn score_issued(prior: &IssuedForecast, later: &Snapshot) -> Result<ForecastScore> {
    prior.verify()?;
    let names: Vec<_> = prior
        .history
        .repositories
        .iter()
        .map(|r| r.repository_id.to_string())
        .collect();
    let history = history::prepare(later, &names)?;
    score_forecast(&prior.forecast, &history)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveScore {
    pub archive_name: String,
    pub archive_sha256: String,
    pub score: Option<ForecastScore>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveScoreReport {
    pub schema: String,
    pub later_snapshot_sha256: String,
    pub entries: Vec<ArchiveScore>,
    pub limitations: Vec<String>,
}

/// Scan only the local command archive. Corrupt/unsupported files stay visible as errors.
/// No entry is rewritten, and no outcome is inferred from an incomplete later cut.
pub fn score_directory(directory: &Path, later: &Snapshot) -> Result<ArchiveScoreReport> {
    later.verify()?;
    let mut paths = if directory.exists() {
        fs::read_dir(directory)?
            .map(|entry| entry.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    paths.retain(|p| p.extension().is_some_and(|e| e == "json"));
    paths.sort();
    let mut entries = Vec::new();
    for path in paths {
        let mut entry = ArchiveScore {
            archive_name: path
                .file_name()
                .context("archive filename missing")?
                .to_string_lossy()
                .into(),
            archive_sha256: String::new(),
            score: None,
            error: None,
        };
        let scored = (|| -> Result<ForecastScore> {
            const LIMIT: u64 = 128 * 1024 * 1024;
            ensure!(
                fs::metadata(&path)?.len() <= LIMIT,
                "archive exceeds 128 MiB limit"
            );
            let mut bytes = Vec::new();
            fs::File::open(&path)?
                .take(LIMIT + 1)
                .read_to_end(&mut bytes)?;
            ensure!(bytes.len() as u64 <= LIMIT, "archive exceeds 128 MiB limit");
            entry.archive_sha256 = digest(&bytes);
            let prior: IssuedForecast =
                serde_json::from_slice(&bytes).context("invalid original forecast bundle")?;
            score_issued(&prior, later)
        })();
        match scored {
            Ok(score) => entry.score = Some(score),
            Err(error) => entry.error = Some(format!("{error:#}")),
        }
        entries.push(entry);
    }
    Ok(ArchiveScoreReport {
        schema: "observatory.forecast-archive-scores/1".into(),
        later_snapshot_sha256: later.sha256.clone(), entries,
        limitations: vec!["Local archive bytes and timestamps bind the declared recipe and source cut; they do not independently prove pre-outcome existence or source completeness.".into(), "Pending and unavailable outcomes are not forecast errors or zero outcomes. Corrupt receipts are reported separately and never silently scored.".into()],
    })
}
