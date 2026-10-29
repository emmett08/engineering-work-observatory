//! Estimate complete-frame labour totals using frozen predictions and a probability sample.
use anyhow::{Result, ensure};
use serde::Serialize;

mod frame;
mod variance;
pub use frame::{
    LabourEstimateInput, LabourFrameUnit, LabourObservation, PredictionProvenance, SamplingDesign,
};
pub use variance::{DesignVariance, VarianceStatus};

#[derive(Clone, Debug, Serialize)]
pub struct LabourEstimate {
    pub schema: String,
    pub frame_id: String,
    pub frame_size: usize,
    pub observed_sample_size: usize,
    pub predictions: PredictionProvenance,
    pub design: SamplingDesign,
    pub prediction_total_person_hours: f64,
    pub ht_total_person_hours: f64,
    /// Signed difference estimate: never clipped to the physical outcome range.
    pub difference_total_person_hours: f64,
    pub variance: DesignVariance,
    pub limitations: Vec<String>,
}

/// Return HT and fixed-prediction difference estimates of the same finite-frame total.
///
/// Design-unbiasedness requires truthful complete-frame and frozen-prediction
/// attestations, accurate observed labels, and the declared inclusion probabilities.
/// Selected nonresponse is an error, never an observed zero. General designs supply
/// point estimates only; declaring SRS without replacement also enables its variance
/// estimate when there are at least two observations (or zero variance for a census).
pub fn estimate_labour(input: &LabourEstimateInput) -> Result<LabourEstimate> {
    input.validate()?;
    let mut predicted = CheckedSum::default();
    let mut observed = Vec::new();
    let mut sampled_predictions = Vec::new();
    for unit in &input.units {
        predicted.add(unit.predicted_person_hours)?;
    }
    for unit in &input.units {
        if let LabourObservation::Observed { person_hours } = unit.observation {
            observed.push(person_hours);
            sampled_predictions.push(unit.predicted_person_hours);
        }
    }
    let (ht, difference) = match input.design {
        SamplingDesign::General {} => general_totals(input)?,
        SamplingDesign::SimpleRandomWithoutReplacement { sample_size } => {
            srs_totals(input, sample_size)?
        }
    };
    let variance = variance::calculate(
        &input.design,
        input.units.len(),
        &observed,
        &sampled_predictions,
    )?;
    Ok(LabourEstimate {
        schema: "observatory.labour-estimate/1".into(),
        frame_id: input.frame_id.clone(),
        frame_size: input.units.len(),
        observed_sample_size: observed.len(),
        predictions: input.predictions.clone(),
        design: input.design.clone(),
        prediction_total_person_hours: predicted.total()?,
        ht_total_person_hours: ht,
        difference_total_person_hours: difference,
        variance,
        limitations: vec![
            "Completeness, sampling probabilities, accurate labels and predictions fixed independently of the realised sample are supplied attestations, not verified facts.".into(),
            "Difference estimates are signed and are not clipped. A frozen predictor can increase design variance; fitting on this sample does not inherit the fixed-prediction guarantee.".into(),
            "A design variance estimate does not by itself supply confidence-interval coverage. No confidence interval or unbiased productivity ratio is claimed.".into(),
        ],
    })
}

fn general_totals(input: &LabourEstimateInput) -> Result<(f64, f64)> {
    let mut ht = CheckedSum::default();
    let mut difference = CheckedSum::default();
    for unit in &input.units {
        difference.add(unit.predicted_person_hours)?;
    }
    for unit in &input.units {
        if let LabourObservation::Observed { person_hours } = unit.observation {
            ht.add(person_hours / unit.inclusion_probability)?;
            // Keep small labels from disappearing when subtracting a large prediction.
            difference.add(person_hours / unit.inclusion_probability)?;
            difference.add(-unit.predicted_person_hours / unit.inclusion_probability)?;
        }
    }
    Ok((ht.total()?, difference.total()?))
}

fn srs_totals(input: &LabourEstimateInput, sample_size: usize) -> Result<(f64, f64)> {
    // For SRS, replacing every prediction f by f-a leaves the difference estimate
    // unchanged: M*a - (M/n)*n*a = 0. Cancel this offset before floating-point
    // weighting, and derive the expansion from the declared integer sizes rather
    // than repeatedly inverting a rounded input inclusion probability.
    let expansion = input.units.len() as f64 / sample_size as f64;
    let mut observed = CheckedSum::default();
    for unit in &input.units {
        if let LabourObservation::Observed { person_hours } = unit.observation {
            observed.add(person_hours)?;
        }
    }
    let ht = expansion * observed.total()?;
    ensure!(ht.is_finite(), "overflow in SRS estimation arithmetic");
    // For a census all auxiliary terms cancel algebraically, even when their
    // individual scales would hide a small observed total in floating point.
    if sample_size == input.units.len() {
        return Ok((ht, ht));
    }
    let anchor = input.units[0].predicted_person_hours;
    let mut centered_predictions = CheckedSum::default();
    let mut centered_sample_residuals = CheckedSum::default();
    for unit in &input.units {
        let centered = unit.predicted_person_hours - anchor;
        centered_predictions.add(centered)?;
        if let LabourObservation::Observed { person_hours } = unit.observation {
            centered_sample_residuals.add(person_hours)?;
            centered_sample_residuals.add(-centered)?;
        }
    }
    centered_predictions.add(expansion * centered_sample_residuals.total()?)?;
    Ok((ht, centered_predictions.total()?))
}

/// Compensate finite floating-point additions; reject overflow rather than emit null JSON.
#[derive(Default)]
struct CheckedSum {
    sum: f64,
    correction: f64,
}

impl CheckedSum {
    fn add(&mut self, value: f64) -> Result<()> {
        ensure!(value.is_finite(), "nonfinite estimation arithmetic");
        let next = self.sum + value;
        ensure!(next.is_finite(), "overflow in estimation sum");
        self.correction += if self.sum.abs() >= value.abs() {
            (self.sum - next) + value
        } else {
            (value - next) + self.sum
        };
        ensure!(self.correction.is_finite(), "overflow in sum correction");
        self.sum = next;
        Ok(())
    }

    fn total(&self) -> Result<f64> {
        let total = self.sum + self.correction;
        ensure!(total.is_finite(), "overflow in corrected estimation sum");
        Ok(total)
    }
}
