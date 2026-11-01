# Public observation-learning demo

Declared human effort, tool execution time and model predictions are separate quantities.

Observation acquisition: **complete**. Latest source observations: 1; retained revisions: 1.

| Observation | Issue state | Declared human minutes | Method / coverage | Model prediction | Use in model |
| --- | --- | ---: | --- | ---: | --- |
| [emmett08/engarg#51](https://github.com/emmett08/engarg/issues/51) | closed | 3.0 | unknown / unknown | unavailable | unknown_measurement_method;incomplete_or_unknown_coverage |

Real-data model decision: **observation_only**. No sampled human-effort estimate is available: no workstation activity or sampled responses were supplied.

## Linked execution evidence

| Task | PRs | Tool invocations with both records | Reported tool seconds | Human minutes inferred from runtime |
| --- | --- | ---: | ---: | --- |
| [engarg-authoring-effort-pilot](https://github.com/emmett08/engarg/issues/49) | [#50](https://github.com/emmett08/engarg/pull/50) | 2 | 42.951 | unavailable |

Execution acquisition complete: **true**. Parallel runtimes can overlap; missing receipts are not zero activity.

## Synthetic software demonstration — not human evidence

The fabricated fixture deliberately gives two repositories different task totals. It tests model fitting, validation and promotion, not real predictive accuracy.

Decision: **promoted**. Training tasks: 83; validation tasks: 45. Identical replay preserved evaluation identity: **true**.

Held-out mean absolute error: pooled-median baseline **20.29 min**; candidate **1.77 min** (fabricated data only).

- Fabricated future task `synthetic/short#10001`: 9.59 min; no outcome exists, so prediction error is unknown.
- Fabricated future task `synthetic/long#10001`: 46.87 min; no outcome exists, so prediction error is unknown.

## Interpretation and limits

- Declared effort is a human report, not certified actual labour. Source accounts do not establish worker identity.
- Unknown method, incomplete coverage, partial totals and conflicting task claims are excluded from training; they remain visible.
- Only configured public sources and registered task links are imported. Open and closed issues are both queried.
- Observed revisions are retained; edits between polls cannot be reconstructed. Missing acquisition pauses live prediction and promotion.
- The fixed task-total predictor uses declared totals and repository groups, never workflow duration. It does not replace the interval-sampling estimator.
- Task-group holdout is retrospective. Training fit and reused validation are not evidence of future accuracy; consumed validation tasks cannot approve another candidate.
- The 8-training/4-validation minima and error-reduction gates are demonstration policies, not precision or statistical-power guarantees.
- Synthetic promotion proves software operation only. No real-data improvement or causal AI productivity effect is established.
- GitHub artifact checkpoints expire after 30 days. Missing or corrupt prior trusted state stops automatic continuation; it does not silently reset history.
