# Repeated cohort-horizon queries

This new, bounded optimisation example compares a direct scan with an immutable
sorted prefix index. It implements discrete accounting queries related to the
paper's retained-output ratio. It is not an implementation of the paper's proposed
empirical study, the preserved original supplement, or a measure of AI effects.

## Contract and units

`WeightedEvent { age: u64, weight: u64 }` holds a nonnegative age since the relevant
task's enrolment and a nonnegative fixed-point weight. In this experiment ages are
integer hours, accepted-output weights are millionths of a reference unit, and
labour weights are millionths of a person-hour. Each accepted requirement has one
retained-output mark assessed at the same fixed follow-up `delta`. Each attributed
labour increment occurs once, including labour from unaccepted tasks. Eligibility,
retention assessment, attribution and absence of double counting are input
conditions; these arrays cannot establish them.

For each acceptance horizon `tau`, both implementations return the exact pair

```
retained_output = sum(event.weight for acceptances with event.age <= tau)
labour          = sum(event.weight for labour with event.age <= tau + delta)
```

All boundaries are inclusive, including tied ages. Queries may be unsorted or
repeated; output order matches input order. `tau + delta` uses checked `u64`
addition, and accumulation uses checked `u128` addition. Overflow returns
`AccountingError` rather than wrapping. The canonical output is `Totals`, not a
floating ratio. Its optional `ratio()` is a display conversion to `f64` and returns
`None` when labour is zero; positive output is still retained in the exact pair.
An empty query batch returns an empty result.

These finite weighted records are a **discrete accounting approximation** to the
continuous integral in the article. A labour interval straddling a queried cutoff
needs a declared splitting rule or finer time discretisation; assigning its full
weight to one event does not integrate a partial interval. The code does not infer
missing labour, retained quality or causal effects. It also does not adjust a
retention mark when `delta` changes: callers must supply marks for the declared
follow-up and rebuild any affected snapshot.

## Implementations and equivalence

`scan_batch(&Cohort, horizons, delta)` rescans both arrays for every query. For `n`
acceptances, `m` labour increments and `q` horizons, it examines `q(n + m)` records:
`O(q(1 + n + m))` time, including empty streams, and `O(q)` result/cutoff storage.

`CohortIndex::new(&Cohort)` copies each array, sorts by age and constructs an exact
prefix sum with an initial zero. `query_batch(horizons, delta)` uses an upper-bound
binary search to find the first age greater than each cutoff. The corresponding
prefix is precisely the sum used by the scan: sorting preserves the multiset, and
the upper bound includes all and only qualifying ages. Equal ages require no stable
order because checked integer addition is exact for these nonnegative inputs.

Build time is `O(1 + n log(n+1) + m log(m+1))` in the general comparison model; batch-query
time is `O(q(1 + log(n+1) + log(m+1)))`, including empty streams. Retained index storage is approximately
`24(n + m) + 32` bytes of vector payload on this 64-bit implementation, excluding
vector headers, spare capacity, allocator overhead and query results. Construction
also copies event records. The index owns a snapshot: source mutation does not
update it. Rebuild when any indexed data change.

The seven unit tests cover empty data/queries, unsorted and duplicate horizons,
inclusive boundaries, zero weights, absent acceptances, zero labour, explicit
horizon overflow, totals exceeding `u64`, snapshot ownership, and generated exact
comparisons with the scan oracle. A `u128` total cannot overflow for a physically
addressable vector of `u64` weights on the measured 64-bit platform, but accumulation
still checks it explicitly. Timing thresholds are not correctness tests.

## Reproduce

Rust 1.85.1 is pinned; the crate has no third-party dependencies. From this directory:

```sh
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo fmt --check
python3 -m unittest discover -s tests -v
python3 collect_benchmarks.py
```

`collect_benchmarks.py` is the explicitly versioned successor collector
(`cohort-horizon.collection/2`). It writes a new collection by default under the
repository's ignored `reports/performance/<UTC>/` directory; choose another new
directory with `--outputdir PATH`. Existing output directories and the retained
`results/` directory are refused. The raw observations, environment manifest,
summary and LaTeX table all stay in the selected new directory.

The successor selects the intended `horizon-bench` executable from Cargo's
`compiler-artifact` JSON, including a cached artifact. It respects configured
target directories and target triples, records the selected executable's SHA-256,
and records its build command, relevant environment overrides and Cargo
configuration hashes. A stale executable at `target/release/horizon-bench` is not
selected merely because it exists there. The successor uses the same declared
workload and paired-run protocol; no new timings are asserted by this collector
repair. `python3 summarise_collection.py PATH` regenerates summaries for a new
collection without running measurements.

The retained 29 October experiment was collected with `run_benchmarks.py` and
summarised with `summarise.py`. Those hash-bound source files, all seven recorded
source hashes, the published 60 pairs, and the figure's input bindings are
preserved. The old collector's fixed executable path does not support arbitrary
Cargo target configuration; no such mismatch was found in the retained evidence.
Future collection uses the successor instead of moving or replacing `results/`.
`python3 summarise.py` remains the frozen summariser for the retained observations.

The release profile has optimisation level 3, one codegen unit, no LTO, unwinding
panics and overflow checks enabled. Build, generation, warmup, result comparison
and output I/O are outside both timed paths. The indexed total includes input
copies, sorting, prefix construction, batch querying and index destruction. Its
build/query/drop subtimings are diagnostics; independent total timing is the metric.
Both paths allocate the same output shape and consume the result through
`std::hint::black_box`; exact outputs are compared outside timing.

## Predeclared experiment

The selected decision rule is at least a **10% reduction in median total batch
time at `q = 1000`**, including construction. All tested batch sizes (`1`, `10`,
`100`, `1000`) remain in the record, including regressions. No search for a favourable
seed or repeated reruns is part of the protocol.

Each dataset has 100,000 accepted-output records and 400,000 labour increments.
The benchmark generates records with SplitMix64 from seed `17`, `41` or `93`.
Starting from that seed, two PRNG outputs per acceptance produce `age mod 1441` and
`weight mod 10000001`; two outputs per labour increment then produce `age mod 2161`
and `weight mod 1000001`; subsequent outputs produce query horizons `mod 1441`.
The fixed follow-up is 720 hours. Thus all datasets contain many tied ages, while
larger query batches preserve the same horizon prefix for a given seed. The PRNG
and modulo operations define synthetic inputs, not a representative task model.
The three seeds specify this new experiment separately from the simulations
in the preserved original supplement.

Five fresh-process paired comparisons per seed and query count give 15 pairs for
each `q`, 60 pairs in total. Each process warms the scan first and then the index once each, then measures
each once; scan goes first in the measured pair exactly when `(seed_index + repetition + q_index)` is even.
The runner visits repetition, seed, query count in that nesting order and runs
only one process at a time. This interleaves methods and sizes but does not make
shared-host noise or repeated observations independent production samples. Reported
ranges describe these observations; they are not confidence intervals or p99s.

`results/raw.csv` retains nanosecond timings and output checksums for every pair.
`results/summary.csv` gives medians, observed ranges, ratio of medians and paired
reduction extrema. `results/environment.json` records the toolchain, host, resource
limits, source SHA-256 hashes, command, policy and exclusions. No observations are
silently discarded. The measured result and its limits are discussed in the paper's
performance section; no production-system or AI-effect improvement follows from
this synthetic in-memory kernel alone.


## Recorded result

The predeclared high-query threshold was met on this host. Every one of the 60
paired runs passed exact equality, with no exclusions or reruns. The following are
medians and observed min–max ranges over the 15 pairs at each query count; all values
include the stated complete batch boundary and are in milliseconds.

| Queries | Scan median [range] | Indexed total median [range] | Reduction from medians |
| ---: | ---: | ---: | ---: |
| 1 | 2.889 [1.008–6.214] | 21.016 [16.248–34.637] | −627.4% |
| 10 | 28.003 [20.162–42.507] | 20.709 [16.702–39.722] | 26.0% |
| 100 | 263.415 [232.034–289.519] | 23.046 [17.468–37.871] | 91.3% |
| 1000 | 2787.159 [2625.665–3988.895] | 23.051 [17.132–47.836] | 99.2% |

At 1,000 queries, the absolute median reduction is 2,764.108 ms and the ratio of
medians is 120.91. Every paired reduction is between 98.35% and 99.38%. This is a
verified improvement for the stated batch kernel. At one query, preprocessing
makes indexing slower in every pair. At ten queries, the median favours indexing
but paired reductions range from −16.35% to 55.14%, so a stable crossover at ten
queries is not established. The first tested size faster in every pair is 100;
this does not locate an exact or portable crossover.

The run on 29 October 2026 used an Intel Xeon Platinum 8573C, Linux 6.18.55,
glibc 2.41 and Rust 1.85.1 (LLVM 19.1.7), with five CPUs in the process affinity,
a four-CPU cgroup quota and a 16 GiB memory limit. The algorithms are sequential;
the benchmark runner did not launch parallel workers. No intentional competing
compilation or benchmark ran, but shared-host contention was uncontrolled. These
are warmed measurements: process startup and cold-cache behaviour are excluded.

The retained index vector payload is approximately 12,000,032 bytes (11.44 MiB)
for this input, in addition to 8,000,000 bytes of raw event payload. While building
the larger labour index, raw input, the completed acceptance index, the copied
labour array and its new times/prefix arrays together account for approximately
26,400,032 bytes (25.18 MiB). These are structural payload calculations, **not
measured peak RSS**; allocator overhead, spare capacity, result/cutoff vectors and
runtime memory are additional. The repeated-scan method avoids this index cost.

Verification completed before timing: `cargo test --locked` (seven tests),
`cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and
`cargo build --release --locked`. An independent read-only review found no
semantic-equivalence or timing-boundary defect. Benchmark source files have not
changed after the run; their hashes are recorded in `environment.json`.
