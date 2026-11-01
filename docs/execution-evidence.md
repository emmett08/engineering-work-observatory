# Tool-execution evidence

The reusable [action](../action.yml) records tools that a workflow explicitly
wraps. It produces one job-local, versioned bundle; the Observatory collector
downloads bundles into a durable store and reconciles them with independently
retrieved GitHub runs, attempts and jobs. Adding setup alone does not observe
other steps, tool internals, other jobs or work outside Actions.

## Identity and provenance

| Evidence | Meaning and source |
| --- | --- |
| `platform.actor`, `platform.triggering_actor`, `platform.event_name` | Original GitHub actor, rerun actor and trigger, copied separately from the runner environment. Neither actor identifies the actual human or assistant controller. |
| Repository ID, workflow ref/SHA, run ID/attempt, job key/name/ID, matrix | Execution context. `workflow_sha` identifies the workflow revision; `github.job` is a job key, not the numeric API job ID. The caller supplies the expanded job name, optional numeric ID and allowlisted matrix. |
| `declared.controller`, provenance and research links | Caller declarations. Controller is `human`, `assistant`, `automation` or `unknown`; absent protocol, task, session, logical-call and parent-invocation links remain null. |
| Tool/version, purpose, configured model and configuration | Caller-supplied labels. Purpose separates development from product execution. Configuration metadata does not configure the executed tool; the Codex adapter explicitly applies `--configured-model`. |
| Start/completion, exit status, elapsed runtime | Observations made by the wrapper around its child process. Runtime is elapsed execution time, never human labour. |
| Observed model/session, tokens and charges | Tool-reported fields where an adapter recognises them; otherwise null with `missing_fields`. Reported usage is not invoiced cost. The initial adapters do not estimate charges. |
| API run/attempt/job and reconciliation records | Independently retrieved GitHub observations. They corroborate supported platform identifiers; they do not authenticate caller declarations or tool output. |

An assistant using a human's GitHub credentials can declare `assistant`; a
human starting a workflow containing Codex can declare `human`. Keep `unknown`
when evidence does not establish the controller. A Codex-reported thread ID is
stored separately from the declared research session ID. Do not manufacture a
session ID from a GitHub run ID.

Hashes establish byte integrity, not the truth of an uploaded receipt. Even
runner-derived fields inside an uploaded bundle are untrusted until checked
against independent metadata. The v1 `platform.source` label describes how the
recorder obtained its default context; `job_context_source: caller` identifies
the explicitly supplied job name/ID/matrix.

## Instrument a job

The action requires Python 3 and Bash. Use setup once in each job, explicitly
wrap selected commands, and call publish with `if: always()`. Pin the action to
a reviewed commit when adopting it; the examples use `@main` as a post-merge
template reference. They do not install instrumentation in another repository.

```yaml
- uses: emmett08/engineering-work-observatory@main # Replace with reviewed full SHA.
  with:
    controller: unknown
    job-name: Build (ubuntu-latest)
    matrix: '{"os":"ubuntu-latest"}'
- name: Test with a receipt
  run: |
    python3 "$OBSERVATORY_RECORDER" run \
      --tool cargo --tool-version "$(cargo --version)" \
      --purpose development --check-name cargo-test \
      -- cargo test --locked
- uses: emmett08/engineering-work-observatory@main # Use the same reviewed SHA.
  if: always()
  with:
    mode: publish
```

Supply the exact expanded GitHub job display name to distinguish matrix siblings.
A numeric job ID can be supplied if independently available. Ambiguous mappings
remain unresolved rather than being guessed. A job's recording directory is
exported as `OBSERVATORY_EVIDENCE_DIR`; `directory` can be passed explicitly when
needed. Parallel jobs publish separate bundles and never share a SQLite store.

The [ordinary-tool example](../examples/execution-evidence/ordinary.yml) wraps
Cargo in a matrix job. The [assistant example](../examples/execution-evidence/assistant.yml)
shows the Codex adapter in an already provisioned assistant job. The adapter
invokes `codex exec --json`, reads recognised events and discards the transcript.
The supported Codex events report a thread ID and token usage but no model
identity, so this adapter always leaves `observed_model` null, including when a
model was configured. Its parser follows the upstream
[Codex event definitions](https://github.com/openai/codex/blob/4b4a5cf25aa5b064c866218b405c848745fd280f/codex-rs/exec/src/exec_events.rs):
`thread.started.thread_id` and `turn.completed.usage` fields `input_tokens`,
`cached_input_tokens` and `output_tokens`. Other event fields are ignored.
Credentials belong in the tool's existing secret mechanism, not action inputs.

For each invocation, optional links and artifact hashes can be supplied:

```sh
python3 "$OBSERVATORY_RECORDER" run \
  --tool cargo --purpose development \
  --protocol-id protocol-1 --task-id requirement-17 \
  --session-id session-42 --logical-call-id test-requirement-17 \
  --input requirement=study/requirement-17.md \
  --output check-report=results/check.json \
  -- cargo test --locked
```

`--input` and `--output` hash designated regular files under logical labels;
file contents and paths are not included in the bundle. The example output
must be produced by the wrapped command to be observed. Missing outputs are
recorded as missing. `--check-name` records the named command's exit-based result,
not a parsed count of individual tests. Configuration accepts only
`reasoning_effort`, `temperature`, `seed`, `max_output_tokens` and
`configuration_sha256`. Do not use free-text labels or references to carry
credentials, prompts or personal data.

## Immutable receipts and delivery

The [receipt schema](../schemas/tool-receipt-v1.schema.json) is
`observatory.tool-receipt/1`; the [bundle schema](../schemas/execution-bundle-v1.schema.json)
is `observatory.execution-bundle/1`. A bundle contains frozen job context and
receipt JSON strings with SHA-256 hashes of their exact UTF-8 bytes. Each actual
tool execution receives a fresh invocation UUID. Its start and completion are
separate immutable files with the same identity. A real retry gets a new UUID;
a logical-call or parent link can connect the attempts without merging them.

Publishing reads only receipt files and emits `bundle.json`. The artifact name
starts with `observatory-executions-` and includes the bundle hash. Re-importing
identical receipts is idempotent, including when different uploads contain the
same receipts. Conflicting content under an existing receipt identity is rejected.
An empty bundle still preserves its job context; it states that no wrapped
receipts were published, not that no tools ran.

Start-only and completion-only deliveries remain incomplete observations. The
wrapper forwards interrupt/termination signals and tries to record completion,
but forceful cancellation or runner loss can prevent both completion and upload.
`if: always()` is best effort, not durable delivery. This composite action has no
post-job hook; a post hook would have the same runner-loss limitation.

## Durable collection and independent reconciliation

Run acquisition from a trusted, persistent collector host with read access to
the selected repository and Actions artifacts:

```sh
observatory --state /srv/observatory/study collect-github-executions \
  --repository owner/repository
observatory --state /srv/observatory/study collect-github-executions \
  --repository owner/repository --run-id 123456789
observatory --state /srv/observatory/study verify
```

Supply `GH_TOKEN` through the host's secret mechanism. `--token-env` selects a
different variable. Schedule collection before artifact expiry and retry failed
acquisitions using the same state directory. Retained immutable bytes survive
later artifact expiry. Use one serialized collector for a state directory;
parallel acquisition workers should hand off bundles for serialized import.

To import an already downloaded bundle or GitHub artifact ZIP:

```sh
observatory --state /srv/observatory/study import-executions bundle.json
observatory --state /srv/observatory/study import-executions artifact.zip
```

The importer accepts a ZIP containing exactly `bundle.json`; it rejects extra
members, unsafe paths, unsupported schemas, incorrect hashes and inconsistent
lifecycle identities. Bounds are 16 MiB per bundle/archive expansion, 10,000
receipts and 256 KiB per receipt. Raw source bytes remain linked to imported
records and are checked by `verify`. No observed project code or assistant is
executed during collection/import.

The opt-in collector inventories runs, every attempt within its documented cap,
and jobs independently of receipts. Failed, cancelled and skipped jobs are
retained. It validates supported repository/run/attempt provenance and records
unresolved job mappings. Missing bundles, missing lifecycle phases and failed
artifact downloads never become zero tool use. API and validation failures
retain partial evidence and coverage before the command returns nonzero.
Absence of a bundle is an observation gap, not automatically an acquisition error.

The [completion observer template](../examples/execution-evidence/observer.yml)
uses `workflow_run: completed` for any preceding conclusion. It builds trusted
Observatory code, collects the completed run and archives an isolated observer
store. This short-lived archive is an additional delivery route; central
scheduled collection remains responsible for durable retention. Copy the
observer to the observed repository's default branch and select actual workflow
names. Never build the triggering PR's checkout in the observer's credential
context. Do not share its SQLite file with instrumented matrix jobs.

## Public-repository pilot and interpretation limits

Observatory's own Rust CI wraps its Cargo test invocation and publishes separate
matrix-job bundles. That exercises adoption in this repository.

The [pilot workflow](../.github/workflows/execution-evidence-pilot.yml) replays
selected commands from two public repositories, as authorised for this initial
pilot. Each case runs at a fixed source revision:

| Public source | Selected existing workflow command |
| --- | --- |
| `emmett08/engarg` at `dd848f3aefc6421e41bc0420ae5c40aae3553893` | `.github/workflows/ci.yml`: `python3 -m unittest discover -s tests -p 'test_*.py'`. |
| `emmett08/eal-mcp` at `fe4a5cc4ef518b62ed0180646da1b7bccfd54249` | `.github/workflows/rust.yml`: `cargo test --manifest-path experiments/eal-context-001/Cargo.toml --locked`. |

See the [pilot evidence log](execution-pilot.md) for execution status, retained
hashes and limitations. These are executions hosted by Observatory against
pinned source checkouts; their platform identity remains Observatory's repository
and run. They do not claim new workflow installations in either target repository
or an NBCUDTC deployment. Neither command requires a paid assistant call.

Before wider adoption, inspect original/rerun actors, matrix-job mapping,
actual tool retries, duplicate delivery, cancellation and missing uploads against
independent completed-job metadata. Check that no credentials, prompts, transcript
or unrestricted context/environment dump entered a bundle. Exercise a live
assistant requirement task separately, with declared controller/session links,
configured versus reported model and missing versus reported usage. Local Codex
event fixtures establish adapter behaviour, not live assistant integration.

Execution evidence can link to existing protocol/task/session observations.
Human attention, requirement acceptance and retained quality still require their
own records and evidence. These receipts remain `source_record` evidence and do
not automatically create effort, acceptance, retained-output or duplicate
EngArg `ai_usage` records.

GitHub references: [actor and triggering-actor contexts](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#github-context),
[action versus workflow reuse](https://docs.github.com/en/actions/concepts/workflows-and-actions/reusing-workflow-configurations),
[action post lifecycle](https://docs.github.com/en/actions/reference/workflows-and-actions/metadata-syntax#runspost),
and [workflow completion events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run).
