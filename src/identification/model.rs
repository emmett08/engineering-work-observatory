//! Public records for exact conditional labour identification.
use super::Exact;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabourBoundInput {
    pub schema: String,
    pub problem_id: String,
    /// All endpoints use hours on one common calendar-time axis.
    pub domain: TimeInterval,
    pub streams: Vec<LabourStream>,
    pub observations: Vec<IntervalObservation>,
    /// Additive windows define a linear functional of cell labour.
    pub target: Vec<TargetWindow>,
    /// Known retained output in declared reference units, held fixed over allocations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_output: Option<Exact>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeInterval {
    pub start: Exact,
    pub end: Exact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabourStream {
    pub id: String,
    pub person_id: String,
    pub task_id: String,
    pub role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalObservation {
    pub id: String,
    pub start: Exact,
    pub end: Exact,
    pub stream_ids: Vec<String>,
    pub lower_person_hours: Exact,
    pub upper_person_hours: Exact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetWindow {
    pub stream_id: String,
    pub start: Exact,
    pub end: Exact,
    pub weight: Exact,
}

#[derive(Clone, Debug, Serialize)]
pub struct LabourBoundReport {
    pub schema: String,
    pub problem_id: String,
    /// Retain the declared constraints and target so the result can be reproduced.
    pub input: LabourBoundInput,
    pub status: BoundStatus,
    pub numerical_semantics: &'static str,
    pub cells: Vec<TimeInterval>,
    pub minimum: Option<Extremum>,
    pub maximum: Option<Extremum>,
    pub productivity: Option<ExactProductivityBounds>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BoundStatus {
    Feasible,
    Inconsistent,
}

#[derive(Clone, Debug, Serialize)]
pub struct Extremum {
    /// Target value in weighted person-hours. For a cohort target weights equal one.
    pub value: Exact,
    pub allocations: Vec<StreamAllocation>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StreamAllocation {
    pub stream_id: String,
    /// One mass per report cell. Divide by cell length to reconstruct a constant allocation.
    pub person_hours: Vec<Exact>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExactProductivityBounds {
    pub lower_reference_units_per_person_hour: Exact,
    pub upper_reference_units_per_person_hour: Exact,
}
