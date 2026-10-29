//! Deterministic enclosures from direct interval mass and the attention bound.
//! Single-interval extrema are sharp using that observation alone; sums need not
//! be jointly attainable when person capacity couples roles or tasks.
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct LabourBounds {
    pub lower_person_hours: f64,
    pub upper_person_hours: f64,
    pub method: LabourBoundsMethod,
    /// The mathematical enclosure need not contain exact values after rounding.
    pub numerical_semantics: &'static str,
}

const NUMERICAL_SEMANTICS: &str = "floating_point_approximation_without_outward_rounding";

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabourBoundsMethod {
    /// Sharp conditional on one observed interval, without other constraints.
    SharpFromSingleDirectInterval,
    /// Contains the total; does not assert joint attainability of its endpoints.
    ConservativeSumOfDirectIntervalBounds,
}

#[derive(Clone, Debug, Serialize)]
pub struct BoundaryEffortBounds {
    pub effort_id: String,
    pub overlap_start: DateTime<Utc>,
    pub overlap_end: DateTime<Utc>,
    pub bounds: LabourBounds,
}

/// Conditional productivity enclosure from known output and positive labour.
/// Labour and productivity endpoints use distinct units and are not interchangeable.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ProductivityBounds {
    pub lower_reference_units_per_person_hour: f64,
    pub upper_reference_units_per_person_hour: f64,
    pub method: ProductivityBoundsMethod,
    pub numerical_semantics: &'static str,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductivityBoundsMethod {
    /// Conditional on known output; does not assert joint attainability or causality.
    ConservativeRatioFromKnownOutputAndDirectLabourBounds,
}

impl LabourBounds {
    pub(super) fn sum<'a>(bounds: impl Iterator<Item = &'a Self>) -> Self {
        let mut result = Self {
            lower_person_hours: 0.0,
            upper_person_hours: 0.0,
            method: LabourBoundsMethod::ConservativeSumOfDirectIntervalBounds,
            numerical_semantics: NUMERICAL_SEMANTICS,
        };
        for bound in bounds {
            result.lower_person_hours += bound.lower_person_hours;
            result.upper_person_hours += bound.upper_person_hours;
        }
        result
    }

    pub(super) fn is_finite(&self) -> bool {
        self.lower_person_hours.is_finite() && self.upper_person_hours.is_finite()
    }
}

/// Known nonnegative output Q and 0 < H_lower <= H <= H_upper imply
/// Q/H_upper <= Q/H <= Q/H_lower. No finite result is supplied when the lower
/// denominator is zero or floating-point division exceeds the finite range.
pub(super) fn productivity(output: f64, labour: &LabourBounds) -> Option<ProductivityBounds> {
    debug_assert!(output.is_finite() && output >= 0.0);
    if labour.lower_person_hours <= 0.0 {
        return None;
    }
    let lower = output / labour.upper_person_hours;
    let upper = output / labour.lower_person_hours;
    (lower.is_finite() && upper.is_finite()).then_some(ProductivityBounds {
        lower_reference_units_per_person_hour: lower,
        upper_reference_units_per_person_hour: upper,
        method: ProductivityBoundsMethod::ConservativeRatioFromKnownOutputAndDirectLabourBounds,
        numerical_semantics: NUMERICAL_SEMANTICS,
    })
}

/// For a validated observed direct average, intersect its interval with a window.
/// If total mass is m, inside duration u and outside duration v, the range is
/// [max(0, m-v), min(m, u)]. Filling inside or outside first attains each bound.
/// No uniform allocation, response model, or sampling estimate is assumed.
pub(super) fn direct_interval(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    allocation: f64,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
) -> LabourBounds {
    debug_assert!(start < end && window_start <= window_end);
    debug_assert!(allocation.is_finite() && (0.0..=1.0).contains(&allocation));
    let left = start.max(window_start);
    let right = end.min(window_end);
    let (lower, upper) = if left >= right || allocation == 0.0 {
        (0.0, 0.0)
    } else {
        let inside = super::elapsed_hours(left, right);
        if allocation == 1.0 {
            (inside, inside)
        } else {
            // Measure each outside piece directly, preserving short durations
            // when the full recorded interval is much longer than the overlap.
            let outside = super::elapsed_hours(start, left) + super::elapsed_hours(right, end);
            let upper = (super::elapsed_hours(start, end) * allocation).min(inside);
            // Algebraically m-v, without subtracting two large near-equal masses
            // when allocation is near one and the overlap is very short.
            let lower = (inside * allocation - outside * (1.0 - allocation)).max(0.0);
            (lower.min(upper), upper)
        }
    };
    LabourBounds {
        lower_person_hours: lower,
        upper_person_hours: upper,
        method: LabourBoundsMethod::SharpFromSingleDirectInterval,
        numerical_semantics: NUMERICAL_SEMANTICS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn at(hours: i64) -> DateTime<Utc> {
        DateTime::UNIX_EPOCH + Duration::hours(hours)
    }

    #[test]
    fn all_overlap_geometries_match_attainable_discrete_extrema() {
        // Exhaust all binary trajectories on six unit cells. For integral mass,
        // filling selected cells realises the continuous attention-bound extrema.
        for mass in 0..=6 {
            for left in -2..=8 {
                for right in left..=8 {
                    let mut minimum = f64::INFINITY;
                    let mut maximum = 0.0_f64;
                    for bits in 0_u32..64 {
                        if bits.count_ones() != mass {
                            continue;
                        }
                        let actual = (0..6)
                            .filter(|cell| {
                                *cell >= left && *cell < right && bits & (1 << cell) != 0
                            })
                            .count() as f64;
                        minimum = minimum.min(actual);
                        maximum = maximum.max(actual);
                    }
                    let result =
                        direct_interval(at(0), at(6), f64::from(mass) / 6.0, at(left), at(right));
                    assert!((result.lower_person_hours - minimum).abs() < 1e-14);
                    assert!((result.upper_person_hours - maximum).abs() < 1e-14);
                }
            }
        }
    }

    #[test]
    fn enlarging_window_cannot_decrease_either_bound() {
        for allocation in [0.0, 0.1, 0.5, 0.9, 1.0] {
            let mut previous = direct_interval(at(0), at(10), allocation, at(0), at(0));
            for end in 1..=12 {
                let current = direct_interval(at(0), at(10), allocation, at(0), at(end));
                assert!(current.lower_person_hours >= previous.lower_person_hours);
                assert!(current.upper_person_hours >= previous.upper_person_hours);
                previous = current;
            }
        }
    }

    #[test]
    fn only_literal_endpoint_allocations_collapse_a_partial_interval() {
        for allocation in [f64::MIN_POSITIVE, 1.0_f64.next_down()] {
            let result = direct_interval(
                at(0),
                at(1),
                allocation,
                at(0),
                at(1) - Duration::minutes(30),
            );
            assert!(result.upper_person_hours > result.lower_person_hours);
        }
        let result = direct_interval(at(0), at(1), 1.0, at(1) - Duration::nanoseconds(1), at(1));
        assert_eq!(result.lower_person_hours, 1.0e-9 / 3_600.0);
        assert_eq!(result.lower_person_hours, result.upper_person_hours);
    }
}
