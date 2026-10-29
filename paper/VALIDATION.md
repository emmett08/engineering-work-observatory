# Validation of the 29 October 2026 extension

The original package and the new extension have separate verification boundaries.

## Original package

- All 83 imported Git blobs match `revbench` commit
  `64314d60104268bffbb6219912657efeb2a11098` at `paper/effort-speed-ai/`.
- The original PDF's SHA-256 matches the supplied attachment, as recorded in the
  paper README.
- `make -C effort-speed-ai check` passes: 82 manifest hashes, matching manuscript
  copies, 54 inventory entries, 43 unique Lean exports, and receipt validation.
- `make baseline` builds 17 pages. PyMuPDF text extraction from every page matches
  extraction from the supplied PDF exactly. Compiler versions and PDF metadata
  differ; byte identity is claimed only for the preserved original PDF.
- Lean was not re-executed in this session. The successful proof-build and axiom
  audit logs are retained historical evidence, not a new proof run.

## Extended article and algorithms

- `make article` builds 20 pages with pdfLaTeX/BibTeX/latexmk, using TeX Live
  2025/dev and the standard `texlive-science` algorithm packages.
- The final LaTeX log has no undefined references/citations or overfull boxes.
  The bibliography has 12 entries, including the user-supplied Van Der Post book.
  BibTeX's missing-pages warning for the original online-first `cui2026` reference
  is retained rather than inventing pagination.
- Both algorithms, their contracts and generated List of Algorithms entries were
  inspected after compilation. The `amsart` list compatibility adapter is in the
  main preamble. Normal article numbering is used.
- Structural checks covered semantic algorithmic commands, numbered steps,
  caption/label order, unique labels, callouts, balanced control flow, package
  ordering and hanging input/output contracts. The algorithms were compared with
  the Rust implementation and its analytical argument.
- The algorithm skill's referenced `check_algorithm_style.py` was unavailable
  through the supplied skill resource and local environment. These checks are a
  project-level review, not a claim that that bundled checker ran.

## Rust and measurements

- Seven tests pass, including inclusive boundaries, duplicate ages, unsorted
  queries, empty streams, zero denominators, horizon overflow, wide totals and
  immutable snapshots. Formatting, Clippy with warnings denied, and release build
  also pass.
- All 60 measured pairs and their warmups produced exactly equal integer totals.
  No measured observations were excluded or rerun. The source hashes and raw CSV
  hash match `performance/results/environment.json`.
- Indexing meets the predeclared high-query runtime criterion on the recorded
  synthetic workload. The one-query regression and mixed ten-query pairs remain
  in the paper, raw data, figure and performance README.
- An independent code review found no semantic-equivalence or timing-boundary
  defect. This is not a Rust-to-Lean refinement proof or an AI-effect estimate.

## Figures

The three TikZ reconstructions were compiled and visually compared with the
original plots. The new performance figure retains its calculation inputs,
alternative composition, specification, vector export, mechanical audit and
independent semantic review. See
[`performance/figure/README.md`](performance/figure/README.md) for reproduction.
Its final integrated manuscript page was inspected for labels, log scales,
units, captions, clipping and the adverse/mixed results.

The figure skill's bundled validator/renderer scripts were also unavailable.
The retained schema checks and local renderer describe their narrower scope
explicitly; mechanical checks and reviewer judgements remain distinct.
