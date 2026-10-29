//! The study's accounting records. Repository observations are retained separately.
use anyhow::{Context, Result, bail, ensure};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

mod corrections;
pub use corrections::{Correction, CorrectionSummary, correction_summaries, effective_records};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub id: String,
    pub kind: String,
    pub data: Value,
}

fn yes() -> bool {
    true
}
fn default_rubric() -> String {
    "requirements-v1".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Protocol {
    pub name: String,
    pub frozen_at: DateTime<Utc>,
    pub acceptance_days: u32,
    pub follow_up_days: u32,
    #[serde(default)]
    pub assessment_tolerance_days: u32,
    #[serde(default = "default_rubric")]
    pub rubric_version: String,
    #[serde(default)]
    pub attended_waiting_counts: Option<bool>,
    #[serde(default)]
    pub eligibility_notes: Option<String>,
    #[serde(default)]
    pub role_convention_notes: Option<String>,
    #[serde(default)]
    pub scope_change_rule: ScopeChangeRule,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeChangeRule {
    BaselineOnly,
    #[default]
    ExcludeFromOutput,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub protocol_id: String,
    pub requirement: String,
    pub task_class: String,
    pub scope_version: String,
    pub enrolled_at: DateTime<Utc>,
    pub score_frozen_at: DateTime<Utc>,
    pub frozen_score: f64,
    #[serde(default = "yes")]
    pub count_output: bool,
    #[serde(default)]
    pub parent_task_id: Option<String>,
    #[serde(default)]
    pub repositories: Vec<String>,
    #[serde(default)]
    pub assigned_policy: Option<String>,
    #[serde(default)]
    pub assignment_unit: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEvent {
    Started,
    Review,
    Accepted,
    Deployed,
    Abandoned,
    LostToFollowUp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lifecycle {
    pub task_id: String,
    pub event: LifecycleEvent,
    pub occurred_at: DateTime<Utc>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub scope_version: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffortMethod {
    #[default]
    Direct,
    Sampled,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStatus {
    #[default]
    Observed,
    Missing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effort {
    pub task_id: String,
    pub person_id: String,
    pub role: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub allocation_fraction: Option<f64>,
    #[serde(default)]
    pub method: EffortMethod,
    #[serde(default)]
    pub inclusion_probability: Option<f64>,
    #[serde(default)]
    pub response_status: ObservationStatus,
    #[serde(default)]
    pub missing_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoverageDomain {
    Effort,
    Lifecycle,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Complete,
    Partial,
    Missing,
}

/// An explicit attestation of the observation frame, not inferred from events.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub task_id: String,
    pub domain: CoverageDomain,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub status: CoverageStatus,
    #[serde(default)]
    pub method: EffortMethod,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentPhase {
    Acceptance,
    FollowUp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assessment {
    pub task_id: String,
    pub phase: AssessmentPhase,
    pub assessed_at: DateTime<Utc>,
    pub status: ObservationStatus,
    pub retained_fraction: Option<f64>,
    pub rubric_version: String,
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(default)]
    pub missing_reason: Option<String>,
    /// Set false for independent reviewer records superseded by adjudication.
    #[serde(default = "yes")]
    pub is_primary: bool,
    #[serde(default)]
    pub scope_version: Option<String>,
    #[serde(default)]
    pub reviewer_id: Option<String>,
    #[serde(default)]
    pub blinded: Option<bool>,
    #[serde(default)]
    pub disagreement_notes: Option<String>,
    #[serde(default)]
    pub adjudication_notes: Option<String>,
    #[serde(default)]
    pub intentional_replacement_notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskLink {
    pub task_id: String,
    pub relation: String,
    pub observed_at: DateTime<Utc>,
    pub method: String,
    #[serde(default)]
    pub source_record_id: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub pull_request: Option<u64>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub ambiguity: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defect {
    pub task_id: String,
    pub detected_at: DateTime<Utc>,
    pub severity: String,
    pub evidence_ids: Vec<String>,
    #[serde(default)]
    pub repair_effort_ids: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskScopeChange {
    pub task_id: String,
    pub occurred_at: DateTime<Utc>,
    pub new_scope_version: String,
    pub reason: String,
    #[serde(default)]
    pub previous_scope_version: Option<String>,
}

/// Provider-specific fields are retained without claiming provider equivalence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiUsage {
    pub purpose: String,
    #[serde(default)]
    pub task_id: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub usage_status: ObservationStatus,
    #[serde(default)]
    pub missing_reason: Option<String>,
    #[serde(flatten)]
    pub fields: BTreeMap<String, Value>,
}

/// Raw source records have adapter-specific schemas; they never count as labour.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceRecord {
    #[serde(flatten)]
    pub fields: BTreeMap<String, Value>,
}

#[derive(Default)]
pub(crate) struct Dataset {
    pub protocols: BTreeMap<String, Protocol>,
    pub tasks: BTreeMap<String, Task>,
    pub lifecycle: Vec<(String, Lifecycle)>,
    pub effort: Vec<(String, Effort)>,
    pub coverage: Vec<(String, Coverage)>,
    pub assessments: Vec<(String, Assessment)>,
    pub ai_usage: Vec<(String, AiUsage)>,
    pub task_links: Vec<(String, TaskLink)>,
    pub defects: Vec<(String, Defect)>,
    pub scope_changes: Vec<(String, TaskScopeChange)>,
    pub source_record_ids: BTreeSet<String>,
    pub ids: BTreeSet<String>,
}

fn decode<T: DeserializeOwned>(record: &Evidence) -> Result<T> {
    serde_json::from_value(record.data.clone())
        .with_context(|| format!("invalid {} record {}", record.kind, record.id))
}

pub(crate) fn dataset(records: &[Evidence]) -> Result<Dataset> {
    let effective = effective_records(records, DateTime::<Utc>::MAX_UTC)?;
    let records = &effective;
    let mut result = Dataset::default();
    let mut ids: BTreeMap<&str, &Evidence> = BTreeMap::new();
    for record in records {
        ensure!(!record.id.trim().is_empty(), "record id must be nonempty");
        if let Some(previous) = ids.get(record.id.as_str()) {
            ensure!(
                *previous == record,
                "conflicting duplicate record id {}",
                record.id
            );
            continue;
        }
        ids.insert(&record.id, record);
        result.ids.insert(record.id.clone());
        match record.kind.as_str() {
            "protocol" => {
                result.protocols.insert(record.id.clone(), decode(record)?);
            }
            "task" => {
                result.tasks.insert(record.id.clone(), decode(record)?);
            }
            "lifecycle" => result.lifecycle.push((record.id.clone(), decode(record)?)),
            "effort" => result.effort.push((record.id.clone(), decode(record)?)),
            "coverage" => result.coverage.push((record.id.clone(), decode(record)?)),
            "assessment" => result
                .assessments
                .push((record.id.clone(), decode(record)?)),
            "ai_usage" => result.ai_usage.push((record.id.clone(), decode(record)?)),
            "task_link" => result.task_links.push((record.id.clone(), decode(record)?)),
            "defect" => result.defects.push((record.id.clone(), decode(record)?)),
            "task_scope_change" => result
                .scope_changes
                .push((record.id.clone(), decode(record)?)),
            "source_record" => {
                ensure!(
                    record.data.is_object(),
                    "source_record {} must be an object",
                    record.id
                );
                let _: SourceRecord = decode(record)?;
                result.source_record_ids.insert(record.id.clone());
            }
            other => bail!("unknown record kind {other} for {}", record.id),
        }
    }
    Ok(result)
}

pub(crate) fn days_after(start: DateTime<Utc>, days: u32) -> Result<DateTime<Utc>> {
    start
        .checked_add_signed(Duration::days(i64::from(days)))
        .context("date horizon overflow")
}

fn nonempty(value: &str, field: &str, id: &str) -> Result<()> {
    ensure!(!value.trim().is_empty(), "{field} must be nonempty in {id}");
    Ok(())
}

/// Validate the complete imported study graph before any calculations or writes.
pub fn validate_records(records: &[Evidence]) -> Result<()> {
    let corrections = correction_summaries(records, DateTime::<Utc>::MAX_UTC)?;
    if corrections.is_empty() {
        return validate_effective_records(records);
    }
    // A later valid amendment must not conceal an invalid historical correction.
    // Apply equal-date changes atomically, then validate every chronological cut.
    let dates: BTreeSet<_> = corrections.iter().map(|c| c.corrected_at).collect();
    for date in dates {
        let effective = effective_records(records, date)?;
        validate_effective_records(&effective)
            .with_context(|| format!("invalid corrected study graph at {date}"))?;
    }
    Ok(())
}

fn validate_effective_records(records: &[Evidence]) -> Result<()> {
    let d = dataset(records)?;
    for (id, p) in &d.protocols {
        nonempty(&p.name, "name", id)?;
        nonempty(&p.rubric_version, "rubric_version", id)?;
        ensure!(
            p.acceptance_days > 0,
            "acceptance_days must be positive in {id}"
        );
        days_after(
            days_after(p.frozen_at, p.acceptance_days)?,
            p.follow_up_days,
        )?;
    }
    for (id, t) in &d.tasks {
        let p = d
            .protocols
            .get(&t.protocol_id)
            .with_context(|| format!("unknown protocol {} in {id}", t.protocol_id))?;
        for (field, value) in [
            ("requirement", &t.requirement),
            ("task_class", &t.task_class),
            ("scope_version", &t.scope_version),
        ] {
            nonempty(value, field, id)?;
        }
        ensure!(
            p.frozen_at <= t.enrolled_at,
            "protocol frozen after enrolment for {id}"
        );
        ensure!(
            t.score_frozen_at <= t.enrolled_at,
            "score frozen after enrolment for {id}"
        );
        ensure!(
            t.frozen_score.is_finite() && t.frozen_score >= 0.0,
            "invalid frozen_score for {id}"
        );
        days_after(
            days_after(t.enrolled_at, p.acceptance_days)?,
            p.follow_up_days,
        )?;
        let mut current = t;
        let mut seen = BTreeSet::from([id.as_str()]);
        while let Some(parent_id) = &current.parent_task_id {
            ensure!(seen.insert(parent_id), "task parent cycle at {id}");
            let parent = d
                .tasks
                .get(parent_id)
                .with_context(|| format!("unknown parent {parent_id} for {id}"))?;
            ensure!(
                parent.protocol_id == t.protocol_id,
                "parent protocol differs for {id}"
            );
            ensure!(
                parent.enrolled_at == t.enrolled_at,
                "parent and child must share the original requirement enrolment in {id}; implementation start belongs in lifecycle records"
            );
            ensure!(
                !(t.count_output && parent.count_output),
                "parent and child both count output for {id}"
            );
            current = parent;
        }
    }
    let require_task = |task_id: &str, id: &str| -> Result<&Task> {
        d.tasks
            .get(task_id)
            .with_context(|| format!("unknown task {task_id} in {id}"))
    };
    for (id, event) in &d.lifecycle {
        let task = require_task(&event.task_id, id)?;
        ensure!(
            event.occurred_at >= task.enrolled_at,
            "lifecycle before enrolment in {id}"
        );
    }
    for task_id in d.tasks.keys() {
        let events: Vec<_> = d
            .lifecycle
            .iter()
            .filter(|(_, e)| &e.task_id == task_id)
            .map(|(_, e)| e)
            .collect();
        let accepted = events
            .iter()
            .filter(|e| e.event == LifecycleEvent::Accepted)
            .map(|e| e.occurred_at)
            .min();
        ensure!(
            events
                .iter()
                .filter(|e| e.event == LifecycleEvent::Accepted)
                .count()
                <= 1,
            "multiple first-acceptance events for {task_id}"
        );
        if accepted.is_some() {
            ensure!(
                !events.iter().any(|e| e.event == LifecycleEvent::Abandoned),
                "accepted task cannot also be permanently abandoned: {task_id}; use retention assessment for post-acceptance loss"
            );
        }
        for event in events {
            if event.event == LifecycleEvent::Deployed {
                ensure!(
                    accepted.is_some_and(|at| at <= event.occurred_at),
                    "deployment before acceptance for {task_id}"
                );
            }
        }
    }
    let mut effort_keys = BTreeSet::new();
    for (id, e) in &d.effort {
        let task = require_task(&e.task_id, id)?;
        nonempty(&e.person_id, "person_id", id)?;
        nonempty(&e.role, "role", id)?;
        ensure!(
            e.end > e.start && e.start >= task.enrolled_at,
            "invalid effort interval in {id}"
        );
        ensure!(
            effort_keys.insert((&e.task_id, &e.person_id, &e.role, e.start, e.end)),
            "duplicate logical effort interval in {id}"
        );
        match e.response_status {
            ObservationStatus::Observed => ensure!(
                e.allocation_fraction
                    .is_some_and(|x| x.is_finite() && (0.0..=1.0).contains(&x)),
                "invalid allocation_fraction in {id}"
            ),
            ObservationStatus::Missing => {
                ensure!(
                    e.allocation_fraction.is_none(),
                    "missing response has allocation in {id}"
                );
                ensure!(
                    e.missing_reason
                        .as_ref()
                        .is_some_and(|x| !x.trim().is_empty()),
                    "missing response needs missing_reason in {id}"
                );
            }
        }
        if e.method == EffortMethod::Sampled {
            ensure!(
                e.inclusion_probability
                    .is_some_and(|x| x.is_finite() && x > 0.0 && x <= 1.0),
                "sampled effort requires 0 < inclusion_probability <= 1 in {id}"
            );
            ensure!(
                (1.0 / e.inclusion_probability.expect("checked probability")).is_finite(),
                "sampling weight exceeds finite numerical range in {id}"
            );
        } else {
            ensure!(
                e.inclusion_probability.is_none() || e.inclusion_probability == Some(1.0),
                "direct effort has nonunit sampling probability in {id}"
            );
        }
    }
    // Sweep person intervals: concurrent task/role allocations must remain <= one.
    let people: BTreeSet<_> = d.effort.iter().map(|(_, e)| &e.person_id).collect();
    for person in people {
        let effort: Vec<_> = d
            .effort
            .iter()
            .filter(|(_, e)| &e.person_id == person)
            .map(|(_, e)| e)
            .collect();
        let points: BTreeSet<_> = effort.iter().flat_map(|e| [e.start, e.end]).collect();
        let points: Vec<_> = points.into_iter().collect();
        for window in points.windows(2) {
            let active: Vec<_> = effort
                .iter()
                .filter(|e| e.start < window[1] && e.end > window[0])
                .collect();
            let allocation: f64 = active.iter().filter_map(|e| e.allocation_fraction).sum();
            ensure!(
                allocation <= 1.0 + 1e-10,
                "overallocated person {person} at {}",
                window[0]
            );
            if active.len() > 1 {
                let first = active[0];
                for e in active.iter().skip(1) {
                    ensure!(
                        e.method == first.method,
                        "mixed direct/sample observations overlap for {person}"
                    );
                    if e.method == EffortMethod::Sampled {
                        ensure!(
                            e.start == first.start
                                && e.end == first.end
                                && e.inclusion_probability == first.inclusion_probability,
                            "overlapping unequal sampled units for {person}"
                        );
                    }
                }
            }
        }
    }
    for (id, c) in &d.coverage {
        let task = require_task(&c.task_id, id)?;
        ensure!(
            c.end > c.start && c.start >= task.enrolled_at,
            "invalid coverage interval in {id}"
        );
        if c.status != CoverageStatus::Complete {
            ensure!(
                c.reason.as_ref().is_some_and(|x| !x.trim().is_empty()),
                "incomplete coverage needs reason in {id}"
            );
        }
        ensure!(
            c.domain != CoverageDomain::Lifecycle || c.method == EffortMethod::Direct,
            "lifecycle coverage cannot use effort sampling in {id}"
        );
    }
    for (id, a) in &d.assessments {
        let task = require_task(&a.task_id, id)?;
        let p = &d.protocols[&task.protocol_id];
        ensure!(
            a.assessed_at >= task.enrolled_at,
            "assessment before enrolment in {id}"
        );
        ensure!(
            a.rubric_version == p.rubric_version,
            "assessment rubric differs from frozen protocol in {id}"
        );
        match a.status {
            ObservationStatus::Observed => {
                ensure!(
                    a.retained_fraction
                        .is_some_and(|x| x.is_finite() && (0.0..=1.0).contains(&x)),
                    "invalid retained_fraction in {id}"
                );
                let accepted = d
                    .lifecycle
                    .iter()
                    .find(|(_, e)| e.task_id == a.task_id && e.event == LifecycleEvent::Accepted)
                    .map(|(_, e)| e.occurred_at)
                    .context(format!("assessment without acceptance in {id}"))?;
                let due = if a.phase == AssessmentPhase::FollowUp {
                    days_after(accepted, p.follow_up_days)?
                } else {
                    accepted
                };
                ensure!(
                    a.assessed_at >= due,
                    "assessment precedes its due time in {id}"
                );
            }
            ObservationStatus::Missing => {
                ensure!(
                    a.retained_fraction.is_none(),
                    "missing assessment has fraction in {id}"
                );
                ensure!(
                    a.missing_reason
                        .as_ref()
                        .is_some_and(|x| !x.trim().is_empty()),
                    "missing assessment needs reason in {id}"
                );
            }
        }
    }
    for (id, a) in &d.ai_usage {
        ensure!(
            ["development", "product_execution", "synthetic"].contains(&a.purpose.as_str()),
            "ai_usage purpose must distinguish development/product_execution/synthetic in {id}"
        );
        if let Some(task_id) = &a.task_id {
            require_task(task_id, id)?;
        }
        if a.usage_status == ObservationStatus::Missing {
            ensure!(
                a.missing_reason
                    .as_ref()
                    .is_some_and(|x| !x.trim().is_empty()),
                "missing usage needs missing_reason in {id}"
            );
        }
    }
    for (id, link) in &d.task_links {
        let task = require_task(&link.task_id, id)?;
        ensure!(
            link.observed_at >= task.enrolled_at,
            "task link predates enrolment in {id}"
        );
        nonempty(&link.relation, "relation", id)?;
        nonempty(&link.method, "method", id)?;
        ensure!(
            link.source_record_id.is_some()
                || link.session_id.is_some()
                || (link.repository.is_some()
                    && (link.pull_request.is_some() || link.commit.is_some())),
            "task link needs a retained source, session, or repository plus PR/commit in {id}"
        );
        if let Some(source_id) = &link.source_record_id {
            ensure!(
                d.source_record_ids.contains(source_id),
                "unknown source_record {source_id} in {id}"
            );
        }
        if let Some(pr) = link.pull_request {
            ensure!(pr > 0, "pull_request must be positive in {id}");
        }
        for (field, value) in [
            ("repository", &link.repository),
            ("commit", &link.commit),
            ("session_id", &link.session_id),
        ] {
            if let Some(value) = value {
                nonempty(value, field, id)?;
            }
        }
    }
    for (id, defect) in &d.defects {
        let task = require_task(&defect.task_id, id)?;
        ensure!(
            defect.detected_at >= task.enrolled_at,
            "defect detection before enrolment in {id}"
        );
        nonempty(&defect.severity, "severity", id)?;
        ensure!(
            !defect.evidence_ids.is_empty(),
            "defect needs evidence_ids in {id}"
        );
        for evidence_id in &defect.evidence_ids {
            ensure!(
                d.ids.contains(evidence_id) && evidence_id != id,
                "unknown/self defect evidence {evidence_id} in {id}"
            );
        }
        for effort_id in &defect.repair_effort_ids {
            let e = d
                .effort
                .iter()
                .find(|(id, _)| id == effort_id)
                .with_context(|| format!("unknown repair effort {effort_id} in {id}"))?;
            ensure!(
                e.1.task_id == defect.task_id,
                "repair effort belongs to another task in {id}"
            );
            ensure!(
                e.1.start >= defect.detected_at,
                "repair effort precedes defect detection in {id}"
            );
        }
    }
    for task_id in d.tasks.keys() {
        let task = &d.tasks[task_id];
        let mut changes: Vec<_> = d
            .scope_changes
            .iter()
            .filter(|(_, c)| &c.task_id == task_id)
            .collect();
        changes.sort_by_key(|(_, c)| c.occurred_at);
        let mut version = task.scope_version.as_str();
        let mut versions = BTreeSet::from([version]);
        for (id, change) in changes {
            ensure!(
                change.occurred_at >= task.enrolled_at,
                "scope change before enrolment in {id}"
            );
            nonempty(&change.reason, "reason", id)?;
            nonempty(&change.new_scope_version, "new_scope_version", id)?;
            ensure!(
                versions.insert(&change.new_scope_version),
                "scope version repeated in {id}"
            );
            if let Some(previous) = &change.previous_scope_version {
                ensure!(previous == version, "scope version chain mismatch in {id}");
            }
            version = &change.new_scope_version;
        }
    }
    for (id, change) in &d.scope_changes {
        require_task(&change.task_id, id)?;
    }
    Ok(())
}
