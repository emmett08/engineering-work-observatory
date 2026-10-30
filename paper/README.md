# Papers and reproducibility

For automatic, label-free team-throughput prediction, read the new
[operational forecasting companion](forecasting/automatic-throughput-forecasting.pdf),
its [source and reproduction guide](forecasting/README.md), and the
[three ranked model alternatives](review/ranked-automatic-model-alternatives.md).
The [fully connected Astra ultra mesh record](review/mesh-forecasting-2026-10-29/README.md)
retains the research, source contracts, frozen comparison and adverse result.
Current queue pressure is observed; future bottlenecks and causal capacity remain
unestablished. The automatic workflow is independent of the labour observations
required by the following manuscript.

## How Much Human Work Is Behind AI-Assisted Software?

*Labour Bounds and Estimation from Work Telemetry*

Read the [article](ai-assisted-software-engineering.pdf), built from
[`article/main.tex`](article/main.tex) and
[`article/references.bib`](article/references.bib). The publication version centres
on attainable labour bounds from interval observations and shared per-person
capacity, an exact finite-cell reduction, and fixed-prediction design-based
estimation. It distinguishes labour, elapsed time to acceptance and retained
reference output. A short validation section states the measurements and
assumptions needed before an empirical intervention comparison.

The 30 October 2026 revision removes the execution/retry/detection tutorials,
Standard Coder correction appendix, runtime benchmark appendix and detailed
hypothetical trial protocol from the compiled article. Their historical sources,
results and reviews remain in this repository and its Git history. The
single-stream shortest-path solution remains as the one directly relevant
technical appendix. The coarse bin envelope is superseded in the article by the
capacity-aware bound and joint formulation.

The application reports conditional direct-observation labour and productivity
bounds and provides a separate `estimate-labour` command for a declared
probability sample with frozen predictions. Its simple-random-sampling variance
calculation is separate from the cohort report's HT point estimates. The
application sums marginal bounds and does not implement the joint linear
programme. Its floating-point endpoints have no outward-rounding guarantee.
Exact interval bounds and sampling estimates retain their different guarantees.
Neither establishes an empirical AI effect or the accuracy of labour and quality
measurements. The new identification and estimation results remain outside the
historical Lean proof receipt's coverage.

The [six-agent review](review/debate-2026-10-29.md) and
[Astra ultra review](review/astra-ultra-2026-10-29.md) retain the earlier analytical,
implementation and editorial history. The current publication revision adds
three further whole-paper applications of `write-for-experienced-engineers`,
each recorded in a separate commit, after the scope and terminology repairs.
See [validation notes](VALIDATION.md) for checks and their limits.

## Original paper and proof evidence

[`effort-speed-ai/`](effort-speed-ai/) is the complete original package moved from
[`emmett08/revbench`](https://github.com/emmett08/revbench/tree/64314d60104268bffbb6219912657efeb2a11098/paper/effort-speed-ai)
at commit `64314d60104268bffbb6219912657efeb2a11098`. Its 83 files retain their
original bytes and relative paths, including the paper, LaTeX/BibTeX source,
Matplotlib figure generators, synthetic calculations, Lean source and lockfile,
proof history bundle, inventory, receipts, and historical review records.

The [original 17-page PDF](effort-speed-ai/effort-speed-ai.pdf) is byte-for-byte
identical to the attachment supplied for this change:

```text
df2301249e10c1773bd3eaa6bb6b0374aa54196fafdc354761db52a8e332d66c
```

The baseline package's 43 Lean declarations cover the statements identified in
its 54-entry inventory. `make -C effort-speed-ai check` validates retained hashes,
inventory and receipt; it does not execute Lean. A fresh proof build requires
the pinned Lean 4.33.1 toolchain and mathlib dependencies; follow its
[formal README](effort-speed-ai/formal/README.md). The complete development
message archive was not part of the original package.

The original figures were generated with Matplotlib, so no original TikZ files
existed. The editable TikZ reconstructions preserve their plotted quantities,
coordinates and captions, with portable Helvetica/Nimbus Sans in place of
DejaVu Sans. The original vector PDFs remain in the baseline package.

## Build

Run from this directory:

```sh
make article
make forecasting
make baseline
```

`make article` compiles the publication source to `build/main.pdf` and copies it to
`ai-assisted-software-engineering.pdf`. `make baseline` builds the original
source to `effort-speed-ai/.build/article/main.pdf`; it does not overwrite the
attached PDF. These use pdfLaTeX, BibTeX and latexmk. `make forecasting` builds the separate
automatic-throughput companion under `forecasting/` without changing either
labour paper.

On Debian/Ubuntu the TeX dependencies are available through:

```sh
apt-get install latexmk texlive-latex-base texlive-latex-recommended \
  texlive-latex-extra texlive-science texlive-pictures texlive-fonts-recommended lmodern
```

The publication source uses `amsart`, Latin Modern, British hyphenation,
`microtype`, and `natbib` with `amsplain`. TikZ/PGFPlots and algorithm packages
remain dependencies of the historical and companion material.
Rust and Python requirements and benchmark commands are documented in
[`performance/README.md`](performance/README.md).

`make check` validates the preserved package, exact measurement and interval-bound
examples, Rust kernel and successor-collector tests. Application estimator and
bound integration tests run through the root Cargo checks. `make performance` writes a separate fresh
collection under `reports/performance/<UTC>/` at the repository root. The recorded
raw measurements remain immutable. The performance README explains how to choose
another output directory; figure regeneration remains a separate, explicit step.

The separate recorded runtime comparison used 100,000 acceptance records and 400,000 labour
contributions. Indexing reduced median total runtime for 1,000 queries from
2,787.159 ms to 23.051 ms, including construction and disposal, while one query
was slower with indexing. Ten-query results were mixed across paired runs.
These are measurements of a warmed, sequential, synthetic kernel on one host.

## Provenance and scope of the revision

The 29 October 2026 extension uses the recovered original LaTeX as its baseline.
The initial PDF transcription is not the source shipped here. Bibliographic
metadata for Hayden Van Der Post's *Data Science with Rust: The Speed & Safety
Revolution* was supplied by the user and has not been independently verified.
It is not evidence for the measured speed-up or correctness of the Rust code.

The imported work carries Revbench's [MIT licence and copyright notice](LICENSE-MIT).
The original proof records describe their historical executions; the current
implementation checks and figure review describe only the new extension.
See [validation notes](VALIDATION.md) for completed checks and their limits.
