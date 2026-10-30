//! Independent checks of the published interval model.
//!
//! The oracle enumerates vertices and solves their active equalities by Gaussian
//! elimination. It does not use the production simplex or its constraint builder.
use observatory::identification::{LabourBoundInput, bound_labour};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Q {
    n: i128,
    d: i128,
}
impl Q {
    fn new(n: i128, d: i128) -> Self {
        assert_ne!(d, 0);
        let (mut a, mut b) = (n.abs(), d.abs());
        while b != 0 {
            (a, b) = (b, a % b);
        }
        let g = a.max(1);
        Self {
            n: n / g * d.signum(),
            d: d.abs() / g,
        }
    }
    fn int(n: i128) -> Self {
        Self::new(n, 1)
    }
    fn parse(v: &Value) -> Self {
        let s = v.as_str().expect("exact values are rational strings");
        let mut p = s.split('/');
        let n = p.next().unwrap().parse().unwrap();
        let d = p.next().map_or(1, |s| s.parse().unwrap());
        assert!(p.next().is_none());
        Self::new(n, d)
    }
    fn text(self) -> String {
        if self.d == 1 {
            self.n.to_string()
        } else {
            format!("{}/{}", self.n, self.d)
        }
    }
}
impl PartialOrd for Q {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        Some((self.n * rhs.d).cmp(&(rhs.n * self.d)))
    }
}
impl Add for Q {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.n * b.d + b.n * self.d, self.d * b.d)
    }
}
impl Sub for Q {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        self + -b
    }
}
impl Mul for Q {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        fn gcd(mut a: i128, mut b: i128) -> i128 {
            while b != 0 {
                (a, b) = (b, a % b);
            }
            a.max(1)
        }
        let g = gcd(self.n.abs(), b.d);
        let h = gcd(b.n.abs(), self.d);
        Self::new((self.n / g) * (b.n / h), (self.d / h) * (b.d / g))
    }
}
impl Div for Q {
    type Output = Self;
    fn div(self, b: Self) -> Self {
        Mul::mul(self, Self::new(b.d, b.n))
    }
}
impl Neg for Q {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.n, self.d)
    }
}

fn stream(id: &str, person: &str) -> Value {
    json!({"id":id,"person_id":person,"task_id":format!("task-{id}"),"role":"implementation"})
}
fn observation(id: &str, start: &str, end: &str, streams: &[&str], lo: &str, hi: &str) -> Value {
    json!({"id":id,"start":start,"end":end,"stream_ids":streams,"lower_person_hours":lo,"upper_person_hours":hi})
}
fn target(id: &str, start: &str, end: &str, weight: &str) -> Value {
    json!({"stream_id":id,"start":start,"end":end,"weight":weight})
}
fn problem(streams: Vec<Value>, observations: Vec<Value>, targets: Vec<Value>) -> Value {
    json!({"schema":"observatory.labour-bound-input/1","problem_id":"independent-test",
        "domain":{"start":"0","end":"2"},"streams":streams,"observations":observations,"target":targets})
}
fn solve(input: &Value) -> Value {
    let typed: LabourBoundInput = serde_json::from_value(input.clone()).unwrap();
    serde_json::to_value(bound_labour(&typed).unwrap()).unwrap()
}
fn dot(a: &[Q], b: &[Q]) -> Q {
    a.iter().zip(b).fold(Q::int(0), |v, (x, y)| v + *x * *y)
}

/// Inspect the returned allocation as an independently integrated history.
fn check_witnesses(input: &Value, report: &Value) {
    assert_eq!(report["numerical_semantics"], "exact_rational");
    if report["status"] == "inconsistent" {
        assert!(report["minimum"].is_null());
        assert!(report["maximum"].is_null());
        return;
    }
    assert_eq!(report["status"], "feasible");
    let cells: Vec<(Q, Q)> = report["cells"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (Q::parse(&c["start"]), Q::parse(&c["end"])))
        .collect();
    assert_eq!(
        cells.first().unwrap().0,
        Q::parse(&input["domain"]["start"])
    );
    assert_eq!(cells.last().unwrap().1, Q::parse(&input["domain"]["end"]));
    for (k, (a, b)) in cells.iter().enumerate() {
        assert!(a < b);
        if k > 0 {
            assert_eq!(cells[k - 1].1, *a);
        }
    }
    let streams: BTreeMap<&str, &str> = input["streams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (s["id"].as_str().unwrap(), s["person_id"].as_str().unwrap()))
        .collect();
    for which in ["minimum", "maximum"] {
        let mut masses = BTreeMap::new();
        for row in report[which]["allocations"].as_array().unwrap() {
            let id = row["stream_id"].as_str().unwrap();
            assert!(streams.contains_key(id));
            let m: Vec<Q> = row["person_hours"]
                .as_array()
                .unwrap()
                .iter()
                .map(Q::parse)
                .collect();
            assert_eq!(m.len(), cells.len());
            assert!(m.iter().all(|x| *x >= Q::int(0)));
            assert!(masses.insert(id, m).is_none());
        }
        assert_eq!(masses.len(), streams.len());
        for person in streams.values().copied().collect::<BTreeSet<_>>() {
            for (k, (a, b)) in cells.iter().enumerate() {
                let used = streams
                    .iter()
                    .filter(|(_, p)| **p == person)
                    .fold(Q::int(0), |sum, (id, _)| sum + masses[id][k]);
                assert!(
                    used <= *b - *a,
                    "{which}: capacity for {person} at cell {k}"
                );
            }
        }
        // The integration deliberately permits boundaries inside a cell, so it
        // would detect a missing endpoint in the production partition as well.
        let integrate = |id: &str, start: Q, end: Q| -> Q {
            cells
                .iter()
                .enumerate()
                .fold(Q::int(0), |sum, (k, (a, b))| {
                    let left = if *a > start { *a } else { start };
                    let right = if *b < end { *b } else { end };
                    if right > left {
                        sum + masses[id][k] * (right - left) / (*b - *a)
                    } else {
                        sum
                    }
                })
        };
        for obs in input["observations"].as_array().unwrap() {
            let value = obs["stream_ids"]
                .as_array()
                .unwrap()
                .iter()
                .fold(Q::int(0), |sum, id| {
                    sum + integrate(
                        id.as_str().unwrap(),
                        Q::parse(&obs["start"]),
                        Q::parse(&obs["end"]),
                    )
                });
            assert!(value >= Q::parse(&obs["lower_person_hours"]));
            assert!(value <= Q::parse(&obs["upper_person_hours"]));
        }
        let value = input["target"]
            .as_array()
            .unwrap()
            .iter()
            .fold(Q::int(0), |sum, t| {
                sum + Q::parse(&t["weight"])
                    * integrate(
                        t["stream_id"].as_str().unwrap(),
                        Q::parse(&t["start"]),
                        Q::parse(&t["end"]),
                    )
            });
        assert_eq!(value, Q::parse(&report[which]["value"]));
    }
    assert!(Q::parse(&report["minimum"]["value"]) <= Q::parse(&report["maximum"]["value"]));
}

#[test]
fn shared_capacity_collapses_the_sum_of_marginal_ranges() {
    let mut input = problem(
        vec![stream("a", "p"), stream("b", "p")],
        vec![
            observation("a-total", "0", "2", &["a"], "1", "1"),
            observation("b-total", "0", "2", &["b"], "1", "1"),
        ],
        vec![target("a", "0", "1", "1"), target("b", "0", "1", "1")],
    );
    input["known_output"] = json!("10");
    let result = solve(&input);
    check_witnesses(&input, &result);
    assert_eq!(Q::parse(&result["minimum"]["value"]), Q::int(1));
    assert_eq!(Q::parse(&result["maximum"]["value"]), Q::int(1));
    assert_eq!(
        Q::parse(&result["productivity"]["lower_reference_units_per_person_hour"]),
        Q::int(10)
    );
    assert_eq!(
        Q::parse(&result["productivity"]["upper_reference_units_per_person_hour"]),
        Q::int(10)
    );
}

#[test]
fn overlapping_scalar_observations_and_unequal_interval_averages_are_feasible() {
    let mut scalar = problem(
        vec![stream("a", "p")],
        vec![
            observation("left", "0", "2", &["a"], "1", "1"),
            observation("right", "1", "3", &["a"], "2", "2"),
        ],
        vec![target("a", "0", "1", "1")],
    );
    scalar["domain"]["end"] = json!("3");
    let result = solve(&scalar);
    check_witnesses(&scalar, &result);
    assert_eq!(Q::parse(&result["minimum"]["value"]), Q::int(0));
    assert_eq!(Q::parse(&result["maximum"]["value"]), Q::int(0));
    let mut joint = problem(
        vec![stream("a", "p"), stream("b", "p")],
        vec![
            observation("left", "0", "2", &["a"], "3/2", "3/2"),
            observation("right", "1", "3", &["b"], "3/2", "3/2"),
        ],
        vec![target("a", "0", "3", "1"), target("b", "0", "3", "1")],
    );
    joint["domain"]["end"] = json!("3");
    let result = solve(&joint);
    check_witnesses(&joint, &result);
    assert_eq!(Q::parse(&result["minimum"]["value"]), Q::int(3));
    assert_eq!(Q::parse(&result["maximum"]["value"]), Q::int(3));
}

#[test]
fn grouped_observations_require_fractional_joint_vertices() {
    let input = problem(
        vec![stream("a", "p"), stream("b", "p"), stream("c", "p")],
        vec![
            observation("ab", "0", "2", &["a", "b"], "1", "1"),
            observation("bc", "0", "2", &["b", "c"], "1", "1"),
            observation("ac", "0", "2", &["a", "c"], "1", "1"),
        ],
        vec![
            target("a", "0", "2", "1"),
            target("b", "0", "2", "1"),
            target("c", "0", "2", "1"),
        ],
    );
    let result = solve(&input);
    check_witnesses(&input, &result);
    assert_eq!(Q::parse(&result["minimum"]["value"]), Q::new(3, 2));
    assert_eq!(Q::parse(&result["maximum"]["value"]), Q::new(3, 2));
}

/// Solve a square active set independently, rejecting singular intersections.
fn intersection(rows: &[(Vec<Q>, Q)], selected: &[usize]) -> Option<Vec<Q>> {
    let n = selected.len();
    let mut matrix: Vec<Vec<Q>> = selected
        .iter()
        .map(|i| {
            let mut r = rows[*i].0.clone();
            r.push(rows[*i].1);
            r
        })
        .collect();
    for k in 0..n {
        let pivot = (k..n).find(|i| matrix[*i][k] != Q::int(0))?;
        matrix.swap(k, pivot);
        let scale = matrix[k][k];
        for value in &mut matrix[k][k..] {
            *value = *value / scale;
        }
        let pivot_row = matrix[k].clone();
        for (i, row) in matrix.iter_mut().enumerate() {
            if i == k {
                continue;
            }
            let scale = row[k];
            for (value, pivot) in row[k..].iter_mut().zip(&pivot_row[k..]) {
                *value = *value - scale * *pivot;
            }
        }
    }
    Some(matrix.iter().map(|r| r[n]).collect())
}
fn vertices(rows: &[(Vec<Q>, Q)], objective: &[Q]) -> Option<(Q, Q)> {
    let n = objective.len();
    let mut choice: Vec<usize> = (0..n).collect();
    let mut result: Option<(Q, Q)> = None;
    loop {
        if let Some(x) = intersection(rows, &choice)
            && rows.iter().all(|(a, b)| dot(a, &x) <= *b)
        {
            let v = dot(objective, &x);
            result = Some(match result {
                None => (v, v),
                Some((lo, hi)) => (if v < lo { v } else { lo }, if v > hi { v } else { hi }),
            });
        }
        let Some(k) = (0..n).rev().find(|k| choice[*k] < rows.len() - n + *k) else {
            break;
        };
        choice[k] += 1;
        for j in k + 1..n {
            choice[j] = choice[j - 1] + 1;
        }
    }
    result
}

/// Build the small oracle directly from two one-hour cells and two streams.
fn small_oracle(input: &Value) -> Option<(Q, Q)> {
    let zero = Q::int(0);
    let one = Q::int(1);
    let mut rows = Vec::new();
    for v in 0..4 {
        let mut a = vec![zero; 4];
        a[v] = -one;
        rows.push((a, zero));
    }
    let same_person = input["streams"][0]["person_id"] == input["streams"][1]["person_id"];
    if same_person {
        rows.push((vec![one, zero, one, zero], one));
        rows.push((vec![zero, one, zero, one], one));
    } else {
        for v in 0..4 {
            let mut a = vec![zero; 4];
            a[v] = one;
            rows.push((a, one));
        }
    }
    let coefficients = |ids: Vec<&str>, start: Q, end: Q| -> Vec<Q> {
        let mut a = vec![zero; 4];
        for (s, id) in ["a", "b"].iter().enumerate() {
            for k in 0..2 {
                if ids.contains(id) && Q::int(k as i128) >= start && Q::int(k as i128 + 1) <= end {
                    a[s * 2 + k] = one;
                }
            }
        }
        a
    };
    for obs in input["observations"].as_array().unwrap() {
        let a = coefficients(
            obs["stream_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect(),
            Q::parse(&obs["start"]),
            Q::parse(&obs["end"]),
        );
        rows.push((a.clone(), Q::parse(&obs["upper_person_hours"])));
        rows.push((
            a.iter().map(|x| -*x).collect(),
            -Q::parse(&obs["lower_person_hours"]),
        ));
    }
    let mut objective = vec![zero; 4];
    for t in input["target"].as_array().unwrap() {
        let a = coefficients(
            vec![t["stream_id"].as_str().unwrap()],
            Q::parse(&t["start"]),
            Q::parse(&t["end"]),
        );
        for v in 0..4 {
            objective[v] = objective[v] + a[v] * Q::parse(&t["weight"]);
        }
    }
    vertices(&rows, &objective)
}

#[test]
fn exact_joint_results_match_independent_vertex_enumeration() {
    // Deterministic varied problems include infeasible intervals, nonzero-width
    // bounds, grouped observations, shared/different people and signed targets.
    let mut seed = 0x12ab_34cd_u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed >> 32
    };
    let mut feasible = 0;
    let mut inconsistent = 0;
    for case in 0..48 {
        let mut observations = Vec::new();
        for i in 0..3 {
            let (start, end) = match next() % 3 {
                0 => ("0", "1"),
                1 => ("1", "2"),
                _ => ("0", "2"),
            };
            let ids = match next() % 3 {
                0 => vec!["a"],
                1 => vec!["b"],
                _ => vec!["a", "b"],
            };
            let lo = Q::new((next() % 7) as i128, 4);
            let hi = lo + Q::new((next() % 5) as i128, 4);
            observations.push(observation(
                &format!("o{i}"),
                start,
                end,
                &ids,
                &lo.text(),
                &hi.text(),
            ));
        }
        let targets = vec![
            target("a", "0", "1", &((next() % 5) as i128 - 2).to_string()),
            target("b", "1", "2", &((next() % 5) as i128 - 2).to_string()),
            target("a", "1", "2", "1"),
        ];
        let input = problem(
            vec![
                stream("a", "p"),
                stream("b", if case % 2 == 0 { "p" } else { "q" }),
            ],
            observations,
            targets,
        );
        let expected = small_oracle(&input);
        let result = solve(&input);
        check_witnesses(&input, &result);
        match expected {
            Some((lo, hi)) => {
                feasible += 1;
                assert_eq!(result["status"], "feasible", "case {case}: {input}");
                assert_eq!(
                    Q::parse(&result["minimum"]["value"]),
                    lo,
                    "minimum case {case}"
                );
                assert_eq!(
                    Q::parse(&result["maximum"]["value"]),
                    hi,
                    "maximum case {case}"
                );
            }
            None => {
                inconsistent += 1;
                assert_eq!(result["status"], "inconsistent", "case {case}: {input}");
            }
        }
    }
    assert!(
        feasible > 5 && inconsistent > 5,
        "both feasibility branches need coverage: {feasible}/{inconsistent}"
    );
}

#[test]
fn subsecond_rational_boundaries_and_unobserved_cells_preserve_mass() {
    let mut input = problem(
        vec![stream("a", "p")],
        vec![observation("total", "0", "2", &["a"], "3/2", "3/2")],
        vec![target("a", "0", "1/3600000000000", "1")],
    );
    let result = solve(&input);
    check_witnesses(&input, &result);
    assert_eq!(Q::parse(&result["minimum"]["value"]), Q::int(0));
    assert_eq!(
        Q::parse(&result["maximum"]["value"]),
        Q::new(1, 3600000000000)
    );
    input["observations"][0]["lower_person_hours"] = json!("2");
    input["observations"][0]["upper_person_hours"] = json!("2");
    let result = solve(&input);
    check_witnesses(&input, &result);
    assert_eq!(
        Q::parse(&result["minimum"]["value"]),
        Q::new(1, 3600000000000)
    );
    assert_eq!(
        Q::parse(&result["maximum"]["value"]),
        Q::new(1, 3600000000000)
    );
}

#[test]
fn results_are_invariant_to_record_order_and_covary_with_time_scale() {
    let input = problem(
        vec![stream("a", "p"), stream("b", "p")],
        vec![
            observation("a-total", "0", "2", &["a"], "3/4", "5/4"),
            observation("b-total", "0", "2", &["b"], "1/2", "1"),
            observation("first-cell", "0", "1", &["a", "b"], "1/4", "3/4"),
        ],
        vec![target("a", "0", "1", "2"), target("b", "1", "2", "-1")],
    );
    let result = solve(&input);
    check_witnesses(&input, &result);
    let mut reordered = input.clone();
    for field in ["streams", "observations", "target"] {
        reordered[field].as_array_mut().unwrap().reverse();
    }
    let permuted = solve(&reordered);
    check_witnesses(&reordered, &permuted);
    for which in ["minimum", "maximum"] {
        assert_eq!(result[which]["value"], permuted[which]["value"]);
    }
    // t -> 2*t-5 preserves rates, doubles all labour masses, and moves the
    // whole problem onto a different calendar origin, including negative times.
    let mut scaled = input.clone();
    for endpoint in ["start", "end"] {
        scaled["domain"][endpoint] =
            json!((Q::parse(&input["domain"][endpoint]) * Q::int(2) - Q::int(5)).text());
    }
    for field in ["observations", "target"] {
        for row in scaled[field].as_array_mut().unwrap() {
            for endpoint in ["start", "end"] {
                row[endpoint] = json!((Q::parse(&row[endpoint]) * Q::int(2) - Q::int(5)).text());
            }
            if field == "observations" {
                for bound in ["lower_person_hours", "upper_person_hours"] {
                    row[bound] = json!((Q::parse(&row[bound]) * Q::int(2)).text());
                }
            }
        }
    }
    let rescaled = solve(&scaled);
    check_witnesses(&scaled, &rescaled);
    for which in ["minimum", "maximum"] {
        assert_eq!(
            Q::parse(&rescaled[which]["value"]),
            Q::parse(&result[which]["value"]) * Q::int(2)
        );
    }
}

#[test]
fn zero_lower_labour_withholds_productivity_and_malformed_constraints_are_errors() {
    let mut input = problem(
        vec![stream("a", "p")],
        vec![],
        vec![target("a", "0", "2", "1")],
    );
    input["known_output"] = json!("1");
    let result = solve(&input);
    check_witnesses(&input, &result);
    assert!(result["productivity"].is_null());
    assert_eq!(Q::parse(&result["minimum"]["value"]), Q::int(0));
    assert_eq!(Q::parse(&result["maximum"]["value"]), Q::int(2));
    for defect in [
        "unknown_stream",
        "duplicate_identity",
        "reversed_bounds",
        "signed_productivity",
        "decimal_mass",
    ] {
        let mut bad = input.clone();
        bad["observations"] = json!([observation("o", "0", "2", &["a"], "0", "1")]);
        match defect {
            "unknown_stream" => bad["observations"][0]["stream_ids"] = json!(["absent"]),
            "duplicate_identity" => bad["streams"]
                .as_array_mut()
                .unwrap()
                .push(stream("a", "q")),
            "reversed_bounds" => bad["observations"][0]["lower_person_hours"] = json!("2"),
            "signed_productivity" => bad["target"][0]["weight"] = json!("-1"),
            "decimal_mass" => bad["observations"][0]["upper_person_hours"] = json!(0.5),
            _ => unreachable!(),
        }
        let rejected = match serde_json::from_value::<LabourBoundInput>(bad) {
            Err(_) => true,
            Ok(typed) => bound_labour(&typed).is_err(),
        };
        assert!(rejected, "accepted {defect}");
    }
}
