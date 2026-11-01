"""Join/provenance, missing-data, privacy and publication-boundary checks."""
from copy import deepcopy
import json
import os
import subprocess
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import pilot
from effort.activity import collect_export
from effort.common import read_json, write_json
from effort.frame import build_frame
from effort.sampling import draw_sample, response_template

ROOT = Path(__file__).resolve().parents[1]


def manifest():
    return {"schema": "observatory.linked-pilot/1", "protocol_id": "protocol-pilot", "tasks": [
        {"task_id": "task-one", "repository": "owner/one", "issue_number": 4, "pull_requests": [5], "run_ids": [100]}]}


def source(kind, payload, identity="source", namespace="github:repository:1", at="2026-01-01T00:00:00Z"):
    return {"record": {"id": identity, "kind": "source_record", "data": {"source": kind, "repository": namespace, "source_id": identity, "payload": payload}}, "observed_at": at}


def records():
    p = {"repository": "owner/one", "repository_id": "1", "run_id": "100", "run_attempt": 1, "job_id": None, "job_name": "test (linux)", "actor": "original", "triggering_actor": "rerunner", "event_name": "pull_request"}
    declared = {"protocol_id": "protocol-pilot", "task_id": "task-one", "controller": "unknown", "session_id": None}
    common = {"invocation_id": "invocation-1", "platform": p, "declared": declared, "tool": {"name": "python3"}, "result": None}
    start = dict(common, phase="start")
    completion = dict(common, phase="completion", result={"status": "success", "exit_code": 0, "runtime_ms": 1234})
    return [
        source("github.repository", {"id": 1, "full_name": "owner/one"}),
        source("github.workflow_run", {"id": 100, "run_attempt": 1, "status": "completed", "conclusion": "success"}, "run:100"),
        source("github.workflow_attempt", {"id": 100, "run_attempt": 1, "actor": {"login": "original"}, "triggering_actor": {"login": "rerunner"}, "event": "pull_request"}, "attempt:100:1"),
        source("github.workflow_job", {"id": 111, "name": "test (linux)", "run_id": 100, "run_attempt": 1, "status": "completed", "conclusion": "success"}, "job:111"),
        source("github.workflow_job", {"id": 112, "name": "other", "run_id": 100, "run_attempt": 1, "status": "completed", "conclusion": "skipped"}, "job:112"),
        source("observatory.tool_receipt", start, "scope/one/start"),
        source("observatory.tool_receipt", completion, "scope/one/completion"),
    ]


def summary(items=None):
    m = manifest()
    return pilot.execution_summary(m, m["tasks"][0], records() if items is None else items)


def real_frame():
    config = read_json(ROOT / "examples/effort/config.json")
    config["protocol_id"] = "protocol-pilot"
    config["synthetic"] = False
    config["links"][0].update(task_id="task-one", repository="owner/one", issue_number=4, pull_requests=[5])
    return build_frame(config, collect_export(config, read_json(ROOT / "examples/effort/activitywatch-export.json")))


class PilotTests(unittest.TestCase):
    def test_manifest_rejects_unsafe_repository_duplicate_targets_and_boolean_ids(self):
        for modify in (lambda m: m["tasks"][0].update(repository="owner/repo/../../escape"),
                       lambda m: m["tasks"].append(deepcopy(m["tasks"][0])),
                       lambda m: m["tasks"][0].update(run_ids=[True])):
            m = manifest()
            modify(m)
            with self.assertRaises(ValueError):
                pilot.validate_manifest(m)

    def test_exact_linkage_excludes_other_protocol_task_repository_and_run(self):
        for field, value in (("protocol_id", "other"), ("task_id", "other"), ("repository", "owner/other"), ("run_id", "200")):
            items = records()
            for item in items[-2:]:
                receipt = item["record"]["data"]["payload"]
                receipt["declared" if field.endswith("_id") and field != "run_id" else "platform"][field] = value
            with self.subTest(field=field):
                self.assertEqual(summary(items)["invocation_count"], 0)

    def test_repeated_delivery_deduplicates_lifecycle_and_runtime(self):
        items = records()
        result = summary(items + deepcopy(items[-2:]))
        self.assertEqual(result["invocation_count"], 1)
        self.assertEqual(result["reported_runtime_seconds"], 1.234)
        self.assertEqual(result["jobs_without_matched_task_receipts"], 1)
        self.assertEqual(result["jobs"][1]["conclusion"], "skipped")
        invocation = result["invocations"][0]
        self.assertEqual(invocation["actor"], "original")
        self.assertEqual(invocation["triggering_actor"], "rerunner")
        self.assertEqual(invocation["declared_controller"], "unknown")
        self.assertEqual(invocation["platform_comparison"], "compared_with_retained_api")
        self.assertEqual(invocation["delivery"], "imported_without_artifact_delivery_comparison")

    def test_delivery_requires_all_receipt_ids_and_matching_repo_run(self):
        items = records()
        value = {"repository": "owner/one", "run_id": 100, "record_ids": ["scope/one/start", "scope/one/completion"]}
        items.append(source("github.execution_artifact_delivery", value, "delivery:1"))
        self.assertEqual(summary(items)["invocations"][0]["delivery"], "github_artifact_collector")
        value["repository"] = "owner/other"
        self.assertEqual(summary(items)["invocations"][0]["delivery"], "imported_without_artifact_delivery_comparison")

    def test_wrong_actor_or_repository_id_cannot_receive_matched_job(self):
        for field, wrong in (("actor", "somebody-else"), ("repository_id", "999")):
            items = records()
            for item in items[-2:]:
                item["record"]["data"]["payload"]["platform"][field] = wrong
            result = summary(items)
            self.assertEqual(result["invocations"][0]["platform_comparison"], "conflict")
            self.assertIsNone(result["invocations"][0]["job_id"])
            self.assertEqual(result["jobs_without_matched_task_receipts"], 2)

    def test_receipts_without_api_or_with_ambiguous_job_remain_unresolved(self):
        self.assertEqual(summary(records()[-2:])["invocations"][0]["platform_comparison"], "unresolved")
        items = records()
        items.append(source("github.workflow_job", {"id": 113, "name": "test (linux)", "run_id": 100, "run_attempt": 1}, "job:113"))
        self.assertEqual(summary(items)["invocations"][0]["platform_comparison"], "unresolved")

    def test_start_only_missing_upload_and_cancelled_run_remain_distinct(self):
        items = records()[:-1]
        self.assertEqual(summary(items)["invocations"][0]["lifecycle"], "start_only")
        self.assertIsNone(summary(items)["reported_runtime_seconds"])
        items = records()[:-2]
        items[1]["record"]["data"]["payload"]["conclusion"] = "cancelled"
        result = summary(items)
        self.assertEqual(result["invocation_count"], 0)
        self.assertIsNone(result["reported_runtime_seconds"])
        self.assertEqual(result["selected_runs"][0]["conclusion"], "cancelled")
        self.assertEqual(result["jobs_without_matched_task_receipts"], 2)

    def test_rerun_and_matrix_identity_cannot_cross_link(self):
        items = records()
        for item in items[-2:]:
            item["record"]["data"]["payload"]["platform"]["run_attempt"] = 2
        self.assertEqual(summary(items)["invocations"][0]["platform_comparison"], "unresolved")
        items = records()
        for item in items[-2:]:
            item["record"]["data"]["payload"]["platform"]["job_name"] = "test (macos)"
        self.assertIsNone(summary(items)["invocations"][0]["job_id"])

    def test_frame_protocol_issue_and_pr_must_match(self):
        frame = real_frame()
        pilot.validate_frame_links(manifest(), frame)
        for mutate in (lambda f: f.update(protocol_id="other"),
                       lambda f: f["links"][0].update(issue_number=88),
                       lambda f: f["links"][0].update(pull_requests=[77])):
            candidate = deepcopy(frame)
            mutate(candidate)
            with self.assertRaises(ValueError):
                pilot.validate_frame_links(manifest(), candidate)

    def test_pending_and_nonresponse_never_become_zero_effort(self):
        task = manifest()["tasks"][0]
        pending = pilot.effort_summary(None, None, [], task, None, None)
        self.assertIsNone(pending["estimate_person_hours"])
        self.assertIsNone(pending["proxy_hours"])
        frame = real_frame()
        sample = draw_sample(frame, 2)
        missing = pilot.effort_summary(frame, sample, [], task, None, None)
        self.assertEqual(missing["sampling"]["missing_count"], 2)
        self.assertIsNone(missing["estimate_person_hours"])
        self.assertIn("sample_nonresponse", missing["unavailable_reasons"])
        serialized = json.dumps(missing)
        for private in ("participant-a", "participant-b", "laptop-a", "aw-watcher", "responses", "frames", "units"):
            self.assertNotIn('"' + private + '"', serialized)

    def test_signed_sample_estimate_uses_real_rust_estimator_and_private_inputs(self):
        binary = Path(os.environ.get("OBSERVATORY_BIN", ROOT / "target/debug/observatory")).resolve()
        if not binary.is_file():
            self.skipTest("build Observatory or set OBSERVATORY_BIN for Rust estimator integration")
        config = read_json(ROOT / "examples/effort/config.json")
        frame = build_frame(config, collect_export(config, read_json(ROOT / "examples/effort/activitywatch-export.json")))
        sample = draw_sample(frame, 2, "0" * 64)
        responses = response_template(frame, sample)
        # Manufactured observed zeros deliberately overcorrect this draw's proxy.
        for response in responses["responses"]:
            response.update(status="observed", allocations=[], non_work_minutes=15, reason=None)
        task = {"task_id": "issue-alpha", "repository": "example/alpha", "issue_number": 101, "pull_requests": [11, 12], "run_ids": []}
        m = {"schema": "observatory.linked-pilot/1", "protocol_id": config["protocol_id"], "tasks": [task]}
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            batch, state, output = root / "batch", root / "state", root / "public"
            write_json(batch / "frame.json", frame)
            write_json(batch / "sample.json", sample)
            write_json(batch / "responses/observed.json", responses)
            subprocess.run([str(binary), "--state", str(state), "verify"], check=True, capture_output=True)
            reports = pilot.report(m, state, output, binary, batch)
            human = reports[0]["human_effort"]
            self.assertEqual(human["mode"], "sample_adjusted_estimate")
            self.assertTrue(human["synthetic"])
            self.assertAlmostEqual(human["estimate_person_hours"], -.5)
            self.assertTrue(human["estimate_outside_declared_capacity"])
            self.assertIsNotNone(human["sampling_standard_error_person_hours"])
            self.assertIn("outside declared physical capacity", pilot.markdown(reports[0]))
            self.assertEqual(len(list((batch / "estimator-inputs").glob("*.json"))), 1)
            serialized = json.dumps(reports)
            for private in ("participant-a", "participant-b", "laptop-a", "aw-watcher", str(batch), "synthetic-two-people-one-hour"):
                self.assertNotIn(private, serialized)
            with patch("pilot._gh") as gh, self.assertRaises(ValueError):
                pilot.publish(reports[0])
            gh.assert_not_called()

    def test_missing_run_metadata_is_explicit_in_public_markdown(self):
        m = manifest()
        value = {"protocol_id": m["protocol_id"], "task_id": "task-one", "execution": summary([]),
                 "human_effort": pilot.effort_summary(None, None, [], m["tasks"][0], None, None),
                 "issue_url": "https://github.com/owner/one/issues/4", "pull_request_urls": [],
                 "limitations": pilot.LIMITATIONS, "manifest_sha256": "a" * 64}
        text = pilot.markdown(value)
        self.assertIn("runs without retained API metadata: 1", text)
        self.assertIn("status unknown; conclusion unknown", text)
        self.assertIn("Human-effort estimate: unavailable", text)

    def test_publication_refuses_synthetic_and_posts_only_registered_destinations(self):
        report = {"human_effort": {"synthetic": True}}
        with patch("pilot._gh") as gh, self.assertRaises(ValueError):
            pilot.publish(report)
        gh.assert_not_called()
        report = {"protocol_id": "p", "task_id": "t", "repository": "owner/one", "issue_number": 4, "pull_requests": [5], "human_effort": {"synthetic": False}}
        calls = []
        def gh(args, payload=None):
            calls.append((args, payload))
            if args == ["user"]:
                return {"login": "publisher"}
            if payload is None:
                return []
            return {"id": 123}
        with patch("pilot._gh", side_effect=gh), patch("pilot.markdown", return_value="Human effort: unavailable"):
            pilot.publish(report)
        posts = [args for args, payload in calls if payload is not None]
        self.assertEqual(posts, [["--method", "POST", "repos/owner/one/issues/4/comments"], ["--method", "POST", "repos/owner/one/issues/5/comments"]])

    def test_publication_never_edits_another_authors_matching_marker(self):
        report = {"protocol_id": "p", "task_id": "t", "repository": "owner/one", "issue_number": 4, "pull_requests": [], "human_effort": {"synthetic": False}}
        marker = "<!-- observatory-linked-pilot:" + pilot.canonical_hash({"protocol": "p", "task": "t"}) + " -->"
        calls = []
        def gh(args, payload=None):
            calls.append(args)
            if args == ["user"]:
                return {"login": "publisher"}
            if payload is None:
                return [{"user": {"login": "another"}, "body": marker, "id": 111}]
            return {"id": 123}
        with patch("pilot._gh", side_effect=gh), patch("pilot.markdown", return_value="Human effort: unavailable"):
            pilot.publish(report)
        self.assertIn(["--method", "POST", "repos/owner/one/issues/4/comments"], calls)
        self.assertFalse(any("PATCH" in call for call in calls))


if __name__ == "__main__":
    unittest.main()
