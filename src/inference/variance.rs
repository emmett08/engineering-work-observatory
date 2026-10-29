//! Compute uncertainty only for an explicitly declared simple random sampling design.
use super::{CheckedSum, SamplingDesign};
use anyhow::{Result, ensure};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VarianceStatus {
    EstimatedSrs,
    KnownZeroCensus,
    Unavailable,
}

#[derive(Clone, Debug, Serialize)]
pub struct DesignVariance {
    pub status: VarianceStatus,
    pub reason: String,
    pub difference_variance_person_hours_squared: Option<f64>,
    pub ht_variance_person_hours_squared: Option<f64>,
    pub difference_standard_error_person_hours: Option<f64>,
    pub ht_standard_error_person_hours: Option<f64>,
}

impl DesignVariance {
    fn unavailable(reason: &str) -> Self {
        Self {
            status: VarianceStatus::Unavailable,
            reason: reason.into(),
            difference_variance_person_hours_squared: None,
            ht_variance_person_hours_squared: None,
            difference_standard_error_person_hours: None,
            ht_standard_error_person_hours: None,
        }
    }

    fn available(status: VarianceStatus, reason: &str, difference: f64, ht: f64) -> Self {
        Self {
            status,
            reason: reason.into(),
            difference_variance_person_hours_squared: Some(difference),
            ht_variance_person_hours_squared: Some(ht),
            difference_standard_error_person_hours: Some(difference.sqrt()),
            ht_standard_error_person_hours: Some(ht.sqrt()),
        }
    }
}

pub(super) fn calculate(
    design: &SamplingDesign,
    population: usize,
    observed: &[f64],
    predictions: &[f64],
) -> Result<DesignVariance> {
    let SamplingDesign::SimpleRandomWithoutReplacement { sample_size } = design else {
        return Ok(DesignVariance::unavailable(
            "First-order inclusion probabilities alone do not specify design variance.",
        ));
    };
    if *sample_size == population {
        return Ok(DesignVariance::available(
            VarianceStatus::KnownZeroCensus,
            "A complete census has zero sampling variance; measurement error remains separate.",
            0.0,
            0.0,
        ));
    }
    if *sample_size < 2 {
        return Ok(DesignVariance::unavailable(
            "Noncensus SRS requires at least two selected labels to estimate design variance.",
        ));
    }
    let population = population as f64;
    let sample_size = *sample_size as f64;
    let multiplier = population * (population - sample_size) / sample_size;
    // Variance is invariant to a common shift. Form residual differences before
    // combining outcome and prediction scales, preserving small outcome variation
    // when predictions have a very large common offset.
    let residual_offsets: Vec<f64> = observed
        .iter()
        .zip(predictions)
        .map(|(y, f)| (y - observed[0]) - (f - predictions[0]))
        .collect();
    let difference = multiplier * sample_variance(&residual_offsets)?;
    let ht = multiplier * sample_variance(observed)?;
    ensure!(
        difference.is_finite() && ht.is_finite(),
        "overflow in design variance estimate"
    );
    Ok(DesignVariance::available(
        VarianceStatus::EstimatedSrs,
        "N^2 (1 - n/N) s^2/n estimates design variance under the declared SRS design; no interval coverage is asserted.",
        difference,
        ht,
    ))
}

fn sample_variance(values: &[f64]) -> Result<f64> {
    let mut sum = CheckedSum::default();
    for value in values {
        sum.add(value - values[0])?;
    }
    let mean = sum.total()? / values.len() as f64;
    let mut squares = CheckedSum::default();
    for value in values {
        squares.add(((value - values[0]) - mean).powi(2))?;
    }
    Ok(squares.total()? / (values.len() - 1) as f64)
}
