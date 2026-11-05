# Exact labour identification

`observatory bound-labour FILE` implements the finite-cell reduction in
*How Much Human Work Is Behind AI-Assisted Software?* It computes the smallest
and largest target values consistent with declared interval observations and
each person's shared capacity across tasks and types of work. It runs offline without
opening the evidence store.

```sh
cargo run --locked -- bound-labour examples/joint-labour-bounds.json
```

The example gives one person two tasks, each requiring one person-hour within
the same two-hour interval. Their combined labour in the first hour is exactly
one person-hour. Adding the separate marginal intervals would instead give
`[0,2]`. Both endpoint allocations are returned so the result can be inspected.

## Input contract

The schema is `observatory.labour-bound-input/1`. Unknown fields are rejected.
All numerical input is an exact rational **string**, such as `"2"`, `"3/2"` or
`"1/3600000000000"`. JSON floating-point numbers and decimal strings are rejected.
Times are hours from one declared common calendar origin. Each interval uses
that same origin, including observations outside the target window. Labour
quantities are person-hours. The input does not convert task-relative ages.

| Field | Meaning |
|---|---|
| `problem_id` | Nonempty identity of the declared problem |
| `domain` | `start` and `end` enclosing every observation and target window |
| `streams` | Unique stream `id` and unique `(person_id, task_id, role)` combination; people may appear in several streams |
| `observations` | Unique `id`, `start`, `end`, `stream_ids`, `lower_person_hours`, `upper_person_hours` |
| `target` | Additive terms with `stream_id`, `start`, `end` and signed `weight` |
| `known_output` | Optional exact nonnegative sum of weighted scores for a conditional productivity range |

The existing JSON field `role` identifies the type of work, such as implementation
or review. Its name is retained for input compatibility.

An observation constrains the sum of labour across its listed streams during
its interval. Equal lower and upper values declare an exact total. Different
values declare deterministic bounds justified by the measurement process.
Overlapping observations constrain the same allocations and are not added as
independent contributions. Include competing tasks outside the target when
observations of those tasks constrain the same person's capacity.

Target terms define a linear quantity. Ordinary cohort labour uses weight one
on each included stream's window from the start of tracking to its stated end. For productivity,
`known_output` requires this ordinary nonduplicated labour target. The result
reports the sum of weighted scores divided by the opposite labour endpoint only when the minimum
labour is positive. General signed weighted targets do not establish a
productivity denominator.

## Calculation and output

The implementation partitions time at all domain, observation and target
boundaries. A nonnegative variable represents each stream's labour in each
cell. Every person's cell variables sum to at most the cell duration.
Observation bounds and target weights become exact linear constraints and an
objective. Two-phase rational simplex solves the minimisation and maximisation,
using deterministic Bland pivots. No floating-point tolerance decides feasibility
or optimality.

A `feasible` `observatory.labour-bound/1` result contains exact `minimum` and
`maximum` values, ordered cells, and allocations attaining both endpoints.
Each allocation supplies the stream's person-hours in each cell. Dividing by
cell duration gives one compatible history with allocations constant within each
cell. The revised paper proves a stronger construction: place each person's
stream totals in consecutive blocks within the cell, doing one activity at a
time and leaving any remaining time idle. Both constructions preserve the same
cell totals and observations. The command returns cell totals, rather than a
chronological schedule. It rechecks both returned vectors against every
constraint and target value using exact arithmetic before reporting them.

The paper also explains how an attaining allocation and bound multipliers can
prove that a minimum or maximum is optimal. Simultaneous uncertainty bounds on
observed interval totals can also give a coverage guarantee for target labour.
The command currently exports no bound multipliers and calculates no sampling
confidence intervals. Its exact simplex calculation and primal feasibility
checks remain separate from those additional features. The research examples
in `paper/review/mas-model-2026-11-01/calculations/` check supplied certificates
and schedules generated for those examples; they are not a general certificate-producing solver.

An `inconsistent` result has no extrema: the supplied observations and capacity
constraints admit no allocation. Invalid input and resource exhaustion are errors,
with no numerical bound emitted. The solver limits tableau size, pivots and
rational coefficient growth; see the constants in
[`simplex.rs`](../src/identification/simplex.rs) and the input limits in
[`reduction.rs`](../src/identification/reduction.rs). This is a bounded implementation
of the model, not a claim of practical scalability to arbitrary portfolios.

The guarantee is conditional on the declared streams, observation bounds and
per-person time limits. The equivalent one-activity-at-a-time construction permits
work to start, stop and change activity at arbitrary times. Software checks do not establish roster
completeness, accurate measurements or the absence of task-interruption
restrictions. Sampled labour estimates do not become exact observation totals
by entering them here. Use [`estimate-labour`](design-based-estimation.md) for
sampling uncertainty.

## Compatibility with study accounting

The existing task/cohort reports preserve their additive effort-record contract
and floating-point marginal bounds. Their method names still disclose that the
sums need not be jointly attainable. General overlapping interval observations
belong in this separate constraint input. Relaxing the old record validator
would let repeated observations become duplicated effort.

No database or snapshot migration is required. The standalone result preserves
its problem identity and declared constraints. Users must retain the supplied
input and source evidence with the result to establish what was observed and
when it became available. No causal AI effect or empirical calibration follows
from solving this model.
