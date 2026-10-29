//! Explicit, immutable correction chains for typed study records.
//! Original evidence remains retained; these functions construct an effective view.
use super::Evidence;
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    pub target_id: String,
    pub replacement_data: Value,
    pub corrected_at: DateTime<Utc>,
    pub reason: String,
    #[serde(default)]
    pub supersedes_correction_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorrectionSummary {
    pub id: String,
    pub target_id: String,
    pub target_kind: String,
    pub corrected_at: DateTime<Utc>,
    pub reason: String,
    pub supersedes_correction_id: Option<String>,
    pub is_current: bool,
}

struct CorrectionGraph {
    originals: BTreeMap<String, Evidence>,
    corrections: Vec<(String, Correction)>,
}

fn correctable(kind: &str) -> bool {
    matches!(
        kind,
        "protocol"
            | "task"
            | "lifecycle"
            | "effort"
            | "coverage"
            | "assessment"
            | "ai_usage"
            | "task_link"
            | "defect"
            | "task_scope_change"
    )
}

fn graph(records: &[Evidence]) -> Result<CorrectionGraph> {
    let mut originals = BTreeMap::new();
    for record in records {
        ensure!(!record.id.trim().is_empty(), "record id must be nonempty");
        if let Some(previous) = originals.get(&record.id) {
            ensure!(
                previous == record,
                "conflicting duplicate record id {}",
                record.id
            );
        } else {
            originals.insert(record.id.clone(), record.clone());
        }
    }
    let mut corrections: BTreeMap<String, Correction> = BTreeMap::new();
    for (id, record) in &originals {
        if record.kind == "correction" {
            let correction: Correction = serde_json::from_value(record.data.clone())
                .with_context(|| format!("invalid correction record {id}"))?;
            ensure!(
                !correction.reason.trim().is_empty(),
                "correction needs reason in {id}"
            );
            ensure!(
                correction.replacement_data.is_object(),
                "replacement_data must be an object in {id}"
            );
            ensure!(
                correction.target_id != *id,
                "correction cannot target itself in {id}"
            );
            let target = originals.get(&correction.target_id).with_context(|| {
                format!("unknown correction target {} in {id}", correction.target_id)
            })?;
            ensure!(
                correctable(&target.kind),
                "correction target must be a typed study record, not {} in {id}",
                target.kind
            );
            corrections.insert(id.clone(), correction);
        }
    }
    let mut roots: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut successors: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, correction) in &corrections {
        if let Some(previous_id) = &correction.supersedes_correction_id {
            ensure!(
                previous_id != id,
                "correction cannot supersede itself in {id}"
            );
            let previous = corrections.get(previous_id).with_context(|| {
                format!("unknown supersedes_correction_id {previous_id} in {id}")
            })?;
            ensure!(
                previous.target_id == correction.target_id,
                "correction predecessor targets another record in {id}"
            );
            ensure!(
                previous.corrected_at < correction.corrected_at,
                "correction dates must strictly increase; cycles are forbidden in {id}"
            );
            ensure!(
                successors.insert(previous_id, id).is_none(),
                "correction chain branches at {previous_id}"
            );
        } else {
            roots.entry(&correction.target_id).or_default().push(id);
        }
    }
    for correction in corrections.values() {
        ensure!(
            roots
                .get(correction.target_id.as_str())
                .is_some_and(|ids| ids.len() == 1),
            "correction chain branches or has no root for {}",
            correction.target_id
        );
    }
    let mut corrections: Vec<_> = corrections.into_iter().collect();
    corrections.sort_by(|(left_id, left), (right_id, right)| {
        (left.corrected_at, left_id).cmp(&(right.corrected_at, right_id))
    });
    Ok(CorrectionGraph {
        originals,
        corrections,
    })
}

/// Construct an as-of view without changing the stable ID, kind, or raw inputs.
/// Replacement schemas and study-graph constraints are checked by validate_records.
pub fn effective_records(records: &[Evidence], as_of: DateTime<Utc>) -> Result<Vec<Evidence>> {
    let graph = graph(records)?;
    let mut effective = graph.originals;
    effective.retain(|_, record| record.kind != "correction");
    for (_, correction) in graph.corrections {
        if correction.corrected_at <= as_of {
            let target = effective
                .get_mut(&correction.target_id)
                .expect("validated original target");
            target.data = correction.replacement_data;
        }
    }
    Ok(effective.into_values().collect())
}

/// Retain the applied correction history and identify the current link per target.
pub fn correction_summaries(
    records: &[Evidence],
    as_of: DateTime<Utc>,
) -> Result<Vec<CorrectionSummary>> {
    let graph = graph(records)?;
    let applied: Vec<_> = graph
        .corrections
        .iter()
        .filter(|(_, c)| c.corrected_at <= as_of)
        .collect();
    let mut current: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, correction) in &applied {
        current.insert(&correction.target_id, id);
    }
    Ok(applied
        .into_iter()
        .map(|(id, correction)| CorrectionSummary {
            id: id.clone(),
            target_id: correction.target_id.clone(),
            target_kind: graph.originals[&correction.target_id].kind.clone(),
            corrected_at: correction.corrected_at,
            reason: correction.reason.clone(),
            supersedes_correction_id: correction.supersedes_correction_id.clone(),
            is_current: current
                .get(correction.target_id.as_str())
                .is_some_and(|current_id| *current_id == id),
        })
        .collect())
}
