# Quantities and relations in the labour paper, 1 November 2026

Baseline: `3df96e77b213db4c0cd46e0f15e313ed0ab83d8b` on `main`.

The review asks whether a reader must translate an abstract phrase back into
the person-hours, time period, work allocation or mathematical relation it
describes. Revisions name those objects and relations where doing so improves
the explanation. Established mathematical terms remain when their meaning is
defined locally or made explicit by the adjacent formula and proof.

## Coverage

Every paragraph was read in the article's include order and checked again in
reverse section order. The review includes the abstract, proposition statements,
proofs, examples, implementation descriptions, reproducibility statement and
technical appendix.

| Compiled source | Material reviewed and revised |
| --- | --- |
| `article/main.tex` | Abstract, introduction, event model, task accounting, sampling introduction, validation, intervention comparisons, conclusion and reproducibility. |
| `article/event-history.tex` | The three measurement times; realised labour versus identification from a population distribution. |
| `article/partial-identification.tex` | Definition of the identified range, person-time capacity, simultaneous feasibility of component bounds, cells, reconstruction and productivity ratios. |
| `article/design-based-estimation.tex` | Residual weighting, fixed predictions, repeated-sample variance, failure examples and implementation checks. |
| `article/sampling-validation.tex` | True labour versus sampled estimates, squared-error comparisons, reference scores and sensitivity of intervention rankings. |
| `article/scalar-bounds.tex` | Cumulative labour, exact observations, shortest-path values and reconstruction of allocations. |

There are no figures or captions in the current include graph. The bibliography
was read in the rebuilt PDF and its source is unchanged. Unincluded LaTeX files,
the historical paper and proof package, the forecasting companion, application
code and recorded calculations are outside the requested paper revision.

## Wording decisions

The following groups account for the recurring defects across the six sources;
the source diff contains the individual sentence revisions.

| Location and original wording | Quantity or relation made explicit |
| --- | --- |
| Abstract: “Its extrema delimit all attainable totals” | The minimum and maximum target totals, and the possibility of every intermediate total under the same constraints. |
| Abstract: “Summing separate interval bounds can give a wider range” | A summed interval can contain totals that no joint allocation can produce. |
| Introduction: “Both limits are attainable” | Work one hour in the first half and two in the second, or two in the first and one in the second. |
| Introduction: “The interval total therefore identifies a range without identifying the allocation within it” | Knowing the three-hour total does not determine the amount of work in the first two hours. A labour total is distinguished from the timing of work. |
| Introduction: “narrow the range” | Additional measurements could rule out some previously possible labour totals. |
| Introduction: “Narrowing that range therefore requires” | Establishing a higher minimum or lower maximum requires additional observations or justified restrictions. This remains a necessary condition, not a promise that every new observation improves a bound. |
| Introduction and estimator: “weighted residual correction” | Subtract predicted labour from observed labour, divide by inclusion probability, and add the weighted differences to the prediction total. |
| Literature: “forecast-adjusted log-time contrast” and “interchangeable parameters” | The times being compared and adjusted, and the different populations, interventions and outcomes to which the studies' estimates refer. |
| Event model: “activity laws”, “event law” and “different target values” | Distributions of activity and commits, and expected numbers of active minutes. |
| Task accounting: “task granularity”, “the unit receiving acceptance credit” and “the score” | What counts as one task, which task is accepted, when follow-up begins, and whether component scores add to a parent's score. |
| Reference score: “canonical task representation” and “additive requirement rubric” | Score the same recorded task representation once, or use component scores that add to the total. |
| Accounting: “the chosen boundary”, “numerator and denominator” and “three clocks” | The end of labour recording, retained output and person-hours, and the distinct times for acceptance, retention and labour. |
| Interval bounds: “the allocation within the interval unresolved” | Uncertainty about the first-hour labour total and uncertainty about exactly when the person worked. |
| Summed bounds: “component allocations can coexist” | Allocations achieving all component minima, or all component maxima, must be possible together within each person's available time. |
| Finite reduction: “streams”, “cells”, “nonzero width” and “cell masses” | Task–person–role combinations, intervals between successive endpoints, different observation bounds, and cell labour totals. |
| Finite reduction: “the histories and the polytope yield” | The original allocation functions and the finite feasible set give exactly the same possible target values. |
| Productivity: “monotonicity and attainability” and “an admissible output–labour pair” | Divide fixed output by maximum and minimum labour; output and labour totals must be jointly permitted to produce a claimed ratio. |
| Sampling: “fixed under the sampling design” and “fixed-vector proof” | Each unit's prediction remains the same whichever sample is drawn; the proof does not cover fitting predictions to the sample used for correction. |
| Variance: “population identity”, “joint inclusion probabilities” and “noncensus” | The variance formula, the probability that two units are selected together, and selecting one unit from a larger population. |
| Prediction validation: “squared-loss difference” and “either loss level” | Squared prediction errors calculated against sampled or true labour, their average difference over validation samples, and each predictor's error against true labour. |
| Validation and intervention comparison: “sampling intensity”, “joint range”, “transmit intervention effects” and “assignment units” | How many person-intervals to sample, changes in the minimum or maximum labour total, effects through shared reviewers or resources, and units assigned to interventions. |
| Reproducibility: “interval geometry” and “without an outward-rounding guarantee” | Whether declared intervals cover a person's window without gaps or overlaps, and whether rounding keeps the reported bounds outside the exact bounds. |
| Appendix: “exact mass” and “potentials rooted at” | An observed labour total and shortest-path values from a starting vertex, normalised before reconstructing cumulative labour. |

“Identified range”, “convex combination”, “compact”, “linear functional” and
“shortest path” remain in their defined mathematical uses. “Both endpoints and
every value between them are attained” remains in the formal proposition,
immediately after its formula and before the constructive proof. These terms
state mathematical properties at the point where the argument needs them.

## Mathematical fidelity

A comparison with the baseline preserves, byte for byte and in occurrence order
within each source file:

- All **20 displayed mathematical blocks**.
- All **226 inline mathematical expressions**.
- All **74 label, reference, citation, input, URL and bibliography commands**.
- All **58 environment begin/end commands**.
- The complete bibliography source.

All **33 labels** are unique and all **13 cited sources** resolve. The semantic
review also checked the prose premises and qualifications: measurable fractional
allocations, shared capacity, signed linear weights, infeasible observations,
zero-length pieces, positive labour denominators, jointly feasible output and
labour, fixed predictions, observed zeros, nonresponse, negative estimates,
single-unit samples and census cases. The sufficient Cartesian-product condition
for productivity bounds remains sufficient; it is not made necessary. The
finite examples remain computational checks, and measurement accuracy and causal
AI effects still require empirical evidence.

The bundled skill scanner was unavailable through the supplied skill resource.
Candidate searches and a recurring-term concordance were therefore made locally,
followed by contextual review of every occurrence. Scanner output is not treated
as evidence of semantic equivalence or readable prose.

## Verification

- `make -C paper article` builds **13 pages**, compared with 12 in the baseline.
  All pages were rendered and visually inspected; the opening example, finite
  reduction, conclusion and appendix were also inspected at page resolution.
- The final LaTeX log has no undefined references, undefined citations or
  overfull boxes. One underfull vertical-box notice on page 11 was inspected;
  the page has no clipped or overlapping material. BibTeX retains the existing
  missing-pages warning for `cui2026`.
- Text bounding boxes on every final PDF page remain within the page.
- `make -C paper check` passes: historical package hashes, inventory and receipt;
  the sampling-noise example and 2,304 exact bin cases; the exact rational
  identification cases; the forecasting reference self-test; seven Rust kernel
  tests; and 11 Python collector tests.
- `git diff --check` passes.

Application code, mathematical expressions, dependency files, reference data and
historical proof evidence are unchanged. The original Lean proofs were not
rerun. This is a bounded editorial and mathematical-fidelity review of the
current article, not an empirical validation or an external peer review.

Final PDF SHA-256:
`12f45601306b38706354ea26e843cfefb2c2990fb2f4b9ad3f77d64fcca62380`.
Build timestamps may change PDF bytes without changing the inspected text.
