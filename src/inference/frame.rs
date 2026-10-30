//! Define and validate the explicit frame and sampling contract for labour estimation.
use anyhow::{Result, bail, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Version 1 relies on the caller to declare a complete, disjoint person-interval frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabourEstimateInput {
    pub schema: String,
    pub frame_id: String,
    pub frame_complete: bool,
    pub predictions: PredictionProvenance,
    pub design: SamplingDesign,
    pub units: Vec<LabourFrameUnit>,
}

/// Version 2 retains the version 1 statistical input and adds checkable geometry.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabourEstimateInputV2 {
    pub schema: String,
    pub estimation: LabourEstimateInput,
    pub frames: Vec<LabourPersonFrame>,
    pub geometry: Vec<LabourUnitGeometry>,
}

/// The declared observation window for one person, covered exactly by their units.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabourPersonFrame {
    pub person_id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// A half-open person-interval belonging to exactly one statistical frame unit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabourUnitGeometry {
    pub unit_id: String,
    pub person_id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredictionProvenance {
    /// Attests that the entire prediction vector was fixed before sample selection.
    pub frozen_before_sample: bool,
    /// Identifies the frozen predictor, inputs, and independent fitting/freeze record.
    pub provenance: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SamplingDesign {
    /// Known first-order inclusion probabilities; joint design is not supplied.
    General {},
    /// Uniform sampling of exactly `sample_size` distinct units from this frame.
    SimpleRandomWithoutReplacement { sample_size: usize },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabourFrameUnit {
    pub id: String,
    /// A fixed prediction in person-hours, allowed to be negative.
    pub predicted_person_hours: f64,
    pub inclusion_probability: f64,
    pub observation: LabourObservation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum LabourObservation {
    Unselected {},
    Observed { person_hours: f64 },
    Nonresponse { reason: String },
}

impl LabourEstimateInput {
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(
            self.schema == "observatory.labour-estimate-input/1",
            "unsupported labour estimate input schema"
        );
        ensure!(
            !self.frame_id.trim().is_empty(),
            "frame_id must be nonempty"
        );
        ensure!(self.frame_complete, "complete sampling frame is required");
        ensure!(!self.units.is_empty(), "sampling frame must be nonempty");
        ensure!(
            self.predictions.frozen_before_sample,
            "predictions must be frozen before sample selection"
        );
        ensure!(
            !self.predictions.provenance.trim().is_empty(),
            "frozen prediction provenance is required"
        );
        let mut ids = BTreeSet::new();
        let mut selected = 0;
        for unit in &self.units {
            ensure!(!unit.id.trim().is_empty(), "unit id must be nonempty");
            ensure!(ids.insert(&unit.id), "duplicate frame unit {}", unit.id);
            ensure!(
                unit.predicted_person_hours.is_finite(),
                "nonfinite prediction for unit {}",
                unit.id
            );
            ensure!(
                unit.inclusion_probability.is_finite()
                    && unit.inclusion_probability > 0.0
                    && unit.inclusion_probability <= 1.0,
                "inclusion probability must be finite and in (0, 1] for unit {}",
                unit.id
            );
            match &unit.observation {
                LabourObservation::Observed { person_hours } => {
                    ensure!(
                        person_hours.is_finite() && *person_hours >= 0.0,
                        "observed person-hours must be finite and nonnegative for unit {}",
                        unit.id
                    );
                    selected += 1;
                }
                LabourObservation::Unselected {} => ensure!(
                    unit.inclusion_probability < 1.0,
                    "certainty unit {} cannot be unselected",
                    unit.id
                ),
                LabourObservation::Nonresponse { reason } => {
                    ensure!(
                        !reason.trim().is_empty(),
                        "nonresponse reason is required for unit {}",
                        unit.id
                    );
                    bail!(
                        "selected unit {} has nonresponse; complete selected labels are required",
                        unit.id
                    );
                }
            }
        }
        if let SamplingDesign::SimpleRandomWithoutReplacement { sample_size } = self.design {
            ensure!(
                sample_size > 0 && sample_size <= self.units.len(),
                "SRS sample_size must be in 1..=frame size"
            );
            ensure!(
                selected == sample_size,
                "SRS sample_size is {sample_size}, but {selected} units are observed"
            );
            let expected = sample_size as f64 / self.units.len() as f64;
            for unit in &self.units {
                // Accept floating-point representation differences, not different weights.
                ensure!(
                    (unit.inclusion_probability - expected).abs() <= 8.0 * f64::EPSILON * expected,
                    "SRS inclusion probability must equal sample_size/frame size for unit {}",
                    unit.id
                );
            }
        }
        Ok(())
    }
}
