# Independent review of sharp identification and design estimation

Review date: 29 October 2026. Base: `b89a2584ef34637b71f5036a67673250b382d4e2`.

This review used a separate adversarial context. Acceptance obligations and a finite test corpus were constructed before inspecting the new implementation. The reviewer did not edit the manuscript or production code. The mathematical, application, sampling, manuscript and source authors were questioned directly about their assumptions and implementation boundaries. Agent review is a development aid, not external peer review.

## Mathematical audit

The one-interval result is sharp: capacity before and after the target window supplies both inequalities, and filling either part first supplies endpoint witnesses. Intermediate values follow by convexity. A complete mass over a boundary-crossing interval does not justify uniform interpolation.

The finite-cell proof preserves every observation interval and target boundary in absolute calendar time, including portions outside a target window. Every continuous admissible history yields feasible cell masses; piecewise-constant reconstruction yields a history for every feasible mass vector. Shared person capacity couples task and role streams. Compactness and convexity establish attained endpoints and every intermediate target value. Empty feasibility is inconsistency, not a zero estimate. The scalar graph has the correct forward upper-bound and reverse negative-lower-bound edges; shortest-path potentials establish feasibility and attain both target endpoints. These proofs do not require a sampling interpretation.

The fixed-prediction difference estimator, general design-variance identity and SRS variance formula were checked algebraically. The sample-variance derivation uses the actual fixed-size uniform design and its pair inclusion probability. Noncensus singleton samples do not identify that variance estimate; a census has zero sampling variance. No assumption of independent inclusion indicators is introduced. The shared-noisy-reference squared-loss comparison already present in the baseline remains intact.

Independent computational evidence:

- 870 exhaustive one-interval mass/window cases were checked by binary allocation enumeration.
- A pre-existing corpus of 2,500 scalar systems, including 1,834 infeasible systems, was compared with the author's exact rational graph checker. All endpoints and infeasibility decisions agreed with independent enumeration.
- The same 2,500 cases were transformed to fractional calendar endpoints and masses. All comparisons passed, and returned endpoint witnesses were independently integrated against every observation and target. These are 5,000 differential cases altogether.
- Three independent continuous LP fixtures checked shared-person constraints on three absolute-time cells. Separate task ranges overstate joint maxima or understate joint minima; shifted task windows have different capacities from coincident windows.
- Complete enumeration of six SRS samples independently gave total 7, variance `251/3`, and a legitimate negative estimate of -4. Four unequal-probability outcomes preserved the fixed total in design expectation.

These finite checks support the analytical review; they are not a machine-checked proof of the continuous theorem. The general shared-capacity LP is not implemented by the application or by the scalar reference checker.

## Implementation audit and findings

An external Rust harness called the public analysis and inference APIs. Its fixtures and expected values were maintained outside the production crate in the review workspace. The final completed run contained 241 requests: 214 valid inference inputs, 15 rejected invalid inputs, and 12 boundary-accounting cases. It covered 200 independently generated SRS populations, exhaustive and unequal-probability examples, empty realised general samples, selected nonresponse, missing frame/freeze attestations, duplicate identities, strict schemas, overflow, direct-boundary geometry, incomplete frames, method mismatches and unfinished observations.

The following findings were raised with the authors:

1. **Unsupported zero before an interval ends.** A complete frame attestation could coexist with a contributing interval ending after the evidence cut, leaving the old point path at zero when the new bound path correctly remained unavailable. Both complete point and bound availability now require completed intersecting observations. The independent reproduction passes after the correction.
2. **Mathematical versus numerical enclosure.** Ordinary `f64` endpoints are not certified outward bounds. For a three-hour interval and stored allocation `0.1`, the computed point exceeds the exact rational product by `2^-55`. Each bound now declares `floating_point_approximation_without_outward_rounding`; the report, HTML, contract and manuscript distinguish numerical approximations from the real-arithmetic result.
3. **SRS cancellation at a nonbinary sampling fraction.** For `N=7`, `n=3`, zero labels and constant predictions `1e16`, the original compensated implementation returned -8 with estimated variance zero. The mathematical estimate is identically zero. This was reported separately from legitimate signed estimates. The centred-SRS correction uses the canonical expansion from the declared integer population and sample sizes, cancelling the common prediction offset before arithmetic. The independent reproduction now returns zero with zero estimated variance. General designs retain their supplied weights.
4. **Scope and wording.** The finite feasible set was clarified as finite-dimensional, the calendar domain explicitly contains all target windows, and the old broad statement that the application does not estimate design variance was scoped to task/cohort interval reports.

Task/cohort bounds retain the distinction between marginal sharpness and conservative aggregation under shared capacity. Missing, sampled, incomplete and unfinished observations do not acquire complete deterministic upper bounds. Known exact output and a strictly positive direct labour lower bound permit an additional conditional productivity interval, with separate units and the same floating-point qualification. This does not make the point ratio available or establish joint sharpness.

## Contribution and exposition audit

The principal contribution is a specified observation-to-target relationship and its direct derivation. The manuscript explicitly attributes standard difference estimation, partial-identification ideas and difference-constraint algorithms. It makes no first-ever, universal-theory, empirical-calibration or organisational productivity claim. Historical Lean evidence is kept separate from every new proof and implementation check.

The reading order is coherent: observational nonidentification, task endpoints, deterministic bounds, sampled estimation and prospective study. The compact marked-history subsection distinguishes pathwise compatibility from identification through a probability law and does not smuggle in Markov or independent-attempt assumptions. Acceptance, retention and fixed-horizon labour retain different clocks. Mechanism examples and the measured discrete query comparison are appendices, and the latter is explicitly not a benchmark of the sharp-bounds solver.

## Literature audit

The reviewer independently retrieved and read the relevant portions of Torgovitsky's journal PDF (§1), Foucart and Liao's author manuscript (§§1.1–1.2), Indyk's official MIT slides (6–11), and Angelopoulos et al.'s versioned manuscript (§1.4 and Appendix B). Their uses are appropriately bounded. In particular, PPI explicitly acknowledges the difference estimator and includes a finite-population extension.

Crossref independently confirms the Denis–Haddou authors, title, DOI, journal, volume 42, issue 3, pages 371–394 and publication date. Direct publisher HTML, PDF and XML requests initially returned HTTP 403. The available web-reading tool subsequently returned the publisher text; the reviewer independently inspected §2.3, equations (2)–(4), and its conditioning statement. These support the manuscript's fixed-prediction attribution. The different estimator in §3 is not imported. All five added source attributions were therefore checked in primary text.

This bounded audit does not establish exhaustive novelty or priority.

## Typesetting and protected evidence

The integrated PDF inspected at `/workspace/work/manuscript-check/main.pdf` had 29 pages. Page-scale inspection covered the title and opening, new bound proofs and cell constraints, graph proof, sampling-variance display, measurement table, measured-performance figure and historical proof-concordance table. These were readable without clipping or overlap. The inspected log had no undefined citation/reference or overfull-box warning; one underfull table-label box was harmless. After the final application-scope sentence and float changes, the stable deliverable was re-inspected on the affected graph/estimation pages and proof-concordance page. Final identity is recorded below.

The 83 historical package files were independently compared byte-for-byte with the base Git blobs. All seven measured source SHA-256 values were independently recomputed against the retained environment manifest. Both checks passed at review start; the final repeat is recorded below. No new Lean execution or new performance measurement is claimed.

## Final verification

- The 241-request external harness passed after the final centred-SRS change, with no remaining finding.
- Nine additional external productivity fixtures passed: positive and zero known output, zero lower denominator, unknown quality, sampled labour, no counted tasks, numerical overflow, and exact point-ratio preservation. Formula direction, unit-specific fields and availability conditions were also inspected directly.
- The final protected-file repeat matched all 83 historical Git blobs, all seven measured source hashes and the retained raw measurement CSV hash.
- Final deliverable: `paper/ai-assisted-software-engineering.pdf`, 29 pages, SHA-256 `91ac161ec6ba1505fff8e637bd6f4f767a1dcae87a9236be0597843bc75c8080`. Its affected pages were re-rendered and visually inspected; all pages were checked for unresolved `??` reference text. No clipping, overlap or unresolved reference was found in the inspected pages.

The final five audits—mathematics, contribution, exposition, literature and typesetting—have no unresolved acceptance blocker within the stated scope. The application and paper consistently distinguish exact mathematical sharpness, approximate local application bounds, design estimation, and empirical assumptions. Acceptance does not establish empirical adequacy, exhaustive novelty, numerical certification, a general application LP solver or new Lean coverage.

Review scripts and extracted primary sources were retained under `/workspace/work/independent-new-pr` during this session. They are reviewer-side checks, not additions to the distributed implementation. The repository contains the authors' executable regressions and exact rational checker. The integration owner runs the complete project gates separately; this report does not relabel those executions as independent reviewer runs.
