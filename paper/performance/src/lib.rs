//! Exact discrete cohort accounting for repeated acceptance-horizon queries.
//!
//! This is a computational kernel, not an estimator of human effort. Input weights
//! are nonnegative fixed-point micro-units. Labour increments must already be
//! attributed without double counting; retention marks use one fixed follow-up.

/// Age since the relevant task's enrolment, and its nonnegative fixed-point weight.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeightedEvent {
    pub age: u64,
    pub weight: u64,
}

/// An immutable snapshot during a query batch. An unaccepted task contributes only
/// labour events. Every accepted requirement appears at most once in acceptances.
#[derive(Clone, Debug, Default)]
pub struct Cohort {
    pub acceptances: Vec<WeightedEvent>,
    pub labour: Vec<WeightedEvent>,
}

/// The exact pair is canonical; the optional floating ratio is only a display aid.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Totals {
    pub retained_output: u128,
    pub labour: u128,
}

impl Totals {
    /// Reference units per person-hour when both streams use 10^-6 fixed-point units.
    pub fn ratio(self) -> Option<f64> {
        (self.labour != 0).then(|| self.retained_output as f64 / self.labour as f64)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingError {
    HorizonOverflow,
    TotalOverflow,
}

fn cutoffs(horizons: &[u64], delta: u64) -> Result<Vec<u64>, AccountingError> {
    horizons
        .iter()
        .map(|h| h.checked_add(delta).ok_or(AccountingError::HorizonOverflow))
        .collect()
}

fn scan_total(events: &[WeightedEvent], cutoff: u64) -> Result<u128, AccountingError> {
    events
        .iter()
        .filter(|event| event.age <= cutoff)
        .try_fold(0_u128, |sum, event| {
            sum.checked_add(u128::from(event.weight))
                .ok_or(AccountingError::TotalOverflow)
        })
}

/// Direct oracle: rescan both event streams for every query, with inclusive cutoffs.
/// The result preserves query order, including duplicate or unsorted horizons.
pub fn scan_batch(
    cohort: &Cohort,
    horizons: &[u64],
    delta: u64,
) -> Result<Vec<Totals>, AccountingError> {
    let labour_cutoffs = cutoffs(horizons, delta)?;
    horizons
        .iter()
        .zip(labour_cutoffs)
        .map(|(&tau, labour_cutoff)| {
            Ok(Totals {
                retained_output: scan_total(&cohort.acceptances, tau)?,
                labour: scan_total(&cohort.labour, labour_cutoff)?,
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct PrefixIndex {
    times: Vec<u64>,
    // prefix[0] = 0; prefix[k] sums the first k sorted events, including zeros.
    prefix: Vec<u128>,
}

impl PrefixIndex {
    fn new(events: &[WeightedEvent]) -> Result<Self, AccountingError> {
        let mut sorted = events.to_vec();
        sorted.sort_unstable_by_key(|event| event.age);
        let mut times = Vec::with_capacity(sorted.len());
        let mut prefix = Vec::with_capacity(sorted.len() + 1);
        prefix.push(0_u128);
        for event in sorted {
            let next = prefix
                .last()
                .unwrap()
                .checked_add(u128::from(event.weight))
                .ok_or(AccountingError::TotalOverflow)?;
            times.push(event.age);
            prefix.push(next);
        }
        Ok(Self { times, prefix })
    }

    fn at(&self, cutoff: u64) -> u128 {
        // partition_point is the upper-bound search: every duplicate at cutoff counts.
        self.prefix[self.times.partition_point(|age| *age <= cutoff)]
    }
}

/// A reusable index owning a snapshot of both input streams. Rebuild if any input
/// changes, including retention assessments. It does not consult the source again.
#[derive(Clone, Debug)]
pub struct CohortIndex {
    acceptances: PrefixIndex,
    labour: PrefixIndex,
}

impl CohortIndex {
    pub fn new(cohort: &Cohort) -> Result<Self, AccountingError> {
        Ok(Self {
            acceptances: PrefixIndex::new(&cohort.acceptances)?,
            labour: PrefixIndex::new(&cohort.labour)?,
        })
    }

    /// Delta must be the follow-up used when deriving the retained-output weights.
    /// This kernel cannot check that semantic condition from the event arrays.
    pub fn query_batch(
        &self,
        horizons: &[u64],
        delta: u64,
    ) -> Result<Vec<Totals>, AccountingError> {
        let labour_cutoffs = cutoffs(horizons, delta)?;
        Ok(horizons
            .iter()
            .zip(labour_cutoffs)
            .map(|(&tau, labour_cutoff)| Totals {
                retained_output: self.acceptances.at(tau),
                labour: self.labour.at(labour_cutoff),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(age: u64, weight: u64) -> WeightedEvent {
        WeightedEvent { age, weight }
    }

    fn both(cohort: &Cohort, horizons: &[u64], delta: u64) -> Vec<Totals> {
        let expected = scan_batch(cohort, horizons, delta).unwrap();
        assert_eq!(
            CohortIndex::new(cohort)
                .unwrap()
                .query_batch(horizons, delta)
                .unwrap(),
            expected
        );
        expected
    }

    #[test]
    fn empty_inputs_and_empty_queries() {
        assert!(both(&Cohort::default(), &[], u64::MAX).is_empty());
        assert_eq!(
            both(&Cohort::default(), &[0, 9], 0),
            vec![Totals::default(); 2]
        );
        assert_eq!(Totals::default().ratio(), None);
    }

    #[test]
    fn duplicate_inclusive_boundaries_unsorted_queries_and_zero_weights() {
        let cohort = Cohort {
            acceptances: vec![event(5, 2), event(3, 7), event(5, 11), event(0, 0)],
            labour: vec![event(7, 13), event(2, 0), event(8, 17), event(7, 19)],
        };
        assert_eq!(
            both(&cohort, &[5, 0, 4, 5, 6], 2),
            vec![
                Totals {
                    retained_output: 20,
                    labour: 32
                },
                Totals {
                    retained_output: 0,
                    labour: 0
                },
                Totals {
                    retained_output: 7,
                    labour: 0
                },
                Totals {
                    retained_output: 20,
                    labour: 32
                },
                Totals {
                    retained_output: 20,
                    labour: 49
                },
            ]
        );
    }

    #[test]
    fn unaccepted_tasks_keep_their_labour_and_zero_labour_keeps_output() {
        let labour_only = Cohort {
            acceptances: vec![],
            labour: vec![event(0, 123)],
        };
        assert_eq!(both(&labour_only, &[0], 0)[0].ratio(), Some(0.0));
        let output_only = Cohort {
            acceptances: vec![event(0, 99)],
            labour: vec![],
        };
        let totals = both(&output_only, &[0], 0)[0];
        assert_eq!(totals.retained_output, 99);
        assert_eq!(totals.ratio(), None);
    }

    #[test]
    fn horizon_overflow_is_an_explicit_error() {
        let cohort = Cohort::default();
        assert_eq!(
            scan_batch(&cohort, &[0, u64::MAX], 1),
            Err(AccountingError::HorizonOverflow)
        );
        assert_eq!(
            CohortIndex::new(&cohort)
                .unwrap()
                .query_batch(&[0, u64::MAX], 1),
            Err(AccountingError::HorizonOverflow)
        );
        assert_eq!(both(&cohort, &[u64::MAX], 0), vec![Totals::default()]);
    }

    #[test]
    fn accumulation_exceeds_u64_without_loss() {
        let cohort = Cohort {
            acceptances: vec![event(0, u64::MAX); 3],
            labour: vec![event(0, u64::MAX); 2],
        };
        let totals = both(&cohort, &[0], 0)[0];
        assert_eq!(totals.retained_output, 3 * u128::from(u64::MAX));
        assert_eq!(totals.labour, 2 * u128::from(u64::MAX));
        assert_eq!(totals.ratio(), Some(1.5));
    }

    #[test]
    fn index_owns_a_snapshot() {
        let mut cohort = Cohort {
            acceptances: vec![event(0, 7)],
            labour: vec![event(0, 3)],
        };
        let index = CohortIndex::new(&cohort).unwrap();
        cohort.acceptances[0].weight = 11;
        assert_eq!(index.query_batch(&[0], 0).unwrap()[0].retained_output, 7);
        assert_eq!(both(&cohort, &[0], 0)[0].retained_output, 11);
    }

    #[test]
    fn generated_cases_match_direct_oracle() {
        // A separate small integer recurrence supplies unsorted, tied and empty cases.
        let mut state = 991_u64;
        for n in 0..80 {
            let mut next = || {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                state
            };
            let mut cohort = Cohort::default();
            for _ in 0..n {
                cohort.acceptances.push(event(next() % 31, next() % 23));
                cohort.labour.push(event(next() % 47, next() % 41));
            }
            let horizons: Vec<_> = (0..50).map(|_| next() % 40).collect();
            both(&cohort, &horizons, n % 8);
        }
    }
}
