#!/usr/bin/env python3
"""Public GitHub-hosted observation import and guarded task-total learning demo."""
import argparse
from collections import Counter
from pathlib import Path
import os
import sqlite3
import subprocess
import sys

import pilot
from effort.common import canonical_hash, now, read_json, require, write_json, write_text
from observations import fixtures, importer, model


def validate_links(config, manifest):
    importer.validate_config(config)
    pilot.validate_manifest(manifest)
    require(config["protocol_id"] == manifest["protocol_id"], "execution and observation protocols differ")
    registered = {(t["repository"].lower(), t["issue_number"]): t for t in config["task_links"]}
    for task in manifest["tasks"]:
        link = registered.get((task["repository"].lower(), task["issue_number"]))
        require(link is not None and task["task_id"] == link["task_id"] and
                set(task["pull_requests"]) == set(link["pull_requests"]),
                "execution task/issue/PR link differs from observation configuration")


def collect_executions(manifest, state, output, observatory, coverage):
    """Never fetch executions from a repository not freshly verified as public."""
    verified = {s["repository"].lower() for s in coverage["sources"] if s["public_verified"]}
    required = {t["repository"].lower() for t in manifest["tasks"]}
    if not required <= verified:
        return {"complete": False, "reason": "repository_public_status_not_verified", "tasks": []}
    execution_state = state / "executions"
    complete = pilot.collect(manifest, execution_state, observatory)
    tasks = pilot.report(manifest, execution_state, output / "execution", observatory)
    # Operational stdout/stderr is not a designated public artifact. Keep only
    # validated evidence and structured collection outcomes in the checkpoint.
    for path in execution_state.glob("collection-*/*.log"):
        path.unlink()
    return {"complete": complete, "reason": None if complete else "receipt_collection_incomplete",
            "tasks": tasks}


def observation_comparisons(rows, evaluation, acquisition_complete):
    excluded = {row["revision_id"]: row["reason"] for row in evaluation.get("exclusions", [])}
    active = evaluation.get("active_model") if acquisition_complete else None
    result = []
    for row in rows:
        observation = row.get("observation")
        task = observation.get("task") if observation else None
        partition = model.task_partition(observation["protocol_id"], task["repository"], task["issue_number"]) if task else None
        reason = excluded.get(row["revision_id"])
        prediction = model.predict(active, task["repository"]) if task else None
        result.append({"observation_id": row["observation_id"], "revision_id": row["revision_id"],
                       "source": row["source"], "status": row["status"], "task": task,
                       "declared_minutes": observation["declared_minutes"] if observation else None,
                       "method": observation["method"] if observation else None,
                       "coverage": observation["coverage"] if observation else None,
                       "scope": observation["scope"] if observation else None,
                       "source_currentness": "acquired_this_run" if acquisition_complete else "unconfirmed_incomplete_acquisition",
                       "model_partition": partition, "model_exclusion": reason,
                       "active_model_prediction_minutes": prediction,
                       "comparison": "unavailable" if prediction is None else
                           "not_comparable_ineligible_observation" if reason else
                           "retrospective_in_sample" if partition == "train" else "retrospective_validation_not_future_accuracy",
                       "errors": row["errors"]})
    return result


def markdown(report):
    lines = ["# Public observation-learning demo", "",
             "Declared human effort, tool execution time and model predictions are separate quantities.", "",
             f"Observation acquisition: **{'complete' if report['acquisition']['complete'] else 'incomplete; live learning paused'}**. "
             f"Latest source observations: {len(report['observations'])}; "
             f"retained revisions: {report['retained_revision_count']}.", "",
             "| Observation | Issue state | Declared human minutes | Method / coverage | Model prediction | Use in model |",
             "| --- | --- | ---: | --- | ---: | --- |"]
    for row in report["observations"]:
        source = row["source"]
        label = f"[{source['repository']}#{source['issue_number']}]({source['url']})"
        minutes = row["declared_minutes"] if row["declared_minutes"] is not None else "unknown"
        prediction = row["active_model_prediction_minutes"]
        use = row["model_exclusion"] or (
            f"eligible; {row['model_partition']} partition; {row['comparison']}"
            if row["status"] == "accepted" and report["acquisition"]["complete"] else row["source_currentness"])
        lines.append(f"| {label} | {source['state']} | {minutes} | {row['method'] or 'unknown'} / {row['coverage'] or 'unknown'} | "
                     f"{f'{prediction:.2f}' if prediction is not None else 'unavailable'} | {use} |")
    lines += ["", f"Real-data model decision: **{report['live_model']['decision']}**. "
              "No sampled human-effort estimate is available: no workstation activity or sampled responses were supplied.", "",
              "## Linked execution evidence", "",
              "| Task | PRs | Tool invocations with both records | Reported tool seconds | Human minutes inferred from runtime |",
              "| --- | --- | ---: | ---: | --- |"]
    for task in report["executions"]["tasks"]:
        execution = task["execution"]
        runtime = execution["reported_runtime_seconds"]
        links = ", ".join(f"[#{n}](https://github.com/{task['repository']}/pull/{n})" for n in task["pull_requests"])
        lines.append(f"| [{task['task_id']}]({task['issue_url']}) | {links} | {execution['completed_invocation_count']} | "
                     f"{f'{runtime:.3f}' if runtime is not None else 'unknown'} | unavailable |")
    if report["live_model"].get("retired_model_id"):
        lines += ["", "The previously active model was retired because supporting declarations changed or became ineligible. "
                  "See the JSON evaluation's retirement reasons; a fresh validation panel is required."]
    lines += ["", f"Execution acquisition complete: **{str(report['executions']['complete']).lower()}**. "
              "Parallel runtimes can overlap; missing receipts are not zero activity.", "",
              "## Synthetic software demonstration — not human evidence", "",
              "The fabricated fixture deliberately gives two repositories different task totals. "
              "It tests model fitting, validation and promotion, not real predictive accuracy.", ""]
    synthetic = report["synthetic_demo"]
    evaluation = synthetic["evaluation"]
    lines += [f"Decision: **{evaluation['decision']}**. Training tasks: {evaluation['counts']['train']}; "
              f"validation tasks: {evaluation['counts']['validation']}. Identical replay preserved evaluation identity: "
              f"**{str(synthetic['replay_idempotent']).lower()}**.", ""]
    metrics = evaluation.get("metrics")
    if metrics:
        lines += [f"Held-out mean absolute error: pooled-median baseline **{metrics['baseline_mae']:.2f} min**; "
                  f"candidate **{metrics['candidate_mae']:.2f} min** (fabricated data only).", ""]
    for prediction in synthetic["future_predictions"]:
        value = prediction["prediction_minutes"]
        lines.append(f"- Fabricated future task `{prediction['repository']}#{prediction['issue_number']}`: "
                     f"{f'{value:.2f} min' if value is not None else 'unavailable'}; no outcome exists, so prediction error is unknown.")
    lines += ["", "## Interpretation and limits", ""]
    lines += ["- " + text for text in report["limitations"]]
    return "\n".join(lines) + "\n"


def run(config, manifest, state, output, observatory, enable_live_learning=False):
    validate_links(config, manifest)
    state, output = Path(state), Path(output)
    with pilot.state_lock(state, create=True):
        coverage = importer.collect(config, state)
        rows = importer.latest_observations(state)
        public_repositories = {item["repository"].lower() for item in coverage["sources"] if item["public_verified"]}
        configured = {item["repository"].lower() for item in config["sources"]}
        require(public_repositories == configured and
                all(row["source"]["repository"].lower() in public_repositories for row in rows),
                "public source verification failed or retained source was removed; no public report/checkpoint may be published")
        policy = {**model.DEFAULT_POLICY, "protocol_id": config["protocol_id"]}
        if coverage["complete"] and enable_live_learning:
            live = model.improve(rows, state, policy, synthetic=False)
        elif coverage["complete"]:
            live = model.assess(rows, policy, synthetic=False)
        else:
            live = {"decision": "acquisition_incomplete", "active_model": None, "exclusions": [],
                    "reason": "Current source coverage must be complete before evaluating or using the live model."}
        executions = collect_executions(manifest, state, output, observatory, coverage)
        synthetic_rows = fixtures.synthetic_revisions()
        synthetic_policy = {**model.DEFAULT_POLICY, "protocol_id": fixtures.PROTOCOL}
        evaluation = model.improve(synthetic_rows, state, synthetic_policy, synthetic=True)
        replay = model.improve(synthetic_rows, state, synthetic_policy, synthetic=True)
        require(evaluation == replay, "identical model delivery changed its evaluation")
        predictions = [{"repository": repo, "issue_number": 10001,
                        "prediction_minutes": model.predict(evaluation.get("active_model"), repo),
                        "observed_minutes": None, "synthetic": True}
                       for repo in ("synthetic/short", "synthetic/long")]
        report = {
            "schema": "observatory.observation-learning-report/1", "generated_at": now(),
            "config_sha256": canonical_hash(config), "execution_manifest_sha256": canonical_hash(manifest),
            "protocol_id": config["protocol_id"], "acquisition": coverage,
            "live_learning_enabled": enable_live_learning,
            "retained_revision_count": len(list((state / "observations" / "revisions").glob("*/*.json"))),
            "source_states": dict(Counter(row["source"]["state"] for row in rows)),
            "observations": observation_comparisons(rows, live, coverage["complete"]),
            "live_model": live, "executions": executions,
            "synthetic_demo": {"synthetic": True, "fixture_sha256": canonical_hash(synthetic_rows),
                               "fixture_rows": len(synthetic_rows), "evaluation": evaluation,
                               "replay_idempotent": True, "future_predictions": predictions},
            "limitations": [
                "Declared effort is a human report, not certified actual labour. Source accounts do not establish worker identity.",
                "Unknown method, incomplete coverage, partial totals and conflicting task claims are excluded from training; they remain visible.",
                "Only configured public sources and registered task links are imported. Open and closed issues are both queried.",
                "Observed revisions are retained; edits between polls cannot be reconstructed. Missing acquisition pauses live prediction and promotion.",
                "The fixed task-total predictor uses declared totals and repository groups, never workflow duration. It does not replace the interval-sampling estimator.",
                "Task-group holdout is retrospective. Training fit and reused validation are not evidence of future accuracy; consumed validation tasks cannot approve another candidate.",
                "The 8-training/4-validation minima and error-reduction gates are demonstration policies, not precision or statistical-power guarantees.",
                "Synthetic promotion proves software operation only. No real-data improvement or causal AI productivity effect is established.",
                "GitHub artifact checkpoints expire after 30 days. Missing or corrupt prior trusted state stops automatic continuation; it does not silently reset history.",
            ],
        }
        write_json(output / "report.json", report)
        write_json(output / "synthetic-fixture.json", {"synthetic": True, "rows": synthetic_rows})
        write_text(output / "report.md", markdown(report))
    return report


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    command = sub.add_parser("run")
    for option in ("config", "manifest", "state", "output", "observatory"):
        command.add_argument("--" + option, type=Path, required=True)
    command.add_argument("--enable-live-learning", action="store_true",
                         help="Enable live fitting only with continuously trusted state; isolated previews observe only")
    args = parser.parse_args()
    report = run(read_json(args.config), read_json(args.manifest), args.state, args.output,
                 args.observatory, args.enable_live_learning)
    print(f"Public observations: {len(report['observations'])}; live decision: {report['live_model']['decision']}; "
          f"synthetic decision: {report['synthetic_demo']['evaluation']['decision']}. See report.md for coverage.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError, TypeError, OverflowError, subprocess.SubprocessError, sqlite3.Error) as error:
        print("observation-learning: " + str(error), file=sys.stderr)
        sys.exit(2)
