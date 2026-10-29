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
* **EXTRACTED:** Immutable correction-chain handling is separate from validation
  of the effective study graph. Reporting separates usage accounting from HTML.
* **RETAINED:** CLI `execute` dispatches commands and wires their inputs/outputs;
  it does not implement the source-specific algorithms or model equations.
* **RETAINED:** CLI `persist_collection` owns one ingestion boundary: retain raw
  bytes, build their manifest, insert facts and return the coverage outcome.
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

The unit and integration suite has 67 tests. It includes incomplete pagination,
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
and direct/HT labour point accounting; sampling-design uncertainty and comparative
causal estimators require a separately specified extension.

Database views expose immutable original study payloads. Corrected results are
the effective view in the analysis report, accompanied by correction history.
Snapshots select each fact's latest source receipt available at their cut; all
earlier receipts remain in SQLite. Source-coverage summaries select the latest
available receipt for each source scope.
