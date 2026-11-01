"""Read GitHub's PR-to-issue relationships without inspecting or running code.

Only registered issue identities can acquire links. Project aliases and initial
bindings remain caller declarations; API observations are retained separately.
GraphQL contract: https://docs.github.com/en/graphql/reference/objects#pullrequest
"""

from __future__ import annotations

import copy
import json
import re
import subprocess
import tempfile

from .common import (canonical_bytes, canonical_hash, identifier, integer, now,
                     require, validate_config)

MAX_PULL_REQUESTS = 50
MAX_ISSUE_PAGES = 10
PAGE_SIZE = 100
MAX_RESPONSE_BYTES = 1024 * 1024
QUERY = """query($owner:String!,$name:String!,$number:Int!,$cursor:String) {
  repository(owner:$owner,name:$name) {
    nameWithOwner
    pullRequest(number:$number) {
      number url headRefName headRepository { nameWithOwner }
      closingIssuesReferences(first:100,after:$cursor) {
        totalCount
        nodes { number url repository { nameWithOwner } }
        pageInfo { hasNextPage endCursor }
      }
    }
  }
}"""


def _repository(value):
    require(isinstance(value, str) and len(value) <= 256
            and re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", value),
            "invalid GitHub repository")
    return value


def _decode(raw):
    require(len(raw) <= MAX_RESPONSE_BYTES, "GitHub response exceeds limit")
    def unique(pairs):
        value = {}
        for key, item in pairs:
            require(key not in value, "duplicate GitHub JSON key")
            value[key] = item
        return value
    def constant(_):
        raise ValueError("invalid GitHub JSON constant")
    try:
        value = json.loads(raw, object_pairs_hook=unique, parse_constant=constant)
        canonical_bytes(value)
    except (UnicodeError, RecursionError, json.JSONDecodeError) as error:
        raise ValueError("invalid GitHub JSON response") from error
    return value


def _fetch_github(repository, number, cursor):
    """Use the configured gh authentication; never expose token or error bodies."""
    owner, name = repository.split("/")
    request = {"query": QUERY, "variables": {
        "owner": owner, "name": name, "number": number, "cursor": cursor,
    }}
    try:
        with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
            result = subprocess.run(
                ["gh", "api", "graphql", "--hostname", "github.com", "--input", "-"],
                input=canonical_bytes(request), stdout=output, stderr=errors,
                timeout=45, check=False,
            )
            require(result.returncode == 0, "GitHub link retrieval failed; check gh authentication and repository access")
            output.seek(0)
            raw = output.read(MAX_RESPONSE_BYTES + 1)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ValueError("GitHub link retrieval unavailable or timed out") from error
    return _decode(raw)


def _page(document, repository, number):
    require(isinstance(document, dict) and not document.get("errors"), "GitHub returned an error or incomplete query")
    data = document.get("data")
    require(isinstance(data, dict) and isinstance(data.get("repository"), dict), "GitHub repository unavailable")
    repo = data["repository"]
    actual_repository = _repository(repo.get("nameWithOwner"))
    require(actual_repository.lower() == repository.lower(), "GitHub response repository mismatch")
    pr = repo.get("pullRequest")
    require(isinstance(pr, dict), "GitHub pull request unavailable")
    require(integer(pr.get("number"), maximum=2**31-1) == number, "GitHub response PR mismatch")
    expected_url = f"https://github.com/{actual_repository}/pull/{number}"
    require(pr.get("url") == expected_url, "GitHub response PR URL mismatch")
    branch = identifier(pr.get("headRefName"), "GitHub branch")
    head = pr.get("headRepository")
    require(head is None or isinstance(head, dict), "invalid GitHub head repository")
    head_repository = None if head is None else _repository(head.get("nameWithOwner"))
    connection = pr.get("closingIssuesReferences")
    require(isinstance(connection, dict), "missing GitHub issue connection")
    total = integer(connection.get("totalCount"), minimum=0, maximum=MAX_ISSUE_PAGES * PAGE_SIZE)
    nodes = connection.get("nodes")
    require(isinstance(nodes, list) and len(nodes) <= PAGE_SIZE, "invalid GitHub issue page")
    issues = []
    for issue in nodes:
        require(isinstance(issue, dict) and isinstance(issue.get("repository"), dict), "invalid GitHub issue identity")
        issue_repository = _repository(issue["repository"].get("nameWithOwner"))
        issue_number = integer(issue.get("number"), maximum=2**31-1)
        url = f"https://github.com/{issue_repository}/issues/{issue_number}"
        require(issue.get("url") == url, "GitHub response issue URL mismatch")
        issues.append({"repository": issue_repository, "number": issue_number, "url": url})
    page = connection.get("pageInfo")
    require(isinstance(page, dict) and type(page.get("hasNextPage")) is bool, "invalid GitHub pagination metadata")
    cursor = page.get("endCursor")
    if page["hasNextPage"]:
        identifier(cursor, "GitHub pagination cursor")
        require(nodes, "empty nonterminal GitHub page")
    else:
        require(cursor is None or isinstance(cursor, str) and len(cursor) <= 256, "invalid GitHub final cursor")
    identity = {"repository": actual_repository, "number": number, "url": expected_url,
                "head_repository": head_repository, "head_branch": branch}
    return identity, issues, total, page["hasNextPage"], cursor


def sync_links(config, repository, pull_requests, fetch=None):
    """Enrich registered issues from a bounded, complete GitHub PR query.

    ``fetch(repository, pr_number, cursor)`` returns one GraphQL response and can
    be injected for offline tests. No updated config is returned on failed,
    truncated, inconsistent or stale relationship evidence. The input is never
    mutated. An issue association supplies provenance, not an effort quantity.
    """
    validate_config(config)
    repository = _repository(repository)
    require(isinstance(pull_requests, (list, tuple)) and 0 < len(pull_requests) <= MAX_PULL_REQUESTS,
            "select 1..50 pull requests")
    for number in pull_requests:
        integer(number, maximum=2**31-1)
    require(len(set(pull_requests)) == len(pull_requests), "duplicate requested pull request")
    fetch = fetch or _fetch_github
    updated = copy.deepcopy(config)
    observations = []
    for number in sorted(pull_requests):
        cursor, identity, expected_total = None, None, None
        cursors, observed_issues = set(), {}
        page_hashes, pages = [], []
        for _ in range(MAX_ISSUE_PAGES):
            document = fetch(repository, number, cursor)
            current, issues, total, more, next_cursor = _page(document, repository, number)
            if identity is None:
                identity, expected_total = current, total
            require(identity == current and expected_total == total, "GitHub relationship changed during pagination; retry with a new collection")
            page_hashes.append(canonical_hash(document))
            record = {"request_cursor": cursor, "pull_request": current,
                      "closing_issues": issues, "total_count": total,
                      "has_next_page": more, "end_cursor": next_cursor}
            pages.append({"record": record, "sha256": canonical_hash(record)})
            for issue in issues:
                key = (issue["repository"].lower(), issue["number"])
                require(key not in observed_issues, "duplicate issue across GitHub pages; collection may have changed")
                observed_issues[key] = issue
            if not more:
                break
            require(next_cursor not in cursors, "repeated GitHub pagination cursor")
            cursors.add(next_cursor)
            cursor = next_cursor
        else:
            raise ValueError("GitHub issue pagination exceeds limit; no links updated")
        require(len(observed_issues) == expected_total, "incomplete GitHub issue connection; no links updated")
        matches = []
        for link in updated["links"]:
            key = (link["repository"].lower(), link["issue_number"])
            matched = key in observed_issues
            existing_url = identity["url"] in link.get("pull_request_urls", [])
            existing_number = link["repository"].lower() == identity["repository"].lower() and number in link["pull_requests"]
            require(not (existing_url or existing_number) or matched,
                    "configured PR association is not corroborated by current closing-issue links; review config")
            if not matched:
                continue
            matches.append(link["task_id"])
            urls = link.setdefault("pull_request_urls", [])
            if identity["url"] not in urls:
                urls.append(identity["url"])
            urls.sort()
            # Explicit head repository avoids collisions between fork/base
            # branches with the same name, including cross-repository issues.
            if identity["head_repository"] is not None:
                branch_link = {"repository": identity["head_repository"], "branch": identity["head_branch"]}
                branch_links = link.setdefault("branch_links", [])
                if branch_link not in branch_links:
                    branch_links.append(branch_link)
                branch_links.sort(key=lambda item: (item["repository"], item["branch"]))
        observations.append({**identity, "closing_issues": [observed_issues[key] for key in sorted(observed_issues)],
                             "matched_task_ids": sorted(matches), "response_sha256": page_hashes,
                             "pages": pages})
    validate_config(updated)
    evidence = {
        "schema": "observatory.github-task-links/1", "collected_at": now(),
        "source": "github_graphql_closing_issues", "input_config_sha256": canonical_hash(config),
        "output_config_sha256": canonical_hash(updated), "requested_repository": repository,
        "observations": observations,
        "limitations": [
            "Only the explicitly requested PRs and API closing-issue relationships were queried; this is not complete work coverage.",
            "Issue registration, project aliases and original configuration bindings remain caller declarations.",
            "A retained relationship is evidence at collection time; it does not identify attention, duration or human control.",
            "Existing bindings are not silently deleted; stale queried PR associations require a reviewed configuration.",
            "Page records retain only queried identity fields. Response hashes refer to canonical decoded API JSON, not raw transport bytes.",
        ],
    }
    return updated, evidence
