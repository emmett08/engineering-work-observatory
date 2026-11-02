# Current manuscript editorial concordance review — 2 November 2026

Outcome: **SEMANTIC_FIDELITY_PRESERVED / MATHEMATICAL_STATEMENTS_UNCHANGED / PASS**.

Independent reviewer agent: `/root/current_fidelity_review`. The reviewer authored none of the eight reviewed sources. I read their complete current text and complete changes against repository baseline `e0ded20cfe7f699863d2e134c904b9eb673b74b7`, including the connections between accounting, identification, sampling, validation, the proposed causal experiment and forecasting. No material semantic defect was found. This is an editorial and source-concordance review of those eight current sources, not an empirical validation.

## Exact reviewed sources

SHA-256 hashes identify the actual baseline and candidate bytes.

| Source | Baseline SHA-256 | Candidate SHA-256 |
|---|---|---|
| `paper/article/design-based-estimation.tex` | `cfad2ac27f70adc63b23c408ab89f09dc30c8cf6cc741d1d0626e3da2759af39` | `fb10092d7a72db49c8e592b132b421d2c15a06f532e73c9afc94fe3de00548e7` |
| `paper/article/event-history.tex` | `8c69d9aa82483e4d962cc1542459f8c43bf65e73da4d60655fef8d3319619b65` | `981e778bbc2131075484826af3b49ca3a6228e0aab24a49b9acc3faa389030ce` |
| `paper/article/main.tex` | `1df5662bbc1b6a2036c60141789bf7c95e6baa810079b9f04060fb1edb97622d` | `8224afe88b5e1ae5afcba834924739125f23f91301c8af8e0676a372210a50ba` |
| `paper/article/partial-identification.tex` | `f0f955912e9da22dd14187610ca290f99a837c184b3b25813ecd4f9cf233a927` | `fe3351e85225f2fee0ae704ed2ae96c09531b10287bf5e8c5dc45be7d8fd9d4a` |
| `paper/article/sampling-validation.tex` | `7fe4aea96194c5ecd85b1c063a87076fd09b7bab769c8bfe109539d613e56f2e` | `0d82c3098f4072241a029e57797a63b427b9e9a38ff1cb650eb4e2cb2cba36dd` |
| `paper/article/scalar-bounds.tex` | `4916565404f62e61d42605078f36659a0339de719eb1b85902c43d31d6d9c229` | `84afcfeaefd619ffd7036f564fdc5ea198b6e2b05766798c399344bfabec31c6` |
| `paper/forecasting/main.tex` | `5607d3ce065c4d810386f3b183f546e8f39efa38d0fc24785fdb66b761ce1609` | `224bad3a7d61e6cdf6194ef51e1f7ed520def8c1cf28a283551b850229102121` |
| `paper/study/ai-labour-causal/causal-study.tex` | `f6f8550e212356addeb1433875d008f0b78474270b42258dfc403d6be695ee1b` | `acc7e4387a36852cf7fef799b8e42d0c867cbc01bddbf3bd572e91291da64f1b` |

## Literal checks

Ordered extraction confirms byte-for-byte equality against the baseline for every category below. Counts are occurrences in each source; categories overlap where equations or commands occur inside complete propositions or proofs. Inline spans include their dollar delimiters; bracket displays include `\[` and `\]`; equation/align counts include starred environments and their complete contents. No `\(...\)` spans occur.

| Source | Inline math | Bracket displays | Equation/align | Complete propositions | Complete proofs |
|---|---:|---:|---:|---:|---:|
| `paper/article/design-based-estimation.tex` | 56 | 2 | 5 | 1 | 1 |
| `paper/article/event-history.tex` | 3 | 0 | 0 | 0 | 0 |
| `paper/article/main.tex` | 43 | 0 | 5 | 1 | 1 |
| `paper/article/partial-identification.tex` | 99 | 1 | 6 | 2 | 2 |
| `paper/article/sampling-validation.tex` | 15 | 1 | 1 | 0 | 0 |
| `paper/article/scalar-bounds.tex` | 36 | 0 | 1 | 1 | 1 |
| `paper/forecasting/main.tex` | 20 | 0 | 5 | 0 | 0 |
| `paper/study/ai-labour-causal/causal-study.tex` | 2 | 0 | 0 | 0 | 0 |
| **Total** | **274** | **4** | **23** | **5** | **5** |

| Source | Citation commands | Labels | Reference/eqref commands | Inclusion/bibliography commands |
|---|---:|---:|---:|---:|
| `paper/article/design-based-estimation.tex` | 3 | 9 | 4 | 0 |
| `paper/article/event-history.tex` | 0 | 1 | 1 | 0 |
| `paper/article/main.tex` | 9 | 12 | 9 | 8 |
| `paper/article/partial-identification.tex` | 6 | 11 | 10 | 2 |
| `paper/article/sampling-validation.tex` | 0 | 0 | 0 | 0 |
| `paper/article/scalar-bounds.tex` | 1 | 3 | 2 | 0 |
| `paper/forecasting/main.tex` | 7 | 9 | 8 | 5 |
| `paper/study/ai-labour-causal/causal-study.tex` | 5 | 1 | 0 | 0 |
| **Total** | **31** | **46** | **34** | **15** |

Citation checks retain command spelling, keys and optional locators. Inclusion/bibliography checks cover `\input`, `\include`, `\bibliography` and `\bibliographystyle`. These literal checks supplement review of the surrounding prose; mathematical meaning is not assumed to reside only in extracted spans.

## Semantic and citation review

The new explanatory connections follow the existing constraints and calculations. The capacity examples still distinguish separately valid bounds from jointly attainable endpoints. Cell reduction retains the complete calendar partition, observations outside target windows, nonempty-feasible-set condition and both directions of reconstruction. One-activity schedules still require arbitrary starts, stops and changes of task or work type. The certificate explanation retains nonnegative multipliers and sign reversal of lower observation bounds; its new fixed-partition and fixed-target qualification correctly limits the right-hand-side perturbation argument. Simultaneous coverage, empty-set failure, positive labour denominators and joint output–labour feasibility remain explicit.

Task identity, attribution rules, separate AI access/use records, person-time capacity, abandonment versus censoring, reference-score calibration and the three observation clocks retain their meanings. The difference estimator still requires fixed predictions, known positive inclusion probabilities, complete target coverage and accurate sampled allocations. The revised account of residuals, zero observations and nonresponse preserves these conditions. The reverse-ranking example explains its existing variance calculation. The validation examples still distinguish prediction error against true labour from disagreement with a noisy estimate; the signed, squared-error, absolute-error and coverage claims keep their separate qualifications.

The proposed causal experiment retains paired allocation, the equal-team target, interference concerns, both benefit criteria, provisional margins and bias allowances, the inadequate hypothetical sample-size scenarios and the need for a measurement pilot. It remains specified and unrun. The forecasting revision preserves its merge-count population, complete-period and cutoff rules, distinction between reconstruction and prospective retention, absence of independently certified chronology, empirical-distribution assumptions and absence of guaranteed future coverage. Its negative comparison still leaves the default's full-history accuracy and performance in the user repositories unestablished. Queue counts, workflow outcomes and implementation checks do not become evidence of product correctness, causal effects, predictive accuracy or bottlenecks.

All 31 citation commands retain their baseline keys and locators, and their surrounding statements preserve the same attribution and qualification scope. No explanatory addition requires a new empirical result: the additions unpack the existing accounting rules, constraints or calculations. This review checks citation scope in the supplied sources; it does not freshly verify the cited literature. The article date changes from 1 to 2 November 2026. No new empirical or causal result is asserted, and no human comprehension or readability outcome was measured.

## Frozen target and retained formal boundary

The frozen `paper/formal/targets/partial-identification.tex` has SHA-256 `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb`. The candidate matches it exactly for all 99 inline mathematical spans, one bracket display, six equation/align environments, two complete propositions, 11 labels, 10 reference commands and two inclusion commands. Both complete proofs match the current baseline exactly. Against the frozen target, the only proof-text difference is the pre-existing replacement of “zero; leave” by “zero. Leave” in the serial schedule construction; its allocation and idle-time instructions are unchanged. The frozen target has three citation commands, while baseline and candidate both have six. The added purification and Lean/mathlib attributions, schedule-panel explanation, formal-coverage paragraph and earlier editorial rewording already exist in the baseline; this revision does not introduce them.

The formal-coverage paragraph is unchanged. Its retained scope is interval/cell integral identities, integrated capacity, binary schedule reconstruction preserving observations and weighted targets, supplied primal–dual certificates and conditional transfer of simultaneous observation coverage to target coverage. It continues to exclude extrema existence, dual-multiplier existence, the numerical block-count bound and Rust source refinement. The analytical arguments retain those distinctions. The immutable target, Lean development and historical proof evidence were not modified. This review performs no new Lean execution, kernel replay, implementation test run, measurement study or empirical validation.

This outcome binds only the candidate bytes above. Later source edits require review of the changed content and renewal of the applicable hash bindings. Existing checks JSON, manuscript bindings and formal receipt were not refreshed by this reviewer.
