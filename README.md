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

The [engarg and eal-mcp worked use case](docs/use-cases/engarg-eal-daily-2026-10-30/README.md)
applies the model to fresh public evidence, retains an original daily forecast,
and reports insufficient weekly history and adverse retrospective checks.

The [papers and reproducibility files](paper/README.md) include the new
[automatic forecasting companion](paper/forecasting/automatic-throughput-forecasting.pdf)
and [three ranked alternatives](paper/review/ranked-automatic-model-alternatives.md).
They also preserve the original measurement draft, its Lean evidence, and the
separate article on labour identification and design-based estimation.

The separate study-accounting path measures human labour in person-hours,
elapsed time to first acceptance and a weighted score. For a task accepted by
the deadline, the weighted score is its initial reference score multiplied by the
fraction of its accepted requirement still satisfied at an assessment a chosen
duration after first acceptance.
Tasks that miss the deadline receive a score of zero. Repository activity alone
leaves these quantities unresolved. The
[measurement protocol](docs/measurement-protocol.md) defines the required
observations. Automatic forecasting runs independently of this study path.

The [engineer's presentation](docs/presentations/observatory-model/README.md)
explains execution evidence, human-work measurement and the separate forecast
through diagrams and worked examples. It includes an editable PowerPoint, a PDF
handout and speaker notes; no advanced mathematics is assumed.

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

## Bound and estimate human labour

The current labour model is described in [*How Much Human Work Is Behind
AI-Assisted Software?*](paper/ai-assisted-software-engineering.pdf).

```sh
cargo run --locked -- bound-labour examples/joint-labour-bounds.json
cargo run --locked -- estimate-labour examples/model-assisted-labour-v2.json
```

`bound-labour` returns the exact minimum and maximum labour totals compatible
with the interval observations and each person's available time, together with
allocations achieving both limits. `estimate-labour` corrects a predicted total
using labour observed in randomly selected time intervals for individual people.
For each selected interval, the correction is observed labour minus predicted
labour, divided by that interval's known selection probability.
Predictions are calculated before sampling and remain unchanged whichever
sample is selected. Its v2 input also checks that the intervals partition each
person's declared observation period. Both commands run offline without
creating an evidence store. Their [identification](docs/labour-identification.md)
and [sampling](docs/design-based-estimation.md) contracts state the different
guarantees and measurement assumptions.

The [issue-linked effort pipeline](docs/effort-estimation.md) imports selected
ActivityWatch events, freezes activity-based predictions, samples short
intervals for human checks and prepares issue/PR-linked estimates. It includes
an offline review form and a synthetic end-to-end pilot; automatic activity
remains distinct from observed human effort.

The [linked EngArg/EAL pilot](docs/linked-pilot.md) connects real issues and PRs
to receipts from those repositories' CI jobs. Its private-runner template joins
execution evidence with sampled effort when observations exist, and reports
human effort as unavailable until then.

The [public observation-learning demo](docs/automatic-observation-learning.md)
imports declared effort from open and closed issues, retains observed revisions,
and shows declarations beside execution receipts. It runs on GitHub-hosted
Actions without a private runner. A separate synthetic example exercises guarded
model promotion; the single live declaration does not establish improved accuracy.

## Try a complete synthetic study

```sh
cargo run --locked -- import examples/study.jsonl
cargo run --locked -- snapshot --output reports/demo-snapshot.json
cargo run --locked -- analyse reports/demo-snapshot.json --output reports/demo
cargo run --locked -- verify
```

Open `reports/demo/index.html`. The synthetic tasks' weighted scores sum to
0.75 task equivalents, with total labour of 5 person-hours. Their ratio is 0.15
task equivalents per person-hour. The labour estimate includes an abandoned
task's sampled work. Sampling variance is unavailable, as the report
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
The general collector does not download binary workflow artifact ZIPs or job logs.
The separate `collect-github-executions` command downloads allowlisted execution
bundles and independently reconciles workflow attempts and jobs.
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

## Record tool executions in Actions

The [reusable action](action.yml) configures explicit command and Codex wrappers,
then publishes immutable start/completion receipts in a separate bundle for each
job. GitHub actors and trigger events remain separate from declared controllers,
assistant sessions and requirement links. Unreported usage and provenance remain
unknown; runtime never becomes human labour.

```sh
observatory --state /srv/observatory/study collect-github-executions \
  --repository owner/repository
observatory --state /srv/observatory/study import-executions artifact.zip
```

See [execution evidence](docs/execution-evidence.md) for schemas, wrapper usage,
durable collection, independent completion observation and the public-repository
pilot. Setup alone does not observe unwrapped tools or other jobs.

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
scope rule sets the reported weighted score to zero when scope changes before
the fixed end of labour observation. The measurement protocol also defines an
option that reports the paper's weighted score for the original requirement
despite later scope changes.

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
4. Reassess every counted accepted requirement after the chosen time since first
   acceptance, using the same assessment criteria. Record how much of the
   requirement is still satisfied. An absent assessment remains unknown.
5. Use `coverage` records to attest the actual lifecycle/effort observation frames.
   A few intervals do not establish a complete census. Sampled frames need known
   inclusion probabilities and recorded nonresponse.

For a 30-day acceptance deadline and reassessment 30 days after acceptance, a
task accepted on day 2 is reassessed on day 32. Labour is still counted through
day 60 from enrolment. A task accepted on day 40 receives a weighted score of
zero for the day-30 deadline and still contributes labour through day 60.
Productivity is `sum(Q)/sum(H)`, including unsuccessful work, with positive total
labour required. Probability-sampled labour uses Horvitz–Thompson point estimates;
the software does not invent confidence intervals without the sampling design.
Missing responses/frame coverage hold the complete labour result unavailable.

With complete direct coverage, reports also give conditional labour bounds for
boundary-crossing averages. Each interval uses its person's attention capacity;
the reported task and cohort sums are conservative because shared allocations
can couple intervals. These deterministic ranges are separate from sampling
uncertainty. A known sum of weighted scores and a positive lower labour bound
also permit a conditional cohort productivity range. An unavailable exact total remains
unavailable; exported bounds disclose floating-point approximation without
outward rounding.

This descriptive study path audits every counted accepted requirement. Quality
subsampling, assignment-weighted causal inference and prediction of labour or
weighted scores are outside its contract. The automatic forecasting module has
its own observed outcome, the number of merged pull requests. See the
[measurement protocol](docs/measurement-protocol.md).

## Estimate labour with frozen predictions

The standalone estimator starts with predictions for every time interval in a
complete list of people and observation periods. It corrects their total using
labour measured in randomly selected intervals with known selection probabilities.
The evidence store and cohort reports remain unchanged:

```sh
cargo run --locked -- estimate-labour examples/model-assisted-labour.json
```

The synthetic example has a predicted total of 4 person-hours, a Horvitz–Thompson
(HT) estimate of 6, and a difference estimate of 5.5. The difference estimator adds the
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

Historical cuts use observation time. An event collected after a cut remains
outside that cut even when the event itself occurred earlier. Snapshots fix
membership, source hashes,
protocols and build identities. Reports identify both snapshot producer and
current analyser revisions. Snapshot inputs for `analyse`, `forecast-snapshot`
and `score-forecast` use buffered JSON reading without the 128 MiB per-import
limit, so collected evidence can grow beyond an individual import. Integrity
checks still apply. Analysis keeps parsed records in memory; this is not a
constant-memory pipeline. Exact reproduction needs the same source revision,
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
