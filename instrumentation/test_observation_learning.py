"""End-to-end public import, eligibility, report and synthetic learning checks."""
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from effort.common import read_json
import learn_effort
from observations import importer


ROOT = Path(__file__).resolve().parents[1]


class ObservationLearningTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.config = read_json(ROOT / "examples/observation-learning/public-config.json")
        self.manifest = read_json(ROOT / "examples/observation-learning/execution-manifest.json")
        self.issue = {
            "number": 51, "id": 1051, "title": "[Effort observation] Issue #49 / PR #50: 3 minutes of declared human effort",
            "state": "closed", "html_url": "https://github.com/emmett08/engarg/issues/51",
            "created_at": "2026-11-01T19:33:03Z", "updated_at": "2026-11-01T20:01:07Z",
            "user": {"login": "fixture-posting-account"},
            "body": read_json(ROOT / "instrumentation/fixtures/legacy-effort-engarg-51.json")["body"],
        }
        self.metadata = {"id": 1431331749, "full_name": "emmett08/engarg", "private": False, "visibility": "public"}
        self.calls = []

    def api(self, endpoint):
        self.calls.append(endpoint)
        if endpoint == "repos/emmett08/engarg":
            return self.metadata
        if "?state=all&" in endpoint:
            return [self.issue]
        if endpoint == "repos/emmett08/engarg/issues/51":
            return self.issue
        self.fail("unexpected API request: " + endpoint)

    def run_demo(self, output="report", api=None, live=True):
        with patch.object(importer, "_api", side_effect=api or self.api), patch.object(
            learn_effort, "collect_executions", return_value={"complete": True, "reason": None, "tasks": []}
        ):
            return learn_effort.run(self.config, self.manifest, self.directory / "state",
                                    self.directory / output, Path("unused-test-binary"), enable_live_learning=live)

    def test_closed_live_declaration_and_separate_synthetic_promotion_survive_replay(self):
        first = self.run_demo()
        second = self.run_demo("report-repeat")
        self.assertTrue(first["acquisition"]["complete"])
        self.assertEqual(first["source_states"], {"closed": 1})
        self.assertEqual(first["retained_revision_count"], 1)
        row = first["observations"][0]
        self.assertEqual(row["declared_minutes"], 3)
        self.assertIn("unknown_measurement_method", row["model_exclusion"])
        self.assertEqual(row["comparison"], "unavailable")
        self.assertIsNone(row["active_model_prediction_minutes"])
        self.assertEqual(first["live_model"]["decision"], "insufficient_data")
        self.assertEqual(first["live_model"]["counts"]["eligible"], 0)
        synthetic = first["synthetic_demo"]
        self.assertEqual(synthetic["evaluation"]["decision"], "promoted")
        self.assertTrue(synthetic["replay_idempotent"])
        self.assertTrue(all(p["prediction_minutes"] is not None and p["observed_minutes"] is None
                            for p in synthetic["future_predictions"]))
        self.assertEqual(first["live_model"]["evaluation_id"], second["live_model"]["evaluation_id"])
        self.assertEqual(synthetic["evaluation"]["evaluation_id"], second["synthetic_demo"]["evaluation"]["evaluation_id"])
        self.assertEqual(second["retained_revision_count"], 1)
        self.assertIn("Synthetic software demonstration", (self.directory / "report/report.md").read_text())

    def test_incomplete_issue_scan_preserves_report_but_does_not_evaluate_stale_labels(self):
        self.run_demo()
        before = sorted((self.directory / "state/model-learning/live/evaluations").glob("*.json"))
        def missing(endpoint):
            if endpoint == "repos/emmett08/engarg":
                return self.metadata
            raise ValueError("simulated issue API outage")
        report = self.run_demo("gap-report", api=missing)
        self.assertFalse(report["acquisition"]["complete"])
        self.assertEqual(report["live_model"]["decision"], "acquisition_incomplete")
        self.assertEqual(report["observations"][0]["source_currentness"], "unconfirmed_incomplete_acquisition")
        self.assertIsNone(report["observations"][0]["active_model_prediction_minutes"])
        self.assertEqual(before, sorted((self.directory / "state/model-learning/live/evaluations").glob("*.json")))

    def test_private_visibility_change_stops_public_report_and_checkpoint_publication(self):
        self.run_demo()
        self.metadata = {**self.metadata, "private": True, "visibility": "private"}
        with self.assertRaisesRegex(ValueError, "public source verification failed"):
            self.run_demo("private-report")
        self.assertFalse((self.directory / "private-report").exists())

    def test_execution_protocol_and_task_alias_must_match(self):
        manifest = copy.deepcopy(self.manifest)
        manifest["tasks"][0]["task_id"] = "different-task"
        with self.assertRaisesRegex(ValueError, "task/issue/PR link"):
            learn_effort.validate_links(self.config, manifest)
        manifest = copy.deepcopy(self.manifest)
        manifest["protocol_id"] = "different-protocol"
        with self.assertRaisesRegex(ValueError, "protocols differ"):
            learn_effort.validate_links(self.config, manifest)

    def test_isolated_preview_never_fits_or_scores_live_labels(self):
        report = self.run_demo(live=False)
        self.assertEqual(report["live_model"]["decision"], "observation_only")
        self.assertEqual(report["observations"][0]["declared_minutes"], 3)
        self.assertIn("unknown_measurement_method", report["observations"][0]["model_exclusion"])
        self.assertFalse((self.directory / "state/model-learning/live").exists())
        self.assertEqual(report["synthetic_demo"]["evaluation"]["decision"], "promoted")


if __name__ == "__main__":
    unittest.main()
