//! Numerically bounded Poisson benchmark; never selected as the operational model.
use anyhow::{Result, ensure};

pub(super) struct PoissonScore {
    pub rps: f64,
    pub median: u64,
    pub lower: u64,
    pub upper: u64,
    pub omitted_tail_bound: f64,
}

pub(super) fn score(mean: f64, outcome: u64) -> Result<PoissonScore> {
    ensure!(
        mean.is_finite() && (0.0..=100_000.0).contains(&mean),
        "Poisson benchmark mean exceeds numerical resource limit"
    );
    if mean == 0.0 {
        return Ok(PoissonScore {
            rps: outcome as f64,
            median: 0,
            lower: 0,
            upper: 0,
            omitted_tail_bound: 0.0,
        });
    }
    let mode = mean.floor() as usize;
    let log_factorial: f64 = (1..=mode).map(|k| (k as f64).ln()).sum();
    let p_mode = (-mean + mode as f64 * mean.ln() - log_factorial).exp();
    let mut probabilities = vec![0.0; mode + 1];
    probabilities[mode] = p_mode;
    for k in (1..=mode).rev() {
        probabilities[k - 1] = probabilities[k] * k as f64 / mean;
    }
    let mut k = mode;
    let tail_bound;
    loop {
        let next = probabilities[k] * mean / (k + 1) as f64;
        let ratio = mean / (k + 2) as f64;
        let bound = if ratio < 1.0 {
            next / (1.0 - ratio)
        } else {
            f64::INFINITY
        };
        if bound <= 1e-12 {
            tail_bound = bound;
            break;
        }
        probabilities.push(next);
        k += 1;
        ensure!(k <= 110_000, "Poisson benchmark support limit reached");
    }
    let total: f64 = probabilities.iter().sum();
    ensure!(
        total.is_finite() && total > 0.0,
        "invalid Poisson benchmark normalization"
    );
    // Explicitly score this numerical approximation, retaining its truncation bound.
    for p in &mut probabilities {
        *p /= total;
    }
    let mut cdf = 0.0;
    let mut weighted_prefix = 0.0;
    let mut first_distance = 0.0;
    let mut half_pair_distance = 0.0;
    let mut median = None;
    let mut lower = None;
    let mut upper = None;
    for (value, &p) in probabilities.iter().enumerate() {
        let x = value as f64;
        first_distance += p * (value as u64).abs_diff(outcome) as f64;
        half_pair_distance += p * (x * cdf - weighted_prefix);
        cdf += p;
        weighted_prefix += p * x;
        if lower.is_none() && cdf >= 0.1 {
            lower = Some(value as u64);
        }
        if median.is_none() && cdf >= 0.5 {
            median = Some(value as u64);
        }
        if upper.is_none() && cdf >= 0.9 {
            upper = Some(value as u64);
        }
    }
    Ok(PoissonScore {
        rps: (first_distance - half_pair_distance).max(0.0),
        median: median.unwrap_or(k as u64),
        lower: lower.unwrap_or(k as u64),
        upper: upper.unwrap_or(k as u64),
        omitted_tail_bound: tail_bound,
    })
}
