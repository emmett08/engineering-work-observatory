# How Much Human Work Is Behind AI-Assisted Software?

*Labour Bounds and Estimation from Work Telemetry* asks what a team can infer about human effort from incomplete records. An engineer who spent three hours on a task during a four-hour window must have spent between one and two of those hours in the first two hours. The paper generalises this calculation to several people and tasks, then shows how a probability sample of work intervals can estimate total labour. It follows the same requirements through acceptance and follow-up so that the numerator and denominator of a productivity estimate refer to a defined cohort.

[Read the paper](ai-assisted-software-engineering.pdf) · [LaTeX source](article/main.tex) · [References](article/references.bib) · [Critical review](review/ai-assisted-software-engineering.md)

## Build the article

From the repository root, run `make -C paper article`. This runs pdfLaTeX and BibTeX through `latexmk`, writes intermediate files to `paper/build/` and copies the finished PDF to `paper/ai-assisted-software-engineering.pdf`. The build uses `amsart`, Latin Modern, `natbib` and TikZ. The shared-capacity figure is included directly from its TikZ source. Run `make -C paper figure` to regenerate the standalone figure and audit its plotted values.

## Check the results

| Material | What the check establishes |
| --- | --- |
| [Exact examples](check_measurement_examples.py) and [identification checks](check_sharp_identification.py) | Reproduce finite examples of sampling error and bounds on person-hours. |
| [Shared-capacity figure](figures/shared-capacity/README.md) and [explanatory figures](check_explanatory_figures.py) | Check the illustrative timelines and sampling distributions against the quantities plotted. |
| [Lean supplement](formal/README.md) | Gives the proof statements, dependencies and recorded checks for the interval-to-cell reduction, attainable allocations and supplied bounds. |
| [Prospective study protocol](study/ai-labour-causal/README.md) | Defines a pilot and compares candidate trial designs in simulation. The study has yet to collect workplace outcomes. |
| [Horizon-query benchmark](performance/README.md) | Tests two implementations of cohort queries on generated data and retains the measurements used to compare their runtime. |

Run `make -C paper check` for the example, figure, study, receipt and software checks. For a fresh proof build and kernel replay with the pinned Lean toolchain, run `make -C paper formal`. The [validation guide](VALIDATION.md) gives the individual commands and their scope.
