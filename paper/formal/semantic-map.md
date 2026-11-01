# Labour-bound extension: semantic concordance map

This file defines the intended correspondence for independent review. It is not
itself a proof-check receipt. The immutable source target is
`targets/partial-identification.tex`, recovered from repository commit
`38c4ce40b789d4c5420c98ff1d568b39c3b1e798`. Later editorial changes, the timeline
panel and citations do not change these mathematical targets.

## Objects and domains

- `ℝ` represents calendar hours, person-hours and real target weights. Units are
  documented distinctions, not separate Lean dimension types.
- `cellMass a b k` is the actual Lebesgue interval integral of history `a` between
  endpoints `b k` and `b (k+1)`. It is not an unconstrained independent variable.
- A finite grid is encoded by `b : ℕ → ℝ` and a cell count `K`; `GridOrdered b K`
  constrains only endpoints through `K`. No assumption concerns the unused tail.
  The formal construction also permits equal consecutive endpoints, necessarily
  with zero feasible masses. The paper's strictly increasing grid is a special
  case, so this broadening adds no premise to its result.
- The first `n` natural indices represent one person's task/work-type streams.
  The construction applies independently to every person. Grouped integral
  identities use arbitrary finite stream types and therefore also apply to
  groups spanning several people.
- The domain-localised forward results assume almost-everywhere strong
  measurability under `volume.restrict D`, nonnegativity only inside `D`, and
  person-time capacity almost everywhere under that restriction. Raw function
  values outside the study domain need no bounds or measurability. This avoids
  silently requiring Borel measurability or global capacity for every admissible
  history. The constructed indicator histories are Borel measurable and satisfy
  capacity at every point, which is stronger than required.

## Integral and finite-cell correspondence

`bounded_measurable_intervalIntegrable` derives integrability on bounded
intervals from measurability and almost-everywhere bounds. It does not assume the
integrals already exist with the desired values.

`allocation_integrable_of_capacity` derives the individual upper bound from
nonnegative finite stream allocations and the person's almost-everywhere sum
constraint. `history_to_cell_constraints` integrates the pointwise conditions:
every cell integral is nonnegative and the sum of a person's cell integrals is
at most the cell duration. This is the forward capacity step of
Proposition `prop:cell-reduction`.

The source model needs these assumptions only on the bounded study domain.
`zero_extension_measurable`, `zero_extension_nonnegative` and
`zero_extension_capacity` derive their global versions for the zero extension.
`zero_extension_preserves_integral` proves equality of the original and extended
integrals on any subinterval whose closed interval lies inside `D`.
`localized_history_intervalIntegrable` and
`localized_history_to_cell_constraints` therefore establish the forward results
for the original raw history without assumptions on its values outside `D`.
The local helper even permits arbitrary measurable `D`; the paper's bounded
calendar domain is a special case. Every integrated subinterval is bounded.

`cell_integrals_recover_interval` invokes the checked adjacent-interval
additivity theorem for actual integrals. `grouped_observation_from_cells` sums
that equality over a finite group of streams. `interval_observation_iff_cells`
rewrites the original lower/upper observation inequalities using those cell
integrals. Its helper integrability assumptions are discharged for admissible
bounded histories by the preceding integrability results.
`grid_cell_subset_domain` proves cell containment from the finite grid order,
and `localized_grouped_observation_on_grid` assembles those integrability and
additivity results directly for the original domain-localised history.

## Binary reconstruction and stitching

`prefixMass` is the finite prefix sum of supplied stream masses. `serialBlock`
is the half-open interval between successive prefix sums inside a cell.
`serialAllocation` is that interval's indicator, with real values zero and one.

The prefix lemmas prove block containment and disjointness from nonnegative
masses and the cell-capacity bound. `serialAllocation_integral` proves that the
Lebesgue integral of each indicator is its block length, hence its supplied cell
mass. `serialAllocation_cell_integral` proves the same equality with integration
restricted to the person's cell. The binary, measurability and capacity lemmas
establish the required pointwise properties.

`stitchedAllocation` sums a stream's indicators over finitely many calendar
cells. `stitchedAllocation_on_cell` proves that only the matching cell can
contribute there. The subsequent lemmas prove binary values, measurability,
person-time capacity, exact cell integrals and interval integrability for the
entire history, including intervals crossing cell boundaries.

`reconstructed_cellMass` connects the stitched history to the actual interval
integral used by `cellMass`. Endpoint choices have no Lebesgue integral effect;
the construction uses half-open intervals, so it requires no overlapping endpoint
allocation. `reconstructed_observation` proves that every endpoint-aligned
grouped observation equals its finite cell sum. This is a checked stitching
result, not an assumption that the individual cell constructions are compatible.

`full_attention_reconstruction` packages the existential result: for every
feasible finite mass array there is a measurable, binary history satisfying
person-time capacity, every required cell integral and every endpoint-aligned
grouped observation. `reconstructed_weighted_target` proves preservation for
arbitrary real cell/stream weights, including negative weights. Thus the
construction preserves the target quantities used in the paper.

The history is explicitly defined using finitely many indicators. A separate
minimal-piece or switch-count theorem is not included in this proof scope.
The numerical `K*(n+1)` bound discussed in the paper is derived there from the
construction, but receives no separate formal-verification status here. Minimum
session lengths, limited switching, precedence and co-working synchronisation
are not hypotheses of the model and are not preserved.

## Certificates, perturbations and coverage

`LabourBounds.Certificates` uses arbitrary finite real matrices and vectors.
Nonnegativity of the primal vector is separate from `A x <= b`; consequently
dual feasibility is `Aᵀλ >= w`, not equality. Lower bounds apply the same theorem
to the negative objective. Matching feasible primal/dual objective values prove
attained greatest/least values of the feasible target image. The theorems check
supplied multipliers; they do not prove that a particular solver finds them.

The relaxed-bound theorems use the same matrix and target with changed right
hand sides. Nonnegative relaxations imply feasible-set inclusion and outward
movement of reused-certificate bounds; certificate validity itself needs no
sign assumption on the perturbation when feasibility is separately supplied.

Coverage results first prove event inclusion from a fixed true feasible vector
and simultaneous lower/upper observation coverage. A generic projection theorem
explicitly assumes an enclosing target interval. The more specialised signed-row
assembly proves enclosure from feasible dual certificates instead. Probability
monotonicity transfers the lower probability bound, with explicit event
measurability and `0 < alpha < 1`. Empty feasible sets count as failure. The
validity of the supplied observation intervals and the measurement/sampling
process are empirical or design premises, not proved facts.

## Remaining boundaries

General LP strong-duality existence and the compactness proof establishing
existence of extrema are not newly formalised. Published mathematical support
remains identified in the paper. No Lean claim is made for empirical accuracy,
frame completeness, causal identification, the Rust simplex implementation,
Python tests, generic solver complexity or a source-language refinement.
