# Data-science improvements

These are proposed additions, not implemented features. They arise from the
current code and the measurement contract. The user-nominated Rust data-science
book is retained in [the bibliography](../references.bib); its exact edition and
contents have not been verified, so none of these proposals is attributed to it.

## 1. Export analysis-ready tables

The present [report writer](../src/report.rs) exports `records.csv` as evidence
envelopes with a `data_json` column. Those rows preserve original observations;
the calculated report uses the corrected effective study view. This is useful
for audit but awkward for ordinary statistical work.

Add versioned `tasks.csv`, `cohorts.csv`, `effort.csv` and `assessments.csv`.
Calculate task/cohort rows from the existing analysis result and effort/assessment
rows from `effective_records` at the snapshot cut. Retain `records.csv` unchanged
as the original evidence export. For a snapshot-replay export, filter each
time-stamped observation to the cut and mark whether it contributes within the
declared accounting horizon; do not export future observations as current facts.

Every row needs snapshot hash, calculation version, stable identity, units,
measurement method and explicit availability/status. Store missing numeric values
as empty fields with a separate status/reason; observed zero stays numeric zero.
Use a link table for multiple evidence IDs instead of multiplying measurement rows.
Preserve the existing spreadsheet-formula protection when adopting a general CSV
writer. See the [maintainer CSV API](https://docs.rs/csv/1.4.0/csv/struct.Writer.html).

**Acceptance:** an effort or assessment correction changes the effective table
without rewriting original evidence. Missing and zero round-trip distinctly;
exported totals equal the report at the same cut; all rows identify their snapshot.

Start with typed CSV. Consider optional Arrow/Parquet after measuring a dataset or
downstream-tool need. Apache's [Rust ArrowWriter](https://arrow.apache.org/rust/parquet/arrow/arrow_writer/struct.ArrowWriter.html)
provides a possible boundary; it does not require a dataframe framework in the
collection or accounting core.

## 2. Add a measurement worklist

Task-level issues already appear in [analysis results](../src/analysis.rs), but
operators must inspect individual reports to find missing frames, nonresponse and
overdue retention audits. Add a read-only command such as `check-data` or
`audit-due`, producing JSON and an operator-readable list from a frozen snapshot.

**Acceptance:** list missing/partial effort coverage, nonresponse, unresolved
links and pending/overdue assessments. Calculate audit due dates from acceptance
plus follow-up, and labour maturity from enrolment plus both horizons. An early
retention audit must not close labour accounting. Corrected observations must
clear only the issues they actually resolve. The command must not create complete
coverage attestations automatically.

## 3. Support prespecified descriptive breakdowns

[Cohorts](../src/analysis/cohort.rs) currently group by protocol. Task class,
assigned policy and assignment unit are already retained. An explicit analysis
specification could request distributions and summaries by these fields, making
task mix and observation gaps easier to inspect.

**Acceptance:** include counts, missingness, maturity and direct/sampled-effort
composition for every group. Preserve unsuccessful work, keep unassigned labels
unknown, and document the allocation of uncounted parent/child labour. Reject or
hold a comparison when that allocation is ambiguous; do not duplicate shared
effort. Label policy comparisons descriptive. Specify any calendar or intake
window and outcome horizon before selecting rows.

## 4. Add uncertainty for one explicit sampling design

[Labour accounting](../src/analysis/labour.rs) supports Horvitz--Thompson point
estimates but intentionally cannot estimate variance from arbitrary sampling
probabilities alone. Implement one declared design before adding generic error
bars, for example stratified simple random sampling without replacement.

**Prerequisites:** retain the person-interval frame, stratum identities, frame and
sample sizes, selection design and response status. Define how task allocations
and shared person-intervals enter joint totals. Nonresponse still needs an explicit
measurement assumption or sensitivity analysis.

**Acceptance:** check total/variance calculations against finite enumerated
samples. Insufficient within-stratum observations yield unavailable variance.
Productivity uncertainty must preserve numerator/denominator dependence; the
design does not justify treating commits, individual rows or two coupled
repositories as independent treatment assignments.

## Later, after the pilot

Assess frozen-score calibration against independently measured effort using a
simple task-class baseline and held-out future periods or independent teams.
Report error, calibration and missingness by policy/class, with sampled-effort
uncertainty carried through. Keep retrospective calibration separate from frozen
intake scores. Prediction and causal-effect claims require their own contracts.

Profile actual collection and analysis workloads before changing data structures,
parallelising Git inspection or adopting Polars, machine-learning crates or a new
storage service. A Rust/data-science resource can inform those choices after its
relevant content is inspected; its title alone is not a reason to add dependencies.
