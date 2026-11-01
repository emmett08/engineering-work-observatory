#!/usr/bin/env python3
"""Replay the manufactured effort pilot; this is not a human accuracy study."""

from __future__ import annotations

import argparse
from copy import deepcopy
from datetime import datetime
import json
import math
from pathlib import Path
import subprocess
import sys


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
CLI = REPO / "instrumentation" / "effort.py"


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def instant(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--observatory", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    binary = args.observatory.resolve()
    output = args.output_dir.resolve()
    if not binary.is_file():
        parser.error("build Observatory first and supply its executable path")
    output.mkdir(parents=True, exist_ok=True)
    if any(output.iterdir()):
        parser.error("output directory must be empty; retain earlier replay results")

    def run(*arguments, succeeds=True):
        completed = subprocess.run(
            [sys.executable, str(CLI), *map(str, arguments)],
            cwd=REPO, text=True, capture_output=True, check=False,
        )
        if succeeds and completed.returncode:
            raise RuntimeError(completed.stdout + completed.stderr)
        if not succeeds and completed.returncode == 0:
            raise AssertionError("invalid evidence unexpectedly produced an estimate")
        return completed

    run("collect", "--config", HERE / "config.json", "--export",
        HERE / "activitywatch-export.json", "--output", output / "activity.json")
    run("freeze", "--config", HERE / "config.json", "--activity",
        output / "activity.json", "--output", output / "frame.json")
    run("sample", "--frame", output / "frame.json", "--size", 4,
        "--synthetic-seed", "0" * 64, "--output", output / "sample.json")
    run("review", "--frame", output / "frame.json", "--sample",
        output / "sample.json", "--all-people", "--output", output / "review-observer.html")
    run("template", "--frame", output / "frame.json", "--sample",
        output / "sample.json", "--all-people", "--output", output / "responses-template.json")

    activity = load(output / "activity.json")
    frame = load(output / "frame.json")
    sample = load(output / "sample.json")
    assert len(frame["units"]) == 8
    assert len(frame["frames"]) == 2
    assert len(sample["selected_unit_ids"]) == 4
    assert sample["inclusion_probability"] == 0.5
    assert any(source["status"] == "missing" for source in activity["sources"])
    sanitized = json.dumps(activity)
    for secret in ("private-host-must-not-be-retained", "/private/", "unconfigured-private-project", "private-topic"):
        assert secret not in sanitized
    assert any(unit["coverage"]["unknown_seconds"] > 0 for unit in frame["units"])

    by_interval = {(unit["person_id"], instant(unit["start"])): unit for unit in frame["units"]}
    overlap = by_interval[("participant-a", instant("2026-11-01T09:00:00Z"))]
    assert math.isclose(overlap["predicted_person_hours"], 0.25, abs_tol=1e-12)
    for unit in frame["units"]:
        assert 0 <= unit["predicted_person_hours"] <= 0.25
    for person, start in [("participant-a", "2026-11-01T09:15:00Z"),
                          ("participant-a", "2026-11-01T09:45:00Z"),
                          ("participant-b", "2026-11-01T09:45:00Z")]:
        assert by_interval[(person, instant(start))]["predicted_person_hours"] == 0

    # These labels are deliberately manufactured, never collected from people.
    # They are applied only after prediction freeze and the saved sample draw.
    truth = load(HERE / "synthetic-truth.json")
    truth_lookup = {(row["person_id"], instant(row["start"])): row for row in truth["units"]}
    unit_lookup = {unit["unit_id"]: unit for unit in frame["units"]}
    responses = load(output / "responses-template.json")
    for response in responses["responses"]:
        unit = unit_lookup[response["unit_id"]]
        actual = truth_lookup[(unit["person_id"], instant(unit["start"]))]
        response.update(status="observed", allocations=actual["allocations"],
                        non_work_minutes=actual["non_work_minutes"], reason=None)
    (output / "responses.json").write_text(json.dumps(responses, indent=2) + "\n")
    response_paths = []
    for person in ("participant-a", "participant-b"):
        run("review", "--frame", output / "frame.json", "--sample", output / "sample.json",
            "--person", person, "--output", output / f"review-{person}.html")
        individual = deepcopy(responses)
        individual["responses"] = [response for response in responses["responses"]
                                   if unit_lookup[response["unit_id"]]["person_id"] == person]
        response_path = output / "responses" / f"{person}.json"
        response_path.parent.mkdir(exist_ok=True)
        response_path.write_text(json.dumps(individual, indent=2) + "\n")
        response_paths.extend(["--responses", response_path])

    run("proxy", "--frame", output / "frame.json", "--output",
        output / "proxy.json", "--markdown", output / "proxy.md")
    estimates = {}
    for target in ("total", "issue-alpha", "issue-beta", "unattributed"):
        run("estimate", "--frame", output / "frame.json", "--sample", output / "sample.json",
            *response_paths, "--observatory", binary,
            "--target", target, "--output", output / f"report-{target}.json",
            "--markdown", output / f"report-{target}.md")
        report = load(output / f"report-{target}.json")
        assert report["mode"] == "sample_adjusted_estimate"
        assert report["synthetic"] is True
        result = report["estimate"]["estimation"]
        assert result["observed_sample_size"] == 4
        assert result["variance"]["status"] == "estimated_srs"

        def prediction(unit):
            if target == "total":
                return unit["predicted_person_hours"]
            if target == "unattributed":
                return unit["unattributed_predicted_person_hours"]
            return unit["task_predictions"].get(target, 0)

        expected = sum(prediction(unit) for unit in frame["units"])
        for response in responses["responses"]:
            observed = sum(a["minutes"] for a in response["allocations"]
                           if target == "total" or a["task_id"] == target) / 60
            expected += 2 * (observed - prediction(unit_lookup[response["unit_id"]]))
        assert math.isclose(result["difference_total_person_hours"], expected, abs_tol=1e-12)
        estimates[target] = result["difference_total_person_hours"]

    # Complete outcomes are required; a missing check must not become zero.
    incomplete = deepcopy(responses)
    incomplete["responses"][0].update(status="nonresponse", allocations=[],
                                      non_work_minutes=None, reason="not_answered")
    (output / "responses-incomplete.json").write_text(json.dumps(incomplete, indent=2) + "\n")
    run("estimate", "--frame", output / "frame.json", "--sample", output / "sample.json",
        "--responses", output / "responses-incomplete.json", "--observatory", binary,
        "--output", output / "report-incomplete.json", "--markdown", output / "report-incomplete.md")
    unavailable = load(output / "report-incomplete.json")
    assert unavailable["mode"] == "estimate_unavailable"
    assert unavailable["estimate"] is None
    assert unavailable["sampling"]["missing_count"] == 1
    assert not (output / "report-incomplete.estimator-input.json").exists()

    invalid = deepcopy(responses)
    invalid["responses"][0]["non_work_minutes"] = 16
    (output / "responses-invalid.json").write_text(json.dumps(invalid, indent=2) + "\n")
    run("estimate", "--frame", output / "frame.json", "--sample", output / "sample.json",
        "--responses", output / "responses-invalid.json", "--observatory", binary,
        "--output", output / "must-not-exist.json", succeeds=False)
    assert not (output / "must-not-exist.json").exists()

    verification = {
        "schema": "observatory.synthetic-effort-pilot/1", "synthetic": True,
        "frame_units": 8, "sampled_units": 4, "person_windows": 2,
        "manufactured_total_minutes": sum(sum(a["minutes"] for a in row["allocations"]) for row in truth["units"]),
        "sample_adjusted_estimates_person_hours": estimates,
        "checks": ["allowlisted export", "overlapping devices count once", "quiet and missing intervals retained",
                   "saved sample precedes manufactured responses", "Rust difference estimates match independent arithmetic",
                   "person-specific responses merge", "nonresponse preserves an unavailable report",
                   "impossible allocation rejected"],
        "scope": "Software verification with manufactured observations; no empirical human-effort accuracy claim.",
    }
    (output / "verification.json").write_text(json.dumps(verification, indent=2) + "\n")
    print(json.dumps(verification, indent=2))


if __name__ == "__main__":
    main()
