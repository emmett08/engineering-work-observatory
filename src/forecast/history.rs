//! Project automatic PR inventory receipts into a bounded count history.
//!
//! Event dates support retrospective reconstruction of the currently visible
//! inventory. They do not establish historical acquisition-time availability.
use crate::{snapshot::Snapshot, store::RetainedRecord};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHistory {
    pub repository_id: u64,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub collection_started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub collection_completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestHistory {
    pub repository_id: u64,
    pub number: u64,
    pub created_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub state: String,
    pub observed_at: DateTime<Utc>,
    #[serde(default)]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub source_record_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentPressure {
    pub status: String,
    pub open_pull_requests: Option<usize>,
    pub open_age_days: Vec<f64>,
    pub open_draft_pull_requests: Option<usize>,
    pub open_review_requested_pull_requests: Option<usize>,
    pub open_at_least_7_days_pull_requests: Option<usize>,
    pub review_request_status: String,
    pub source_record_ids: Vec<String>,
    pub interpretation: String,
    pub review_status: String,
    pub ci_status: String,
    pub oldest_collection_age_seconds: Option<f64>,
}

impl Default for CurrentPressure {
    fn default() -> Self {
        Self {
            status: "unavailable".into(),
            open_pull_requests: None,
            open_age_days: vec![],
            open_draft_pull_requests: None,
            open_review_requested_pull_requests: None,
            open_at_least_7_days_pull_requests: None,
            review_request_status: "unavailable".into(),
            source_record_ids: vec![],
            interpretation: "Last observed open PR inventory; not a simultaneous queue measurement or a causal bottleneck diagnosis".into(),
            review_status: "not_collected".into(),
            ci_status: "not_collected".into(),
            oldest_collection_age_seconds: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedHistory {
    pub schema: String,
    pub mode: String,
    /// Knowledge/issuance cut. This does not extend source observation coverage.
    pub snapshot_cut: DateTime<Utc>,
    /// Conservative common exposure boundary: earliest selected scan start.
    pub observed_cut: DateTime<Utc>,
    /// The portfolio did not exist before its newest selected repository.
    pub history_start: DateTime<Utc>,
    pub complete: bool,
    pub reasons: Vec<String>,
    pub repositories: Vec<RepositoryHistory>,
    pub pull_requests: Vec<PullRequestHistory>,
    #[serde(default)]
    pub source_snapshot_sha256: String,
    #[serde(default)]
    pub collection_started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub collection_completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub current_pressure: CurrentPressure,
    #[serde(default)]
    pub notes: Vec<String>,
}

fn source(record: &RetainedRecord) -> Option<&str> {
    (record.record.kind == "source_record")
        .then(|| record.record.data.get("source").and_then(Value::as_str))
        .flatten()
}

fn payload(record: &RetainedRecord) -> Option<&Value> {
    record.record.data.get("payload")
}

fn timestamp(value: &Value, field: &str) -> Option<DateTime<Utc>> {
    value.get(field)?.as_str()?.parse().ok()
}

fn nullable_timestamp(value: &Value, field: &str) -> Option<Option<DateTime<Utc>>> {
    match value.get(field)? {
        Value::Null => Some(None),
        Value::String(text) => text.parse().ok().map(Some),
        _ => None,
    }
}

fn review_requested(value: &Value) -> Option<bool> {
    let users = value.get("requested_reviewers").and_then(Value::as_array);
    let teams = value.get("requested_teams").and_then(Value::as_array);
    if users.is_some_and(|items| !items.is_empty()) || teams.is_some_and(|items| !items.is_empty())
    {
        Some(true)
    } else {
        users.zip(teams).map(|_| false)
    }
}

fn namespace_id(record: &RetainedRecord) -> Option<u64> {
    record
        .record
        .data
        .get("repository")?
        .as_str()?
        .strip_prefix("github:repository:")?
        .parse()
        .ok()
}

fn prefer<'a>(
    existing: &mut Option<&'a RetainedRecord>,
    candidate: &'a RetainedRecord,
    reasons: &mut Vec<String>,
    context: &str,
) {
    let Some(previous) = existing else {
        *existing = Some(candidate);
        return;
    };
    if candidate.observed_at > previous.observed_at {
        *existing = Some(candidate);
        let obsolete = format!("{context}: conflicting source versions at one receipt cut");
        reasons.retain(|reason| reason != &obsolete);
    } else if candidate.observed_at == previous.observed_at
        && candidate.record.data != previous.record.data
    {
        reasons.push(format!(
            "{context}: conflicting source versions at one receipt cut"
        ));
    }
}

fn scope_matches(scope: &str, name: &str, pulls: bool) -> bool {
    let Ok(url) = reqwest::Url::parse(scope) else {
        return false;
    };
    let expected = format!("/repos/{name}{}", if pulls { "/pulls" } else { "" });
    url.path()
        .trim_end_matches('/')
        .to_ascii_lowercase()
        .ends_with(&expected.to_ascii_lowercase())
        && (!pulls
            || url
                .query_pairs()
                .any(|(key, value)| key == "state" && value == "all"))
}

fn parse_pr(
    record: &RetainedRecord,
    repository_id: u64,
    cut: DateTime<Utc>,
) -> Option<PullRequestHistory> {
    let data = payload(record)?;
    let number = data.get("number")?.as_u64().filter(|n| *n > 0)?;
    data.get("id")?.as_u64().filter(|n| *n > 0)?;
    let created_at = timestamp(data, "created_at")?;
    let merged_at = nullable_timestamp(data, "merged_at")?;
    let closed_at = nullable_timestamp(data, "closed_at")?;
    let state = data.get("state")?.as_str()?;
    if !matches!(state, "open" | "closed")
        || created_at > cut
        || merged_at.is_some_and(|time| time < created_at || time > cut)
        || closed_at.is_some_and(|time| time < created_at || time > cut)
        || (state == "open" && (merged_at.is_some() || closed_at.is_some()))
        || (state == "closed" && closed_at.is_none())
    {
        return None;
    }
    Some(PullRequestHistory {
        repository_id,
        number,
        created_at,
        merged_at,
        closed_at,
        state: state.to_owned(),
        observed_at: record.observed_at,
        source_record_id: record.record.id.clone(),
    })
}

/// Select only automatically collected throughput-profile inventories.
///
/// `repositories` contains full names or numeric repository IDs. An empty list
/// selects every retained GitHub repository identity. Missing required sources
/// return an unavailable history, never a complete history containing zeros.
pub fn prepare(snapshot: &Snapshot, repositories: &[String]) -> Result<PreparedHistory> {
    snapshot.verify()?;
    let mut reasons = Vec::new();
    let mut metadata: BTreeMap<u64, Option<&RetainedRecord>> = BTreeMap::new();
    let mut metadata_reasons: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    for record in &snapshot.records {
        if source(record) != Some("github.repository") {
            continue;
        }
        let Some(id) = payload(record)
            .and_then(|p| p.get("id"))
            .and_then(Value::as_u64)
            .filter(|id| *id > 0)
        else {
            continue;
        };
        if namespace_id(record) != Some(id) {
            continue;
        }
        prefer(
            metadata.entry(id).or_default(),
            record,
            metadata_reasons.entry(id).or_default(),
            "repository metadata",
        );
    }
    let mut selected = BTreeSet::new();
    if repositories.is_empty() {
        selected.extend(metadata.keys().copied());
    } else {
        for requested in repositories {
            let matches: Vec<_> = metadata
                .iter()
                .filter_map(|(id, record)| {
                    let name = payload(record.as_ref()?)?.get("full_name")?.as_str()?;
                    (requested == &id.to_string() || requested.eq_ignore_ascii_case(name))
                        .then_some(*id)
                })
                .collect();
            if matches.len() != 1 {
                reasons.push(format!(
                    "repository {requested}: missing or ambiguous numeric identity"
                ));
            } else {
                selected.insert(matches[0]);
            }
        }
    }
    if selected.is_empty() {
        reasons.push("no selected automatic GitHub repository inventory".into());
    }
    let mut output = PreparedHistory {
        schema: "observatory.throughput-history/1".into(),
        mode: "retrospective_visible_inventory".into(),
        snapshot_cut: snapshot.as_of,
        observed_cut: snapshot.as_of,
        history_start: snapshot.as_of,
        complete: false,
        reasons: vec![],
        repositories: vec![],
        pull_requests: vec![],
        source_snapshot_sha256: snapshot.sha256.clone(),
        collection_started_at: None,
        collection_completed_at: None,
        current_pressure: CurrentPressure::default(),
        notes: vec![
            "Historical merged_at dates reconstruct currently visible PRs; they do not establish past acquisition-time availability or recover deleted history".into(),
            "Merged PRs are observable workflow units, not amounts of functionality, human labour, quality or an AI causal effect".into(),
            "Counts before the newest selected repository existed and partial initial/current time bins must be excluded".into(),
        ],
    };
    for repository_id in selected {
        reasons.extend(metadata_reasons.remove(&repository_id).unwrap_or_default());
        let Some(meta) = metadata.get(&repository_id).and_then(|value| *value) else {
            continue;
        };
        let Some(data) = payload(meta) else {
            continue;
        };
        let Some(full_name) = data.get("full_name").and_then(Value::as_str) else {
            reasons.push(format!("repository {repository_id}: missing full_name"));
            continue;
        };
        let Some(created_at) = timestamp(data, "created_at").filter(|date| *date <= snapshot.as_of)
        else {
            reasons.push(format!("repository {full_name}: invalid created_at"));
            continue;
        };
        output.repositories.push(RepositoryHistory {
            repository_id,
            full_name: full_name.into(),
            created_at,
            collection_started_at: None,
            collection_completed_at: None,
        });
        let mut summary = None;
        for record in &snapshot.records {
            if source(record) != Some("github.throughput_collection") {
                continue;
            }
            let namespace = record
                .record
                .data
                .get("repository")
                .and_then(Value::as_str)
                .unwrap_or("");
            if namespace_id(record) == Some(repository_id)
                || namespace.eq_ignore_ascii_case(&format!("github:{full_name}"))
            {
                prefer(&mut summary, record, &mut reasons, full_name);
            }
        }
        let Some(summary) = summary else {
            reasons.push(format!(
                "repository {full_name}: missing automatic throughput-profile collection summary"
            ));
            continue;
        };
        output
            .current_pressure
            .source_record_ids
            .push(summary.record.id.clone());
        let Some(summary_data) = payload(summary) else {
            reasons.push(format!(
                "repository {full_name}: missing collection payload"
            ));
            continue;
        };
        if summary_data.get("schema").and_then(Value::as_str)
            != Some("observatory.github-throughput-collection/1")
            || summary_data.get("profile").and_then(Value::as_str) != Some("throughput")
        {
            reasons.push(format!(
                "repository {full_name}: unsupported automatic collection summary"
            ));
            continue;
        }
        let span =
            timestamp(summary_data, "started_at").zip(timestamp(summary_data, "completed_at"));
        let Some((started, completed)) =
            span.filter(|(start, end)| start <= end && *end <= summary.observed_at)
        else {
            reasons.push(format!(
                "repository {full_name}: invalid acquisition interval"
            ));
            continue;
        };
        output.collection_started_at = Some(
            output
                .collection_started_at
                .map_or(started, |previous| previous.min(started)),
        );
        output.collection_completed_at = Some(
            output
                .collection_completed_at
                .map_or(completed, |previous| previous.max(completed)),
        );
        output.observed_cut = output.observed_cut.min(started);
        if let Some(repository) = output.repositories.last_mut() {
            repository.collection_started_at = Some(started);
            repository.collection_completed_at = Some(completed);
        }
        let Some(coverage) = summary_data.get("coverage").and_then(Value::as_array) else {
            reasons.push(format!(
                "repository {full_name}: missing automatic scope coverage"
            ));
            continue;
        };
        let mut expected_prs = None;
        for family in ["github.repository", "github.pull_request"] {
            let scopes: Vec<_> = coverage
                .iter()
                .filter(|entry| entry.get("source").and_then(Value::as_str) == Some(family))
                .collect();
            if scopes.len() != 1 {
                reasons.push(format!(
                    "repository {full_name}: missing or ambiguous {family} coverage"
                ));
                continue;
            }
            let entry = scopes[0];
            let valid = entry.get("state").and_then(Value::as_str) == Some("complete")
                && entry
                    .get("pages")
                    .and_then(Value::as_u64)
                    .is_some_and(|pages| pages > 0)
                && entry
                    .get("scope")
                    .and_then(Value::as_str)
                    .is_some_and(|scope| {
                        scope_matches(scope, full_name, family == "github.pull_request")
                    });
            if !valid {
                reasons.push(format!(
                    "repository {full_name}: {family} retrieval incomplete or scope mismatch"
                ));
            }
            if family == "github.pull_request" {
                expected_prs = entry.get("records").and_then(Value::as_u64);
            } else if entry.get("records").and_then(Value::as_u64) != Some(1) {
                reasons.push(format!("repository {full_name}: metadata count is not one"));
            }
        }
        if meta.raw_sha256 != summary.raw_sha256 {
            reasons.push(format!("repository {full_name}: metadata and inventory summary have different acquisition receipts"));
        }
        let mut pulls: BTreeMap<u64, &RetainedRecord> = BTreeMap::new();
        let mut list_numbers = BTreeSet::new();
        for record in &snapshot.records {
            let family = source(record);
            if !matches!(
                family,
                Some("github.pull_request" | "github.pull_request_detail")
            ) || namespace_id(record) != Some(repository_id)
                || record.raw_sha256 != summary.raw_sha256
            {
                continue;
            }
            let Some(row) = parse_pr(record, repository_id, completed.min(snapshot.as_of)) else {
                reasons.push(format!(
                    "repository {full_name}: malformed, contradictory or future-dated PR source {}",
                    record.record.id
                ));
                continue;
            };
            if row.created_at < created_at {
                reasons.push(format!(
                    "repository {full_name}: PR {} predates repository creation",
                    row.number
                ));
            }
            if family == Some("github.pull_request") {
                list_numbers.insert(row.number);
            }
            if let Some(previous) = pulls.get(&row.number) {
                let rank = |entry: &RetainedRecord| {
                    (
                        entry.observed_at,
                        source(entry) == Some("github.pull_request_detail"),
                    )
                };
                if rank(record) > rank(previous) {
                    pulls.insert(row.number, record);
                } else if rank(record) == rank(previous) && payload(record) != payload(previous) {
                    reasons.push(format!(
                        "repository {full_name}: PR {} has conflicting latest variants",
                        row.number
                    ));
                }
            } else {
                pulls.insert(row.number, record);
            }
        }
        if let Some(unique) = summary_data
            .get("unique_pull_requests")
            .and_then(Value::as_u64)
        {
            if expected_prs.is_none_or(|raw| unique > raw)
                || summary_data
                    .get("duplicate_pull_request_rows")
                    .and_then(Value::as_u64)
                    != expected_prs.map(|raw| raw.saturating_sub(unique))
            {
                reasons.push(format!(
                    "repository {full_name}: inconsistent automatic duplicate accounting"
                ));
            }
            expected_prs = Some(unique);
        }
        if expected_prs != Some(list_numbers.len() as u64) {
            reasons.push(format!("repository {full_name}: automatic inventory count does not match retained unique PR list"));
        }
        for (_, record) in pulls {
            if let Some(row) = parse_pr(record, repository_id, completed.min(snapshot.as_of)) {
                if !list_numbers.contains(&row.number) {
                    reasons.push(format!("repository {full_name}: PR {} detail absent from the same complete inventory", row.number));
                }
                output.pull_requests.push(row);
            }
        }
    }
    output.history_start = output
        .repositories
        .iter()
        .map(|repository| repository.created_at)
        .max()
        .unwrap_or(snapshot.as_of);
    output
        .pull_requests
        .sort_by_key(|row| (row.repository_id, row.number));
    reasons.sort();
    reasons.dedup();
    output.complete = reasons.is_empty();
    output.reasons = reasons;
    if output.complete {
        let mut ages: Vec<_> = output
            .pull_requests
            .iter()
            .filter(|row| row.state == "open")
            .map(|row| (row.observed_at - row.created_at).num_milliseconds() as f64 / 86_400_000.0)
            .collect();
        ages.sort_by(f64::total_cmp);
        output.current_pressure.status = "last_observed_inventory".into();
        output.current_pressure.open_pull_requests = Some(ages.len());
        output.current_pressure.open_at_least_7_days_pull_requests =
            Some(ages.iter().filter(|age| **age >= 7.0).count());
        output.current_pressure.open_age_days = ages;
        let records: BTreeMap<_, _> = snapshot
            .records
            .iter()
            .map(|record| (record.record.id.as_str(), record))
            .collect();
        let open_payloads: Vec<_> = output
            .pull_requests
            .iter()
            .filter(|row| row.state == "open")
            .filter_map(|row| {
                records
                    .get(row.source_record_id.as_str())
                    .and_then(|record| payload(record))
            })
            .collect();
        let drafts: Option<Vec<_>> = open_payloads
            .iter()
            .map(|value| value.get("draft").and_then(Value::as_bool))
            .collect();
        output.current_pressure.open_draft_pull_requests =
            drafts.map(|flags| flags.into_iter().filter(|flag| *flag).count());
        let requests: Option<Vec<_>> = open_payloads
            .iter()
            .map(|value| review_requested(value))
            .collect();
        output.current_pressure.open_review_requested_pull_requests =
            requests.map(|flags| flags.into_iter().filter(|flag| *flag).count());
        output.current_pressure.review_request_status = if output
            .current_pressure
            .open_review_requested_pull_requests
            .is_some()
        {
            "observed_request_fields"
        } else {
            "optional_fields_unavailable"
        }
        .into();
        output.current_pressure.source_record_ids.extend(
            output
                .pull_requests
                .iter()
                .filter(|row| row.state == "open")
                .map(|row| row.source_record_id.clone()),
        );
        output.current_pressure.oldest_collection_age_seconds =
            Some((snapshot.as_of - output.observed_cut).num_milliseconds() as f64 / 1000.0);
        output.current_pressure.interpretation = "Last observed open PR inventory. Draft, requested-review and age-at-least-seven-days predicates may overlap. Ages are at acquisition receipt; freshness is separate. Review requests do not establish incomplete review, blocked capacity or a causal bottleneck. These current fields are not historical forecast features".into();
    }
    Ok(output)
}
