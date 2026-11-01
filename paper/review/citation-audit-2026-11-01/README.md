# Citation audit, 1 November 2026

This read-only audit concerns the six sources compiled by `paper/article/main.tex`,
their BibTeX file, and the mathematical additions in the preceding MAS Math Lab
review. It nominates citations for the coordinator to integrate; this audit does
not itself edit the manuscript or canonical bibliography. Causal-trial methodology
is being researched separately with the new study protocol.

## Findings to integrate

### 1. Attribute the full-attention construction to its mathematical context

The cellwise construction is an elementary finite-partition instance of
Dvoretzky--Wald--Wolfowitz purification. The original [vector-measure paper,
Theorem 4](https://msp.org/pjm/1951/1-1/pjm-v1-n1-p06-s.pdf) equates the integral
ranges obtained from fractional component functions with those obtained from a
measurable partition, for finitely many finite atomless measures. Its definitions
and proof were inspected in the primary scan (printed pp. 59--67).

[Khan, Rath and Sun (2006), Section 2](https://doi.org/10.1007/s00182-005-0004-3)
states the theorem in modern function notation. The publisher supplies reliable
metadata; the author-uploaded August 2005 manuscript on
[ResearchGate](https://www.researchgate.net/publication/24058874_The_Dvoretzky-Wald-Wolfowitz_theorem_and_purification_in_atomless_finite-action_games)
supplies the full theorem and subsequent argument, which were inspected.

For the paper's setting, add idle time as another activity and use Lebesgue
measure restricted to each cell, separately for each person. This explains the
relationship. The paper's consecutive-block proof also gives a finite block count;
do not claim that this count follows merely from the general existence theorem.

Suggested insertion after the reconstruction proof or its explanatory paragraph:

> Preserving these integrals while replacing fractional allocations by one activity
> at a time is a finite-partition instance of the Dvoretzky--Wald--Wolfowitz
> purification theorem \cite[Theorem~4]{dvoretzky1951ranges}; see also
> \cite[Section~2]{khanrathsun2006}. The consecutive-block construction gives the
> required history directly and bounds its number of blocks.

This is attribution and positioning, not a dependence on the more general
theorem or a novelty claim. A single citation to the original theorem is sufficient
if space is tight; the modern account helps readers inspect the exact formulation.

### 2. Source the qualification about hidden Markov model identifiability

The paper correctly proves that its unrestricted independent-state example has
different latent activity parameters producing the same commit distribution. It
then cautions that other HMMs can be identifiable. [Allman, Matias and Rhodes
(2009), Section 6.1 and Theorem 6](https://jarhodesuaf.github.io/papers/Latent.pdf)
provides a primary source for generic identifiability of finite-state HMMs from
enough consecutive observations. The relevant statement and proof setup were
inspected in the author-hosted full manuscript.

Add `\cite[Section~6.1]{allman2009}` to a sentence distinguishing this example
from generic-identifiability results. The citation does not establish
identifiability of the present activity/commit model, identify its exceptional
parameter set, or validate interpreting a hidden state as human work. Avoid
suggesting that persistent activity alone is a sufficient identifiability condition.

Suggested wording:

> The example concerns identifiability across every allowed parameter value.
> Generic identifiability of finite-state HMMs, apart from exceptional parameter
> sets and up to hidden-state relabelling, is a different claim
> \cite[Section~6.1]{allman2009}.

The current constructive proof needs no external attribution as a novel theorem;
the reference supports the qualification and prevents an overbroad reading.

### 3. Credit Lean and mathlib when introducing the new formal supplement

The official [Lean project citation guidance](https://lean-lang.org/learn/)
requests de Moura and Ullrich's 2021 system paper. Its
[full primary paper](https://lean-lang.org/papers/lean4.pdf) and Crossref metadata
were inspected. The [mathlib4 citation file](https://raw.githubusercontent.com/leanprover-community/mathlib4/master/CITATION.md)
requests `mathlib2020`; the community's
[full paper](https://leanprover-community.github.io/papers/mathlib-paper.pdf)
and publication metadata were inspected.

Use `\cite{demouraullrich2021,mathlib2020}` in the formal-supplement paragraph.
These sources identify the tools and library. They do not certify the new
declarations, establish correspondence to the paper, or identify the actual pinned
release; the project lockfiles, semantic map and replay receipt serve those roles.
The 2020 library paper describes the earlier Lean 3 era; the current mathlib4
repository nevertheless explicitly requests this citation.

## Conditional source, not a bibliography filler

[Bardenet and Maillard (2015), Corollary 2.5](https://arxiv.org/pdf/1309.4029)
provides a finite-population concentration bound for sampling without replacement.
Its assumptions, correction factor and statement were inspected in the primary
PDF. Include `bardenetmaillard2015` only if the paper or study protocol actually
uses such a bound for the sampled person-interval outcomes or fixed residuals.
The current simultaneous-coverage paragraph is an elementary event-inclusion
argument followed by a union bound; its displayed argument does not require a
special concentration citation. The source does not justify substituting an HT
point estimate for an exact observation or using sample extrema as known global
bounds on unsampled residuals.

The paper already cites Boyd--Vandenberghe at the duality and sensitivity claims,
Horvitz--Thompson at the estimator, Denis--Haddou and prediction-powered inference
at the fixed-prediction construction, Torgovitsky at the partial-identification
perspective, and Indyk at the scalar difference-constraint graph. None needs an
additional generic textbook citation merely because the new proof is formalised.
The elementary sum/integral and interval-additivity steps are mathematical proof
obligations, not empirical assertions in need of a replacement citation.

Do not add Charnes--Cooper, Fieller, McNaughton or aggregation papers to the
article bibliography unless corresponding results are used or discussed there.
They are comparisons retained in the research review rather than dependencies of
the current article. No unsupported claim of novelty should be added.

## Existing metadata checked

The [Cui et al. publisher page](https://pubsonline.informs.org/doi/10.1287/mnsc.2025.00535)
still labels the paper *Articles in Advance* and supplies no assigned pagination.
Its cite-as line uses `Management Science 0(0)` as an online-first placeholder;
Crossref likewise supplies no volume, issue or page. Retain the verified online
publication date, 27 February 2026, and DOI. Do not invent final pages to suppress
the current BibTeX warning. The linked author PDF still identifies itself as the
June 2025 manuscript, so the paper's version-specific instrumental-variable
citation remains appropriate.

`crossref-metadata.json` retains seven live publisher-deposited metadata records.
`candidates.bib` gives exact candidate entries and labels the conditional one.
The two tool-paper metadata records were checked live against Crossref and the
primary papers; their bibliographic fields are also recorded in that file.
All full-source inspection claims above distinguish the publisher version,
author manuscript or original scan actually inspected. No human experiment or
source-code proof is established by this citation audit.
