# Current manuscript editorial concordance review — 2 November 2026

Outcome: **SEMANTIC_FIDELITY_PRESERVED / MATHEMATICAL_STATEMENTS_UNCHANGED / PASS**.

This record summarises the read-only assessment returned by independent reviewer `/root/whole_paper_review` and the separate analytical review by `/root/proof_review`. The baseline is PR #16 at `fe14cb6b2105681036a263debf01b7e2454c8149`. The whole-paper reviewer read all seven compiled text sources, the figure caption and bibliography using the updated `write-for-experienced-engineers` skill. No actual defect was found. This pass concerns the labour article.

## Reviewed source bytes

| Source | Baseline SHA-256 | Candidate SHA-256 |
|---|---|---|
| paper/article/main.tex | 3ac71fea65409532afdd8edac01495bad2a2a9c876e14a1e6aee45b4b640cab1 | 6dcfa616a6ce1baa508d953d31a0c50bc9ff89e5d112ee9872cbda5eca2fea1d |
| paper/article/event-history.tex | 3b4eaef3689a702356bc83ded203535d6c0b6ec20909ac471968cff0b9072e41 | 74339eabaf00054826bd282368d7bc049a3d0991e5768feffa8ab46ecc51d05d |
| paper/article/partial-identification.tex | 6b5a8cc288887c023e3fa09499d5237c2a05c288349c165a0ddc44664cf91360 | 12b1b49de907fa56535465b45da063bd4b5a2922862d15f00e380926143ab067 |
| paper/article/design-based-estimation.tex | 353ed54a82168bafd2a7022302659131295a5fba2aea4d5628a111e81e3c707c | 9277efb1f2bf85979bee3328c6422d021a559b273f817f0bc64ed3cdbe8d93e8 |
| paper/article/sampling-validation.tex | 1be5cc6f714538a118a1c4b5f031adc520fca72ebeacf91829b88950beb187ee | fb54cff649fb392dc45ba88130c8a067c0f739c7fbaa778d52d12d5b32f16104 |
| paper/article/scalar-bounds.tex | 84afcfeaefd619ffd7036f564fdc5ea198b6e2b05766798c399344bfabec31c6 | 107a796e296b199f24fc4ff0c31383384375c3cfa6d564ee23c4656629002b32 |
| paper/study/ai-labour-causal/causal-study.tex | 99f544d05a8125444ba16534f159b254b4aa2c8243b4c0fa01dba80968da8f23 | e356ba2c2ca8f452a9dc6ddf5c35738406cc5adca2e6c6ae334af74cc3ffecf9 |

## Technical and editorial assessment

All five complete propositions retain their baseline bytes. Mathematical spans outside the expanded cutoff proof remain byte-identical in source order. The other four complete proofs are unchanged. The cutoff proof now explicitly defines its piecewise constant function, calculates both integrals and treats the two zero-length cutoff cases separately. The rates remain between zero and one; the integrals are the requested partial total and its complement. Its necessity and attainability arguments retain their original domain and hypotheses.

The certificate explanation pairs a feasible attaining allocation with an upper or lower inequality valid for every feasible allocation. Its directions, nonnegative multipliers and nonempty-feasible-set qualification are correct. The productivity paragraph now calls the bounded quantity an output score rather than quality in general. Its ratio and joint-feasibility conditions are unchanged.

The output account states acceptance eligibility, initial reference score, fulfilment fraction, follow-up time and reference units locally. Its ten-unit example is explicitly constructed. Common labour observation periods and acceptance-relative follow-up remain distinct. The empirical assessment procedure remains an input needing validation. The proposed study stays specified and unrun, with its provisional criteria and hypothetical adverse planning results preserved.

AI is defined independently in the abstract and main text. Standard Coding Hours and Horvitz--Thompson are written in full in prose. HT is explicitly introduced before its equation-label use. The one-off framework and organisation references retain their cited contribution and source identity. Citation-command sequences and labels are unchanged; all 24 citation keys resolve. One added local reference names the difference-estimator equation. The bibliography and figure inputs are byte-identical to baseline. Citation attachment and local scope were reviewed; external literature was not freshly reverified in full.

## Formal boundary and validation scope

The frozen formal target, Lean definitions, checker scripts and historical proof executions retain their baseline bytes. The independently reviewed current manuscript is separately hash-bound to that evidence. The clearer proof exposition does not change the theorem stated, and no new Lean coverage or Rust refinement is asserted. Existing exclusions for extrema existence, dual-multiplier existence, the block-count bound and empirical premises remain visible.

The editor separately ran exact finite calculations, rebuilt and rendered the PDF, and validated the static receipt and changed-byte guard. Those results are recorded in `current-manuscript-editorial-checks.json`. The reviewers performed local source, argument and analytical checks. No measured reader comprehension, workplace result or fresh proof-kernel replay is claimed. This assessment binds only the source hashes above.
