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

The build produces `build/main.pdf`. The [work examples with known hours](../study/labour-measurement) give exact values against which to check the calculations. The [paired trial design](../study/ai-labour-causal) supplies the assumptions for the trial simulations. The model notes show how to bound the effect of AI access when some team outcomes remain unknown. Before using that method in a trial, teams need to test how often it detects a worthwhile benefit across plausible effects and measurement errors.
