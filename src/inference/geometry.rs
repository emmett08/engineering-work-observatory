//! Check declared person-interval coverage and capacity before statistical estimation.
use super::{
    LabourEstimate, LabourEstimateInputV2, LabourObservation, LabourPersonFrame,
    LabourUnitGeometry, estimate_labour,
};
use anyhow::{Result, ensure};
use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use num_rational::BigRational;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Retain the checked declarations alongside the unchanged statistical result.
#[derive(Clone, Debug, Serialize)]
pub struct LabourEstimateV2 {
    pub schema: String,
    pub frames: Vec<LabourPersonFrame>,
    pub geometry: Vec<LabourUnitGeometry>,
    pub estimation: LabourEstimate,
    pub limitations: Vec<String>,
}

/// Estimate a frame whose supplied geometry partitions each declared person's window.
///
/// Geometry checks establish internal coverage, disjointness and observed capacity.
/// They do not establish the roster's completeness or the accuracy of observations.
/// All statistical assumptions and signed-estimate semantics of version 1 remain.
pub fn estimate_labour_v2(input: &LabourEstimateInputV2) -> Result<LabourEstimateV2> {
    ensure!(
        input.schema == "observatory.labour-estimate-input/2",
        "unsupported labour estimate geometry input schema"
    );
    input.estimation.validate()?;
    validate_geometry(input)?;
    Ok(LabourEstimateV2 {
        schema: "observatory.labour-estimate/2".into(),
        frames: input.frames.clone(),
        geometry: input.geometry.clone(),
        estimation: estimate_labour(&input.estimation)?,
        limitations: vec![
            "Geometry checks establish exact coverage and nonoverlap of supplied person-intervals within declared person windows, not completeness of the roster or coverage of actual work.".into(),
            "Observed labour is checked against elapsed interval capacity using exact rational comparison of the supplied f64 value and timestamp nanoseconds. Statistical totals and variances retain floating-point arithmetic.".into(),
            "Accurate observations, truthful frozen predictions and compliance with the declared sampling design still require evidence from collection.".into(),
        ],
    })
}

fn validate_geometry(input: &LabourEstimateInputV2) -> Result<()> {
    ensure!(!input.frames.is_empty(), "person frames must be nonempty");
    let mut frames = BTreeMap::new();
    for frame in &input.frames {
        ensure!(
            !frame.person_id.trim().is_empty(),
            "person_id must be nonempty"
        );
        ensure!(
            frame.start < frame.end,
            "person frame must have positive duration for {}",
            frame.person_id
        );
        ensure!(
            frames.insert(frame.person_id.as_str(), frame).is_none(),
            "duplicate person frame {}",
            frame.person_id
        );
    }
    let units: BTreeMap<_, _> = input
        .estimation
        .units
        .iter()
        .map(|unit| (unit.id.as_str(), unit))
        .collect();
    let mut seen = BTreeSet::new();
    let mut intervals = BTreeMap::<&str, Vec<&LabourUnitGeometry>>::new();
    for geometry in &input.geometry {
        let Some(unit) = units.get(geometry.unit_id.as_str()) else {
            anyhow::bail!("geometry references unknown unit {}", geometry.unit_id);
        };
        ensure!(
            seen.insert(geometry.unit_id.as_str()),
            "duplicate geometry for unit {}",
            geometry.unit_id
        );
        let Some(frame) = frames.get(geometry.person_id.as_str()) else {
            anyhow::bail!(
                "geometry for unit {} references unknown person {}",
                geometry.unit_id,
                geometry.person_id
            );
        };
        ensure!(
            geometry.start < geometry.end,
            "unit {} must have positive duration",
            geometry.unit_id
        );
        ensure!(
            geometry.start >= frame.start && geometry.end <= frame.end,
            "unit {} is outside the declared person frame",
            geometry.unit_id
        );
        if let LabourObservation::Observed { person_hours } = unit.observation {
            // Convert the supplied binary float exactly. Comparing against a rounded
            // f64 duration could accept labour slightly above physical capacity.
            let labour = BigRational::from_float(person_hours)
                .expect("statistical input validation requires finite observations");
            let capacity = BigRational::new(
                BigInt::from(elapsed_nanoseconds(geometry.start, geometry.end)),
                BigInt::from(3_600_000_000_000_u64),
            );
            ensure!(
                labour <= capacity,
                "observed labour exceeds interval capacity for unit {}",
                geometry.unit_id
            );
        }
        intervals
            .entry(geometry.person_id.as_str())
            .or_default()
            .push(geometry);
    }
    ensure!(
        seen.len() == units.len(),
        "geometry is missing for one or more frame units"
    );
    for (person_id, frame) in frames {
        let Some(person_intervals) = intervals.get_mut(person_id) else {
            anyhow::bail!("missing interval coverage for person {person_id}");
        };
        person_intervals.sort_by_key(|geometry| (geometry.start, geometry.end));
        let mut cursor = frame.start;
        for geometry in person_intervals {
            ensure!(
                geometry.start >= cursor,
                "overlapping intervals for person {person_id} at unit {}",
                geometry.unit_id
            );
            ensure!(
                geometry.start == cursor,
                "gap in interval coverage for person {person_id} before unit {}",
                geometry.unit_id
            );
            cursor = geometry.end;
        }
        ensure!(
            cursor == frame.end,
            "gap at end of interval coverage for person {person_id}"
        );
    }
    Ok(())
}

/// Avoid the approximately 292-year range limit of Duration::num_nanoseconds.
fn elapsed_nanoseconds(start: DateTime<Utc>, end: DateTime<Utc>) -> i128 {
    let elapsed = end.signed_duration_since(start);
    i128::from(elapsed.num_seconds()) * 1_000_000_000 + i128::from(elapsed.subsec_nanos())
}
