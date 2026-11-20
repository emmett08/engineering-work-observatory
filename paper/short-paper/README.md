# Measuring human labour in AI-assisted software engineering

This short methods paper develops bounds from partial observations and estimates from sampled intervals. The comparison with a hypothetical AI trial shows why checks on human hours and retained functionality belong in the measurement design.

- [Read the paper](measuring-human-labour.pdf)
- [Model assumptions and ways to weaken them](model-notes.md)
- [Rhetorical and word-frequency audit](editorial-audit.md)
- [Full paper and reproducibility materials](../README.md)

Build from this directory with LaTeX, BibTeX and `latexmk`. The check uses `pdfinfo` to enforce the 12-page limit, including references.

```sh
make check
```

The generated file is `build/main.pdf`. The existing synthetic evidence comes from the [known work histories](../study/labour-measurement) and the [paired trial protocol](../study/ai-labour-causal). The alternative trial rule requires its own planning evaluation before field use.
