"""Regression tests for public declarations, source revisions and completeness."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from effort.common import read_json
from observations.importer import (CONFIG_SCHEMA, OBSERVATION_SCHEMA, collect,
                                   import_issue, latest_observations,
                                   validate_config, validate_observation)


def config(repository="demo/tools", number=7):
    return {"schema": CONFIG_SCHEMA, "protocol_id": "pilot-v1",
            "sources": [{"repository": repository, "issue_numbers": [number],
                         "title_prefix": "[Effort observation]", "max_pages": 2}],
            "task_links": [{"repository": repository, "issue_number": 3,
                            "task_id": "task-three", "pull_requests": [4]}]}


def observation(configuration=None):
    configuration = configuration or config()
    return {"schema": OBSERVATION_SCHEMA, "protocol_id": configuration["protocol_id"],
            "task": copy.deepcopy(configuration["task_links"][0]), "declared_minutes": 3,
            "method": "recall", "coverage": "complete", "scope": "task_total",
            "completed_at": None, "synthetic": False}


def metadata(repository="demo/tools", private=False):
    return {"id": 100, "full_name": repository, "private": private,
            "visibility": "private" if private else "public"}


def issue(payload=None, state="open", number=7, updated="2026-11-01T12:00:00Z", repository="demo/tools"):
    payload = observation() if payload is None else payload
    return {"number": number, "id": 1000 + number,
            "title": "[Effort observation] Task three", "state": state,
            "html_url": f"https://github.com/{repository}/issues/{number}",
            "created_at": "2026-11-01T10:00:00Z", "updated_at": updated,
            "user": {"login": "posting-assistant-account"},
            "body": "```observatory-effort\n" + json.dumps(payload) + "\n```"}


def form_body(minutes="3", **overrides):
    values = {"Observation schema": OBSERVATION_SCHEMA, "Protocol ID": "pilot-v1",
              "Task issue URL": "https://github.com/demo/tools/issues/3",
              "Task ID (optional)": "_No response_", "Related PR URLs (optional)": "unknown",
              "Declared human person-minutes": minutes, "Measurement method": "unknown",
              "Coverage": "unknown", "Scope": "task_total",
              "Work completed at (UTC, optional or unknown)": "unknown",
              "Notes (optional)": "optional provenance", "Provenance": "- [X] Declared observation"}
    values.update(overrides)
    return "\n\n".join(f"### {heading}\n\n{value}" for heading, value in values.items())


class ObservationImportTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.state = Path(self.temporary.name)
        self.config = config()

    def ingest(self, source=None, **kwargs):
        return import_issue(self.config, self.state, metadata(), source or issue(), **kwargs)

    def test_open_closed_and_repeated_deliveries_are_one_latest_observation(self):
        first = self.ingest(observed_at="2026-11-01T12:01:00Z")
        repeat = self.ingest(observed_at="2026-11-01T12:02:00Z")
        self.assertEqual(first, repeat)
        self.assertEqual(len(list(self.state.rglob("*.json"))), 1)
        closed = self.ingest(issue(state="closed", updated="2026-11-01T12:03:00Z"))
        rows = latest_observations(self.state)
        self.assertEqual(rows, [closed])
        self.assertEqual(rows[0]["observation"]["declared_minutes"], 3)
        self.assertEqual(rows[0]["source"]["state"], "closed")
        self.assertEqual(len(list(self.state.rglob("*.json"))), 2)

    def test_revisions_are_preserved_and_older_delivery_does_not_roll_back(self):
        first = self.ingest()
        updated = observation()
        updated["declared_minutes"] = 5
        newer = self.ingest(issue(updated, updated="2026-11-01T13:00:00Z"))
        self.ingest(observed_at="2026-11-01T14:00:00Z")
        self.assertEqual(latest_observations(self.state), [newer])
        self.assertNotEqual(first["revision_id"], newer["revision_id"])

    def test_invalid_latest_edit_does_not_resurrect_previous_declaration(self):
        self.ingest()
        source = issue(updated="2026-11-01T13:00:00Z")
        source["body"] = "Withdrawn: this was an incorrect declaration."
        invalid = self.ingest(source)
        self.assertEqual(invalid["status"], "rejected")
        self.assertEqual(latest_observations(self.state), [invalid])
        self.assertIsNone(invalid["observation"])

    def test_title_removal_invalidates_latest(self):
        self.ingest()
        source = issue(updated="2026-11-01T13:00:00Z")
        source["title"] = "Withdrawn observation"
        self.ingest(source)
        self.assertEqual(latest_observations(self.state)[0]["status"], "rejected")

    def test_same_update_time_conflicting_bodies_are_quarantined(self):
        self.ingest()
        alternate = observation()
        alternate["declared_minutes"] = 5
        self.ingest(issue(alternate))
        row = latest_observations(self.state)[0]
        self.assertEqual(row["status"], "conflict")
        self.assertIsNone(row["observation"])
        self.ingest(issue(updated="2026-11-01T14:00:00Z"))
        self.assertEqual(latest_observations(self.state)[0]["status"], "accepted")

    def test_changed_issue_identity_is_quarantined(self):
        self.ingest()
        other = issue(updated="2026-11-01T14:00:00Z")
        other["id"] = 9999
        self.ingest(other)
        self.assertEqual(latest_observations(self.state)[0]["status"], "conflict")

    def test_unknown_form_provenance_does_not_become_complete_or_timed(self):
        source = issue()
        source["body"] = form_body()
        row = self.ingest(source)
        self.assertEqual(row["status"], "accepted")
        self.assertEqual(row["observation"]["declared_minutes"], 3)
        self.assertEqual(row["observation"]["method"], "unknown")
        self.assertEqual(row["observation"]["coverage"], "unknown")
        self.assertIsNone(row["observation"]["completed_at"])
        self.assertEqual(row["source"]["author_login"], "posting-assistant-account")
        self.assertNotIn("worker", json.dumps(row))
        source["body"] = form_body(minutes="unknown")
        unknown = self.ingest(source)
        self.assertIsNone(unknown["observation"]["declared_minutes"])

    def test_form_typed_declarations_and_linkage(self):
        source = issue()
        source["body"] = form_body("3.5", **{"Measurement method": "timer", "Coverage": "complete",
                                             "Related PR URLs (optional)": "https://github.com/demo/tools/pull/4",
                                             "Task ID (optional)": "task-three",
                                             "Work completed at (UTC, optional or unknown)": "2026-11-01T11:00:00Z"})
        row = self.ingest(source)
        self.assertEqual(row["status"], "accepted")
        self.assertEqual(row["observation"]["method"], "timer")
        source["body"] = form_body(**{"Task ID (optional)": "wrong-task"})
        self.assertEqual(self.ingest(source)["status"], "rejected")

    def test_form_rejects_ranges_duplicate_headings_and_unrecognized_fields(self):
        for body in [form_body("3-5"), form_body("3 minutes"), form_body("-3"),
                     form_body() + "\n\n### Coverage\n\ncomplete",
                     form_body() + "\n\n### Actual minutes\n\n9"]:
            with self.subTest(body=body):
                source = issue()
                source["body"] = body
                self.assertEqual(self.ingest(source)["status"], "rejected")

    def test_nonfinite_boolean_negative_and_unknown_keys_are_rejected(self):
        for value in [float("nan"), float("inf"), -1, True, "3", 1e100]:
            with self.subTest(value=value):
                payload = observation()
                payload["declared_minutes"] = value
                self.assertEqual(self.ingest(issue(payload))["status"], "rejected")
        payload = observation()
        payload["worker"] = "posting-assistant-account"
        self.assertEqual(self.ingest(issue(payload))["status"], "rejected")

    def test_duplicate_json_keys_and_multiple_representations_rejected(self):
        source = issue()
        source["body"] = source["body"].replace('"declared_minutes": 3', '"declared_minutes": 3, "declared_minutes": 5')
        self.assertEqual(self.ingest(source)["status"], "rejected")
        source["body"] = issue()["body"] + "\n\n" + form_body()
        self.assertEqual(self.ingest(source)["status"], "rejected")

    def test_synthetic_or_unlinked_live_payload_is_rejected(self):
        payload = observation()
        payload["synthetic"] = True
        self.assertEqual(self.ingest(issue(payload))["status"], "rejected")
        payload = observation()
        payload["task"]["issue_number"] = 8
        self.assertEqual(self.ingest(issue(payload))["status"], "rejected")

    def test_private_repository_or_forged_source_url_not_retained(self):
        with self.assertRaises(ValueError):
            import_issue(self.config, self.state, metadata(private=True), issue())
        source = issue()
        source["html_url"] = "https://github.com/other/project/issues/7"
        with self.assertRaises(ValueError):
            self.ingest(source)
        self.assertEqual(list(self.state.rglob("*.json")), [])

    def test_revision_tampering_fails_closed(self):
        self.ingest()
        path = next(self.state.rglob("*.json"))
        envelope = read_json(path)
        envelope["record"]["observation"]["declared_minutes"] = 999
        path.write_text(json.dumps(envelope))
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            latest_observations(self.state)

    def test_legacy_closed_pilot_imports_only_known_three_minutes(self):
        cfg = config("emmett08/engarg", 51)
        cfg["protocol_id"] = "observatory-linked-pilot-v1"
        cfg["task_links"] = [{"repository": "emmett08/engarg", "issue_number": 49,
                              "task_id": "engarg-authoring-effort-pilot", "pull_requests": [50]}]
        source = issue(repository="emmett08/engarg", number=51, state="closed")
        source["body"] = read_json(Path(__file__).parent / "fixtures" / "legacy-effort-engarg-51.json")["body"]
        row = import_issue(cfg, self.state, metadata("emmett08/engarg"), source)
        self.assertEqual(row["status"], "accepted")
        self.assertEqual(row["observation"]["declared_minutes"], 3)
        self.assertEqual(row["observation"]["method"], "unknown")
        self.assertEqual(row["observation"]["coverage"], "unknown")
        self.assertEqual(row["source"]["state"], "closed")
        source["body"] += "\nCorrection: 10 minutes."
        source["updated_at"] = "2026-11-01T15:00:00Z"
        edited = import_issue(cfg, self.state, metadata("emmett08/engarg"), source)
        self.assertEqual(edited["status"], "rejected")
        self.assertIsNone(latest_observations(self.state)[0]["observation"])

    def test_invalid_config_and_missing_explicit_synthetic_are_rejected(self):
        cfg = config()
        cfg["sources"][0]["max_pages"] = 0
        with self.assertRaises(ValueError):
            validate_config(cfg)
        payload = observation()
        del payload["synthetic"]
        with self.assertRaises(ValueError):
            validate_observation(payload)

    def test_collect_all_states_and_explicit_delivery_deduplicates(self):
        closed = issue(state="closed")
        def api(endpoint):
            if endpoint == "repos/demo/tools":
                return metadata()
            if "state=all" in endpoint:
                return [closed, {"number": 9, "pull_request": {"url": "ignored"}}]
            self.assertEqual(endpoint, "repos/demo/tools/issues/7")
            return closed
        with patch("observations.importer._api", side_effect=api) as mock:
            coverage = collect(self.config, self.state)
        self.assertTrue(coverage["complete"])
        self.assertTrue(coverage["sources"][0]["public_verified"])
        self.assertEqual(mock.call_count, 3)
        self.assertEqual(coverage["imported"], 1)
        self.assertEqual(len(list(self.state.rglob("*.json"))), 1)
        self.assertEqual(latest_observations(self.state)[0]["source"]["state"], "closed")

    def test_collect_private_repository_never_requests_issues(self):
        with patch("observations.importer._api", return_value=metadata(private=True)) as mock:
            coverage = collect(self.config, self.state)
        self.assertFalse(coverage["complete"])
        self.assertFalse(coverage["sources"][0]["public_verified"])
        self.assertEqual(mock.call_count, 1)
        self.assertEqual(list(self.state.rglob("*.json")), [])

    def test_collect_marks_truncated_or_missing_sources_incomplete(self):
        self.config["sources"][0]["max_pages"] = 1
        with patch("observations.importer._api", side_effect=[metadata(), [issue()] * 100, issue()]):
            coverage = collect(self.config, self.state)
        self.assertFalse(coverage["complete"])
        with patch("observations.importer._api", side_effect=[metadata(), [], ValueError("GitHub API request failed")]):
            missing = collect(self.config, self.state)
        self.assertFalse(missing["complete"])

    def test_collect_refetches_tracked_issue_even_when_prefix_removed(self):
        self.ingest()
        self.config["sources"][0]["issue_numbers"] = []
        source = issue(updated="2026-11-01T15:00:00Z")
        source["title"] = "Withdrawn"
        with patch("observations.importer._api", side_effect=[metadata(), [], source]) as mock:
            coverage = collect(self.config, self.state)
        self.assertTrue(coverage["complete"])
        self.assertEqual(mock.call_count, 3)
        self.assertEqual(latest_observations(self.state)[0]["status"], "rejected")

    def test_removed_source_cannot_silently_reuse_restored_labels(self):
        self.ingest()
        cfg = config("demo/other")
        with patch("observations.importer._api", side_effect=[metadata("demo/other"), [],
                   issue(observation(cfg), repository="demo/other")]):
            coverage = collect(cfg, self.state)
        self.assertFalse(coverage["complete"])
        self.assertIn("removed from config", coverage["errors"][0])

    def test_task_link_changes_reparse_and_quarantine_old_label(self):
        self.ingest()
        self.config["task_links"][0]["pull_requests"] = [5]
        with patch("observations.importer._api", side_effect=[metadata(), [issue()], issue()]):
            coverage = collect(self.config, self.state)
        self.assertTrue(coverage["complete"])
        row = latest_observations(self.state)[0]
        self.assertEqual(row["status"], "conflict")
        self.assertIsNone(row["observation"])


if __name__ == "__main__":
    unittest.main()
