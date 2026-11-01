"""A fixed, auditable task-total prediction demo, separate from interval sampling.

Targets are declared complete task totals. GitHub duration is deliberately absent.
The fixed hash partition groups revisions of a task; validation tasks are consumed
once. Immutable evaluation records are the transaction log and model registry.
"""
from collections import defaultdict
from copy import deepcopy
import fcntl
import math
from pathlib import Path
import re

from effort.common import canonical_hash, fields, identifier, integer, parse_time, read_json, require, write_json


DEFAULT_POLICY = {
    "schema": "observatory.task-total-policy/1",
    "split_train_percent": 70,
    "prior_weight": 3,
    "min_train_tasks": 8,
    "min_validation_tasks": 4,
    "min_relative_mae_improvement": 0.10,
    "min_absolute_mae_improvement_minutes": 1.0,
}
MODEL_SCHEMA = "observatory.task-total-model/1"
EVALUATION_SCHEMA = "observatory.task-total-evaluation/1"
MAX_OBSERVATIONS = 20000


def _valid_minutes(value):
    try:
        return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value) and value >= 0
    except OverflowError:
        return False


def _median(values):
    ordered = sorted(values)
    middle = len(ordered)//2
    # Halve before adding: finite accepted declarations must not overflow merely
    # because an even-sized median internally adds the middle two values.
    return ordered[middle] if len(ordered) % 2 else ordered[middle-1]/2 + ordered[middle]/2


def _repository(value):
    require(isinstance(value, str) and re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", value), "invalid repository")
    return value.lower()


def task_key(protocol_id, repository, issue_number):
    """Task identity deliberately excludes revision, PR, source issue and alias."""
    return canonical_hash([identifier(protocol_id), _repository(repository), integer(issue_number, maximum=2**53-1)])


def task_partition(protocol_id, repository, issue_number):
    """Frozen v1 grouping, independent of labels and collection order."""
    key = task_key(protocol_id, repository, issue_number)
    return "train" if int(key, 16) % 100 < 70 else "validation"


def _policy(policy):
    fields(policy, set(DEFAULT_POLICY) | {"protocol_id"})
    identifier(policy["protocol_id"], "protocol_id")
    # Deliberately no tunable search space for the demo's held-out panel.
    require(all(type(policy[k]) is type(v) and policy[k] == v for k, v in DEFAULT_POLICY.items()), "unsupported task-total policy; v1 settings are fixed")
    return deepcopy(policy)


def _stable_record(record):
    # Delivery time is not observation identity and cannot cause reevaluation.
    return {key: value for key, value in record.items() if key != "observed_at"}


def _latest(revisions):
    require(isinstance(revisions, list) and len(revisions) <= MAX_OBSERVATIONS, "invalid or excessive observation revisions")
    sources = defaultdict(dict)
    seen = {}
    for record in revisions:
        require(isinstance(record, dict), "invalid observation revision")
        identifier(record.get("revision_id"), "revision_id")
        identifier(record.get("observation_id"), "observation_id")
        source = record.get("source")
        require(isinstance(source, dict), "missing observation source")
        key = (_repository(source.get("repository")), integer(source.get("issue_number"), maximum=2**53-1))
        parse_time(source.get("updated_at"))
        stable = _stable_record(record)
        revision_id = record["revision_id"]
        require(revision_id not in seen or seen[revision_id] == stable, "revision identity has conflicting contents")
        seen[revision_id] = stable
        sources[key][revision_id] = stable
    result = []
    for key, versions in sorted(sources.items()):
        newest = max(parse_time(row["source"]["updated_at"]) for row in versions.values())
        latest = sorted((row for row in versions.values() if parse_time(row["source"]["updated_at"]) == newest), key=lambda row: row["revision_id"])
        if len(latest) == 1:
            result.append(latest[0])
        else:
            # GitHub's timestamp resolution cannot order competing edits.
            row = deepcopy(latest[0])
            row.update(revision_id=canonical_hash([v["revision_id"] for v in latest]), status="conflict", observation=None,
                       errors=["conflicting revisions with the same source timestamp"])
            result.append(row)
    return result


def _units(rows, protocol_id, synthetic):
    groups = defaultdict(list)
    excluded = []
    for row in rows:
        reference = {"observation_id": row["observation_id"], "revision_id": row["revision_id"],
                     "source_repository": row["source"]["repository"], "source_issue_number": row["source"]["issue_number"]}
        observation = row.get("observation")
        if observation is not None:
            require(isinstance(observation, dict), "invalid observation payload")
            require(type(observation.get("synthetic")) is bool and observation["synthetic"] is synthetic, "live and synthetic observations must not be mixed")
            require(observation.get("protocol_id") == protocol_id, "observation protocol does not match model policy")
        if row.get("status") != "accepted" or observation is None:
            excluded.append({**reference, "reason": "latest_revision_not_accepted"})
            continue
        task = observation.get("task", {})
        require(isinstance(task, dict), "invalid observation task")
        repository = _repository(task.get("repository"))
        issue = integer(task.get("issue_number"), maximum=2**53-1)
        key = task_key(protocol_id, repository, issue)
        reasons = []
        minutes = observation.get("declared_minutes")
        if not _valid_minutes(minutes):
            reasons.append("missing_or_invalid_declared_minutes")
        if observation.get("schema") != "observatory.effort-observation/1":
            reasons.append("unsupported_observation_schema")
        if observation.get("method") not in {"timer", "recall"}:
            reasons.append("unknown_measurement_method")
        if observation.get("coverage") != "complete":
            reasons.append("incomplete_or_unknown_coverage")
        if observation.get("scope") != "task_total":
            reasons.append("not_a_complete_task_total")
        unit = {**reference, "task_key": key, "repository": repository, "issue_number": issue,
                "minutes": minutes, "method": observation.get("method"),
                "partition": task_partition(protocol_id, repository, issue)}
        groups[key].append((unit, reasons))
    eligible = []
    for key, observations in sorted(groups.items()):
        # Two observation issues can describe the same work. Never sum or average
        # their totals, even when equal; require source-level resolution instead.
        duplicate = len(observations) != 1
        for unit, reasons in observations:
            if duplicate:
                reasons = ["multiple_observation_issues_for_task", *reasons]
            if reasons:
                excluded.append({key: value for key, value in unit.items() if key != "minutes"} | {"reason": ";".join(reasons)})
            else:
                eligible.append(unit)
    return eligible, sorted(excluded, key=lambda row: (row["source_repository"], row["source_issue_number"], row["revision_id"]))


def _save_model(root, training, policy, synthetic):
    global_median = _median(row["minutes"] for row in training)
    grouped = defaultdict(list)
    for row in training:
        grouped[row["repository"]].append(row["minutes"])
    repositories = {}
    for repository, minutes in sorted(grouped.items()):
        repositories[repository] = {
            "training_tasks": len(minutes), "median_minutes": _median(minutes),
            "prediction_minutes": (len(minutes)/(len(minutes)+policy["prior_weight"]))*_median(minutes) + (policy["prior_weight"]/(len(minutes)+policy["prior_weight"]))*global_median,
        }
    body = {"schema": MODEL_SCHEMA, "protocol_id": policy["protocol_id"], "synthetic": synthetic,
            "policy_sha256": canonical_hash(policy), "method": "repository_median_shrunk_to_training_global_median",
            "global_median_minutes": global_median, "prior_weight": policy["prior_weight"],
            "repositories": repositories, "training_units": training,
            "target": "declared_complete_task_total_minutes"}
    model = {**body, "model_id": canonical_hash(body)}
    write_json(root / "models" / (model["model_id"] + ".json"), model)
    return model


def predict(active_model, repository):
    """Predict a future declared task total; None means no validated model."""
    if active_model is None:
        return None
    require(isinstance(active_model, dict) and active_model.get("schema") == MODEL_SCHEMA, "invalid active task-total model")
    repository = _repository(repository)
    prediction = active_model["repositories"].get(repository, {}).get("prediction_minutes", active_model["global_median_minutes"])
    require(_valid_minutes(prediction), "invalid model prediction")
    return prediction


def _load_model(root, model_id):
    if model_id is None:
        return None
    require(isinstance(model_id, str) and re.fullmatch(r"[0-9a-f]{64}", model_id), "invalid model identifier")
    model = read_json(root / "models" / (model_id + ".json"))
    require(model.get("model_id") == model_id and canonical_hash({k: v for k, v in model.items() if k != "model_id"}) == model_id, "model hash mismatch")
    return model


def _history(root, policy, synthetic):
    history, consumed, active_id, previous = [], set(), None, None
    for index, path in enumerate(sorted((root / "evaluations").glob("*.json")), 1):
        evaluation = read_json(path)
        digest = canonical_hash(evaluation)
        require(path.name == f"{index:08d}-{digest}.json", "evaluation ledger is incomplete or corrupt")
        require(evaluation.get("schema") == EVALUATION_SCHEMA and evaluation.get("sequence") == index and evaluation.get("previous_evaluation_sha256") == previous, "invalid evaluation chain")
        require(evaluation.get("synthetic") is synthetic and evaluation.get("policy_sha256") == canonical_hash(policy), "evaluation namespace or policy mismatch")
        panel = evaluation["consumed_validation_tasks"]
        require(isinstance(panel, list) and len(panel) == len(set(panel)) and not consumed.intersection(panel), "validation tasks were reused")
        consumed.update(panel)
        if evaluation.get("retired_model_id") is not None:
            require(evaluation["retired_model_id"] == active_id and evaluation.get("retirement_reasons"), "invalid model retirement")
            active_id = None
        if evaluation["decision"] == "promoted":
            require(evaluation["active_model_id"] == evaluation["candidate_model_id"], "invalid model promotion")
            active_id = evaluation["active_model_id"]
        else:
            require(evaluation["active_model_id"] == active_id, "rejected model changed the active registry")
        _load_model(root, evaluation["candidate_model_id"])
        panel_id = evaluation["frozen_predictions_sha256"]
        if panel_id is not None:
            frozen = read_json(root / "predictions" / (panel_id + ".json"))
            require(canonical_hash(frozen) == panel_id, "frozen prediction hash mismatch")
        history.append((evaluation, path))
        previous = digest
    return history, consumed, _load_model(root, active_id), previous


def _retirement_reasons(active, history, units):
    if active is None:
        return []
    latest = {row["task_key"]: row for row in units}
    reasons = []
    # An earlier evaluation is still auditable after correction, but it must not
    # silently authorize predictions based on withdrawn/corrected declarations.
    approved = next(evaluation for evaluation, _ in reversed(history)
                    if evaluation["decision"] == "promoted" and evaluation["active_model_id"] == active["model_id"])
    for role, labels in (("training", active["training_units"]), ("validation", approved["validation_targets"])):
        for label in labels:
            current = latest.get(label["task_key"])
            minutes = label["minutes"] if role == "training" else label["declared_minutes"]
            if current is None:
                reasons.append({"task_key": label["task_key"], "role": role, "reason": "supporting_observation_unavailable_or_ineligible"})
            elif current["minutes"] != minutes or current["observation_id"] != label["observation_id"] or current["method"] != label["method"]:
                reasons.append({"task_key": label["task_key"], "role": role, "reason": "supporting_declaration_changed"})
    return reasons


def _result(root, evaluation, path, active_model):
    # active_model_id records the historical decision. This explicit current ID
    # avoids mistaking an old replay's promotion for a model that is still active.
    return {**deepcopy(evaluation), "evaluation_id": canonical_hash(evaluation), "active_model": deepcopy(active_model),
            "current_active_model_id": active_model["model_id"] if active_model else None,
            "artifact_paths": {"evaluation": str(path),
                               "candidate_model": str(root / "models" / (evaluation["candidate_model_id"] + ".json")) if evaluation["candidate_model_id"] else None,
                               "frozen_predictions": str(root / "predictions" / (evaluation["frozen_predictions_sha256"] + ".json")) if evaluation["frozen_predictions_sha256"] else None}}


def _prepare(revisions, policy, synthetic):
    require(type(synthetic) is bool, "synthetic must be boolean")
    policy = _policy(policy)
    # Validate all rows, including superseded ones, before discarding revisions.
    require(isinstance(revisions, list), "revisions must be a list")
    for row in revisions:
        if isinstance(row, dict) and isinstance(row.get("observation"), dict):
            observation = row["observation"]
            require(type(observation.get("synthetic")) is bool and observation["synthetic"] is synthetic, "live and synthetic observations must not be mixed")
            require(observation.get("protocol_id") == policy["protocol_id"], "observation protocol does not match model policy")
    rows = _latest(revisions)
    units, exclusions = _units(rows, policy["protocol_id"], synthetic)
    return policy, rows, units, exclusions


def assess(revisions, policy, synthetic=False):
    """Read-only eligibility assessment without fitting or scoring a holdout.

    Isolated PR runs can report declarations without opening a fresh real-data
    validation history. Only trusted runs with durable state may call improve.
    """
    policy, _, units, exclusions = _prepare(revisions, policy, synthetic)
    return {
        "schema": "observatory.task-total-assessment/1", "protocol_id": policy["protocol_id"], "synthetic": synthetic,
        "policy_sha256": canonical_hash(policy), "policy": policy, "decision": "observation_only",
        "counts": {"eligible": len(units), "train": sum(row["partition"] == "train" for row in units),
                   "validation": sum(row["partition"] == "validation" for row in units), "unused_validation": None},
        "exclusions": exclusions, "metrics": None, "candidate_model_id": None, "active_model_id": None,
        "active_model": None, "current_active_model_id": None, "retired_model_id": None, "retirement_reasons": [],
        "consumed_validation_tasks": [], "frozen_predictions_sha256": None,
        "artifact_paths": {"evaluation": None, "candidate_model": None, "frozen_predictions": None},
        "interpretation": "Observation eligibility only; no fitting, holdout scoring, or model promotion. Validation availability is unknown without trusted durable state.",
    }


def improve(revisions, state, policy, synthetic=False):
    """Import one complete observation snapshot into an immutable model ledger.

    The caller must acquire a complete source snapshot first. The result reports
    exclusions and unavailable predictions rather than manufacturing zero labels.
    Thresholds are demo operation guards, not statistical accuracy guarantees.
    """
    policy, rows, units, exclusions = _prepare(revisions, policy, synthetic)
    root = Path(state) / "model-learning" / ("synthetic" if synthetic else "live")
    root.mkdir(parents=True, exist_ok=True)
    namespace = {"schema": "observatory.task-total-namespace/1", "synthetic": synthetic, "policy": policy}
    with (root / ".lock").open("a+b") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        write_json(root / "namespace.json", namespace)
        history, consumed, active, previous = _history(root, policy, synthetic)
        input_id = canonical_hash({"policy": policy, "synthetic": synthetic, "latest_revisions": rows})
        for evaluation, path in history:
            if evaluation["input_sha256"] == input_id:
                return _result(root, evaluation, path, active)
        retirement = _retirement_reasons(active, history, units)
        retired_id = active["model_id"] if retirement else None
        if retirement:
            active = None
        training = [row for row in units if row["partition"] == "train"]
        validation = [row for row in units if row["partition"] == "validation"]
        fresh = [row for row in validation if row["task_key"] not in consumed]
        evaluation = {
            "schema": EVALUATION_SCHEMA, "sequence": len(history)+1, "previous_evaluation_sha256": previous,
            "input_sha256": input_id, "protocol_id": policy["protocol_id"], "synthetic": synthetic,
            "policy_sha256": canonical_hash(policy), "policy": policy,
            "counts": {"eligible": len(units), "train": len(training), "validation": len(validation), "unused_validation": len(fresh)},
            "exclusions": exclusions, "decision": "insufficient_data", "metrics": None,
            "candidate_model_id": None, "active_model_id": active["model_id"] if active else None,
            "retired_model_id": retired_id, "retirement_reasons": retirement,
            "consumed_validation_tasks": [], "frozen_predictions_sha256": None,
            "interpretation": "Retrospective held-out comparison of declared task totals; operational thresholds do not establish prospective accuracy or causality.",
        }
        if len(training) >= policy["min_train_tasks"] and len(validation) >= policy["min_validation_tasks"]:
            if len(fresh) < policy["min_validation_tasks"]:
                evaluation["decision"] = "validation_exhausted"
            else:
                candidate = _save_model(root, training, policy, synthetic)
                baseline = candidate["global_median_minutes"]
                predictions = [{"task_key": row["task_key"], "revision_id": row["revision_id"],
                                "baseline_minutes": baseline, "candidate_minutes": predict(candidate, row["repository"]),
                                "incumbent_minutes": predict(active, row["repository"])} for row in fresh]
                frozen = {"schema": "observatory.task-total-predictions/1", "synthetic": synthetic,
                          "policy_sha256": canonical_hash(policy), "candidate_model_id": candidate["model_id"],
                          "incumbent_model_id": active["model_id"] if active else None, "predictions": predictions}
                panel_id = canonical_hash(frozen)
                # Publish predictions before reading validation labels for scoring.
                write_json(root / "predictions" / (panel_id + ".json"), frozen)
                metrics = {}
                for name in ("baseline", "candidate", "incumbent"):
                    errors = [prediction[name+"_minutes"] - row["minutes"] for prediction, row in zip(predictions, fresh)] if name != "incumbent" or active else []
                    metrics[name+"_mae"] = math.fsum(abs(error)/len(errors) for error in errors) if errors else None
                    metrics[name+"_bias"] = math.fsum(error/len(errors) for error in errors) if errors else None
                improvement = metrics["baseline_mae"] - metrics["candidate_mae"]
                relative = improvement/metrics["baseline_mae"] if metrics["baseline_mae"] > 0 else None
                metrics.update(absolute_improvement=improvement, relative_improvement=relative)
                gates = {
                    "absolute_improvement": improvement >= policy["min_absolute_mae_improvement_minutes"],
                    "relative_improvement": relative is not None and relative >= policy["min_relative_mae_improvement"],
                    "no_incumbent_regression": active is None or metrics["candidate_mae"] <= metrics["incumbent_mae"],
                }
                promote = all(gates.values())
                evaluation.update(decision="promoted" if promote else "rejected", metrics=metrics, gates=gates,
                                  candidate_model_id=candidate["model_id"], consumed_validation_tasks=[row["task_key"] for row in fresh],
                                  frozen_predictions_sha256=panel_id,
                                  validation_targets=[{"task_key": row["task_key"], "revision_id": row["revision_id"],
                                                       "observation_id": row["observation_id"], "method": row["method"],
                                                       "declared_minutes": row["minutes"]} for row in fresh])
                if promote:
                    active = candidate
                    evaluation["active_model_id"] = candidate["model_id"]
        # A single atomic immutable record both consumes the panel and promotes
        # the model. Orphan candidate/prediction files never activate a model.
        path = root / "evaluations" / (f"{evaluation['sequence']:08d}-" + canonical_hash(evaluation) + ".json")
        write_json(path, evaluation)
        return _result(root, evaluation, path, active)
