#!/usr/bin/env python3
"""Independently verify the fixed reference study from a compact event ledger.

Uses only the Python standard library. Empirical RPS is recomputed with Fraction
and a pairwise-distance identity; Poisson RPS uses an 80-digit Decimal CDF sum.
Neither calculation imports the Rust implementation or the table generator.

The supplied ledger is derived evidence, not a substitute for the retained raw
snapshot. Its payload digest detects edits; its source file/content hashes bind
it to that snapshot but do not prove acquisition time or historical visibility.
"""
from __future__ import annotations

import argparse
from collections import Counter
from copy import deepcopy
from datetime import datetime, timedelta, timezone
from decimal import Decimal, localcontext
from fractions import Fraction
from hashlib import sha256
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY_ID = 100869457
METHODS = {
    "empirical_recent", "empirical_expanding", "zero_point_mass",
    "previous_period_point_mass", "poisson_prior_four_period_mean",
}
WEEK = timedelta(days=7)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def date(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    require(parsed.tzinfo is not None, "A source timestamp has no timezone")
    return parsed.astimezone(timezone.utc)


def read(path: Path) -> tuple[dict, str]:
    raw = path.read_bytes()
    return json.loads(raw), sha256(raw).hexdigest()


def payload_hash(ledger: dict) -> str:
    content = {key: value for key, value in ledger.items() if key != "payload_sha256"}
    return sha256(json.dumps(content, sort_keys=True, separators=(",", ":"),
                             ensure_ascii=False).encode()).hexdigest()


def derive(snapshot: dict, source_file_hash: str, report_file_hash: str) -> dict:
    """Project only identity/event fields from the matching latest acquisition."""
    require(snapshot["schema"] == "observatory.snapshot/1", "Unsupported source snapshot")
    records = [row for row in snapshot["records"] if row["record"]["kind"] == "source_record"]
    namespace = f"github:repository:{REPOSITORY_ID}"

    def family(name: str) -> list[dict]:
        return [row for row in records
                if row["record"]["data"].get("source") == name
                and row["record"]["data"].get("repository") == namespace]

    metadata = max(family("github.repository"), key=lambda row: date(row["observed_at"]))
    summary = max(family("github.throughput_collection"), key=lambda row: date(row["observed_at"]))
    require(metadata["raw_sha256"] == summary["raw_sha256"], "Mixed source acquisitions")
    repo = metadata["record"]["data"]["payload"]
    require(repo["id"] == REPOSITORY_ID and repo["full_name"].lower() == "cpmpy/cpmpy",
            "Reference repository differs from the predeclared identity")
    collection = summary["record"]["data"]["payload"]
    require(collection["profile"] == "throughput", "Wrong collection profile")
    for name in ("github.repository", "github.pull_request"):
        coverage = [row for row in collection["coverage"] if row["source"] == name]
        require(len(coverage) == 1 and coverage[0]["state"] == "complete"
                and coverage[0]["pages"] > 0, f"Incomplete {name} coverage")
    events = {}
    for row in family("github.pull_request"):
        if row["raw_sha256"] != summary["raw_sha256"]:
            continue
        source = row["record"]["data"]["payload"]
        event = {key: source[key] for key in ("id", "number", "created_at", "merged_at")}
        require(event["number"] not in events or events[event["number"]] == event,
                "Conflicting PR versions in the derived acquisition")
        events[event["number"]] = event
    coverage = next(row for row in collection["coverage"] if row["source"] == "github.pull_request")
    require(len(events) == collection.get("unique_pull_requests", coverage["records"]),
            "Derived PR inventory count does not match the complete source inventory")
    ledger = {
        "schema": "observatory.reference-event-ledger/1",
        "derivation": "Identity and event fields projected from the retained current-visible PR inventory; not raw evidence or an observed-at historical replay.",
        "source_snapshot_sha256": snapshot["sha256"],
        "source_snapshot_file_sha256": source_file_hash,
        "source_acquisition_manifest_sha256": summary["raw_sha256"],
        "reference_report_file_sha256": report_file_hash,
        "repository": {key: repo[key] for key in ("id", "full_name", "created_at")},
        "snapshot_cut": snapshot["as_of"],
        "collection_started_at": collection["started_at"],
        "collection_completed_at": collection["completed_at"],
        "events": sorted(events.values(), key=lambda event: event["number"]),
    }
    ledger["payload_sha256"] = payload_hash(ledger)
    return ledger


def empirical_score(train: list[int], outcome: int) -> Fraction:
    """E|X-y| - 1/2 E|X-X'| with exact rational frequency mass."""
    mass = {value: Fraction(count, len(train)) for value, count in Counter(train).items()}
    first = sum(probability * abs(value - outcome) for value, probability in mass.items())
    pair = sum(p * q * abs(value - other) for value, p in mass.items() for other, q in mass.items())
    return first - pair / 2


def poisson_score(mean: Fraction, outcome: int) -> Decimal:
    """Independent forward recurrence from P(0), followed by a CDF-sum score."""
    with localcontext() as context:
        context.prec = 80
        rate = Decimal(mean.numerator) / Decimal(mean.denominator)
        require(0 <= rate <= 1000, "Reference rate exceeds the oracle's declared resource bound")
        upper = max(outcome, int(rate) + int(20 * rate.sqrt()) + 100)
        probability = (-rate).exp()
        cumulative = Decimal(0)
        score = Decimal(0)
        for count in range(upper + 1):
            if count:
                probability *= rate / Decimal(count)
            cumulative += probability
            score += (cumulative - int(outcome <= count)) ** 2
        # Beyond upper >= outcome, only squared survival probabilities remain.
        # Bound their sum using the decreasing Poisson probability ratios.
        ratio = rate / Decimal(upper + 2)
        next_probability = probability * rate / Decimal(upper + 1)
        omitted_mass_bound = next_probability / (1 - ratio)
        omitted_score_bound = omitted_mass_bound ** 2 / (1 - ratio ** 2)
        require(omitted_score_bound < Decimal("1e-50"), "Poisson oracle tail bound insufficient")
        require(abs(1 - cumulative) < Decimal("1e-50"), "Poisson oracle normalization failed")
        return +score


def near(actual: float, expected: float, context: str, tolerance: float = 1e-10) -> None:
    require(isinstance(actual, (int, float)) and math.isfinite(actual)
            and math.isclose(actual, expected, rel_tol=tolerance, abs_tol=tolerance),
            f"{context}: reported {actual!r}, independently expected {expected!r}")


def verify(ledger: dict, report: dict) -> dict:
    require(ledger["schema"] == "observatory.reference-event-ledger/1", "Unsupported event ledger")
    require(ledger["payload_sha256"] == payload_hash(ledger), "Event ledger payload digest mismatch")
    require(ledger["repository"]["id"] == REPOSITORY_ID
            and ledger["repository"]["full_name"].lower() == "cpmpy/cpmpy", "Wrong reference identity")
    require(report["source_snapshot_sha256"] == ledger["source_snapshot_sha256"], "Source identity mismatch")
    require(report["schema"] == "observatory.merge-count-backtest/1"
            and report["status"] == "available" and report["period"] == "week"
            and report["evidence_mode"] == "retrospective_visible_inventory_reconstruction"
            and report["study_selection"] == "predeclared_trailing_window", "Wrong evaluation contract")
    for key, expected in (("requested_history_bins", 78), ("selected_history_bins", 78),
                          ("initial_training_bins", 52), ("evaluation_origins", 26)):
        require(report[key] == expected, f"Changed study frame: {key}")
    exposure = date(ledger["collection_started_at"])
    completed = date(ledger["collection_completed_at"])
    snapshot_cut = date(ledger["snapshot_cut"])
    require(exposure <= completed <= snapshot_cut, "Inconsistent collection clock")
    require(date(report["complete_through"]) == exposure
            and date(report["snapshot_cut"]) == snapshot_cut, "Report/source cutoff mismatch")
    end = exposure.replace(hour=0, minute=0, second=0, microsecond=0) - timedelta(days=exposure.weekday())
    start = end - 78 * WEEK
    repository_created = date(ledger["repository"]["created_at"])
    require(repository_created <= start, "Study predates repository existence")
    require(date(report["history_start"]) == repository_created, "Report history start differs from repository creation")
    require(report["available_complete_bins"] == (end - repository_created).days // 7,
            "Report count of available complete weeks differs from the source calendar")
    counts = [0] * 78
    ids, numbers = set(), set()
    for event in ledger["events"]:
        require(isinstance(event["id"], int) and event["id"] > 0 and event["id"] not in ids,
                "Invalid or duplicated stable PR ID")
        require(isinstance(event["number"], int) and event["number"] > 0 and event["number"] not in numbers,
                "Invalid or duplicated PR number")
        ids.add(event["id"])
        numbers.add(event["number"])
        created = date(event["created_at"])
        require(date(ledger["repository"]["created_at"]) <= created <= completed, "Invalid PR creation date")
        if event["merged_at"] is None:
            continue
        merged = date(event["merged_at"])
        require(created <= merged <= completed, "Merge predates creation or follows collection")
        if start <= merged < end:
            counts[(merged - start).days // 7] += 1
    rows = {}
    for row in report["rows"]:
        key = row["method"], date(row["target_start"])
        require(key not in rows and row["method"] in METHODS, "Duplicate or unknown method/origin row")
        rows[key] = row
    require(len(rows) == 130, "Expected exactly 26 origins for five fixed methods")
    losses = {method: [] for method in METHODS}
    for index in range(52, 78):
        origin = start + index * WEEK
        histories = {
            "empirical_recent": counts[index - 8:index],
            "empirical_expanding": counts[:index],
            "zero_point_mass": [0],
            "previous_period_point_mass": [counts[index - 1]],
            "poisson_prior_four_period_mean": counts[index - 4:index],
        }
        for method, train in histories.items():
            row = rows.get((method, origin))
            require(row is not None, f"Missing {method} at {origin}")
            require(date(row["target_end"]) == origin + WEEK and row["outcome"] == counts[index]
                    and row["training_bins"] == len(train) and row["unavailable_reason"] is None,
                    f"Chronology, label or training mismatch: {method} at {origin}")
            expected = (poisson_score(Fraction(sum(train), 4), counts[index])
                        if method == "poisson_prior_four_period_mean"
                        else empirical_score(train, counts[index]))
            near(row["rps"], float(expected), f"RPS {method} at {origin}")
            near(row["mean"], sum(train) / len(train), f"Predictive mean {method} at {origin}")
            losses[method].append(float(expected))
    require(len(report["summaries"]) == 5, "Unexpected summary count")
    require({row["method"] for row in report["summaries"]} == METHODS, "Wrong summary methods")
    means = {method: math.fsum(values) / 26 for method, values in losses.items()}
    for summary in report["summaries"]:
        require(summary["scored_origins"] == 26, "Unequal scoring support")
        near(summary["mean_rps"], means[summary["method"]], "Aggregate RPS")
    return {"verified_method_origin_scores": len(rows), "visible_prs": len(ids),
            "frame_start": start.isoformat(), "frame_end": end.isoformat(),
            "initial_training_weeks": 52, "evaluation_weeks": 26, "mean_rps": means,
            "recent_relative_rps_increase": means["empirical_recent"] / means["empirical_expanding"] - 1,
            "interpretation": "Score verification is not prospective validation, calibrated coverage, or evidence of transfer to user repositories."}


def corruption_checks(ledger: dict, report: dict) -> None:
    for mutation in ("loss", "outcome", "duplicate", "frame", "source"):
        changed = deepcopy(report)
        if mutation == "loss":
            changed["rows"][0]["rps"] += 1
        elif mutation == "outcome":
            changed["rows"][0]["outcome"] += 1
        elif mutation == "duplicate":
            changed["rows"].append(deepcopy(changed["rows"][0]))
        elif mutation == "frame":
            changed["initial_training_bins"] = 51
        else:
            changed["source_snapshot_sha256"] = "0" * 64
        try:
            verify(ledger, changed)
        except (ValueError, KeyError):
            continue
        raise ValueError(f"The independent verifier accepted {mutation} corruption")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", type=Path, default=HERE / "generated/reference-events.json")
    parser.add_argument("--report", type=Path, default=HERE / "generated/reference-weekly.json")
    parser.add_argument("--snapshot", type=Path, help="Re-derive the compact ledger from the retained raw snapshot")
    parser.add_argument("--write-ledger", action="store_true", help="Write a newly derived ledger; never replace different retained bytes")
    parser.add_argument("--self-test", action="store_true", help="Also reject five independently corrupted report copies")
    args = parser.parse_args()
    report, report_hash = read(args.report)
    if args.snapshot:
        snapshot, snapshot_hash = read(args.snapshot)
        ledger = derive(snapshot, snapshot_hash, report_hash)
        if args.write_ledger:
            raw = (json.dumps(ledger, indent=2, sort_keys=True) + "\n").encode()
            require(not args.ledger.exists() or args.ledger.read_bytes() == raw,
                    "Refusing to replace different retained ledger bytes; choose a new --ledger path")
            args.ledger.parent.mkdir(parents=True, exist_ok=True)
            args.ledger.write_bytes(raw)
    else:
        require(not args.write_ledger, "--write-ledger requires --snapshot")
        ledger, _ = read(args.ledger)
    require(ledger["reference_report_file_sha256"] == report_hash, "Reference report file hash changed")
    result = verify(ledger, report)
    if args.self_test:
        corruption_checks(ledger, report)
        result["corruption_checks_passed"] = 5
    result["source_snapshot_sha256"] = ledger["source_snapshot_sha256"]
    result["source_snapshot_file_sha256"] = ledger["source_snapshot_file_sha256"]
    result["ledger_payload_sha256"] = ledger["payload_sha256"]
    retained = args.ledger.exists() and (not args.snapshot or args.write_ledger)
    result["ledger_file_sha256"] = sha256(args.ledger.read_bytes()).hexdigest() if retained else None
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
