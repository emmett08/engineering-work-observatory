# Sources for sharp identification and design-based estimation

Date: 29 October 2026. Scope: the new partial-identification, event-history and model-assisted-estimation components of `paper/article/main.tex`, on base `b89a2584ef34637b71f5036a67673250b382d4e2`. This is a bounded source and attribution audit, not an exhaustive novelty review. Existing bibliography keys and the historical proof package are preserved.

The paper derives its labour-specific results directly. The sources below establish mathematical frameworks, standard algorithms or estimator antecedents. None establishes empirical superiority of the proposed engineering measurements, validates their observation assumptions or identifies an AI policy effect in an organisation.

## Verified sources and permitted uses

### `torgovitsky2019`

Alexander Torgovitsky, “Partial identification by extending subdistributions”, *Quantitative Economics* **10**(1), 105–144 (2019), [DOI 10.3982/QE634](https://doi.org/10.3982/QE634). Journal metadata were cross-checked against Crossref and the publisher. The [author-hosted journal PDF](https://a-torgovitsky.github.io/pies.pdf) supplies the full text.

Read §1, especially p. 106; §3.3, equation (12), Theorem 1 and its hypotheses; §3.4, Proposition 1; and their Appendix C proofs, pp. 138–140. The paper characterises admissible latent distributions compatible with an observed distribution and projects them to a target. Sharpness requires the converse construction, not merely necessary inequalities. Linear optimisation is available under the stated linearity and finite-dimensional conditions.

Permitted attribution: conceptual precedent for compatibility-defined sharp sets and optimisation of their targets. The present finite interval partition and shared human-capacity reconstruction are proved in this manuscript. They are not instances claimed to follow automatically from the subdistribution theorem. No novelty or priority claim is established by this comparison.

### `foucartliao2023`

Simon Foucart and Chunyang Liao, “Optimal Recovery from Inaccurate Data in Hilbert Spaces: Regularize, But What of the Parameter?”, *Constructive Approximation* **57**(2), 489–520 (2023), [DOI 10.1007/s00365-022-09590-5](https://doi.org/10.1007/s00365-022-09590-5). The issue year is 2023; the publisher records online publication on 14 October 2022. Crossref confirms authors, volume, issue and pages. Full-text access used the [author PDF](https://foucart.github.io/publi/ORHilbert_Reg2.pdf); the publisher page provides only a preview.

Read author-PDF §§1.1–1.2, pp. 1–4, including equations (2)–(5). The formulation separates the model set, observation map, deterministic error restriction and recovery objective. Its main setting uses Hilbert-space approximation priors and Euclidean error balls.

Permitted attribution: the distinction between admissible model/data consistency and a recovery rule. The paper does not prove the manuscript's polyhedral labour result, box-error constraints or scalar sharp endpoints. No regularisation theorem is imported.

### `indyk2011`

Piotr Indyk, “Lecture 17: Shortest Paths IV: Applications”, MIT 6.006, 7 April 2011. The [official Spring 2011 schedule](https://courses.csail.mit.edu/6.006/spring11/notes.shtml) verifies the title and date; the [complete eleven-slide PDF](https://courses.csail.mit.edu/6.006/spring11/lectures/lec17.pdf) names Indyk.

Read slides 6–11: constraints (x_j-x_i\leq w_{ij}), their directed graph, the negative-cycle contradiction, the shortest-path construction of a feasible assignment and Bellman–Ford complexity. Slides 6–7 contain the representation; slides 8–11 contain feasibility and the algorithm.

Permitted attribution: the standard difference-constraints method. The manuscript separately proves how allocation capacities and observed interval totals produce its graph, and why its endpoint bounds are attained. This is the Spring course, not the distinct Fall 2011 lecture 17 by Srini Devadas.

### `denishaddou2026`

Nicholas Denis and Mohammed Haddou, “Prediction-Powered Estimation: Unbiased Model-Assisted Estimation”, *Journal of Official Statistics* **42**(3), 371–394 (2026), [DOI 10.1177/0282423X261451320](https://doi.org/10.1177/0282423X261451320). Publisher and Crossref agree: online 2 June 2026, September issue. The [publisher full text](https://journals.sagepub.com/doi/10.1177/0282423X261451320) was accessible.

Read §§2.1–2.5 and the distinction from §3. Section 2.3, equations (2)–(4), gives the fixed auxiliary-value difference estimator, its pre-trained-prediction version and the residual pair-inclusion variance estimator. Predictors are independent of the survey sample; the full-population prediction term is fixed under the design. Positive pair-inclusion probabilities are needed for the stated unbiased variance estimator. Section 3's complement-weighted PPE is a different estimator.

Permitted attribution: the paper's labour estimator specialises the established difference estimator. Its exact unbiasedness is conditional on the fixed values and the sampling design. Lower variance requires informative predictions; external application results do not establish efficiency for engineering labour measurement. The manuscript supplies its own expectation and variance derivation.

### `angelopoulos2023`

Anastasios N. Angelopoulos, Stephen Bates, Clara Fannjiang, Michael I. Jordan and Tijana Zrnic, “Prediction-powered inference”, *Science* **382**(6671), 669–674 (2023), [DOI 10.1126/science.adi6000](https://doi.org/10.1126/science.adi6000). Crossref and PubMed confirm publication metadata. Read the [author manuscript, arXiv:2301.09633v4](https://arxiv.org/abs/2301.09633v4), revised 9 November 2023.

Read §§1.2–1.4 and Appendix B, especially its introductory sampling assumptions, Theorem B.1 and Corollary B.1. Section 1.4 explicitly acknowledges difference-estimator antecedents. Appendix B treats a fixed finite population with a uniformly sampled subset, so describing PPI as exclusively an independent-sample framework would be inaccurate.

Permitted attribution: related prediction-plus-correction inference and its established survey-estimation lineage. Denis and Haddou §2.3 is the direct citation for the manuscript's general inclusion-probability formula. No confidence guarantee is transferred without its sampling and interval assumptions.

## Event-history comparators reviewed, not cited

The final event-history subsection only defines a vector of path functionals; it does not fit a multistate stochastic law or import a cumulative-utility estimator. After comparing its actual claims with the sources, the manuscript and source reviewers agreed that clinical citations were unnecessary. The two sources below remain in the audit record but were not added to the final bibliography.

### Bühler, Cook and Lawless (2023)

Alexandra Bühler, Richard J. Cook and Jerald F. Lawless, “Multistate models as a framework for estimand specification in clinical trials of complex processes”, *Statistics in Medicine* **42**(9), 1368–1397 (2023), [DOI 10.1002/sim.9675](https://doi.org/10.1002/sim.9675). Crossref and the [publisher full text](https://onlinelibrary.wiley.com/doi/full/10.1002/sim.9675) agree on metadata.

Read §§2.1–2.2 and §4.2.1. These define process features through state occupancy, event times and history-dependent transition intensities; they distinguish loss of observation from events in the process being studied. Their clinical estimands and inference assumptions do not become engineering results by changing state labels.

Permitted attribution: a framework for defining several endpoints on one event history, and for explicitly separating observation loss from substantive transitions. The engineering state vocabulary and labour allocation marks remain modelling choices made here. No clinical result is cited as validating those choices.

### Bühler, Cook and Lawless (2025)

Alexandra Bühler, Richard J. Cook and Jerald F. Lawless, “Specification of Estimands for Complex Disease Processes Using Multistate Models and Utility Functions”, *Statistics in Medicine* **44**(23–24), e70269 (2025), [DOI 10.1002/sim.70269](https://doi.org/10.1002/sim.70269). The [publisher full text](https://onlinelibrary.wiley.com/doi/full/10.1002/sim.70269) and Crossref verify the article number and 14 October 2025 online publication.

Read §2.1 and §3.1, equations (3.1)–(3.4). State occupancy durations yield cumulative utility and a corresponding treatment contrast. The paper explicitly limits its main utility formulation rather than treating every history-dependent reward as already covered.

Permitted attribution: precedent for explicit state-dependent cumulative rewards and time-at-risk endpoints. Engineering person-hours, acceptance events and assessed retention are separately specified here. The manuscript does not import the paper's utility weights, clinical observations, Aalen–Johansen consistency assumptions or jackknife inference.

## Candidates inspected but not required by the core argument

- Silvia Bartolucci and Pierpaolo Vivo, [“Queue & AI: When Faster Tasks Slow Down the Workflow”, arXiv:2605.27202v1](https://arxiv.org/abs/2605.27202v1), submitted 26 May 2026. Version 1 and its full PDF were checked. Sections 2.1–2.2, equations (3)–(4), and §§4.2–4.3 develop an M/G/1 waiting-time mechanism; §3's empirical-calibration discussion describes measurements still needed. This is a theoretical preprint, suitable at most for a bounded future-work comparison. It is not empirical validation of the manuscript or a source for its identification theorem.
- Särndal, Swensson and Wretman, *Model Assisted Survey Sampling* (Springer, 1992), [DOI 10.1007/978-1-4612-4378-6](https://doi.org/10.1007/978-1-4612-4378-6): publisher and Crossref metadata verified. The relevant textbook chapter was not accessible in this audit. No precise textbook theorem or page is represented as independently read; the inspected Denis–Haddou and Angelopoulos texts supply the required estimator attribution.
- Deville–Särndal calibration and more general moment-programming sources are unnecessary for the equations proved here and were not added merely to enlarge the bibliography.

## Search and access record

All searches and reads above were performed on 29 October 2026. General search located primary material; substantive claims were checked in full-text passages, not search snippets. Crossref was queried directly for each journal DOI. No institutional login or subscription access was assumed.

| Search or verification route | Disposition |
| --- | --- |
| Exact Torgovitsky title and `QE634`; author PDF and Wiley record | Retained for the sharp-set framework; inspected statement, hypotheses and proof. |
| Exact Foucart–Liao title; author PDF, author publications list, Springer and Crossref | Retained for model/data consistency; resolved 2022 online versus 2023 issue year. |
| MIT 6.006 Spring 2011 lecture 17; official schedule and PDF | Retained; disambiguated from the Fall course. |
| Exact Denis–Haddou title and DOI; publisher HTML and Crossref | Retained; equations (2)–(4) are the direct fixed-predictor antecedent. |
| Science DOI and arXiv author manuscript | Retained as related work; checked finite-population Appendix B to avoid an overbroad contrast. |
| Both Bühler–Cook–Lawless DOIs; publisher full texts and Crossref | Reviewed conceptual comparators; omitted from the bibliography because the subsection only defines its own functionals. |
| Exact queue-preprint identifier, versioned landing page and PDF | Theoretical comparator only; no empirical transfer. |
| Springer textbook metadata and chapter-access attempt | Metadata verified; substantive chapter access unavailable. |

## Integration citation audit

The integrated text in `main.tex`, `partial-identification.tex`, `design-based-estimation.tex` and `event-history.tex` was inspected after the component edits. All five new bibliography entries are cited. A key scan across `paper/article/*.tex` and `references.bib` found 17 distinct bibliography keys, no duplicate keys, no undefined citation keys and no uncited new entries. Existing references were preserved.

The partial-identification component attributes only the general compatibility perspective to Torgovitsky, only the model/data/recovery distinction to Foucart and Liao, and the graph algorithm to Indyk. Its original slide locator was refined from 8–11 to 6–11 to include the graph encoding. The sampling component explicitly calls the estimator established, cites Denis and Haddou equations (2)–(4), and accurately acknowledges the PPI manuscript's finite-population extension. The event-history component defines its own vector of functionals without claiming that the clinical papers prove or validate it.

The introduction and abstract identify direct analytical proofs and finite computational checks as the evidence, without claiming empirical superiority or extending historical Lean coverage. The sampling text preserves the distinctions between fixed-predictor design unbiasedness, ratio-estimator behaviour, a variance identity and available uncertainty estimates. No new source-support defect remains in the inspected text. This source pass does not substitute for the separate clean LaTeX build, mathematical review or software checks.

This audit does not establish that the labour specialisation is absent from all prior literature. The contribution should therefore be described as the formulation and direct derivation developed here, without a first-ever claim.
