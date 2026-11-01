# Terminology repair and mathematical model review

The joint linear programme remains the best-supported general formulation among
the alternatives examined. The revision strengthens its interpretation and makes
its bounds easier to check. It does not claim that every possible alternative
has been excluded, that the measurements are empirically valid, or that a new
model has received formal proof certification.

## Changes included in the paper

The terminology review covers all six compiled sources and the figure caption.
It distinguishes conditions for including a task in a study from permission to
use a tool; original comparison-group allocation from actual tool use; and
attributing person-hours to a task from classifying shared supporting work.
Work that cannot be attributed to a task is recorded separately. The index
`r` now explicitly denotes type of work, such as implementation or review.
The equations and indices retain their meanings.

Task identifiers, component counting, scoring inputs, calibration populations,
the endpoint of labour recording, follow-up checks and independent record
checking are stated directly. Scoring rules kept unchanged across groups are
distinguished from predictions that remain unchanged whichever sample is drawn.
The mathematical expression “difference-constraints representation” remains
because its object and relation are explicitly defined. An independent final
wording review also checked the new mathematical passages.

Three mathematical changes are included:

1. **Full attention suffices.** Every feasible cell vector can be realised by a
   finite schedule in which each person does one activity at a time. In each
   person-cell, place the stream totals in consecutive blocks and leave the
   remainder idle. Their indicator functions reproduce every cell integral.
   Interpolate feasible cell vectors and reconstruct each one to attain every
   intermediate target total. Fractional cell totals require no simultaneous
   fractional attention. Arbitrary interruption remains permitted; session
   lengths, precedence, synchronisation and switching costs are not inferred.

2. **Bounds have arithmetic certificates.** With nonnegativity kept separate,
   `P = {x >= 0 : A x <= b}`, nonnegative multipliers satisfying
   `A^T lambda >= w` and `A^T mu >= -w` certify
   `-mu^T b <= w^T x <= lambda^T b`. Published finite-dimensional LP strong
   duality supplies tight multipliers when the cell polytope is nonempty.
   Matching them to feasible attaining vectors certifies both endpoints.
   For relaxed observation bounds `b + epsilon`, the same multipliers give
   valid, possibly conservative, bounds. No derivative or exact value of an
   additional measurement is claimed.

3. **Sampling bounds need simultaneous coverage.** If all supplied observation
   intervals contain their true totals simultaneously with probability at
   least `1 - alpha`, the resulting optimised interval covers true target
   labour with at least that probability, conditional on the structural
   assumptions being correct. An empty feasible set counts as failure to
   cover. Constructing those observation intervals remains specific to the
   sampling and measurement process. The paper does not turn a sampled point
   estimate into an exact observation.

The original fixed-prediction difference estimator, its variance conditions,
the scalar shortest-path result and the fixed-output productivity formula are
retained. The previous Reproducibility-section removal and editable TikZ figure
remain in place.

## Alternatives and published results

| Approach | Finding and decision |
| --- | --- |
| Finite-cell LP | Retains arbitrary overlapping observation groups and shared person-time constraints. Keep it as the general formulation. |
| Scalar difference constraints | Exact for the stated single-stream special case. Independent scalar bounds lose shared-capacity coupling. Keep the appendix's scope. |
| Full-attention reconstruction | Same feasible cell and target sets, with an elementary finite-block proof. Include the stronger statement and proof. |
| General purification theorem | Related published structure, but more machinery than needed for the finite partition. Use the explicit construction; make no novelty claim. |
| Integral-flow shortcut | The permitted three-pair observation matrix has determinant minus two and forces half-hour cell totals from integer inputs. This refutes automatic total unimodularity of the original matrix, without excluding every possible extended formulation. |
| Equal-column aggregation | Exact within a person-cell when every observation and requested target coefficient matches, with a feasible lift. Optional preprocessing; no guaranteed reduction. Not implemented as an application feature in this revision. |
| LP duality and sensitivity | Adds exact certificate checks and valid bounds after observation-error limits are widened. Include the concise published-theorem application. |
| Linear-fractional optimisation | Charnes–Cooper applies to a justified joint polyhedral output–labour model with a positive denominator. The paper supplies no such joint model. Keep the simpler exact fixed-output formula; do not invent coupling constraints. |
| Fitted calibration, cross-fitting or automatic predictor rescaling | Not a drop-in replacement for the current exact finite-population unbiasedness argument. An oracle coefficient needs unknown labour covariances. Defer without evidence and a design-specific proof. |
| Sampling confidence-region projection | Valid under simultaneous coverage; the same LP handles rectangular/polyhedral observation regions. Include the general implication, without promising a universal interval-construction method. |

Primary sources inspected by the coordinator or independent workers include:

- Boyd and Vandenberghe, *Convex Optimization* (2004),
  [Sections 5.2–5.3 and 5.6](https://web.stanford.edu/~boyd/cvxbook/bv_cvxbook.pdf).
  The coordinator and checker separately inspected the relevant full text.
  The paper cites this source for dual attainment and perturbation bounds.
- Charnes and Cooper, [“Programming with linear fractional functionals”
  (1962), pp. 181–186](https://iiif.library.cmu.edu/file/Cooper_box00010_fld00009_bdl0001_doc0001/Cooper_box00010_fld00009_bdl0001_doc0001.pdf).
  The transformation was inspected as a conditional alternative, not used
  to justify an output–labour relationship absent from the observations.
- Khan, Rath and Sun, [“The Dvoretzky–Wald–Wolfowitz theorem and purification
  in atomless finite-action games”](https://doi.org/10.1007/s00182-005-0004-3).
  The formulator and checker inspected the author-posted theorem text.
  Its general existence theorem is related to the elementary reconstruction;
  the paper's finite-block bound follows from its own construction.
- Denis and Haddou, [“Prediction-Powered Estimation: Unbiased Model-Assisted
  Estimation”](https://doi.org/10.1177/0282423X261451320), Section 2.3.
  Its fixed-prediction result supports retaining the existing estimator.
- Bardenet and Maillard, [“Concentration inequalities for sampling without
  replacement”](https://arxiv.org/pdf/1309.4029), Corollary 2.5.
  The estimator and checker inspected the full-source bound. Its SRS residual
  interval is a possible design-specific implementation, beyond the general
  projection argument included in the manuscript.

Detailed peer reports retain the counterexamples, hypotheses and source-access
limits. Optional scheduling and aggregation literature is discussed there;
it is not a dependency of the included proofs.

## MAS Math Lab execution

Applied the installed `model-with-mas` and `run-mas` skills in host-assisted mode.
The packaged runtime version is 0.3.0; setup verified its source digest, installed
Rust 1.85.1 in task-local storage, built the locked Rust/nom 8.0.0 codec, and
performed an actual round trip. No provider calls were issued.

Run `73f2f5977c5349f3a110c4d50d2acfcc` registered the target, baseline hashes,
boundaries, comparison criteria and research-prototype resource limits before
worker execution. Four separate fresh native contexts served as formulator,
challenger, estimator and checker. Their initial work used the same immutable
baseline from commit `bd29b2b6faa05afb519b9f9d23d99d3d7e1d3377`.
The checker saved an independent initial audit before receiving candidates.
The formulator and checker then consumed actual peer deliveries from the Rust
bus. Six native turns produced five structured bus responses plus the checker's
initial written audit. The bus contains ten messages, of which six were consumed
and acknowledged. The remaining four deliveries were not counted as consumed.

The coordinator contributed the duality/perturbation proposal and primary-source
search, checked the serial construction, selected the bounded manuscript changes,
and coordinated implementation and finite calculations. Native preliminary
updates and authorised shared-file reads are recorded as additional information
channels. This is logical context separation on a shared filesystem, not process
isolation. Protocol adaptation was disabled. No comparison of MAS efficacy with
single-agent work was performed.

Native token use and monetary charges are unavailable. The runtime's provider
accounting does not meter native host calls. No zero-cost or matched-budget
superiority claim is made.

The included results have written proofs, independent proof/source review and
exact finite computational checks. New Lean certification is unavailable: the
pinned toolchain is absent and the packaged theory registry covers four supplied
shear-matrix claims, not this paper. No restricted-registry result is substituted
for a proof of the labour model, and no new model is promoted to the plugin's
formal acceptance status. Empirical measurement adequacy remains unestablished.

The checkpoint and bus exports retain the execution evidence. No Library-save
tool was available; the checkpoint is retained in Git and the downloadable
deliverables, with no claim of a Library item ID or version. It preserves the
original deadline; a later investigation requires a new bounded run with prior
disclosures, rather than extending this run's allowance.

## Computation and reproduction

Run from the repository root:

```sh
python3 paper/review/mas-model-2026-11-01/calculations/check_model.py --check
make -C paper check
make -C paper article
```

The dependency-free exact checker covers 1,225 exhaustive two-cell histories,
33,075 independently integrated equalities, 284 scalar parameter cases,
figure and relaxed-bound certificates, infeasibility/unavailability cases, and
structural counterexamples. `--check` does not rewrite recorded results or timings.

Five structured cases have 4, 8, 16, 32 and 48 variables. The largest has 72
explicit inequalities, or 120 including nonnegativity. Exact primal and dual
checks give zero gaps. Recorded timings cover assembly, certificate verification,
serial reconstruction and target evaluation, with separate geometric integration
checks. They are not benchmarks of a general LP solver or evidence of arbitrary-
input tractability. See `calculations/scaling.json` for measurements, input-bit
limits, complexity qualifications, environment and memory accounting.

`contract.json`, `candidate.md`, peer reports, actual response JSON, bus inspection,
replay and `checkpoint.zip` preserve the bounded execution. Recreate the packaged
runtime through MAS Math Lab's `run-mas` skill, then use `cloud.py restore` with a
new database path to inspect the checkpoint. Baseline sources can be recovered
from the recorded Git commit and checked against the contract's source hashes.
