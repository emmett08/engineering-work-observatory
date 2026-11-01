"""Freeze conservative activity proxies; never create direct labour observations."""
from collections import defaultdict
import datetime as dt
import math

from .activity import CATEGORIES, _allowlist
from .common import (canonical_hash, fields, format_time, identifier, now,
                     parse_time, require, validate_config, validate_frame)

AUTOMATION = {"automation", "ai coding", "ai", "assistant", "background"}
WORK = CATEGORIES - AUTOMATION - {"unknown", "building"}


def resolve_task(data, links):
    """Use explicit URL, then repository/branch, then unique project evidence.

    Contradictory or explicitly redacted linkage never falls back to a guess.
    Multiple issues linked to one PR remain ambiguous.
    """
    if data.get("linkage_unknown"):
        return None
    candidates = []
    for field in ("issue_url", "pr_url"):
        if field in data:
            matches = set()
            for link in links:
                urls = ([f'https://github.com/{link["repository"]}/issues/{link["issue_number"]}']
                        if field == "issue_url" else
                        [f'https://github.com/{link["repository"]}/pull/{p}' for p in link["pull_requests"]] + link.get("pull_request_urls", []))
                if data[field] in urls:
                    matches.add(link["task_id"])
            candidates.append(matches)
    if "branch" in data:
        candidates.append({link["task_id"] for link in links
                           if ((data["branch"] in link["branches"]
                                and ("repository" not in data or data["repository"] == link["repository"]))
                               or any(data["branch"] == b["branch"] and data.get("repository") == b["repository"] for b in link.get("branch_links", [])))
                           and ("project" not in data or data["project"] in link["projects"])})
    elif not candidates and "project" in data:
        candidates.append({link["task_id"] for link in links
                           if data["project"] in link["projects"]
                           and ("repository" not in data or data["repository"] == link["repository"])})
    if not candidates:
        return None
    possible = set.intersection(*candidates)
    return next(iter(possible)) if len(possible) == 1 else None


def _validate_activity(config, activity):
    fields(activity, {"schema", "collected_at", "config_sha256", "sources"}, {"merged_from"})
    require(activity["schema"] == "observatory.activity-evidence/1", "unsupported activity schema")
    require(activity["config_sha256"] == canonical_hash(config), "activity belongs to another config")
    parse_time(activity["collected_at"])
    if "merged_from" in activity:
        import re
        require(isinstance(activity["merged_from"], list) and 0 < len(activity["merged_from"]) <= 1000, "invalid merged source list")
        require(all(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) for value in activity["merged_from"]), "invalid merged source hash")
    expected = {}
    windows = {f["person_id"]: (parse_time(f["start"]), parse_time(f["end"])) for f in config["frames"]}
    for source in config["sources"]:
        for bucket, kind in [(source["afk_bucket"], "afk"), *((b, "context") for b in source["context_buckets"])]:
            expected[(source["person_id"], source["device_id"], bucket)] = kind
    require(isinstance(activity["sources"], list) and len(activity["sources"]) == len(expected), "activity must report every configured source")
    allowed = _allowlist(config)
    seen, count = set(), 0
    for source in activity["sources"]:
        fields(source, {"person_id", "device_id", "bucket_id", "kind", "status", "events", "error"})
        key = (source["person_id"], source["device_id"], source["bucket_id"])
        require(key in expected and key not in seen and source["kind"] == expected[key], "unexpected or duplicate activity source")
        seen.add(key)
        require(source["status"] in {"retrieved", "missing", "partial"}, "invalid source status")
        if source["error"] is not None:
            identifier(source["error"], "source error category")
        require(isinstance(source["events"], list), "invalid source events")
        require(source["status"] != "missing" or not source["events"], "missing source cannot contain events")
        count += len(source["events"])
        require(count <= 100000, "activity exceeds 100000 events")
        ids = set()
        for event in source["events"]:
            fields(event, {"event_id", "start", "end", "data"})
            identity = identifier(event["event_id"])
            require(identity not in ids, "duplicate event ID in normalized source")
            ids.add(identity)
            start, end = parse_time(event["start"]), parse_time(event["end"])
            require(windows[source["person_id"]][0] <= start < end <= windows[source["person_id"]][1], "activity event outside declared window")
            data = event["data"]
            if source["kind"] == "afk":
                fields(data, {"status"})
                require(data["status"] in {"afk", "not-afk"}, "invalid AFK state")
            else:
                fields(data, set(), {"project", "branch", "repository", "issue_url", "pr_url", "category", "linkage_unknown"})
                for name in allowed:
                    if name in data:
                        require(data[name] in allowed[name], "context outside declared allowlist")
                if "category" in data:
                    require(data["category"] in CATEGORIES, "unknown activity category")
                if "linkage_unknown" in data:
                    require(type(data["linkage_unknown"]) is bool, "invalid linkage flag")


def _state(active, source_meta, links, waiting):
    by_device = defaultdict(lambda: {"afk": set(), "context": []})
    for (source_index, _), event in active.items():
        source = source_meta[source_index]
        device = by_device[source["device_id"]]
        if source["kind"] == "afk":
            device["afk"].add(event["data"]["status"])
        else:
            device["context"].append(event["data"])
    afk_known = context_known = paired = False
    candidates, flags = [], set()
    for device in by_device.values():
        states = device["afk"]
        afk_known |= len(states) == 1
        context_known |= bool(device["context"])
        paired |= len(states) == 1 and bool(device["context"])
        if len(states) > 1:
            flags.add("conflicting_afk")
        if states != {"not-afk"}:
            continue
        for data in device["context"]:
            category = data.get("category")
            if category in AUTOMATION or (category == "building" and not waiting):
                flags.add("automation_or_waiting_excluded_from_proxy")
                continue
            task = resolve_task(data, links)
            # Unknown foreground context is not automatically engineering work.
            if task is not None or category in WORK or (category == "building" and waiting):
                candidates.append(task)
            else:
                flags.add("unclassified_context")
    if not paired:
        flags.add("telemetry_gap")
    if not candidates:
        return False, None, afk_known, context_known, paired, flags
    tasks = set(candidates)
    if len(tasks) != 1 or None in tasks:
        flags.add("ambiguous_or_unattributed_task")
        task = None
    else:
        task = next(iter(tasks))
    return True, task, afk_known, context_known, paired, flags


def build_frame(config, activity):
    validate_config(config)
    _validate_activity(config, activity)
    frozen_at = now()
    require(parse_time(activity["collected_at"]) <= parse_time(frozen_at), "activity acquisition is in the future")
    require(all(parse_time(f["end"]) <= parse_time(frozen_at) for f in config["frames"]), "freeze only completed windows")
    units = []
    for frame in sorted(config["frames"], key=lambda f: f["person_id"]):
        start, end = parse_time(frame["start"]), parse_time(frame["end"])
        boundaries = [start]
        while boundaries[-1] < end:
            boundaries.append(min(end, boundaries[-1] + dt.timedelta(minutes=config["interval_minutes"])))
        changes = defaultdict(list)
        sources = [s for s in activity["sources"] if s["person_id"] == frame["person_id"]]
        source_flags = {"source_" + s["status"] for s in sources if s["status"] != "retrieved"}
        for index, source in enumerate(sources):
            for event in source["events"]:
                key = (index, event["event_id"])
                changes[parse_time(event["start"])].append((True, key, event))
                changes[parse_time(event["end"])].append((False, key, event))
        points = sorted(set(changes) | set(boundaries))
        active, unit_index = {}, 0
        accumulators = [{"total": 0, "tasks": defaultdict(int), "other": 0, "afk": 0, "context": 0, "unknown": 0, "flags": set(source_flags)} for _ in boundaries[:-1]]
        for a, b in zip(points, points[1:]):
            for addition, key, event in changes[a]:
                if addition:
                    active[key] = event
                else:
                    active.pop(key, None)
            require(len(active) <= 1000, "too many overlapping source events; resolve duplicated telemetry")
            while a >= boundaries[unit_index+1]:
                unit_index += 1
            value = accumulators[unit_index]
            duration = (b-a) // dt.timedelta(microseconds=1)
            predicted, task, afk, context, paired, flags = _state(active, sources, config["links"], config["attended_waiting_counts"])
            value["flags"].update(flags)
            value["afk"] += duration if afk else 0
            value["context"] += duration if context else 0
            value["unknown"] += duration if not paired else 0
            if predicted:
                value["total"] += duration
                if task is None:
                    value["other"] += duration
                else:
                    value["tasks"][task] += duration
        for index, (a, b) in enumerate(zip(boundaries, boundaries[1:])):
            value = accumulators[index]
            identity = canonical_hash({"frame_id": config["frame_id"], "person_id": frame["person_id"], "start": format_time(a), "end": format_time(b)})[:24]
            units.append({"unit_id": "interval-" + identity, "person_id": frame["person_id"], "start": format_time(a), "end": format_time(b),
                          "predicted_person_hours": value["total"]/3_600_000_000,
                          "task_predictions": {key: total/3_600_000_000 for key, total in sorted(value["tasks"].items())},
                          "unattributed_predicted_person_hours": value["other"]/3_600_000_000,
                          "coverage": {"afk_seconds": value["afk"]/1_000_000, "context_seconds": value["context"]/1_000_000, "unknown_seconds": value["unknown"]/1_000_000},
                          "flags": sorted(value["flags"])})
    result = {"schema": "observatory.effort-frame/1", "protocol_id": config["protocol_id"], "frame_id": config["frame_id"],
              "config_sha256": canonical_hash(config), "activity_sha256": canonical_hash(activity), "frozen_at": frozen_at,
              "frame_complete": config["frame_complete"], "frames": config["frames"], "links": config["links"],
              "attended_waiting_counts": config["attended_waiting_counts"],
              "synthetic": config.get("synthetic", False),
              "model": {"id": "activity-proxy-v1", "description": "Union of same-device not-AFK and recognised engineering context spans across a person's devices. Watcher spans need not prove foreground attention. Ambiguous tasks stay unattributed. Missing or unclassified spans predict zero as an auxiliary model choice, never as observed zero effort. No idle-time padding. This is an uncalibrated activity proxy."},
              "units": units}
    return validate_frame(result)
