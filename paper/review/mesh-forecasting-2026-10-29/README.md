# Fully connected Astra mesh: automatic operational model

The user selected team throughput and bottlenecks and preferred few assumptions,
practical application, automatic observations and predictions. Six independently
executing `gpt-6-astra` workers ran with `ultra` reasoning, plus the coordinating
host. Every worker sent its independent proposal and challenges directly to all
five peers, consumed their responses and recorded resulting revisions. The six
worker nodes therefore exercised all 30 directed peer edges. The host exchanged
messages with every node. This was native communicating collaboration; no DAG-only
MAS runtime is represented as a mesh.

| Worker | Other five proposals consumed and answered | Retained record |
|---|---|---|
| Observability | Yes | [Source contract and exchange ledger](observability.md) |
| Survival | Yes | [Event-history challenge and exchange ledger](survival.md) |
| Sequence/count modelling | Yes | [Exchange ledger](sequence.md) |
| Queueing/reliability | Yes | [Source-grounded model review and exchange ledger](reliability.md) |
| Independent validation | Yes | [Independent review](../mesh-independent-forecast-validation-2026-10-29.md) |
| Integration | Yes | [Collector and integration ledger](integration.md) |

Daily cold-start, source-freshness and scope amendments also circulated among all
five peers of each worker. Research preceded implementation. Peer objections
changed the mechanism, not merely its presentation: survival of currently open
PRs was rejected as a total-throughput model because future arrivals and reopened
closed requests also contribute; a stale snapshot could not extend exposure;
current queue fields could not become historical features; and no independent
per-repository convolution was accepted.

## Decision and evidence

The operational endpoint is a future count of visible PR merges across selected
repositories. It is not labour productivity, accepted functionality, quality or a
causal AI effect. The original labour-identification model remains a separate
research option. The new automatic workflow needs no AI labels, task sizes, human
effort records or acceptance judgements.

The experimental recent-eight-period empirical model and its expanding reference
were specified before inspecting their public-reference scores. The public
repository and fixed 78/52-week frame were also declared before scoring; see the
[frozen protocol](frozen-acceptance.json), [independent acceptance record](validation-acceptance.md)
and [daily/freshness amendment](daily-freshness-amendment.md). These local records
retain the declared chronology; they are not independent trusted-time certificates.

On 26 reference weeks, recent-eight mean RPS was 2.525841 versus 2.274930 for the
expanding comparator: 11.03% higher loss. Its lower point-forecast absolute error
did not override the distributional primary score. The recent challenger failed
promotion, so the reference rule was retained as default. No window was tuned on
these outcomes. The operational full-supported-history rule and bounded-study
52–77-week comparator have different training frames. No superiority or transport
to the user's repositories follows from retaining the reference.

The user portfolio supplied only five complete common UTC days. The resulting
next-day distribution is an unvalidated baseline; weekly and fortnightly forecasts
are unavailable until their calendar-support requirements are met. A genuinely
issued [public-source example](../../forecasting/examples/README.md) is retained
separately from the retrospective manuscript tables. Automatic scoring retains
pending outcomes without partial scores, checks mature source exposure and binds
unchanged forecasts to their original source snapshot and explicit method.

Current open, draft, requested-review and ageing predicates report observed queue
pressure. They do not forecast a future limiting stage or identify a causal
bottleneck. The smallest follow-on is accumulated automatic fixed-time queue-state
snapshots and a separately tested future queue-pressure endpoint, against frozen
persistence and empirical baselines. Historical final PR states cannot reconstruct
all close/reopen or review-stage episodes. No unsupported future-pressure model
was added to fill that evidence gap.

## ⚠️ EVALUATION ALERT: Non-Definitive Outcome Detected

* **Classification:** Negative Result (recent-window promotion failed); Limited Result (user predictive skill and future bottlenecks unestablished).
* **Target Objective:** An automatic, practical predictive model of team throughput and bottlenecks with minimal assumptions.
* **The Gap:** The tested recent challenger has 11.03% higher primary loss than its fixed reference. Five user days do not establish predictive accuracy. Queue pressure is observed; future stage congestion is not predicted. The adverse comparison is a completed scientific result, not an omitted success.

## 🎯 HIGH-PROBABILITY PIVOT STRATEGY

* **Alternative Approach:** Retain the simpler expanding reference, preserve the challenger and adverse results, and automatically archive and score future unchanged forecasts. Accumulate queue observations before attempting a future-pressure model.
* **Why This Will Work Better:** It avoids promoting a candidate that failed the frozen criterion and makes future claims falsifiable from automatic outcomes. Comparative success probability and future predictive improvement are unestablished.
* **Immediate Next Steps:** 1) Implemented and checked the retained-reference workflow and source-bound archive scorer; 2) independently reproduced all 130 public method/origin scores and retained the negative comparison; 3) collect mature future observations automatically before claiming user accuracy or promoting a future-pressure model.

The [ranked alternative analysis](../ranked-automatic-model-alternatives.md) gives
three distinct approaches, their assumptions, automatic inputs and promotion
conditions. Source ledgers retain full primary-source support and adverse evidence.
