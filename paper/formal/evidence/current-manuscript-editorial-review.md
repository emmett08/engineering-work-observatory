# Current manuscript editorial concordance review — 2 November 2026

Outcome: **SEMANTIC_FIDELITY_PRESERVED / MATHEMATICAL_STATEMENTS_UNCHANGED / PASS**.

This record summarises independent read-only review by `/root/concurrency_scope` of the complete current `main.tex` and `partial-identification.tex`, compared with PR #16 at `af928f3bbcbcaf5ace64e9bd00a29a87149eeb0e`. The other five compiled text sources are byte-identical to that baseline. No defect was found in the requested concurrency and bound-verification changes. The editor separately checks source preservation across all seven sources.

## Current source bytes

| Source | Baseline SHA-256 | Candidate SHA-256 |
|---|---|---|
| paper/article/main.tex | 6dcfa616a6ce1baa508d953d31a0c50bc9ff89e5d112ee9872cbda5eca2fea1d | 044822230baf2141106bad7d32eccd9f0b65942f07c5079d3ff68d1f024ae9a3 |
| paper/article/event-history.tex | 74339eabaf00054826bd282368d7bc049a3d0991e5768feffa8ab46ecc51d05d | 74339eabaf00054826bd282368d7bc049a3d0991e5768feffa8ab46ecc51d05d |
| paper/article/partial-identification.tex | 12b1b49de907fa56535465b45da063bd4b5a2922862d15f00e380926143ab067 | d428b8bd0bd52954500a1130766e2070d5f86d5ab05a4e4fbfd0c85792b25f45 |
| paper/article/design-based-estimation.tex | 9277efb1f2bf85979bee3328c6422d021a559b273f817f0bc64ed3cdbe8d93e8 | 9277efb1f2bf85979bee3328c6422d021a559b273f817f0bc64ed3cdbe8d93e8 |
| paper/article/sampling-validation.tex | fb54cff649fb392dc45ba88130c8a067c0f739c7fbaa778d52d12d5b32f16104 | fb54cff649fb392dc45ba88130c8a067c0f739c7fbaa778d52d12d5b32f16104 |
| paper/article/scalar-bounds.tex | 107a796e296b199f24fc4ff0c31383384375c3cfa6d564ee23c4656629002b32 | 107a796e296b199f24fc4ff0c31383384375c3cfa6d564ee23c4656629002b32 |
| paper/study/ai-labour-causal/causal-study.tex | e356ba2c2ca8f452a9dc6ddf5c35738406cc5adca2e6c6ae334af74cc3ffecf9 | e356ba2c2ca8f452a9dc6ddf5c35738406cc5adca2e6c6ae334af74cc3ffecf9 |

## Concurrency and mathematical scope

The allocation constraint limits the human time attributed to a person, while several AI workflows may execute concurrently. The new constructed example uses separate ten-minute and twenty-minute human activity periods, giving half a person-hour, and records one execution hour for each workflow separately. The account makes no claim about the physical impossibility of cognitive multitasking. The pilot addition calls for checks of shared oversight and attended waiting during concurrent execution.

The binary reconstruction preserves cell totals, interval observations and weighted labour targets. It supplies an alternative history under the model's capacity and observation constraints. It does not establish preservation of actual task order, acceptance times or delivered output. Additional dependencies and switching restrictions require further constraints and analysis. Their feasible allocations form a subset of the original feasible set, so their labour totals remain within the original extrema while their attainability requires a separate argument. This is a mathematical scope explanation, not new empirical evidence.

The two requested replacement sentences are included verbatim. Matching a valid upper or lower bound with a feasible attained total establishes the corresponding extremum. Exact arithmetic is expressly conditional on rational problem data, cell totals and multipliers. The existing numerical-solver qualification remains visible.

## Preservation and verification boundary

All 281 mathematical spans, five complete propositions and five complete proofs remain byte-identical in source order. Citation-command sequences and labels are unchanged. The only new local reference points to the task-accounting section. The bibliography, figure inputs, frozen target, Lean sources, checker scripts and historical execution evidence are unchanged.

The source binding identifies this independently reviewed current article separately from the frozen formal target. The formal supplement's mathematical coverage and exclusions are retained. The concurrent-workflow explanation adds no claim that Lean formalises agent execution or verifies workplace measurement. Static receipt and changed-byte rejection checks, finite calculations and PDF inspection are recorded in `current-manuscript-editorial-checks.json`. No fresh Lean replay, Rust refinement proof, empirical validation or measured human comprehension is claimed. Later source edits require renewed review and binding.
