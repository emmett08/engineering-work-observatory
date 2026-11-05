# Issue-linked human-effort estimation

The effort pipeline collects selected ActivityWatch events, associates activity
with issues and PRs, freezes interval predictions, and prepares a random sample
for brief human checks. It sends the resulting predictions and observations to
the existing Rust `estimate-labour/v2` estimator. It does not require continuous
timers or daily timesheets.

Automatic activity is a predictor of effort. The reported estimate is corrected
using sampled observations; its accuracy still depends on those observations,
the declared people and time window, and correct task attribution. The included
pilot is synthetic software verification. No live human calibration or accuracy
claim has been established by running it.

## What gets associated with an issue

An issue is the work identity. The configuration associates it with a stable
`task_id`, repository, issue number, exact branch names, project labels and PR
numbers. One issue may have several PRs and contributors. A PR author is not
assumed to have performed all review, testing or coordination work.

For registered issues, refresh the PR associations from GitHub before collection:

```sh
python3 instrumentation/effort.py sync-links \
  --config private/config.json --repository owner/repository \
  --pr 11 --pr 12 \
  --output private/linked-config.json --evidence private/link-evidence.json
```

This read-only operation uses the authenticated `gh` CLI and retains GitHub's
closing-issue relationships, PR identity and head branch. Cross-repository issue
links and fork branches retain their repository identities. It enriches only
registered issues; it does not guess links from titles or silently create tasks.
If a queried PR no longer matches a retained binding, review the config rather
than silently keeping stale attribution. Use the resulting config consistently
for the whole batch. Links are task associations, not observed effort.

Each batch declares a participant roster and one contiguous observation window
per person. Fixed intervals partition those windows, including intervals with
no detected input, unavailable sources or ambiguous task attribution. Overlapping
devices do not multiply a person's time. Task attribution conflicts stay unattributed rather than being
split arbitrarily or assigned to the most recent commit.

The declared frame is distinct from telemetry coverage. `frame_complete: true`
attests that the people and windows cover the intended target. It does not say
that every activity source was available. Do not infer roster completeness from
the people who happened to produce GitHub events. Work outside those windows
is outside the estimate.

The baseline `activity-proxy-v1` uses recognised task context during periods
that ActivityWatch classifies as being present at the keyboard. This reproducible
prediction rule still requires observations of actual time allocation for
calibration. Automation categories are excluded. Build activity can contribute to
the proxy only under the configured attended-waiting convention; that convention
does not establish that someone actually attended a build. Reading or thinking
with no input can still be work; a missing source is not an observed zero. The
random checks can supply observations in those intervals.

## Install and select local sources

Use Python 3.10 or later and the repository's pinned Rust toolchain:

```sh
cargo build --locked --release
python3 instrumentation/effort.py --help
```

Install [ActivityWatch](https://docs.activitywatch.net/en/latest/introduction.html)
on participating machines. It stores activity locally and exposes a local API.
Select its away-from-keyboard (AFK) event bucket and compatible editor or
browser context buckets in the configuration. The collector does not install
watchers or infer which people have consented to participate.

The default active-window watcher supplies application/window titles, which this
collector does not retain or interpret as tasks. Default window and AFK watchers
alone therefore do not provide issue attribution. Use an editor watcher that
supplies `project` and preferably `branch` or `repository`, or a browser watcher
whose `url` identifies a configured GitHub issue or PR. Inspect the selected
watcher's local event fields before adopting it. A project label identifies an
issue only when that association is unambiguous; a shared repository normally
needs the branch or issue/PR context as well.

Start from [the example configuration](../examples/effort/config.json), replacing
its fictional people, projects, repositories, issues, PRs and window. Set
`synthetic: false` only for actual participant evidence. The config
contains no credentials. Keep participant identity mappings outside the report.
The collector retains only configured project/branch/repository references,
canonical configured GitHub issue/PR URLs, recognised categories, timestamps and
AFK status. Raw titles, paths, URL queries, code and hostname values are omitted.

For a local ActivityWatch export:

```sh
python3 instrumentation/effort.py collect \
  --config private/config.json \
  --export private/activitywatch-export.json \
  --output private/activity.json
```

For a running local ActivityWatch server:

```sh
python3 instrumentation/effort.py collect \
  --config private/config.json \
  --server http://127.0.0.1:5600 \
  --output private/activity.json
```

Live collection accepts literal loopback HTTP origins and refuses redirects.
It requests only configured buckets and the declared time windows. It marks
missing, failed or incomplete responses explicitly. Retrieved events establish
their own covered spans; a successful request is not an attestation that the
watcher ran continuously. For a multi-device batch, use the same config on each
machine, collect separate sanitised activity files, then merge before freezing:

```sh
python3 instrumentation/effort.py merge-activity \
  --config private/config.json \
  --activity private/activity-laptop.json \
  --activity private/activity-desktop.json \
  --output private/activity.json
```

The configuration explicitly maps each device to a person. Conflicting source
deliveries are rejected; repeated matching events do not multiply elapsed time.
Run a local scheduled task after each agreed observation window to collect a
new batch. A hosted GitHub runner cannot observe an engineer's desktop.

## Prepare a local batch automatically

For one local ActivityWatch server, `prepare` performs collection, interval
construction, prediction freeze, sampling, participant-form generation and proxy
reporting in one command:

```sh
python3 instrumentation/effort.py prepare \
  --config private/config-template.json \
  --day 2026-11-01 --size 4 \
  --server http://127.0.0.1:5600 \
  --directory private/batches/2026-11-01
```

The template supplies explicit UTC person windows. `--day` shifts those windows
to the selected UTC date; it does not infer work hours from activity or extend
the declared scope. Use `--export` instead of `--server` for a completed combined
export. Multi-device collection from separate servers uses the earlier
`collect`/`merge-activity` steps and the `freeze`/`sample` steps below.

The command writes `config.json`, `activity.json`, `frame.json`, `sample.json`,
an offline `review-index.html`, participant-specific forms and a proxy report.
Running it again for the same retained batch reuses its frozen evidence and
saved draw. It does not silently collect a new version or select another sample.
Use a distinct batch directory for a new period. The initial proxy remains
labelled unavailable as a human-effort estimate until observations are supplied.

For example, on a host whose cron clock is UTC, the following runs after a
declared observation period ending before 18:15 UTC:

```cron
15 18 * * 1-5 /usr/bin/python3 /srv/observatory/instrumentation/effort.py prepare --config /srv/private-effort/config-template.json --day "$(date -u +\%F)" --size 4 --server http://127.0.0.1:5600 --directory "/srv/private-effort/batches/$(date -u +\%F)" >> /srv/private-effort/prepare.log 2>&1
```

Replace the paths, window, schedule and sample size for the prespecified study.
Keep the private directory outside a public checkout. The operating system must
run ActivityWatch and the scheduled command under an account that can access its
local buckets. Review the selected intervals promptly, then run `estimate` or
the private report workflow after responses arrive. No timer entry is needed,
but a scheduler cannot supply the missing human checks.

## Freeze, sample and review

Freeze the completed batch before drawing the evaluation sample:

```sh
python3 instrumentation/effort.py freeze \
  --config private/config.json --activity private/activity.json \
  --output private/frame.json
python3 instrumentation/effort.py sample \
  --frame private/frame.json --size 4 --output private/sample.json
python3 instrumentation/effort.py review \
  --frame private/frame.json --sample private/sample.json \
  --person participant-a --output private/review-participant-a.html
```

The sample size of four above is only command syntax, not a recommended size for
an empirical study. Choose sampling burden and target precision before drawing
the sample. All frame intervals have a chance of selection, including periods
without editor activity. The saved random seed and stable selection algorithm
make the selection inspectable. User-supplied seeds are restricted to explicitly
synthetic replay. Retain the first sample; changing filenames must not become a
way to draw repeatedly until a desirable sample appears.

Generate a separate form for each participant with `--person`. Use `--all-people`
only for an explicitly planned observer form. Open the participant's HTML in a
browser. It runs offline and shows only that person's selected intervals,
task options and role options without displaying the predicted hours. Record
minutes allocated to each task/role and non-work minutes. The response must
account for the whole interval. An uncertain or unanswered interval remains
nonresponse. Submit an observed zero only when the interval was checked and all
of it was non-work under the frozen protocol. Follow the configured convention
for attended waiting. Download completed responses into a private `responses/`
directory in the batch; keep each participant's file separately. A JSON response
template is also available:

```sh
python3 instrumentation/effort.py template \
  --frame private/frame.json --sample private/sample.json \
  --person participant-a --output private/responses-template-a.json
```

Use short completed batches and prompt review to limit recall error. Independently
check a subset of labels during calibration. The UI makes observations easier
to record; it does not make retrospective recollections exact measurements.
The Rust estimator rejects nonresponse; the wrapper preserves an
`estimate_unavailable` report with the proxy, response counts and reasons when
observations are incomplete. Do not drop unanswered intervals,
replace them with predictions, or label them zero to obtain a complete result.

## Estimate and report

```sh
python3 instrumentation/effort.py estimate \
  --frame private/frame.json --sample private/sample.json \
  --responses private/responses/participant-a.json \
  --responses private/responses/participant-b.json \
  --observatory target/release/observatory \
  --target total \
  --output private/report.json --markdown private/report.md
```

Use a configured `task_id` as `--target` to estimate one issue, or `unattributed`
for work whose task identity is unknown. The all-task total includes unattributed
engineering work. Reports retain issue/PR links, frame identity, model version,
telemetry coverage, prediction totals and the Rust estimator result.

Repeat `--responses` for every completed participant file. The merger accepts
matching duplicate responses but rejects conflicting revisions and references
to unselected units. Unanswered selected units still leave the complete
estimate unavailable. Estimates also retain the exact input passed to the Rust
command as a neighbouring `.estimator-input.json` file.

For a batch without sampled observations, generate a clearly labelled proxy:

```sh
python3 instrumentation/effort.py proxy \
  --frame private/frame.json \
  --output private/proxy.json --markdown private/proxy.md
```

The estimator corrects the predicted total using differences between observed
and predicted person-hours in the selected intervals. Each difference is divided
by that interval's selection probability:

```text
estimated total = prediction total + sampling-weighted observed prediction errors
```

See [the estimator contract](design-based-estimation.md) for the complete
assumptions. The interval geometry is checked, and genuine simple random
sampling can support a sampling standard error. This is not a bound on all
measurement error or an automatic confidence interval: inaccurate labels,
omitted people, wrong issue links and missing work outside the frame remain
separate concerns. A small sample may give an unstable issue estimate even when
the team total is useful. Realised estimates can be negative or exceed physical
capacity; the estimator preserves them rather than silently clipping and
retaining an unchanged unbiasedness claim.

These files remain separate from direct `effort` study records and the frozen
cohort report. A prediction does not become an observed person-hour allocation.
Passing tests and completing a PR do not supply requirement acceptance or
evidence that the accepted requirement is still satisfied at reassessment.
Those observations are still needed for productivity claims.

## Automation and validation

The [report workflow template](../examples/effort/report-workflow.yml) runs on a
trusted Linux self-hosted runner with access to a designated private batch
directory. It reads `frame.json`, `sample.json` and completed JSON files under
`responses/`, computes an aggregate report and retains it under the private
batch's `reports/RUN_ID-RUN_ATTEMPT/` directory. Data stays private by default.
Copy it into an adopting repository
only after configuring its runner, evidence directory and trusted revision.
The runner needs Git, Python, the pinned Rust toolchain and, for optional
publication, the authenticated GitHub CLI supplied with the workflow token.
Its scheduled runs do not publish reports. A manually dispatched, explicitly
enabled publication can post the aggregate Markdown to selected issue/PR
numbers. Choose the configured issue task ID as `report_target` to publish that
issue estimate; `total` explicitly means the whole frame, regardless of the
comment location. Scheduled calculations use `EFFORT_REPORT_TARGET` if configured,
otherwise `total`. No raw activity or responses are uploaded. The template is
not enabled for real human data in this repository. An issue-specific report can
be published only to issue/PR identities retained in that target's links;
unlinked destinations are rejected.

Before interpreting estimates, evaluate the complete process on later held-out
batches. Check systematic over/underestimation, absolute error, coverage and
missingness, separately for relevant work types and AI-use conditions. Compare
the simple baseline with any proposed replacement before changing models.
Keep each batch's predictions fixed during evaluation; train revisions on
earlier calibration data. A random split of correlated events from the same PR
does not provide an independent evaluation. Prespecify a useful error tolerance
and increase the sample or narrow the claim if it is not met.

Start with team-period totals and sufficiently observed issues. Do not interpret
an issue's sparse sample or consistently repeated proxy as proof of accuracy.
Known statistical assumptions and reproducible code are prerequisites for an
empirical check, not a substitute for one.

## Reproduce the synthetic pilot

```sh
cargo build --locked
python3 examples/effort/replay.py \
  --observatory target/debug/observatory \
  --output-dir reports/effort-synthetic
```

The fixture has two fictional participants, two issues, overlapping devices,
periods without detected input, an automation category, unattributed context and a missing
bucket. A separate manufactured truth file supplies selected responses only
after predictions and sampling have been saved. The replay checks proxy and
sample-adjusted reporting, missingness and capacity handling. Its files and
outputs must never be reported as observations of the public repositories or
as an estimate of PR #9's human effort.
