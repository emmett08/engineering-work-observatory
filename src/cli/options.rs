use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Collect evidence and measure effort, acceptance and retained output"
)]
pub(super) struct Cli {
    #[arg(long, global = true, default_value = ".observatory")]
    pub(super) state: PathBuf,
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum CollectionProfile {
    Full,
    Throughput,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum ForecastHorizon {
    Day,
    Week,
    Fortnight,
}

impl From<ForecastHorizon> for observatory::forecast::ForecastPeriod {
    fn from(value: ForecastHorizon) -> Self {
        match value {
            ForecastHorizon::Day => Self::Day,
            ForecastHorizon::Week => Self::Week,
            ForecastHorizon::Fortnight => Self::Fortnight,
        }
    }
}

#[derive(Subcommand)]
pub(super) enum Command {
    /// Create the local evidence store.
    Init,
    /// Import a complete JSONL study batch or an observatory.bundle/1 JSON document.
    Import {
        file: PathBuf,
        #[arg(long, default_value = "study-records")]
        source: String,
    },
    /// Collect both project repositories by default; retain partial-source coverage on errors.
    CollectGithub {
        #[arg(long = "repository")]
        repositories: Vec<String>,
        #[arg(long, default_value = "https://api.github.com")]
        api_base: String,
        #[arg(long, default_value = "GH_TOKEN")]
        token_env: String,
        #[arg(long, value_enum, default_value_t = CollectionProfile::Full)]
        profile: CollectionProfile,
    },
    /// Capture a local repository without checking out or executing its code.
    CollectGit {
        path: PathBuf,
        #[arg(long)]
        repository: String,
    },
    /// Import a report, audit or checkpoint with an explicit stable lineage manifest.
    ImportEngarg {
        file: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
    },
    /// Import a validated immutable tool-execution bundle (JSON or GitHub artifact ZIP).
    ImportExecutions { file: PathBuf },
    /// Collect execution bundles and independently reconcile workflow attempts and jobs.
    CollectGithubExecutions {
        #[arg(long)]
        repository: String,
        #[arg(long)]
        run_id: Option<u64>,
        #[arg(long, default_value = "https://api.github.com")]
        api_base: String,
        #[arg(long, default_value = "GH_TOKEN")]
        token_env: String,
    },
    /// Take a consistent read-only snapshot of an EAL SQLite store, or import an export.
    ImportEal {
        file: PathBuf,
        #[arg(long)]
        namespace: String,
    },
    /// Freeze a content-identified knowledge cut (default: now).
    Snapshot {
        #[arg(long)]
        as_of: Option<DateTime<Utc>>,
        #[arg(long)]
        output: PathBuf,
    },
    /// Recompute an offline report from a verified frozen snapshot.
    Analyse {
        snapshot: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Check retained source hashes, record hashes and SQLite integrity.
    Verify,
    /// Evaluate a synthetic conditional retry scenario, separate from empirical analysis.
    Simulate { file: PathBuf },
    /// Estimate frame labour from frozen predictions and probability-sampled observations.
    EstimateLabour { file: PathBuf },
    /// Compute exact attainable labour bounds under interval and shared-capacity constraints.
    BoundLabour { file: PathBuf },
    /// Collect automatic PR evidence, freeze a cut and issue an immutable team forecast.
    ForecastTeam {
        #[arg(long = "repository")]
        repositories: Vec<String>,
        #[arg(long, value_enum, default_value_t = ForecastHorizon::Day)]
        horizon: ForecastHorizon,
        #[arg(long, default_value = "https://api.github.com")]
        api_base: String,
        #[arg(long, default_value = "GH_TOKEN")]
        token_env: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Reproduce forecasts or event-time reconstruction scores from a retained source cut.
    ForecastSnapshot {
        snapshot: PathBuf,
        #[arg(long = "repository")]
        repositories: Vec<String>,
        #[arg(long, value_enum, default_value_t = ForecastHorizon::Day)]
        horizon: ForecastHorizon,
        #[arg(long)]
        backtest: bool,
        #[arg(long, requires = "initial_training_bins")]
        history_bins: Option<usize>,
        #[arg(long, requires = "history_bins")]
        initial_training_bins: Option<usize>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Score an original command-issued forecast against a later verified snapshot.
    ScoreForecast {
        bundle: PathBuf,
        later_snapshot: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}
