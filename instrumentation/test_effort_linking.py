"""Observed GitHub relationships must not become guessed task attribution."""
import copy
import json
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

from effort.common import canonical_hash
from effort.linking import _decode, _fetch_github, sync_links


ROOT = Path(__file__).resolve().parents[1]


def config():
    value = json.loads((ROOT / "examples/effort/config.json").read_text())
    for link in value["links"]:
        link["pull_requests"] = []
        link["branches"] = []
    return value


def response(number=11, issues=(("example/alpha", 101),), *, repository="example/alpha",
             head_repository="example/alpha", branch="issue-101-report", total=None,
             more=False, cursor=None):
    return {"data": {"repository": {"nameWithOwner": repository, "pullRequest": {
        "number": number, "url": f"https://github.com/{repository}/pull/{number}",
        "headRefName": branch,
        "headRepository": None if head_repository is None else {"nameWithOwner": head_repository},
        "closingIssuesReferences": {
            "totalCount": len(issues) if total is None else total,
            "nodes": [{"repository": {"nameWithOwner": repo}, "number": issue,
                       "url": f"https://github.com/{repo}/issues/{issue}"} for repo, issue in issues],
            "pageInfo": {"hasNextPage": more, "endCursor": cursor},
        },
    }}}}


class LinkTests(unittest.TestCase):
    def test_api_association_is_retained_separately_from_caller_aliases(self):
        source = config()
        original = copy.deepcopy(source)
        updated, evidence = sync_links(source, "example/alpha", [11], lambda *args: response())
        self.assertEqual(source, original)
        self.assertEqual(updated["links"][0]["projects"], ["alpha"])
        self.assertEqual(updated["links"][0]["pull_request_urls"], ["https://github.com/example/alpha/pull/11"])
        self.assertEqual(updated["links"][0]["branch_links"], [{"repository": "example/alpha", "branch": "issue-101-report"}])
        self.assertEqual(evidence["input_config_sha256"], canonical_hash(source))
        self.assertEqual(evidence["output_config_sha256"], canonical_hash(updated))
        self.assertEqual(evidence["observations"][0]["matched_task_ids"], ["issue-alpha"])
        page = evidence["observations"][0]["pages"][0]
        self.assertEqual(page["sha256"], canonical_hash(page["record"]))

    def test_cross_repository_and_fork_head_have_explicit_identities(self):
        updated, evidence = sync_links(config(), "example/tools", [11], lambda *args: response(
            repository="example/tools", head_repository="contributor/tools"))
        link = updated["links"][0]
        self.assertEqual(link["pull_requests"], [])
        self.assertEqual(link["pull_request_urls"], ["https://github.com/example/tools/pull/11"])
        self.assertEqual(link["branch_links"], [{"repository": "contributor/tools", "branch": "issue-101-report"}])
        self.assertEqual(evidence["observations"][0]["repository"], "example/tools")

    def test_deleted_head_still_retains_issue_pr_link_without_invented_branch(self):
        updated, _ = sync_links(config(), "example/alpha", [11], lambda *args: response(head_repository=None))
        self.assertNotIn("branch_links", updated["links"][0])
        self.assertIn("pull_request_urls", updated["links"][0])

    def test_multiple_linked_issues_are_kept_and_unknown_issues_are_not_registered(self):
        updated, evidence = sync_links(config(), "example/alpha", [11], lambda *args: response(
            issues=(("example/alpha", 101), ("example/beta", 202), ("example/other", 999))))
        self.assertEqual(len(updated["links"]), 2)
        self.assertEqual(updated["links"][0]["pull_request_urls"], updated["links"][1]["pull_request_urls"])
        self.assertEqual(evidence["observations"][0]["matched_task_ids"], ["issue-alpha", "issue-beta"])

    def test_complete_pagination_and_stable_identity(self):
        calls = []
        def fetch(repo, number, cursor):
            calls.append(cursor)
            if cursor is None:
                return response(issues=(("example/alpha", 101),), total=2, more=True, cursor="next")
            return response(issues=(("example/beta", 202),), total=2, cursor="done")
        _, evidence = sync_links(config(), "example/alpha", [11], fetch)
        self.assertEqual(calls, [None, "next"])
        self.assertEqual(len(evidence["observations"][0]["response_sha256"]), 2)

    def test_changed_identity_or_count_fails_whole_collection(self):
        for second in (response(issues=(("example/beta", 202),), total=2, branch="changed"),
                       response(issues=(("example/beta", 202),), total=1)):
            with self.subTest(second=second), self.assertRaisesRegex(ValueError, "changed"):
                sync_links(config(), "example/alpha", [11], lambda repo, num, cursor:
                           response(total=2, more=True, cursor="next") if cursor is None else second)

    def test_duplicate_pages_and_incomplete_counts_fail(self):
        with self.assertRaisesRegex(ValueError, "duplicate issue"):
            sync_links(config(), "example/alpha", [11], lambda repo, num, cursor:
                       response(total=2, more=cursor is None, cursor="next"))
        with self.assertRaisesRegex(ValueError, "incomplete"):
            sync_links(config(), "example/alpha", [11], lambda *args: response(total=2))

    def test_existing_uncorroborated_binding_requires_review(self):
        value = config()
        value["links"][0]["pull_requests"] = [11]
        with self.assertRaisesRegex(ValueError, "not corroborated"):
            sync_links(value, "example/alpha", [11], lambda *args: response(issues=()))

    def test_error_partial_data_or_mismatched_urls_do_not_create_links(self):
        malformed = response()
        malformed["errors"] = [{"message": "sensitive error body"}]
        wrong_url = response()
        wrong_url["data"]["repository"]["pullRequest"]["url"] = "https://example.invalid/secret"
        for document in (malformed, {"data": {"repository": None}}, wrong_url):
            with self.subTest(document=document), self.assertRaises(ValueError) as error:
                sync_links(config(), "example/alpha", [11], lambda *args: document)
            self.assertNotIn("sensitive", str(error.exception))
            self.assertNotIn("secret", str(error.exception))

    def test_requested_numbers_reject_bool_duplicates_and_shell_like_repository(self):
        for repository, numbers in (("example/alpha", [True]), ("example/alpha", [11, 11]),
                                    ("example/alpha;echo-secret", [11])):
            with self.subTest(repository=repository, numbers=numbers), self.assertRaises(ValueError):
                sync_links(config(), repository, numbers, lambda *args: self.fail("fetch must not run"))

    def test_failure_on_later_pr_does_not_mutate_input(self):
        source = config()
        original = copy.deepcopy(source)
        def fetch(repo, number, cursor):
            return response() if number == 11 else {"errors": [{"message": "unavailable"}]}
        with self.assertRaises(ValueError):
            sync_links(source, "example/alpha", [11, 12], fetch)
        self.assertEqual(source, original)

    def test_irrelevant_api_fields_are_not_retained(self):
        document = response()
        document["extra"] = "private-value-must-not-be-exported"
        _, evidence = sync_links(config(), "example/alpha", [11], lambda *args: document)
        self.assertNotIn("private-value", json.dumps(evidence))

    def test_bounded_strict_json(self):
        for raw in (b'{"data":1,"data":2}', b'{"data":NaN}', b'x'*(1024*1024+1)):
            with self.subTest(raw_length=len(raw)), self.assertRaises(ValueError):
                _decode(raw)

    def test_cli_fetch_uses_read_only_query_stdin_and_never_shell(self):
        def run(args, **kwargs):
            self.assertEqual(args, ["gh", "api", "graphql", "--hostname", "github.com", "--input", "-"])
            self.assertNotIn("shell", kwargs)
            document = json.loads(kwargs["input"])
            self.assertTrue(document["query"].startswith("query("))
            self.assertNotIn("mutation", document["query"])
            self.assertEqual(document["variables"]["number"], 11)
            kwargs["stdout"].write(json.dumps(response()).encode())
            return subprocess.CompletedProcess(args, 0)
        with patch("effort.linking.subprocess.run", side_effect=run):
            self.assertEqual(_fetch_github("example/alpha", 11, None), response())


if __name__ == "__main__":
    unittest.main()
