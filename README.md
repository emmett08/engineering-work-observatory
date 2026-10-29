# Engineering Work Observatory

A local Rust application that automatically collects GitHub pull requests,
forecasts team merge counts and reports observed queue pressure. The default
portfolio is `emmett08/eal-mcp` plus `emmett08/engarg`; other repositories can be
selected. Forecasting needs no task labels, time diaries, AI-use classifications
or manual completeness attestations.

The forecast counts merged PRs. Current open, draft, requested-review and aged-PR
counts help inspect the queue, but do not identify its causal bottleneck. The
default model is an expanding empirical reference; predictive skill on a new
team remains unestablished. See [automatic forecasting](docs/automatic-forecasting.md)
for the assumptions, source boundaries and scoring workflow.

The [papers and reproducibility files](paper/README.md) include the new
[automatic forecasting companion](paper/forecasting/automatic-throughput-forecasting.pdf)
and [three ranked alternatives](paper/review/ranked-automatic-model-alternatives.md).
They also preserve the original measurement draft, its Lean evidence, and the
separate article on sharp identification and design-based estimation.

The separate study-accounting path measures human effort, elapsed acceptance
time and retained requirement output from explicit observations. Repository
activity alone does not establish those quantities, an AI causal effect or a
staffing requirement. The [measurement protocol](docs/measurement-protocol.md)
defines that path; it is not a prerequisite for automatic forecasting.

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

## Forecast team throughput automatically

For private repositories, provide a read-only `GH_TOKEN` through your shell's
secret mechanism. Public unauthenticated collection is also supported.

```sh
cargo run --locked -- forecast-team
cargo run --locked -- forecast-team --repository owner/repository --horizon week
```

Each run collects repository metadata and every PR-list page, retains source
evidence, freezes a snapshot and saves an immutable forecast bundle under
`.observatory/forecasts/`. It also scores earlier archived forecasts when their
entire target period is observable; scoring results are appended under
`.observatory/forecast-scores/`. The command prints the forecast, current queue
pressure and prior-forecast score statuses. Run it from an existing scheduler
for continuing collection and evaluation. No per-PR input is needed.

The default target is the next full UTC day. `--horizon week` forecasts the next
Monday-based week; `--horizon fortnight` forecasts a 14-day period beginning on
the next Monday. Four complete target-width bins are required: four days, four
weeks or eight weeks, respectively. Insufficient history produces an unavailable
forecast. Daily counts are never scaled into weekly predictions.

The reference distribution uses all supported complete bins in the selected
portfolio's history. Its reported quantiles are empirical ranges, without a
guarantee of future coverage. Optional `--output FILE` exports the same immutable
bundle while retaining the normal archive. Earlier bundles are never rewritten.

For offline verification and scoring:

```sh
cargo run --locked -- snapshot --output reports/later-snapshot.json
cargo run --locked -- score-forecast \
  .observatory/forecasts/FORECAST.json reports/later-snapshot.json
```

The later snapshot must contain a successful collection covering the complete
target. Taking a new snapshot without collecting does not extend observation.
See [automatic forecasting](docs/automatic-forecasting.md) for replay, historical
comparison, missing-data behaviour and the initial evidence.

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
cargo run --locked -- collect-github --profile throughput --repository emmett08/engarg
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
The `throughput` profile collects only repository metadata and PR inventories,
with an automatic summary of required-source coverage. This is the profile used
by `forecast-team`; review submissions, CI and timeline history are not collected.

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

These measurements belong to the optional study-accounting path. The default
scope rule assigns zero protocol credit when scope changes before the fixed
labour horizon closes; the measurement protocol explains the original-scope
reporting option and its distinction from the paper's retention endpoint.

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

With complete direct coverage, reports also give conditional labour bounds for
boundary-crossing averages. Each interval uses its person's attention capacity;
the reported task and cohort sums are conservative because shared allocations
can couple intervals. These deterministic ranges are separate from sampling
uncertainty. Known retained output and a positive lower labour bound also permit
a conditional cohort productivity range. An unavailable exact total remains
unavailable; exported bounds disclose floating-point approximation without
outward rounding.

This descriptive study path audits every counted accepted requirement. Quality
subsampling, assignment-weighted causal inference and prediction of labour or
retained output are outside its contract. The automatic forecasting module has
its own observable PR-count endpoint. See the
[measurement protocol](docs/measurement-protocol.md).

## Estimate labour with frozen predictions

The standalone estimator combines predictions for a complete person-interval frame
with probability-sampled labour observations. It leaves the evidence store and
cohort reports unchanged:

```sh
cargo run --locked -- estimate-labour examples/model-assisted-labour.json
```

The manufactured example has a prediction total of 4 person-hours, an HT estimate
of 6, and a difference estimate of 5.5. The difference estimator adds the
sampling-weighted observed prediction errors to the full-frame prediction total.
Predictions must be fixed before evaluation sampling; the JSON requires their
provenance, a complete-frame attestation, inclusion probabilities, and explicit
observed/unselected/nonresponse statuses. Nonresponse is rejected rather than
treated as zero. Keep the input file with its output for reproduction.

`general` designs return point estimates with unavailable variance. Declared simple
random sampling without replacement validates the sample size and equal inclusion
probabilities, then estimates HT and residual-based variances when at least two
units were observed. A census has known zero sampling variance; a noncensus sample
of one does not identify variance. The command reports standard errors without
inventing a confidence interval or causal contrast. An unfavourable predictor can
increase variance, and a signed difference estimate can be negative. No clipping
or automatic substitution into a productivity ratio is performed. See the
[estimator contract](docs/design-based-estimation.md).

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
