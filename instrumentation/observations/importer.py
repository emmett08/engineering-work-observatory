"""Import explicitly declared public issue observations without inferring labour.

Only configured public repositories are queried. Every observed source version is
immutable and content addressed; GitHub's unseen intermediate edits are unknown.
The latest version can invalidate an earlier declaration. Issue state is metadata,
not a filter, and a posting account is never treated as the worker's identity.
"""
import copy
import hashlib
import json
from pathlib import Path
import re
import subprocess

from effort.common import (MAX_BYTES, canonical_hash, fields, finite, identifier,
                           integer, now, parse_time, read_json, require, write_json)

CONFIG_SCHEMA = "observatory.observation-config/1"
OBSERVATION_SCHEMA = "observatory.effort-observation/1"
REVISION_SCHEMA = "observatory.observation-revision/1"
LEGACY_PILOT_BODY_SHA256 = "9fadd6eded223343e074aae259aa9e0aea76554633fb8dbd0016ea772c844775"
REPOSITORY_RE = r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+"
SOURCE_KEYS = {"repository", "repository_id", "issue_number", "issue_id", "url",
               "state", "created_at", "updated_at", "body_sha256", "author_login"}
FORM_REQUIRED = {"Observation schema", "Protocol ID", "Task issue URL",
                 "Declared human person-minutes", "Measurement method", "Coverage", "Scope"}
FORM_OPTIONAL = {"Task ID (optional)", "Related PR URLs (optional)",
                 "Work completed at (UTC, optional or unknown)", "Notes (optional)", "Provenance"}


def _repository(value):
    require(isinstance(value, str) and re.fullmatch(REPOSITORY_RE, value), "invalid repository")
    return value.lower()


def validate_config(config):
    fields(config, {"schema", "protocol_id", "sources", "task_links"})
    require(config["schema"] == CONFIG_SCHEMA, "unsupported observation config schema")
    identifier(config["protocol_id"], "protocol_id")
    require(isinstance(config["sources"], list) and 0 < len(config["sources"]) <= 100, "invalid sources")
    repositories = set()
    for source in config["sources"]:
        fields(source, {"repository", "issue_numbers", "title_prefix", "max_pages"})
        repository = _repository(source["repository"])
        require(repository not in repositories, "duplicate repository source")
        repositories.add(repository)
        require(isinstance(source["issue_numbers"], list) and len(source["issue_numbers"]) <= 1000, "invalid issue numbers")
        for number in source["issue_numbers"]:
            integer(number, maximum=2**53-1)
        require(len(set(source["issue_numbers"])) == len(source["issue_numbers"]), "duplicate issue number")
        identifier(source["title_prefix"], "title prefix")
        integer(source["max_pages"], maximum=100)
    require(isinstance(config["task_links"], list) and len(config["task_links"]) <= 1000, "invalid task links")
    keys, aliases = set(), set()
    for link in config["task_links"]:
        _validate_task(link)
        repository = _repository(link["repository"])
        require(repository in repositories, "task repository must be a configured public source")
        key, alias = (repository, link["issue_number"]), link["task_id"]
        require(key not in keys and alias not in aliases, "duplicate task link or task_id")
        keys.add(key)
        aliases.add(alias)
    return config


def _validate_task(task):
    fields(task, {"repository", "issue_number", "task_id", "pull_requests"})
    _repository(task["repository"])
    integer(task["issue_number"], maximum=2**53-1)
    identifier(task["task_id"], "task_id")
    require(isinstance(task["pull_requests"], list) and len(task["pull_requests"]) <= 1000, "invalid pull requests")
    for number in task["pull_requests"]:
        integer(number, maximum=2**53-1)
    require(len(set(task["pull_requests"])) == len(task["pull_requests"]), "duplicate pull request")


def validate_observation(observation, config=None):
    fields(observation, {"schema", "protocol_id", "task", "declared_minutes", "method",
                         "coverage", "scope", "completed_at", "synthetic"})
    require(observation["schema"] == OBSERVATION_SCHEMA, "unsupported effort observation schema")
    identifier(observation["protocol_id"], "protocol_id")
    _validate_task(observation["task"])
    minutes = observation["declared_minutes"]
    if minutes is not None:
        require(0 <= finite(minutes, "declared minutes") <= 1e9, "declared minutes outside supported range")
    require(observation["method"] in {"timer", "recall", "unknown"}, "invalid measurement method")
    require(observation["coverage"] in {"complete", "partial", "unknown"}, "invalid coverage")
    require(observation["scope"] in {"task_total", "partial_task", "unknown"}, "invalid scope")
    if observation["completed_at"] is not None:
        parse_time(observation["completed_at"])
        require(observation["completed_at"].endswith(("Z", "+00:00")), "completed_at must be UTC")
    require(type(observation["synthetic"]) is bool, "synthetic must be an explicit boolean")
    if config is not None:
        require(observation["protocol_id"] == config["protocol_id"], "observation protocol differs from config")
        require(observation["synthetic"] is False, "live issue observations cannot be synthetic")
        link = _task_link(config, observation["task"]["repository"], observation["task"]["issue_number"])
        require(observation["task"] == link, "observation task or PRs differ from configured link")
    return observation


def _task_link(config, repository, number):
    matches = [link for link in config["task_links"] if _repository(link["repository"]) == _repository(repository)
               and link["issue_number"] == number]
    require(len(matches) == 1, "observation task is not uniquely configured")
    return copy.deepcopy(matches[0])


def _strict_json(text):
    require(isinstance(text, str) and len(text.encode("utf-8")) <= MAX_BYTES, "JSON exceeds supported size")
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result
    def invalid(_):
        raise ValueError("non-finite JSON number")
    try:
        value = json.loads(text, object_pairs_hook=pairs, parse_constant=invalid)
        canonical_hash(value)
        return value
    except (UnicodeError, RecursionError, json.JSONDecodeError) as error:
        raise ValueError("invalid JSON document") from error


def _sections(body):
    matches = list(re.finditer(r"(?m)^#{2,3} ([^\r\n]+)\r?$", body))
    sections = {}
    for index, match in enumerate(matches):
        key = match.group(1).strip()
        require(key not in sections, "duplicate observation heading")
        end = matches[index + 1].start() if index + 1 < len(matches) else len(body)
        sections[key] = body[match.end():end].strip()
    return sections


def _value(value):
    value = value.strip()
    if value.startswith("`") and value.endswith("`") and value.count("`") == 2:
        value = value[1:-1]
    return value


def _unknown(value):
    return _value(value) in {"", "unknown", "_No response_"}


def _parse_form(config, sections):
    require(FORM_REQUIRED <= sections.keys(), "missing versioned observation form headings")
    require(sections.keys() <= FORM_REQUIRED | FORM_OPTIONAL, "unknown versioned observation form heading")
    require(_value(sections["Observation schema"]) == OBSERVATION_SCHEMA, "unsupported observation form schema")
    issue_match = re.fullmatch(r"https://github\.com/(" + REPOSITORY_RE + r")/issues/([1-9][0-9]*)", _value(sections["Task issue URL"]))
    require(issue_match is not None, "task issue requires a canonical GitHub issue URL")
    task = _task_link(config, issue_match[1], int(issue_match[2]))
    task_id = sections.get("Task ID (optional)", "unknown")
    require(_unknown(task_id) or _value(task_id) == task["task_id"], "task alias differs from configured link")
    prs = sections.get("Related PR URLs (optional)", "unknown")
    if not _unknown(prs):
        expected = {f"https://github.com/{task['repository']}/pull/{number}" for number in task["pull_requests"]}
        supplied = set(_value(prs).split())
        require(supplied == expected, "related PR URLs differ from configured link")
    raw_minutes = _value(sections["Declared human person-minutes"])
    minutes = None
    if not _unknown(raw_minutes):
        require(re.fullmatch(r"(?:0|[1-9][0-9]*)(?:\.[0-9]+)?", raw_minutes), "minutes must be one nonnegative decimal or unknown")
        minutes = float(raw_minutes)
    completed = sections.get("Work completed at (UTC, optional or unknown)", "unknown")
    return {"schema": OBSERVATION_SCHEMA, "protocol_id": _value(sections["Protocol ID"]),
            "task": task, "declared_minutes": minutes,
            "method": _value(sections["Measurement method"]), "coverage": _value(sections["Coverage"]),
            "scope": _value(sections["Scope"]), "completed_at": None if _unknown(completed) else _value(completed),
            "synthetic": False}


def _parse_legacy(config, source, sections):
    # The explicitly identified pilot declaration is the only unversioned adapter.
    # It never harvests lifecycle durations, title numbers or free prose as effort.
    require(source["repository"] == "emmett08/engarg" and source["issue_number"] == 51,
            "unversioned observation has no supported adapter")
    require(source["body_sha256"] == LEGACY_PILOT_BODY_SHA256,
            "legacy pilot declaration changed; migrate the edited body to a versioned observation")
    require(set(sections) == {"Protocol ID", "Task ID", "Task issue and related PR", "Human effort",
                              "API-observed lifecycle timestamps (UTC)", "Scope and uncertainty"},
            "legacy pilot headings changed; use a versioned observation")
    task = _task_link(config, "emmett08/engarg", 49)
    require(_value(sections["Task ID"]) == task["task_id"], "legacy task alias differs from config")
    require(re.search(r"(?m)^- Task: #49$", sections["Task issue and related PR"]) and
            re.search(r"(?m)^- Implementation: #50$", sections["Task issue and related PR"]), "legacy task linkage changed")
    effort = sections["Human effort"]
    first = effort.splitlines()[0]
    match = re.fullmatch(r"\*\*((?:0|[1-9][0-9]*)(?:\.[0-9]+)?) human person-minutes \((?:0|[1-9][0-9]*)(?:\.[0-9]+)? person-hours\)\.\*\*", first)
    require(match is not None, "legacy effort declaration is not recognized")
    require("Observation basis: explicit user declaration:" in effort,
            "legacy declaration provenance missing")
    # This adapter intentionally cannot upgrade unknown measurement provenance.
    return {"schema": OBSERVATION_SCHEMA, "protocol_id": _value(sections["Protocol ID"]), "task": task,
            "declared_minutes": float(match[1]), "method": "unknown", "coverage": "unknown",
            "scope": "task_total", "completed_at": None, "synthetic": False}


def _parse(config, source, title, body):
    configured = next(item for item in config["sources"] if _repository(item["repository"]) == source["repository"])
    require(title.startswith(configured["title_prefix"]), "issue no longer has configured observation title prefix")
    if "```observatory-effort" in body:
        blocks = re.findall(r"(?ms)^```observatory-effort[ \t]*\r?\n(.*?)^```[ \t]*\r?$", body)
        require(len(blocks) == 1 and body.count("```observatory-effort") == 1, "expected exactly one observation JSON block")
        require("Observation schema" not in _sections(body), "mixed observation representations")
        observation = _strict_json(blocks[0])
    else:
        sections = _sections(body)
        observation = (_parse_form(config, sections) if "Observation schema" in sections
                       else _parse_legacy(config, source, sections))
    return validate_observation(observation, config)


def _public_repository(config, repository_metadata):
    require(isinstance(repository_metadata, dict), "repository metadata must be an object")
    repository = _repository(repository_metadata.get("full_name"))
    require(repository in {_repository(item["repository"]) for item in config["sources"]}, "repository is not configured")
    require(repository_metadata.get("private") is False and repository_metadata.get("visibility", "public") == "public",
            "only public repositories may supply this demo")
    integer(repository_metadata.get("id"), maximum=2**53-1)
    return repository


def _source(config, metadata, issue):
    repository = _public_repository(config, metadata)
    require(isinstance(issue, dict) and "pull_request" not in issue, "expected an issue, not a pull request")
    number = integer(issue.get("number"), maximum=2**53-1)
    integer(issue.get("id"), maximum=2**53-1)
    url = f"https://github.com/{repository}/issues/{number}"
    require(isinstance(issue.get("html_url"), str) and issue["html_url"].lower() == url, "issue URL differs from source repository")
    require(issue.get("state") in {"open", "closed"}, "invalid source issue state")
    require(parse_time(issue.get("created_at")) <= parse_time(issue.get("updated_at")), "source update predates creation")
    body, title = issue.get("body"), issue.get("title")
    body = "" if body is None else body
    require(isinstance(body, str) and len(body.encode("utf-8")) <= 1024 * 1024, "issue body exceeds supported size")
    require(isinstance(title, str) and len(title) <= 1024, "invalid issue title")
    user = issue.get("user")
    author = user.get("login") if isinstance(user, dict) else None
    if author is not None:
        identifier(author, "source posting account")
    source = {"repository": repository, "repository_id": metadata["id"], "issue_number": number,
              "issue_id": issue["id"], "url": url, "state": issue["state"], "created_at": issue["created_at"],
              "updated_at": issue["updated_at"], "body_sha256": hashlib.sha256(body.encode("utf-8")).hexdigest(),
              "author_login": author}
    return source, title, body


def _observation_id(source):
    return canonical_hash({"repository": source["repository"], "issue_number": source["issue_number"]})


def _envelope_identity(envelope):
    record = {key: value for key, value in envelope["record"].items() if key not in {"observed_at", "revision_id"}}
    return canonical_hash({"record": record, "title": envelope["title"], "body": envelope["body"]})


def import_issue(config, state, repository_metadata, issue, observed_at=None):
    """Retain one immutable source revision, including rejected new declarations."""
    validate_config(config)
    source, title, body = _source(config, repository_metadata, issue)
    timestamp = observed_at or now()
    parse_time(timestamp)
    try:
        observation = _parse(config, source, title, body)
        status, errors = "accepted", []
    except (ValueError, TypeError, OverflowError) as error:
        observation, status, errors = None, "rejected", [str(error)]
    record = {"schema": REVISION_SCHEMA, "observation_id": _observation_id(source), "revision_id": None,
              "source": source, "observed_at": timestamp, "status": status, "observation": observation, "errors": errors}
    envelope = {"record": record, "title": title, "body": body}
    record["revision_id"] = _envelope_identity(envelope)
    path = Path(state) / "observations" / "revisions" / record["observation_id"] / (record["revision_id"] + ".json")
    if path.exists():
        return _read_revision(path)
    try:
        write_json(path, envelope)
    except ValueError:
        # Another delivery may have published the same content identity with a
        # different first-observed time between the existence check and link.
        if path.exists():
            return _read_revision(path)
        raise
    return record


def _read_revision(path):
    envelope = read_json(path)
    fields(envelope, {"record", "title", "body"})
    require(isinstance(envelope["title"], str) and isinstance(envelope["body"], str), "invalid source snapshot text")
    record = envelope["record"]
    fields(record, {"schema", "observation_id", "revision_id", "source", "observed_at", "status", "observation", "errors"})
    require(record["schema"] == REVISION_SCHEMA, "unsupported observation revision schema")
    fields(record["source"], SOURCE_KEYS)
    require(record["revision_id"] == path.stem == _envelope_identity(envelope), "observation revision hash mismatch")
    require(record["observation_id"] == path.parent.name == _observation_id(record["source"]), "observation identity mismatch")
    require(record["source"]["body_sha256"] == hashlib.sha256(envelope["body"].encode("utf-8")).hexdigest(), "source body hash mismatch")
    require(record["status"] in {"accepted", "rejected"}, "invalid stored observation status")
    require(isinstance(record["errors"], list) and all(isinstance(item, str) for item in record["errors"]), "invalid observation errors")
    if record["status"] == "accepted":
        validate_observation(record["observation"])
        require(not record["errors"], "accepted revision contains errors")
    else:
        require(record["observation"] is None and record["errors"], "rejected revision contains a label")
    parse_time(record["observed_at"])
    parse_time(record["source"]["updated_at"])
    return record


def latest_observations(state):
    """Verify all retained revisions and project latest sources without fallback."""
    groups = {}
    paths = sorted((Path(state) / "observations" / "revisions").glob("*/*.json"))
    require(len(paths) <= 100000, "too many observation revisions")
    for path in paths:
        record = _read_revision(path)
        groups.setdefault(record["observation_id"], []).append(record)
    latest = []
    for identity in sorted(groups):
        revisions = groups[identity]
        timestamp = max(parse_time(row["source"]["updated_at"]) for row in revisions)
        rows = [row for row in revisions if parse_time(row["source"]["updated_at"]) == timestamp]
        row = copy.deepcopy(rows[0])
        if len(rows) > 1 or len({r["source"]["issue_id"] for r in revisions}) > 1:
            row["status"], row["observation"] = "conflict", None
            row["errors"] = ["conflicting source identities or revisions at the latest source update time"]
            row["revision_id"] = canonical_hash({"conflicting_revision_ids": sorted(r["revision_id"] for r in rows)})
        latest.append(row)
    return latest


def _api(endpoint):
    """Read bounded public API responses; never expose token, headers or stderr."""
    try:
        result = subprocess.run(["gh", "api", "--hostname", "github.com", "--method", "GET", endpoint], capture_output=True,
                                text=True, timeout=60, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ValueError("GitHub API request unavailable") from error
    require(result.returncode == 0, "GitHub API request failed")
    return _strict_json(result.stdout)


def collect(config, state):
    """Read open and closed issues with bounded pagination and explicit coverage.

    A false complete flag must prevent model promotion. A successful partial
    delivery is still retained, but does not certify that restored labels remain
    current. Previously tracked issues are explicitly refetched after renaming.
    """
    validate_config(config)
    known = latest_observations(state)
    result = {"schema": "observatory.observation-coverage/1", "complete": True,
              "observed_at": now(), "sources": [], "imported": 0, "errors": []}
    configured_repositories = {_repository(item["repository"]) for item in config["sources"]}
    archived_repositories = {row["source"]["repository"] for row in known} - configured_repositories
    if archived_repositories:
        result["complete"] = False
        result["errors"].append("retained observations include repositories removed from config; use a separate state or an explicit migration")
    for specification in config["sources"]:
        repository = _repository(specification["repository"])
        coverage = {"repository": repository, "public_verified": False, "complete": True,
                    "pages": 0, "issues_seen": 0, "revisions_imported": 0, "errors": []}
        result["sources"].append(coverage)
        observed_revisions = set()
        try:
            metadata = _api(f"repos/{repository}")
            require(_public_repository(config, metadata) == repository, "API repository identity differs from request")
            coverage["public_verified"] = True
            tracked = {r["source"]["issue_number"] for r in known if r["source"]["repository"] == repository}
            required = set(specification["issue_numbers"]) | tracked
            issues = {}
            for page in range(1, specification["max_pages"] + 1):
                data = _api(f"repos/{repository}/issues?state=all&per_page=100&sort=created&direction=asc&page={page}")
                require(isinstance(data, list) and len(data) <= 100, "invalid issue listing")
                coverage["pages"] += 1
                coverage["issues_seen"] += len(data)
                for issue in data:
                    require(isinstance(issue, dict), "invalid issue listing entry")
                    if "pull_request" in issue:
                        continue
                    if issue.get("number") in required or str(issue.get("title", "")).startswith(specification["title_prefix"]):
                        record = import_issue(config, state, metadata, issue, result["observed_at"])
                        issues[issue["number"]] = record
                        observed_revisions.add(record["revision_id"])
                if len(data) < 100:
                    break
            else:
                coverage["complete"] = False
                coverage["errors"].append("issue pagination limit reached; acquisition may be incomplete")
            # Fetch every explicitly configured/tracked issue again, including
            # ones listed earlier. A newer edit invalidates the earlier snapshot.
            for number in sorted(required):
                issue = _api(f"repos/{repository}/issues/{number}")
                require(isinstance(issue, dict) and issue.get("number") == number, "explicit issue identity differs from request")
                record = import_issue(config, state, metadata, issue, result["observed_at"])
                issues[number] = record
                observed_revisions.add(record["revision_id"])
        except (ValueError, TypeError, KeyError) as error:
            coverage["complete"] = False
            coverage["errors"].append(str(error))
        coverage["revisions_imported"] = len(observed_revisions)
        result["imported"] += coverage["revisions_imported"]
        if not coverage["complete"]:
            result["complete"] = False
            result["errors"].append(f"{repository}: acquisition incomplete")
    return result
