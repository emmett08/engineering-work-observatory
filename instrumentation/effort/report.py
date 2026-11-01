"""Keep proxy activity, sampled estimates and unsupported claims distinguishable."""
import math
from pathlib import Path
import re
import subprocess

from .common import canonical_hash, parse_time, require, validate_frame, write_json
from .sampling import export_estimator, merge_responses, validate_sample

LIMITATIONS = [
    "Activity is an auxiliary prediction, not a direct human-effort observation. Quiet, unclassified or missing telemetry may predict zero without establishing zero work.",
    "The declared people and observation windows define the target. Work outside them is not estimated. Source-event span coverage is not evidence that every device or work activity was captured.",
    "Issue/PR association supplies task linkage, not acceptance, retained quality, labour savings or a causal AI effect.",
    "Sampled interval responses are human declarations; accuracy and recall require independent checking. Selected missing responses are not replaced by zero or predictions.",
    "Sampling standard errors exclude roster, observation, attribution and model-drift errors. No confidence interval or validated predictive accuracy is claimed.",
    "Immutable artifacts and a saved draw receipt support replay but cannot prove honest sampling, no alternate draws, or that predictions were genuinely frozen before sample selection.",
]


def _prediction(unit, target):
    if target == "total":
        return unit["predicted_person_hours"]
    if target == "unattributed":
        return unit["unattributed_predicted_person_hours"]
    return unit["task_predictions"].get(target, 0)


def build_report(frame, target="total", sample=None, response_documents=None, observatory=None, input_path=None):
    validate_frame(frame)
    require(target in {"total", "unattributed", *(link["task_id"] for link in frame["links"])}, "unknown estimate target")
    coverage = {name: math.fsum(unit["coverage"][name] for unit in frame["units"]) for name in ("afk_seconds", "context_seconds", "unknown_seconds")}
    flags = sorted({flag for unit in frame["units"] for flag in unit["flags"]})
    result = {
        "schema": "observatory.effort-report/1", "mode": "activity_proxy", "frame_id": frame["frame_id"],
        "frame_sha256": canonical_hash(frame), "target": target, "synthetic": frame.get("synthetic", False),
        "model": frame["model"], "declared_frame_complete": frame["frame_complete"],
        "proxy": {"predicted_person_hours": math.fsum(_prediction(u, target) for u in frame["units"]),
                  "unattributed_predicted_person_hours": math.fsum(u["unattributed_predicted_person_hours"] for u in frame["units"]),
                  "declared_person_hours_capacity": math.fsum((parse_time(f["end"])-parse_time(f["start"])).total_seconds()/3600 for f in frame["frames"]),
                  "coverage": coverage, "flags": flags},
        "links": frame["links"] if target in {"total", "unattributed"} else [l for l in frame["links"] if l["task_id"] == target],
        "sampling": None, "estimate": None, "limitations": list(LIMITATIONS),
    }
    if sample is None:
        return result
    validate_sample(frame, sample)
    responses = merge_responses(frame, sample, response_documents or [])
    by_id = {r["unit_id"]: r for r in responses["responses"]}
    unanswered = [identity for identity in sample["selected_unit_ids"] if identity not in by_id or by_id[identity]["status"] != "observed"]
    result["synthetic"] |= sample["synthetic"]
    result["sampling"] = {"sample_sha256": canonical_hash(sample), "responses_sha256": canonical_hash(responses),
                          "population_size": len(frame["units"]), "sample_size": len(sample["selected_unit_ids"]),
                          "observed_count": len(sample["selected_unit_ids"])-len(unanswered), "missing_count": len(unanswered),
                          "inclusion_probability": sample["inclusion_probability"]}
    if unanswered or not frame["frame_complete"]:
        result["mode"] = "estimate_unavailable"
        result["unavailable_reasons"] = (["sample_nonresponse"] if unanswered else []) + (["declared_frame_incomplete"] if not frame["frame_complete"] else [])
        return result
    require(observatory is not None and input_path is not None, "estimation requires an Observatory binary and retained input path")
    estimator_input = export_estimator(frame, sample, responses, target)
    write_json(input_path, estimator_input)
    completed = subprocess.run([str(Path(observatory).resolve()), "estimate-labour", str(Path(input_path).resolve())],
                               capture_output=True, text=True, timeout=120, check=False)
    require(completed.returncode == 0, "Observatory estimator failed; inspect the retained estimator input with estimate-labour")
    require(len(completed.stdout) <= 16*1024*1024, "estimator output exceeds limit")
    import json
    value = json.loads(completed.stdout)
    require(value.get("schema") == "observatory.labour-estimate/2", "unexpected estimator output schema")
    require(value.get("estimation", {}).get("frame_id") == estimator_input["estimation"]["frame_id"], "estimator returned another frame")
    result["mode"] = "sample_adjusted_estimate"
    result["estimator_input_sha256"] = canonical_hash(estimator_input)
    result["estimate"] = value
    return result


def _escape(value):
    # GitHub Markdown should not interpret caller IDs as HTML, mentions or links.
    return re.sub(r"([\\`*_{}\[\]()#+.!|<>@])", r"\\\1", str(value))


def markdown(report):
    marker = "SYNTHETIC DEMONSTRATION — " if report["synthetic"] else ""
    lines = [f"### {marker}Effort evidence", "", f"Target: {_escape(report['target'])}. Frame: {_escape(report['frame_id'])}.", ""]
    proxy = report["proxy"]
    if report["mode"] == "sample_adjusted_estimate":
        estimate = report["estimate"]["estimation"]
        lines.append(f"**Sample-adjusted estimate: {estimate['difference_total_person_hours']:.3f} person-hours.**")
        variance = estimate["variance"]
        se = variance.get("difference_standard_error_person_hours")
        lines.append(f"Sampling standard error: {se:.3f} person-hours." if se is not None else "Sampling standard error: unavailable.")
        lines += ["This is not a confidence interval or a claim of validated prediction accuracy.", "",
                  f"Sampling-only comparator: {estimate['ht_total_person_hours']:.3f} person-hours."]
        if variance.get("difference_variance_person_hours_squared") is not None and variance.get("ht_variance_person_hours_squared") is not None:
            lines.append("Estimated sampling variance (adjusted / sampling-only): "
                         f"{variance['difference_variance_person_hours_squared']:.6g} / {variance['ht_variance_person_hours_squared']:.6g} squared person-hours. One sample does not establish which method is better.")
        if estimate["difference_total_person_hours"] < 0 or estimate["difference_total_person_hours"] > proxy["declared_person_hours_capacity"]:
            lines.append("The realised statistical estimate falls outside declared physical capacity. It is retained without clipping and needs investigation.")
    elif report["mode"] == "estimate_unavailable":
        lines += ["**Human-effort estimate: unavailable.**", "Reasons: " + ", ".join(_escape(x) for x in report["unavailable_reasons"]) + "."]
    else:
        lines.append("**Human-effort estimate: unavailable; activity proxy only.**")
    lines += ["", f"Automatic activity proxy: {proxy['predicted_person_hours']:.3f} hours (not observed person-hours).",
              f"Unattributed activity proxy across the frame: {proxy['unattributed_predicted_person_hours']:.3f} hours.",
              f"Declared frame capacity: {proxy['declared_person_hours_capacity']:.3f} person-hours.",
              f"Without paired AFK/context event coverage: {proxy['coverage']['unknown_seconds']/3600:.3f} person-hours of the declared window."]
    if proxy["flags"]:
        lines.append("Evidence flags: " + ", ".join(_escape(flag) for flag in proxy["flags"]) + ".")
    if report["sampling"]:
        sample = report["sampling"]
        lines.append(f"Sample responses: {sample['observed_count']}/{sample['sample_size']}; population: {sample['population_size']} intervals.")
    if report["links"]:
        lines += ["", "Work links:"]
        for link in report["links"]:
            url = f'https://github.com/{link["repository"]}/issues/{link["issue_number"]}'
            prs = [f'https://github.com/{link["repository"]}/pull/{p}' for p in link["pull_requests"]] + link.get("pull_request_urls", [])
            lines.append(f"- [{_escape(link['task_id'])}]({url})" + ("; PRs: " + ", ".join(f"[#{u.rsplit('/',1)[1]}]({u})" for u in sorted(set(prs))) if prs else ""))
    lines += ["", "Limitations:"] + ["- " + limitation for limitation in report["limitations"]]
    lines += ["", f"Frozen evidence: `{report['frame_sha256']}`.", ""]
    return "\n".join(lines)
