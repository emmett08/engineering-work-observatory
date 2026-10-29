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

## What software-work telemetry identifies

Read the [extended article](ai-assisted-software-engineering.pdf), built from
[`article/main.tex`](article/main.tex) and
[`article/references.bib`](article/references.bib). The revised paper centres on
labour bounds from interval observations and bounded attention,
an exact finite-cell reduction, and model-assisted design-based estimation.
A compact marked-history representation preserves the separate acceptance,
retention and labour horizons. The elementary execution examples and retained
runtime experiment support appendices.

The application reports conditional direct-observation labour and productivity bounds and provides a
separate `estimate-labour` command for a declared probability sample with frozen
predictions. Its SRS variance calculation is separate from the cohort report's
HT point estimates. Exact interval masses, statistical estimates and conditional
bounds remain distinct. These additions establish neither an empirical AI
effect nor the accuracy of underlying labour and quality observations. The new
mathematics and Rust code are outside the original Lean proof receipt's coverage.

The three original diagrams have editable reconstructions in [`tikz.tex`](tikz.tex).
The performance appendix retains its algorithms, measured timings and figure
showing the cost of preprocessing; those data were not recollected.

The [six-agent review](review/debate-2026-10-29.md) adds explicit sampling-noise
validation and bounds connecting complete labour bins to continuous accounting.
These analytical additions are also outside the historical Lean receipt. The
application now preserves fractional-second effort and withholds ambiguous
boundary-crossing allocations rather than silently interpolating them.

The subsequent [Astra ultra review](review/astra-ultra-2026-10-29.md) records
the further observation and summary-validation repairs, followed by three
whole-paper applications of `write-for-experienced-engineers`. Each editorial
pass has a separate commit and mathematical/citation fidelity check.

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

`make article` compiles the extended source to `build/main.pdf` and copies it to
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

The main source uses `amsart`, Latin Modern, British hyphenation, `microtype`,
TikZ/PGFPlots, `algorithm`/`algpseudocode`, and `natbib` with `amsplain`. Algorithms
are numbered continuously because the article class has no chapter counter.
Rust and Python requirements and benchmark commands are documented in
[`performance/README.md`](performance/README.md).

`make check` validates the preserved package, exact measurement and interval-bound
examples, Rust kernel and successor-collector tests. Application estimator and
bound integration tests run through the root Cargo checks. `make performance` writes a separate fresh
collection under `reports/performance/<UTC>/` at the repository root. The recorded
raw measurements remain immutable. The performance README explains how to choose
another output directory; figure regeneration remains a separate, explicit step.

The recorded comparison used 100,000 acceptance records and 400,000 labour
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
