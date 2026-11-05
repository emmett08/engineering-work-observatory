# Current manuscript editorial concordance review — 5 November 2026

Outcome: **SEMANTIC_FIDELITY_PRESERVED / MATHEMATICAL_STATEMENTS_UNCHANGED / PASS**.

Independent reviewer `/root/independent_fidelity` compared the nine labour text sources below against public revision `5215d6a9b19e7ee20a9f06b3562a6bd5e231b6bc` in `emmett08/engineering-work-observatory`. The public Git tree was fetched independently through the GitHub connector, and all nine baseline source blob hashes matched the local comparison originals. No local checkpoint SHA is represented as a public revision. The review covers terminology, scoring semantics, timing, mathematical conditions and local explanatory progression. It supplies no measured reader-comprehension result, fresh literature audit or new Lean proof replay.

## Reviewed source bytes

| Source | Baseline SHA-256 | Candidate SHA-256 |
|---|---|---|
| paper/article/binning-bounds.tex | 714d75696b4a12439cf9ce39ee4084d3b9fb919196aa2b72a6d385d115a348dd | ed1a875bb5468d5693d4f9272dc3e7942b7f418ed2976c04e26df16815db2b60 |
| paper/article/design-based-estimation.tex | 3993b40a586eeeb11129204711f5173b01c57de724da8183a1260ff0b4c3bbd3 | 7832b0b3a60dc00ccadc43f41666ac48a2f692ff190934c85a4ddd7333da70a1 |
| paper/article/event-history.tex | 8055b16642d328c79fd517adae746dd5e9d8893c1d3545e07bd0563cc66405b3 | df53036b1ccdcf31b4a7bc58b02892a2df8cc588868a7f104213884c8baecda0 |
| paper/article/main.tex | 8b5269db48dd8dca5329e1d56abb751259690fcbef635bbc6f63222ef4f43d93 | ad48aa72b50730dae0d597c36a4718b35d732464f3536e7ad0761d1c3d19f8c3 |
| paper/article/partial-identification.tex | b4cfe7045327fd49ff7a083195fe4d614a6df216670f45bd7d76e70f0e79b27b | f5812c85b785e3841d90e02f1ae02a7ca899478b78a225f79f6f79e3b17db52a |
| paper/article/performance.tex | e87f5a22d99e443f2c529d3a527989d105355f7bded9537bc4402a95fbc44b3a | 62510d4d57e92a5c7fe850967151b0805fcee43d344128d2d809c545fb3b504b |
| paper/article/sampling-validation.tex | 556fe98975bd157ca202b633b46854ad00aa473f8751d0f3991d5051328c3142 | f1ac0790c00a7a8764509c5a8de73eb57581a410e8b919a2c8eff3caf0666252 |
| paper/article/scalar-bounds.tex | 69a06460f73d7327b831a01d586e8543c097293725f9af9452ac457d6a76feb8 | 69a06460f73d7327b831a01d586e8543c097293725f9af9452ac457d6a76feb8 |
| paper/study/ai-labour-causal/causal-study.tex | fa074efbd80dd038873ab64c842039e54b5f44d5a4276256bffcbad792dd7b0b | 99af867774d90106a023ef96d497b6c9e1999c02837dbc66a6a5eed171fd668b |

## Scoring meaning and units

The reference score S, assessed fulfilment fraction R and deadline-gated weighted score Q remain distinct. For a task accepted by the deadline, Q multiplies its reference score by the fraction of the accepted requirement still satisfied at the specified delay after first acceptance. Other tasks contribute zero. A zero weighted score can also follow from a zero reference score or zero assessed fulfilment. The revised label therefore introduces no claim that every zero denotes a failed task or absence of delivered functionality.

The productivity numerator remains the sum of weighted scores, with reference units preserved. Its denominator remains human labour through the common task-age limit. The team comparison instead divides each team's weighted-score sum by that team's positive reference-score total. This dimensionless ratio is a reference-score-weighted average of fulfilment fractions, assigning zero to deadline misses. The five-percentage-point loss criterion and 0.01 measurement-bias allowance apply to this ratio. The labour saving threshold and allowance retain person-hours per reference unit. Equal team weights remain separate from the task-score weighting.

The performance index stores an acceptance age and the raw reference-score-times-fulfilment product for each accepted requirement. Query inclusion applies the deadline. The added acceptance-age inequality makes this distinction explicit without changing the computation. Returned totals, checksums, accepted components, task delivery and requirement assessments are named according to their particular roles rather than through one repeated umbrella noun.

Independent review identified and the authors repaired three local issues before the source freeze: the abstract again gives a specified assessment time after first acceptance, the noninferiority explanation identifies a decrease in the score ratio, and the serial-schedule limitation concerns actual task order, acceptance times and requirement fulfilment. That last statement is distinct from the fixed acceptance/assessment inputs used in labour optimisation. No material semantic regression remains in this bounded review.

## Word and mathematical preservation

The audited source scope contains 18 paper/performance/figure files. Whole-word output/outputs occurrences are zero in the candidate sources. Every commit/commits token retains its original text and source order in each audited file. Bibliography, protected schema/code identifiers and historical evidence are outside this prose-count scope. Lexical counts describe coverage and do not establish readability.

Every existing mathematical expression remains in source order. The only added mathematical expression is the performance explanation of acceptance-age inclusion. Displayed equations and mathematical assertions retain their content. The binning proposition changes only its label for Q. The performance algorithm input description and query caption change their quantity names, and one proof sentence names weighted scores. Their operations, hypotheses and conclusions are unchanged. These exact permitted prose changes were checked separately from the mathematical expressions.

The actual compiled manuscript closure has 13 source files, 40 labels and 35 reference commands, with no missing or duplicate labels. All 26 citation keys across the nine text sources resolve in the article bibliography. The unused binning/performance sources retain their pre-existing references outside the compiled closure; the clean cross-reference claim applies to the compiled manuscript.

## Proof and evidence boundary

The frozen formal target, Lean source files and checker scripts retain their baseline bytes. The current manuscript binding is renewed separately from that frozen target. The formal receipt preserves its historical source snapshot, proof executions, axiom declarations and exclusions; only its current-manuscript hash changes. The accompanying checks contain the static validators and isolated changed-byte guard actually run during this review. The primary agent manages PDF builds and visual inspection, which are not asserted as this reviewer's own observations.
