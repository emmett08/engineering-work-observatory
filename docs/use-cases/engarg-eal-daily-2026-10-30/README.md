# Applying the model to engarg and eal-mcp

On 30 October 2026, the operational model was applied to a practical question:
what merged-PR count can the maintainer use as a provisional reference for the
next full day across `emmett08/engarg` and `emmett08/eal-mcp`? A second question
was whether the same evidence supports planning the next full week.

The daily forecast for **31 October 2026 is 11 merged PRs at the median**, with
empirical 10th–90th percentile endpoints of **5–17**. The weekly forecast is
**unavailable**. The model remains an unvalidated reference: the ranges
reconstructed at two earlier origins both missed their outcomes.
Use it to establish a forecast that can later be scored; it does not justify
promising eleven merges or assigning a weekly delivery target.

## Scope and source evidence

The outcome is the combined number of PRs merged in the two repositories during
the half-open interval **31 October 00:00 to 1 November 00:00 UTC**. PRs arriving
during that day count too. PR size and delivered functionality are not weighted.
The portfolio and the default `expanding_empirical` method were selected before
the fresh collection; the results did not select or tune the method.

The unchanged application at revision
`f4daf8a5b93238ef297de8c6f764d6a9126c44fe` collected public repository metadata
and every PR-list page using its throughput profile. Both selected repositories
had complete required-source coverage: 79 unique visible PRs, 78 merged and one
open. Collection ran from 11:38:15.986 to 11:38:22.815 UTC on 30 October.
The conservative common data cutoff is the earliest scan start,
11:38:15.986 UTC; the frozen snapshot cut is 11:38:22.846 UTC.
Complete retrieval concerns the currently accessible inventory and cannot recover
deleted or inaccessible objects.

Common history starts at engarg's creation on 23 October at 17:10:11 UTC.
Its partial first day and the unfinished 30 October period are excluded.
The six complete training days are:

| UTC day in October 2026 | eal-mcp merges | engarg merges | Portfolio merges |
|---|---:|---:|---:|
| 24 | 6 | 11 | 17 |
| 25 | 4 | 7 | 11 |
| 26 | 1 | 12 | 13 |
| 27 | 6 | 6 | 12 |
| 28 | 2 | 6 | 8 |
| 29 | 2 | 3 | 5 |

These are 66 merges within the common training frame; the 78 merged PRs in the
full inventory also include earlier periods. [daily-counts.csv](daily-counts.csv)
retains the complete dates and counts.

## Forecast and its immediate use

Model `observatory.empirical-merge-count/2` assigns probability 1/6 to each
observed daily count: 5, 8, 11, 12, 13 and 17. Its mean is 11 and its
inverse-CDF median is 11. The inclusive 5–17 quantile range contains **100% of
the training distribution**, not a demonstrated 80% of future outcomes.
The original daily bundle was created at 11:38:22.992 UTC on 30 October, before
its target began. Local receipt times are declarations, without independent
trusted-time certification.

The same snapshot supplies **zero complete Monday-based weeks**, where four
are required. The replay for 2–9 November therefore has no distribution.
Multiplying eleven by seven would bypass the model's history requirement.

At collection, the observed queue had one open PR,
[engarg #46](https://github.com/emmett08/engarg/pull/46), about 27 minutes old.
It was not a draft; both reviewer-request arrays were empty. Draft,
requested-review and at-least-seven-days counts were all zero. These are captured
queue observations, not forecasts. Review submissions and CI were not collected,
so they do not establish readiness or a cause of delay. One open PR also does
not cap tomorrow's merge count, because new PRs can arrive and merge.

The practical action is to retain this daily forecast, inspect the identified
open PR through the normal review process, and collect later evidence before
scoring the target. The observed counts have recently fallen; neither their
future relevance nor a stable launch-period pattern is established. No human
effort, productivity, AI benefit or staffing requirement is inferred.

## Adverse checks and pending evaluation

The fixed default historical comparison reconstructs two eligible daily origins
from today's visible merge timestamps. For the expanding empirical reference:

| Reconstructed target | Prior training days | Median | Quantile endpoints | Actual merges | RPS |
|---|---:|---:|---:|---:|---:|
| 28 October | 4 | 12 | 11–17 | 8 | 4.0625 |
| 29 October | 5 | 12 | 8–17 | 5 | 5.6000 |

Mean median absolute error is 5.5 PRs, mean ranked probability score (RPS; lower
is better) is 4.83125, and range hits are 0/2. The previous-day point forecast
has mean RPS 3.5 in the same comparison. Every comparator is retained in
[daily-reconstruction.json](daily-reconstruction.json); none is promoted from
two retrospective, potentially dependent origins. These checks expose poor
recent agreement and do not establish historical source availability or
prospective predictive skill.

The [original 29 October forecast](../../../paper/forecasting/examples/README.md)
was also scored against the new snapshot. It remains **pending**, with no
outcome or loss: its 30 October target ends at 31 October 00:00 UTC, beyond the
new common data cutoff. [prior-forecast-scores.json](prior-forecast-scores.json)
retains that result. The new 31 October forecast likewise cannot be scored until
a successful collection covers its entire target through 1 November 00:00 UTC.
One or two completed scores would still be insufficient to validate the model.

## Reproduce and continue

From the repository root, build with the pinned toolchain and reproduce the
retained analysis offline:

```sh
cargo build --locked --release
python3 docs/use-cases/engarg-eal-daily-2026-10-30/reproduce.py
```

The script checks file hashes, reconstructs the daily counts from embedded
source payloads, replays both horizons and the historical comparison, verifies
the original daily receipt, and recomputes the earlier forecast's pending score.
It uses only the Python standard library and the application binary, makes no
network requests, and writes temporary files under `work/`.
[manifest.json](manifest.json) binds the source revision, snapshot and artifacts.
[issued-day.json.gz](issued-day.json.gz) preserves the original command bundle
byte for byte when decompressed, including its public source payloads.
[week-replay.json](week-replay.json) is explicitly an offline replay.

The original collection command was:

```sh
./target/release/observatory --state work/observatory-use-case-20261030/state \
  forecast-team --repository emmett08/engarg --repository emmett08/eal-mcp \
  --horizon day --output work/observatory-use-case-20261030/issued-day.json
```

The actual run used absolute paths under `/workspace/work/` for its private
state and export. The earlier public bundle was copied unchanged into that
state's `forecasts/` directory before collection, enabling automatic scoring.
Reproduction extracts the source snapshot from the retained daily bundle; it
does not pretend a fresh collection will reproduce the old inventory or receipt.

For continued evaluation, preserve that state and rerun `forecast-team` after
the target ends; it will archive another forecast and append scores for mature
earlier targets. Alternatively, use `score-forecast` with the decompressed
original bundle and a newly collected, complete later snapshot, as described in
the [forecasting contract](../../automatic-forecasting.md#score-original-forecasts-automatically-or-offline).
Do not overwrite original forecasts or treat a newer snapshot of stale sources
as later observation. No recurring collection was scheduled by this use case.

Validation: the release build, retained-state verification (119 records), offline
reproduction, independent source-count and RPS calculations, and 59 existing
forecast/throughput integration tests passed. This validates execution and
reproducibility; the empirical accuracy question remains open.
