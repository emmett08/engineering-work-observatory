# Current manuscript editorial concordance review — 1 November 2026

Outcome: **MATHEMATICAL_STATEMENTS_UNCHANGED / PASS**.

Independent reviewer agent: `/root/formal_guard`. This reviewer did not draft the candidate prose or the underlying Lean development. This is a fallible source-concordance review, not a new Lean execution or empirical validation.

## Exact reviewed sources

| Source | SHA-256 |
|---|---|
| Candidate `paper/article/partial-identification.tex` | `574287fea71e64e8078b6593d15ce3e004ce502c4c2ed59700a3d28b465682c6` |
| Previous current article at `5936f2c28fd075217b679c0b2ad3aba57b9801f3` | `9fa0f67b732dea619417902459d3854e7cc211253f918a5b7f0f1f74e2391716` |
| Frozen `paper/formal/targets/partial-identification.tex` from `38c4ce40b789d4c5420c98ff1d568b39c3b1e798` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |

Read the full candidate, immutable target and complete candidate-versus-previous diff. Also checked the local labour and cohort-productivity definitions in the current main article, the formal semantic map, scope inventory and retained source-review descriptions. No material mathematical mismatch was found.

## Literal checks

The ordered sequences of all 99 inline mathematical spans, the single bracketed mathematical display, the six equation/align environments, both complete proposition environments and all 11 labels are byte-identical to both previous current article and immutable target. Proposition statements retain all original quantifiers, hypotheses, boundary cases and conclusions. These are source observations, not proof that all mathematical meaning is confined to extracted spans.

All six citation commands, arguments and optional locators are byte-identical to the previous current article. The frozen target has fewer citations because the earlier reviewed manuscript added purification and Lean/mathlib attributions. Those remain contextual and do not change the proof target.

## Semantic review of changed passages

1. **Identified range.** The opening retains both inclusions: every listed total has a history satisfying observations and allocation constraints, and every such history gives a listed total. The promised extrema and intermediate attainability are established under the explicit nonempty-feasible-set hypothesis of the unchanged proposition.
2. **Separate bounds.** Separate sentences name simultaneous attainment of all component minima or all component maxima. The original necessary conditions, distinct lower/upper cases and exception for linked contributions are preserved. The prose does not claim independently calculated valid bounds are jointly sharp.
3. **Observation groups.** “Streams whose labour the observation includes” identifies membership in the same set used by the integral constraint. The interval, groups, finite bounds and units are unchanged. The revised Horvitz–Thompson warning still requires accounting for sampling uncertainty before justifying either inequality.
4. **Binary reconstruction.** Inside the proof, the only prose change splits the instruction setting the corresponding allocation to one and the others to zero from the instruction leaving remaining time idle. Block lengths, half-open boundaries, zero-length handling, endpoint convention, measurability, cell totals, observation bounds, target preservation and arbitrary-switching assumptions retain their meaning. Proof prose is semantically preserved but is not byte-identical.
5. **Purification attribution.** Replacing the semicolon by a sentence boundary retains the Dvoretzky–Wald–Wolfowitz and Khan–Rath–Sun citations at the same finite-partition interpretation. This review checks citation attachment and claim scope, not a new primary-source bibliographic audit.
6. **Sensitivity.** The revised explanation names the weighted sums of inequalities that reuse nonnegative dual multipliers. The changed right-hand-side vector, its nonnegative relaxation, possible improvement after reoptimisation and observational justification for error limits remain intact.
7. **Sampling coverage.** The text still distinguishes marginal from simultaneous coverage, preserves event inclusion, the probability lower bound, `0 < alpha < 1`, empty-feasible-set failure, the union-bound condition and the need for an argument for the actual sampling and measurement process. It does not promote individually calibrated marginal intervals to the same joint coverage.
8. **Formal scope.** The new description of integral/cell identities, integrated capacity, one-activity-at-a-time schedules, supplied allocation/multiplier checks and conditional coverage matches the recorded Lean components. It still excludes extrema existence, dual-multiplier existence, the numerical block-count proof and Rust source refinement. The final explicit reference to the Rust solver's source code resolves the potentially ambiguous pronoun without changing this exclusion. The paragraph does not claim that empirical assumptions hold or that a single theorem packages every indexing choice.

## Comparison with frozen target and preserved boundary

The three earlier additions remain: the Figure 1(c) schedule callout, contextual purification citations and formal-supplement coverage paragraph. This editorial pass changes their wording only as described above. Mathematical definitions, assumptions, proposition statements, proof steps and interpretations retain their prior meaning. Acceptance times and retained fractions remain fixed during allocation optimisation. Positivity and joint-attainability conditions on productivity bounds are unchanged.

The immutable target, Lean sources, dependencies, checker scripts, semantic map, original semantic review and original proof-replay evidence were not changed. This report binds only the exact candidate bytes above. A later byte change requires a new concordance review and binding. The review supplies no new proof, implementation refinement, measurement premise or causal-identification evidence.
