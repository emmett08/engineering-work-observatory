# Automatic team throughput forecasting

`forecast-team` collects GitHub evidence, forecasts merged-PR counts and reports
observed queue pressure. It requires no per-task labels, effort records, AI-use
classification, acceptance rubric or manual coverage attestation. Configure the
repository portfolio and credential once, then run the command through an
existing scheduler.

The endpoint is the number of PRs merged in the selected repositories during a
specified UTC period. A PR counts once, identified by numeric repository ID and
PR number. PRs opened during the target count, as do previously closed PRs that
reopen and merge. A zero open backlog therefore does not imply zero future
throughput. The count is not a measure of delivered functionality, human labour,
productivity or an AI causal effect.

## Run and retain a forecast

Build with the pinned Rust toolchain:

```sh
cargo build --locked --release
./target/release/observatory forecast-team
```

The default scope is `emmett08/eal-mcp` and `emmett08/engarg`. Select another
portfolio with repeated `--repository owner/name` arguments:

```sh
./target/release/observatory --state .team-observatory forecast-team \
  --repository owner/service --repository owner/client --horizon day
```

Supply `GH_TOKEN` through your shell's secret mechanism. Private repositories
need repository access and read access to pull requests. Public collection also
works without authentication, subject to the smaller API allowance. `--token-env`
selects a different environment-variable name; the token itself is never a CLI
argument. `--api-base` selects an alternative GitHub API base.

Each run performs these operations:

1. Collect repository metadata and every PR-list page, using conditional caching.
2. Retain raw responses, source versions, acquisition times and automatic coverage.
3. Freeze a snapshot, derive its supported count history and issue a forecast.
4. Archive the original bundle under `STATE/forecasts/`; optional `--output FILE`
   exports the same bytes to another new path.
5. Evaluate older archived forecasts against the new source cut and append a
   score report under `STATE/forecast-scores/`.

Standard output contains `forecast`, `current_pressure` and
`previous_forecast_scores`. A bundle retains its original snapshot, derived
history, model/version, method, training bins, distribution and declared issue
time. Existing forecast and score files are not overwritten. Preserve the state
directory and bundles for reproducibility.

The command's receipt records its locally declared creation time. Content hashes
bind retained bytes; they are not an independently trusted timestamp certificate.

## Targets, history and the reference model

| `--horizon` | Target | Minimum usable history |
|---|---|---|
| `day` (default) | Next full UTC day | Four complete days |
| `week` | Next full Monday-based UTC week | Four complete weeks |
| `fortnight` | Fourteen days beginning at the next Monday boundary | Four complete, disjoint 14-day bins: eight weeks |

At an exact midnight or Monday boundary, a full period can start immediately.
Otherwise the target starts at the next corresponding boundary. Inspect
`target_start` and `target_end`; intervals include the start and exclude the end.
The application does not multiply daily predictions into weekly totals or
convolve days under an independence assumption.

The operational default is `method: expanding_empirical`, model
`observatory.empirical-merge-count/2`. It assigns probability to observed counts
in proportion to their frequencies across **all supported complete bins** in
the selected portfolio's history. It has no fitted arrival-rate, hazard or
queue-equilibrium parameter. Historical periods still have to be relevant to
the future for the distribution to predict well. Launch bursts, policy changes,
seasonality and changing repository scope can undermine that assumption.

The portfolio's history begins when its newest selected repository was created.
Partial initial periods and the unfinished current period are excluded; days
before a repository existed are not filled with zeros. Insufficient support
returns `status: unavailable`, a reason and no predictive distribution.

An available forecast reports the empirical mass function, mean, inverse-CDF
median, `quantile_10`, `quantile_90` and `quantile_range_mass`. The last field gives
the historical probability mass actually included by the inclusive quantile
endpoints. With few observations this may be 100%, despite the 10th/90th percentile
labels. Neither the range nor the historical maximum guarantees future coverage.
The default remains marked `unvalidated_operational_reference` until separate
evidence supports stronger claims.

`recent_empirical` remains an experimental comparison: at most eight daily or
weekly bins, or four fortnightly bins. The library exposes it through
`forecast_with_method`; historical comparison reports retain it. It is not the
operational CLI default.

## What current queue pressure means

The `history.current_pressure` object in a bundle, also printed by `forecast-team`,
contains current observations from the same PR inventory:

| Field | Interpretation |
|---|---|
| `open_pull_requests` | PRs last observed open |
| `open_draft_pull_requests` | Open PRs whose captured `draft` flag is true |
| `open_review_requested_pull_requests` | Open PRs with an observed outstanding reviewer or team request |
| `open_at_least_7_days_pull_requests` | Open PRs aged at least seven elapsed days at acquisition receipt |
| `open_age_days` | Ages of the last-observed open PRs, measured at acquisition receipt |
| `oldest_collection_age_seconds` | Staleness of the oldest selected scan start at issue time |

These predicates may overlap. A draft can also have a review request and be older
than seven days. They are not mutually exclusive process stages. A review request
does not establish that work is blocked, that no review has occurred, or that
reviewer capacity is the cause of slow delivery. The seven-day threshold is a
fixed descriptive convention, not a learned failure threshold.

Missing optional fields yield unavailable counters, rather than fabricated zero
counts. A known empty open inventory yields zero counters. A positive observed
reviewer or team request is enough to establish that PR's request predicate;
absence requires both request arrays to be observed empty. Source IDs and scan
intervals preserve the provenance of these observations.

The lightweight profile does not collect review submissions or CI, so full
`review_status` and `ci_status` remain `not_collected`. Draft and request fields
are used only for these current diagnostics. They are not backdated into
historical forecast features. This release reports queue pressure; it does not
identify a causal bottleneck or prescribe a staffing change.

## Observation boundaries and missing data

The projection distinguishes three clocks:

- `snapshot_cut` is the acquired-knowledge and forecast-issuance cut.
- `observed_cut`, exposed as forecast `data_cutoff`, is the earliest start of the
  latest selected repository scans. Only bins ending by this conservative common
  boundary can be complete training observations.
- Each repository retains its collection start and completion times. The combined
  inventory is assembled over that interval, not observed simultaneously.

A scan can cross midnight after an earlier page was fetched. Using the scan's
completion as proof of coverage through that midnight would invent information
about intervening merges. The common scan-start rule avoids that error. Taking a
new snapshot of an old collection likewise cannot manufacture later zero days.

Every selected repository needs an automatic throughput-profile summary with
complete metadata and PR-inventory retrieval. Source rows must match that
collection's manifest. Exact repeated PR rows are deduplicated; contradictory
equal-cut versions, missing pages, request limits, inaccessible repositories or
invalid event chronology prevent a complete forecast. A latest failed collection
does not silently fall back to an earlier successful inventory. Errors and partial
responses remain retained evidence. A command-level collection failure returns
nonzero rather than issuing a complete forecast from a reduced portfolio.

Successful pagination means the currently accessible inventory was retrieved.
It does not recover deleted or inaccessible historical objects. GitHub's
[PR API](https://docs.github.com/en/rest/pulls/pulls) supplies the timestamps and
current fields used here; historical close/reopen paths require additional
[timeline events](https://docs.github.com/en/rest/issues/timeline).

## Score original forecasts automatically or offline

Running `forecast-team` again evaluates earlier bundles in the same state's
`forecasts/` directory. Results distinguish pending, available and unavailable
scores, and preserve validation errors. A pending target has not yet reached its
end under the later common observation boundary. Missing coverage is unavailable,
not a zero outcome. Score reports are new artifacts; the original forecast is
unchanged.

For explicit offline scoring, first collect later evidence and freeze it:

```sh
cargo run --locked -- collect-github --profile throughput \
  --repository owner/service --repository owner/client
cargo run --locked -- snapshot --output reports/later-snapshot.json
cargo run --locked -- score-forecast \
  .observatory/forecasts/FORECAST.json reports/later-snapshot.json \
  --output reports/forecast-score.json
```

Use the same state directory as the original run, or pass its `--state` path.
The scorer validates the original bundle, verifies the embedded snapshot and
recomputes its history and method-specific forecast. It rejects replay output as
an original issued bundle. The later numeric repository portfolio must match,
and its complete exposure must cover the entire target before an outcome is
scored. Original model/version records are never relabelled as a newer model.

Available scores include ranked probability score (`rps`, lower is better),
absolute error of the empirical median and whether the outcome fell inside the
reported quantile range. One score, or a few daily scores, does not demonstrate
predictive superiority. Missing and pending cases remain visible alongside scored
cases. Scoring does not tune the issued model.

## Replay and retrospective comparison

Offline forecasting from a retained snapshot is available without another API
collection:

```sh
cargo run --locked -- forecast-snapshot reports/source-snapshot.json \
  --horizon day --output reports/replay.json
```

This produces a replay, not evidence that a forecast was issued at the snapshot's
declared time. Historical count comparison is also explicit:

```sh
cargo run --locked -- forecast-snapshot reports/reference-snapshot.json \
  --horizon week --backtest --history-bins 78 --initial-training-bins 52 \
  --output reports/reference-comparison.json
```

The bounded example evaluates the final 26 weekly bins after an initial 52-bin
training block within a predeclared 78-bin frame. Its expanding comparator uses
the preceding bins **within that bounded frame**. The operational expanding
reference instead uses all supported complete bins. These are different training
frames and must not be described as identical fitted forecasts.

Reconstructed history uses present inventory timestamps to count past merges.
It does not show that every source record was available at each simulated past
origin. Mutable current draft, diff, head, review-request and queue fields are
excluded. Genuine prospective evaluation requires original forecasts retained
before their outcomes, followed by later evidence and scoring.

## Initial evidence and limits

At the recorded 29 October 2026 user-repository source cut, the common portfolio
had five complete daily counts: `17, 11, 13, 12, 8`. Their empirical mean is 12.2
and median is 12. The inclusive 10th–90th percentile range is 8–17 and contains
all five observed counts; its future coverage is unknown. There were no complete
common weekly bins, so weekly and fortnightly forecasts were unavailable.

A separate, predeclared `cpmpy/cpmpy` reference exercise evaluated 26 future weeks
within the 78-week frame. Mean RPS was 2.525841 for the recent-window challenger
and 2.274930 for the expanding reference: the challenger was about 11% worse and
failed its promotion screen. It remains experimental. The operational reference
is retained as the simpler baseline, without claiming validated accuracy on the
user's repositories or automatic transfer from the reference repository.

The [companion paper and retained results](../paper/forecasting/README.md) document
that exercise. The separate labour-identification study remains available through
the [measurement protocol](measurement-protocol.md); its independent labour and
quality observations are not replaced by automatic PR counts.
