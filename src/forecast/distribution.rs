//! Empirical count distributions and proper scores, without a parametric count law.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CountMass {
    pub count: u64,
    pub probability: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EmpiricalDistribution {
    pub observations: usize,
    pub mass: Vec<CountMass>,
    pub mean: f64,
    /// The inverse empirical CDF at 0.5; for an even sample this is the lower median.
    pub median: u64,
    pub quantile_10: u64,
    pub quantile_90: u64,
    /// Mass actually included by the inclusive empirical quantile endpoints.
    pub quantile_range_mass: f64,
    pub historical_minimum: u64,
    pub historical_maximum: u64,
}

impl EmpiricalDistribution {
    pub fn from_counts(counts: &[u64]) -> Result<Self> {
        ensure!(
            !counts.is_empty(),
            "an empirical forecast needs observations"
        );
        ensure!(
            counts.iter().all(|&v| v <= (1_u64 << 53)),
            "count exceeds exact integer representation for floating-point summaries"
        );
        let mut sorted = counts.to_vec();
        sorted.sort_unstable();
        let n = sorted.len();
        let quantile = |p: f64| sorted[((p * n as f64).ceil() as usize).saturating_sub(1)];
        let quantile_10 = quantile(0.1);
        let quantile_90 = quantile(0.9);
        let mut frequencies = BTreeMap::new();
        for &count in counts {
            *frequencies.entry(count).or_insert(0_usize) += 1;
        }
        let mass = frequencies
            .into_iter()
            .map(|(count, frequency)| CountMass {
                count,
                probability: frequency as f64 / n as f64,
            })
            .collect();
        Ok(Self {
            observations: n,
            mass,
            mean: counts.iter().map(|&v| u128::from(v)).sum::<u128>() as f64 / n as f64,
            median: quantile(0.5),
            quantile_10,
            quantile_90,
            quantile_range_mass: sorted
                .iter()
                .filter(|&&v| v >= quantile_10 && v <= quantile_90)
                .count() as f64
                / n as f64,
            historical_minimum: sorted[0],
            historical_maximum: sorted[n - 1],
        })
    }
}

/// Integer-count RPS/CRPS, lower is better. Sorting avoids looping over count support.
///
/// For sorted x_i, half the sum over all ordered pair distances is
/// sum_i (2*i + 1 - n)*x_i. Division by n² gives the second CRPS term.
/// This is a score for the issued empirical distribution, not an unbiased estimate
/// of the score of an unknown population distribution.
pub fn empirical_rps(counts: &[u64], outcome: u64) -> Result<f64> {
    ensure!(!counts.is_empty(), "RPS requires a nonempty forecast");
    ensure!(
        outcome <= (1_u64 << 53) && counts.iter().all(|&v| v <= (1_u64 << 53)),
        "count exceeds exact integer representation for scoring"
    );
    let mut sorted = counts.to_vec();
    sorted.sort_unstable();
    let n = sorted.len() as f64;
    let distance = sorted
        .iter()
        .map(|&v| v.abs_diff(outcome) as f64 / n)
        .sum::<f64>();
    // Use adjacent gaps and nonnegative weights instead of a cancelling signed sum.
    let half_pair_distance = sorted
        .windows(2)
        .enumerate()
        .map(|(i, pair)| {
            let left = (i + 1) as f64;
            pair[1].abs_diff(pair[0]) as f64 * (left / n) * ((n - left) / n)
        })
        .sum::<f64>();
    let score = distance - half_pair_distance;
    ensure!(score.is_finite(), "nonfinite RPS");
    // Rounding can produce a tiny negative number at a degenerate boundary.
    Ok(score.max(0.0))
}
