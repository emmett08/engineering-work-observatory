# Measuring human labour in AI-assisted software engineering

The paper shows how to bound person-hours from partially observed work and how to estimate them from randomly sampled intervals. A hypothetical trial of AI access illustrates how an unreliable measure of hours or retained functionality can change a decision about benefit.

- [Read the paper](measuring-human-labour.pdf)
- [Model assumptions and ways to weaken them](model-notes.md)
- [Editorial review and word frequencies](editorial-review.md)
- [Full paper and reproducibility materials](../README.md)

Build from this directory with LaTeX, BibTeX and `latexmk`. The check uses `pdfinfo` to enforce the 12-page limit, including references.

```sh
make check
```

The build produces `build/main.pdf`. The [work examples with known hours](../study/labour-measurement) give exact values against which to check the calculations. The [paired trial design](../study/ai-labour-causal) supplies the assumptions for the trial simulations. The alternative decision rule in the model notes requires a separate evaluation before teams use it in a trial.
