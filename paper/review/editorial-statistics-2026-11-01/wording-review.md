# Whole-paper wording audit

Baseline: `5936f2c28fd075217b679c0b2ad3aba57b9801f3` (merged main). Reviewer: `/root/wording_audit`. Mode: contextual audit, without source edits. Applied `audit-engineering-wording` and read its complete precision catalogue and contrast/negation reference. The skill's advertised `scripts/scan_wording.py` resource could not be read through the skill provider. No execution of that scanner is claimed. Candidate generation used `rg` and a small literal concordance, followed by complete contextual reading.

Scope read in publication order: `paper/article/main.tex`, including `event-history.tex`, `partial-identification.tex` and its Figure 1 caption, `design-based-estimation.tex`, `sampling-validation.tex`, `../study/ai-labour-causal/causal-study.tex`, and `scalar-bounds.tex`. Unused `binning-bounds.tex` and `performance.tex` are outside the compiled manuscript. Reader: experienced engineer comfortable with mathematics, but not assumed to know the project's protocol labels. Protected: displayed and inline mathematics, notation, proposition claims, conditions, bibliography keys, citations and references.

## Material repairs

Line locations below refer to the immutable baseline. Suggestions are wording recommendations unless marked as a scope clarification requiring statistical/mathematical review.

| Location | Source wording | Referent, defect and consequence | Suggested repair and fidelity check |
| --- | --- | --- | --- |
| causal-study.tex:8 | `component-null boundaries` | The labour-saving and retained-output decision thresholds. The label requires the reader to reconstruct the two component hypotheses from an external protocol. | State the actual boundaries, e.g. cases where labour saving is exactly one person-hour per reference unit or retained-output loss is exactly five percentage points. Confirm exact direction and scenario coverage against protocol. Preserve the distinction between these null cases and useful-effect planning cases. |
| causal-study.tex:8 | `Neither allocation ... before choosing a principal allocation` | Counts of teams in the candidate main studies, rather than a randomised allocation or a person's time allocation. The paper otherwise uses allocation for those different operations. | `Neither team count ... before choosing the principal-study size`. Preserve 48/96 teams and do not imply either size is currently selected. |
| causal-study.tex:8 | `provisional joint-success target` | Probability of establishing both the labour and retained-output claims under specified useful-effect scenarios. Neither the probability criterion nor the scenarios required to pass are locally supplied. | State the provisional probability criterion from the protocol, and make clear that it applies to the required useful-effect cases, not null scenarios in which success should be rare. This needs the statistics audit's exact rule. |
| main.tex:162 | `before a causal study is designed. That study must also choose whether to compare a fixed workload or organisation-wide adoption` | The current project's status. The immediately preceding subsection already specifies a provisional fixed-workload experiment. Future design language makes that specification appear absent or undecided. | Say the pilot informs the final measurement procedures and principal-study size. Explain organisation-wide adoption as a separate study question, if it remains in the conclusion. Preserve that no workplace trial has been run and the current protocol is provisional. |
| partial-identification.tex:108 | `binary histories preserving observations and weighted targets` | One-activity-at-a-time measurable allocations whose interval integrals and weighted cell sums match the supplied array. The compressed phrase hides what is binary. | `work schedules in which each person does at most one activity at a time while preserving the observation totals and weighted targets`. Keep the proof's almost-everywhere convention and scope. |
| partial-identification.tex:108 | `supplied primal--dual certificates` | Given feasible allocations and nonnegative multipliers, checked by inequalities and matching endpoint values. The named formal object is correct, but only locally described earlier. | `supplied allocations and dual multipliers that prove the claimed bounds` (or retain `primal--dual certificates` after this description). Do not imply the supplement proves such multipliers exist. |
| partial-identification.tex:108 | `conditional sampling-coverage implication` | On simultaneous observation coverage and a true history satisfying capacity, the target interval covers at least as often. The label makes the reader repeat the previous argument. | Describe the implication directly: simultaneous coverage of observation totals yields at least the same coverage for the target total, subject to the stated allocation assumptions. Preserve the explicit exclusions immediately following. |
| event-history.tex:4 | `a time allocation satisfies the record` | An allocation consistent with the observations contained in a task-history record. A record is an artefact, rather than a mathematical constraint. | `an allocation is consistent with the recorded observations and the model's constraints`. Preserve the existence condition and the all-compatible-allocations definition of point identification. |
| scalar-bounds.tex:8 | `The scalar observations ... When it is feasible` | Observations for one allocation function, followed by existence of a compatible allocation. `it` can denote the graph or the observation model; `scalar` depends on specialist shorthand. | `The observations for this single allocation function admit an allocation ... When such an allocation exists`. Preserve the exact if-and-only-if negative-cycle criterion. |
| main.tex:107 | `how fractional scores are calculated` | Fraction of the accepted requirement still met, which determines the retained score. It is potentially unclear whether the rule applies to the baseline score or retention fraction. | `how reviewers determine the fraction of an accepted requirement that remains satisfied`. Keep the distinction between baseline score and retained fraction and do not invent an adjudication process. |
| partial-identification.tex:42 | `a set ... of attributed streams` | The person/task/work-type triples whose labour the observation includes. `attributed` is a second inference step for the reader. | `a set ... of streams whose labour is included in that observation`. Preserve the exact mathematical set, units and grouped-observation relation. |

## Additional scope/meaning checks sent to statistics review

The abstract and introduction say that minimising and maximising produce exact bounds and all intermediate totals without locally repeating compatibility/nonemptiness. The theorem states nonemptiness explicitly. A brief `When the observations admit an allocation` would prevent readers from inferring extrema for inconsistent input. This is a faithful restoration of the theorem's condition, not a new result. Do not add an invented data-cleaning or feasibility procedure.

The introductory count of task-specific totals and person-interval sample weights is otherwise consistent. No wording edit should replace the set of possible totals with a history attaining one total, or turn observation bounds into an empirical measurement claim. Retain the exact covariance and sampling statements pending the statistical audit.

## Canonical vocabulary and sense-by-sense concordance

- **Allocation**: a fraction of a person's time, the finite cell-total array, and a randomised comparison-group allocation are all legitimate separate operations, explicitly signalled by local objects. Retain those uses. The two study-size uses in causal-study:8 should become `team count`/`study size`; neither denotes allocation in either formal sense.
- **Bound/range**: observation lower/upper limits, extrema of feasible target totals, prediction/variance quantities and mathematical set ranges are distinct. The manuscript defines the identified range as all attainable totals, and the interval theorem supplies attainability. Retain the technical vocabulary. Do not rotate `range`, `interval`, `set` and `allocation` as stylistic synonyms.
- **Reference**: baseline task scores and retained output are introduced by `S_i^0` and used with the same score scale. SCH as a reference index is explicitly distinguished from measured person-hours. `reference checker` in the appendix denotes a software implementation, a separate conventional sense; `exact-arithmetic checker` is an optional clearer name, not a required mathematical repair.
- **Fixed/unchanged**: predictions remain unchanged across later choices of validation sample; task requirements and reference scores remain the same across the study groups; sample size is fixed under simple random sampling; retention observations stay fixed while labour allocations are optimised. These senses have stated objects/comparisons and should be retained. The abstract's shorter summary relies on the fuller definitions later, as is normal for this genre.
- **Record**: task/requirement histories, observed allocation values and missing/selected sampling statuses are specified locally. Retain these literal uses. Repair the predicate `allocation satisfies the record` to name consistency with its observations.
- **Type**: every reader-facing occurrence means a type of work (implementation/review/testing/etc.), not organisational job roles. Retain. The set indexed by `r` and `R` is introduced before use.
- **Stream**: explicitly defined as one person's work of one type on one task. The later uses denote this same triple; retain. `attributed streams` benefits from the direct grouped-observation description above.
- **Capacity**: the one-person time constraint is stated as at most one person-hour per hour; computing capacity in the causal interference paragraph is explicitly a different resource. Retain both local senses.
- **Prediction/residual**: predictions are in person-hours after any SCH conversion; residuals are observed minus predicted labour. Retain the standard terms and their formulas. `total residual` in the conclusion may optionally become `sum of observed-minus-predicted labour` for readers entering at that paragraph.
- **Sampling design**: the probability law selecting frame units, with first- and second-order inclusion probabilities. The exact variance, estimator and coverage qualifications use it literally; retain.
- **Joint**: joint commit distribution, jointly feasible allocations, joint inclusion information and joint success of two study claims are contextually separate senses. The first three have clear objects. `joint-success target` needs the explicit two claims and probability rule.
- **Independent**: Bernoulli independence, prior independent training population, independent checking of task records, fair coins within pairs and independent finite checker enumeration each state a different intended relation. Retain and do not convert organisational separation into stochastic independence.
- **Framework**: only the named SPACE framework, attributed to its source. Retain proper name; this is not an unsupported generic framework claim.
- **Index/label/representation**: SCH common reference index, minute-indexed time, indexed endpoints, inferred coding-state labels, and the appendix's difference-constraints graph representation are literal, locally recoverable operations. Retain the exact domain terms. No global substitutions are warranted.
- **Evidence**: measurement evidence, evidence used by independent record checkers, and evidence from collection refer to observations needed to justify coverage/measurement assumptions. Their absence is acknowledged. Do not invent a recording instrument or evidence type to replace the general requirement.
- The user-nominated families **eligible, assigned, intervention, overhead, frozen, assessment, role** have no reader-facing occurrences in this compiled manuscript. **Representation** occurs only in the established graph-theoretic phrase above. No reintroduction is needed.
- The other nominated families **contract, distinction, bridge, grain, marker, class, smuggle, control, policy** have no reader-facing matches. LaTeX `label` commands and bibliography paths are syntax, not prose findings.

## First-use and predicate checks

All mathematical symbol families are introduced in the reader's order: labour allocations and task age; acceptance time and baseline scores; acceptance/follow-up deadlines; continuous-time stream/domain sets; grouped observations; cells/totals; target weights and feasible set; dual multipliers; finite sampling frame, inclusion probabilities, predictions and residuals; variance quantities; and appendix cumulative totals/graph distances. Reuse of `M`, `r` and `pi` is local and typed; no notation redesign is indicated. The figure independently names its task totals and midpoint `P`, rather than relying on project codes. No opaque internal assumption/theorem/stage codes appear in the finished prose. Standard SCH and HT abbreviations are expanded, while Lean/mathlib and SPACE are named and cited.

All three uses of **conversely** reverse the actual existence/integral or graph-feasibility relations in their proofs. Retain them. The essential negatives concern observational nonidentification, double counting, one-person capacity, differences between clocks, lack of calibration/measurement validation, impossibility of inferring quality equivalence from nonrejection, one-unit variance estimation, sample-dependent predictor counterexamples, signed estimators, zero denominators, empty feasible sets, and limitations of the formal supplement. These negatives carry mathematical or empirical content. No zero-negation target is appropriate.

The abstract/introduction, detailed definitions/theorems and conclusion repeat several propositions for distinct orientation/proof/synthesis purposes. Do not remove those functions mechanically. The main opportunity for consolidation is the conclusion's future-study paragraph, whose current status conflicts with the preceding specified protocol. The other suggested edits are local referent and relation repairs, not a rationale for changing proofs or empirical claims.

No source files edited by this reviewer. A final independent reread of the revised source remains to be completed after the author's edits.

## Fresh candidate pass

Read all seven current manuscript units and the unchanged caption again in publication order, without using the initial finding list as a substitute for reading. The rewritten explanation now exposes the sample correction, clock differences, grouped observations, schedule construction and study-size/decision rules. Necessary technical terminology and negatives remain justified. All three converse arguments remain valid. The conclusion now correctly points to the specified pilot and provisional fixed-workload experiment.

Two residual edits sent to the author: (1) replace the ambiguous `its source code` in the formal-scope paragraph with `the Rust solver's source code`; (2) make compatibility/nonempty observations explicit before the abstract/introduction promise attained extrema. The latter restores a condition already present in the unchanged cell-reduction theorem. No other material wording or sentence/paragraph continuity defect was found in this pass. No source edits were made by this reviewer. This is an agent editorial assessment, not human comprehension evidence.

Exact candidate source hashes at this pass:

- `paper/article/main.tex`: `1373d7b620a2dee39abfbf68a3d8fc07832af74ae8df74e4e40fb628eb962b98`
- `paper/article/event-history.tex`: `9f4cdc267c248e517df40658744241141f2cdba7029f9c46bcd62ffa80a3ec33`
- `paper/article/partial-identification.tex`: `53693b6b56bc272d509c21e01c0c268fe16068d66f3eb3ceeba4a75ef731a507`
- `paper/article/design-based-estimation.tex`: `89756222dd144a9ac9fd5317f23216acf67311de346ad87870ffb3b49f14aeb0`
- `paper/article/sampling-validation.tex`: `50331455dd4027b685252eb71cfec11dac177bdd1b5a5b97618311b15aabb694`
- `paper/study/ai-labour-causal/causal-study.tex`: `9871ecbca23b95b6b2a4b17c2f07fab4915d544bb58ac9fefde667a1853184e8`
- `paper/article/scalar-bounds.tex`: `0a653a5f2906630fb29d81650dfd785d45ebd2c49b23055d14a46d47e96f747f`
- `paper/figures/shared-capacity/caption.tex`: `f5bb12df4d2aa8cd6b8441f03cf3f4716f8509fe74794a288c2c11e90de966fd`

## Final closure

Both residual repairs are closed. I verified the exact final hashes, reread the abstract and introduction with their neighbouring paragraphs, and reread the formal-scope paragraph with the coverage and productivity paragraphs around it. The abstract now conditions attained minima, maxima and intermediate values on the observations admitting an allocation. The introduction explicitly requires compatibility with each person's available time. These restore the unchanged theorem's nonemptiness condition without inventing an observation procedure. The final scope sentence names the Rust solver's source code, removing the competing antecedent.

The other five manuscript units and unchanged Figure 1 caption match the hashes reviewed in the fresh full pass. No material wording or continuity finding remains from these passes. This conclusion is a fallible agent editorial assessment, not a mathematical equivalence certificate or human reader study. No source edits were made by this reviewer.

Final reviewed source hashes:

- `paper/article/main.tex`: `b6060ee82d09d45dc60cb6b02c73dd8346f0a97f2f6a2084f6a40691b02b06b3`
- `paper/article/event-history.tex`: `9f4cdc267c248e517df40658744241141f2cdba7029f9c46bcd62ffa80a3ec33`
- `paper/article/partial-identification.tex`: `574287fea71e64e8078b6593d15ce3e004ce502c4c2ed59700a3d28b465682c6`
- `paper/article/design-based-estimation.tex`: `89756222dd144a9ac9fd5317f23216acf67311de346ad87870ffb3b49f14aeb0`
- `paper/article/sampling-validation.tex`: `50331455dd4027b685252eb71cfec11dac177bdd1b5a5b97618311b15aabb694`
- `paper/study/ai-labour-causal/causal-study.tex`: `9871ecbca23b95b6b2a4b17c2f07fab4915d544bb58ac9fefde667a1853184e8`
- `paper/article/scalar-bounds.tex`: `0a653a5f2906630fb29d81650dfd785d45ebd2c49b23055d14a46d47e96f747f`
- `paper/figures/shared-capacity/caption.tex`: `f5bb12df4d2aa8cd6b8441f03cf3f4716f8509fe74794a288c2c11e90de966fd`
