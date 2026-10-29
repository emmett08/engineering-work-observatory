//! Complete calendar-bin construction from the common repository existence frame.
use super::{CountBin, ForecastPeriod, history::PreparedHistory};
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Datelike, Duration, Utc};
use std::collections::BTreeSet;

pub(super) fn validate_history(history: &PreparedHistory) -> Result<()> {
    ensure!(
        history.history_start <= history.snapshot_cut,
        "history starts after its knowledge cut"
    );
    ensure!(
        history.observed_cut <= history.snapshot_cut,
        "exposure cut exceeds knowledge cut"
    );
    ensure!(
        !history.repositories.is_empty(),
        "no repositories in forecast scope"
    );
    let ids: BTreeSet<_> = history
        .repositories
        .iter()
        .map(|r| r.repository_id)
        .collect();
    ensure!(
        ids.len() == history.repositories.len(),
        "duplicate repository identity"
    );
    ensure!(!ids.contains(&0), "repository identity must be positive");
    let start = history
        .repositories
        .iter()
        .map(|r| r.created_at)
        .max()
        .unwrap();
    ensure!(
        history.history_start == start,
        "history start must be latest repository creation"
    );
    let mut seen = BTreeSet::new();
    for pr in &history.pull_requests {
        ensure!(ids.contains(&pr.repository_id), "PR outside forecast scope");
        ensure!(
            pr.number > 0 && seen.insert((pr.repository_id, pr.number)),
            "invalid or duplicate PR identity"
        );
        ensure!(pr.created_at <= history.snapshot_cut, "future PR creation");
        ensure!(
            pr.observed_at <= history.snapshot_cut,
            "future PR observation"
        );
        if let Some(merged_at) = pr.merged_at {
            ensure!(merged_at >= pr.created_at, "PR merged before creation");
            ensure!(merged_at <= history.snapshot_cut, "future merge outcome");
        }
    }
    Ok(())
}

pub(super) fn floor_boundary(at: DateTime<Utc>, period: ForecastPeriod) -> Result<DateTime<Utc>> {
    let midnight = at
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .context("invalid UTC midnight")?
        .and_utc();
    if period == ForecastPeriod::Day {
        Ok(midnight)
    } else {
        midnight
            .checked_sub_signed(Duration::days(i64::from(
                at.weekday().num_days_from_monday(),
            )))
            .context("calendar boundary underflow")
    }
}

pub(super) fn target_start(at: DateTime<Utc>, period: ForecastPeriod) -> Result<DateTime<Utc>> {
    let floor = floor_boundary(at, period)?;
    if at == floor {
        Ok(floor)
    } else {
        // A 14-day forecast starts at the next Monday, not at an arbitrary fortnight parity.
        floor
            .checked_add_signed(Duration::days(if period == ForecastPeriod::Day {
                1
            } else {
                7
            }))
            .context("calendar boundary overflow")
    }
}

pub(super) fn complete_bins(
    history: &PreparedHistory,
    period: ForecastPeriod,
) -> Result<Vec<CountBin>> {
    complete_bins_ending(
        history,
        period,
        floor_boundary(history.observed_cut, period)?,
    )
}

pub(super) fn complete_bins_ending(
    history: &PreparedHistory,
    period: ForecastPeriod,
    mut end: DateTime<Utc>,
) -> Result<Vec<CountBin>> {
    let width = Duration::days(period.days());
    let mut bins = Vec::new();
    while let Some(start) = end.checked_sub_signed(width) {
        if start < history.history_start {
            break;
        }
        ensure!(
            bins.len() < 100_000,
            "calendar history exceeds supported bin limit"
        );
        bins.push(CountBin {
            start,
            end,
            count: 0,
        });
        end = start;
    }
    bins.reverse();
    if let (Some(first), Some(last)) = (bins.first(), bins.last()) {
        let begin = first.start;
        let finish = last.end;
        for merged_at in history.pull_requests.iter().filter_map(|pr| pr.merged_at) {
            if merged_at >= begin && merged_at < finish {
                let index = ((merged_at - begin).num_seconds() / width.num_seconds()) as usize;
                bins[index].count = bins[index].count.checked_add(1).context("count overflow")?;
            }
        }
    }
    Ok(bins)
}
