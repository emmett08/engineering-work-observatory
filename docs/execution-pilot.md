# Public repository execution pilot

This pilot replays two existing public-repository CI checks through Observatory's
explicit command wrapper. The checks run in Observatory's pilot workflow, so
their platform repository and run identity belong to Observatory. The subject
repository and pinned source revision are separate caller-declared matrix
metadata. Neither subject repository's workflow is modified.

Repository metadata confirmed both subjects public on 1 November 2026. This
bounded public pilot prepares wider adoption without changing the subjects'
workflows.

| Subject and pinned revision | Source workflow and selected check |
| --- | --- |
| `emmett08/engarg` at `dd848f3aefc6421e41bc0420ae5c40aae3553893` | [Rust checks, `checks` job](https://github.com/emmett08/engarg/blob/dd848f3aefc6421e41bc0420ae5c40aae3553893/.github/workflows/ci.yml): `python3 -m unittest discover -s tests -p 'test_*.py'` |
| `emmett08/eal-mcp` at `fe4a5cc4ef518b62ed0180646da1b7bccfd54249` | [Rust language and runtime, `context-experiment` job](https://github.com/emmett08/eal-mcp/blob/fe4a5cc4ef518b62ed0180646da1b7bccfd54249/.github/workflows/rust.yml): `cargo test --manifest-path experiments/eal-context-001/Cargo.toml --locked`, using Rust 1.90.0 |

These are the original workflow commands. They exercise a Python test suite and
an isolated Rust experiment test suite without running the projects' full
release pipelines or making model calls. The pilot does not attest to those
omitted checks.

## Live Actions trial

[Run 41329132331, attempt 1](https://github.com/emmett08/engineering-work-observatory/actions/runs/41329132331)
completed successfully at instrumentation commit
`9fe7717a9eb561b582967dd0ef18b6993316ff55`.

| Job | Retained bundle | Result and observed runtime |
| --- | --- | --- |
| [EngArg](https://github.com/emmett08/engineering-work-observatory/actions/runs/41329132331/job/124915331148) | [GitHub EngArg bundle](execution-pilot/github-attempt1-engarg.bundle.json), artifact `13092343025` | Exit 0; 12,814 ms |
| [EAL](https://github.com/emmett08/engineering-work-observatory/actions/runs/41329132331/job/124915331467) | [GitHub EAL bundle](execution-pilot/github-attempt1-eal-mcp.bundle.json), artifact `13092353087` | Exit 0; 16,522 ms |

The exact bundle SHA-256 hashes are:

```text
github-attempt1-engarg.bundle.json  3ee28f020e09ca8118232e58726704fa071095197984cb21e7ca00fab58f214b
github-attempt1-eal-mcp.bundle.json 81ee870908aa74a0ef393d56e74bfe226697d6c56c18da31726385981d57b528
```

Both artifact ZIPs downloaded through the GitHub connector passed the Rust
importer: six records on first import, zero new records on repeated imports,
and six verified records. The development workspace's direct artifact download
transport rejected the signed redirects; that acquisition retained independent
API metadata and explicit incomplete coverage.

[Run 41329655162](https://github.com/emmett08/engineering-work-observatory/actions/runs/41329655162)
then passed both subject jobs and the direct collection check at commit
`881c99343c2caaea9d7752bb6696b2c85af7ceac`. Its
[retained verification result](execution-pilot/github-collection-verification.json)
records four receipt records, two complete invocations, two independently
matched GitHub jobs and unchanged receipts after repeated collection. The
downloaded collector store passed a second local `verify` of all 29 retained
records and source objects; the snapshot assertions also replayed successfully.
The collector-state artifact is `13092399035`. This verifies the Rust artifact
download and reconciliation path on a GitHub runner, independently of the
development workspace's download restriction.

[The pilot workflow](../.github/workflows/execution-evidence-pilot.yml) runs on
pull requests changing the instrumentation and can also be dispatched manually.
Its two matrix jobs check out the pinned subjects into separate directories,
hash the source-revision file and source workflow, execute the selected check,
and publish separate immutable bundles. The EAL receipt also hashes its
dependency lockfile. Checkout credentials are not persisted.

After both subject jobs finish, a separate job downloads their artifacts using
the Rust collector, repeats collection, verifies the retained store, and checks
that two complete invocation pairs map to the two matrix jobs without duplicate
receipt identities. Its own job is still running during this check; that status
is preserved rather than presented as workflow-completion evidence. The
independent `workflow_run` observer template and scheduled collector handle
completed workflows.

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
