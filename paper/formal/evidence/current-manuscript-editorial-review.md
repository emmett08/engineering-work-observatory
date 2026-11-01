# Current manuscript editorial concordance review — 1 November 2026

Outcome: **MATHEMATICAL_STATEMENTS_UNCHANGED / PASS**.

Independent reviewer agent: `/root/supporting_sections_edit`. The reviewer authored neither reviewed source nor the Lean development. The review covers the two sources below, their complete diff against repository baseline `07c80fde31a520da012fc54d2dffb50ced420e8a`, and the immutable proof target. It is a source-concordance review. It supplies no new Lean execution, kernel replay or empirical validation.

## Exact reviewed sources

| Source | SHA-256 |
|---|---|
| Candidate `paper/article/partial-identification.tex` | `f0f955912e9da22dd14187610ca290f99a837c184b3b25813ecd4f9cf233a927` |
| Baseline current `paper/article/partial-identification.tex` | `574287fea71e64e8078b6593d15ce3e004ce502c4c2ed59700a3d28b465682c6` |
| Frozen `paper/formal/targets/partial-identification.tex` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |
| Candidate `paper/article/main.tex` | `1df5662bbc1b6a2036c60141789bf7c95e6baa810079b9f04060fb1edb97622d` |

## Literal checks

Ordered extraction from the candidate and baseline confirms exact equality of the following source spans:

| Category | `partial-identification.tex` | `main.tex` |
|---|---:|---:|
| Inline mathematical spans | 99 | 43 |
| Bracketed mathematical displays | 1 | 0 |
| Equation/align environments | 6 | 5 |
| Complete proposition environments | 2 | 1 |
| Complete proof environments | 2 | 1 |
| Citation commands with arguments and optional locators | 6 | 9 |
| Labels | 11 | 12 |
| Reference/eqref commands | 10 | 9 |

The partial-identification candidate also matches the frozen target exactly for all mathematical spans, complete propositions, labels and reference commands. Its complete proofs match the repository baseline exactly. Relative to the frozen target, the only proof-text difference is the previously retained replacement of a semicolon by a full stop in the binary schedule construction. The original instructions to set one allocation to one, set the others to zero and leave remaining time idle retain their meaning. The frozen target has fewer citations because the previously reviewed manuscript added contextual purification and Lean/mathlib attributions. These are retained unchanged by this revision.

These literal checks supplement the semantic review. They do not assume that mathematical meaning is confined to extracted spans. The final economy pass was reviewed again: its shorter framing retains the same definitions, example mechanisms and qualifications, and the literal checks above were repeated on the final bytes.

## Semantic review

The partial-identification revision preserves both inclusions defining the identified range, nonempty-feasible-set conditions for attainability, the distinction between separately valid bounds and simultaneous endpoint attainment, and the need to retain observation cells outside target windows. The examples remain constructed calculations. The allocation assumptions, interval bounds and fixed target weights are unchanged.

The revised explanation of fractional cell totals still requires unrestricted interruption and switching within each cell. Independent review identified a weaker formulation of that assumption, and the candidate now states explicitly that reconstruction requires work to be allowed to start, stop and change type or task at arbitrary times. Restrictions on session lengths, interruption, smoothness and switching costs continue to require separate analysis.

The certificate and observation-error paragraphs retain nonnegative dual multipliers, the sign-reversed lower observation inequality, nonnegative relaxation of right-hand sides, possible improvement by reoptimisation and the need to justify error limits. Sampling coverage still requires simultaneous coverage, counts an empty feasible set as failure, and preserves the union-bound condition. Productivity bounds retain positive lower labour, joint feasibility of output and labour, and fixed acceptance times and retained fractions throughout allocation optimisation.

The main-source revision preserves task identity and output-counting rules, separate access/use records, per-person capacity, abandonment/censoring, reference-score calibration, the three clocks and the positive-denominator condition. Its two-component example illustrates the existing grouping-and-acceptance mechanism without claiming a workplace observation. Independent review also restored the explicit incompleteness of line-survival, testing and incident observations of quality. Sampling, measurement and causal-study qualifications remain visible. No new empirical or causal result is asserted.

## Preserved proof boundary

The retained Lean-coverage paragraph is unchanged. It continues to exclude existence of extrema and dual multipliers, the numerical bound on schedule blocks and Rust source refinement. The immutable proof target and Lean sources were not changed by this editorial review. Historical proof execution and replay evidence retain their recorded scope.

This review binds only the candidate bytes identified above. It does not review the other included manuscript sources or establish manuscript-wide semantic fidelity. A later change to either reviewed source requires a fresh review of the changed content and an updated binding where applicable.
