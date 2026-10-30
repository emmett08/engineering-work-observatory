//! Validate interval records and construct the exact finite-cell linear programmes.
use super::{Exact, LabourBoundInput, TimeInterval};
use anyhow::{Result, ensure};
use num_rational::BigRational;
use num_traits::{One, Zero};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Rational = BigRational;
pub(super) struct CellProblem {
    pub cells: Vec<TimeInterval>,
    pub a: Vec<Vec<Rational>>,
    pub b: Vec<Rational>,
    pub c: Vec<Rational>,
}

const MAX_STREAMS: usize = 64;
const MAX_OBSERVATIONS: usize = 128;
const MAX_TARGET_WINDOWS: usize = 128;
const MAX_CELLS: usize = 128;
const MAX_VARIABLES: usize = 512;
const MAX_TABLEAU_ENTRIES: usize = 250_000;

fn nonempty(value: &str, name: &str) -> Result<()> {
    ensure!(!value.trim().is_empty(), "{name} must be nonempty");
    ensure!(value.len() <= 1024, "{name} exceeds 1024 characters");
    Ok(())
}

fn validate_interval(input: &LabourBoundInput, start: &Exact, end: &Exact) -> Result<()> {
    ensure!(
        input.domain.start <= *start && start <= end && *end <= input.domain.end,
        "interval must have ordered endpoints inside the calendar-time domain"
    );
    Ok(())
}

impl CellProblem {
    pub fn build(input: &LabourBoundInput) -> Result<Self> {
        ensure!(
            input.schema == "observatory.labour-bound-input/1",
            "unsupported labour bound input schema"
        );
        nonempty(&input.problem_id, "problem_id")?;
        ensure!(
            input.domain.start < input.domain.end,
            "domain must have positive duration"
        );
        ensure!(
            !input.streams.is_empty() && input.streams.len() <= MAX_STREAMS,
            "stream count must be in 1..={MAX_STREAMS}"
        );
        ensure!(
            input.observations.len() <= MAX_OBSERVATIONS,
            "too many observations (limit {MAX_OBSERVATIONS})"
        );
        ensure!(
            input.target.len() <= MAX_TARGET_WINDOWS,
            "too many target windows (limit {MAX_TARGET_WINDOWS})"
        );
        let mut indices = BTreeMap::new();
        let mut combinations = BTreeSet::new();
        let mut people = BTreeSet::new();
        for (index, stream) in input.streams.iter().enumerate() {
            nonempty(&stream.id, "stream id")?;
            nonempty(&stream.person_id, "person_id")?;
            nonempty(&stream.task_id, "task_id")?;
            nonempty(&stream.role, "role")?;
            ensure!(
                indices.insert(stream.id.as_str(), index).is_none(),
                "duplicate stream id {}",
                stream.id
            );
            ensure!(
                combinations.insert((&stream.person_id, &stream.task_id, &stream.role)),
                "duplicate task-person-role stream"
            );
            people.insert(stream.person_id.as_str());
        }
        let mut endpoints = BTreeSet::from([input.domain.start.clone(), input.domain.end.clone()]);
        let mut observation_ids = BTreeSet::new();
        for observation in &input.observations {
            nonempty(&observation.id, "observation id")?;
            ensure!(
                observation_ids.insert(&observation.id),
                "duplicate observation id {}",
                observation.id
            );
            validate_interval(input, &observation.start, &observation.end)?;
            ensure!(
                observation.lower_person_hours <= observation.upper_person_hours,
                "observation lower bound exceeds upper bound"
            );
            ensure!(
                observation.stream_ids.len() <= input.streams.len(),
                "observation has too many streams"
            );
            let mut selected = BTreeSet::new();
            for id in &observation.stream_ids {
                ensure!(
                    indices.contains_key(id.as_str()),
                    "unknown observation stream {id}"
                );
                ensure!(
                    selected.insert(id),
                    "duplicate stream in observation {}",
                    observation.id
                );
            }
            endpoints.insert(observation.start.clone());
            endpoints.insert(observation.end.clone());
        }
        for window in &input.target {
            ensure!(
                indices.contains_key(window.stream_id.as_str()),
                "unknown target stream {}",
                window.stream_id
            );
            validate_interval(input, &window.start, &window.end)?;
            endpoints.insert(window.start.clone());
            endpoints.insert(window.end.clone());
        }
        if let Some(output) = &input.known_output {
            ensure!(
                output.0 >= Rational::zero(),
                "known_output must be nonnegative"
            );
            for (index, window) in input.target.iter().enumerate() {
                ensure!(
                    window.weight.0 == Rational::one(),
                    "productivity requires unit-weight labour windows"
                );
                for previous in &input.target[..index] {
                    ensure!(
                        window.stream_id != previous.stream_id
                            || window.start == window.end
                            || previous.start == previous.end
                            || window.start >= previous.end
                            || previous.start >= window.end,
                        "productivity target windows must not overlap within a stream"
                    );
                }
            }
        }
        ensure!(
            endpoints.len() - 1 <= MAX_CELLS,
            "too many cells (limit {MAX_CELLS})"
        );
        let points: Vec<_> = endpoints.into_iter().collect();
        let cells: Vec<_> = points
            .windows(2)
            .map(|pair| TimeInterval {
                start: pair[0].clone(),
                end: pair[1].clone(),
            })
            .collect();
        let n = input.streams.len() * cells.len();
        ensure!(
            n <= MAX_VARIABLES,
            "too many cell variables (limit {MAX_VARIABLES})"
        );
        let rows = people.len() * cells.len() + 2 * input.observations.len();
        // Reserve for the worst case of one artificial variable per row. The
        // solver applies its own exact cap to the actual augmented tableau.
        let entries = (rows + 1).checked_mul(n + 2 * rows + 1);
        ensure!(
            entries.is_some_and(|count| count <= MAX_TABLEAU_ENTRIES),
            "joint problem exceeds exact tableau resource limit"
        );
        let mut a = Vec::with_capacity(rows);
        let mut b = Vec::with_capacity(rows);
        for person in people {
            for (cell_index, cell) in cells.iter().enumerate() {
                let mut row = vec![Rational::zero(); n];
                for (stream_index, stream) in input.streams.iter().enumerate() {
                    if stream.person_id == person {
                        row[stream_index * cells.len() + cell_index] = Rational::one();
                    }
                }
                a.push(row);
                b.push(&cell.end.0 - &cell.start.0);
            }
        }
        for observation in &input.observations {
            let mut row = vec![Rational::zero(); n];
            for id in &observation.stream_ids {
                let stream_index = indices[id.as_str()];
                for (cell_index, cell) in cells.iter().enumerate() {
                    if observation.start <= cell.start && cell.end <= observation.end {
                        row[stream_index * cells.len() + cell_index] = Rational::one();
                    }
                }
            }
            a.push(row.iter().map(|value| -value).collect());
            b.push(-&observation.lower_person_hours.0);
            a.push(row);
            b.push(observation.upper_person_hours.0.clone());
        }
        let mut c = vec![Rational::zero(); n];
        for window in &input.target {
            let stream_index = indices[window.stream_id.as_str()];
            for (cell_index, cell) in cells.iter().enumerate() {
                if window.start <= cell.start && cell.end <= window.end {
                    c[stream_index * cells.len() + cell_index] += &window.weight.0;
                }
            }
        }
        Ok(Self { cells, a, b, c })
    }

    /// Independently re-evaluate the original records on the returned cell masses.
    pub fn verify_witness(
        &self,
        input: &LabourBoundInput,
        masses: &[Rational],
    ) -> Result<Rational> {
        ensure!(
            masses.len() == input.streams.len() * self.cells.len(),
            "solver returned wrong witness size"
        );
        ensure!(
            masses.iter().all(|mass| *mass >= Rational::zero()),
            "solver returned negative labour"
        );
        let index: BTreeMap<_, _> = input
            .streams
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.as_str(), i))
            .collect();
        for (cell_index, cell) in self.cells.iter().enumerate() {
            let mut totals: BTreeMap<&str, Rational> = BTreeMap::new();
            for (stream_index, stream) in input.streams.iter().enumerate() {
                *totals
                    .entry(&stream.person_id)
                    .or_insert_with(Rational::zero) +=
                    &masses[stream_index * self.cells.len() + cell_index];
            }
            let duration = &cell.end.0 - &cell.start.0;
            ensure!(
                totals.values().all(|mass| *mass <= duration),
                "solver witness exceeds shared person capacity"
            );
        }
        let integral = |stream: usize, start: &Exact, end: &Exact| -> Rational {
            self.cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| *start <= cell.start && cell.end <= *end)
                .map(|(cell, _)| masses[stream * self.cells.len() + cell].clone())
                .sum()
        };
        for observation in &input.observations {
            let total: Rational = observation
                .stream_ids
                .iter()
                .map(|id| integral(index[id.as_str()], &observation.start, &observation.end))
                .sum();
            ensure!(
                observation.lower_person_hours.0 <= total
                    && total <= observation.upper_person_hours.0,
                "solver witness violates observation {}",
                observation.id
            );
        }
        Ok(input
            .target
            .iter()
            .map(|window| {
                &window.weight.0
                    * integral(index[window.stream_id.as_str()], &window.start, &window.end)
            })
            .sum())
    }
}
