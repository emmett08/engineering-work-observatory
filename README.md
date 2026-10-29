# Engineering Work Observatory

A local Rust application for collecting evidence from `emmett08/eal-mcp` and
`emmett08/engarg`, then measuring human effort, elapsed acceptance time and retained
requirement output. It implements §§3 and 6 of *Human Effort, Delivery Speed and
Retained Output in AI-Assisted Software Engineering*, the 13 October 2026
methodological draft supplied for this project.

The [paper and reproducibility files](paper/README.md) include that original draft,
its LaTeX and Lean evidence, and an extended article with editable TikZ figures,
performance algorithms, and a separate Rust benchmark of repeated accounting queries.

The software implements descriptive accounting. Repository activity does not
establish human labour, AI exposure, acceptance, or retained correctness. Those
outcomes need explicit task records and independently collected measurements.
There is no automatic causal AI-effect estimator or staffing predictor.

The default scope rule assigns zero protocol credit when scope changes before
the fixed labour horizon closes. This predeclared penalty is separate from the
paper's retention endpoint. The [measurement protocol](docs/measurement-protocol.md)
defines the distinction and the original-scope reporting option.

## Install and run

Requires Git and Rust 1.99.1; `rust-toolchain.toml` pins compiler, formatter and
Clippy. SQLite is bundled, so a database server is unnecessary.

```sh
cargo build --locked --release
./target/release/observatory --help
./target/release/observatory init
```

Commands default to a private `.observatory/` directory. Use `--state PATH` to
choose another directory. Keep it and exported reports out of source control:
they can contain private requirements, source comments and raw run data.

## Try a complete synthetic study

```sh
cargo run --locked -- import examples/study.jsonl
cargo run --locked -- snapshot --output reports/demo-snapshot.json
cargo run --locked -- analyse reports/demo-snapshot.json --output reports/demo
cargo run --locked -- verify
```

Open `reports/demo/index.html`. The example is synthetic: 0.75 retained task
equivalents / 5 person-hours = 0.15 task equivalents per person-hour. It includes
an abandoned task's sampled effort. Sampling variance is unavailable, as the report
states. Reimporting adds no duplicate facts. Snapshot and report paths must be new.

## Collect repository evidence

Set `GH_TOKEN` through your shell's secret mechanism, using a read-only credential
with repository access. Do not put it in configuration or command arguments.
Public unauthenticated collection is supported with a smaller API allowance.

```sh
cargo run --locked -- collect-github
cargo run --locked -- collect-github --repository emmett08/engarg
cargo run --locked -- collect-git /path/to/eal-mcp --repository emmett08/eal-mcp
cargo run --locked -- collect-git /path/to/engarg --repository emmett08/engarg
```

GitHub collection preserves paginated PR, issue, review, comment and workflow
attempt/job evidence. Endpoint coverage is separate from engineering outcomes.
Partial responses/errors are retained; incomplete collection returns nonzero.
Refreshes use validated, credential-scoped ETag/Last-Modified caches, revisiting
every page instead of assuming a PR timestamp covers all related events.
Rate, page, attempt, body and request bounds never become complete-history claims.
The collector does not download binary workflow artifact ZIPs or all job logs.

Local Git collection retains refs, the reachable graph, commit objects,
per-parent patches and Cargo/workflow definitions across revisions. It reads Git
objects without executing repository code or hooks. It does not fetch remotes;
update local clones through your normal Git workflow before collecting. Previously
retained bytes remain available after a later force push.

Schedule collection/imports with your existing scheduler. One state directory is
for one researcher's credential context. Ingestion never builds or runs the
observed projects or invokes an assistant.

## Import EngArg and assistant records

Reports, audit JSONL and authoring checkpoints use a manifest providing stable
execution identity. Filenames, identical responses and branch names are not IDs.

```sh
cargo run --locked -- import-engarg examples/engarg-cursor-report.json \
  --manifest examples/import-manifest.json
cargo run --locked -- import-eal /path/to/.eal/runs.sqlite3 --namespace my-workspace
```

The adapter understands HTTP providers and Cursor, Augment/Auggie and generic
command metadata inspected at `876f720ec1abe951e391e8ff7bfc9a056a5e333b`.
At inspection that revision was PR #45, while `main` remained
`e8b21c929f2aa9e8b870131e534e354926354094`. Record the actual producing revision;
support for its schema does not assert that a branch was merged. Unknown schemas
are rejected.

Representations of one execution share `provenance_chain_id` and `session_id`;
resumed invocations have distinct `attempt_id`s. Multi-call audits/checkpoints
require `call_ordinals`, since completion order can differ from report order.
Parent calls reuse their ordinals; only new calls receive new ones. Known
contradictory content/usage is rejected; missing optional metadata can be enriched.
See the [data contract](docs/data-contract.md).

Configured assistant selection is distinct from an observed model identity.
Unreported tokens and internal inference counts remain unknown. Imported calls
describe **product execution**, never automatically development exposure or human
effort. Transformed records omit prompts/response text; raw artifacts retain them
privately. EAL imports take a read-only, consistent online backup including WAL;
they also accept explicit `eal.records-export/1` JSON exports. EAL run identities
never become development-session identities.

## Record a real study

Each JSONL row follows `{"id":"stable-id","kind":"task","data":{...}}`.
`{"schema":"observatory.bundle/1","records":[...]}` JSON is also accepted.
Use `examples/study.jsonl` as the executable reference. Import related records
together: validation and insertion share one immediate SQLite transaction.
Identical IDs/content are idempotent; changing an existing ID's content is rejected.
Append a `correction` record with `target_id`, complete `replacement_data`,
`corrected_at` and `reason` to revise a typed study observation while retaining
its original evidence. Later corrections explicitly name their predecessor through
`supersedes_correction_id`; branching chains are rejected. Analysis applies only
corrections dated and retained by its knowledge cut, preserves target ID/kind,
and reports the correction history. Source records are versioned separately and
cannot be correction targets. See the [correction contract](docs/data-contract.md#corrections-and-versioning).

1. Freeze protocol, eligibility, requirement, score and scope at intake.
2. Record all-role effort, including unsuccessful work. Direct or probability-
   sampled intervals are supported. One person's simultaneous allocations cannot
   exceed one; pair work counts both people.
3. Record acceptance separately from merge/deployment, and abandonment separately
   from loss of observation.
4. Assess every counted accepted requirement after follow-up, against the frozen
   rubric. Missing assessment is explicit; passing CI does not imply retention=1.
5. Use `coverage` records to attest the actual lifecycle/effort observation frames.
   A few intervals do not establish a complete census. Sampled frames need known
   inclusion probabilities and recorded nonresponse.

For a 30-day deadline and 30-day follow-up, acceptance on day 2 means retention
assessment on day 32, but labour collection still ends on day 60 from enrolment.
Acceptance on day 40 yields zero deadline output while retaining labour.
Productivity is `sum(Q)/sum(H)`, including unsuccessful work, with positive total
labour required. Probability-sampled labour uses Horvitz–Thompson point estimates;
the software does not invent confidence intervals without the sampling design.
Missing responses/frame coverage hold the complete labour result unavailable.

This descriptive implementation audits every counted accepted requirement.
Quality subsampling, assignment-weighted causal inference and prediction are
outside its contract. See [measurement protocol](docs/measurement-protocol.md).

## Freeze and reproduce

```sh
cargo run --locked -- snapshot --as-of 2026-10-29T23:59:59Z \
  --output reports/cut-20261029.json
cargo run --locked -- analyse reports/cut-20261029.json \
  --output reports/cut-20261029
cargo run --locked -- simulate examples/retry-scenario.json
```

Historical cuts use observation time: collecting an old event today does not make
it available in yesterday's cut. Snapshots fix membership, source hashes,
protocols and build identities. Reports identify both snapshot producer and
current analyser revisions. Exact reproduction needs the same source revision,
`Cargo.lock`, snapshot and pinned toolchain.

Reports contain JSON, escaped offline HTML, a snapshot and CSV records. They are
private evidence exports, not anonymised aggregate publications. Snapshot hashes
check exported records; `verify` checks original artifacts recursively. Back up
the state directory and snapshots together. Synthetic retry/gate calculations
are separate from empirical cohort accounting and are not fitted AI effects.

## Design and verification

See [architecture](docs/architecture.md) and [ADR 0001](docs/adr/0001-explicit-rust-pipeline.md)
for the explicit Rust pipeline and the Aspect decision. Set `RUST_LOG=info` for
collector diagnostics; operational logs are separate from research evidence.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo build --locked --release
```

CI runs these on Linux and macOS. Tests use synthetic records, mock HTTP and
temporary Git/SQLite sources without calling assistants or spending model tokens.
Passing checks verifies software behaviour, not an empirical productivity claim.

Existing repository licence: GPL-3.0-only.

## References and next improvements

[references.bib](references.bib) includes the user-supplied
`vanderpost2024datascience` book entry. Its exact edition and contents have not
been independently verified; [source notes](docs/references.md) record the
boundary. The [prioritised roadmap](docs/roadmap.md) proposes analysis-ready
exports, measurement worklists, descriptive breakdowns and design-specific
uncertainty, with acceptance criteria grounded in the existing code and model.
