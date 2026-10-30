#!/usr/bin/env python3
"""Reproduce this frozen public use case without collecting new evidence."""
import argparse
import csv
import gzip
import hashlib
import json
import subprocess
import tempfile
from collections import Counter
from pathlib import Path


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def main():
    case = Path(__file__).resolve().parent
    repo = case.parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--observatory", type=Path, default=repo / "target/release/observatory")
    args = parser.parse_args()
    binary = args.observatory.resolve()
    manifest = json.loads((case / "manifest.json").read_bytes())
    for name, expected in manifest["files"].items():
        require(digest((case / name).read_bytes()) == expected, f"Hash mismatch: {name}")
    raw = gzip.decompress((case / "issued-day.json.gz").read_bytes())
    require(digest(raw) == manifest["uncompressed_bundle_sha256"], "Bundle hash mismatch")
    bundle = json.loads(raw)
    snapshot = bundle["snapshot"]
    require(snapshot["sha256"] == manifest["source_snapshot_sha256"], "Snapshot ID mismatch")
    require(snapshot["implementation_revision"] == manifest["source_revision"], "Producer mismatch")
    require(snapshot["implementation_source_sha256"] == manifest["implementation_source_sha256"],
            "Source hash mismatch")

    # Count raw PR payloads independently of the application's prepared history.
    sources = {}
    for retained in snapshot["records"]:
        data = retained["record"]["data"]
        if data.get("source") == "github.pull_request":
            pr = data["payload"]
            key = (int(data["repository"].rsplit(":", 1)[1]), pr["number"])
            require(key not in sources, f"Duplicate PR source: {key}")
            sources[key] = pr
    with (case / "daily-counts.csv").open(newline="") as stream:
        rows = list(csv.DictReader(stream))
    require(len(rows) == len(bundle["forecast"]["training_bins"]), "Count frame mismatch")
    for row, training in zip(rows, bundle["forecast"]["training_bins"]):
        require((row["start_utc"], row["end_utc"]) == (training["start"], training["end"]),
                "Count period mismatch")
        counts = Counter(repo_id for (repo_id, _), pr in sources.items()
                         if pr["merged_at"] is not None
                         and row["start_utc"] <= pr["merged_at"] < row["end_utc"])
        require(counts[1427362428] == int(row["eal_mcp_merged_prs"]), "EAL count mismatch")
        require(counts[1431331749] == int(row["engarg_merged_prs"]), "EngArg count mismatch")
        require(sum(counts.values()) == int(row["portfolio_merged_prs"]) == training["count"],
                "Portfolio count mismatch")

    previous = gzip.decompress((repo / manifest["prior_bundle_path"]).read_bytes())
    require(digest(previous) == manifest["prior_uncompressed_bundle_sha256"], "Prior hash mismatch")
    work = repo / "work"
    work.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="reproduce-use-case-", dir=work) as temp:
        temp = Path(temp)
        source = temp / "snapshot.json"
        source.write_text(json.dumps(snapshot))
        issued = temp / "issued.json"
        issued.write_bytes(raw)
        prior = temp / "prior.json"
        prior.write_bytes(previous)

        def run(*command):
            return json.loads(subprocess.check_output([str(binary), *map(str, command)], cwd=repo))

        day = run("forecast-snapshot", source, "--horizon", "day")
        require(day["forecast"] == bundle["forecast"], "Daily replay mismatch")
        require(day["history"] == bundle["history"], "History replay mismatch")
        week = run("forecast-snapshot", source, "--horizon", "week")
        require(week == json.loads((case / "week-replay.json").read_bytes()), "Weekly replay mismatch")
        comparison = run("forecast-snapshot", source, "--horizon", "day", "--backtest")
        require(comparison == json.loads((case / "daily-reconstruction.json").read_bytes()),
                "Historical comparison mismatch")
        scores = json.loads((case / "prior-forecast-scores.json").read_bytes())
        require(scores["later_snapshot_sha256"] == snapshot["sha256"], "Score cut mismatch")
        require(len(scores["entries"]) == 1, "Unexpected prior score inventory")
        expected = scores["entries"][0]
        require(expected["error"] is None, "Prior archive has an error")
        require(expected["archive_sha256"] == digest(previous), "Prior score identity mismatch")
        require(run("score-forecast", prior, source) == expected["score"], "Prior score mismatch")
        # score-forecast verifies original receipt chronology and embedded sources.
        require(run("score-forecast", issued, source)["status"] == "pending", "New receipt not pending")
    print("Verified hashes, raw daily counts, original receipt, both horizons, "
          "retrospective comparison and prior pending score.")


if __name__ == "__main__":
    main()
