# Automatic observability and forecast contract

Base audited: `87372e5a140dff1df01ba68a25939c60e63031c1`.
Scope: no repository edits, no external posts, zero recurring human classification.

## Decision

Lead with the distribution of merged-PR counts in the next complete 7-day UTC week across a frozen repository portfolio; report 14-day counts separately. Define the unit as a merged pull request, not a requirement or a quantity of valuable functionality. The observed count includes PRs created during the forecast window, previously open PRs, and previously closed PRs reopened and merged during it. A direct aggregate count model includes all three without requiring a model of each path.

Use the validation node's frozen empirical distribution of the previous eight complete weekly counts; use all available if four to seven exist and show support. Below four, mark insufficient support. Compare against the predeclared baselines on identical rolling origins. Predictive usefulness has not yet been demonstrated. The recent-window transport assumption remains real and must be tested; avoiding a parametric count law does not eliminate it. Historical empirical quantiles do not have guaranteed future coverage.

Use automatic snapshots for current queue pressure. Open count, age distribution, absence of a submitted nonauthor review, and current-head CI state are separate observed predicates. They can overlap; do not invent a serial process stage or infer that a reviewer or CI system causally limits throughput. With only current PR metadata, the first release can report open count and ages. Review and CI diagnosis should be enabled only when their own collectors are available and fresh.

## Why the existing measurement model should not lead this workflow

The paper's original labour/output model requires registered requirements, effort allocations, acceptance rubrics and follow-up assessments. These are scientifically explicit measurements, but cannot be manufactured from GitHub metadata. They conflict with the user's no-manual-work constraint if made prerequisites.

Two complete work histories can share identical commits, PRs, CI results and even AI call traces while differing in unlogged thinking, meetings, parallel activity and review attention. They therefore have different true labour. The same observed tests and lack of reported incidents can coexist with different hidden defects. The same association between agent use and elapsed delivery can arise from easier task assignment, different policy or a treatment effect. Automatic traces do not identify person-hours, retained requirement quality, value, productivity or AI causal effects. Preserve these distinctions rather than silently renaming commit or token counts as those quantities.

## Executable source-to-field map

| Observation | Existing source | Automatic use | Constraint |
|---|---|---|---|
| Repository ID and PR number | `src/collectors/github.rs:540` and PR list at 548 | Stable count identity | Freeze numeric repository scope; names can change |
| `created_at`, `merged_at`, state | PR list and detail at 548–588 | Merge-count history; current open count/age | Present inventory, not archive of deleted objects |
| `closed_at` | PR payload | Current terminal metadata | Cannot reconstruct every close/reopen event |
| Draft, additions, deletions, changed files, head SHA | Detail at 572 | Future snapshot predictors | Mutable; never attach current values to past cuts |
| Submitted reviews | Reviews loop at 590 | First submitted nonauthor review, observed review status | `PENDING` lacks `submitted_at`; human-looking account is not proof of human work |
| Conversation comments | Repository endpoint at 629 | Activity age/count observed at cut | Edited/deleted history not fully recovered |
| Workflow attempts/jobs | 634–681 | Recorded job timing and outcomes | Attempt cap 50; missing/expired runs/logs separate |
| Check runs/statuses | 685–699 | Current captured head's CI observation | Missing source is not passing; neutral/skipped != success by default |
| Git patches/refs | `src/collectors/git.rs` | Optional code-change descriptors | Unneeded for core count forecast; timestamps do not measure labour |
| Content versions/receipt times | `src/collectors/mod.rs`, `src/store.rs:243` | Reproducible acquired-by cuts | Several hashes can represent one PR; latest saved snapshot has latest receipt per version |
| Collection completeness | Coverage entries and raw manifest | Availability gates | Complete page traversal is not complete historical event coverage |
| EngArg calls | `src/importers/engarg.rs` | Optional future automatically attributed trace features | Current manifest requires operator session/attempt identities; omit from no-manual core |
| EAL runtime rows | `src/importers/eal.rs` | Optional source observations | No general automatic PR linkage in current contract |

The collector explicitly marks `github.history` not collected at github.rs:705. There is no PR timeline/webhook/deployment/incident collector. The throughput-only collector should request repository identity and all PR inventory pages, avoiding all-commit/check/job work. Failure of unrelated CI scopes must not disable a count forecast; failure of a required repository or PR scope must.

## Two distinct evaluation modes

**Retrospective reconstruction:** today's accessible PR inventory supplies `merged_at` dates. Build past count bins from currently visible objects, exclude future event dates at each origin, and describe the result as retrospective count reconstruction. It does not establish that every record was available at the historical origin, and historical deleted objects can be absent even after successful pagination.

**Prospective replay:** archive each automated acquisition, cut and forecast. A feature must have been acquired by its issue time; an old event timestamp received later does not make it past knowledge. A fitted model at historical cut C can use only labels available by C. Saved prior snapshots or the full receipt table are needed; the most recent snapshot alone may discard earlier receipt times for repeated identical content.

This distinction is essential: a strict acquisition-time replay of a brand-new backfill has no historic predictor availability. It must not be made to pass by assigning old `merged_at` timestamps as receipt times.

## Minimal contracts

Portfolio definition: schema/version, numeric repository IDs, endpoint (`merged_pull_request`), UTC week boundary, horizon, scope effective time. Changes in repositories create forecast-population drift and are retained explicitly.

Acquisition: collection ID, start/end timestamps, source-response timestamps and hashes, pagination/request-cap status, visibility/permission state, source API version. The existing persistence timestamp is at collection completion, safely conservative for availability. It does not make pages a simultaneous snapshot. Show freshness or last-observed status for queue data.

Count projection: unique key `(repository_id, pr_number)`; prefer one declared payload family; resolve newer observed versions by receipt provenance, reject contradictory equal-cut variants rather than selecting by hash order. A record occurring in both list/detail families or several versions is one PR. A missing repository or incomplete PR list does not create zero-count weeks. Ascending-created pagination plus request/page caps can preferentially omit recent PRs.

Forecast: forecast ID, issued_at, repository-scope version, mode, training cutoff/bins, horizon start/end, source snapshot/hash, model and feature-schema versions, empirical distribution/support, mean/quantiles, availability and collection freshness. No requirement/task, human-effort, assessment, AI-label or manual task_link record is needed.

Outcome: forecast ID, label collection/cut, merged count within the exact horizon, deduplication rule, label status and coverage. Incomplete/immature/unavailable labels stay in the denominator of reported coverage, though they are not scored as known outcomes. Later source corrections create a new label version; they do not overwrite old forecasts.

Optional queue predicate rows: same observed cut, PR identity, last seen state/head, observation interval, freshness, missingness, relevant source IDs. Review/CI predicates remain independent flags. Count predicates and their overlaps when useful; do not force mutually exclusive stages.

## Rejected shortcuts and tests

- Do not treat unmerged closure as final abandonment or independent censoring when forecasting actual merge counts. A close→reopen→merge path within a week is a merge.
- Do not label the sum of current-open PR merge probabilities total throughput: new PRs and previously closed PRs can merge. Summing marginal probabilities gives an expected count without independence, but independent Bernoulli intervals ignore shared team dependence.
- Do not reconstruct historical draft/head/files/queue state from final current payloads. Mutating a future payload must not change an archived past forecast.
- Do not infer zero AI use from absent traces; do not require manual AI/commit labels.
- Do not turn missing pages, access loss or absent checks into a zero count, no failure or a passing state.
- Use complete nonoverlapping weekly origins for primary evaluation; report overlapping 14-day predictions with appropriate time dependence. Promotion thresholds are engineering choices, not laws of nature.
- Suppress or qualify forecast claims under incomplete required scope, scope changes, inadequate history or evident population drift. No holdout gain is a substantive negative result.

## Mesh exchange ledger

| Peer | Proposal sent | Proposal consumed | Substantive challenge sent | Reply consumed / revision acknowledged |
|---|---|---|---|---|
| survival | Yes | Yes | First-close competing-risk probability misses reopened merges; require actual merge endpoint | Accepted; dropped AJ v1; supplied closed-at-origin/reopened term; accepted cap-related missingness gate |
| sequence | Yes | Yes | Today's backfill cannot support historical receipt cuts; defer complex count law | Accepted two-mode R/P distinction and deferred Poisson/NB; accepted its historical-backlog challenge |
| reliability | Yes | Yes | Align frozen window with validation; empirical intervals have coarse bounded support | Accepted eight-week common baseline and unguaranteed coverage; accepted its overlap/noncausal queue-state challenge |
| validation | Yes | Yes | Reconstruction vs replay; missing label denominator; threshold is engineering choice | Accepted all; incorporated repository-scope drift, empirical interval support, observed-pressure-only output |
| integration | Yes | Yes | Stable-ID projection, tied variants, relevant-scope gating, non-atomic pages | Accepted; incorporated collection interval/freshness and current-inventory limitation; no manual manifests |

All five peer proposals were consumed and each received a substantive reply. No peer work was read before the independent proposal was written. The independent proposal preceded the user's team-throughput clarification and is retained separately.

## Primary source anchors

- GitHub PR API: https://docs.github.com/en/rest/pulls/pulls — inventory filters, pagination and PR payload.
- GitHub reviews API: https://docs.github.com/en/rest/pulls/reviews — submitted review timestamps and PENDING semantics.
- GitHub timeline API: https://docs.github.com/en/rest/issues/timeline — recorded issue/PR timeline events.
- GitHub event types: https://docs.github.com/en/rest/using-the-rest-api/issue-event-types — closed/reopened/merged event semantics.
- GitHub webhook troubleshooting: https://docs.github.com/en/webhooks/testing-and-troubleshooting-webhooks/troubleshooting-webhooks — events may arrive out of order and after delays.
- GitHub webhook redelivery: https://docs.github.com/en/webhooks/testing-and-troubleshooting-webhooks/redelivering-webhooks — three-day delivery window and no automatic retry of failures.

Webhook capture, if later added, therefore needs durable receipts, deduplication and automated gap recovery. It is not needed for initial weekly merge counts.

## Cold-start amendment and implementation

The automatic source inventory subsequently established that the user's two repositories were only five to nine days old. Before observing predictive scores, the host introduced an explicitly exploratory daily model: previous eight complete UTC days, minimum four. Weekly and fortnightly models retain their original support requirements; there is no daily-to-weekly scaling or independence convolution.

The amendment was sent to all five peers. All five consumed it and responded. Survival emphasised common portfolio exposure and early page receipts; sequence and validation required a future full UTC target day, a separate issue cut and a conservative common exposure bound; reliability required actual empirical mass/support rather than claiming exact 80% coverage; integration supplied a versioned automatic collection summary, acquisition spans and exact-duplicate accounting. All were accepted.

Implementation ownership: new `src/forecast/history.rs` and `tests/throughput_history.rs`. The projection requires automatic throughput collection summaries, selected numeric repository identities, complete required-source coverage and source rows from the same manifest receipt. It rejects a latest failed scan rather than falling back silently. Manual study coverage does not satisfy this contract.

There are three distinct clocks: `snapshot_cut` is issuance/knowledge time, `observed_cut` is the earliest selected latest scan START time (conservative complete-through bound), and each repository retains its scan start/end. Source merges found after the common boundary remain visible but cannot enter later complete historical bins. Thus a stale snapshot or a crawl spanning midnight cannot manufacture zero-count days. Current open ages are measured at acquisition receipt, with collection staleness reported separately.

Automatic exact-duplicate row counts reconcile raw pagination counts with unique PR identities. Equal-cut contradictory variants remain unavailable. Metadata conflicts in an unselected repository do not invalidate the requested portfolio. The forecast unit never requires a human task, AI, effort or assessment label.
