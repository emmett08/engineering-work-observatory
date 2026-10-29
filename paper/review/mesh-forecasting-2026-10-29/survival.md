# Survival-node design and mesh record

Repository examined: `/workspace/work/engineering-work-observatory`, requested PR3 head `87372e5a140dff1df01ba68a25939c60e63031c1`.
Date: 2026-10-29. No repository edits or external posts. All work is research/design.

## Final decision

Do not implement a survival model in the first version. The user selected **team throughput and bottlenecks**, and a direct forecast of next complete UTC week's merged-PR count matches that target with fewer modelling assumptions. Freeze the same eight prior complete weeks for the empirical forecast distribution, as agreed by validation, observability and reliability peers before evaluation. A surviving optional reference for the future is actual merge-within-horizon prediction from an observed historical state, allowing close and reopen; it is not first-disposition survival.

No predictive superiority has been established. The empirical distribution is a falsifiable operational reference. It assumes that the selected recent weeks inform the coming week. It cannot produce guaranteed coverage under arbitrary temporal change.

## Independent proposal and why it changed

The independent first proposal was a same-repository, age-conditioned Aalen–Johansen competing-risk forecast. For a PR not yet closed or merged at age a, it would forecast first disposition as merge, unmerged close, or still unresolved over 1/7/30 days. It used all enumerated PRs, including open right-censored observations at the fitting cutoff. It avoided proportional hazards, a finite duration mean, eventual merging, manual task labels and latent labour.

The initial proposal was sent to the host within the requested first five minutes, before the host relayed the user's team-level preference. After `MESH READY`, the adapted proposal was sent to all five peers. Peer challenges exposed a target mismatch: a PR closed on day2, reopened on day3 and merged on day5 contributes one weekly merge but zero first-disposition merges. Treating unmerged close as absorbing is legitimate for first-disposition prediction but wrong for this throughput target. A full multistate model could model reopening, but adds data and assumptions without evidence it improves the user's forecast. All five peers accepted deferral.

The corrected counting identity is:

`weekly merges = merges from PRs open at origin + merges from existing unmerged PRs closed at origin + merges from PRs created during the forecast window`.

The middle category contains PRs that reopen. Even an otherwise correct current-open-backlog forecast plus a new-arrival forecast misses this category unless it is included explicitly. A direct aggregate merge-count forecast includes all three categories automatically.

For any correctly defined merge indicators I_i, `E[sum I_i | information at origin] = sum E[I_i | information at origin]` requires no independence. A variance calculation that sums Bernoulli variances does require zero covariances and is unjustified here. This distinction is an elementary derivation, not an empirical claim about this project.

## Repository findings

`src/collectors/github.rs:548` enumerates all-state PRs ascending by creation date. It retains list payloads, PR details, commits, files, submitted reviews and review comments. The collector does not collect issue close/reopen timelines. `closed_at` in a later payload cannot reconstruct first closure or past open state after reopening. Current draft, size, head, status and backlog values cannot be silently used as historical predictors.

`src/collectors/mod.rs` retains source event timestamp fields in payloads, with acquisition timestamps in raw response envelopes. Source records are content-addressed. The integration peer identified a further issue: later snapshots do not automatically preserve a complete replayable receipt history for every past prediction. Freeze each forecast's actual input snapshot or features at acquisition time.

Ascending enumeration and request caps are especially important: a truncated collection may omit the newest PRs and newest merges. Incomplete enumeration must not create artificial zero-count weeks. A light throughput collector should request repository identity and PR list only, and treat missing PR inventory coverage as unavailable. Unrelated CI/log endpoint failures should not invalidate a complete merge-count inventory.

Complete API pagination means complete traversal of currently visible records, not proof that deleted or inaccessible historical objects never existed. Name the measured population accordingly.

## Survival method retained as a future research comparator

For a first-disposition target only, let d_M(t), d_C(t) be merge and unmerged-close event counts at age t, and n(t) the risk set immediately before t. The standard competing-risk recursion is:

`S(t) = S(t-) [1 - (d_M(t)+d_C(t))/n(t)]`

`F_k(t) = F_k(t-) + S(t-) d_k(t)/n(t)`.

Conditioning on being first-disposition-free at age a gives `q_k(a,h) = [F_k(a+h)-F_k(a)]/S(a)` and `q_open(a,h)=S(a+h)/S(a)`. Fit requires sorting O(n log n), then a linear sweep with O(n) stored curve points; binary-search queries cost O(log n). This complexity analysis follows directly from the algorithm.

The calendar training cutoff C and PR age a are different clocks. Staggered entry means censoring age is C-created_at; if workflow performance changes by creation cohort, purely administrative censoring need not be innocuous after pooling across cohorts. A mature complete-follow-up reference avoids estimating censoring: for a chosen max horizon H, use training PRs created no later than C-a-H, known eligible at age a, and score actual merge within (a,a+h]. Still-open mature PRs contribute valid no-merge outcomes. This costs O(n) per age query without indexing, sacrifices recent data, and still needs temporal transport. Historical open-at-age eligibility needs a retained snapshot or adequate event timeline.

Support rules must distinguish sparse information from impossibility. Report risk counts and time support; abstain beyond observed follow-up with positive unresolved mass. Do not return a finite median merge time unless the relevant cumulative merge probability reaches one-half. Neither shrinkage across repositories nor conformal intervals belong in the first version: both introduce assumptions that are unnecessary for the selected team forecast.

## Fixed empirical count forecast and scoring

For m=8 prior complete weekly counts x_j, the empirical predictive mass gives each observed week weight 1/m, combining duplicate counts. The expected count is the sample mean; the median is the point summary aligned to absolute-error scoring. Training-window fallback/support decisions belong to the frozen validation contract, not post-score selection.

For observed count y, integer ranked probability score equals:

`RPS = mean_j |x_j-y| - sum_{j,k}|x_j-x_k|/(2m^2)`.

Sorted x_(j) makes the correction `sum_j (2j-m-1)x_(j)/m^2`. Preparation costs O(m log m) and each score O(m), with O(log m) possible using prefix sums. This avoids iterating every integer up to a very large observed count.

The empirical 10th and 90th percentiles are coarse with eight samples, and the distribution assigns no mass outside observed support. Report distributional score, achieved interval coverage, width and training support. Do not describe the interval as a guaranteed 80% future-coverage interval. Nonoverlapping test weeks avoid overlap-induced dependence but may still be serially dependent.

Use the same supported, frozen forward origins for all competitors; fit only prior complete weeks. Freeze the repository scope, calendar boundaries, training rule, models, score and decision threshold before looking at holdout outcomes. If support or calendar span is small, report raw paired scores and call the result inconclusive; resampling does not manufacture information.

## Primary literature inspected

1. Aalen, O. O. and Johansen, S. (1978). *An Empirical Transition Matrix for Non-Homogeneous Markov Chains Based on Censored Observations*. Scandinavian Journal of Statistics 5, 141–150. Full primary PDF retrieved; relevant censoring model and estimator sections 2–3 read, plus model framing and risk-set remarks. https://gwern.net/doc/statistics/survival-analysis/1978-aalen.pdf . Local research copy: `aalen1978.pdf`. The source provides a product-integral transition estimator under a counting-process censoring model; it does not establish applicability or predictive accuracy for this repository.
2. van Houwelingen, H. C. and Putter, H. (2008). *Dynamic predicting by landmarking as an alternative for multi-state modeling: an application to acute lymphoid leukemia data*. Lifetime Data Analysis 14, 447–463. DOI 10.1007/s10985-008-9099-8. Full primary text retrieved; landmark cohort construction, available-at-landmark covariates and dependence from repeated subjects read. https://link.springer.com/article/10.1007/s10985-008-9099-8 . It supports the landmark construction and the need to account for repeated-subject dependence; it does not validate any GitHub throughput forecast.
3. van Houwelingen (2007), DOI 10.1111/j.1467-9469.2006.00529.x: publisher metadata and abstract retrieved only, not relied on for an uninspected technical detail.
4. Putter, Fiocco and Geskus (2007), DOI 10.1002/sim.2712: publisher metadata/abstract retrieved; an external full-PDF mirror failed with HTTP503, so not claimed as fully read.

## Full mesh send/receive/consumption ledger

All proposal messages included target, data, assumptions, algorithm, falsifier and primary-source links. All five peers' initial proposals were received, consumed and substantively replied to before this node's final handoff.

| Peer | Proposal sent | Proposal received/consumed | Substantive reply and resulting decision |
|---|---|---|---|
| mesh_observability | yes | yes | Accepted actual merge-by-horizon allowing reopening and knowledge-time constraints. Challenged ascending-list truncation causing false zeros; peer accepted unavailable on incomplete PR enumeration. Explicitly dropped absorbing close. |
| mesh_sequence | yes | yes | Accepted direct aggregate count forecast with count-distribution scoring. Challenged Poisson/NB tail and retrospective feature availability; peer deferred NB and AJ from v1. |
| mesh_reliability | yes | yes | Accepted direct count distribution and queue-conservation audit only. Challenged arbitrary13-week choice and empirical interval guarantees; peer adopted common8 weeks and withdrew independence implication. |
| mesh_validation | yes | yes | Accepted fixed weekly RPS/MAE evaluation and mature outcomes. Challenged5% threshold as policy rather than theorem, tinyN uncertainty and changing support. Reaffirmed target correction and no AJ v1. |
| mesh_integration | yes | yes | Accepted minimal throughput collector and snapshot freezing. Challenged missing origin-closed→reopen contribution and incomplete enumeration; peer accepted both. Explicitly supported no extra survival CLI modes. |

Host updates sent: independent survival proposal within five minutes; calendar-censoring refinement; after all five peer exchanges, decisive no-AJ-v1 conclusion and corrected three-category merge decomposition; implementation note giving exact efficient empirical RPS.

## Daily feasibility amendment — all five peers consumed

After the initial handoff, the host found the user's repositories are only about five to nine days old and all currently visible PRs are merged. Before scoring, the team proposed the same empirical PMF for an explicitly exploratory one-day forecast from the prior eight complete UTC days, minimum four. The frozen weekly primary protocol remains unchanged; seven-day forecasts require at least four complete weeks and fourteen-day forecasts at least eight. No daily-to-weekly scaling or independent-day convolution is allowed.

All five peers sent this amendment to this node. I consumed and substantively replied to all five, and sent the host a combined acknowledgment. Challenges accepted: target the next full UTC day starting at or after actual issue time; print exact dates; exclude creation/current partial days; use a fixed portfolio and common historical lower boundary after the latest repository creation; preserve a distinct conservative source-exposure upper cutoff so a crawl crossing midnight does not certify the just-finished day from earlier pages. Integration confirmed use of the earliest selected collection start as that conservative upper exposure cutoff.

Clarification sent to the host and validation peer: repository creation determines the historical lower boundary; collection start is an upper observation-exposure cutoff, not the beginning of all usable history. A complete current inventory can reconstruct past visible merge counts, including zero counts, under its source-scope assumptions. Requiring historical bins to follow the first acquisition would intentionally restrict to prospective history and leave no initial daily training data. Retrospective reconstruction must not be called an observed-at prospective evaluation.

Four to eight launch-phase days provide weak support and may mix weekdays with a changing launch rate. Report actual support and empirical values; do not claim calibrated coverage or validated accuracy. An all-zero empirical PMF is a model reference, not evidence of certain future zero throughput. A currently empty open backlog does not prevent newly created PRs from merging tomorrow. A mature public reference repository can test the evaluation machinery but cannot transfer accuracy evidence to these repositories.

| Peer | Amendment received/consumed | Substantive acknowledgment sent |
|---|---|---|
| mesh_observability | yes | yes — common fixed-portfolio start and conservative page exposure cutoff |
| mesh_sequence | yes | yes — target-day clock, no scaling/convolution, sparse support |
| mesh_reliability | yes | yes — empirical interval mass, launch drift, all-zero uncertainty |
| mesh_validation | yes | yes — unchanged frozen weekly design; retrospective/prospective boundary clarification |
| mesh_integration | yes | yes — crawler midnight issue; explicit issue/target/exposure times |

## Implemented automatic forecast scoring and version-2 amendment

The host subsequently authorized implementation of only `src/forecast/validation.rs` and `tests/forecast_validation.rs`. These now expose `score_forecast(&ForecastReport, &PreparedHistory) -> Result<ForecastScore>`, with pending, available and unavailable states. Scoring waits for conservative common observed exposure to reach the entire target, requires the same numeric repository portfolio and adequate coverage/frame, and counts every merge in the half-open target interval, including new arrivals and reopened PRs. It returns RPS, absolute error of the predictive median, empirical-quantile hit and original/later source digests. Malformed canonical clocks, training bins, model metadata or PMFs reject. No human labels or implicit horizon scaling are introduced.

The source digest and receipt timestamps are not trusted-time certification. The host owns archive binding and recomputation from the embedded original source; a standalone internally consistent fabricated history cannot be disproved by algebra alone. The independent validation peer reviewed the scorer and ran its own boundary/chronology/scoring harness with no blocking findings.

The host's prespecified promotion gate failed for the recent-window challenger in the separate public-reference exercise (reported mean RPS about 11% worse than the expanding comparator). The default therefore remains the expanding empirical reference; the recent window is an explicit experimental method. Sequence introduced model version2 and mandatory method metadata, keeping calendar targets and minimum support unchanged. I updated scoring to use the shared method-specific training-count rule and retain method metadata. Missing-method/version1 artifacts reject rather than silently changing their meaning.

A new realistic serialization regression over sample sizes4–64 exposed a genuine issue: default serde_json float parsing can shift a serialized probability by one last binary bit, causing strict canonical equality to reject a legitimate saved forecast. The host enabled serde_json's `float_roundtrip` feature. The regression then passed while strict integer and floating summary checks remained intact; no numerical tolerance was added.

Final owned-file verification: all12 `forecast_validation` tests pass; targeted Clippy with warnings denied passes; owned files formatted. Tests cover half-open target edges, new arrivals with empty origin backlog, zero outcomes, conservative maturity despite later knowledge time, incomplete inventory, scope/frame mismatch, future/duplicate/out-of-scope events, forged clocks/training/PMF, exact serialization, method preservation and old-model rejection. The method amendment and implementation outcome were communicated to all five peers. No commits made.
