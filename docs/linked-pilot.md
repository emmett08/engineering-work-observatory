# Linked execution and human-effort pilot

The linked pilot instruments checks in the two subject repositories, collects
their execution receipts, and associates them with real issues and PRs. Human
effort stays unavailable until a private participant batch and sampled responses
exist. A passing CI run demonstrates execution evidence; it does not establish
how long an engineer worked.

| Task | Issue and implementing PR | Selected check |
| --- | --- | --- |
| `engarg-authoring-effort-pilot` | [EngArg #49](https://github.com/emmett08/engarg/issues/49), [PR #50](https://github.com/emmett08/engarg/pull/50) | `python3 -m unittest discover -s tests -p 'test_*.py'` in each existing checks matrix job |
| `eal-context-effort-pilot` | [EAL #36](https://github.com/emmett08/eal-mcp/issues/36), [PR #37](https://github.com/emmett08/eal-mcp/pull/37) | `cargo test --manifest-path experiments/eal-context-001/Cargo.toml --locked` in a dedicated PR-triggered pilot |

Both use protocol `observatory-linked-pilot-v1`. The IDs are identical in CI
setup, the [execution manifest](../examples/linked-pilot/manifest.json), and the
[effort configuration template](../examples/linked-pilot/config-template.json).
The task-to-issue relationship is a declaration; the receipt's run identity and
GitHub's issue/PR relationship provide separate evidence. Actor, rerun actor and
trigger event remain separate from controller/session provenance. An absent
controller or assistant session remains unknown.

The original [execution pilot](execution-pilot.md) replayed checks inside
Observatory. This pilot collects the subject repositories' own run identities.
EAL's full language/runtime workflow remains manual; only the bounded context
pilot gains a PR trigger, as requested for this demo.

## Run the execution-only demo now

Build the reviewed Observatory revision with its pinned Rust toolchain, Python
3.10 or later and authenticated `gh`. The selected run IDs in the manifest are
explicit scope, not a claim to have observed every repository execution. Add
subsequent selected run IDs to a retained new manifest when extending the pilot.

```sh
cargo build --locked --release
python3 instrumentation/pilot.py collect \
  --manifest examples/linked-pilot/manifest.json \
  --state /srv/observatory-private/executions \
  --observatory target/release/observatory
python3 instrumentation/pilot.py report \
  --manifest examples/linked-pilot/manifest.json \
  --state /srv/observatory-private/executions \
  --output-dir /srv/observatory-private/reports/execution-demo \
  --observatory target/release/observatory
```

Collection uses the Rust artifact importer and independent GitHub run/attempt/job
metadata, then verifies the retained store. Repeated deliveries keep receipt
identity. Parallel jobs publish separate immutable bundles. A failed or cancelled
job can have incomplete receipt coverage; successful publication is not assumed.
The report joins receipts by repository, protocol and task, retains selected run
references, and states when no human estimate is available. Its output is a
task-level JSON/Markdown report plus an index. It does not calculate labour from
tool runtime, commit counts or PR duration.

The public [linked-pilot validation workflow](../.github/workflows/linked-pilot-demo.yml)
exercises this path with public execution evidence only. Private participant
activity and responses are not inputs to that workflow.

Add `--publish` to `report` only when an aggregate comment should be posted to the
manifest's linked issues and PRs. Publication uses the authenticated GitHub
identity and upserts its marked report. An execution-only comment explicitly
says human effort is unavailable. Synthetic effort cannot be published as real
human effort. No raw activity, person IDs or response files belong in a public
comment.

## Prepare the private runner

Runner setup is prepared for adoption. No participant workstation, runner
registration or live human observations were configured by this change.

1. Create or select a **private control repository** and a dedicated trusted
   Linux self-hosted runner with the `effort-collector` label. Install Git,
   Python 3.10+, `gh`, Rustup and the pinned Rust toolchain. Restrict who can
   modify or dispatch its default-branch workflow. Do not attach this runner to
   pull-request workflows in either public pilot repository.
2. Create durable private storage outside the checkout, for example
   `/srv/observatory-private`, owned by the runner account with directory mode
   `0700`. Set a restrictive `umask 077` for local collection. Back up this
   directory using the study's retention policy; Actions artifacts are not its
   durable store.
3. Copy the reviewed manifest to `manifest.json` in that directory. Copy the
   effort configuration template as `config-template.json` only when configuring
   an actual participant batch. Replace all placeholder participant, device,
   bucket and time values. Preserve the protocol/task IDs and real issue/PR
   links. Declare all people and windows before sampling. Set `frame_complete`
   true only when that scope is the intended target; telemetry alone cannot
   establish this declaration.
4. Copy the [private workflow template](../examples/linked-pilot/private-runner-workflow.yml)
   to `.github/workflows/observatory-private.yml` in the private control
   repository. Keep its reviewed full Observatory commit pin. Set repository
   variable `EFFORT_PRIVATE_ROOT` to the private directory. Set
   `EFFORT_SAMPLE_SIZE` to the sample size chosen before drawing. An optional
   `EFFORT_ACTIVITY_EXPORT` points to a privately supplied completed
   ActivityWatch export; otherwise preparation uses local loopback ActivityWatch.
5. Set `OBSERVATORY_READ_TOKEN` to a GitHub App token or fine-grained token scoped
   to the two pilot repositories with Actions read and required metadata read.
   If publishing is wanted, separately configure `OBSERVATORY_PUBLISH_TOKEN`
   as a fine-grained personal/user access token with issue and pull-request
   comment access for those same repositories. The current publisher uses
   `GET /user` to identify its own comments, so installation tokens and the
   Actions `GITHUB_TOKEN` are not supported for publication.
   The control repository's default `GITHUB_TOKEN` does not grant cross-repository
   access. Keep credentials in the private repository's secret store; do not put
   them in either JSON configuration or participant exports.

The workflow checks that its control repository is private and that dispatch
uses the default branch. It checks out pinned Observatory code, not a pilot PR's
code, and treats downloaded artifacts as evidence rather than executables. Its
command logs, database, forms and report files remain in private storage. No
participant artifacts are uploaded. Schedule runs collect and report without
posting comments; publication requires a dispatch with `publish_comment` true.

The retained layout is:

```text
/srv/observatory-private/
  manifest.json
  config-template.json
  executions/                 # Verified receipt store and acquisition evidence
  batches/YYYY-MM-DD/
    config.json
    activity.json
    frame.json
    sample.json
    review-index.html
    review-<hash>.html        # Send only the participant's own form
    responses/               # Returned, hash-bound observation JSON
  logs/<run-id>-<attempt>/
  reports/<run-id>-<attempt>/
```

## Add actual activity and occasional sampled checks

Install [ActivityWatch](https://docs.activitywatch.net/en/latest/introduction.html)
on each participating workstation. Select an AFK bucket and a compatible editor
or browser context watcher. The standard active-window title watcher does not
give the collector issue attribution. Use exact project/branch/repository
references or configured GitHub issue/PR URLs; check the selected watcher's local
fields before collecting. Do not infer the participant from a GitHub account.

The two pilot branches share the same branch name, so the context should include
the repository. Configure exact project aliases only when their task mapping is
unambiguous. Unresolved or conflicting activity remains unattributed. No raw
window titles, file paths, prompt text, code or arbitrary environment dumps are
needed.

Refresh registered issue/PR relationships before creating a new private batch:

```sh
python3 instrumentation/effort.py sync-links \
  --config /srv/observatory-private/config-template.json \
  --repository emmett08/engarg --pr 50 \
  --output /srv/observatory-private/linked-engarg.json \
  --evidence /srv/observatory-private/links-engarg.json
python3 instrumentation/effort.py sync-links \
  --config /srv/observatory-private/linked-engarg.json \
  --repository emmett08/eal-mcp --pr 37 \
  --output /srv/observatory-private/linked-config.json \
  --evidence /srv/observatory-private/links-eal.json
```

Review the linked config and use it consistently for the new batch. The workflow
reads `config-template.json`; install the reviewed linked config under that name
before dispatching preparation. Existing frozen batches keep their original
config and sample. Use dated link evidence paths for subsequent refreshes rather
than overwriting retained evidence.

For a runner on the participant workstation, dispatch operation `prepare` after
the declared window ends. For a separate collector host, collect on the
workstation and transfer the allowlisted export or sanitised activity privately.
A hosted or separate self-hosted GitHub runner cannot reach a desktop's loopback
server. The [multi-device procedure](effort-estimation.md#install-and-select-local-sources)
merges separately collected devices under one declared configuration before
freezing and sampling.

Preparation creates offline forms; distribute only each participant's own form.
The participant checks the selected intervals, allocating minutes to task/role
and non-work, then saves the downloaded JSON under the batch's `responses/`.
Uncertain recall stays nonresponse. There is no continuous timer or full-day
timesheet, but code cannot supply the missing observations. Predictions remain
hidden in the review form. The first probability sample is retained across
retries; repeated preparation does not choose a more convenient sample.

For routine workstation preparation, use the operating system scheduler under
the account running ActivityWatch. This example assumes a UTC cron clock and
declared windows ending before 18:15. Choose the window, sample size, schedule
and task scope before collection; `4` is illustrative, not a recommended sample
size for a precision target.

```cron
15 18 * * 1-5 umask 077; /usr/bin/python3 /srv/observatory/instrumentation/pilot.py prepare --config /srv/observatory-private/config-template.json --day "$(date -u +\%F)" --size 4 --server http://127.0.0.1:5600 --directory "/srv/observatory-private/batches/$(date -u +\%F)" >> /srv/observatory-private/prepare.log 2>&1
```

After responses arrive, dispatch `report` for that UTC day or let a scheduled
collection/report run process the current day's batch. For an older day, select
it explicitly. The workflow's two UTC evening runs are examples; adjust them to
the declared windows and expected response time. Add `publish_comment` only for
the desired aggregate publication. A missing batch or missing responses remains
unavailable, and the report can still show the execution evidence.

An optional **effort observation** issue template in each pilot repository can
record a deliberately public, occasional aggregate self-report. It is declared
evidence, not an automatic measurement or a replacement for the random sample.
Do not paste private sampled response JSON into an issue. An unbound reported
hour count is not inserted into the estimator; only responses bound to the
frozen frame and saved sample can perform that correction.

## Interpret the result

Execution reports show what the selected wrappers observed and what GitHub
independently retained. The effort pipeline reports activity predictions
separately from sample-adjusted person-hours. Its sampling standard error covers
the specified sampling design; it does not quantify recall error, missing
contributors, activity-watcher misclassification or work outside the declared
window. Consistency of collection does not by itself establish accuracy.

Calibrate with short windows, prompt checks and independent review of a subset
of observations before using these estimates for decisions. Requirement
acceptance and retained quality still require their own observations. Neither
an execution receipt nor an effort estimate establishes productivity or an
individual engineer's performance.
