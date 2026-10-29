use cohort_horizon_performance::{scan_batch, Cohort, CohortIndex, Totals, WeightedEvent};
use std::env;
use std::hint::black_box;
use std::time::Instant;

const ACCEPTANCES: usize = 100_000;
const LABOUR_EVENTS: usize = 400_000;
const DELTA_HOURS: u64 = 720;

// SplitMix64 is used only to specify a reproducible synthetic workload, not inference.
fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn dataset(seed: u64, q: usize) -> (Cohort, Vec<u64>) {
    let mut state = seed;
    let acceptances = (0..ACCEPTANCES)
        .map(|_| WeightedEvent {
            age: next(&mut state) % 1_441,
            weight: next(&mut state) % 10_000_001,
        })
        .collect();
    let labour = (0..LABOUR_EVENTS)
        .map(|_| WeightedEvent {
            age: next(&mut state) % 2_161,
            weight: next(&mut state) % 1_000_001,
        })
        .collect();
    let horizons = (0..q).map(|_| next(&mut state) % 1_441).collect();
    (
        Cohort {
            acceptances,
            labour,
        },
        horizons,
    )
}

fn digest(values: &[Totals]) -> u128 {
    values.iter().fold(0_u128, |acc, value| {
        acc.rotate_left(13)
            .wrapping_add(value.retained_output)
            .wrapping_mul(31)
            .wrapping_add(value.labour)
    })
}

fn scan(cohort: &Cohort, horizons: &[u64]) -> (Vec<Totals>, u128) {
    let start = Instant::now();
    let output = scan_batch(black_box(cohort), black_box(horizons), DELTA_HOURS).unwrap();
    let ns = start.elapsed().as_nanos();
    black_box(digest(&output));
    (output, ns)
}

fn indexed(cohort: &Cohort, horizons: &[u64]) -> (Vec<Totals>, [u128; 4]) {
    let total_start = Instant::now();
    let index = CohortIndex::new(black_box(cohort)).unwrap();
    let build_ns = total_start.elapsed().as_nanos();
    let query_start = Instant::now();
    let output = index.query_batch(black_box(horizons), DELTA_HOURS).unwrap();
    let query_ns = query_start.elapsed().as_nanos();
    let drop_start = Instant::now();
    drop(index);
    let drop_ns = drop_start.elapsed().as_nanos();
    let total_ns = total_start.elapsed().as_nanos();
    black_box(digest(&output));
    (output, [build_ns, query_ns, drop_ns, total_ns])
}

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 5, "usage: horizon-bench SEED Q REP scan|index");
    let seed: u64 = args[1].parse().unwrap();
    let q: usize = args[2].parse().unwrap();
    let rep: usize = args[3].parse().unwrap();
    let first = &args[4];
    assert!(matches!(first.as_str(), "scan" | "index"));
    assert!(matches!(q, 1 | 10 | 100 | 1000));
    let (cohort, horizons) = dataset(seed, q);

    // One full, unmeasured warmup per method. It also verifies exact equality.
    let scan_warm = scan(&cohort, &horizons).0;
    let index_warm = indexed(&cohort, &horizons).0;
    assert_eq!(scan_warm, index_warm);
    drop(scan_warm);
    drop(index_warm);

    let (scan_result, scan_ns, index_result, timings) = if first == "scan" {
        let (a, ns) = scan(&cohort, &horizons);
        let (b, timings) = indexed(&cohort, &horizons);
        (a, ns, b, timings)
    } else {
        let (b, timings) = indexed(&cohort, &horizons);
        let (a, ns) = scan(&cohort, &horizons);
        (a, ns, b, timings)
    };
    assert_eq!(scan_result, index_result);
    let [build_ns, query_ns, drop_ns, total_ns] = timings;
    println!(
        "{seed},{rep},{q},{first},{ACCEPTANCES},{LABOUR_EVENTS},{DELTA_HOURS},{scan_ns},{build_ns},{query_ns},{drop_ns},{total_ns},{}",
        digest(&scan_result)
    );
}
