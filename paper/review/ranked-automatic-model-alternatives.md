# Three ranked alternatives for automatic throughput forecasting

This ranking serves the chosen objective: forecast team PR throughput with automatic GitHub evidence, little operational work and explicit uncertainty. It does not rank models universally. The selected repositories supplied only five complete common portfolio days at collection, despite containing 78 PRs. PR records are not 78 independent daily forecasting observations. No approach below has demonstrated predictive skill on this portfolio. Only approach 1 is implemented.

The outcome is the number of observed merges across a fixed repository portfolio in a declared future interval. It includes merges of newly opened PRs and reopened older PRs. A merge is not a requirement, person-hour, correctness certificate or unit of business value. Human-labour identification remains the paper's separate question; predicting repository events does not solve it.

| Rank | Mathematical basis | Main reason for this position |
|---|---|---|
| 1 | Nonparametric probability forecasting | Directly predicts the requested count with the fewest collection and fitting requirements. |
| 2 | Conditional count time series | Tests whether automatic recent history improves forecasts without reconstructing every workflow transition. |
| 3 | Landmark event-history prediction | Supports per-PR and stage-specific risks, but needs much stronger historical state coverage and aggregation machinery. |

## 1. Direct empirical portfolio counts: retain the reference

For completed comparable calendar periods with counts \(y_1,\ldots,y_n\), issue

\[
\widehat F(k)=n^{-1}\sum_{j=1}^{n}\mathbf 1\{y_j\le k\}.
\]

The operational default uses all supported complete periods: an expanding empirical reference. The recent-window challenger uses the preceding eight daily or weekly periods, or four observed disjoint fortnight totals. Both require at least four complete periods. Four is an operational support rule, not evidence of calibration. Daily forecasts remain exploratory; unavailable weekly history is not replaced by scaling daily predictions.

Inputs are stable repository and PR identifiers, repository creation dates, merge timestamps, retrieval coverage and collection times. Sum synchronized repository counts before fitting; independent convolution would discard historical cross-repository dependence. Fourteen-day totals likewise come from observed pairs, not independent weekly draws. Training ends at the common conservative collection boundary; issuance and future target boundaries are separate.

The assumptions are visible-source coverage, a stable counting definition and sufficient relevance of historical periods to the future. The model imposes no Poisson law or PR independence. Its finite support can miss unseen extremes, and expanding history can react slowly to change. Report mean, median, quantiles, actual empirical mass and subsequent proper scores; historical maxima are not physical limits. Computation is elementary, with sorting-based scoring costing \(O(n\log n)\).

The adverse result matters. In the frozen public-reference study of `cpmpy/cpmpy`, 78 weeks supplied 52 initial training weeks and 26 held-out weekly origins. Recent-window mean RPS was **2.525841**, versus **2.274930** for the expanding reference: about **11.03% worse**. The challenger failed promotion. Retaining the reference does not establish a validated winner, and the study's bounded frame differs from the operational full supported frame. The five-day portfolio forecast remains unvalidated. See [the experiment report](../forecasting/README.md).

## 2. Regularised conditional count dynamics: the next serious challenger

Use a small negative-binomial count regression, with Poisson as a declared restricted comparator:

\[
M_{t+1}\mid\mathcal F_t\sim\mathrm{NB}(\mu_t,\kappa),
\qquad \log\mu_t=\beta_0+\beta^\top x_t,
\qquad \operatorname{Var}(M_{t+1}\mid\mathcal F_t)
=\mu_t+\mu_t^2/\kappa.
\]

Here \(x_t\) contains a few frozen predictors: previous complete-period merges and openings, and later, genuinely archived queue summaries. Ridge regularisation is a proposed fitting choice, not a theorem that repairs scant data. No developer identity, AI attribution or manually classified task is required.

This model can represent changing conditional rates and overdispersion while keeping the target at portfolio level. It adds assumptions about the link, selected lags, conditional distribution and stability across observed regimes. Unknown future predictors cannot simply be supplied as known: use a direct horizon-specific target or model their future evolution. Configuration hashes may identify observed workflow changes; they do not identify AI exposure or causal policy effects.

Practical effort is moderate: construct chronological feature rows, implement fitting and diagnostics, preserve model versions, and evaluate forward. Computation is small for a few predictors. The present five common days cannot separate weekday, launch-period and lag effects or support reliable dispersion estimation. A generic minimum PR count would be misleading; the relevant support is calendar variation and attainable forecast precision.

Freeze the feature set, regularisation, distribution choice and material improvement threshold before a **new** untouched evaluation. Fit every transformation using earlier data only. Promote only for a useful count-distribution score improvement over the retained reference, with matched coverage and acceptable calibration and interval width. The already inspected 26 weeks cannot become a fresh confirmation set for this new model.

Sequence neural networks are a credible later rival, but not the preferred next step. Published comparisons provide both positive LSTM results and newer findings of no significant advantage among sophisticated methods. Neither establishes benefit for these small-repository aggregate counts. A 2026 process foundation model also reports limited held-out logs and leaves dedicated calibration for later work; transfer learning does not remove the need for local validation.

## 3. Landmark merge risks with arrival and reopening contributions

At a retained forecast cut \(t\), estimate actual merge-by-horizon probabilities from PR age, observed state and recorded review/CI history. Let \(U_t\) contain every existing unmerged PR, including closed ones that could reopen, and let \(m_i\) be its merge timestamp, or infinity if it never merges. Then

\[
E[M(t,h)\mid\mathcal F_t]
=\sum_{i\in U_t}P(t\le m_i<t+h\mid\mathcal F_t)
 +E[M_{\mathrm{new}}(t,h)\mid\mathcal F_t].
\]

Linearity justifies this mean decomposition without independent PRs. It does **not** justify independent Bernoulli count intervals. Shared reviewers, release waves and cross-repository changes can couple outcomes. Aggregate intervals need a joint model or a separately calibrated aggregate error distribution; forward calendar-block evaluation tests their adequacy.

The minimal candidate uses empirical landmark cohorts with a complete observed outcome horizon. More elaborate multistate hazards become relevant only if needed. An unmerged closure is not absorbing: first-disposition merge probability differs from eventual merge within the horizon after reopening. Restricting the sum to currently open PRs also misses previously closed PRs revived during the horizon. Future arrivals need their own forecast contribution.

Automatic inputs therefore extend to archived origin states or sufficiently complete close/reopen timelines, timestamped review submissions, and correctly associated CI attempts where used. Current mutable PR payloads cannot reconstruct past draft status, heads or queue membership. “Review submitted” need not mean human review. Complete outcome horizons remove target censoring but restrict the training population. Mature age/state cohorts must remain relevant to future eligible PRs; complete follow-up does not remove cohort selection or temporal transport assumptions.

This approach offers more detailed risk forecasts, but collection, linkage, state reconstruction and joint aggregation cost more than approach 2. The current portfolio lacks longitudinal state histories and adequate age/state-specific follow-up. Promotion requires new frozen future origins, support checks for each risk cohort, and useful aggregate improvement over the direct reference. Unsupported cohorts remain unavailable or use the declared reference. The approach is proposed, not implemented.

## What remains unsupported

The implemented system forecasts merges and reports last-observed queue pressure. **It does not predict future bottlenecks.** The smallest automatic next target is the number of open PRs aged at least seven days at a fixed future cut, using accumulated daily snapshots and comparison with frozen persistence and empirical baselines. Success would establish prediction of observed pressure, not a causal resource constraint.

With complete transition accounting,
\(B_{t+h}=B_t+A+R-M-C\), where openings, reopenings, merges and unmerged closures are distinct flows. This conservation identity does not forecast its future terms. Openings minus merges alone is not backlog change. Little's law and mechanistic service queues likewise cannot supply unobserved human service requirements, capacity or future arrivals from timestamps. Such models require independently defensible inputs and assumptions before they can answer capacity questions.

## Primary-source ledger

| Source | Specific support and boundary |
|---|---|
| [Gneiting and Raftery, 2007, §4.2](https://sites.stat.washington.edu/people/raftery/Research/PDF/Gneiting2007jasa.pdf) | Proper probabilistic scoring and CRPS; no guarantee that an empirical forecast is calibrated. |
| [Fokianos, Rahbek and Tjøstheim, 2009](https://doi.org/10.1198/jasa.2009.tm08270) | Observation-driven Poisson dynamics; asymptotic conclusions require their ergodicity conditions, not five-day samples. |
| [Christou and Fokianos, 2014](https://doi.org/10.1111/jtsa.12050) | Negative-binomial feedback models and distributional diagnostics; published abstract inspected, not a small-repository validation. |
| [Verenich et al., 2019, conclusions](https://kodu.ut.ee/~dumas/pubs/TISTTimePrediction.pdf) | Sixteen methods across seventeen event logs favoured LSTM overall; different targets and data support prevent direct transfer. |
| [Roider et al., 2024](https://cris.fau.de/publications/328671158/) | XGBoost, DA-LSTM and PGT-Net lacked significant performance differences; baseline-relative evaluation exposed poor performance on some logs. |
| [Khatoonabadi et al., 2024, §§2–4](https://das-concordia.github.io/pdf/Khatoonabadi_TSE2024.pdf) | Strong CatBoost response-latency results on large projects; manual bot inspection and identity features exceed this automatic contract. |
| [Berti and van der Aalst, 2026, §VI.J and conclusion](https://publications.rwth-aachen.de/record/1029172/files/1029172.pdf) | Few-shot sequence transfer with small, largely synthetic pretraining and limited held-out logs; calibration remains an open task. |
| [van Houwelingen and Putter, 2008, §§3–5](https://link.springer.com/article/10.1007/s10985-008-9099-8) | Landmark prediction compared with multistate modelling; methodological precedent, not evidence about PR forecasts. |
| [Little, 2011, §§2.1.4–2.3](https://people.cs.umass.edu/~emery/classes/cmpsci691st/readings/OS/Littles-Law-50-Years-Later.pdf) | Finite-window queue measurement and boundary conditions; identities do not predict exogenous future flows. |
