#!/usr/bin/env python3
"""Run the predeclared paired, separate-process experiment sequentially."""
import csv
import hashlib
import json
import os
import pathlib
import platform
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent
RESULTS = ROOT / "results"
SEEDS = (17, 41, 93)
QS = (1, 10, 100, 1000)
REPETITIONS = 5
HEADER = ["seed", "rep", "q", "first", "accepted_count", "labour_count", "delta_hours",
          "scan_ns", "index_build_ns", "index_query_ns", "index_drop_ns", "index_total_ns", "checksum"]


def read_optional(path):
    try:
        return pathlib.Path(path).read_text().strip()
    except OSError:
        return None


def source_hashes():
    files = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "rust-toolchain.toml",
             ROOT / "run_benchmarks.py", ROOT / "summarise.py", *sorted((ROOT / "src").rglob("*.rs"))]
    return {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}


def main():
    RESULTS.mkdir(exist_ok=True)
    if (RESULTS / "raw.csv").exists():
        sys.exit("results/raw.csv exists: move it explicitly before collecting a new experiment")
    cargo = os.environ.get("CARGO", "cargo")
    subprocess.run([cargo, "build", "--release", "--locked"], cwd=ROOT, check=True)
    binary = ROOT / "target/release/horizon-bench"
    cpu = read_optional("/proc/cpuinfo") or ""
    model = next((line.split(":", 1)[1].strip() for line in cpu.splitlines() if line.startswith("model name")), None)
    manifest = {
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "command": "python3 run_benchmarks.py",
        "seeds": SEEDS, "query_counts": QS, "repetitions_per_seed_q": REPETITIONS,
        "independent_unit": "One fresh process runs one paired comparison after one warmup of each method; repeated host runs are not random independent production samples.",
        "order": "scan first iff (seed_index + rep + q_index) is even; nested loops rep, seed, q",
        "scope": "In-memory query batch: indexed total includes owned input copies, sorting, prefix construction, queries and index disposal. Input generation, process startup, warmup, output verification and result I/O excluded equally.",
        "preselected_threshold": "At q=1000, indexed median total <= 0.90 * scan median. Report every q, including regressions.",
        "host": platform.platform(), "cpu_model": model,
        "available_cpus": len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count(),
        "cpu_max": read_optional("/sys/fs/cgroup/cpu.max"),
        "memory_max": read_optional("/sys/fs/cgroup/memory.max"),
        "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
        "cargo": subprocess.check_output([cargo, "--version"], text=True).strip(),
        "python": platform.python_version(),
        "base_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "source_sha256": source_hashes(),
        "exclusions": "None; subprocess failures abort collection and retain the partial CSV.",
        "load_control": "Single benchmark runner; no intentional parallel benchmark or compilation. Shared host load is uncontrolled.",
    }
    (RESULTS / "environment.json").write_text(json.dumps(manifest, indent=2) + "\n")
    with (RESULTS / "raw.csv").open("w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(HEADER)
        for rep in range(REPETITIONS):
            for seed_index, seed in enumerate(SEEDS):
                for q_index, q in enumerate(QS):
                    first = "scan" if (seed_index + rep + q_index) % 2 == 0 else "index"
                    line = subprocess.check_output([str(binary), str(seed), str(q), str(rep), first], text=True).strip()
                    row = next(csv.reader([line]))
                    if len(row) != len(HEADER):
                        raise RuntimeError(f"Unexpected result row: {line}")
                    writer.writerow(row)
                    f.flush()
                    print(f"seed={seed} rep={rep} q={q} completed", flush=True)
    manifest["finished_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    manifest["raw_sha256"] = hashlib.sha256((RESULTS / "raw.csv").read_bytes()).hexdigest()
    (RESULTS / "environment.json").write_text(json.dumps(manifest, indent=2) + "\n")
    subprocess.run([sys.executable, str(ROOT / "summarise.py")], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
