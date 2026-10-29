# Pre-score feasibility amendment and independent audit record

## Daily feasibility amendment

User-selected repositories have only roughly five to nine days since creation, so four complete weekly training bins and eight weeks for direct fourteen-day forecasts are unavailable. Before examining model scores, the host proposed an additional operational one-day empirical forecast from the preceding eight complete UTC days, with a four-day minimum and explicit sparse/exploratory status. This does not change the primary frozen weekly endpoint or validate it. No scaling, repeated resampling, convolution, independent-arrival model, or multiplication converts this daily distribution into a weekly forecast.

The daily target is the next full UTC day starting at or after issue time. An exact midnight issue can target the day beginning then; an intraday issue targets the following UTC day. Exclude partial repository-creation/current days. Report the actual training counts and sample size; acknowledge launch and weekday effects, coarse quantiles and unvalidated transport. Zero currently open PRs do not imply zero future mergers because new arrivals may merge in the target period.

All five peers received the amendment and replied before any model scores were viewed by validation:

| Peer | Sent | Response consumed | Accepted constraints |
| --- | --- | --- | --- |
| observability | yes | yes | Separate issue, snapshot and source-exposure cuts; no manual coverage flag |
| survival | yes | yes | Next full day; retrospective lower frame after repo creation; no weekly transfer |
| sequence | yes | yes | Direct daily PMF and explicit cutoff API; four-day fallback; no scaling |
| reliability | yes | yes | Day-of-week/launch limitation; included discrete interval mass; zero-open boundary |
| integration | yes | yes | Lightweight collection only; actual request span; incomplete required sources unavailable |

## Confirmed freshness defect and accepted correction

Initial implementation populated `PreparedHistory.observed_cut` from `snapshot.as_of`, then constructed complete bins to that date. If the latest complete collection was October 5 but a snapshot was frozen October 10, October 5–10 could become invented zero-count exposure. Open-PR ages were similarly advanced while their state remained an old observation. Validation reported this to observability, sequence and the host.

The accepted correction separates:

- `snapshot_cut` / issue time: the knowledge cut used to select retained source records and to start a future forecast target;
- common source exposure cut: the earliest **started_at** among each selected repository's latest complete collection;
- per-repository collection start/end: freshness of last-observed queue pressure.

Use started_at rather than completed_at because pagination can straddle midnight: an early page cannot know that its old PR merged after it was fetched but before the final page completed. Full training bins must end no later than the common conservative source-exposure cut. A later incomplete collection must not silently be replaced by an earlier complete one. Source events from a later collected repository may occur after the portfolio's common cut; exclude them from earlier count bins, while validating them against their own collection span and snapshot cut. Do not manufacture an observed-at historical availability claim.

## Independent score audit

`rust-audit` compiles the production `src/forecast/distribution.rs` directly, without editing it. Expected results come from an independent Python Fraction oracle using `E|X-y| - 0.5 E|X-X'|`, rather than the production adjacent-gap calculation. Eight distributions include zero/constant, bursty, skewed, two-point, monotone, four-day fallback and an outlier beyond the recent-eight window. Forty-eight RPS cases include observations beyond support. The audit also checks mean versus median, empirical interval endpoints/included mass, empty input, and integer precision bounds.

Result: all eight PMFs and 48 exact-reference RPS cases passed, as did boundary rejection checks. This establishes those implementation calculations on the tested cases, not empirical predictive accuracy.
