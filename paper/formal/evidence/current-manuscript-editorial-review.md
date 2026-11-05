# Current manuscript editorial concordance review — 5 November 2026

Outcome: **SEMANTIC_FIDELITY_PRESERVED / MATHEMATICAL_STATEMENTS_UNCHANGED / PASS**.

Independent reviewer `/root/independent_fidelity` compared the nine labour text sources below against the published PR #18 checkpoint `a5487d11463cfe24f2e0a2b03b4ffd0728871b6f`. The review covers the source meaning, mathematical conditions, explanation order and every remaining use of record-family, observation and outcome wording in the stated paper/performance/figure scope. It does not claim measured reader comprehension, a fresh literature audit or a new Lean proof replay.

## Reviewed source bytes

| Source | Baseline SHA-256 | Candidate SHA-256 |
|---|---|---|
| paper/article/binning-bounds.tex | 714d75696b4a12439cf9ce39ee4084d3b9fb919196aa2b72a6d385d115a348dd | 714d75696b4a12439cf9ce39ee4084d3b9fb919196aa2b72a6d385d115a348dd |
| paper/article/design-based-estimation.tex | 240cb42f0fff2dea71cda3068d74e00b9e218e151c810fe181aa33e12f706c49 | 3993b40a586eeeb11129204711f5173b01c57de724da8183a1260ff0b4c3bbd3 |
| paper/article/event-history.tex | d80bb0ca30ef79c6ee73decbdbbc78c304f5a07a9825077721b2e6175fb6b73f | 8055b16642d328c79fd517adae746dd5e9d8893c1d3545e07bd0563cc66405b3 |
| paper/article/main.tex | da39a3ca09111519beeae08db7ad361f8b8a4a547713043c61a157b6c5b3f8b3 | 8b5269db48dd8dca5329e1d56abb751259690fcbef635bbc6f63222ef4f43d93 |
| paper/article/partial-identification.tex | c76e5998db92e2f9f888a616cc02883ba87a8c3bb2ceabd832722187f6cc1d29 | b4cfe7045327fd49ff7a083195fe4d614a6df216670f45bd7d76e70f0e79b27b |
| paper/article/performance.tex | 4d682ea8e04d91f5d601683d7f1f5146f961c37a9d2daa421b345c90e3520abc | e87f5a22d99e443f2c529d3a527989d105355f7bded9537bc4402a95fbc44b3a |
| paper/article/sampling-validation.tex | f0ab516fbe74751a941b28104966f60654cff8af95b6d5ac12c9d3cae9b5ea52 | 556fe98975bd157ca202b633b46854ad00aa473f8751d0f3991d5051328c3142 |
| paper/article/scalar-bounds.tex | 7dca65ce92f708fe3ddb53a9fcc852db6d4a05fc6e5d2318accdb3e712373303 | 69a06460f73d7327b831a01d586e8543c097293725f9af9452ac457d6a76feb8 |
| paper/study/ai-labour-causal/causal-study.tex | 566771f696b27817534ce1ba8e6daa46805d29f71ff6a7dd7bdc428621b33a7d | fa074efbd80dd038873ab64c842039e54b5f44d5a4276256bffcbad792dd7b0b |

## Meaning and progression

The revision identifies the engineering objects before explaining how the calculations use them. A task is a requested change with acceptance criteria. The GitHub issue example distinguishes that task from its commits and pull requests, its requirement version at the tracking start, later scope changes and study eligibility. Work diaries provide person/time measurements; acceptance confirmations give event times; later assessment notes support the fraction of the accepted requirement still satisfied. The reference score and output score remain distinct.

The issue, diary, release-log, scoring-entry and assessment-note formats are explicitly examples rather than claims about collected workplace data or mandatory new systems. The equal-weight acceptance-criteria example illustrates one possible rule for assessing partial fulfilment. The mathematical definition still permits a study-specific rule.

Independent review prompted five repairs. The pilot mean treatment-effect estimate is distinguished from variation used in sample-size planning. Actual tool use is described using diaries or logs separately from the assignment-based comparison. Request statuses preserve whether a request enters the included task set. Acceptance and deployment retain distinct timestamps. First acceptance times apply to accepted tasks, while other task statuses remain explicit. No material semantic regression remains in this bounded review.

The output score still equals the reference score multiplied by the fraction of the accepted requirement satisfied at the assessment delay after first acceptance. A missed acceptance deadline still gives zero. Labour remains the sum of person-hours through the common task-age limit. Shared human capacity is enforced across tasks and work types. Probability sampling still needs a covering frame, known positive selection probabilities, accurate allocation measurements and predictions unchanged across possible samples. Workplace measurement checks explain application of the results rather than adding a sampling-frame hypothesis to every theorem.

The causal experiment remains proposed and unrun. Its assignment-based comparison, equal team weights, deadlines, effect thresholds, bias allowances and adverse hypothetical planning results retain their mathematical meaning. None of the new engineering examples asserts an empirical result. The required hypophora remains, with each answer developing its question before the next topic.

## Per-occurrence wording scope

Whole-word record/records/recorded/recording occurrences fall from 46 to 2 in the 13-file compiled source closure and from 74 to 10 in the extended paper/performance/figure source scope. The two compiled uses explicitly concern actual tool use and selection status for a person-interval, each followed by an illustrative format. Eight performance uses refer to defined age/weight input pairs or preserve formal algorithm/proof wording. Counts describe coverage and do not establish readability.

The retained term interval observation is defined through its time interval, included people/tasks/work types and supplied total or bounds. HMM observations denote commit events conditioned on hidden activity states. Literature outcomes are the named study measures, and team outcomes are immediately defined as the two normalised labour/output quantities. Benchmark observations denote measured execution durations, while generated numeric inputs and hypothetical schedules remain identified as such. The review accepts these terms where their referents and role are explicit.

## Mathematical and reference checks

Every existing mathematical expression remains in source order. Performance prose adds only tau for the acceptance deadline and v_j for the labour contribution weight. Every proposition, proof, displayed-equation and algorithm block is byte-identical to the published checkpoint. The frozen target, Lean sources and checker scripts are unchanged.

The actual compiled manuscript closure has 13 source files, 40 labels and 35 reference commands, with no missing or duplicate labels. All 26 citation keys across the nine text sources resolve in the article bibliography. The unused binning source retains its pre-existing reference to app:coverage outside the compiled closure; the clean cross-reference claim applies to the compiled manuscript.

## Proof and evidence boundary

The current manuscript binding is renewed separately from the frozen proof target. The formal receipt preserves its historical source snapshot, proof executions, axiom declarations and exclusions. Only its current-manuscript hash is refreshed. The accompanying checks contain the static validators and isolated changed-byte guard actually run during this review. The primary agent manages PDF builds and visual inspection; those are not asserted as this reviewer's own observations.
