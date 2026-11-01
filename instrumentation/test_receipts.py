"""Behavioral tests for the explicit, job-local execution recorder."""

import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest


RECORDER = Path(__file__).with_name("receipts.py")
SECRET = "secret-argv-env-transcript-must-not-enter-evidence"


class RecorderTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.directory = self.root / "evidence"
        self.env = {key: value for key, value in os.environ.items() if not key.startswith("GITHUB_")}
        self.env.update({
            "GITHUB_REPOSITORY_ID": "123456",
            "GITHUB_REPOSITORY": "example/project",
            "GITHUB_WORKFLOW_REF": "example/project/.github/workflows/build.yml@refs/heads/main",
            "GITHUB_WORKFLOW_SHA": "a" * 40,
            "GITHUB_SHA": "b" * 40,
            "GITHUB_RUN_ID": "98765",
            "GITHUB_RUN_ATTEMPT": "1",
            "GITHUB_JOB": "verify",
            "GITHUB_ACTOR": "credential-owner",
            "GITHUB_TRIGGERING_ACTOR": "rerun-operator",
            "GITHUB_EVENT_NAME": "workflow_dispatch",
            "GITHUB_TOKEN": SECRET,
            "OPENAI_API_KEY": SECRET,
            "UNRESTRICTED_CONTEXT": SECRET,
        })

    def cli(self, *args, expected=0, env=None, timeout=10):
        result = subprocess.run(
            [sys.executable, str(RECORDER), *map(str, args)],
            env=env or self.env,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        return result

    def init(self, *args, directory=None, env=None):
        directory = directory or self.directory
        self.cli("init", "--directory", directory, *args, env=env)
        return directory

    def run_tool(self, code="pass", *args, directory=None, expected=0):
        return self.cli(
            "run", "--directory", directory or self.directory,
            "--tool", "python", *args, "--", sys.executable, "-c", code,
            expected=expected,
        )

    def receipts(self, directory=None):
        return [json.loads(path.read_text()) for path in sorted((directory or self.directory).joinpath("receipts").glob("*.json"))]

    def phases(self, directory=None):
        values = self.receipts(directory)
        self.assertEqual(len(values), 2)
        return {value["phase"]: value for value in values}

    def publish(self, output=None, directory=None):
        output = output or self.root / "bundle.json"
        self.cli("publish", "--directory", directory or self.directory, "--output", output)
        return output

    def fake_codex(self, events):
        executable = self.root / "fake-codex"
        executable.write_text(
            "#!" + sys.executable + "\n"
            "import json, sys\n"
            "assert sys.argv[1:3] == ['exec', '--json'], sys.argv\n"
            + "\n".join("print(" + repr(json.dumps(event)) + ")" for event in events)
            + "\n"
        )
        executable.chmod(0o700)
        return executable

    def test_accounts_and_controller_are_independent_and_workflow_revision_is_exact(self):
        self.init("--controller", "assistant", "--controller-provenance", "session:declared",
                  "--session-id", "assistant-session", "--protocol-id", "protocol-1",
                  "--task-id", "requirement-7", "--job-name", "verify (linux)",
                  "--matrix", '{"os":"linux"}')
        self.run_tool("pass", "--tool-version", "3.test", "--purpose", "development",
                      "--logical-call-id", "logical-1", "--check-name", "unit tests")
        phases = self.phases()
        start, completion = phases["start"], phases["completion"]
        self.assertEqual(start["platform"]["actor"], "credential-owner")
        self.assertEqual(start["platform"]["triggering_actor"], "rerun-operator")
        self.assertEqual(start["platform"]["event_name"], "workflow_dispatch")
        self.assertEqual(start["platform"]["workflow_sha"], "a" * 40)
        self.assertEqual(start["platform"]["matrix"], {"os": "linux"})
        self.assertEqual(start["platform"]["source"], "runner_environment")
        self.assertEqual(start["declared"]["source"], "caller")
        self.assertEqual(start["declared"]["controller"], "assistant")
        self.assertEqual(start["declared"]["controller_provenance"], ["session:declared"])
        self.assertEqual(start["declared"]["task_id"], "requirement-7")
        self.assertEqual(start["declared"]["logical_call_id"], "logical-1")
        for key in ("invocation_id", "platform", "declared", "tool"):
            self.assertEqual(start[key], completion[key])
        self.assertIsNone(start["result"])
        self.assertEqual(completion["result"]["status"], "success")
        self.assertEqual(completion["result"]["exit_code"], 0)
        self.assertGreaterEqual(completion["result"]["runtime_ms"], 0)
        self.assertEqual(completion["result"]["checks"], [{"name": "unit tests", "status": "passed"}])

    def test_unknown_context_and_usage_are_preserved(self):
        env = {key: value for key, value in self.env.items() if not key.startswith("GITHUB_")}
        self.init(env=env)
        self.run_tool()
        completion = self.phases()["completion"]
        self.assertEqual(completion["platform"]["actor"], "unknown")
        self.assertEqual(completion["platform"]["run_id"], "unknown")
        self.assertIsNone(completion["platform"]["run_attempt"])
        self.assertEqual(completion["declared"]["controller"], "unknown")
        self.assertEqual(completion["tool"]["purpose"], "unknown")
        self.assertIsNone(completion["tool"]["version"])
        self.assertIsNone(completion["result"]["input_tokens"])
        self.assertIsNone(completion["result"]["output_tokens"])
        self.assertIsNone(completion["result"]["charge_amount"])
        self.assertEqual(completion["result"]["usage_source"], "unknown")
        self.assertIn("tool.version", completion["missing_fields"])
        self.assertIn("result.input_tokens", completion["missing_fields"])

    def test_command_failure_keeps_exit_status_and_failed_check(self):
        self.init()
        self.run_tool("raise SystemExit(42)", "--check-name", "tests", expected=42)
        result = self.phases()["completion"]["result"]
        self.assertEqual(result["exit_code"], 42)
        self.assertEqual(result["status"], "failure")
        self.assertEqual(result["checks"], [{"name": "tests", "status": "failed"}])

    def test_spawn_failure_retains_start_and_completion(self):
        self.init()
        self.cli("run", "--directory", self.directory, "--tool", "absent",
                 "--", self.root / "no-such-tool", expected=127)
        result = self.phases()["completion"]["result"]
        self.assertEqual(result["status"], "failure")
        self.assertEqual(result["exit_code"], 127)

    def test_retries_get_fresh_invocations_and_keep_declared_link(self):
        self.init("--session-id", "session-1")
        self.run_tool("raise SystemExit(1)", "--logical-call-id", "call-7", expected=1)
        parent = self.phases()["start"]["invocation_id"]
        self.run_tool("pass", "--logical-call-id", "call-7", "--parent-invocation-id", parent)
        starts = [receipt for receipt in self.receipts() if receipt["phase"] == "start"]
        self.assertEqual(len(starts), 2)
        self.assertEqual(len({receipt["invocation_id"] for receipt in starts}), 2)
        self.assertEqual({receipt["declared"]["logical_call_id"] for receipt in starts}, {"call-7"})
        self.assertEqual({receipt["declared"]["session_id"] for receipt in starts}, {"session-1"})
        self.assertEqual(sum(receipt["declared"]["parent_invocation_id"] == parent for receipt in starts), 1)

    def test_matrix_siblings_and_reruns_have_distinct_contexts(self):
        self.init("--job-name", "verify (linux)", "--matrix", '{"os":"linux"}')
        self.run_tool()
        sibling = self.root / "sibling"
        rerun_env = {**self.env, "GITHUB_RUN_ATTEMPT": "2", "GITHUB_TRIGGERING_ACTOR": "second-operator"}
        self.init("--job-name", "verify (macos)", "--matrix", '{"os":"macos"}', directory=sibling, env=rerun_env)
        self.run_tool(directory=sibling)
        first = self.phases()["start"]
        second = self.phases(sibling)["start"]
        self.assertEqual(first["platform"]["run_id"], second["platform"]["run_id"])
        self.assertEqual(first["platform"]["run_attempt"], 1)
        self.assertEqual(second["platform"]["run_attempt"], 2)
        self.assertEqual(second["platform"]["triggering_actor"], "second-operator")
        self.assertNotEqual(first["platform"]["matrix"], second["platform"]["matrix"])
        self.assertNotEqual(first["invocation_id"], second["invocation_id"])

    def test_bundle_is_hashed_repeatable_and_excludes_undesignated_files(self):
        self.init()
        self.run_tool("print(" + repr(SECRET) + ")", "--configuration", '{"temperature":0.2}')
        (self.directory / "unrestricted-context.txt").write_text(SECRET)
        output = self.publish()
        initial = output.read_bytes()
        self.publish(output)
        self.assertEqual(output.read_bytes(), initial)
        bundle = json.loads(initial)
        self.assertEqual(bundle["schema"], "observatory.execution-bundle/1")
        self.assertEqual(len(bundle["receipts"]), 2)
        for entry in bundle["receipts"]:
            self.assertEqual(entry["sha256"], hashlib.sha256(entry["receipt_json"].encode()).hexdigest())
        self.assertNotIn(SECRET, initial.decode())
        self.assertNotIn("GITHUB_TOKEN", initial.decode())
        self.assertNotIn("OPENAI_API_KEY", initial.decode())
        self.assertNotIn("UNRESTRICTED_CONTEXT", initial.decode())
        self.assertNotIn("unrestricted-context.txt", initial.decode())

    def test_designated_artifacts_are_hashed_with_labels_without_contents_or_paths(self):
        self.init()
        input_file = self.root / "private-input.txt"
        output_file = self.root / "private-output.txt"
        input_file.write_text(SECRET)
        output_file.write_text("result")
        self.run_tool("pass", "--input", "source=" + str(input_file), "--output", "result=" + str(output_file))
        phases = self.phases()
        self.assertIn({"role": "input", "name": "source", "sha256": hashlib.sha256(SECRET.encode()).hexdigest(), "size_bytes": len(SECRET)}, phases["start"]["artifacts"])
        self.assertIn({"role": "output", "name": "result", "sha256": hashlib.sha256(b"result").hexdigest(), "size_bytes": 6}, phases["completion"]["artifacts"])
        bundle_text = self.publish().read_text()
        self.assertNotIn(SECRET, bundle_text)
        self.assertNotIn(str(input_file), bundle_text)
        self.assertNotIn(str(output_file), bundle_text)

    def test_published_snapshot_cannot_be_overwritten_with_later_execution(self):
        self.init()
        self.run_tool()
        output = self.publish()
        original = output.read_bytes()
        self.run_tool()
        result = subprocess.run(
            [sys.executable, str(RECORDER), "publish", "--directory", str(self.directory),
             "--output", str(output)], env=self.env, capture_output=True, text=True, timeout=10,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(output.read_bytes(), original)
        later = self.publish(self.root / "later-bundle.json")
        self.assertEqual(len(json.loads(later.read_text())["receipts"]), 4)

    def test_codex_uses_reported_usage_without_turn_text_or_credential_inference(self):
        self.init("--controller", "human", "--session-id", "declared-session")
        executable = self.fake_codex([
            {"type": "thread.started", "thread_id": "observed-thread"},
            {"type": "item.completed", "item": {"text": SECRET, "model": "not-a-model-observation"}},
            {"type": "turn.completed", "usage": {"input_tokens": 100, "cached_input_tokens": 20, "output_tokens": 17}},
        ])
        self.cli("codex", "--directory", self.directory, "--executable", executable,
                 "--configured-model", "configured-model", "--purpose", "development", "--", SECRET)
        completion = self.phases()["completion"]
        self.assertEqual(completion["declared"]["controller"], "human")
        self.assertEqual(completion["declared"]["session_id"], "declared-session")
        self.assertEqual(completion["tool"]["adapter"], "codex_exec")
        self.assertEqual(completion["tool"]["configured_model"], "configured-model")
        self.assertIsNone(completion["result"]["observed_model"])
        self.assertEqual(completion["result"]["input_tokens"], 100)
        self.assertEqual(completion["result"]["cached_input_tokens"], 20)
        self.assertEqual(completion["result"]["output_tokens"], 17)
        self.assertEqual(completion["result"]["usage_source"], "tool_reported")
        self.assertEqual(completion["result"]["observed_session_id"], "observed-thread")
        self.assertIsNone(completion["result"]["charge_amount"])
        self.assertNotIn(SECRET, self.publish().read_text())

    def test_codex_without_usage_does_not_invent_zero_counts(self):
        self.init()
        executable = self.fake_codex([{"type": "turn.completed"}])
        self.cli("codex", "--directory", self.directory, "--executable", executable, "--", "benign prompt")
        result = self.phases()["completion"]["result"]
        self.assertIsNone(result["input_tokens"])
        self.assertIsNone(result["output_tokens"])
        self.assertEqual(result["usage_source"], "unknown")

    def test_codex_partial_usage_does_not_report_partial_sum_as_complete(self):
        self.init()
        executable = self.fake_codex([
            {"type": "turn.completed", "model": "model-a", "usage": {"input_tokens": 10, "output_tokens": 2, "cached_input_tokens": 1}},
            {"type": "turn.completed", "model": "model-b", "usage": {"input_tokens": 20, "output_tokens": 3}},
        ])
        self.cli("codex", "--directory", self.directory, "--executable", executable, "--", "benign prompt")
        result = self.phases()["completion"]["result"]
        self.assertEqual(result["input_tokens"], 30)
        self.assertEqual(result["output_tokens"], 5)
        self.assertIsNone(result["cached_input_tokens"])
        self.assertIsNone(result["observed_model"], "conflicting model reports must remain unresolved")

    def test_missing_designated_output_retains_success_and_explicit_missingness(self):
        self.init()
        self.run_tool("pass", "--output", "report=" + str(self.root / "absent.txt"))
        completion = self.phases()["completion"]
        self.assertEqual(completion["result"]["status"], "success")
        self.assertIn("artifacts.output.report", completion["missing_fields"])
        self.assertEqual(completion["artifacts"], [])

    def test_empty_job_bundle_retains_context_without_inventing_execution(self):
        self.init("--job-name", "conditionally empty")
        bundle = json.loads(self.publish().read_text())
        self.assertEqual(bundle["receipts"], [])
        self.assertEqual(bundle["platform"]["run_id"], "98765")
        self.assertEqual(bundle["platform"]["job_name"], "conditionally empty")

    def test_codex_invalid_or_overflowing_usage_preserves_completion_with_unknown_fields(self):
        self.init()
        executable = self.fake_codex([
            {"type": "turn.completed", "usage": {"input_tokens": 2**64 - 1, "cached_input_tokens": 2**64 - 1, "output_tokens": 1}},
            {"type": "turn.completed", "usage": {"input_tokens": 1, "cached_input_tokens": 2, "output_tokens": 2}},
        ])
        self.cli("codex", "--directory", self.directory, "--executable", executable, "--", "prompt")
        result = self.phases()["completion"]["result"]
        self.assertEqual(result["status"], "success")
        self.assertIsNone(result["input_tokens"])
        self.assertIsNone(result["cached_input_tokens"])
        self.assertEqual(result["output_tokens"], 3)
        self.publish()

    def test_publish_rejects_unallowlisted_metadata_before_creating_bundle(self):
        self.init()
        self.run_tool()
        path = next((self.directory / "receipts").glob("*.completion.json"))
        original = json.loads(path.read_text())
        for location in ("top", "result", "configuration"):
            with self.subTest(location=location):
                value = json.loads(json.dumps(original))
                if location == "top":
                    value["environment"] = {"SECRET": SECRET}
                elif location == "result":
                    value["result"]["prompt"] = SECRET
                else:
                    value["tool"]["configuration"]["api_key"] = SECRET
                path.write_text(json.dumps(value))
                output = self.root / f"rejected-{location}.json"
                self.cli("publish", "--directory", self.directory, "--output", output, expected=2)
                self.assertFalse(output.exists())

    def test_publish_rejects_duplicate_keys_that_could_hide_sensitive_bytes(self):
        self.init()
        self.run_tool()
        path = next((self.directory / "receipts").glob("*.completion.json"))
        original = path.read_text()
        path.write_text('{"schema":' + json.dumps(SECRET) + ',' + original[1:])
        output = self.root / "duplicate-key-bundle.json"
        self.cli("publish", "--directory", self.directory, "--output", output, expected=2)
        self.assertFalse(output.exists())

    def test_publish_rejects_invalid_usage_types(self):
        self.init()
        self.run_tool()
        path = next((self.directory / "receipts").glob("*.completion.json"))
        receipt = json.loads(path.read_text())
        receipt["result"]["input_tokens"] = "seven"
        receipt["result"]["usage_source"] = "tool_reported"
        path.write_text(json.dumps(receipt))
        output = self.root / "invalid-usage-bundle.json"
        self.cli("publish", "--directory", self.directory, "--output", output, expected=2)
        self.assertFalse(output.exists())

    def test_invalid_metadata_is_rejected_before_tool_launch(self):
        self.init()
        marker = self.root / "must-not-run"
        self.run_tool("from pathlib import Path; Path(" + repr(str(marker)) + ").touch()",
                      "--check-name", "x" * 257, expected=2)
        self.assertFalse(marker.exists())
        self.assertEqual(self.receipts(), [])

    def test_configuration_rejects_unallowlisted_metadata(self):
        self.init()
        result = subprocess.run(
            [sys.executable, str(RECORDER), "run", "--directory", str(self.directory),
             "--tool", "python", "--configuration", json.dumps({"api_key": SECRET}),
             "--", sys.executable, "-c", "pass"],
            env=self.env, capture_output=True, text=True, timeout=10,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn(SECRET, "".join(path.read_text() for path in self.directory.rglob("*.json")))

    @unittest.skipUnless(os.name == "posix", "requires POSIX process signals")
    def test_sigterm_reaches_child_and_retains_cancelled_completion(self):
        self.init()
        ready = self.root / "ready"
        stopped = self.root / "stopped"
        child = self.root / "child.py"
        child.write_text(
            "import pathlib, signal, sys, time\n"
            "def stop(signum, frame):\n"
            "    pathlib.Path(sys.argv[2]).write_text(str(signum))\n"
            "    raise SystemExit(143)\n"
            "signal.signal(signal.SIGTERM, stop)\n"
            "pathlib.Path(sys.argv[1]).write_text('ready')\n"
            "while True: time.sleep(0.05)\n"
        )
        process = subprocess.Popen(
            [sys.executable, str(RECORDER), "run", "--directory", str(self.directory),
             "--tool", "sleeper", "--", sys.executable, str(child), str(ready), str(stopped)],
            env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        try:
            deadline = time.monotonic() + 10
            while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                time.sleep(0.02)
            self.assertTrue(ready.exists(), "child did not start")
            starts = [receipt for receipt in self.receipts() if receipt["phase"] == "start"]
            self.assertEqual(len(starts), 1, "start must be durable before child execution")
            process.send_signal(signal.SIGTERM)
            stdout, stderr = process.communicate(timeout=10)
            self.assertEqual(process.returncode, 143, stdout + stderr)
            self.assertEqual(stopped.read_text(), str(signal.SIGTERM))
            result = self.phases()["completion"]["result"]
            self.assertEqual(result["status"], "cancelled")
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=5)


if __name__ == "__main__":
    unittest.main()
