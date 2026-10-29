//! Observation-frame coverage, shared by labour and lifecycle accounting.
use crate::model::{CoverageDomain, CoverageStatus, Dataset};
use chrono::{DateTime, Utc};

pub(super) fn covered(
    d: &Dataset,
    task_id: &str,
    domain: CoverageDomain,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    as_of: DateTime<Utc>,
) -> bool {
    if end > as_of {
        return false;
    }
    // A partial/missing attestation cannot be silently overruled by another row.
    if d.coverage.iter().any(|(_, c)| {
        c.task_id == task_id
            && c.domain == domain
            && c.end <= as_of
            && c.start < end
            && c.end > start
            && c.status != CoverageStatus::Complete
    }) {
        return false;
    }
    let mut spans: Vec<_> = d
        .coverage
        .iter()
        .filter(|(_, c)| {
            c.task_id == task_id
                && c.domain == domain
                && c.status == CoverageStatus::Complete
                && c.end <= as_of
        })
        .map(|(_, c)| (c.start, c.end))
        .collect();
    spans.sort();
    let mut cursor = start;
    for (left, right) in spans {
        if left > cursor {
            return false;
        }
        cursor = cursor.max(right);
        if cursor >= end {
            return true;
        }
    }
    cursor >= end
}
