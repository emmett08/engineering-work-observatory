# Public repository execution pilot

This pilot replays two existing public-repository CI checks through Observatory's
explicit command wrapper. The checks run in Observatory's pilot workflow, so
their platform repository and run identity belong to Observatory. The subject
repository and pinned source revision are separate caller-declared matrix
metadata. Neither subject repository's workflow is modified.

The user selected any two public repositories in `emmett08` in place of the
initial NBCUDTC pilot. Repository metadata confirmed both subjects public on
1 November 2026.

| Subject and pinned revision | Source workflow and selected check |
| --- | --- |
| `emmett08/engarg` at `dd848f3aefc6421e41bc0420ae5c40aae3553893` | [Rust checks, `checks` job](https://github.com/emmett08/engarg/blob/dd848f3aefc6421e41bc0420ae5c40aae3553893/.github/workflows/ci.yml): `python3 -m unittest discover -s tests -p 'test_*.py'` |
| `emmett08/eal-mcp` at `fe4a5cc4ef518b62ed0180646da1b7bccfd54249` | [Rust language and runtime, `context-experiment` job](https://github.com/emmett08/eal-mcp/blob/fe4a5cc4ef518b62ed0180646da1b7bccfd54249/.github/workflows/rust.yml): `cargo test --manifest-path experiments/eal-context-001/Cargo.toml --locked`, using Rust 1.90.0 |

These are the original workflow commands. They exercise a Python test suite and
an isolated Rust experiment test suite without running the projects' full
release pipelines or making model calls. The pilot does not attest to those
omitted checks.

## Live Actions trial

[The pilot workflow](../.github/workflows/execution-evidence-pilot.yml) runs on
pull requests changing the instrumentation and can also be dispatched manually.
Its two matrix jobs check out the pinned subjects into separate directories,
hash the source-revision file and source workflow, execute the selected check,
and publish separate immutable bundles. The EAL receipt also hashes its
dependency lockfile. Checkout credentials are not persisted.

The workflow leaves controller and assistant session provenance unknown. A
GitHub account triggering this workflow does not establish its actual human or
assistant controller. The durable collector independently retrieves the
Observatory run's original actor, rerun actor, attempts, jobs and bundles:

```sh
cargo run --locked -- --state /path/to/durable/observatory collect-github-executions \
  --repository emmett08/engineering-work-observatory --run-id RUN_ID
cargo run --locked -- --state /path/to/durable/observatory verify
```

Repeat collection to check delivery deduplication. Use a rerun to check that
attempt identity changes while the original and rerun actors remain separate.
The unit and HTTP integration tests cover retries, matrix identity,
cancellation, skipped jobs, missing uploads and malformed artifacts; those
tests are not claims that each failure mode has occurred in the live pilot.

## Local trial evidence

The local trial completed on 1 November 2026 using isolated clones of the pinned
revisions. Its GitHub platform identifiers remain `unknown`; it is not a GitHub
run. Both source checkouts remained unchanged.

| Retained bundle | Result | Observed process runtime |
| --- | --- | --- |
| [EngArg local bundle](execution-pilot/engarg-local.bundle.json) | 73 Python tests passed; exit 0 | 41,644 ms |
| [EAL local bundle](execution-pilot/eal-mcp-local.bundle.json) | 24 Rust tests passed; exit 0 | 44,107 ms |

Each bundle contains an immutable start and completion for one invocation. Their
SHA-256 hashes are:

```text
engarg-local.bundle.json  9fb31fe41b00028f4a6f349d99c9a0dd62183c9021881a3f69157b18304c436a
eal-mcp-local.bundle.json 942f2a502b5bf7ae51f9def7adeace87151dc989a1f6063479d330e49a3d41f7
```

Both bundles passed the Rust importer. First imports added three records each
(one bundle record and two receipt records); importing both again added zero
records. `verify` reported six retained records, including recursively verified
source bytes. Reproduce those checks in an empty state directory:

```sh
cargo run --locked -- --state /tmp/observatory-local-pilot import-executions \
  docs/execution-pilot/engarg-local.bundle.json
cargo run --locked -- --state /tmp/observatory-local-pilot import-executions \
  docs/execution-pilot/eal-mcp-local.bundle.json
# Repeat both imports: each reports zero new records.
cargo run --locked -- --state /tmp/observatory-local-pilot verify
```

Runtime describes the wrapped process, including startup and any dependency
build. These receipts do not measure human attention, requirement acceptance,
retained quality, invoiced charges or AI usage. The Codex adapter has separate
fixture-based protocol tests; this two-repository pilot makes no live assistant
execution claim.
