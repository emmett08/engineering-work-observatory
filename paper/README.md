# Bounding and estimating human labour in AI-assisted software engineering

An engineer may know that a review took an hour and still be unsure how much of it fell before a release. The paper derives the lowest and highest labour totals that fit observations over overlapping periods and the time each contributor had available. It checks those bounds and two sampling estimators against synthetic histories with known hours. In the proposed workplace trial, one team in each pair would receive AI access. The trial would measure labour per initial reference unit and the share of the initial score retained at follow-up by requirements accepted on time.

[Read the paper](ai-assisted-software-engineering.pdf) · [LaTeX source](article/main.tex) · [References](article/references.bib) · [Critical review](review/ai-assisted-software-engineering.md)

## Build the article

From the repository root, run `make -C paper article` to build `paper/ai-assisted-software-engineering.pdf`. The recipe uses `latexmk` with pdfLaTeX and BibTeX and stores intermediate files in `paper/build/`. The article uses `amsart`, Latin Modern, `natbib` and TikZ. To rebuild the standalone shared-capacity diagram and check its plotted values, run `make -C paper figure`.

## Check the results

| Material | What the check establishes |
| --- | --- |
| [Known work histories](study/labour-measurement/README.md) | Checks the Rust bounds and sampling estimates against 96 two-person histories. Counts how often a team can correctly determine whether first-hour labour exceeded one person-hour when a contributor or some hours go unobserved. |
| [Forecasts at intake](study/labour-forecast-check/README.md) | Compares forecasts from past tasks of the same class with a forecast that pools all classes. Tests cover a new class and an increase in repair hours. |
| [Exact examples](check_measurement_examples.py) and [identification checks](check_sharp_identification.py) | Reproduce finite examples of sampling error and bounds on person-hours. |
| [Shared-capacity figure](figures/shared-capacity/README.md) and [explanatory figures](check_explanatory_figures.py) | Check the times and labour totals on the diagrams and the estimate for every possible sample in the dot plot. |
| [Lean supplement](formal/README.md) | Gives the proofs and verification results for calculating interval labour from time cells and certifying lower and upper bounds. |
| [Workplace trial protocol](study/ai-labour-causal/README.md) | Specifies the pilot and gives the frequency with which each simulated trial design met both decision criteria. Workplace data collection has yet to begin. |
| [Horizon-query benchmark](performance/README.md) | Compares the runtime of a direct scan and a sorted index for repeated queries on the same synthetic cohort. |

Run `make -C paper check` to check the worked examples, figures, numerical results, proof receipts and software. For a fresh proof build and kernel replay with the pinned Lean toolchain, run `make -C paper formal`. The [validation guide](VALIDATION.md) gives the individual commands and their scope.

With the [planning dependencies](study/ai-labour-causal/requirements.txt) installed, `make -C paper check-planning` repeats the trial simulation, varies individual assumptions and checks how measurement errors in hypothetical team histories affect the trial decision.
