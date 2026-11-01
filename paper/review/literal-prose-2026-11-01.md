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

First-pass PDF SHA-256:
`12f45601306b38706354ea26e843cfefb2c2990fb2f4b9ad3f77d64fcca62380`.
Build timestamps may change PDF bytes without changing the inspected text.

## Second pass against `8395320`

The user requested a fresh pass for missed wording problems. The primary editor
and two separate reviewing agents each read all six compiled sources. One agent
concentrated on reader effort and the other on mathematical fidelity. The
fidelity reviewer then checked the applied revisions and final compression.
Neither final review found a further material issue within its scope.

| Missed wording or relation | Repair |
| --- | --- |
| Abstract: an “exact” programme and “weighted differences” | State that the programme has the same possible labour totals even with overlapping observations and shared time; name inverse inclusion weights and the repeated-sample meaning of unbiasedness. The correction applies to the prediction total. |
| Introduction: allocations compared with available time | State the dimensionless constraint: at any time the fractions assigned to tasks and roles sum to at most one. |
| Introduction: person-intervals and inclusion probabilities | Define the sampling unit on first use and state that probabilities must be known and positive. |
| Introduction: an identification interval “contains every” consistent total | Say “contains exactly” and include the allocation constraints, preserving the difference between an exact range and a conservative enclosing interval. |
| Event model: parameters not determined “throughout” the family | State the counterexample directly: different parameter values can give identical commit distributions. |
| Calibration: predictions “measure independently observed person-hours” | Compare model predictions with independently measured person-hours. |
| Task regrouping: acceptance after “its last component” | Specify that all components must first have been accepted. |
| Task age | State that age is measured from enrolment. |
| Labour accounting: work “assigned to the task after acceptance” | Count work done after acceptance that is attributable to the task; the time of its assignment is irrelevant. |
| Three clocks: quantities “measured at different times” | Distinguish the timing rules without asserting that their numerical times differ. |
| Capacity example | Repeat the essential one-person assumption next to the example. |
| Interval proof: “Their intersection” | State that the preceding inequalities together give the bound. |
| Linear target: “continuous target” | Identify the continuous-time allocation history and the weighted sum of its cell integrals. Real weights, including negative weights, remain allowed. |
| Reconstruction: “feasible set of cell labour totals” | Describe an assignment of totals satisfying the linear constraints, rather than conflating a feasible vector with the feasible set. |
| Variance: “improves on HT” and “labour itself” | Name lower sampling variance as the comparison criterion and person-interval labour totals as the quantities whose population variance is compared with residual variance. |
| Variance: “increases variance fourfold” | Name HT as the baseline. The second predictor has four times the HT variance, not four times the first predictor's variance. |
| Noninferiority: “the margin is met” | Define the maximum acceptable quality reduction and say that failure to detect a reduction does not establish that the loss is within that limit. |
| Floating-point bounds “outside the exact bounds” | State the containment guarantee at issue: the rounded interval contains its exact-arithmetic counterpart. |

Repeated introductory and concluding announcements were removed where the
neighbouring result already states their content. The labour definition,
measurement boundaries, capacity constraint, attainable bounds and condition for
improving those bounds remain explicit. The shorter sampling instructions still
divide sampled observed-minus-predicted labour by inclusion probability and add
these corrections to predictions fixed before sampling.

Protected-content comparisons against both `8395320` and the original `3df96e7`
preserve all 20 displayed mathematical blocks, 226 inline mathematical
expressions, 74 reference-related commands, 58 environment commands and the
bibliography bytes. No mathematical expression, citation or cross-reference was
added, removed or changed. The positivity premise was already present in the
estimator proposition; this pass makes it explicit in the summaries too.

The final article remains **13 pages**. All pages were rendered and visually
inspected; their text stays within the page boundaries. The final LaTeX log has
no undefined references/citations or overfull/underfull boxes. The existing
BibTeX missing-pages warning remains. `make -C paper check` and
`git diff --check` pass. These checks and the separate agent reviews provide
bounded editorial verification, not a claim that no possible defect remains.

Second-pass PDF SHA-256:
`15921bf4b606554c1a3692e4cbc57812b9532c1168e1cdb0b522bbadc4e0a36f`.

## Final pass and Reproducibility removal

Baseline: `4f174560ebe8a2506ee5780d5b0a2bfc2a84859a`. At the user's request,
the Reproducibility heading and both paragraphs are removed. The conclusion
now leads directly to the scalar appendix. No surviving passage refers to the
removed section, and no proof premise depends on it.

The primary editor and two new reviewing agents read all six remaining compiled
sources. The wording reviewer found no further material issue. The mathematical
reviewer identified one definitional edge case: an inconsistent record must not
count as point-identifying labour merely because there are no admissible
allocations to compare. The definition now requires at least one allocation
satisfying the record and constraints, and the same labour total for every such
allocation. This matches the later proposition's treatment of an empty feasible
set. No other wording change was needed.

Verification against the baseline permits exactly the section deletion and that
definition change. All 20 displayed mathematical blocks, 226 inline mathematical
expressions, 58 environment commands, 33 labels, citations and bibliography bytes
remain unchanged. The deleted section contained one repository URL command, so
there are now 73 retained reference-related commands rather than 74. Those 73
commands retain their exact bytes and order; all 13 cited sources resolve.

`make -C paper article` builds 13 pages. Reproducibility is absent from the
source and extracted PDF text. Every final page was rendered and visually
inspected; all text stays within the page boundaries. The final LaTeX log has no
undefined references/citations or overfull/underfull boxes. The existing BibTeX
missing-pages warning remains. `make -C paper check` and `git diff --check` pass.
These checks are bounded to the current manuscript and retained reference
calculations; they do not constitute empirical validation.

Final-pass PDF SHA-256:
`660ac7ec4833eac2dc22b7625b5831449e3c0669a46a1ad59d7e336ce634696c`.


## Shared-capacity figure added after the prose review

At the user's request, Figure 1 now visualises the Section 4.1 two-task example.
The exact calculation, include-ready `tikz.tex`, separate caption, vector PDF,
rendered alternative, specification, audit and independent agent review are
retained under [figures/shared-capacity](../figures/shared-capacity/README.md).
The figure is a constructed mathematical example. Its square contains pairs
permitted by separate bounds; its diagonal and singleton enforce shared capacity
in both hours. No empirical or statistical-coverage interpretation is intended.

The figure is placed on page 6 before the general cell formulation. The full
article now has 14 pages. Existing mathematical expressions and bibliography
bytes are unchanged. The manuscript's Reproducibility section remains removed.
The figure verifier, exact calculation, paper checks and rendered inspection pass;
see the figure's audit and review for their bounded scope and qualifications.
