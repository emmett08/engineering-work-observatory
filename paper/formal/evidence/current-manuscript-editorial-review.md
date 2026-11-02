# Current manuscript editorial concordance review — 2 November 2026

Outcome: **SEMANTIC_FIDELITY_PRESERVED / MATHEMATICAL_STATEMENTS_UNCHANGED / PASS**.

This record combines exact source comparisons with the read-only review returned by independent reviewer `/root/manuscript_review`. The reviewer authored none of the manuscript sources. The baseline is repository revision `e0ac5f741f4e029fed52b6366c952fb36acf0a99`. The review covers the complete labour article and its six included text sources, figure caption, TikZ inputs and bibliography. It finds no remaining defect in the candidate bytes identified below. The forecasting companion is outside this pass.

## Reviewed source bytes

| Source | Baseline SHA-256 | Candidate SHA-256 |
|---|---|---|
| paper/article/main.tex | 8224afe88b5e1ae5afcba834924739125f23f91301c8af8e0676a372210a50ba | 3ac71fea65409532afdd8edac01495bad2a2a9c876e14a1e6aee45b4b640cab1 |
| paper/article/event-history.tex | 981e778bbc2131075484826af3b49ca3a6228e0aab24a49b9acc3faa389030ce | 3b4eaef3689a702356bc83ded203535d6c0b6ec20909ac471968cff0b9072e41 |
| paper/article/partial-identification.tex | fe3351e85225f2fee0ae704ed2ae96c09531b10287bf5e8c5dc45be7d8fd9d4a | 6b5a8cc288887c023e3fa09499d5237c2a05c288349c165a0ddc44664cf91360 |
| paper/article/design-based-estimation.tex | fb10092d7a72db49c8e592b132b421d2c15a06f532e73c9afc94fe3de00548e7 | 353ed54a82168bafd2a7022302659131295a5fba2aea4d5628a111e81e3c707c |
| paper/article/sampling-validation.tex | 0d82c3098f4072241a029e57797a63b427b9e9a38ff1cb650eb4e2cb2cba36dd | 1be5cc6f714538a118a1c4b5f031adc520fca72ebeacf91829b88950beb187ee |
| paper/article/scalar-bounds.tex | 84afcfeaefd619ffd7036f564fdc5ea198b6e2b05766798c399344bfabec31c6 | 84afcfeaefd619ffd7036f564fdc5ea198b6e2b05766798c399344bfabec31c6 |
| paper/study/ai-labour-causal/causal-study.tex | acc7e4387a36852cf7fef799b8e42d0c867cbc01bddbf3bd572e91291da64f1b | 99f544d05a8125444ba16534f159b254b4aa2c8243b4c0fa01dba80968da8f23 |

## Source and argument preservation

All 276 mathematical spans and all five complete propositions are byte-identical in source order. The ordered citation, reference, label, input and bibliography commands are also identical. Four complete proofs are byte-identical. In the remaining proof, the endpoint instruction now explains that a zero numerator permits omission of the zero-length piece. Its mathematical expressions and construction are unchanged. Extracted-span comparisons supplement review of the surrounding argument.

The abstract establishes the distinct outcomes before narrowing to labour, and explains shared capacity beside the finite construction. Feasibility, attainment and unrestricted interruption retain their conditions. The clock definitions still distinguish the acceptance deadline, acceptance-relative retained-output check and common labour endpoint. The fixed-prediction estimator retains its sampling and measurement premises. The variance comparison is tied to its common positive design factor. The prediction-validation prose correctly states that the cross term has expectation zero and that expected squared errors share the cancelling sampling variance.

The team-normalisation paragraph now states the positive reference-score total already required in the companion protocol README. Equal team weighting and the direction of the paired comparison are explicit. The experiment remains specified and unrun, with its provisional criteria, adverse hypothetical planning results and measurement-pilot requirement intact. Attribution and citation attachment are preserved. This review does not freshly retrieve the cited literature or measure reader comprehension.

## Frozen target and proof boundary

The immutable target, Lean definitions, checker scripts and recorded proof executions retain their baseline bytes. The current manuscript remains separately bound to its reviewed source; its byte identity is not equated with that of the frozen target. Earlier contextual differences and the additional proof-punctuation revision are covered by this concordance.

The formal-coverage paragraph retains interval/cell integral identities, integrated capacity, binary schedule reconstruction, supplied certificate checks and conditional sampling coverage. Extrema existence, dual-multiplier existence, the block-count bound, Rust source refinement and empirical premises retain their stated exclusions. No fresh Lean execution, kernel replay or empirical validation is claimed.

The independent reviewer inspected sources and argument only. The editor separately rebuilt and rendered the PDF and ran the finite calculation and static binding checks recorded in `current-manuscript-editorial-checks.json`. This review applies only to the source hashes above. Later edits require renewed review and binding.
