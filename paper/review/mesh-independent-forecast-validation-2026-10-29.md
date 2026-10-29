# Independent validation review: automatic merge-count forecasting

## Scope and result

The validation node froze the operational endpoint, chronology, score and comparators before inspecting candidate empirical losses. It then challenged all five other mesh nodes and consumed their responses. The primary endpoint remains the count of PR merges in a complete seven-day portfolio window. A separately declared one-day forecast was added before scoring because the selected user repositories lacked four complete weeks. The daily extension does not establish weekly forecasting skill.

The audited implementation passes the independent numerical and adversarial checks described below. This supports its tested calculation and source-handling behavior. It does not establish predictive superiority, future calibration, staffing implications, delivered functionality, or an AI effect. No user-repository or public-reference performance result was used to select the model by this reviewer.

## Frozen evaluation decisions

The frozen experimental candidate uses the preceding eight complete UTC days or weeks, with a four-bin minimum. After that candidate failed the frozen improvement screen, the application retained the prespecified expanding empirical reference as its default; this decision does not establish that reference as a validated predictor. The recent candidate’s fourteen-day forecast uses four observed disjoint fourteen-day counts and requires eight weeks; it does not convolve independent weekly draws. The expanding reference uses all supported complete periods, also with a four-period minimum. Portfolio counts are aggregated before fitting, preserving the observed within-period dependence across repositories. Model version 2 records the method explicitly; version 1 artifacts are not silently reinterpreted.

The main probability score is discrete ranked probability score (RPS), equivalent to CRPS on integer counts. The main comparator is an expanding empirical distribution; fixed additional comparators are zero, the preceding period, and Poisson with the preceding four periods' mean. All share the same eligible origins. The predictive median is used for absolute error; the mean is reported separately. Quantile endpoints include their actual enclosed empirical mass because discrete sample quantiles need not enclose exactly 80% of the empirical distribution.

Only completed training bins may enter a forecast. Only mature, covered targets may be scored. Open, closed-unmerged and newly arriving PRs are not removed from the source population; all merges within the target interval count, including PRs created during that interval. Historical counts reconstructed from today's visible inventory are labelled retrospective reconstruction. They are not forecasts demonstrated to have existed at those earlier times.

The public reference, if collected successfully, was named before outcome inspection: `cpmpy/cpmpy`, the last 78 complete UTC weeks, with 52 initial training weeks and 26 fixed chronological evaluation weeks. Failure to obtain that frame produces an unavailable result rather than a revised favorable window. Its performance does not establish transport to the user's repositories. A point-score improvement on dependent weeks is descriptive unless a justified, predeclared uncertainty analysis supports more.

## Findings that changed the implementation

1. **Source freshness and false zeros.** The initial projection used snapshot time as observation exposure. A snapshot frozen weeks after the last collection could therefore invent intervening zero-count days. The implementation now distinguishes snapshot/issue time from the conservative common source-exposure boundary, the earliest start time among the selected repositories' latest collection scans. Training bins end before that boundary; forecast targets begin at or after issue time. Using scan start also handles pagination crossing midnight.
2. **Latest failures and incomplete source scope.** A failed latest inventory cannot silently be replaced by an earlier successful one. Incomplete required coverage produces an unavailable history. The projection checks matching repository/all-state inventory scope and collection receipts. Events after their own collection cannot become historical observations merely because a later snapshot was saved.
3. **Duplicate identities.** Identical duplicate API rows initially conflicted with the raw returned-row count after source identity deduplication. The collection summary now reports distinct PR count and duplicate-row count separately, allowing identical duplicates to count once while conflicting duplicates remain invalid.
4. **Frozen comparator.** The expanding empirical comparator was added explicitly to the rolling-origin output. Before the recent window begins dropping older observations, identical predictions between it and the recent model are expected and retained.

Current queue pressure is described as the last observed inventory with acquisition freshness. Open ages are tied to acquisition, not advanced to imply a current observed state. Review and CI stages outside the lightweight profile remain uncollected. These observations do not identify a causal bottleneck.

## Independent checks

A separate Rust harness in the validation workspace compiled the production modules without modifying them. Its expected empirical scores came from an exact rational Python oracle using

`E|X-y| - (1/2) E|X-X'|`,

independent of the production sorted-gap calculation. Eight empirical distributions and 48 outcome scores passed, including all-zero, constant, bursty, skewed, two-point, monotone, short-history and dropped-old-outlier cases. The harness also checked means, medians, quantiles, enclosed mass, empty histories and integer precision limits.

The Poisson benchmark passed 40 independent score and quantile comparisons against an 80-digit decimal CDF-sum implementation. The checked means ranged from zero to 500, with outcomes within and beyond the central predictive range. These checks verify the tested numerical approximation; they do not establish the Poisson law as an appropriate model of repository work.

Independent chronology/source fixtures passed for:

- a stale snapshot after collection, and a collection spanning midnight;
- the latest failed collection overriding an earlier success;
- an event dated after its own source collection;
- wrong inventory scope, identical duplicate identities, and complete empty inventory;
- forecasts that start in the future, and refusal to fabricate weekly/fourteen-day forecasts from short history;
- appending events after the exposure cut without altering earlier forecasts or scores;
- recent and expanding empirical losses checked directly against an independent pairwise-distance calculation;
- the fixed 78/52 weekly evaluation frame producing exactly 26 evaluation origins, with expanding training sizes 52 through 77.

The separate numerical oracle also verified the empirical score identity for 300 deterministic randomized rational cases. These tests are synthetic and labelled as such.

## Remaining limits

Complete traversal describes the currently visible API inventory, not recoverability of deleted or inaccessible historical PRs. Acquisition is a time interval rather than an atomic repository snapshot. Recent empirical distributions may fail under launch effects, changing weekday composition, drift or bursts. Four to eight observations do not justify a confidence claim about future interval coverage. A finite empirical maximum is not a physical bound on future merges.

The executable application tests preserve the repaired behaviors. The independent workspace harness and exact oracle are audit artifacts; implementation correctness and empirical predictive support remain separate conclusions.

## Held-out result independently checked after the design was frozen

The public source snapshot contained 742 visible PRs. The fixed 78-week frame ran from 28 April 2025 to 26 October 2026, with the last 26 weeks used for evaluation. Independent reconstruction directly from retained source timestamps checked all 130 reported method/origin scores. Finite-distribution scores used exact rational arithmetic; Poisson scores used an independent 80-digit decimal CDF calculation.

| Method | Mean RPS, lower is better |
| --- | ---: |
| Expanding empirical reference | 2.274930 |
| Recent eight-week candidate | 2.525841 |
| Poisson prior-four-week mean | 2.766031 |
| Previous-week point mass | 4.500000 |
| Zero point mass | 4.615385 |

The recent candidate had **11.03% higher primary loss** than the expanding reference. It failed the predeclared improvement screen. Its lower median absolute error does not reverse the primary probability-score result. Consequently there is no claim that the recent candidate improves the retained reference. No additional seasonal comparator was introduced after these results were inspected.

The expanding reference in this study used 52 through 77 training weeks within the fixed 78-week frame. The operational reference uses all supported history. These are different training frames, and this one-repository result does not establish the operational method's future performance or transport to the user's repositories. The user's five complete daily bins support only an explicitly unvalidated short-history forecast; no weekly forecast is available yet.

The newly added unchanged-forecast scorer also passed independent checks for half-open target boundaries, arrivals during the target, source-cut maturity, malformed stored predictions, scope mismatch, and score agreement with the independent pairwise oracle. Pending or incomplete follow-up yields no partial outcome score. Original-bundle/source binding remains a separate archive responsibility.

Strict validation of a saved forecast exposed a JSON floating-point round-trip discrepancy in the standard parser. The application enables `serde_json`'s `float_roundtrip` feature rather than weakening distribution equality checks. The independent harness then serialized, parsed and scored expanding forecasts with 4 through 64 training periods successfully. It also checked that the explicit recent method retains eight periods while the default expanding method retains the full supported frame, and that old model-version declarations are rejected rather than silently reinterpreted.

The delivered predictor concerns merge throughput. The queue output is an observed diagnostic, not a prediction of future bottlenecks. A separate, minimal future-pressure study could forecast the number of open PRs aged at least seven days from automatically archived inventory snapshots, against persistence and empirical baselines. It requires longitudinal queue observations and mature future outcomes; it cannot be obtained from the current zero backlog or from an arrivals-minus-merges identity that omits closure and reopening.
