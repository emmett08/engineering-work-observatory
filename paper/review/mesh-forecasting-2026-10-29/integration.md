# Integration mesh sent/received ledger

All five peer proposals were consumed and substantively challenged. All outgoing messages below were sent directly through collaboration tools. Initial independent inspection preceded the host's MESH READY message. No production files were edited.

| Peer / host | Direction | Message | Substance and resolution |
|---|---|---|---|
| host | sent | initial independent findings | Reuse store/receipts/snapshots; expensive all-source collector; absent timeline; later snapshots lose earlier per-content receipts. Initial target PR event forecast pending host direction. |
| host | sent | API refinement | Issue event docs cover closed/merged/reopened; repository event stream may replace per-PR timelines; distinctions in permissions and generic Events API. |
| host | received | MESH READY / refocus | User selected team throughput and bottlenecks. Primary next7/14-day count, lightweight PR timestamps, timeline optional, no manual records, no production edits until assigned. |
| observability | sent | PROPOSAL integration v1 | Count7/14 + observed backlog/ages; PR-only profile; immutable forecasts/receipts; event vs acquisition cuts; empirical counts and baseline falsifier; verified API URLs. |
| survival | sent | PROPOSAL integration v1 | Same complete integration proposal. |
| sequence | sent | PROPOSAL integration v1 | Same complete integration proposal. |
| reliability | sent | PROPOSAL integration v1 | Same complete integration proposal. |
| validation | sent | PROPOSAL integration v1 | Same complete integration proposal. |
| survival | received | independent proposal | Direct count primary; secondary age-conditioned AJ backlog contribution; new arrivals need separate treatment; no independent-Bernoulli bands; timeline for first disposition. |
| observability | received | independent proposal | Direct counts naturally include fast new PRs; snapshot-only backlog/review/CI diagnostics; close can reopen; counts are not delivered value; mutable fields cannot be reconstructed. |
| sequence | received | independent proposal | Rolling count baselines; optional regularized Poisson/NB only with temporal gain; no manual inputs; fixed nonoverlap evaluation; freeze knowledge and event cutoffs. |
| reliability | received | independent proposal and response | Empirical prior13 weekly counts, interval/mean/median; queue conservation only coverage audit; no independent queue equilibrium; visible history limitations; stage names must describe observations. |
| validation | received | independent proposal and challenge | Monday UTC primary weeks,14d secondary; RPS/CRPS+MAE; prior8 empirical and simple baselines; provisional gain threshold with sufficient blocks; current inventory not historical completeness. |
| survival | sent | consumption/challenge | Absorbing first-close target misses reopened open PRs; omit AJ minimal core; observed origin-open/new-arrival decomposition later; no independent count bands. |
| observability | sent | consumption/challenge | Collection is asynchronous across pages; show acquisition interval/freshness; fail incomplete required scopes; retrospective merged_at count history is reconstruction rather than knowledge replay. |
| sequence | sent | consumption/challenge | Empirical finite support makes unsmoothed log score unsuitable; use count CRPS/MAE/coverage; plain mean/distribution before new fitted dependencies; distinguish reconstruction. |
| reliability | sent | consumption/challenge |13-week empirical tails lack guarantees; interval labelled empirical; nonoverlap does not establish independence; visible PR population scope and current-only diagnostics. |
| validation | sent | consumption/challenge | Freeze8 vs13 window before scores; fallback4–7 must be explicit; repository creation does not establish truly observed zero history; separate acquired-by and reconstructed folds. |
| survival | received | challenge response / revision | Agrees no AJ core. Adds third origin-closed→reopened contribution category. Incomplete ascending traversal may omit newest PRs; fail/unavailable, never zero. |
| sequence | received | challenge response / revision | Agrees empirical counts/no fitting first. Corrects claim backlog must use separate transition model: direct backlog forecast possible. Current-only diagnostics for v1; exact Monday origins. |
| observability | received | challenge response / revision | Dedup repoID+PRnumber across versions/families; batch timestamps not synchronous; pagination can mutate; profile checks required scopes only; minimal inventory supports reconstructed merge counts. |
| sequence | received | further revision | Discrete RPS primary; MAE of median; expected count separate. Adopt fixed8 prior complete weeks/fallback4–7 and avoid unsmoothed log score. |
| reliability | received | further revision | Withdraws13 in favour fixed8 before scores/fallback4–7. Empirical80 interval has no guarantee. No independence claim; visible scope; stage predicates overlap. |
| sequence | sent | acknowledgement | Accept median MAE/mean expectation and no logscore. Correct own overstrong backlog transition claim; v1 defers direct or joint backlog prediction. |
| survival | sent | acknowledgement | Accept origin-closed/reopen third class, fail incomplete traversal, prospective receipts, omit AJ. |
| observability | sent | acknowledgement | Adopt identity/conflict checking, acquisition interval, relevant-scope completeness; no atomic or exhaustive history claim; no manual manifests. |
| host | sent | consolidated architecture | All5 consumed/replied; direct count core, PR-only profile, immutable forecast, identity, retrospective vs prospective distinction, no new crates. |
| host | sent | temporal edge case | Monday benchmark origins must not backdate Monday00:05 acquisition; explicit issued_at,target_start,target_end; next-complete-week default or exact rolling horizon. Consensus fixed8 weeks/fallback4–7. |

## Refinements accepted into final architecture

1. Direct portfolio counts lead; survival deferred entirely from minimal release.
2. Eight complete prior weeks, explicit four-to-seven fallback, sparse support flagged.
3. Count CRPS/RPS and median MAE; mean reported separately; empirical intervals never guaranteed.
4. No source failure becomes a zero count; required-profile coverage only.
5. Last observed backlog has acquisition interval and lag; it is not atomic true backlog.
6. Duplicate source versions collapse by logical PR identity with conflicts visible.
7. Historical event reconstruction and prospective acquired-by replay are distinct modes.
8. Origin-closed/reopened PRs are a separate merge contribution if decomposition is later implemented.
9. Nonoverlap removes mechanical overlap but does not establish temporal independence.
10. Backlog can be forecast directly; v1 simply defers it rather than claiming impossibility.

Outstanding host choices: exact CLI naming, target start convention, count-model/scoring code ownership, whether legacy full-profile default remains unchanged. These are routine integration choices, not requests for user permission.

## Daily feasibility amendment and implementation exchanges

| Peer / host | Direction | Message | Substance and resolution |
|---|---|---|---|
| host | received | implementation assignment and data finding |5–9day-old repos,33/45PRs all merged. Add throughput collector wrappers/tests; read evolution skill; daily8/min4 exploratory fallback, no weekly scaling. |
| host | sent | public API and environment check | New collect_github_throughput/_cached return existing Collection; full wrappers unchanged; requested compiler path after PATH lackedcargo. |
| host | received | compiler and ownership | Pinned binaries under /workspace/.rustup; root main/lib; observability history projection; sequence model facade. |
| sequence | sent | API and daily proposal | Projection recipe, requiredcoverage/manifest, daily fallback distinct unvalidated target; no scaling/convolution. |
| observability | sent | daily proposal | Short-history feasibility, next-day fallback only, zero current backlog does not imply zero future merges. |
| survival | sent | daily proposal | Same feasibility proposal and request for clock/cold-start challenge. |
| reliability | sent | daily proposal | Same feasibility proposal and request for clock/cold-start challenge. |
| validation | received | daily amendment | Frozen before model scores, next fullUTCday, weekly min4weeks/14d min8weeks; audit acquisition/coverage/missingness. |
| validation | sent | amendment consumption/API | Accepted distinct daily exploratory target; source summary contract and assigned files; no model fitting or manual coverage flags. |
| observability | received | projection ownership and amendment | New history module; require numeric identity and summary; daily fallback agreed; exclude partial creation/current days. |
| observability | sent | detailed source summary | github.throughput_collection same manifest; numericnamespace; started/completed; copied requiredcoverage after validation; deliberate optional omissions. |
| sequence | received | daily revision/challenge | Same empirical model daily, min4; nextfullUTCday; commonstart;14day means four disjoint14day blocks/eightweeks. |
| sequence | sent | acknowledgement | Confirms explicit14day support and no added weekday/stability assumptions or transformations. |
| reliability | received | daily challenge | Targetstart>=issue; no partialbins; launch-phase/weekday/serialdependence limitations; empirical80%quantiles atsmalln may include entire sample mass. |
| survival | received | daily challenge | NextfullUTCday; common portfolio start; first-page staleness means completiontime alone cannot certify complete bins; no scaling. |
| observability | received | summary consumption | Requires newest summary+same raw manifest rows; full collector missing summary gives incomplete, not manual override. |
| observability | sent | summary confirmation | Numericnamespace+same persistence batch; malformedcoverage captured; originals cached-body timestamps excluded. |
| validation | received | independent collector review | Require chronology/state sanity and conflicting duplicateversion detection; arbitrary hash sorting cannot order simultaneously acquired versions. |
| validation | sent | review consumption | Added chronology/future/state and sameid/samenumber conflictingduplicates validation+mocktests. |
| observability | sent | exposure-cut amendment | Conservative trainingcut<=minimum started_at, issuance>=completion; midnightcrossing can otherwise falsely mature bins. |
| survival | sent | acknowledgement | Adopts exposurecut, nextfullday/commonstart/shortsupport, communicates to projectionauthor. |
| reliability | sent | acknowledgement and source contract | Accept empiricalmass/nominalinterval distinctions, support limitations; cheaprequestcost and collectionsummary contract. |
| validation | received | exposure agreement | Rootaccepted minimum STARTED_AT exposurecut, acquired/issuedcut separate; allfive amendmentresponses consumed. |
| host | sent | implementation status |10new+12legacy pass; schema/chronology/duplicates checked; daily amendments consumed; final checks ongoing. |
| validation | sent | audit handoff | Stable files and tests; exact additional validation; rawreturned coveragecount may include identical duplicates; projection dedups. |

All five peers' amendment contributions were consumed and reflected in the revised integration conclusion. No weekly evidence was invented from the daily fallback.

| Peer / host | Direction | Message | Substance and resolution |
|---|---|---|---|
| validation | received | exactduplicate integration defect | Raw coverage records can2 while immutable store has1 identicalPR; projection equality would make idempotent duplicates unavailable. |
| observability | received | duplicatecount repair request | Add unique_pull_requests and duplicate_pull_request_rows to summary while retaining rawcoveragecount; projection verifies consistency. |
| observability | sent | repair accepted | Added summary counts and regression assertions; conflictingduplicates still incomplete. |
| host | sent | repair status | Shared duplicate-count fix; target/statistics unchanged; focused regression+Clippy rerun. |

## Extracted ingestion and command-line verification

| Peer / host | Direction | Message | Substance and resolution |
|---|---|---|---|
| host | received | follow-up assignment | Own new tests/ingestion.rs only; test Full legacy policies, Throughput required scopes/omissions, retained failures and idempotence; report production bugs without editing ingestion.rs. |
| validation | sent | independent ambiguity check | Throughput any-complete check plus blanket allowed omissions can accept conflicting required coverage. |
| validation | received | confirmation | Require exactly one complete scope per required family; projection rejects ambiguity but ingestion incorrectly succeeds. |
| host | sent | bug report/regression | Reported ambiguity and proposed exact-one-complete policy; tests to require rejection while preserving evidence. |
| host | received | repair and extension | Root implemented exact-one-complete; optionally author tests/forecast_cli.rs after ingestion tests. |
| host | sent | ten ingestion tests pass | Legacy Full exceptions, required scopes, optional failures, raw failure preservation, idempotence/conflicting identity; focused Clippy passes. |
| reliability | received | version-2 model amendment | Public recent-window challenger11.03% worse mean RPS; default expanding reference over supported frame; no deployed-frame transfer claim. |
| survival | received | scorer integration | Pending/available/unavailable; all target merges; source binding by archive; strict PMF preserved with float_roundtrip; reject old/missing method. |
| host | sent | fourteen tests complete | Ten ingestion and four CLI tests pass, Clippy/format/diff checks pass; no production edits in follow-up. |
| reliability | sent | version-2 acknowledgement | Notes supersede recent-window proposal with explicit reference/default and no transfer/timestamp certification claims. |
| survival | sent | scorer acknowledgement | CLI verifies source-bound bundle, exact offline replay, repeat pending score without old-byte mutation, short weekly support unavailable. |
