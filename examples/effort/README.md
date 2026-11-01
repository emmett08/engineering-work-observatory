# Synthetic issue-linked effort pilot

This example verifies collection, prediction, sampling, response validation and
reporting using manufactured events and outcomes. It observes no engineers and
makes no claim about the effort behind PR #9 or the public repository pilot.

From the repository root:

```sh
cargo build --locked
python3 examples/effort/replay.py \
  --observatory target/debug/observatory \
  --output-dir reports/effort-synthetic
```

Use a new empty output directory for each replay. The files contain:

- `config.json`: two fictional people, one hour each, two issue identities and
  explicit project/branch/PR links. `synthetic` is true.
- `activitywatch-export.json`: ordinary ActivityWatch export structure with two
  devices overlapping for one person, quiet time, an excluded build interval,
  unattributed context, a missing bucket and a period with no telemetry. Raw
  private-path and hostname examples must disappear during sanitisation.
- `synthetic-truth.json`: a separate manufactured full set of allocations,
  including work during quiet and missing activity periods.
- `replay.py`: freezes predictions, saves a reproducible synthetic sample,
  generates participant-specific review forms and supplies only selected
  manufactured responses to the estimator. It checks the Rust estimates against
  independent arithmetic and verifies incomplete/invalid response handling.
- `report-workflow.yml`: an adoption template for completed private batches;
  it is not run by this synthetic example and does not publish reports by default.

With the retained seed, four of eight person-intervals are selected. The
manufactured population totals **74 person-minutes**, while this realised sample
produces:

| Quantity | Minutes |
| --- | ---: |
| Automatic activity proxy | 60 |
| Sample-adjusted estimate | 54 |
| Sampling-only estimate | 84 |
| Manufactured full-population total | 74 |

The sample-adjusted result is farther from the manufactured truth in this draw
than either comparator, and its estimated sampling variance is slightly larger
than the sampling-only variance. This is retained adverse evidence. A correction
does not promise a better realised estimate; the example verifies the estimator
contract, not an accuracy improvement. The sample-adjusted sampling standard
error is approximately 12.30 minutes and is not a complete measurement-error
bound or a confidence interval.

The output directory contains frozen inputs, draw provenance, offline HTML
forms, manufactured response files, issue and total Markdown/JSON reports, exact
Rust estimator inputs and `verification.json`. The missing-response case emits
an unavailable report with one nonresponse. An impossible allocation is rejected
without creating an estimate.

Follow [the adoption guide](../../docs/effort-estimation.md) for real collection,
issue association, participant checks and independent calibration.
