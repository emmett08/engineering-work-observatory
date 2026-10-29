//! Observation-frame coverage, shared by labour and lifecycle accounting.
use crate::model::{CoverageDomain, CoverageStatus, Dataset, EffortMethod};
use chrono::{DateTime, Utc};

pub(super) fn covered(
    d: &Dataset,
    task_id: &str,
    domain: CoverageDomain,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    as_of: DateTime<Utc>,
) -> bool {
    covered_with_method(d, task_id, domain, start, end, as_of, None)
}

/// Attestation boundaries do not split an effort observation or its sampling unit.
pub(super) fn effort_covered(
    d: &Dataset,
    task_id: &str,
    method: &EffortMethod,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    as_of: DateTime<Utc>,
) -> bool {
    covered_with_method(
        d,
        task_id,
        CoverageDomain::Effort,
        start,
        end,
        as_of,
        Some(method),
    )
}

fn covered_with_method(
    d: &Dataset,
    task_id: &str,
    domain: CoverageDomain,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    as_of: DateTime<Utc>,
    method: Option<&EffortMethod>,
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
                && method.is_none_or(|method| c.method == *method)
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
