"""Read ActivityWatch evidence without retaining unrestricted desktop metadata.

The official API exposes ``start``, ``end`` and ``limit`` but no offset. Full
responses are retried with a larger bounded limit over the same time window.
This avoids skipping tied timestamps and avoids treating the server's clipped
event spans as changed content. Repeated events are deduplicated.

Timestamps and numeric durations are normalised to Python's microsecond
precision. These are activity observations, not exact human-allocation data.

API contract: https://docs.activitywatch.net/en/latest/api/python.html
"""

from __future__ import annotations

import datetime as dt
import copy
import ipaddress
import json
import math
import re
import urllib.error
import urllib.parse
import urllib.request

from .common import canonical_hash, format_time, parse_time, validate_config


PAGE_SIZE = 1000
MAX_REQUESTS = 256
MAX_EVENTS = 20000
MAX_RESPONSE_BYTES = 4 * 1024 * 1024
MAX_DURATION_SECONDS = 366 * 24 * 60 * 60
TIMEOUT_SECONDS = 10
CATEGORIES = frozenset({
    "coding", "implementation", "review", "code reviewing", "debugging",
    "testing", "documentation", "reading", "research", "meeting",
    "coordination", "specification", "repair", "building", "automation",
    "ai coding", "ai", "assistant", "background", "unknown",
})


def _now():
    return format_time(dt.datetime.now(dt.timezone.utc))


def _string(value):
    return isinstance(value, str) and 0 < len(value) <= 512


def _allowlist(config):
    links = config["links"]
    branch_links = [entry for link in links for entry in link.get("branch_links", [])]
    external_prs = {url for link in links for url in link.get("pull_request_urls", [])}
    return {
        "project": {x for link in links for x in link["projects"]},
        "branch": {x for link in links for x in link["branches"]} | {x["branch"] for x in branch_links},
        "repository": ({link["repository"] for link in links}
                       | {x["repository"] for x in branch_links}
                       | {"/".join(urllib.parse.urlsplit(url).path.split("/")[1:3]) for url in external_prs}),
        "issue_url": {
            f'https://github.com/{link["repository"]}/issues/{link["issue_number"]}'
            for link in links
        },
        "pr_url": external_prs | {
            f'https://github.com/{link["repository"]}/pull/{number}'
            for link in links for number in link["pull_requests"]
        },
    }


def _github_url(value):
    """Strip query, fragment and subview only from an exact GitHub issue/PR URL."""
    if not _string(value):
        return None
    try:
        parts = urllib.parse.urlsplit(value)
        if (parts.scheme != "https" or parts.netloc != "github.com"
                or parts.username or parts.password):
            return None
    except ValueError:
        return None
    match = re.fullmatch(
        r"/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/(issues|pull)/([1-9][0-9]*)(?:/(?:files|commits|checks))?/?",
        parts.path,
    )
    if not match:
        return None
    owner, repo, kind, number = match.groups()
    return ("issue_url" if kind == "issues" else "pr_url",
            f"https://github.com/{owner}/{repo}/{kind}/{number}")


def _context(data, allowed):
    safe = {}
    unknown = False
    for field in ("project", "branch", "repository"):
        if field not in data or data[field] in (None, ""):
            continue
        value = data[field]
        if _string(value) and value in allowed[field]:
            safe[field] = value
        else:
            unknown = True
    for field in ("url", "issue_url", "pr_url"):
        if field not in data or data[field] in (None, ""):
            continue
        parsed = _github_url(data[field])
        if parsed and parsed[1] in allowed[parsed[0]]:
            target, canonical = parsed
            if target in safe and safe[target] != canonical:
                unknown = True
                safe.pop(target, None)
            else:
                safe[target] = canonical
        else:
            unknown = True
    category = data.get("category")
    if isinstance(category, str) and category in CATEGORIES:
        safe["category"] = category
    elif category not in (None, ""):
        # An unrecognised activity class must not be silently treated as coding.
        unknown = True
    if unknown:
        safe["linkage_unknown"] = True
    return safe


def _normalise_event(raw, kind, start, end, allowed):
    if not isinstance(raw, dict) or not isinstance(raw.get("data"), dict):
        raise ValueError("malformed_event")
    timestamp = parse_time(raw.get("timestamp"))
    duration = raw.get("duration")
    if (isinstance(duration, bool) or not isinstance(duration, (int, float))
            or not math.isfinite(duration) or duration < 0 or duration > MAX_DURATION_SECONDS):
        raise ValueError("malformed_event")
    try:
        finish = timestamp + dt.timedelta(seconds=duration)
    except (OverflowError, ValueError):
        raise ValueError("malformed_event") from None
    # A single unextended heartbeat carries no duration. Keep no artificial tail.
    if finish <= start or timestamp >= end or finish <= timestamp:
        return None
    if kind == "afk":
        status = raw["data"].get("status")
        if status not in {"afk", "not-afk"}:
            raise ValueError("malformed_event")
        data = {"status": status}
    else:
        data = _context(raw["data"], allowed)
    full = {"start": format_time(timestamp), "end": format_time(finish), "data": data}
    identity = raw.get("id")
    if identity is not None:
        if isinstance(identity, bool) or not isinstance(identity, int) or identity < 0:
            raise ValueError("malformed_event")
        event_id = f"aw-{identity}"
    else:
        # ActivityWatch's exported buckets omit database IDs. This stable
        # digest uses retained fields only; it does not retain hidden content.
        event_id = "content-" + canonical_hash(full)
    event = {"event_id": event_id, "start": format_time(max(timestamp, start)),
             "end": format_time(min(finish, end)), "data": data}
    return event, full


def _source_spec(config):
    frames = {item["person_id"]: item for item in config["frames"]}
    for source in config["sources"]:
        frame = frames[source["person_id"]]
        start, end = parse_time(frame["start"]), parse_time(frame["end"])
        for bucket, kind in [(source["afk_bucket"], "afk"), *(
                (bucket, "context") for bucket in source["context_buckets"])]:
            yield {"person_id": source["person_id"], "device_id": source["device_id"],
                   "bucket_id": bucket, "kind": kind, "status": "retrieved",
                   "events": [], "error": None}, start, end


def _set_error(source, code):
    source["status"] = "partial"
    # Error details and URLs can contain private metadata. Preserve categories.
    if source["error"] is None:
        source["error"] = code


def _append_events(source, raw_events, start, end, allowed, seen):
    if not isinstance(raw_events, list):
        _set_error(source, "malformed_response")
        return
    for raw in raw_events:
        if len(seen) >= MAX_EVENTS:
            _set_error(source, "event_limit")
            return
        try:
            normal = _normalise_event(raw, source["kind"], start, end, allowed)
        except (ValueError, TypeError, KeyError, OverflowError):
            _set_error(source, "malformed_event")
            continue
        if normal is None:
            continue
        event, full = normal
        key = event["event_id"]
        fingerprint = canonical_hash(full)
        if key in seen:
            if seen[key] != fingerprint:
                _set_error(source, "conflicting_event_id")
                # Neither version is a defensible observation of that period.
                source["events"] = [item for item in source["events"] if item["event_id"] != key]
                seen[key] = "conflict"
            continue
        seen[key] = fingerprint
        source["events"].append(event)


def _envelope(config, sources):
    for source in sources:
        source["events"].sort(key=lambda x: (x["start"], x["end"], x["event_id"]))
    return {"schema": "observatory.activity-evidence/1", "collected_at": _now(),
            "config_sha256": canonical_hash(config), "sources": sources}


def collect_export(config, export_data):
    """Sanitise the standard ``{buckets: {...}}`` ActivityWatch export in memory."""
    validate_config(config)
    if not isinstance(export_data, dict) or not isinstance(export_data.get("buckets"), dict):
        raise ValueError("expected ActivityWatch export with a buckets object")
    buckets = export_data["buckets"]
    allowed, sources = _allowlist(config), []
    for source, start, end in _source_spec(config):
        bucket = buckets.get(source["bucket_id"])
        if bucket is None:
            source.update(status="missing", error="bucket_missing")
        elif not isinstance(bucket, dict) or not isinstance(bucket.get("events"), list):
            _set_error(source, "malformed_bucket")
        else:
            events = bucket["events"]
            if len(events) > MAX_EVENTS:
                _set_error(source, "event_limit")
                events = events[:MAX_EVENTS]
            _append_events(source, events, start, end, allowed, {})
        sources.append(source)
    return _envelope(config, sources)


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def _server_origin(server_url):
    if not isinstance(server_url, str) or len(server_url) > 256:
        raise ValueError("ActivityWatch server must be an HTTP loopback origin")
    try:
        parts = urllib.parse.urlsplit(server_url)
        # Literal addresses avoid DNS rebinding and ambiguous localhost aliases.
        address = ipaddress.ip_address(parts.hostname or "")
        port = parts.port
    except (ValueError, TypeError):
        raise ValueError("ActivityWatch server must use a literal loopback IP") from None
    if (parts.scheme != "http" or not address.is_loopback or parts.username
            or parts.password or parts.path not in {"", "/"}
            or parts.query or parts.fragment or port == 0):
        raise ValueError("ActivityWatch server must be an HTTP loopback origin")
    return urllib.parse.urlunsplit(("http", parts.netloc, "", "", ""))


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate_json_key")
        result[key] = value
    return result


def _reject_constant(_value):
    raise ValueError("nonfinite_json")


def _get_json(opener, url):
    request = urllib.request.Request(url, headers={"Accept": "application/json"})
    with opener.open(request, timeout=TIMEOUT_SECONDS) as response:
        data = response.read(MAX_RESPONSE_BYTES + 1)
    if len(data) > MAX_RESPONSE_BYTES:
        raise ValueError("response_limit")
    return json.loads(data, object_pairs_hook=_unique_object, parse_constant=_reject_constant)


def _live_source(source, start, end, origin, opener, allowed):
    seen, requests, page_size = {}, 0, min(PAGE_SIZE, MAX_EVENTS)
    while True:
        if requests >= MAX_REQUESTS:
            _set_error(source, "request_limit")
            break
        # One overflow item distinguishes an exactly full page from a page
        # that may have omitted events. No unsupported offset parameter.
        query = urllib.parse.urlencode({"start": format_time(start), "end": format_time(end), "limit": page_size + 1})
        bucket = urllib.parse.quote(source["bucket_id"], safe="")
        url = f"{origin}/api/0/buckets/{bucket}/events?{query}"
        requests += 1
        try:
            events = _get_json(opener, url)
        except urllib.error.HTTPError as exc:
            if exc.code == 404 and requests == 1:
                source.update(status="missing", error="bucket_missing")
            else:
                _set_error(source, "http_error")
            break
        except (urllib.error.URLError, TimeoutError, OSError):
            source["status"] = "partial" if seen else "missing"
            source["error"] = source["error"] or "transport_error"
            break
        except (ValueError, TypeError, UnicodeError, RecursionError):
            _set_error(source, "malformed_response")
            break
        if not isinstance(events, list) or len(events) > page_size + 1:
            _set_error(source, "malformed_response")
            break
        # Keep received evidence even if a later request cannot finish. Using
        # the identical window matters: AW servers may clip event timestamps
        # and durations to the query endpoints.
        _append_events(source, events, start, end, allowed, seen)
        if len(events) <= page_size:
            break
        if len(seen) >= MAX_EVENTS:
            _set_error(source, "event_limit")
            break
        page_size = min(MAX_EVENTS, page_size * 2)


def collect_live(config, server_url="http://127.0.0.1:5600"):
    """Fetch only declared buckets and windows from a local ActivityWatch server."""
    validate_config(config)
    origin = _server_origin(server_url)
    # Loopback is local IPC, not a remote-service request. Avoid forwarding
    # desktop data through a configured HTTP proxy; redirects are never followed.
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), _NoRedirect())
    allowed, sources = _allowlist(config), []
    for source, start, end in _source_spec(config):
        _live_source(source, start, end, origin, opener, allowed)
        sources.append(source)
    return _envelope(config, sources)


def merge_activity(config, bundles):
    """Combine disjoint machine collections without reconciling changed evidence.

    Every input must describe the same declared frame and report every source,
    using explicit missing placeholders for devices unavailable on that machine.
    Differing nonmissing versions of one source require a new collection cut;
    this function never chooses a newer version or unions event snapshots.
    """
    # Keep the same normalized-evidence validation used before frame freezing.
    # The function-local import avoids a module-load cycle: frame uses this
    # module's allowlist and categories.
    from .frame import _validate_activity

    validate_config(config)
    if not isinstance(bundles, list) or not 1 <= len(bundles) <= 100:
        raise ValueError("merge requires 1..100 activity bundles")
    grouped = {}
    for bundle in bundles:
        _validate_activity(config, bundle)
        for source in bundle["sources"]:
            key = (source["person_id"], source["device_id"], source["bucket_id"], source["kind"])
            grouped.setdefault(key, {})[canonical_hash(source)] = source
    sources = []
    for spec, _start, _end in _source_spec(config):
        key = (spec["person_id"], spec["device_id"], spec["bucket_id"], spec["kind"])
        candidates = grouped[key]
        nonmissing = {digest: source for digest, source in candidates.items() if source["status"] != "missing"}
        if len(nonmissing) > 1:
            raise ValueError("conflicting nonmissing activity source; choose a new explicit collection cut")
        # Different missing error categories need not make a usable machine's
        # data disappear. When all are missing, preserve one deterministically.
        choices = nonmissing or candidates
        sources.append(copy.deepcopy(choices[min(choices)]))
    result = {
        "schema": "observatory.activity-evidence/1",
        "collected_at": format_time(max(parse_time(bundle["collected_at"]) for bundle in bundles)),
        "config_sha256": canonical_hash(config),
        "sources": sources,
        "merged_from": sorted({canonical_hash(bundle) for bundle in bundles}),
    }
    _validate_activity(config, result)
    return result
