# Validation of the 29 October 2026 extension

The original package and the new extension have separate verification boundaries.
The earlier sections retain the validation history of PR #2. The final section
records the subsequent sharp-identification revision and its current PDF.

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

- `make article` builds 22 pages with pdfLaTeX/BibTeX/latexmk, using TeX Live
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

## Debate-driven revision

Six live `gpt-6.1-sol` workers at `max` reasoning effort completed an independent
review and paired cross-examination. The [decision record](review/debate-2026-10-29.md)
distinguishes accepted repairs, withdrawn criticisms and optional extensions.

The sampling-validation identities and continuous-bin envelope received a separate
mathematical review. Exact rational checks cover the noisy-HT counterexample and
2,304 manufactured allocation/cutoff cases, including positive-denominator ratio
bounds. These are analytical arguments with finite computational checks; they add
no Lean declarations or empirical observation evidence.

Five application regressions cover ambiguous average allocations at the labour
boundary, separately observed splits, exact direct zero/one allocation exceptions,
submillisecond subdivision, and a long duration. The two original failure cases
were reproduced before repair. The current point estimate is withheld when an
interior average does not identify the contribution inside the boundary.

The successor collector has a separate output directory and Cargo-artifact/binary
binding. Collector tests cover external/cached artifact selection, missing or
ambiguous executables, immutable-output protection and new-summary isolation.
The recorded collector, seven measured source hashes, raw measurements and
published figure bindings remain unchanged; future collections are separate
experiments. The successor changes support reproduction and make no new runtime
improvement claim.

The full application gate passes: formatting, Clippy with warnings denied,
79 tests across all targets, documentation tests, and the release build. The
paper gate passes its original receipt checks, seven kernel tests, 11 collector
tests and exact model examples. An actual build with an external
`CARGO_TARGET_DIR` and explicit native `CARGO_BUILD_TARGET` selected Cargo's
reported executable; one seed-17, single-query invocation passed protocol-row
checks. This smoke check is separate from the preserved 60-pair experiment.

The final source passed contribution/scope, mathematical, exposition, attribution
and rendered-layout review. The 22-page PDF has no unresolved references or
overfull boxes. New analysis pages and all-page layout were inspected after
integration. This review is agent screening rather than journal peer review.

## Astra ultra review and three editorial passes

The follow-up used six live `gpt-6-astra` workers with `ultra` reasoning effort,
independent reviews and three paired debates. Its [decision record](review/astra-ultra-2026-10-29.md)
separates bounded repairs from optional extensions. New regressions reproduced
the output-alias, malformed-row, fragmented-coverage and repeated-observation
failures before repair. Scope-endpoint tests document the existing predeclared
penalty without changing its numerical behaviour.

`write-for-experienced-engineers` was applied three times to the complete paper.
Each pass has its own source diff, preserved protected formal blocks, labels,
citation occurrences and references, and compiled successfully. Independent
mathematics and evidence reviews accepted the final wording. The final 22-page
PDF has no undefined references/citations or overfull material, and its extracted
text matches the inspected final preview.

The local application gate passes formatting, Clippy with warnings denied,
all 79 tests, documentation tests and release compilation. The paper gate passes
the seven kernel tests, 11 successor-collector tests and exact model examples,
including the paired squared-loss checks. All seven measured source hashes,
retained raw data, figure bindings and historical proof evidence remain intact.
No performance recollection or Lean execution was performed for these repairs.

A prior macOS CI failure was consistent with an accepted mock socket inheriting
nonblocking mode. The mock now explicitly uses blocking reads and verifies a
complete request header before routing. All 12 collector tests pass locally.
The new macOS workflow run remains the direct platform confirmation.

## Sharp identification and design-based estimation

This revision starts from PR #2 head `b89a2584ef34637b71f5036a67673250b382d4e2`
on a separate branch. Six separate agents were configured as `gpt-6-astra` with
`ultra` reasoning effort; their responsibilities, exchanges and accepted
corrections are recorded in the [decision record](review/sharp-identification-2026-10-29.md).
Source support is documented in the [primary-source audit](review/sharp-identification-sources-2026-10-29.md).

The Linux application gate passes the pinned Rust 1.99.1 formatter, all-target
Clippy with warnings denied, all **114** unit/integration tests, documentation
tests and release compilation. The suite includes 14 estimator tests, four
standalone-estimator CLI tests and 14 bounds integration tests. Finite sample
enumeration verifies unbiasedness and both variance improvement and deterioration;
numerical regressions cover large common prediction offsets, including all 35
SRS samples of a seven-unit population. General sampling retains its supplied
weights, whereas SRS centres predictions before the canonical expansion. These
checks do not establish measurement accuracy or exact real-number arithmetic.

The release executable's synthetic estimator smoke returns prediction total 4,
HT total 6 and difference total 5.5 person-hours, with estimated SRS difference
variance 0.125 squared person-hours. It creates no evidence store. An old
unsupported complete zero was reproduced on the exact base in an isolated
worktree; the current regression requires an unavailable point and bounds when
an intersecting observation has not ended by the evidence cut.

`make check` passes the historical package/inventory/receipt checks, the existing
2,304 exact measurement-bin cases, the new rational reference checks, seven Rust
kernel tests and 11 Python collector tests. The new checker covers 284 scalar
cutoff cases, 5,445 exact-observation target cases from 141 enumerated allocation
histories, 1,500 bounded-observation cases (150 infeasible), seven explicit
inconsistencies and shared-capacity/window fixtures. It uses `fractions.Fraction`
without a floating optimiser. CI now runs both exact Python example checkers in
addition to its application gates.

The independent reviewer prepared separate cases before inspecting author code,
then compared 5,000 graph cases (the 2,500-case corpus, including 1,834 infeasible
systems, and its fractional transformation), 241 external implementation requests
and nine additional productivity-bound fixtures. These are finite agent
screening results, not an empirical replication or universal proof. The independent
review record states its scope and remaining boundaries.

`make article` builds the revised **29-page** PDF; the main argument ends on page
15. Five ordered contribution, mathematics, exposition, literature and typesetting
audits were repeated on stable text with no further material corrections. All
pages were rendered and inspected. The final root build's extracted text matches
the separately inspected author build exactly. There are 87 unique labels and
17 cited bibliography keys, no unresolved references/citations, duplicate labels,
overfull boxes or build errors. One harmless underfull text line remains in the
proof concordance. The original diagrams and measured runtime figure remain in
the appendices.

The local TeX installation lacked `algorithm` and `algorithmicx`. Official CTAN
packages, including the generated `algorithm.sty`, were supplied through a
temporary `TEXINPUTS` path; manuscript source uses their standard interfaces.
The skill-referenced `scripts/check_manuscript.py` was unavailable both locally
and through its full skill-resource URI. Explicit expanded-source/reference
checks, clean compilation, log inspection and all-page rendering were performed
instead; the unavailable checker is not counted as executed.

Final `main.tex` SHA-256:
`470af6ccd815eec3d8ef83e54c2c5580a48366af46339f2ca04f3d4f2a78bb7f`.
The committed revised PDF SHA-256 is
`91ac161ec6ba1505fff8e637bd6f4f767a1dcae87a9236be0597843bc75c8080`.
Build timestamps can change PDF bytes without changing the inspected text.

All 83 historical package files, seven measured source hashes, raw measurement
hash and figure inputs remain unchanged. No performance recollection or Lean
execution was performed. The new analytical results and Rust code remain outside
the historical 43-theorem Lean receipt. The application exposes marginal labour
and conservative aggregate/productivity formulas; the general joint LP is proved
but is not implemented as an application solver. Float bounds disclose their lack
of outward rounding. Empirical calibration, field-study feasibility and external
specialist peer review remain open.
