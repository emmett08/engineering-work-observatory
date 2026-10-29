# Mesh exchange ledger: sequence node

All peer sends occurred after root's MESH READY. Messages below are faithful substantive summaries, not verbatim transcripts; original delivered messages remain in the collaboration record. All five independent proposals were consumed and challenged before this node finalized. Shared sources and peer agreement are not independent empirical validation.

| Peer | Sent proposal | Received proposal consumed | Sent challenge/reply | Received challenge/reply consumed | Revision/acknowledgment |
|---|---|---|---|---|---|
| observability | v1 direct seven-day portfolio merges; baseline-first, optional NB; historical-state caution | Primary7/14 merge counts, direct empirical forecast, prospective queue snapshots; fixed-horizon merge allows reopens | Required true historical eligibility for backlog; common UTC cutoffs; accepted source visibility caveat | Present merged_at does not prove known at historical cutoff; current snapshots keep latest receipt; deleted/access-lost records remain absent; defer NB | Explicit R reconstruction / P prospective modes; defer NB. Peer ACKed and accepted no historical backlog from closed_at; added revived previouslyclosed PR term |
| survival | v1 direct count forecast; survival only if direct score gain; no independent Bernoulli intervals | sum q_i gives expected existing-backlog merges without independence; arrivals add extra term; initial AJ/first-disposition suggestion | Direct aggregate absorbs arrival/dependence outcome; historical reopening prevents first-disposition substitution; support direct count reference | Peer withdrew AJ: first-disposition merge differs from actual horizon merge after reopening; direct model preferred; NB mean gain cannot hide tails | Accepted no AJ/NB first implementation; confirmed actual merge target; peer accepted first-disposition correction |
| reliability | v1 direct count; observed queue pressure; event coverage; optional NB challenger | Prior13-week empirical PMF, complete-week count, queue conservation audit, CI attempt distinction | Mean vs median distinction; primary RPS;26 heldoutweeks aspiration too demanding; agree common pre-frozen8 window rather than shopping | Poisson/NB not minimal; CI no-attempt !=pass; serial dependence remains after nonoverlap; adopted8weeks and withdrew13 | Deferred fitting; accepted explicit short-history support, CI first-attempt provenance, no causal bottleneck claim |
| validation | v1 direct count and delayed-label prequential; RPS candidate; nonoverlap origins | Frozen Monday00 next7d,14 secondary; prior8 empirical (4–7 allprior); zero/priorweek/Poisson4 benchmarks; RPS primary | Accepted fixed family/window; challenged confidence interval on few dependent blocks; MAE median; sparse quantile resolution | Empirical primary; NB needs data; historical currentqueue unavailable; freeze candidate hyperparameters; reconstruction distinct | Adopted all frozen choices, defer NB; sent further14d direct paired-block warning and aggregate-before-fit cross-repo rule |
| integration | v1 count target; lightweight feasible source; strict saved cutoff; accounting separate | New minimal throughput collector metadata+allPRlists; prepare→forecast→score; immutable predictions; no deep CI needed | Corrected claim backlog forecast requires separate arrivals: direct backlog target possible, but defer v1; exact origins; history modes | Empirical zero support causes infinite logscore; retain CRPS+MAE+coverage, sparse flags; minimalprofile viable | Accepted CRPS, median/mean distinction,8weeks. Peer ACKed direct-backlog correction and deferred feature-rich forecasts |

## Complete exchange inventory

| Order within peer thread | Direction | Peer | Content consumed or delivered |
|---|---|---|---|
| 1 | sent | observability | Structured PROPOSAL v1 with target,data,assumptions,algorithm,falsifier,sources |
| 1 | received | observability | Structured independent PROPOSAL and request for forthcoming challenge |
| 2 | received | observability | Historical acquisition-time challenge, source deletion and permission caveat, request explicit reconstruction mode |
| 2 | sent | observability | CONSUMED/REVISE v2: separate retrospective/prospective; challenge historical-backlog eligibility |
| 3 | received | observability | ACK v2; accepted eligibility challenge; previouslyclosed/reopened merge contribution correction |
| 1 | received | survival | Structured independent PROPOSAL direct count primary/AJ backlog diagnostic; question directmodel absorbing arrivals |
| 1 | sent | survival | Structured own PROPOSAL plus CONSUMED response; endorsed direct aggregation and challenged firstdisposition data |
| 2 | received | survival | CONSUMED; first-disposition correction; withdraw AJ; challenge NB tails and retrospective features |
| 2 | sent | survival | CONSUMED correction/REVISE: direct empirical, no AJ/NBv1; actual merge vs firstdisposition distinction |
| 1 | sent | reliability | Structured PROPOSAL v1 and request challenge throughput/CI distinction |
| 1 | received | reliability | Structured PROPOSAL13week empirical + RESPONSE rejecting NBfirst/CI attempt provenance |
| 2 | sent | reliability | CONSUMED/REVISE: defer NB;RPS,meanmedian; challenge26week requirement, agree frozen window |
| 2 | received | reliability | REVISION/ACK:8week adopted,13withdrawn;26 aspirational; sparse flags and dependence confirmed |
| 1 | sent | validation | Structured PROPOSAL v1; future origins/delayedlabels; request score/interval challenge |
| 1 | received | validation | Frozen PROPOSAL8weeks/RPS/comparators plus CHALLENGE NB, uncertainty, availability |
| 2 | sent | validation | CONSUMED/REVISE adopted frozen design; challenged tinydependentblock uncertainty; explicit median |
| 3 | sent | validation |14d independent-convolution challenge; directhistoricalpairs and portfolioaggregate rule; newest primary comparator |
| 1 | sent | integration | Structured PROPOSAL v1; narrow forecast contract and no retrospective mutable fields |
| 1 | received | integration | Structured PROPOSAL throughputprofile, purepipeline, savedknowledgecut caveat |
| 2 | sent | integration | CONSUMED; lightweightprofile; challenged necessity of mechanistic backlog decomposition; R/P modes |
| 2 | received | integration | CONSUMED/challenge empiricalsupport vs logscore; sparse flags, minimalprofile, R/P modes |
| 3 | sent | integration | CONSUMED adopted RPS and meanmedian split, empirical uncertainty language, frozen8weeks |
| 3 | received | integration | ACK v2; direct backlog alternative accepted, v1 remains deferred |

Root received independent initial finding before MESH READY; subsequent root updates reported user-target adaptation, all-five consumption, selected count method and 14-day dependence warning. No external messages or repository modifications occurred.

## Implementation and daily-horizon amendment

Root subsequently authorised new forecast core files and tests. The user's selected repositories were only five to nine days old; no weekly model could meet the frozen minimum. Before model scores were inspected, all five peers received the separate operational one-day proposal: eight previous complete UTC days, minimum four; no day-to-week scaling or convolution; still unvalidated. Every response was consumed:

| Peer | Response consumed | Change or acknowledgment |
|---|---|---|
| observability | Accepted daily rule; required source exposure separate from snapshot issuance; asynchronous repos can have valid later events | Mandatory snapshot_cut for issue and observed_cut=min repository scan start for complete bins; later retained events excluded from training, not falsely rejected; final explicit daily ACK |
| survival | Accepted; cold-start/weekday/launch drift remains; all-zero PMF does not prove future zero | Sparse unvalidated output; empirical support described as historical; current open zero does not constrain future merges |
| reliability | Accepted; quantile endpoints at N<=8 can enclose100% of empirical mass, not80% | Actual enclosed mass included; median distinct from mean; future coverage not guaranteed |
| validation | Accepted; independently checked RPS; required future calendar targets and no stale false zeros | Exact-midnight target may equal issue; otherwise next boundary; fixed-window benchmark API added before scores; expanding empirical primary comparator included |
| integration | Accepted;14-day minimum means four disjoint14-day periods=eight weeks | No extra collector sources or manually supplied annotations; source projection integrates lightweight automatic profile |

Implementation owned here: new `src/forecast.rs`, `src/forecast/calendar.rs`, `src/forecast/distribution.rs`, `src/forecast/poisson.rs`, `src/forecast/backtest.rs`, and `tests/forecast.rs`. History projection and CLI/exports remain peer/root owned. No commits made. Focused forecast tests:13 pass. `cargo clippy --locked --lib -- -D warnings`:pass. Initial fixture-only stale-date assertion and floating mean accumulation issue were corrected before reporting success; count sums now use u128 before f64 division.

## Failed-promotion decision and v2 method exchange

After the host's frozen 26-week comparison reported recent-window RPS about11.03% worse than the predeclared expanding reference, root instructed retaining expanding as operational default and recent as explicit challenger. No benchmark, parameter or test outcome was changed. Sequence sent exact v2 method/API changes to survival, validation, reliability, observability and root. Survival accepted canonical method-aware validation via shared training_count helper; validation accepted preservation of old artifacts and explicit v1 rejection; reliability and observability requested exact metadata for paper/docs and received it. The new default is the retained reference, not a claim of universally superior or transferable prediction. Core verification:15 tests and library Clippy pass.

A serialization issue was found independently by scorer tests and anticipated in sequence's review: serde_json's default float parser could shift legitimate saved PMF values by1ulp, defeating exact canonical equality. Root authorised the existing float_roundtrip feature to preserve exact comparisons; no arbitrary tolerance or model reinterpretation was introduced.

## Three-ranked-alternatives deliverable

Root requested preserving the original three-ranked research deliverable in `paper/review/ranked-automatic-model-alternatives.md`. Sequence proposed:1 direct empirical counts,2 conditional count dynamics,3 landmark event history with arrival/reopening contributions. Reliability consumed and endorsed ranking, challenged mechanistic queues as unjustified without observed service/capacity. Validation consumed and endorsed, required fresh untouched evaluation for2/3, calendar-level support rather than PRcounts, proper-score/calibration evaluation, historical state availability and dependence-safe aggregation. These revisions are incorporated. Both also independently confirmed that current outputs predict throughput only and do not forecast future bottlenecks. Survival was sent the exact actual-merge formula over all existing unmerged PRs plus future arrivals for final review.

Survival consumed the complete ranked report and found no blocker. Its three precision challenges were accepted: actual merge event uses explicit half-open [t,t+h), block evaluation tests but does not construct dependence-aware intervals, and complete landmark follow-up does not eliminate cohort selection/temporal transport. Final ranked report is approximately1,480words, within the requested1,000–1,600range.
