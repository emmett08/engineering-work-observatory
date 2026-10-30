//! Exact optimisation of finite rational linear programmes.
//!
//! Rows encode `A*x <= b`, with nonnegative variables. A nonnegative right-hand
//! side gets a slack basis variable. A negative right-hand side is reversed and
//! gets a surplus and artificial variable. Phase I maximises minus the sum of
//! artificials. A strictly negative optimum certifies infeasibility. At zero,
//! artificial basics are pivoted out or their redundant rows removed. Phase II
//! uses the requested objective and never permits an artificial to enter.
//!
//! Every tableau has an identity basis and nonnegative basic values. Entering
//! variables use the smallest positive-reduced-cost index; ratio-test ties use
//! the smallest basic-variable index (Bland's rule). Arithmetic and comparisons
//! are exact. Resource limits produce an error, never an approximate bound.
use anyhow::{Result, bail, ensure};
use num_rational::BigRational as Rational;
use num_traits::{One, Signed, Zero};

const MAX_TABLEAU_ENTRIES: usize = 250_000;
const MAX_PIVOTS: usize = 50_000;
const MAX_COEFFICIENT_BITS: u64 = 4_096;

/// Maximise `c*x` subject to `A*x <= b` and `x >= 0`.
///
/// Returns an attaining vector, `None` for an infeasible programme, or an error
/// for malformed, unbounded or resource-limited problems. A returned vector has
/// exactly `c.len()` coordinates; slack and artificial variables are internal.
pub(super) fn maximise(
    a: Vec<Vec<Rational>>,
    b: Vec<Rational>,
    c: Vec<Rational>,
) -> Result<Option<Vec<Rational>>> {
    ensure!(
        a.len() == b.len(),
        "LP row and right-hand-side counts differ"
    );
    ensure!(
        a.iter().all(|row| row.len() == c.len()),
        "LP row width differs from objective dimension"
    );
    for value in a.iter().flatten().chain(&b).chain(&c) {
        check_rational(value)?;
    }
    if c.is_empty() {
        return Ok(b.iter().all(|value| !value.is_negative()).then(Vec::new));
    }
    let original = c.len();
    let ordinary = original
        .checked_add(b.len())
        .ok_or_else(|| anyhow::anyhow!("LP dimension overflow"))?;
    let artificial_count = b.iter().filter(|value| value.is_negative()).count();
    let variables = ordinary
        .checked_add(artificial_count)
        .ok_or_else(|| anyhow::anyhow!("LP dimension overflow"))?;
    let width = variables
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("LP dimension overflow"))?;
    // Include the objective workspace even when there are no constraint rows.
    let entries = b
        .len()
        .checked_add(1)
        .and_then(|rows| rows.checked_mul(width))
        .ok_or_else(|| anyhow::anyhow!("LP tableau size overflow"))?;
    ensure!(
        entries <= MAX_TABLEAU_ENTRIES,
        "LP tableau exceeds {MAX_TABLEAU_ENTRIES} entries"
    );
    let mut rows = Vec::with_capacity(a.len());
    let mut basis = Vec::with_capacity(a.len());
    let mut next_artificial = ordinary;
    for (i, (coefficients, rhs)) in a.into_iter().zip(b).enumerate() {
        let negative = rhs.is_negative();
        let mut row = vec![Rational::zero(); width];
        for (j, value) in coefficients.into_iter().enumerate() {
            row[j] = if negative { -value } else { value };
        }
        row[original + i] = if negative {
            -Rational::one()
        } else {
            Rational::one()
        };
        row[variables] = rhs.abs();
        if negative {
            row[next_artificial] = Rational::one();
            basis.push(next_artificial);
            next_artificial += 1;
        } else {
            basis.push(original + i);
        }
        rows.push(row);
    }
    let mut tableau = Tableau {
        rows,
        basis,
        variables,
        pivots: 0,
    };
    if artificial_count > 0 {
        let mut phase_one = vec![Rational::zero(); variables];
        phase_one[ordinary..].fill(-Rational::one());
        tableau.optimise(&phase_one, variables)?;
        let objective = tableau.objective(&phase_one);
        ensure!(
            !objective.is_positive(),
            "LP phase-I objective violated its nonpositive invariant"
        );
        if objective.is_negative() {
            return Ok(None);
        }
        tableau.remove_artificial_basics(ordinary)?;
    }
    let mut objective = vec![Rational::zero(); variables];
    objective[..original].clone_from_slice(&c);
    tableau.optimise(&objective, ordinary)?;
    let mut solution = vec![Rational::zero(); original];
    for (row, &basic) in tableau.rows.iter().zip(&tableau.basis) {
        if basic < original {
            solution[basic] = row[variables].clone();
        }
    }
    Ok(Some(solution))
}

/// Own the canonical equality rows and their current feasible basis.
struct Tableau {
    rows: Vec<Vec<Rational>>,
    basis: Vec<usize>,
    variables: usize,
    pivots: usize,
}

impl Tableau {
    fn objective(&self, costs: &[Rational]) -> Rational {
        self.rows
            .iter()
            .zip(&self.basis)
            .map(|(row, &basic)| &costs[basic] * &row[self.variables])
            .sum()
    }

    fn optimise(&mut self, costs: &[Rational], active_columns: usize) -> Result<()> {
        loop {
            let mut reduced = costs[..active_columns].to_vec();
            for (row, &basic) in self.rows.iter().zip(&self.basis) {
                if !costs[basic].is_zero() {
                    for (value, coefficient) in reduced.iter_mut().zip(row) {
                        *value -= &costs[basic] * coefficient;
                        check_rational(value)?;
                    }
                }
            }
            let Some(entering) = reduced.iter().position(Signed::is_positive) else {
                return Ok(());
            };
            let mut leaving: Option<(usize, Rational)> = None;
            for (i, row) in self.rows.iter().enumerate() {
                if !row[entering].is_positive() {
                    continue;
                }
                let ratio = &row[self.variables] / &row[entering];
                if leaving.as_ref().is_none_or(|(best, minimum)| {
                    ratio < *minimum || (ratio == *minimum && self.basis[i] < self.basis[*best])
                }) {
                    leaving = Some((i, ratio));
                }
            }
            let Some((leaving, _)) = leaving else {
                bail!("LP objective is unbounded");
            };
            self.pivot(leaving, entering)?;
        }
    }

    fn pivot(&mut self, leaving: usize, entering: usize) -> Result<()> {
        ensure!(self.pivots < MAX_PIVOTS, "LP exceeds {MAX_PIVOTS} pivots");
        self.pivots += 1;
        let divisor = self.rows[leaving][entering].clone();
        ensure!(!divisor.is_zero(), "LP attempted a zero pivot");
        for value in &mut self.rows[leaving] {
            *value /= &divisor;
            check_rational(value)?;
        }
        let pivot_row = self.rows[leaving].clone();
        for (i, row) in self.rows.iter_mut().enumerate() {
            if i == leaving {
                continue;
            }
            let multiplier = row[entering].clone();
            if multiplier.is_zero() {
                continue;
            }
            for (value, coefficient) in row.iter_mut().zip(&pivot_row) {
                *value -= &multiplier * coefficient;
                check_rational(value)?;
            }
            ensure!(
                !row[self.variables].is_negative(),
                "LP pivot violated primal feasibility"
            );
        }
        self.basis[leaving] = entering;
        Ok(())
    }

    fn remove_artificial_basics(&mut self, ordinary: usize) -> Result<()> {
        let mut i = 0;
        while i < self.rows.len() {
            if self.basis[i] < ordinary {
                i += 1;
                continue;
            }
            ensure!(
                self.rows[i][self.variables].is_zero(),
                "LP feasible phase-I basis contains a positive artificial"
            );
            if let Some(entering) = self.rows[i][..ordinary]
                .iter()
                .position(|value| !value.is_zero())
            {
                // The leaving basic has value zero. Either sign of coefficient
                // preserves all basic values, and existing basic columns are zero.
                self.pivot(i, entering)?;
                i += 1;
            } else {
                // With artificials fixed to zero this row is identically 0 = 0.
                self.rows.remove(i);
                self.basis.remove(i);
            }
        }
        Ok(())
    }
}

fn check_rational(value: &Rational) -> Result<()> {
    ensure!(
        value.numer().bits() <= MAX_COEFFICIENT_BITS
            && value.denom().bits() <= MAX_COEFFICIENT_BITS,
        "LP coefficient exceeds {MAX_COEFFICIENT_BITS} bits"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(value: i64) -> Rational {
        Rational::from_integer(value.into())
    }

    fn solve(a: &[&[i64]], b: &[i64], c: &[i64]) -> Result<Option<Vec<Rational>>> {
        maximise(
            a.iter()
                .map(|row| row.iter().map(|&x| q(x)).collect())
                .collect(),
            b.iter().map(|&x| q(x)).collect(),
            c.iter().map(|&x| q(x)).collect(),
        )
    }

    #[test]
    fn fractional_optimum_and_negative_objective() {
        let a: &[&[i64]] = &[&[2, 1], &[1, 2]];
        let x = solve(a, &[4, 4], &[1, 1]).unwrap().unwrap();
        assert_eq!(x, vec![q(4) / q(3), q(4) / q(3)]);
        assert_eq!(
            solve(a, &[4, 4], &[-1, -1]).unwrap(),
            Some(vec![q(0), q(0)])
        );
    }

    #[test]
    fn phase_one_handles_lower_bounds_and_inconsistency() {
        assert_eq!(
            solve(&[&[-1], &[1]], &[-2, 3], &[1]).unwrap(),
            Some(vec![q(3)])
        );
        assert_eq!(
            solve(&[&[-1], &[1]], &[-2, 3], &[-1]).unwrap(),
            Some(vec![q(2)])
        );
        assert_eq!(solve(&[&[-1], &[1]], &[-4, 3], &[1]).unwrap(), None);
        assert_eq!(solve(&[&[0]], &[-1], &[0]).unwrap(), None);
    }

    #[test]
    fn duplicate_equalities_and_degenerate_bases() {
        let x = solve(
            &[&[1, 1], &[-1, -1], &[-2, -2], &[1, 0], &[0, 1]],
            &[1, -1, -2, 1, 1],
            &[1, 0],
        )
        .unwrap()
        .unwrap();
        assert_eq!(x, vec![q(1), q(0)]);
    }

    #[test]
    fn bland_rule_resolves_classical_cycling_example() {
        // Beale's degenerate example cycles under largest-reduced-cost pivoting.
        let a = vec![
            vec![q(1) / q(2), -q(11) / q(2), -q(5) / q(2), q(9)],
            vec![q(1) / q(2), -q(3) / q(2), -q(1) / q(2), q(1)],
            vec![q(1), q(0), q(0), q(0)],
        ];
        let c = vec![q(10), q(-57), q(-9), q(-24)];
        let x = maximise(a, vec![q(0), q(0), q(1)], c.clone())
            .unwrap()
            .unwrap();
        assert_eq!(x.iter().zip(&c).map(|(x, c)| x * c).sum::<Rational>(), q(1));
    }

    #[test]
    fn empty_dimensions_and_unbounded_objectives() {
        assert_eq!(solve(&[], &[], &[]).unwrap(), Some(vec![]));
        assert_eq!(solve(&[&[]], &[0], &[]).unwrap(), Some(vec![]));
        assert_eq!(solve(&[&[]], &[-1], &[]).unwrap(), None);
        assert_eq!(solve(&[], &[], &[-1]).unwrap(), Some(vec![q(0)]));
        assert!(
            solve(&[], &[], &[1])
                .unwrap_err()
                .to_string()
                .contains("unbounded")
        );
        assert!(
            solve(&[&[0]], &[1], &[1])
                .unwrap_err()
                .to_string()
                .contains("unbounded")
        );
    }

    /// Independent two-dimensional oracle: intersect every pair of boundary
    /// lines, retain feasible vertices, and compare exact objective values.
    fn vertex_maximum(a: &[Vec<Rational>], b: &[Rational], c: &[Rational]) -> Option<Rational> {
        let mut rows = a.to_vec();
        let mut rhs = b.to_vec();
        rows.extend([vec![q(-1), q(0)], vec![q(0), q(-1)]]);
        rhs.extend([q(0), q(0)]);
        let mut maximum: Option<Rational> = None;
        for i in 0..rows.len() {
            for j in (i + 1)..rows.len() {
                let determinant = &rows[i][0] * &rows[j][1] - &rows[i][1] * &rows[j][0];
                if determinant.is_zero() {
                    continue;
                }
                let x = (&rhs[i] * &rows[j][1] - &rows[i][1] * &rhs[j]) / &determinant;
                let y = (&rows[i][0] * &rhs[j] - &rhs[i] * &rows[j][0]) / &determinant;
                if rows
                    .iter()
                    .zip(&rhs)
                    .all(|(row, bound)| &row[0] * &x + &row[1] * &y <= *bound)
                {
                    let value = &c[0] * x + &c[1] * y;
                    if maximum.as_ref().is_none_or(|old| value > *old) {
                        maximum = Some(value);
                    }
                }
            }
        }
        maximum
    }

    #[test]
    fn bounded_programmes_match_independent_vertex_enumeration() {
        let mut state = 0x5eed_u64;
        let mut draw = || {
            // Deliberately wrapping recurrence defines deterministic test inputs.
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            ((state >> 32) % 7) as i64 - 3
        };
        for _ in 0..256 {
            let mut a = vec![vec![q(1), q(0)], vec![q(0), q(1)]];
            let mut b = vec![q(3), q(3)];
            for _ in 0..4 {
                a.push(vec![q(draw()), q(draw())]);
                b.push(q(draw()));
            }
            for c in [vec![q(1), q(1)], vec![q(-1), q(2)], vec![q(0), q(0)]] {
                let expected = vertex_maximum(&a, &b, &c);
                let solution = maximise(a.clone(), b.clone(), c.clone()).unwrap();
                if let Some(x) = &solution {
                    assert!(x.iter().all(|value| !value.is_negative()));
                    assert!(a.iter().zip(&b).all(|(row, bound)| {
                        row.iter().zip(x).map(|(v, x)| v * x).sum::<Rational>() <= *bound
                    }));
                }
                let actual = solution.map(|x| x.iter().zip(&c).map(|(x, cost)| x * cost).sum());
                assert_eq!(actual, expected, "A={a:?}, b={b:?}, c={c:?}");
            }
        }
    }

    #[test]
    fn malformed_and_resource_limited_inputs_are_errors() {
        assert!(solve(&[&[1]], &[], &[1]).is_err());
        assert!(solve(&[&[1, 2]], &[1], &[1]).is_err());
        let huge = Rational::from_integer(num_bigint::BigInt::one() << 4096_u32);
        assert!(maximise(vec![vec![huge]], vec![q(1)], vec![q(1)]).is_err());
        assert!(maximise(vec![], vec![], vec![q(0); MAX_TABLEAU_ENTRIES]).is_err());
    }
}
