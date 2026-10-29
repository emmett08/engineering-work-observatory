# Frozen automatic throughput forecast evaluation — v2

## Status and amendment history

Frozen before inspecting candidate forecast losses or outcomes. Supersedes the PR-level endpoint in v1 because the user selected **team throughput and bottlenecks** while the independent mesh was forming. The initial v1 file is retained. Peer discussion changed the default empirical window from one peer's proposed 13 to 8 before scoring; it deferred GLM/NB fitting and survival. This is a design decision, not a result of benchmark selection.

A throughput forecast is accepted as an implemented experimental feature if its chronology, coverage, distribution, score and automatic provenance contracts pass adversarial checks. It is not accepted as empirically superior merely by producing a useful-looking number. Superior predictive performance requires separately sufficient future evidence.

## Estimand

Freeze the selected repository IDs. Primary outcome at each Monday 00:00 UTC origin t is the total number of distinct PRs merged in `[t,t+7 days)` across that portfolio. Count new PRs opened and merged during the forecast horizon. A backlog-only forecast has a different target. Fourteen-day count is secondary and its adjacent weekly forecasts overlap; do not treat these outcomes as independent.

Report portfolio total and per-repository results separately. A merge count is not a functionality, labour, quality, AI-effect or staffing outcome. No manual AI, labour, task-class or task-acceptance label is needed.

Observed backlog pressure may include current open count, age distribution, oldest items, submitted-review state and CI state where their source coverage/freshness is explicit. These are operational observations; they do not identify a capacity constraint or its cause. Historical queue reconstruction requires complete close/reopen and other relevant event history, or retained snapshots. Today's `closed_at`, draft, head, review state and diff do not supply that history.

## Data/provenance gate

Retain immutable raw payloads, collector receipt times, repository/PR identities, source hashes and complete-pagination status. Deduplicate by stable repository ID and PR ID (or repository ID plus PR number). Missing/partial enumeration must not generate zero counts. Complete present enumeration identifies the visible inventory; it does not prove deleted, transferred or inaccessible historical records never existed. Report that boundary.

Mode R: **retrospective count reconstruction from currently visible PRs** uses `merged_at` to reconstruct outcome weeks and only immutable past timestamps for historical count features. Its assumption is that these event timestamps describe the past faithfully; historical availability was not demonstrated. Mode P: **prospective observed-at evaluation** uses source receipts and forecasts actually retained before t. Never aggregate these two modes into a single validation claim. Persist forecasts with actual acquisition time, event cutoff, selected scope, model/version, fitted periods, horizon, distribution and source/snapshot digest.

Do not invent a history of zeros before the visible observation frame. Report repository creation and earliest visible PR creation. Start complete portfolio bins after all selected repositories exist and the declared visible-history frame begins; discard the partial initial week. This lower bound remains conditional on current visibility, not a certificate of original historical inventory.

## Frozen model and comparators

Candidate: empirical predictive PMF from the preceding eight fully completed UTC weeks. With four to seven weeks use all available prior weeks, explicitly record training n, and call this the declared fallback. With fewer than four weeks the empirical model abstains. Equal outcomes retain their frequency mass; retain genuine zero weeks.

Primary comparator: expanding-history empirical PMF from all prior complete weeks on exactly the same supported origins. Additional fixed comparators: zero point mass; previous-week point mass; Poisson distribution with mean equal to the preceding four completed weekly counts' mean. The Poisson comparator is explicitly a restrictive benchmark, not a claim that merge counts are equidispersed or independent.

Do not pick a different best comparator each week. Compare the same candidate to each fixed comparator and report every loss. Candidate and expanding comparator intentionally coincide while no observations have aged out of the eight-week window; this is not a defect.

## Chronology and scoring

At origin t, training windows must have ended at or before t. A merge or PR created after t cannot influence its forecast. Freeze all windows, feature definitions, model families and scoring before comparing outcomes; no random train/test splits or outcome-informed hyperparameter choice. Predict tied-origin repositories together. All scored forecasts require a fully elapsed horizon and complete target coverage. Retain immature and missing forecasts as pending/missing, not in the score denominator. Repeated or overlapping forecasts do not increase the independent replication count.

Primary loss is the discrete ranked probability score, equivalent to CRPS on nonnegative integers:

`RPS(F,y) = sum over k >= 0 of (F(k) - 1{y <= k})²`.

For finite empirical PMFs, sum through `max(max_support,y)-1`; a point-mass forecast gives `abs(point-y)`. Poisson computations need a documented numerical tail rule or an independently verified closed-form/exact numerical implementation. Do not silently truncate a substantial upper tail. RPS rewards the whole forecast distribution and makes an over-wide interval costly.

Report mean RPS, each paired loss difference, n origins, distinct repositories/PRs, date span, horizon maturity, and missing/pending denominator. Secondary MAE uses a predictive **median**. Report predictive **mean** separately as expected count. MAE of a mean is allowed only if labelled and not silently conflated with the median-optimal score.

Historical-quantile intervals can show empirical 10th and 90th percentiles, width, achieved held-out coverage and training n. They carry no guaranteed 80% future coverage under sparse or drifting history. Empirical quantile resolution is coarse with eight observations. Neither coverage without width nor point error without a proper distribution score establishes a good uncertainty forecast.

## Claim and uncertainty gate

A provisional practical-superiority screen is at least 5% lower mean RPS than the primary expanding-empirical comparator, without a material MAE regression. Five percent is an engineering decision threshold, not a statistical theorem; if baseline RPS is zero, superiority is unavailable. Report scores against all additional fixed comparators regardless.

A confident superiority claim additionally requires sufficient future calendar replication and a predeclared dependence-aware uncertainty analysis whose lower improvement bound is positive. Two repositories are not automatically independent replication. Nonoverlapping weeks may still be serially dependent. A bootstrap cannot make a handful of weeks sufficient. For the current small-data demonstration, report raw paired losses and an **unvalidated/exploratory or inconclusive** verdict whenever a defensible uncertainty analysis is unavailable. Do not manufacture a confidence interval or a victory.

Drift is a failure of forecast transport, not evidence that observations were wrong. Report recent-vs-earlier counts/pressure and loss by calendar period; do not tune the window to the evaluation outcome. Forecast scope changes create a new population and invalidate direct comparison with the old portfolio unless explicitly handled.

## Acceptance checks owned by validation

1. Future immutability: changing/appending later events cannot alter a historical forecast.
2. Complete-week boundary: events exactly at t enter the next target week, not training; events exactly t+7d enter the following week.
3. Missing pagination and empty data: missing is not zero; complete zero weeks remain zeros.
4. New arrivals: a PR created after origin and merged before horizon end counts in actual throughput.
5. Open/closed-unmerged: remain inventory and never get transformed into merge events; reopen does not break a merge count endpoint.
6. Maturity: immature outcomes, including early-known positives, do not enter primary score denominators.
7. Identity: duplicate source versions/imports leave counts, predictions and scores unchanged.
8. RPS independent oracle: empirical identity `E|X-y| - .5 E|X-X'|`; point-mass reduction to MAE; known two-point distribution; zero support; large observed count and support tail.
9. Mean/median: skewed PMF reports the two distinctly and evaluates declared point estimate.
10. Chronological benchmark: same supported origins for every comparator; fallback counts and abstentions explicit.
11. Current mutable snapshot: changing draft/head/currentreview/size does not change retrospective count forecasts.
12. Scope/drift: no hidden repository pooling, stable scope ID, report zero-rich and bursty histories without Poisson truth claims.

## Mesh ledger

| Peer | Proposal received | Our proposal sent | Challenge sent | Reply received | Reconciled change |
| --- | --- | --- | --- | --- | --- |
| mesh_observability | yes | yes | yes | yes | Visible-inventory reconstruction wording; queue pressure only; scope drift; missing targets retained |
| mesh_survival | yes | yes | yes | yes | AJ deferred; first-disposition differs from reopen-safe merge target; empirical quantiles/coarse support |
| mesh_sequence | yes | yes | yes | yes | Empirical8 leads; NB/GLM deferred; median used for MAE; tiny-N uncertainty withheld |
| mesh_reliability | yes | yes | yes | yes | Prior13 withdrawn before outcomes; all fixed comparators; no steady-state queue assumption |
| mesh_integration | yes | yes | yes | yes | Retain actual acquisition versus event cutoff; complete-current inventory boundary; explicit4–7 fallback |

## Grounding

Inspected repository `src/snapshot.rs`, `src/store.rs`, `src/collectors/github.rs`, `README.md`, and existing manual-accounting tests. Existing source/knowledge-time retention is useful but current accounting tests do not demonstrate predictive accuracy. Applied the user's `analyse-software-engineering-statistics` skill, especially chronology, censoring/follow-up, independent replication and frozen confirmation rules. Peer source claims were read as peer evidence, not independently verified by this node.

### Pre-score dependence clarification

After v2 was written, sequence raised a valid serial/cross-repository dependence issue. Accepted before candidate losses: first sum synchronized observed repository weekly counts, then fit the portfolio PMF; do not convolve repository PMFs. Fourteen-day output uses four disjoint observed two-week totals covering the preceding eight full weeks, with an eight-week minimum and no weekly-PMF convolution. Score fourteen-day forecasts only every other Monday, with parity anchored to 1970-01-05 UTC; this is secondary and usually severely underpowered. These choices retain dependence within observed multiweek/portfolio blocks without claiming independence across future blocks.
