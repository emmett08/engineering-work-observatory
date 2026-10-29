#!/usr/bin/env python3
"""Generate manuscript evidence tables from immutable forecast/backtest JSON.

This formats retained replay, comparison and original-command reports. It does
not fit a model, select a window, alter a score, or convert a reconstruction into
evidence of pre-outcome issuance or prospective validation.
"""
from __future__ import annotations

import argparse
from collections import defaultdict
from hashlib import sha256
import gzip
import json
import math
from pathlib import Path
import shutil


METHODS = {
    "empirical_recent": "Recent empirical",
    "empirical_expanding": "Expanding empirical",
    "zero_point_mass": "Zero",
    "previous_period_point_mass": "Previous period",
    "poisson_prior_four_period_mean": "Poisson, prior-four mean",
}


def read(path: Path) -> tuple[dict, dict]:
    raw = path.read_bytes()
    return json.loads(raw), {"file": path.name, "sha256": sha256(raw).hexdigest()}


def tex(value: object) -> str:
    replacements = {"\\": r"\textbackslash{}", "&": r"\&", "%": r"\%", "$": r"\$",
                    "#": r"\#", "_": r"\_", "{": r"\{", "}": r"\}", "~": r"\textasciitilde{}",
                    "^": r"\textasciicircum{}"}
    return "".join(replacements.get(char, char) for char in str(value))


def number(value: float | None, digits: int = 3) -> str:
    if value is None:
        return "Unavailable"
    if not math.isfinite(value):
        raise ValueError("Nonfinite result cannot be published")
    return f"{value:.{digits}f}"


def user_table(day: dict, week: dict) -> tuple[str, dict]:
    daily, weekly = day["forecast"], week["forecast"]
    history = day["history"]
    assert daily["source_snapshot_sha256"] == weekly["source_snapshot_sha256"]
    assert history["source_snapshot_sha256"] == daily["source_snapshot_sha256"]
    assert daily["model_version"] == weekly["model_version"] == "observatory.empirical-merge-count/2"
    assert daily["method"] == weekly["method"] == "expanding_empirical"
    assert day["schema"] == week["schema"] == "observatory.team-forecast-replay/1"
    assert daily["issued_at"] == history["snapshot_cut"]
    assert daily["period"] == "day" and weekly["period"] == "week"
    assert daily["coverage_complete"] and weekly["coverage_complete"]
    groups = defaultdict(list)
    for row in history["pull_requests"]:
        groups[row["repository_id"]].append(row)
    rows = []
    summary = {"source_snapshot_sha256": daily["source_snapshot_sha256"],
               "report_kind": day["schema"], "evidence_mode": daily["evidence_mode"],
               "declared_snapshot_cut": daily["issued_at"], "data_cutoff": daily["data_cutoff"],
               "repositories": [], "daily_forecast": daily, "weekly_forecast": weekly,
               "current_pressure": history["current_pressure"]}
    for repo in history["repositories"]:
        prs = groups[repo["repository_id"]]
        merged = sum(row["merged_at"] is not None for row in prs)
        rows.append(f"{tex(repo['full_name'])} & {repo['created_at'][:10]} & {len(prs)} & {merged} \\\\")
        summary["repositories"].append({"repository_id": repo["repository_id"],
                                         "full_name": repo["full_name"], "created_at": repo["created_at"],
                                         "visible_prs": len(prs), "merged_prs": merged})
    out = [r"\begin{center}", r"\begin{tabular}{@{}llrr@{}}", r"\toprule",
           r"Repository & Created (UTC) & Visible PRs & Merged PRs \\", r"\midrule", *rows,
           r"\bottomrule", r"\end{tabular}", r"\end{center}"]
    out.append("The table and distribution below are offline reconstructions from a retained visible-inventory snapshot, "
               "not evidence that a forecast was actually issued at that snapshot's declared cut. "
               "The inventory includes all visible PRs, including merges outside the fitted portfolio period. "
               f"Its conservative complete-through cutoff is {tex(daily['data_cutoff'])}. "
               f"The last-observed open inventory is {history['current_pressure']['open_pull_requests']}; "
               "this does not constrain future arrivals.")
    if daily["distribution"] is not None:
        d = daily["distribution"]
        counts = ", ".join(str(row["count"]) for row in daily["training_bins"])
        out.append(f"The expanding reference uses {len(daily['training_bins'])} complete daily counts: "
                   f"$({counts})$, ordered by date. The replayed prediction for [{daily['target_start'][:10]}, {daily['target_end'][:10]}) UTC "
                   f"has mean {number(d['mean'], 1)}, median {d['median']}, and empirical 10th--90th percentile "
                   f"range {d['quantile_10']}--{d['quantile_90']}. That inclusive range contains "
                   f"{100*d['quantile_range_mass']:.0f}\\% of the reported empirical mass, with no guaranteed future coverage. "
                   "The target is future relative to the declared snapshot cut; this replay is not a mature forecast score.")
    else:
        out.append("The daily empirical forecast is unavailable under the declared support rule.")
    out.append(f"The weekly report contains {weekly['available_complete_bins']} complete historical weekly bins "
               f"and is {tex(weekly['status'])}. "
               "These feasibility outputs establish automatic operation, not forecast accuracy.")
    return "\n".join(out) + "\n", summary


def issued_example(path: Path, day: dict) -> tuple[str, dict]:
    """Describe the separate original bundle without upgrading the replay's provenance."""
    compressed = path.read_bytes()
    raw = gzip.decompress(compressed)
    bundle = json.loads(raw)
    manifest = json.loads((path.parent / "manifest.json").read_text())
    assert sha256(compressed).hexdigest() == manifest["compressed_bundle_sha256"]
    assert sha256(raw).hexdigest() == manifest["uncompressed_bundle_sha256"]
    assert bundle["schema"] == "observatory.team-forecast-bundle/1"
    forecast = bundle["forecast"]
    assert forecast["source_snapshot_sha256"] == manifest["source_snapshot_sha256"]
    assert forecast["source_snapshot_sha256"] != day["forecast"]["source_snapshot_sha256"]
    assert bundle["created_at"] == manifest["created_at"]
    assert forecast["training_bins"] == day["forecast"]["training_bins"]
    assert forecast["distribution"] == day["forecast"]["distribution"]
    created = bundle["created_at"][:19].replace("T", " ")
    paragraph = (
        "A separate original \\texttt{forecast-team} bundle is retained in "
        r"\path{examples/issued-20261029.json.gz}. "
        f"Its locally declared creation time is {created} UTC, for the target "
        f"[{forecast['target_start'][:10]}, {forecast['target_end'][:10]}) UTC. "
        "Its source snapshot differs from the replay's, but its same five complete counts produce the same numerical distribution. "
        "The accompanying score is pending and contains no outcome or loss. This original command receipt is distinct from the "
        "offline table reconstruction; its timestamps remain declarations rather than independent trusted-time certification.\n"
    )
    return paragraph, {"file": path.name, "compressed_sha256": sha256(compressed).hexdigest(),
                        "uncompressed_sha256": sha256(raw).hexdigest(), "created_at": bundle["created_at"],
                        "source_snapshot_sha256": forecast["source_snapshot_sha256"],
                        "target_start": forecast["target_start"], "target_end": forecast["target_end"]}


def reference_table(report: dict) -> tuple[str, dict]:
    assert report["schema"] == "observatory.merge-count-backtest/1"
    assert report["period"] == "week"
    assert report["study_selection"] == "predeclared_trailing_window"
    assert report["requested_history_bins"] == 78 and report["initial_training_bins"] == 52
    if report["status"] != "available":
        return ("The predeclared public-reference comparison was unavailable; no predictive claim is inferred.\n",
                {"status": report["status"], "reasons": report["reasons"]})
    assert report["selected_history_bins"] == 78 and report["evaluation_origins"] == 26
    groups = defaultdict(list)
    for row in report["rows"]:
        assert row["method"] in METHODS
        groups[row["method"]].append(row)
    assert set(groups) == set(METHODS)
    origins = [{(row["target_start"], row["target_end"]) for row in groups[method]} for method in METHODS]
    assert all(origins[0] == dates for dates in origins) and len(origins[0]) == 26
    summaries, table_rows = {}, []
    for method, label in METHODS.items():
        rows = groups[method]
        scored = [row for row in rows if row["rps"] is not None]
        n = len(scored)
        rps = sum(row["rps"] for row in scored) / n if n else None
        mae = sum(row["median_absolute_error"] for row in scored) / n if n else None
        coverage = sum(row["within_quantile_range"] for row in scored) / n if n else None
        width = sum(row["quantile_90"] - row["quantile_10"] for row in scored) / n if n else None
        summaries[method] = {"scored_origins": n, "mean_rps": rps, "median_mae": mae,
                             "quantile_range_coverage": coverage, "mean_quantile_range_width": width}
        declared = next(row for row in report["summaries"] if row["method"] == method)
        assert declared["scored_origins"] == n
        if n:
            assert math.isclose(declared["mean_rps"], rps, rel_tol=1e-10, abs_tol=1e-10)
            assert math.isclose(declared["mean_median_absolute_error"], mae, rel_tol=1e-10, abs_tol=1e-10)
        coverage_text = f"{100*coverage:.1f}\\%" if coverage is not None else "Unavailable"
        table_rows.append(f"{tex(label)} & {n} & {number(rps)} & {number(mae)} & {number(width, 2)} & {coverage_text} \\\\")
    out = [r"\subsection{Predeclared public-reference comparison}",
           "The following values are generated from the fixed 26 chronological targets in \\texttt{cpmpy/cpmpy}. "
           "MAE uses the predictive median. The empirical percentile range is descriptive; its mean width and observed coverage are reported separately from its requested quantiles.",
           r"\begin{center}", r"\small", r"\begin{tabular}{@{}lrrrrr@{}}", r"\toprule",
           r"Method & Targets & Mean RPS & MAE & Mean width & Coverage \\", r"\midrule", *table_rows,
           r"\bottomrule", r"\end{tabular}", r"\end{center}"]
    recent, expanding = summaries["empirical_recent"], summaries["empirical_expanding"]
    if recent["scored_origins"] == expanding["scored_origins"] == 26 and expanding["mean_rps"] > 0:
        improvement = 100 * (expanding["mean_rps"] - recent["mean_rps"]) / expanding["mean_rps"]
        direction = "lower" if improvement >= 0 else "higher"
        out.append(f"The recent empirical model's mean RPS is {abs(improvement):.2f}\\% {direction} "
                   "than the expanding empirical comparator's on this fixed reference window. "
                   "This signed result is retained regardless of whether it favours the recent-window model. "
                   "No dependence-adjusted confidence interval or superiority claim is supplied, and no result is transferred to the user's repositories.")
        if improvement < 5:
            out.append("The recent-window model does not meet the predeclared 5\\% distributional-improvement screen on this reference window. "
                       "Its point-error result cannot override that primary criterion.")
        summaries["recent_vs_expanding_percent_improvement"] = improvement
    out.append("The complete report retains target dates, outcomes and paired losses for every method. "
               "All results are retrospective reconstructions from the retained visible inventory.")
    return "\n".join(out) + "\n", {"source_snapshot_sha256": report["source_snapshot_sha256"],
                                              "evaluation_origins": 26, "summaries": summaries,
                                              "first_target": min(origins[0])[0],
                                              "last_target_end": max(origins[0])[1]}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--user-day", required=True, type=Path)
    parser.add_argument("--user-week", required=True, type=Path)
    parser.add_argument("--reference", type=Path)
    parser.add_argument("--reference-snapshot", type=Path,
                        help="Full source snapshot, used to verify the reference repository and digest")
    parser.add_argument("--issued-example", type=Path,
                        default=Path(__file__).parent / "examples/issued-20261029.json.gz")
    parser.add_argument("--output", type=Path, default=Path(__file__).parent / "generated")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    day, dmeta = read(args.user_day)
    week, wmeta = read(args.user_week)
    user_tex, summary = user_table(day, week)
    (args.output / "user-results.tex").write_text(user_tex)
    evidence = {"generator": "generate_results.py", "inputs": [dmeta, wmeta], "user": summary}
    example_tex, example_evidence = issued_example(args.issued_example, day)
    (args.output / "issued-example.tex").write_text(example_tex)
    evidence["original_issued_example"] = example_evidence
    for source, dest in [(args.user_day, "user-day.json"), (args.user_week, "user-week.json")]:
        if source.resolve() != (args.output / dest).resolve():
            shutil.copyfile(source, args.output / dest)
    if args.reference:
        report, rmeta = read(args.reference)
        source_file = args.output / "reference-source.json"
        if args.reference_snapshot:
            source, smeta = read(args.reference_snapshot)
            assert source["sha256"] == report["source_snapshot_sha256"]
            repositories = []
            for retained in source["records"]:
                data = retained["record"]["data"]
                if data.get("source") == "github.repository":
                    payload = data["payload"]
                    repositories.append({"id": payload["id"], "full_name": payload["full_name"],
                                         "created_at": payload["created_at"]})
            assert {(r["id"], r["full_name"].lower()) for r in repositories} == {(100869457, "cpmpy/cpmpy")}
            source_evidence = {"snapshot_sha256": source["sha256"], "input_file": smeta,
                               "repositories": repositories, "snapshot_records": len(source["records"])}
            source_file.write_text(json.dumps(source_evidence, indent=2) + "\n")
        else:
            source_evidence = json.loads(source_file.read_text())
            assert source_evidence["snapshot_sha256"] == report["source_snapshot_sha256"]
            assert {(r["id"], r["full_name"].lower()) for r in source_evidence["repositories"]} == {(100869457, "cpmpy/cpmpy")}
        result_tex, result = reference_table(report)
        (args.output / "reference-results.tex").write_text(result_tex)
        if args.reference.resolve() != (args.output / "reference-weekly.json").resolve():
            shutil.copyfile(args.reference, args.output / "reference-weekly.json")
        evidence["inputs"].append(rmeta)
        evidence["reference"] = result
        evidence["reference_source"] = source_evidence
    elif (args.output / "reference-results.tex").exists():
        raise ValueError("Existing reference output requires an explicit --reference input")
    (args.output / "evidence-summary.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
