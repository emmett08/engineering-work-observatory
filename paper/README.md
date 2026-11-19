# Bounding and estimating human labour in AI-assisted software engineering

This paper derives the range of person-hours consistent with overlapping observations and the time each contributor has available. It tests the bounds and two sampling estimators on work histories with known hours. A proposed field study would measure labour on fixed requirements and assess how much of each requirement the software still satisfies at follow-up.

[Read the paper](ai-assisted-software-engineering.pdf) · [LaTeX source](article/main.tex) · [References](article/references.bib) · [Critical review](review/ai-assisted-software-engineering.md)

## Build the article

From the repository root, run `make -C paper article`. This runs pdfLaTeX and BibTeX through `latexmk`, writes intermediate files to `paper/build/` and copies the finished PDF to `paper/ai-assisted-software-engineering.pdf`. The build uses `amsart`, Latin Modern, `natbib` and TikZ. The shared-capacity figure is included directly from its TikZ source. Run `make -C paper figure` to regenerate the standalone figure and audit its plotted values.

## Check the results

| Material | What the check establishes |
| --- | --- |
| [Known work histories](study/labour-measurement/README.md) | Runs the Rust bounds and sampling commands on 96 two-person histories. Counts correct, wrong and unresolved decisions when hours or a contributor are missing. |
| [Forecasts at intake](study/labour-forecast-check/README.md) | Compares two forecasts on later synthetic tasks, including a new task class and a change in the hours required for repairs. |
| [Exact examples](check_measurement_examples.py) and [identification checks](check_sharp_identification.py) | Reproduce finite examples of sampling error and bounds on person-hours. |
| [Shared-capacity figure](figures/shared-capacity/README.md) and [explanatory figures](check_explanatory_figures.py) | Check the illustrative timelines and sampling distributions against the quantities plotted. |
| [Lean supplement](formal/README.md) | Gives the proof statements, dependencies and recorded checks for the interval-to-cell reduction, attainable allocations and supplied bounds. |
| [Prospective study protocol](study/ai-labour-causal/README.md) | Defines a pilot and compares candidate trial designs in simulation. The study has yet to collect workplace outcomes. |
| [Horizon-query benchmark](performance/README.md) | Tests two implementations of cohort queries on generated data and retains the measurements used to compare their runtime. |

Run `make -C paper check` for the example, figure, study, receipt and software checks. For a fresh proof build and kernel replay with the pinned Lean toolchain, run `make -C paper formal`. The [validation guide](VALIDATION.md) gives the individual commands and their scope.

With the [planning dependencies](study/ai-labour-causal/requirements.txt) installed, `make -C paper check-planning` repeats the trial simulation, varies individual assumptions and checks how measurement errors in constructed team histories affect the decision.
