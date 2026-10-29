# Implementation and verification boundaries

The initial repository contained its licence, ignore file and a short README at
`35d67ed7be690dc60b5fd605a74c213eed18aa15`. The implementation preserves the
licence and adds a Rust 2024 application with Rust 1.99.1, `rustfmt`, ordinary
Clippy warnings denied in CI, and a committed application lockfile.

## Structural review

* **EXTRACTED:** GitHub HTTP/pagination/cache and local Git inspection occupy
  separate collector modules because their external contracts change separately.
* **EXTRACTED:** EngArg assistant/audit/checkpoint import and EAL SQLite/export
  import have separate adapters behind a small module facade.
* **EXTRACTED:** Labour integration, lifecycle/retention, coverage and cohort
  summarisation are independent calculation modules. `analyse` assembles results.
* **EXTRACTED:** Direct-interval and productivity-bound arithmetic occupies
  `analysis/bounds.rs`; cohort eligibility and aggregation remain in the existing
  cohort calculation.
* **EXTRACTED:** Standalone model-assisted estimation separates typed frame
  validation and SRS variance policy from total estimation. No trait or numeric
  framework is introduced for these two declared sampling designs.
* **EXTRACTED:** Immutable correction-chain handling is separate from validation
  of the effective study graph. Reporting separates usage accounting from HTML.
* **RETAINED:** CLI `execute` dispatches commands and wires their inputs/outputs;
  it does not implement the source-specific algorithms or model equations.
* **EXTRACTED:** `ingestion::persist_collection` owns the shared raw-byte, manifest,
  insertion and coverage boundary. CLI profiles select full or throughput policy;
  only explicitly omitted optional sources may be absent in the lightweight path.
* **RETAINED:** Store transaction and recursive-integrity helpers remain with
  storage because they depend on its ownership and persistence invariants.
* **RETAINED:** Small date, duration, escaping and source-format helpers remain
  beside their consumer. No traits, factories or weaving were added solely to
  display a design pattern.

## Executed checks

The implementation was checked in the Linux development environment with:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo build --locked --release
```

The original 67-test unit and integration suite covered incomplete pagination,
conditional refresh, credential-origin isolation, workflow reruns, consistent WAL
snapshots, assistant identity/resumption, null usage, sampled allocations,
abandonment, horizons, scope changes, correction histories, concurrent import
validation, recursive artifact integrity and the complete CLI report path.

Live smoke checks also collected local Git history from both source projects and
GitHub evidence from this repository, then verified retained evidence and froze
an offline report. Research evidence produced by those checks is outside Git.
The live API check exposed a trailing-slash repository URL defect; a focused
regression test now checks the canonical metadata endpoint.

CI is configured to execute the same checks on Linux and macOS. A local Linux pass
does not assert a remote CI or macOS pass; inspect the actual workflow run.

## What these checks establish

They establish the exercised software behaviour. They do not establish accurate
human diaries, truthful coverage attestations, reliable quality rubrics, calibrated
Standard Coding Hours or a causal AI effect. The first study should check those
measurement properties. The implementation supports all-task quality assessment
and direct/HT labour point accounting. The separate complete-frame
`estimate-labour` command adds frozen-prediction difference estimation and SRS
design variance under its explicit contract; generic cohort reports still supply
HT points without design variance. Comparative causal estimators remain outside
the implemented contract.

Database views expose immutable original study payloads. Corrected results are
the effective view in the analysis report, accompanied by correction history.
Snapshots select each fact's latest source receipt available at their cut; all
earlier receipts remain in SQLite. Source-coverage summaries select the latest
available receipt for each source scope.

The sharp-identification revision adds conditional direct-observation labour and
productivity ranges, an exact-rational graph reference checker, and the standalone
statistical estimator. Existing point results remain unchanged except that an
unfinished intersecting observation no longer permits a complete zero at an
earlier evidence cut. The failure was reproduced on `b89a258` in an isolated
worktree before the repaired regression was accepted. Output ranges disclose
ordinary floating-point approximation without outward rounding. They are neither
sampling confidence intervals nor results from a general joint LP solver.

Current commands, suite counts, source/receipt checks, manuscript inspection and
review scope are recorded in [the paper validation notes](../paper/VALIDATION.md)
and the revision's [decision record](../paper/review/sharp-identification-2026-10-29.md).


## Automatic forecasting revision

The forecasting extension adds source-bound history, complete calendar bins,
explicit empirical methods, retrospective comparator evaluation and unchanged
prospective-forecast scoring. Its 69 additional tests bring the application suite
to 183 passing tests. Mock CLI tests exercise automatic collection, archived and
exported originals, replay, pending scores, unsupported history, partial-source
failure and pre-fetch overwrite protection. The JSON parser's `float_roundtrip`
feature preserves exact verification of saved fitted probabilities.

The [forecast validation record](../paper/forecasting/VALIDATION.md) distinguishes
software checks, independent exact/decimal score checks, the failed public-reference
promotion and the still-unvalidated user forecast. The [mesh decision record](../paper/review/mesh-forecasting-2026-10-29/README.md)
and [three ranked alternatives](../paper/review/ranked-automatic-model-alternatives.md)
retain model choices and unsupported future-bottleneck claims. The existing paper,
proof package and historical performance observations remain unchanged.
