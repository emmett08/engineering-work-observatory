# Independent scientific/statistical review

Final review: 2026-11-01. Reviewed the causal-study README, planning.py, retained JSON/CSV, causal-study.tex, references.bib and protocol.json, including the targeted corrections following the first review. No study-author files were edited. This review addresses a specified, unrun study; it is not empirical validation.

## Final disposition

No outstanding material issue was found in the specified one-team-per-cluster estimand, the implemented co-primary/missingness decision, or the targeted corrections. The package supports its stated conclusion: neither 48 nor 96 teams is established as an adequate principal allocation. A separate measurement/feasibility pilot and supported nuisance ranges are needed before a confirmatory allocation is selected. The six-pair pilot cannot provide precise variance or rare-loss estimates and must not supply a favourable treatment effect for sample-size selection.

Final verification:

- `planning.py --check` passed: all **24 design/scenario cells**, each with **20,000 simulated experiments**, reproduce exactly in the current environment.
- `validate_protocol.py` passed its expressly bounded local structural/reference/numeric checks: 12 scenarios, 24 simulation cells and **86 assessment results**, with all six required joint-success design/scenario failures retained. This is not a scientific, empirical or bundled-skill validation claim.
- No candidate is selected. No participant recruitment, data collection or human study was executed by this review.

## Targeted corrections confirmed

1. **Assignment clusters and estimand weights.** README and protocol now explicitly restrict this version to one team per assignment cluster. If interference mapping requires clusters containing multiple teams, the design stops for revision of pairing, estimator weights, randomisation/analysis units, independent-unit counts and planning. This resolves the former possibility of silently changing an equal-team estimand into an equal-cluster estimand.

2. **Severe defects and the bounded primary claim.** Protocol DM-002 now concerns retained-output harm established beyond the margin. A separate DM-006 records severe security/reliability defects and bars interpretation of co-primary success as overall safety or an adoption decision. README and analysis explicitly state that severe-defect incidence is unmodelled. This resolves the former overlap between severe-defect failure and co-primary success without implying rare-event power has been assessed.

3. **Maximum-bias component calibration.** Two partial-null scenarios add maximum favourable allowed bias to the true-null component. The code now records each true-null component's rejection probability and checks its upper Monte Carlo bound against the 3.5% tolerance, alongside joint rejection. The protocol contains T-FALSE-L and T-FALSE-Q assessment rows. The largest observed component rejection remains 2.760%; it is transparently reported as approximate calibration, not exact 2.5% size. Intersection-union control remains conditional on component validity.

4. **Sensitivity-expanded intervals.** README, protocol and planning metadata now identify reported coverage as coverage of t intervals expanded by the provisional measurement-bias allowances. They do not claim raw t coverage or universal finite-sample guarantees.

5. **Missing endpoints and reportable intervals.** Incomplete simulated records now produce no component rejection and the full parameter-domain confidence set. Unconditional coverage properly includes those uninformative sets; finite half-width and bias summaries are restricted and labelled as conditional on complete simulated records. Latent generated missing values are not used to report a finite interval unavailable to the actual analysis. This is distinct from redefining the estimand or performing an available-pair analysis.

6. **Shared time and unresolved totals.** The ledger instructions now require capacity to hold across all tasks and teams, shared review/platform time to be recorded once and allocated by a predeclared rule, and ambiguous attribution to remain an interval or unresolved record. Sampled or ambiguous totals are not silently treated as exact complete observations.

## Scientific features checked

For a fixed pair A,B, averaging its two possible signed observed contrasts gives `[(Y_A(1)-Y_A(0))+(Y_B(1)-Y_B(0))]/2`. Independent fair pair coins therefore identify the stated equal-team finite-sample average under consistency and no material between-cluster interference. Exact point-estimator unbiasedness does not imply an exact Student-t interval; that distinction is retained.

Initial reference-score denominators are fixed before assignment. Failure, abandonment and nonacceptance remain in the workload and labour frame. Analysis follows assigned workflow; actual uptake is diagnostic. The primary claim requires both worthwhile labour superiority and retained-output noninferiority. Non-significant quality loss alone cannot establish preservation, and no independence between endpoint tests is required for the intersection-union argument.

Quality is assessed at acceptance age plus Delta for acceptance by tau; labour is accumulated through the common task age tau+Delta. With tau=Delta=14 days, required quality checks fall within the 28-day task observation period. These clocks agree with the paper's accounting definitions. Repairs triggered by observation are included only when within the declared labour window.

The simulation uses actual within-pair signs shared across outcomes, correlated team baselines, positive latent labour, bounded true retained fractions, measurement noise and contrast bias. The actual code implements the described primary decision. Wilson intervals remain informative when no simulated rejections occur, and simulation uncertainty is small relative to the principal joint-success shortfall.

## Remaining limits carried forward correctly

This is a team-level planning model with constant additive treatment effects. Task count enters the cost calculation, rather than an explicit task-level outcome variance model. It does not establish behaviour under treatment-effect heterogeneity, altered workload size, severe-failure mixtures, arbitrary tails, differential measurement reactivity or outcome-dependent observation loss. The missingness example is a sensitivity scenario, not an empirical loss model. Expanded intervals rely on defensible differential-bias bounds that have not been established.

The six-pair pilot is a bounded procedural and measurement exercise. Its nuisance uncertainty must be carried into replanning, together with externally justified useful effects and actual recruitment/observation budgets. The numerical collection/review budgets are provisional accounting assumptions and exclude unknown development labour. Sources are relevant to matching, interference, noninferiority, simulation reporting and pilot-effect-size selection; no external developer effect estimate is treated as transferable workplace evidence.

The final package consistently retains specified/unrun status, provisional margins, unestablished principal adequacy and no overall safety/adoption inference. These limits are scientifically consequential and should remain visible in any abridged manuscript account.
