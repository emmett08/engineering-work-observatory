"""Bounded JSON, immutable artifacts and declared-frame validation."""
import datetime as dt
import hashlib
import json
import math
import os
from pathlib import Path
import re
import tempfile

MAX_BYTES = 16 * 1024 * 1024
MAX_UNITS = 20000
ROLES = {"specification", "implementation", "review", "testing", "coordination", "repair", "other"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fields(value, required, optional=()):
    require(isinstance(value, dict), "expected an object")
    require(set(required) <= value.keys(), "missing fields: " + str(sorted(set(required) - value.keys())))
    require(value.keys() <= set(required) | set(optional), "unknown fields: " + str(sorted(value.keys() - set(required) - set(optional))))


def identifier(value, label="identifier"):
    require(isinstance(value, str) and 0 < len(value) <= 256 and not any(ord(c) < 32 for c in value), "invalid " + label)
    return value


def finite(value, label="number"):
    require(isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value), "invalid " + label)
    return value


def integer(value, minimum=1, maximum=MAX_UNITS):
    require(type(value) is int and minimum <= value <= maximum, "integer outside supported range")
    return value


def parse_time(value):
    require(isinstance(value, str) and len(value) <= 40, "invalid timestamp")
    require(re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,6})?(?:Z|[+-]\d{2}:\d{2})", value), "RFC3339 timestamp requires seconds and at most microsecond precision")
    try:
        result = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise ValueError("invalid timestamp") from error
    require(result.tzinfo is not None and "T" in value, "timestamps require an explicit timezone")
    return result.astimezone(dt.timezone.utc)


def format_time(value):
    return value.astimezone(dt.timezone.utc).isoformat(timespec="microseconds").replace("+00:00", "Z")


def now():
    return format_time(dt.datetime.now(dt.timezone.utc))


def canonical_bytes(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False).encode()


def canonical_hash(value):
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def _pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def read_json(path):
    with Path(path).open("rb") as handle:
        raw = handle.read(MAX_BYTES + 1)
    require(len(raw) <= MAX_BYTES, "JSON exceeds 16 MiB")
    def invalid_constant(_):
        raise ValueError("non-finite JSON number")
    try:
        value = json.loads(raw, object_pairs_hook=_pairs, parse_constant=invalid_constant)
        canonical_bytes(value)  # Reject exponent overflow too.
        return value
    except (UnicodeError, RecursionError) as error:
        raise ValueError("invalid JSON encoding or nesting") from error


def write_text(path, text):
    """Publish atomically without replacing an existing immutable artifact."""
    path = Path(path)
    raw = text.encode("utf-8")
    require(len(raw) <= MAX_BYTES, "output exceeds 16 MiB")
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        require(path.read_bytes() == raw, "refusing to overwrite immutable artifact: " + str(path))
        return
    name = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as handle:
            name = handle.name
            handle.write(raw)
            handle.flush()
            os.fsync(handle.fileno())
        try:
            os.link(name, path)
        except FileExistsError:
            require(path.read_bytes() == raw, "conflicting concurrent artifact publication")
    finally:
        if name is not None:
            Path(name).unlink(missing_ok=True)


def write_json(path, value):
    write_text(path, json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")


def validate_windows(frames):
    require(isinstance(frames, list) and 0 < len(frames) <= 1000, "frames must contain 1..1000 people")
    people = set()
    for frame in frames:
        fields(frame, {"person_id", "start", "end"})
        person = identifier(frame["person_id"], "person_id")
        require(person not in people, "duplicate person window")
        people.add(person)
        start, end = parse_time(frame["start"]), parse_time(frame["end"])
        require(start < end and (end-start).total_seconds() <= 60*86400, "window must be positive and at most 60 days")
    return people


def validate_links(links):
    require(isinstance(links, list) and len(links) <= 1000, "too many links")
    tasks, issues = set(), set()
    for link in links:
        fields(link, {"task_id", "repository", "issue_number", "branches", "pull_requests", "projects"}, {"pull_request_urls", "branch_links"})
        task = identifier(link["task_id"], "task_id")
        require(task not in {"total", "unattributed"} and task not in tasks, "duplicate or reserved task_id")
        tasks.add(task)
        require(isinstance(link["repository"], str) and re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", link["repository"]), "invalid repository")
        issue = integer(link["issue_number"], maximum=2**53-1)
        key = (link["repository"].lower(), issue)
        require(key not in issues, "issue assigned more than one task_id")
        issues.add(key)
        for name in ("branches", "projects", "pull_requests"):
            require(isinstance(link[name], list) and len(link[name]) <= 1000, "invalid link list")
            for item in link[name]:
                integer(item, maximum=2**53-1) if name == "pull_requests" else identifier(item)
            require(len(set(link[name])) == len(link[name]), "duplicate link entry")
        urls = link.get("pull_request_urls", [])
        require(isinstance(urls, list) and len(urls) <= 1000, "invalid PR URL list")
        require(all(isinstance(url, str) and re.fullmatch(r"https://github.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+/pull/[1-9][0-9]*", url) for url in urls), "invalid canonical PR URL")
        require(len(set(urls)) == len(urls), "duplicate PR URL")
        branches = link.get("branch_links", [])
        require(isinstance(branches, list) and len(branches) <= 1000, "invalid branch links")
        branch_keys = set()
        for branch in branches:
            fields(branch, {"repository", "branch"})
            require(isinstance(branch["repository"], str) and re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", branch["repository"]), "invalid branch repository")
            identifier(branch["branch"])
            key = (branch["repository"], branch["branch"])
            require(key not in branch_keys, "duplicate branch link")
            branch_keys.add(key)
    return tasks


def validate_config(config):
    fields(config, {"schema", "protocol_id", "frame_id", "frames", "interval_minutes", "sources", "links", "frame_complete", "attended_waiting_counts"}, {"synthetic"})
    require(config["schema"] == "observatory.effort-config/1", "unsupported effort config schema")
    identifier(config["protocol_id"])
    identifier(config["frame_id"])
    people = validate_windows(config["frames"])
    validate_links(config["links"])
    integer(config["interval_minutes"], maximum=1440)
    require(type(config["frame_complete"]) is bool and type(config["attended_waiting_counts"]) is bool, "coverage and waiting convention must be booleans")
    require(type(config.get("synthetic", False)) is bool, "synthetic must be boolean")
    require(isinstance(config["sources"], list) and len(config["sources"]) <= 1000, "invalid sources")
    devices, buckets = set(), set()
    for source in config["sources"]:
        fields(source, {"person_id", "device_id", "afk_bucket", "context_buckets"})
        require(source["person_id"] in people, "source person not in frame")
        device = identifier(source["device_id"])
        require(device not in devices, "device must have one declared owner in this batch")
        devices.add(device)
        identifier(source["afk_bucket"])
        require(isinstance(source["context_buckets"], list) and 0 < len(source["context_buckets"]) <= 100, "declare context buckets")
        for bucket in source["context_buckets"]:
            identifier(bucket)
        require(len(set(source["context_buckets"])) == len(source["context_buckets"]) and source["afk_bucket"] not in source["context_buckets"], "duplicate source bucket")
        for bucket in [source["afk_bucket"], *source["context_buckets"]]:
            require(bucket not in buckets, "one ActivityWatch bucket cannot be attributed to multiple devices or people")
            buckets.add(bucket)
    count = sum(math.ceil((parse_time(f["end"])-parse_time(f["start"])).total_seconds()/(config["interval_minutes"]*60)) for f in config["frames"])
    require(count <= MAX_UNITS, "frame exceeds 20000 intervals")
    return config


def validate_frame(frame):
    fields(frame, {"schema", "protocol_id", "frame_id", "config_sha256", "activity_sha256", "frozen_at", "frame_complete", "frames", "links", "model", "units", "attended_waiting_counts"}, {"synthetic"})
    require(frame["schema"] == "observatory.effort-frame/1", "unsupported effort frame schema")
    identifier(frame["protocol_id"])
    identifier(frame["frame_id"])
    people = validate_windows(frame["frames"])
    tasks = validate_links(frame["links"])
    for name in ("config_sha256", "activity_sha256"):
        require(isinstance(frame[name], str) and re.fullmatch(r"[0-9a-f]{64}", frame[name]), "invalid source digest")
    frozen = parse_time(frame["frozen_at"])
    require(all(parse_time(f["end"]) <= frozen for f in frame["frames"]), "freeze only completed observation windows")
    require(type(frame["frame_complete"]) is bool and type(frame["attended_waiting_counts"]) is bool, "invalid frame declarations")
    require(type(frame.get("synthetic", False)) is bool, "synthetic must be boolean")
    fields(frame["model"], {"id", "description"})
    require(frame["model"]["id"] == "activity-proxy-v1", "unsupported prediction model")
    require(isinstance(frame["model"]["description"], str) and len(frame["model"]["description"]) <= 4000, "invalid model description")
    units = frame["units"]
    require(isinstance(units, list) and 0 < len(units) <= MAX_UNITS, "invalid frame size")
    ids, by_person = set(), {person: [] for person in people}
    for unit in units:
        fields(unit, {"unit_id", "person_id", "start", "end", "predicted_person_hours", "task_predictions", "unattributed_predicted_person_hours", "coverage", "flags"})
        identity = identifier(unit["unit_id"])
        require(identity not in ids, "duplicate unit_id")
        ids.add(identity)
        require(unit["person_id"] in people, "unknown unit person")
        start, end = parse_time(unit["start"]), parse_time(unit["end"])
        require(start < end, "nonpositive unit")
        capacity = (end-start).total_seconds()/3600
        total = finite(unit["predicted_person_hours"])
        require(0 <= total <= capacity + 1e-12, "proxy exceeds person capacity")
        predictions = unit["task_predictions"]
        require(isinstance(predictions, dict) and predictions.keys() <= tasks, "unknown task prediction")
        require(all(finite(v) >= 0 for v in predictions.values()), "negative task prediction")
        other = finite(unit["unattributed_predicted_person_hours"])
        require(other >= 0 and abs(math.fsum(predictions.values()) + other - total) <= 1e-12, "task predictions must conserve person total")
        fields(unit["coverage"], {"afk_seconds", "context_seconds", "unknown_seconds"})
        require(all(0 <= finite(v) <= capacity*3600+1e-9 for v in unit["coverage"].values()), "invalid coverage duration")
        require(isinstance(unit["flags"], list) and len(unit["flags"]) <= 100, "invalid flags")
        for flag in unit["flags"]:
            identifier(flag)
        by_person[unit["person_id"]].append((start, end))
    for person in frame["frames"]:
        cursor = parse_time(person["start"])
        for start, end in sorted(by_person[person["person_id"]]):
            require(start == cursor, "units must partition person window without overlaps or gaps")
            cursor = end
        require(cursor == parse_time(person["end"]), "units do not cover declared window")
    return frame
