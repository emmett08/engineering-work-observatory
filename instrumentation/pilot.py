#!/usr/bin/env python3
"""Join selected execution evidence and optional private sampled-effort batches."""
import argparse
from contextlib import contextmanager
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import re
import sqlite3
import subprocess
import sys
import tempfile

from effort.batch import prepare_batch
from effort.common import (canonical_hash, fields, identifier, integer, read_json,
                           require, validate_frame, write_json, write_text)
from effort.report import build_report
from effort.sampling import response_template

REPOSITORY = r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+"
LIMITATIONS = [
    "Tool runtime is elapsed execution time, not human labour. Parallel runtimes may overlap.",
    "Receipt hashes and schema validation establish integrity, not truthful controller, session or task declarations.",
    "Missing receipts can mean uninstrumented work or failed delivery; they do not establish zero tool use.",
    "Issue/PR links do not establish acceptance, retained quality or a causal AI effect.",
    "Sampled responses are human declarations. Sampling standard errors exclude recall, attribution, roster and model-drift errors; they are not accuracy guarantees.",
]


def validate_manifest(value):
    fields(value, {"schema", "protocol_id", "tasks"})
    require(value["schema"] == "observatory.linked-pilot/1", "unsupported pilot manifest")
    identifier(value["protocol_id"])
    require(isinstance(value["tasks"], list) and 0 < len(value["tasks"]) <= 100, "declare 1–100 tasks")
    tasks, issues = set(), set()
    for task in value["tasks"]:
        fields(task, {"task_id", "repository", "issue_number", "pull_requests", "run_ids"})
        identifier(task["task_id"])
        require(task["task_id"] not in tasks and task["task_id"] not in {"total", "unattributed"}, "duplicate or reserved task")
        tasks.add(task["task_id"])
        require(isinstance(task["repository"], str) and re.fullmatch(REPOSITORY, task["repository"]), "invalid repository")
        integer(task["issue_number"], maximum=2**53-1)
        issue = (task["repository"].lower(), task["issue_number"])
        require(issue not in issues, "issue has multiple tasks")
        issues.add(issue)
        for key in ("pull_requests", "run_ids"):
            require(isinstance(task[key], list) and len(task[key]) <= 1000, "invalid pilot ID list")
            for identity in task[key]:
                integer(identity, maximum=2**53-1)
            require(len(task[key]) == len(set(task[key])), "duplicate pilot ID")
    return value


@contextmanager
def state_lock(state, create=False):
    state = Path(state)
    if create:
        state.mkdir(parents=True, exist_ok=True, mode=0o700)
    require(state.is_dir(), "collector state directory is unavailable")
    with (state / ".pilot.lock").open("a") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        yield


def _run_private(command, log):
    """Keep collector errors and paths on the private host, outside public output."""
    log.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    with log.open("xb") as handle:
        return subprocess.run(command, stdout=handle, stderr=subprocess.STDOUT, timeout=1800, check=False).returncode


def collect(manifest, state, observatory, bundles=()):
    validate_manifest(manifest)
    selected = sorted({(task["repository"], run) for task in manifest["tasks"] for run in task["run_ids"]})
    require(selected or bundles, "select run IDs or explicit bundles before collection")
    state, binary = Path(state).resolve(), str(Path(observatory).resolve())
    with state_lock(state, create=True):
        log_dir = Path(tempfile.mkdtemp(prefix="collection-", dir=state))
        outcomes = []
        for index, (repository, run) in enumerate(selected):
            code = _run_private([binary, "--state", str(state), "collect-github-executions", "--repository", repository, "--run-id", str(run)], log_dir / f"run-{index}.log")
            outcomes.append({"repository": repository, "run_id": run, "returncode": code})
        for index, bundle in enumerate(bundles):
            code = _run_private([binary, "--state", str(state), "import-executions", str(Path(bundle).resolve())], log_dir / f"import-{index}.log")
            outcomes.append({"bundle_sha256": hashlib.sha256(Path(bundle).read_bytes()).hexdigest(), "returncode": code})
        verified = _run_private([binary, "--state", str(state), "verify"], log_dir / "verify.log") == 0
        write_json(log_dir / "collection.json", {"schema": "observatory.pilot-collection/1", "manifest_sha256": canonical_hash(manifest), "outcomes": outcomes, "store_verified": verified})
        require(verified, "collector store verification failed; inspect private collection logs")
        return all(item["returncode"] == 0 for item in outcomes)


def read_records(state):
    """Caller must run Rust verify first and serialize access to this store."""
    database = Path(state).resolve() / "observatory.sqlite3"
    require(database.is_file(), "collector store is unavailable")
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True) as connection:
        rows = connection.execute("SELECT r.payload, MAX(o.observed_at) FROM records r JOIN observations o ON r.id=o.record_id WHERE r.kind='source_record' GROUP BY r.id ORDER BY MAX(o.observed_at), r.id")
        result = []
        for raw, observed in rows:
            require(len(raw) <= 16*1024*1024, "stored record exceeds limit")
            value = json.loads(raw)
            result.append({"record": value, "observed_at": observed})
            require(len(result) <= 250000, "pilot store exceeds record limit; use a bounded study store")
        return result


def _latest(records, source, namespace):
    found = {}
    for entry in records:
        record = entry["record"]
        data = record["data"]
        if data.get("source") == source and data.get("repository") == namespace:
            found[data["source_id"]] = data["payload"]
    return list(found.values())


def execution_summary(manifest, task, records):
    """Join only exact declared links and selected runs; independently compare API metadata."""
    repo, runs = task["repository"], set(task["run_ids"])
    repository_snapshots = [entry["record"]["data"]["payload"] for entry in records
                            if entry["record"]["data"].get("source") == "github.repository"
                            and entry["record"]["data"]["payload"].get("full_name", "").lower() == repo.lower()]
    repository_id = str(repository_snapshots[-1]["id"]) if repository_snapshots else None
    namespace = "github:repository:" + repository_id if repository_id else None
    attempts = {(v.get("id"), v.get("run_attempt")): v for v in _latest(records, "github.workflow_attempt", namespace)} if namespace else {}
    jobs = [v for v in _latest(records, "github.workflow_job", namespace) if v.get("run_id") in runs] if namespace else []
    run_metadata = {v.get("id"): v for v in _latest(records, "github.workflow_run", namespace)} if namespace else {}
    deliveries = _latest(records, "github.execution_artifact_delivery", namespace) if namespace else []
    delivered_ids = {identity for value in deliveries if value.get("run_id") in runs and value.get("repository", "").lower() == repo.lower() for identity in value.get("record_ids", [])}
    selected, unlinked = {}, set()
    for entry in records:
        record, data = entry["record"], entry["record"]["data"]
        if data.get("source") != "observatory.tool_receipt":
            continue
        receipt = data["payload"]
        platform, declared = receipt["platform"], receipt["declared"]
        if platform["repository"].lower() != repo.lower() or platform["run_id"] not in {str(run) for run in runs}:
            continue
        scope = record["id"].rsplit("/", 1)[0]
        if declared["protocol_id"] != manifest["protocol_id"] or declared["task_id"] != task["task_id"]:
            unlinked.add(scope)
            continue
        selected.setdefault(scope, {})[receipt["phase"]] = (record["id"], receipt)
    invocations, matched_job_ids = [], set()
    for scope, phases in sorted(selected.items()):
        receipt = next(iter(phases.values()))[1]
        p, declared = receipt["platform"], receipt["declared"]
        run, attempt = int(p["run_id"]), p["run_attempt"]
        metadata = attempts.get((run, attempt))
        comparisons, conflicts = [], []
        if repository_id is not None and p["repository_id"] != "unknown":
            (comparisons if p["repository_id"] == repository_id else conflicts).append("repository_id")
        if metadata:
            for name, observed in (("actor", metadata.get("actor", {}).get("login")), ("triggering_actor", metadata.get("triggering_actor", {}).get("login")), ("event_name", metadata.get("event"))):
                if observed is not None and p[name] != "unknown":
                    (comparisons if p[name] == observed else conflicts).append(name)
        candidates = [job for job in jobs if job.get("run_id") == run and job.get("run_attempt") == attempt and
                      (str(job.get("id")) == p["job_id"] if p["job_id"] is not None else p["job_name"] != "unknown" and job.get("name") == p["job_name"])]
        job = candidates[0] if len(candidates) == 1 and not conflicts and metadata and repository_id == p["repository_id"] else None
        if job:
            matched_job_ids.add(job["id"])
        completion = phases.get("completion", (None, None))[1]
        result = completion["result"] if completion else None
        invocations.append({
            "invocation_id": receipt["invocation_id"], "run_id": run, "run_attempt": attempt,
            "job_id": job["id"] if job else None, "job_name": p["job_name"],
            "receipt_ids": sorted(value[0] for value in phases.values()),
            "lifecycle": "start_and_completion_observed" if len(phases) == 2 else "start_only" if "start" in phases else "completion_only",
            "delivery": "github_artifact_collector" if all(value[0] in delivered_ids for value in phases.values()) else "imported_without_artifact_delivery_comparison",
            "platform_comparison": "conflict" if conflicts else "compared_with_retained_api" if job else "unresolved",
            "compared_fields": sorted(comparisons), "conflicting_fields": sorted(conflicts),
            "actor": p["actor"], "triggering_actor": p["triggering_actor"], "event_name": p["event_name"],
            "declared_controller": declared["controller"], "declared_session_present": declared["session_id"] is not None,
            "tool": receipt["tool"]["name"], "result": result["status"] if result else None,
            "exit_code": result["exit_code"] if result else None, "runtime_ms": result["runtime_ms"] if result else None,
        })
    return {"selected_runs": [{"run_id": run, "url": f"https://github.com/{repo}/actions/runs/{run}",
                               "metadata_observed": run in run_metadata, "status": run_metadata.get(run, {}).get("status"),
                               "conclusion": run_metadata.get(run, {}).get("conclusion")} for run in sorted(runs)],
            "invocation_count": len(invocations), "invocations": invocations,
            "unlinked_invocations_in_selected_runs": len(unlinked),
            "completed_invocation_count": sum(v["lifecycle"] == "start_and_completion_observed" for v in invocations),
            "reported_runtime_seconds": math.fsum(v["runtime_ms"] / 1000 for v in invocations if v["runtime_ms"] is not None) if any(v["runtime_ms"] is not None for v in invocations) else None,
            "jobs": [{"run_id": v["run_id"], "run_attempt": v["run_attempt"], "job_id": v["id"],
                      "status": v.get("status"), "conclusion": v.get("conclusion"),
                      "selected_task_receipts": "matched" if v["id"] in matched_job_ids else "none_or_unresolved"} for v in sorted(jobs, key=lambda j: (j["run_id"], j["run_attempt"], j["id"]))],
            "jobs_without_matched_task_receipts": sum(v["id"] not in matched_job_ids for v in jobs),
            "receipt_integrity": "verified_store", "repository_metadata_observed": repository_id is not None}


def validate_frame_links(manifest, frame):
    validate_frame(frame)
    require(frame["protocol_id"] == manifest["protocol_id"], "effort frame protocol differs from pilot")
    links = {link["task_id"]: link for link in frame["links"]}
    for task in manifest["tasks"]:
        link = links.get(task["task_id"])
        require(link is not None and link["repository"].lower() == task["repository"].lower() and link["issue_number"] == task["issue_number"], "effort task/issue differs from pilot")
        require(set(task["pull_requests"]) <= set(link["pull_requests"]), "pilot PR is not retained in effort links")


def effort_summary(frame, sample, responses, task, observatory, private_input):
    if frame is None:
        return {"mode": "estimate_unavailable", "synthetic": False, "estimate_person_hours": None,
                "unavailable_reasons": ["private_activity_and_sampled_responses_not_configured"], "proxy_hours": None, "sampling": None}
    report = build_report(frame, task["task_id"], sample=sample, response_documents=responses or ([response_template(frame, sample)] if sample else []),
                          observatory=observatory, input_path=private_input)
    estimation = report["estimate"]["estimation"] if report["estimate"] else None
    sampling = report["sampling"]
    return {"mode": report["mode"], "synthetic": report["synthetic"], "frame_sha256": report["frame_sha256"],
            "estimate_person_hours": estimation["difference_total_person_hours"] if estimation else None,
            "sampling_standard_error_person_hours": estimation["variance"].get("difference_standard_error_person_hours") if estimation else None,
            "sampling_only_estimate_person_hours": estimation["ht_total_person_hours"] if estimation else None,
            "unavailable_reasons": report.get("unavailable_reasons", ["sample_not_available"] if not estimation else []),
            "proxy_hours": report["proxy"]["predicted_person_hours"],
            "unclassified_window_hours": report["proxy"]["coverage"]["unknown_seconds"] / 3600,
            "declared_frame_complete": report["declared_frame_complete"],
            "declared_person_hours_capacity": report["proxy"]["declared_person_hours_capacity"],
            "estimate_outside_declared_capacity": bool(estimation and not 0 <= estimation["difference_total_person_hours"] <= report["proxy"]["declared_person_hours_capacity"]),
            "sampling": {key: sampling[key] for key in ("population_size", "sample_size", "observed_count", "missing_count", "inclusion_probability")} if sampling else None}


def _escape(value):
    return re.sub(r"([\\`*_{}\[\]()#+.!|<>@])", r"\\\1", str(value))


def markdown(report):
    execution, effort = report["execution"], report["human_effort"]
    lines = ["### Observatory linked pilot", "", "Protocol: " + _escape(report["protocol_id"]) + ". Task: " + _escape(report["task_id"]) + ".", ""]
    if effort["synthetic"]:
        lines += ["**SYNTHETIC DEMONSTRATION: these effort observations are manufactured.**", ""]
    missing_runs = sum(not run["metadata_observed"] for run in execution["selected_runs"])
    lines.append(f"Selected runs: {len(execution['selected_runs'])}; runs without retained API metadata: {missing_runs}.")
    lines.append("")
    for run in execution["selected_runs"]:
        lines.append(f"- Run [{run['run_id']}]({run['url']}): status {_escape(run['status'] or 'unknown')}; conclusion {_escape(run['conclusion'] or 'unknown')}.")
    lines.append("")
    lines.append(f"Recorded tool invocations: {execution['invocation_count']}; complete start/completion pairs: {execution['completed_invocation_count']}.")
    runtime = execution["reported_runtime_seconds"]
    lines.append(f"Sum of reported tool runtimes: {runtime:.3f} seconds (parallel executions may overlap)." if runtime is not None else "Tool runtime: unavailable.")
    lines.append(f"Observed jobs without a matched receipt for this task: {execution['jobs_without_matched_task_receipts']} (includes other checks and possible delivery gaps).")
    conflicts = sum(v["platform_comparison"] == "conflict" for v in execution["invocations"])
    unresolved = sum(v["platform_comparison"] == "unresolved" for v in execution["invocations"])
    lines.append(f"Receipt platform comparisons: {conflicts} conflicting; {unresolved} unresolved.")
    imported = sum(v["delivery"] != "github_artifact_collector" for v in execution["invocations"])
    lines.append(f"Invocations imported without a retained GitHub artifact-delivery comparison: {imported}.")
    lines += ["", "| Run | Attempt | Job | Tool | Outcome | GitHub actor | Rerun actor | Declared controller |", "| --- | --- | --- | --- | --- | --- | --- | --- |"]
    for value in execution["invocations"]:
        lines.append("| " + " | ".join(_escape(value[key] if value[key] is not None else "unknown") for key in ("run_id", "run_attempt", "job_name", "tool", "result", "actor", "triggering_actor", "declared_controller")) + " |")
    lines += ["", "**Human-effort estimate: " + (f"{effort['estimate_person_hours']:.3f} person-hours.**" if effort["estimate_person_hours"] is not None else "unavailable.**")]
    if effort["estimate_person_hours"] is None:
        lines.append("Reasons: " + ", ".join(_escape(v) for v in effort["unavailable_reasons"]) + ".")
    else:
        se = effort["sampling_standard_error_person_hours"]
        lines.append(f"Sampling standard error: {se:.3f} person-hours." if se is not None else "Sampling standard error: unavailable.")
    if effort.get("estimate_outside_declared_capacity"):
        lines.append("**The realised estimate falls outside declared physical capacity. It is retained without clipping and needs investigation.**")
    if effort["proxy_hours"] is not None:
        lines.append(f"Automatic activity proxy: {effort['proxy_hours']:.3f} hours; this is not observed human effort.")
    if effort["sampling"]:
        sample = effort["sampling"]
        lines.append(f"Observed sampled responses: {sample['observed_count']}/{sample['sample_size']}.")
    lines += ["", "Work: " + report["issue_url"] + ".", "PRs: " + (", ".join(report["pull_request_urls"]) or "none registered") + ".", "Runs: " + (", ".join(v["url"] for v in execution["selected_runs"]) or "none registered") + ".", ""]
    lines += ["- " + limitation for limitation in report["limitations"]]
    lines += ["", "This aggregate omits private activity, participant identities and response contents.", "", f"Manifest evidence: `{report['manifest_sha256']}`.", ""]
    return "\n".join(lines)


def _gh(arguments, payload=None):
    command = ["gh", "api", *arguments]
    if payload is not None:
        command += ["--input", "-"]
    completed = subprocess.run(command, input=json.dumps(payload) if payload is not None else None, text=True, capture_output=True, check=False, timeout=120)
    require(completed.returncode == 0, "GitHub publication failed; check private host authorization")
    require(len(completed.stdout) <= 16*1024*1024, "GitHub response exceeds limit")
    return json.loads(completed.stdout)


def publish(report):
    require(not report["human_effort"]["synthetic"], "synthetic effort cannot be published as a live pilot report")
    repo = report["repository"]
    marker = "<!-- observatory-linked-pilot:" + canonical_hash({"protocol": report["protocol_id"], "task": report["task_id"]}) + " -->"
    body = marker + "\n" + markdown(report)
    require(len(body.encode("utf-8")) <= 60000, "aggregate comment exceeds publication limit; narrow selected runs")
    login = _gh(["user"])["login"]
    for number in [report["issue_number"], *report["pull_requests"]]:
        comments, page = [], 1
        while True:
            batch = _gh([f"repos/{repo}/issues/{number}/comments?per_page=100&page={page}"])
            require(isinstance(batch, list), "invalid GitHub comments response")
            comments.extend(batch)
            if len(batch) < 100:
                break
            page += 1
            require(page <= 100, "comment pagination exceeds bound")
        own = [v for v in comments if v.get("user", {}).get("login") == login and v.get("body", "").startswith(marker)]
        require(len(own) <= 1, "multiple aggregate comments require manual reconciliation")
        if own and own[0]["body"] == body:
            continue
        endpoint = f"repos/{repo}/issues/comments/{own[0]['id']}" if own else f"repos/{repo}/issues/{number}/comments"
        _gh(["--method", "PATCH" if own else "POST", endpoint], {"body": body})


def report(manifest, state, output, observatory, batch=None, publish_report=False):
    validate_manifest(manifest)
    output = Path(output)
    frame = sample = None
    responses = []
    if batch is not None:
        batch = Path(batch)
        frame, sample = read_json(batch / "frame.json"), read_json(batch / "sample.json")
        validate_frame_links(manifest, frame)
        responses = [read_json(path) for path in sorted((batch / "responses").glob("*.json"))]
    with state_lock(state):
        with tempfile.TemporaryDirectory(prefix="pilot-verify-", dir=state) as scratch:
            verified = _run_private([str(Path(observatory).resolve()), "--state", str(Path(state).resolve()), "verify"], Path(scratch) / "verify.log") == 0
            require(verified, "collector store verification failed")
        records = read_records(state)
    reports, index = [], []
    for task in manifest["tasks"]:
        name = "task-" + hashlib.sha256(task["task_id"].encode()).hexdigest()[:16]
        human = effort_summary(frame, sample, responses, task, observatory,
                               batch / "estimator-inputs" / (name + ".json") if batch else None)
        value = {"schema": "observatory.linked-pilot-report/1", "manifest_sha256": canonical_hash(manifest),
                 "protocol_id": manifest["protocol_id"], "task_id": task["task_id"],
                 "repository": task["repository"], "issue_number": task["issue_number"], "pull_requests": task["pull_requests"],
                 "issue_url": f"https://github.com/{task['repository']}/issues/{task['issue_number']}",
                 "pull_request_urls": [f"https://github.com/{task['repository']}/pull/{p}" for p in task["pull_requests"]],
                 "execution": execution_summary(manifest, task, records), "human_effort": human, "limitations": LIMITATIONS}
        write_json(output / (name + ".json"), value)
        write_text(output / (name + ".md"), markdown(value))
        reports.append(value)
        index.append({"task_id": task["task_id"], "json": name + ".json", "markdown": name + ".md", "sha256": canonical_hash(value)})
    write_json(output / "report-index.json", {"schema": "observatory.pilot-report-index/1", "reports": index})
    if publish_report:
        require(not any(v["human_effort"]["synthetic"] for v in reports), "synthetic effort cannot be published as live pilot evidence")
        for value in reports:
            publish(value)
    return reports


def parser():
    cli = argparse.ArgumentParser(description=__doc__)
    sub = cli.add_subparsers(dest="command", required=True)
    collect_parser = sub.add_parser("collect", help="Serially collect selected runs and retain validated receipts")
    collect_parser.add_argument("--manifest", type=Path, required=True)
    collect_parser.add_argument("--state", type=Path, required=True)
    collect_parser.add_argument("--observatory", type=Path, required=True)
    collect_parser.add_argument("--import-bundle", type=Path, action="append", default=[])
    prepare = sub.add_parser("prepare", help="Freeze private activity and prepare sampled review forms")
    prepare.add_argument("--config", type=Path, required=True)
    prepare.add_argument("--day", required=True)
    prepare.add_argument("--size", type=int, required=True)
    prepare.add_argument("--directory", type=Path, required=True)
    source = prepare.add_mutually_exclusive_group()
    source.add_argument("--export", type=Path)
    source.add_argument("--server", default="http://127.0.0.1:5600")
    reporting = sub.add_parser("report", help="Join selected executions to optional private sampled effort")
    reporting.add_argument("--manifest", type=Path, required=True)
    reporting.add_argument("--state", type=Path, required=True)
    reporting.add_argument("--output-dir", type=Path, required=True)
    reporting.add_argument("--observatory", type=Path, required=True)
    reporting.add_argument("--batch", type=Path)
    reporting.add_argument("--publish", action="store_true")
    return cli


def main(argv=None):
    os.umask(0o077)
    args = parser().parse_args(argv)
    if args.command == "collect":
        complete = collect(read_json(args.manifest), args.state, args.observatory, args.import_bundle)
        print("Collection finished; private evidence retained." if complete else "Collection incomplete; inspect private logs. Retained evidence remains usable with explicit gaps.")
        return 0 if complete else 1
    if args.command == "prepare":
        prepare_batch(read_json(args.config), args.directory, args.day, args.size, args.server,
                      read_json(args.export) if args.export else None)
        print("Private review forms prepared; distribute each participant's own form and retain returned responses privately.")
    elif args.command == "report":
        reports = report(read_json(args.manifest), args.state, args.output_dir, args.observatory, args.batch, args.publish)
        print(f"Prepared {len(reports)} task aggregates; human estimates remain unavailable when observations are missing.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError, TypeError, OverflowError, subprocess.SubprocessError, sqlite3.Error) as error:
        print("pilot: " + str(error), file=sys.stderr)
        sys.exit(2)
