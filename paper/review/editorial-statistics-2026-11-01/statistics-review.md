# Engineering-statistics review of the AI labour paper

Baseline: `5936f2c28fd075217b679c0b2ad3aba57b9801f3`. Read actual `main.tex` include tree: event-history, partial-identification, design-based-estimation, sampling-validation, causal-study and scalar-bounds; also the Figure 1 caption, causal protocol/README/planning program, and Rust inference/frame/variance implementation. `performance.tex` and `binning-bounds.tex` are not compiled into this article and are outside this reading scope.

Reader: experienced software engineer able to follow mathematical arguments, without assumed sampling-design expertise. This review applies `analyse-software-engineering-statistics` and its design notes. No workplace observations were available or analysed.

## Result

The finite-population estimator, SRS variance and variance-estimator formula, sampled-validation identities, simultaneous-coverage implication and paired equal-team causal expectation are sound under their stated assumptions. No statistical equation or estimator implementation change is required. One consequential interpretation in the linked protocol should be corrected; several conditions should be easier to recover in the article.

| Priority/location | Finding and risk | Suggested disposition |
| --- | --- | --- |
| Substantive, `protocol.json:294,298,305`, DM-003 | An interval excluding a worthwhile labour saving is called a “practically small labour effect”. An interval entirely within [3,5] person-hours/reference-unit excludes savings beyond one hour while indicating a substantial increase. Excluding benefit does not establish equivalence or practical smallness. | Rename disposition/rationale “Exclude a worthwhile labour saving; report any increase or unresolved harm”. Replace observation region with “The expanded labour lower bound is at least -1; report the retained-output result separately”. If preserving the original quality-preserved branch, retain that explicit condition. State that practical equivalence requires its own declared region and interval wholly within it. |
| Clarification, `README.md:135–137` | “practically small effects” is not linked to an equivalence region. The protocol has improvement and noninferiority boundaries, no two-sided labour-equivalence margin. | “Report excluded worthwhile savings, evidence of increased labour, and unresolved intervals separately. Do not infer practically negligible effects without a prespecified equivalence criterion.” |
| Clarification, `causal-study.tex:6–8` | The article specifies the conjunction of effects but not the implemented decision procedure: approximate pair-t bounds expanded by provisional measurement-bias allowances, and missing endpoints block every affirmative claim. These materially affect the interpretation of “joint success” and the planning result. | Add that benefit requires both sensitivity-expanded one-sided bounds to cross their boundaries, these are approximate t procedures, and any unresolved team endpoint prevents a benefit claim under this plan. Keep error allowances provisional and unvalidated. |
| Clarity, `causal-study.tex:8` | “Twenty thousand simulated experiments per design and scenario” can leave unclear which quantities are assumptions and whether nonqualification concerns detection or false-positive calibration. | Name the hypothetical outcome, error and missingness scenarios; state neither tested team count reaches the required probability of meeting both decision criteria across the required useful-effect scenarios. Do not turn this into a universal impossibility claim. |
| Interpretation, `main.tex:162` | Says measurement must be assessed “before a causal study is designed”, although the paper already includes a specified design and simulated planning. | Distinguish the existing provisional design from final sample-size and execution decisions: measurement pilot informs a later principal study. |
| Scope, `planning.py:63–89`, README:196–202 | Simulated variation is generated at team level. Twenty initial tasks determines the score denominator and budget but there is no separately fitted/task-level outcome model or power evidence for changing the number of tasks. | Retain the fixed twenty-task workload and hypothetical team-level distribution; do not infer adequacy under alternative task counts. A short audit disclosure suffices; no numerical change is warranted. |

## Measurement and inference boundaries retained

| Object | Unit/denominator | Conditions and permitted conclusion |
| --- | --- | --- |
| Interval target labour | Person-hours integrated over stated calendar windows | Exact-data identification conditional on true observation bounds, attributable streams and per-person capacity; not a sampling interval by itself. |
| Difference estimator | Person-hours over the complete disjoint person-interval frame | Accurate measured selected allocations, positive known inclusion probabilities and predictions invariant to sample selection give design-unbiasedness; no prediction accuracy or lower-variance guarantee. |
| SRS uncertainty | Person-hours squared; fixed frame size M/sample size n | n >= 2 for the sample-variance estimate, zero sampling variance for a census; errors in measurements remain distinct. |
| Prediction validation | Signed error and squared-error comparisons | Sampling variance inflates squared agreement error; shared unbiased validation cancels it in predictor differences. Absolute error and coverage against noisy estimates are different targets. |
| Causal labour/retention effects | Equal-team mean potential-outcome differences, each normalised by preallocation reference-score total | Independent pair coins, consistency, no material between-cluster interference and valid measurement. All failures stay in original groups; no completers-only denominator or use-selected target. |
| Principal-study decision | Labour saving > 1 hour/reference-unit AND retained-output loss < .05 | Approximate one-sided t bounds with stated sensitivity expansion and completeness override; neither tested team allocation meets the declared joint-success criterion. |
| Simulated calibration | 20,000 replications per design–scenario cell | Conditional on hypothetical distributions and error/missingness assumptions. Wilson intervals quantify Monte Carlo error, not workplace uncertainty or a simultaneous universal calibration guarantee. |

## Checks actually made

- Reviewed the exact unbiasedness algebra and general inclusion-indicator variance.
- Reviewed the pairwise expression for unbiased sample variance and its n=1/census limits.
- Matched manuscript formula to Rust `general_totals`, offset-cancelled `srs_totals`, `variance::calculate` and frame validation. No behavioural code defect found in this scope.
- Reviewed randomisation sign and fixed additive effect in `planning.py`, component-null union/conjunction logic, sensitivity allowances, completeness override and per-scenario calibration gates.
- Constructed the analytic DM-003 counterexample: [3,5] excludes all savings below -1 but is not a small labour effect.
- Ran the bounded existing protocol validator: PASS for structural/reference/numeric consistency, retaining six adequacy failures and specified/unrun status. This validator cannot detect the interpretation error above.
- Prior-study descriptions remain outcome/population-specific and no effect estimates or variance inputs from those studies are transported into the planning model.

No new empirical, causal-identification, calibration or power claim is justified by editorial revision. Preserve all existing uncertainty and caveats; list the DM-003 repair separately from faithful wording changes.
