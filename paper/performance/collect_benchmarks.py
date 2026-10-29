#!/usr/bin/env python3
"""Version 2 collector: bind Cargo's executable and write a separate experiment."""
import argparse
import csv
import datetime
import hashlib
import json
import os
import pathlib
import platform
import subprocess
import sys

from run_benchmarks import HEADER, QS, REPETITIONS, ROOT, SEEDS, read_optional, source_hashes
from summarise_collection import write_summary

SCHEMA = "cohort-horizon.collection/2"
BUILD_ENVIRONMENT = (
    "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR", "CARGO_BUILD_RUSTFLAGS", "RUSTUP_TOOLCHAIN", "RUSTC", "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_PROFILE_RELEASE_OPT_LEVEL", "CARGO_PROFILE_RELEASE_CODEGEN_UNITS",
    "CARGO_PROFILE_RELEASE_LTO", "CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS",
    "CARGO_PROFILE_RELEASE_PANIC", "CARGO_PROFILE_RELEASE_DEBUG",
    "CARGO_PROFILE_RELEASE_INCREMENTAL",
)


def digest(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc)


def select_executable(messages, package_id):
    """Select the intended Cargo bin artifact, including a cached/fresh artifact."""
    candidates = set()
    for message in messages:
        target = message.get("target") or {}
        if (message.get("reason") == "compiler-artifact"
                and message.get("package_id") == package_id
                and target.get("name") == "horizon-bench"
                and "bin" in target.get("kind", [])
                and message.get("executable")):
            candidates.add(pathlib.Path(message["executable"]).resolve())
    if len(candidates) != 1:
        raise ValueError(f"Expected one horizon-bench executable artifact, found {len(candidates)}")
    return candidates.pop()


def build_executable(cargo):
    metadata = json.loads(subprocess.check_output(
        [cargo, "metadata", "--locked", "--no-deps", "--format-version=1"], cwd=ROOT, text=True))
    packages = [package for package in metadata["packages"]
                if pathlib.Path(package["manifest_path"]).resolve() == (ROOT / "Cargo.toml").resolve()]
    if len(packages) != 1:
        raise ValueError("Cannot identify the performance crate in Cargo metadata")
    package_id = packages[0]["id"]
    command = [cargo, "build", "--release", "--locked", "--bin", "horizon-bench", "--message-format=json"]
    result = subprocess.run(command, cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE)
    messages = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    executable = select_executable(messages, package_id)
    if not executable.is_file():
        raise ValueError(f"Cargo's executable artifact is missing: {executable}")
    return executable, package_id, command


def collector_hashes():
    hashes = source_hashes()
    for name in ("collect_benchmarks.py", "summarise_collection.py"):
        hashes[name] = digest(ROOT / name)
    return hashes


def cargo_config_hashes():
    # Keep configuration identities without copying registry credentials or other
    # potentially private configuration contents into the collection manifest.
    candidates = set()
    for parent in (ROOT, *ROOT.parents):
        candidates.update(parent / ".cargo" / name for name in ("config", "config.toml"))
    cargo_home = pathlib.Path(os.environ.get("CARGO_HOME", pathlib.Path.home() / ".cargo"))
    candidates.update(cargo_home / name for name in ("config", "config.toml"))
    return {str(path): digest(path) for path in sorted(candidates) if path.is_file()}


def prepare_output_directory(directory):
    directory = pathlib.Path(directory).resolve()
    retained = (ROOT / "results").resolve()
    if directory == retained or retained in directory.parents:
        raise ValueError("Choose a new output directory; retained results are immutable")
    directory.mkdir(parents=True, exist_ok=False)
    return directory


def parse_row(line, seed, q, rep, first):
    values = next(csv.reader([line]))
    if len(values) != len(HEADER):
        raise ValueError("Unexpected benchmark result column count")
    row = dict(zip(HEADER, values))
    expected = {"seed": seed, "rep": rep, "q": q, "accepted_count": 100_000,
                "labour_count": 400_000, "delta_hours": 720}
    if row["first"] != first or any(int(row[key]) != value for key, value in expected.items()):
        raise ValueError("Benchmark result does not match the requested protocol")
    for field in ("scan_ns", "index_build_ns", "index_query_ns", "index_drop_ns", "index_total_ns", "checksum"):
        if int(row[field]) < 0:
            raise ValueError(f"Negative benchmark value: {field}")
    if int(row["scan_ns"]) == 0 or int(row["index_total_ns"]) == 0:
        raise ValueError("Measured batch times must be positive")
    return values


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    default = ROOT.parent.parent / "reports" / "performance" / utc_now().strftime("%Y%m%dT%H%M%S.%fZ")
    parser.add_argument("--outputdir", "--output-dir", type=pathlib.Path, default=default,
                        help="new experiment directory (default: ignored reports/performance/<UTC>)")
    args = parser.parse_args(argv)
    try:
        output = prepare_output_directory(args.outputdir)
    except (ValueError, FileExistsError) as error:
        parser.error(str(error))
    cargo = os.environ.get("CARGO", "cargo")
    hashes = collector_hashes()
    executable, package_id, command = build_executable(cargo)
    binary_hash = digest(executable)
    cpu = read_optional("/proc/cpuinfo") or ""
    model = next((line.split(":", 1)[1].strip() for line in cpu.splitlines()
                  if line.startswith("model name")), None)
    manifest = {
        "schema": SCHEMA, "collector": "collect_benchmarks.py",
        "command": [sys.executable, str(ROOT / "collect_benchmarks.py"), "--outputdir", str(output)],
        "started_utc": utc_now().isoformat(), "output_directory": str(output),
        "seeds": SEEDS, "query_counts": QS, "repetitions_per_seed_q": REPETITIONS,
        "independent_unit": "One fresh process runs one paired comparison after one warmup of each method; repeated host runs are not independent production samples.",
        "order": "scan first iff (seed_index + rep + q_index) is even; nested loops rep, seed, q",
        "scope": "In-memory query batch: indexed total includes owned copies, sorting, prefix construction, queries and disposal. Generation, process startup, warmup, verification and I/O excluded.",
        "preselected_threshold": "At q=1000, indexed median total <= 0.90 * scan median. Report every q, including regressions.",
        "host": platform.platform(), "cpu_model": model,
        "available_cpus": len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count(),
        "cpu_max": read_optional("/sys/fs/cgroup/cpu.max"),
        "memory_max": read_optional("/sys/fs/cgroup/memory.max"),
        "rustc": subprocess.check_output([os.environ.get("RUSTC", "rustc"), "-Vv"], cwd=ROOT, text=True),
        "cargo": subprocess.check_output([cargo, "--version"], cwd=ROOT, text=True).strip(),
        "python": platform.python_version(), "build_command": command,
        "cargo_package_id": package_id, "executable": str(executable), "binary_sha256": binary_hash,
        "build_environment": {
            key: value for key, value in os.environ.items()
            if key in BUILD_ENVIRONMENT or (key.startswith("CARGO_TARGET_")
                                           and key.endswith(("_RUSTFLAGS", "_LINKER", "_RUNNER")))
        },
        "cargo_config_sha256": cargo_config_hashes(),
        "base_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "source_sha256": hashes,
        "exclusions": "None; failures abort collection and retain partial new-directory evidence.",
        "load_control": "Sequential runner; shared-host load is uncontrolled.",
    }
    environment = output / "environment.json"
    environment.write_text(json.dumps(manifest, indent=2) + "\n")
    with (output / "raw.csv").open("x", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(HEADER)
        for rep in range(REPETITIONS):
            for seed_index, seed in enumerate(SEEDS):
                for q_index, q in enumerate(QS):
                    first = "scan" if (seed_index + rep + q_index) % 2 == 0 else "index"
                    line = subprocess.check_output(
                        [str(executable), str(seed), str(q), str(rep), first], text=True).strip()
                    writer.writerow(parse_row(line, seed, q, rep, first))
                    stream.flush()
                    print(f"seed={seed} rep={rep} q={q} completed", flush=True)
    if digest(executable) != binary_hash or collector_hashes() != hashes:
        raise RuntimeError("Source or executed binary changed during collection")
    manifest["finished_utc"] = utc_now().isoformat()
    manifest["raw_sha256"] = digest(output / "raw.csv")
    environment.write_text(json.dumps(manifest, indent=2) + "\n")
    write_summary(output)
    print(f"Collection and summaries: {output}")


if __name__ == "__main__":
    main()
