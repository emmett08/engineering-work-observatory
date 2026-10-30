# Architecture

The observatory is a local Rust CLI for automatic PR-throughput forecasting and for separate task-level measurement of human effort, acceptance time and retained output. A requirement is the accounting unit. Commits, pull requests, assistant calls and verification runs provide evidence about that requirement; they do not replace it.

## Implementation boundaries

The application uses Rust 2024, the pinned Rust 1.99.1 toolchain, `reqwest` blocking HTTP, `rusqlite` with bundled SQLite, Serde records, SHA-256 evidence identifiers and `tracing` diagnostics. `Cargo.lock` fixes resolved application dependencies. The repository retains its existing GPL-3.0 license; the package declares `GPL-3.0-only`.

One process performs a command's collection or import. This makes request ordering, rate limits and database transactions inspectable without an async runtime, queue service or hosted database. Modules separate source collection, imports, model validation, storage, calculations, snapshots and reporting. A new source needs an adapter, not a change to the accounting equations. Introduce a shared trait only when real callers need interchangeable implementations with the same contract.

Operational logs explain what the program did. Research records retain the evidence used in an analysis. A log message is not a substitute for a source receipt or a human-effort observation.

## Three retained layers

| Layer | Representation | Contract |
|---|---|---|
| Source evidence | Exact imported or retrieved bytes under SHA-256 identifiers | Content is retained before facts refer to it; existing objects are checked rather than overwritten. |
| Study records | Validated, typed payloads in SQLite with source observations | A record ID identifies immutable content. New source versions have new IDs; explicit correction chains produce an effective study view while retaining the originals. Reimporting identical content does not add another fact. |
| Analysis snapshot | Frozen records, provenance, configuration and calculated output | A snapshot fixes its evidence cut and inputs so later collection does not change the earlier result. |

The storage tables hold record envelopes and their observed provenance, with indexes and queryable `tasks`, `effort_intervals` and `source_records` views. Relationships and domain validation live in the typed study model. Generic `source_record` evidence retains external objects and collector provenance without pretending that every GitHub object is already a task or an independently verified outcome. Task links, defects, scope-change events and assessment metadata make the investigator's interpretation explicit.

Corrections carry full replacement data, target identity, date, reason and an explicit predecessor for later links. The original ID and kind remain stable in the effective view; source records and correction envelopes cannot be correction targets. Branching chains and invalid effective study graphs are rejected. Analysis applies only the correction chain available and dated by its cut, reports that history, and leaves earlier snapshots intact. The raw SQLite views expose original stored records, not a silently rewritten corrected table. See the [correction contract](data-contract.md#corrections-and-versioning).

Source objects are written atomically to the configured data directory. Complete graph validation and batch insertion share an immediate SQLite transaction. A rejected batch can leave an unreferenced retained source object; it must not leave a partially accepted set of study records. Integrity verification checks record hashes and recursively checks linked source objects. Database schema versions are checked on opening. The code repository contains implementation, schemas, protocols and synthetic fixtures; collected work traces belong in the separately configured data directory.

## Collection and identity

GitHub collection retains repository identity, revision, request context, source content and coverage. Workflow run attempts remain distinct. PR text claiming validation remains a source statement until linked to an actual observation. A failed or incomplete request cannot establish an observed zero, passing verification or exhaustive absence. API pagination and retries are explicit collector policies.

The CLI's persistent conditional HTTP cache revalidates enumerated pages using ETag/Last-Modified headers when available. Cache entries are integrity checked and scoped to the credential context without saving the token itself. A 304 receipt retains both revalidation metadata and the cached body's original provenance; it is not relabelled as a fresh 200 response. This reduces repeated transfers but is not an event-stream completeness guarantee or a source deletion detector.

EAL and EngArg imports preserve their own source identities. A live EAL SQLite store must be read through a consistent backup, not copied without its WAL state. EngArg reports, checkpoints and audits may contain the same call; call identity and retained provenance prevent repeated representations from becoming repeated usage.

Assistant usage records distinguish development assistance from executing an AI product. EngArg's native assistant providers can leave internal model identity or token usage unobserved. Preserve that absence. Requested settings, returned settings, observed tokens, estimated tokens, elapsed execution and human person-hours are different measurements.

A task can span several PRs and both repositories. Link these records explicitly. EngArg depends on EAL; repository count and commit count do not supply independent treatment assignments. Preserve dependency revisions and cross-repository links when analysing changes.

## Time, evidence cuts and accounting

Each record's event time describes the work or assessment. Its observation time describes when it became available to the observatory. An `as_of` cut restricts available evidence by observation time; calculations also apply the relevant event-time boundary. A historical report must not acquire later knowledge merely because it describes an earlier event.

The synthetic study example uses acceptance deadline `tau = 30 days` and retention follow-up `delta = 30 days`. The paper leaves these protocol parameters to the study design. For each enrolled task:

* Acceptance is measured from enrolment. Deployment is separate.
* Retention is assessed at acceptance plus `delta`.
* Cohort labour is counted until enrolment plus `tau + delta`, including unsuccessful work and attributable repair.

A task accepted on day 2 is assessed for retention on day 32, but its cohort labour remains open through day 60. A task accepted on day 40 receives zero output for the 30-day acceptance deadline while still contributing labour through day 60. An absent retention assessment is unknown; a task whose deadline has not arrived is pending. Abandonment is an observed nonacceptance outcome, whereas loss of observation is censoring.

Human allocations include specification, prompting, implementation, review, testing, integration and repair. One person's simultaneous allocations cannot exceed one. Two people pairing for one hour contribute two person-hours. Agent execution time and charges remain separate. Sampled effort requires the sampling frame, inclusion probability, response status and uncertainty; missing responses and work outside the frame do not become zero labour.

Retained productivity is a ratio of cohort totals, with numerator and denominator reported separately. Missing effort, unknown usage and unavailable follow-up remain visible in coverage and exclusions. A zero aggregate labour denominator leaves the ratio unavailable. CI success and surviving code do not establish complete retained functionality.

Complete direct observations can additionally support deterministic labour bounds
when a query clips an interval average. Individual interval bounds use the
person's attention capacity; task/cohort sums are conservative outer bounds
because shared allocations can couple intervals. They do not turn an unavailable
exact total into a point observation or a sampled estimate into an exact mass.

`inference` is a separate finite-frame statistical component. Its typed input
declares frozen predictions, the complete person-interval frame and the sampling
design. `estimate-labour` wires JSON input to that component without opening the
evidence store. Signed prediction-error correction and design-specific variance
remain separate from the nonnegative cohort accounting and its immutable input
records. General-design point estimation and SRS variance have distinct validation
contracts; the latter uses the finite-population correction and explicit
unavailable/census states. See [the estimator contract](design-based-estimation.md).

## Joint interval identification

`identification` implements the paper's exact finite-cell allocation model for
standalone interval constraints. Typed rational input, cell construction,
optimisation and witness verification occupy separate modules. `bound-labour`
only reads input and serialises the result. The solver uses exact rational
arithmetic and explicit resource limits, preserving the distinction between
inconsistency and failure to complete a calculation.

This component is separate from additive study effort records. Repeated or
overlapping observations constrain the same allocation in a linear programme;
they cannot be added as if they were independent labour contributions. Existing
snapshots and their conservative floating-point reports retain their established
semantics. See [the identification contract](labour-identification.md).

The estimator's v2 input adds declared per-person frame windows and unit
geometry. It checks a complete, non-overlapping interval partition and observed
labour against interval capacity before invoking the existing statistical
calculation. V1 remains supported as a caller-declared abstract finite frame.
Neither version can establish that the declared frame covers all actual work.

## Interpretation and extension

The initial use is a descriptive measurement and feasibility study. The arithmetic and synthetic examples verify bounded implementation behaviour; they do not establish a causal AI effect, general labour calibration or the adequacy of a quality rubric. A comparative study needs an explicit assignment/comparison design and interference assumptions.

The core uses ordinary Rust control flow rather than Aspect weaving. [ADR 0001](adr/0001-explicit-rust-pipeline.md) records the source-grounded decision. A hosted service becomes justified if multiple operators require concurrent collection or remote forms. Further indexed projections or analytical export formats become justified when actual query costs require them; neither changes retained evidence or the task-level measurement contract. Reports contain task text and evidence exports, including source payloads. Treat them as private research data rather than anonymised aggregate publications.


## Automatic forecasting

`forecast` is a separate mathematical component over retained automatic source
records. The lightweight GitHub profile collects repository metadata and all-state
PR inventory, retaining raw HTTP responses, pagination coverage and scan windows.
`ingestion` applies the requested profile's completeness policy after retaining
partial evidence; the existing full profile remains the default for collection.

`forecast::history` selects the latest scan per stable numeric repository identity,
binds its records to their collection receipt, and derives the common observation
cut from the earliest selected scan start. It reports current queue pressure with
freshness and unknown stages. This does not infer human labour or causal bottlenecks.
`calendar` forms complete UTC bins; `distribution` fits the explicit empirical
method; `backtest` reconstructs a declared retrospective evaluation. Portfolio
counts are summed before fitting. These calculations do not train on stage labels,
AI labels, human assessments or subjective task sizes.

`forecast::archive` binds each issued distribution and history to its original
verified snapshot and declared method. The CLI archives it without overwriting,
then later runs score unchanged originals against mature source cuts. Replay
outputs have a different schema and cannot pass as original issuance receipts.
Corrupt or unsupported archived inputs remain explicit errors in the score report;
they do not prevent new collection. Times are retained declarations, not trusted
independent proof of pre-outcome existence. JSON floating-point round trips are
correctly rounded so an unchanged saved PMF can be verified exactly.

The default expanding reference was retained after the recent-window challenger
failed its frozen public-reference promotion screen. The bounded public comparison
and the operational all-supported-history frame are distinct. Neither establishes
accuracy for a user's portfolio. See [the operational contract](automatic-forecasting.md)
and [the companion paper](../paper/forecasting/README.md).
