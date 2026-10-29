# Verification of the automatic forecasting extension

All checks below were executed on 29 October 2026. This record distinguishes
calculation checks, public-reference reconstruction, genuine issued forecasts and
still-unestablished predictive claims.

## Application and source handling

Pinned Rust 1.99.1 formatting, all-target Clippy with warnings denied, all **183**
application tests, documentation tests and the release build pass locally on Linux.
The 69 added tests cover:

- lightweight metadata/PR-only HTTP traversal, cache reuse, duplicates, invalid
  chronology, source failure and retained incomplete coverage;
- a common conservative scan-start cutoff, complete calendar bins, empty versus
  unavailable observations, stable repository identities and current-only pressure;
- empirical and Poisson calculations, distinct expanding/recent methods, future
  targets, fixed reference framing and correct mean/median/quantile interpretation;
- unchanged-forecast scoring, mature versus pending source exposure, half-open
  horizons, new arrivals, reopened requests and unavailable later portfolios;
- original snapshot/history/PMF binding, explicit replay rejection, renamed
  repositories, malformed receipts and exact saved floating-point round trips;
- mock-API command collection, immutable archive/export, offline replay, automatic
  pending scores and refusal to fetch when an export would overwrite an old file.

The existing lockfile is unchanged. `serde_json` enables its `float_roundtrip`
feature; no new crate is added. A strict saved-PMF regression caught one-ULP parser
changes before release. Correct rounding resolves the defect without weakening
receipt equality checks.

## Independent mathematics and public reference

The independent reviewer used an exact rational pairwise-distance RPS oracle,
including 300 randomized score-identity checks. A separate 80-digit decimal CDF
oracle checked the Poisson comparator over 40 cases with means from zero to 500.
Chronology and source fixtures were tested independently of the production test
expectations. These manufactured checks establish tested numerical behavior,
not forecast calibration.

Run from the repository root:

```sh
python3 paper/forecasting/check_reference_results.py --self-test
```

The standalone standard-library checker rebuilds the fixed 78-week counts from
742 retained PR identity/event rows. It checks the 52 initial training weeks and
26 common evaluation origins, all 130 five-method scores, source/report hashes,
ledger integrity and five corrupted-result cases. The compact ledger is derived
evidence, not the original raw snapshot or proof of historical access. Its source
content/file/acquisition hashes identify the separate retained snapshot. The
checker can re-derive it when that raw snapshot is supplied with `--snapshot`.
Both CI operating systems run this check.

The recent-eight candidate's primary mean RPS is 2.525841 versus 2.274930 for its
expanding reference: **11.03% worse**. Lower median point-forecast absolute error
does not reverse that result. The recent candidate was not promoted. No new model,
window or seasonal comparator was selected on these inspected outcomes. The
operational all-supported-history reference and the study's bounded 52–77-week
training frame differ. This one-repository reconstruction does not establish
prospective accuracy or transport to the user's repositories.

## Issued example and paper

The [public-source example](examples/README.md) is an actual automatic command
bundle, not a historical replay. Its gzip and uncompressed hashes match its
manifest. Independent verification of the original bundle and its embedded source
snapshot exactly reproduced the supplied **pending** score. The next-day target
had not started at declared creation. Local timestamps are declarations, not
independent trusted-time certificates.

Five common complete daily counts support mean 12.2 and predictive median 12.
The reported 8–17 empirical range encloses 100% of the issued discrete PMF; it has
no guaranteed future coverage. Weekly and fortnightly history are insufficient.
No mature user-outcome score, user predictive skill or future-bottleneck forecast
is claimed. Current draft/request/age predicates describe last-observed pressure.

The companion PDF was compiled with resolved citations and a clean LaTeX log.
All final pages were rendered and inspected. Generated result tables retain
canonical report inputs and their file hashes; retrospective table reconstruction
is distinguished from the original command-issued example.

## Retained material and scope

The full `make -C paper check` passes: historical package/hash/receipt checks,
exact measurement and labour-identification examples, seven retained kernel tests,
11 existing Python tests, and the new independent reference checker. No performance
recollection or new Lean execution was performed. At forecasting commit `1ef269334c774df4f6e46fe4a5bac52d52420ec3`, comparison
with parent `87372e5a140dff1df01ba68a25939c60e63031c1` confirmed that the original
83-file package, existing article/PDF and performance source/data package were
unchanged. The subsequent wording revision updates both authored papers and
the caption generator, as requested. Its verification is recorded separately in
[the two-paper wording review](../review/precise-writing-2026-10-29.md). The original
17-page manuscript and proof package, measured Rust kernels and retained benchmark
observations remain unchanged.

The [mesh decision record](../review/mesh-forecasting-2026-10-29/README.md) records
all 30 directed worker peer edges, the primary-source research and the adverse
result. Agent review and successful tests do not substitute for external peer
review or future empirical validation. Remote CI status is reported separately
in the pull request after GitHub executes it.
