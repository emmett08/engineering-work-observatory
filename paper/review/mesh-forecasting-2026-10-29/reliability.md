# Reliability, flow and forecast node

Repository reviewed: `/workspace/work/engineering-work-observatory`, starting PR head `87372e5a140dff1df01ba68a25939c60e63031c1`. Research began after host `MESH READY`. No original manuscript, historical proof or performance artefact was changed.

## Independent proposal and accepted revision

The independent proposal was a direct empirical distribution of next-week GitHub-visible merged-PR counts, initially using 13 preceding complete weeks. It required no manually supplied AI labels, task classes, labour diaries or acceptance judgements. Exact queue conservation was retained as a source-coverage check, not a prediction equation. CI outcomes and reopen/revert signals were explicitly separated from actual correctness.

Before model scores, the node accepted the validation node's common eight-week window, with four-to-seven-week fallback, and withdrew 13 as the default. No window was selected by observed forecast accuracy. All five peers challenged tail resolution, unavailable history or temporal dependence; these challenges were accepted. Twenty-six held-out weeks became an aspiration where available, not an arbitrary prerequisite for experimental operation.

The host subsequently reported that the user's repositories were only 5–9 days old. Before model scoring, the operational candidate was amended to an eight-complete-UTC-day empirical distribution with at least four complete days. The frozen weekly comparison remains unchanged. Seven-day forecasts need at least four complete weeks; 14-day forecasts use four disjoint two-week observations and need eight weeks. No daily-to-weekly scaling or independent convolution is proposed.

## Target, data and assumptions

The target is the count of distinct currently GitHub-visible PR identities with `merged_at` in a future, half-open calendar interval, within a fixed repository-ID scope. It includes PRs created after the forecasting cutoff and merged during the horizon. It is a merge-activity endpoint, not effort, accepted functionality, staffing capacity or correctness.

Existing `src/collectors/github.rs` enumerates all PRs (`state=all`, created ascending), retains detail/review/check information, and explicitly records that current inventories do not reconstruct historical edits or deletion. It already retrieves each workflow-run attempt separately, so an initial failure need not be overwritten by later success. It does not retrieve the complete close/reopen timeline. Its full profile is much more expensive than the PR-only inventory needed for throughput.

Complete current pagination supports a claim about the currently visible inventory, not proof that deleted, transferred or inaccessible historical objects never existed. Empty complete calendar bins are zeros; missing or incomplete collection is not zero. The partial repository-creation bin and current incomplete bin are excluded. Portfolio bins start only after every selected repository exists and the declared observation frame begins. Future forecast intervals begin no earlier than issuance; a forecast issued mid-day targets the next full UTC day rather than the already-partly-observed day.

The numerical empirical distribution can be formed without Poisson arrivals, independent PRs, independent days, proportional hazards, stationary queue equilibrium or latent labour states. Its use for a future outcome still assumes that selected recent counts are informative about that future. Launch bursts, weekday composition, changed policies and changed PR granularity can defeat that assumption. No calibration guarantee is supplied.

## Queue and reliability interpretation

For complete state transitions, `B_end = B_start + opens + reopens - merges - unmerged_closes`. Merge and unmerged-close transitions must be mutually exclusive; counting an ordinary closure event again when it accompanies a merge subtracts twice. This is an accounting identity. Two futures with identical observed past can have zero or many merges, so the identity alone does not identify a forecast or capacity.

Little (2011), section 2.1.4, explicitly distinguishes exact sample-path measurement from forecasting. Its finite-window extension in section 2.2 includes initial inventory in its population and counts only time accumulated within the window. It does not license dividing a current snapshot backlog by mean latency among completed PRs and calling the result future capacity.

The 2026 Queue & AI comparator uses total human-attention requirements, a single effective reviewer capacity, service-time moments, residual-error and rework quantities, and stable queue assumptions; its main formula also assumes Poisson arrivals. The Kingman alternative is a queue approximation with its own conditions, not an assumption-free repair. These quantities are not supplied by PR timestamps. The paper supplies a mechanism illustration, not an automatically calibrated model for the user's repositories.

Review requests and pending CI are overlapping observable predicates. They are not disjoint serial stages or identified causal bottlenecks. A recorded request without a later submitted review may reflect policy or scheduling. A CI failure is a workflow outcome, not proof of a product defect; success is not certification of correctness. First-attempt outcomes should use stable repository/workflow/run/attempt identity and preserve the actual conclusion categories. Canceled, skipped, missing and unobserved attempts must not be silently converted to passing or failing. Reopens likewise do not imply defects. No reliable universal automatic revert-to-defect label has been established by this node; text matching a revert subject is only a heuristic and is not proposed as a primary outcome.

## Score and quantile semantics

Use integer RPS/CRPS, median MAE, mean count, empirical quantiles, actual included PMF mass, historical support and achieved future coverage. For an empirical PMF, the exact loss is `mean_i |x_i-y| - (1/(2n^2)) sum_ij |x_i-x_j|`. The formal independent draws used in this score identity are draws from the issued PMF, not an assumption that the recorded calendar counts were independent. Gneiting and Raftery (2007), section 4.2, verifies the identity and the sign convention; use its nonnegative loss orientation.

Inverse-ECDF 10th and 90th percentiles with eight distinct observations are the sample minimum and maximum. The inclusive range therefore contains all empirical mass, despite the requested central level being 80%. With repeated counts it can also contain all mass. Show actual enclosed mass and do not imply exact 80% future coverage. The historical maximum is not a physical upper bound on future activity. RPS remains finite for an unforeseen burst outside empirical support.

The validation protocol's expanding empirical, zero, previous-count and Poisson-four-mean comparators are fixed. The Poisson comparator is explicitly restrictive. Compare each fixed comparator on the same mature supported origins. Nonoverlapping windows remove overlap-induced dependence but do not establish independence. A provisional 5% mean-RPS improvement is an engineering screen; strong performance claims need adequate calendar replication and a defensible predeclared dependence-aware uncertainty analysis. The user's 5–9-day histories cannot supply that evidence.

## Primary sources acquired and inspected

* John D. C. Little (2011), *Little's Law as Viewed on Its 50th Anniversary*, Operations Research 59(3):536–549, DOI `10.1287/opre.1110.0940`. Full PDF retained as `little-2011.pdf`; sections 2.1–2.3 distinguish retrospective identities, boundary inventory and forecasts. URL: https://people.cs.umass.edu/~emery/classes/cmpsci691st/readings/OS/Littles-Law-50-Years-Later.pdf
* Silvia Bartolucci and Pierpaolo Vivo (2026), *Queue & AI: When Faster Tasks Slow Down the Workflow*, arXiv:2605.27202v1, 26 May 2026. Complete prose/methods and references acquired and read, with the relevant mathematical display equations inspected in the original HTML. Its sections 2.1, 3 (Empirical calibration), and 4.2 establish the data/assumption burden. URL: https://arxiv.org/html/2605.27202v1
* Tilmann Gneiting and Adrian E. Raftery (2007), *Strictly Proper Scoring Rules, Prediction, and Estimation*, JASA 102(477):359–378, DOI `10.1198/016214506000001437`. Complete published PDF acquired from the author's University of Washington site; section 4.2 and equations 20–21 inspected directly. URL: https://sites.stat.washington.edu/raftery/Research/PDF/Gneiting2007jasa.pdf
* Tilmann Gneiting, Fadoua Balabdaoui and Adrian E. Raftery (2007), *Probabilistic forecasts, calibration and sharpness*, JRSS B 69(2):243–268, DOI `10.1111/j.1467-9868.2007.00587.x`. Complete published PDF acquired; its evaluation framework and empirical examples separate calibration from concentration and inspect temporal dependence. URL: https://sites.stat.washington.edu/raftery/Research/PDF/Gneiting2007jrssb.pdf
* GitHub's official workflow-run-attempt and issue-event-type documentation was inspected for attempt identity/conclusions, and close/reopen/review-request event availability. URLs: https://docs.github.com/en/rest/actions/workflow-runs#get-a-workflow-run-attempt and https://docs.github.com/en/rest/using-the-rest-api/issue-event-types

## Initial fully connected mesh ledger

| Peer | Independent proposal sent | Independent proposal received/read | Challenge/reply sent | Peer response consumed | Revision acknowledged |
|---|---|---|---|---|---|
| mesh_observability | yes | yes | yes | yes | eight-window freeze; visible inventory; retrospective/prospective distinction; overlapping pressure predicates |
| mesh_survival | yes | yes | yes | yes | current-cohort sum incomplete; reopen-safe fixed horizon; survival deferred; no interval guarantee |
| mesh_sequence | yes | yes | yes | yes | empirical distribution first; NB/Poisson fitting deferred; mean/median distinct; sparse history exploratory |
| mesh_validation | yes | yes | yes | yes | eight-window protocol; RPS oracle; actual interval mass; no independent-week assumption |
| mesh_integration | yes | yes | yes | yes | minimal collector; immutable forecasts; queue conservation only with full transitions; current-inventory scope |

## Daily amendment mesh ledger

The same daily-horizon, support, quantile-mass and clock cautions were sent individually to all five peers. Received replies are updated as implementation contracts settle; the original all-five proposal/reply obligation is already complete.

| Peer | Daily cautions sent | Daily proposal/correction consumed | Main consequence |
|---|---|---|---|
| mesh_observability | yes | yes | first full shared portfolio day; no synthetic pre-existence zeros; typed history contract |
| mesh_survival | yes | yes | common portfolio start; all-zero support cannot establish a deterministic future; reference-repository skill does not transfer |
| mesh_sequence | yes | yes | next full UTC day; pure forecast API; included PMF mass; current mutable fields excluded |
| mesh_validation | yes | yes | weekly protocol preserved; daily amendment pre-score; tiny support remains inconclusive |
| mesh_integration | yes | yes | light inventory profile; current open zero does not constrain future arrivals |

All five daily responses were consumed. The implementation-phase source contract additionally uses the minimum latest-collection `started_at` across selected repositories as the conservative complete-through cutoff. Source summaries and PR rows must belong to the same retained manifest. Snapshot/issuance time is separate, so stale or midnight-crossing scans cannot invent zero days. Current queue ages are evaluated at source observation time, with scan span and staleness reported. These corrections were accepted in the companion manuscript.

## Implemented comparison and resulting default decision

The predeclared public-reference comparison used `CPMpy/cpmpy`, 78 complete weeks, 52 initial training weeks and 26 chronological targets. Generated results show mean RPS 2.525841 for recent eight and 2.274930 for the expanding comparator: recent is 11.03% worse and fails the 5% promotion screen. Its predictive-median MAE is lower (3.038 versus 3.538), which does not override the distributional primary criterion. This negative result is retained in the companion abstract and tables. The reference result does not validate the user's short-history forecasts.

After that failed promotion, the host retained the expanding empirical reference as the operational default and left recent history as an explicit experimental method. Version 2 requires a method field. Operational expanding history uses the entire supported portfolio frame; the public comparator used only its declared 52--77-week training frame. The manuscript distinguishes those frames and makes no global superiority claim. With five complete user days, the two methods have the same numerical distribution.

The new original-forecast archive records actual bundle creation separately, verifies the declared pre-target chronology and snapshot digest, rebuilds source history and the selected model, and scores only after later complete-through exposure reaches the horizon end. Offline replay bundles are rejected as issuance receipts. Local timestamps remain declarations rather than independently certified evidence of pre-outcome existence.

Independent claim review by the survival node found no material count/survival/causal overclaim. Its correction from ambiguous 'median absolute error' to 'mean absolute error of the predictive median' was accepted. The source node's overlapping current draft/requested-review/age-at-least-seven-days predicates were added with missingness and source-age qualifications.

