"""Task-level leakage, missingness, registry and held-out reuse regressions."""
from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from effort.common import canonical_hash, read_json, write_json
from observations.model import DEFAULT_POLICY, assess, improve, predict, task_key, task_partition


PROTOCOL = "manufactured-model-test-v1"
POLICY = {**DEFAULT_POLICY, "protocol_id": PROTOCOL}


def record(repository, issue_number, minutes, *, synthetic=True, version=1, source_issue=None):
    source_issue = source_issue or 10000 + issue_number
    observation = {
        "schema": "observatory.effort-observation/1", "protocol_id": PROTOCOL,
        "task": {"repository": repository, "issue_number": issue_number, "task_id": f"task-{issue_number}", "pull_requests": []},
        "declared_minutes": minutes, "method": "timer", "coverage": "complete", "scope": "task_total",
        "completed_at": "2026-01-01T12:00:00Z", "synthetic": synthetic,
    }
    return {
        "schema": "observatory.observation-revision/1", "observation_id": canonical_hash([repository, source_issue]),
        "revision_id": canonical_hash([repository, source_issue, version, observation]),
        "source": {"repository": repository, "issue_number": source_issue, "issue_id": source_issue,
                   "url": f"https://github.com/{repository}/issues/{source_issue}", "state": "open",
                   "created_at": "2026-01-01T12:00:00Z", "updated_at": f"2026-01-0{version}T12:00:00Z", "body_sha256": canonical_hash(observation)},
        "observed_at": "2026-02-01T12:00:00Z", "status": "accepted", "observation": observation, "errors": [],
    }


def selected(repository, partition, count, minutes, *, start=1, synthetic=True):
    rows = []
    for issue in range(start, start+10000):
        if task_partition(PROTOCOL, repository, issue) == partition:
            rows.append(record(repository, issue, minutes, synthetic=synthetic))
            if len(rows) == count:
                return rows
    raise AssertionError("partition fixture search exhausted")


def fixture():
    # Explicit manufactured data verify a gate. They are not accuracy evidence.
    return (selected("example/short", "train", 4, 10) + selected("example/long", "train", 4, 90)
            + selected("example/short", "validation", 2, 10) + selected("example/long", "validation", 2, 90))


def changed(row, **changes):
    result = deepcopy(row)
    result["observation"].update(changes)
    result["source"]["updated_at"] = "2026-01-02T12:00:00Z"
    result["revision_id"] = canonical_hash([row["revision_id"], changes])
    result["source"]["body_sha256"] = canonical_hash(result["observation"])
    return result


class ObservationModelTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.state = Path(self.temp.name)
        self.rows = fixture()

    def run_model(self, rows=None, state=None, synthetic=True):
        return improve(self.rows if rows is None else rows, state or self.state, POLICY, synthetic=synthetic)

    def test_promoted_model_is_used_for_future_predictions(self):
        result = self.run_model()
        self.assertEqual(result["decision"], "promoted")
        self.assertEqual(result["counts"], {"eligible": 12, "train": 8, "validation": 4, "unused_validation": 4})
        self.assertEqual(result["metrics"]["baseline_mae"], 40)
        self.assertAlmostEqual(result["metrics"]["candidate_mae"], 120/7)
        model = result["active_model"]
        self.assertEqual(model["target"], "declared_complete_task_total_minutes")
        self.assertAlmostEqual(predict(model, "EXAMPLE/SHORT"), 190/7)
        self.assertEqual(predict(model, "example/new-repository"), 50)
        self.assertIsNone(predict(None, "example/short"))
        frozen = read_json(result["artifact_paths"]["frozen_predictions"])
        self.assertNotIn("declared_minutes", str(frozen))
        self.assertEqual(len(result["consumed_validation_tasks"]), 4)
        self.assertEqual(set(unit["partition"] for unit in model["training_units"]), {"train"})

    def test_assessment_matches_eligibility_without_fitting_or_touching_state(self):
        rows = [*self.rows, changed(self.rows[0], coverage="unknown")]
        with patch("observations.model.write_json", side_effect=AssertionError("assessment cannot publish")), \
             patch("observations.model._save_model", side_effect=AssertionError("assessment cannot fit")), \
             patch("observations.model.predict", side_effect=AssertionError("assessment cannot predict")):
            result = assess(rows, POLICY, synthetic=True)
        self.assertEqual(result["decision"], "observation_only")
        self.assertIsNone(result["active_model"])
        self.assertIsNone(result["metrics"])
        self.assertIsNone(result["counts"]["unused_validation"])
        self.assertEqual(result["consumed_validation_tasks"], [])
        self.assertTrue(all(path is None for path in result["artifact_paths"].values()))
        self.assertEqual(list(self.state.iterdir()), [])
        learning = self.run_model(rows)
        self.assertEqual(result["exclusions"], learning["exclusions"])
        for field in ("eligible", "train", "validation"):
            self.assertEqual(result["counts"][field], learning["counts"][field])

    def test_assessment_preserves_live_synthetic_boundary(self):
        with self.assertRaisesRegex(ValueError, "must not be mixed"):
            assess(self.rows, POLICY, synthetic=False)

    def test_delivery_replay_reordering_and_duplicates_are_idempotent(self):
        first = self.run_model()
        delivered = deepcopy(self.rows[::-1] + self.rows)
        for row in delivered:
            row["observed_at"] = "2026-02-03T00:00:00Z"
        second = self.run_model(delivered)
        self.assertEqual(first, second)
        self.assertEqual(len(list((self.state/"model-learning/synthetic/evaluations").glob("*.json"))), 1)

    def test_same_task_stays_in_partition_across_alias_pr_and_revision_changes(self):
        row = self.rows[0]
        task = row["observation"]["task"]
        self.assertEqual(task_key(PROTOCOL, task["repository"], task["issue_number"]), task_key(PROTOCOL, task["repository"].upper(), task["issue_number"]))
        revised = changed(row, task={**task, "task_id": "another-alias", "pull_requests": [99, 100]})
        result = self.run_model([*self.rows, revised])
        self.assertEqual(result["counts"]["eligible"], 12)
        self.assertEqual(result["counts"]["train"], 8)
        self.assertNotIn(row["revision_id"], [unit["revision_id"] for unit in result["active_model"]["training_units"]])

    def test_validation_labels_never_change_fitted_parameters(self):
        first = self.run_model()
        labels_changed = [changed(row, declared_minutes=500) if row["observation"]["task"] and task_partition(PROTOCOL, row["observation"]["task"]["repository"], row["observation"]["task"]["issue_number"]) == "validation" else row for row in self.rows]
        second = self.run_model(labels_changed, state=self.state/"independent")
        self.assertEqual(first["candidate_model_id"], second["candidate_model_id"])
        self.assertEqual(second["decision"], "rejected")
        self.assertIsNone(second["active_model"])

    def test_latest_invalid_edit_never_falls_back_to_previous_accepted_label(self):
        invalid = changed(self.rows[0], declared_minutes=None)
        invalid.update(status="rejected", observation=None, errors=["malformed payload"])
        result = self.run_model([*self.rows, invalid])
        self.assertEqual(result["decision"], "insufficient_data")
        self.assertEqual(result["counts"]["train"], 7)
        self.assertIn("latest_revision_not_accepted", [row["reason"] for row in result["exclusions"]])

    def test_conflicting_same_timestamp_revisions_are_quarantined(self):
        competing = changed(self.rows[0], declared_minutes=300)
        competing["source"]["updated_at"] = self.rows[0]["source"]["updated_at"]
        result = self.run_model([*self.rows, competing])
        self.assertEqual(result["counts"]["train"], 7)
        self.assertEqual(result["decision"], "insufficient_data")

    def test_multiple_source_issues_for_one_task_are_not_summed_or_averaged(self):
        task = self.rows[0]["observation"]["task"]
        for minutes in (10, 500):
            with self.subTest(minutes=minutes):
                duplicate = record(task["repository"], task["issue_number"], minutes, source_issue=80000)
                result = self.run_model([*self.rows, duplicate], state=self.state/str(minutes))
                self.assertEqual(result["counts"]["eligible"], 11)
                self.assertEqual(len(result["exclusions"]), 2)
                self.assertTrue(all("multiple_observation_issues_for_task" in row["reason"] for row in result["exclusions"]))

    def test_closed_observation_is_eligible_and_closure_does_not_refresh_holdout(self):
        first = self.run_model()
        updated = deepcopy(self.rows)
        for row in updated:
            row["source"].update(state="closed", updated_at="2026-01-02T12:00:00Z")
            row["revision_id"] = canonical_hash([row["revision_id"], "closed"])
        second = self.run_model(updated)
        self.assertEqual(second["counts"]["eligible"], 12)
        self.assertEqual(second["counts"]["unused_validation"], 0)
        self.assertEqual(second["decision"], "validation_exhausted")
        self.assertEqual(first["active_model"], second["active_model"])
        self.assertEqual(second["retirement_reasons"], [])

    def test_unknown_metadata_keeps_real_three_minutes_out_of_fitting(self):
        row = record("emmett08/engarg", 49, 3, synthetic=False, source_issue=51)
        row["observation"].update(method="unknown", coverage="unknown", scope="task_total")
        result = self.run_model([row], synthetic=False)
        self.assertEqual(row["observation"]["declared_minutes"], 3)
        self.assertEqual(result["counts"]["eligible"], 0)
        self.assertEqual(result["decision"], "insufficient_data")
        self.assertIsNone(result["active_model"])
        self.assertIn("unknown_measurement_method", result["exclusions"][0]["reason"])

    def test_missing_boolean_negative_and_nonfinite_targets_are_not_zero(self):
        for minutes in (None, True, -1, float("nan"), float("inf")):
            with self.subTest(minutes=minutes):
                row = deepcopy(self.rows[0])
                row["observation"]["declared_minutes"] = minutes
                # Non-finite input must fail strict canonical encoding, not enter
                # an evaluation or become a zero. Finite invalid labels exclude.
                if isinstance(minutes, float):
                    with self.assertRaises(ValueError):
                        self.run_model([row], state=self.state/str(minutes))
                else:
                    result = self.run_model([row], state=self.state/str(minutes))
                    self.assertEqual(result["counts"]["eligible"], 0)

    def test_known_zero_is_eligible_but_zero_baseline_does_not_promote(self):
        rows = [changed(row, declared_minutes=0) for row in self.rows]
        result = self.run_model(rows)
        self.assertEqual(result["counts"]["eligible"], 12)
        self.assertEqual(result["decision"], "rejected")
        self.assertEqual(result["metrics"]["baseline_mae"], 0)
        self.assertIsNone(result["metrics"]["relative_improvement"])

    def test_large_finite_targets_do_not_overflow_model_arithmetic(self):
        result = self.run_model([changed(row, declared_minutes=1e308) for row in self.rows])
        self.assertEqual(result["counts"]["eligible"], 12)
        self.assertEqual(result["decision"], "rejected")
        self.assertEqual(result["metrics"]["baseline_mae"], 0)

    def test_partial_unknown_and_non_task_total_observations_are_excluded(self):
        for changes in ({"coverage": "partial"}, {"scope": "partial_task"}, {"method": "unknown"}):
            with self.subTest(changes=changes):
                result = self.run_model([changed(self.rows[0], **changes)], state=self.state/canonical_hash(changes))
                self.assertEqual(result["counts"]["eligible"], 0)

    def test_minimum_train_and_validation_guards(self):
        for drop in (0, -1):
            with self.subTest(drop=drop):
                rows = self.rows[:]
                rows.pop(drop)
                result = self.run_model(rows, state=self.state/str(drop))
                self.assertEqual(result["decision"], "insufficient_data")
                self.assertIsNone(result["metrics"])
                self.assertEqual(result["consumed_validation_tasks"], [])

    def test_training_or_validation_corrections_retire_active_model(self):
        for index in (0, -1):
            with self.subTest(index=index):
                state = self.state/str(index)
                first = self.run_model(state=state)
                rows = self.rows[:]
                rows[index] = changed(rows[index], declared_minutes=777)
                result = self.run_model(rows, state=state)
                self.assertEqual(result["decision"], "validation_exhausted")
                self.assertEqual(result["retired_model_id"], first["active_model"]["model_id"])
                self.assertIsNone(result["active_model"])
                self.assertEqual(result["counts"]["unused_validation"], 0)
                self.assertEqual(self.run_model(rows, state=state), result)
                # Replaying a previously evaluated input cannot reactivate it.
                replay = self.run_model(state=state)
                self.assertIsNone(replay["active_model"])
                self.assertIsNone(replay["current_active_model_id"])
                self.assertEqual(replay["active_model_id"], first["active_model"]["model_id"])

    def test_invalidated_training_total_retires_incumbent_without_erasing_artifact(self):
        first = self.run_model()
        rows = self.rows[:]
        rows[0] = changed(rows[0], coverage="unknown")
        result = self.run_model(rows)
        self.assertEqual(result["decision"], "insufficient_data")
        self.assertIsNone(result["active_model"])
        self.assertTrue(Path(first["artifact_paths"]["candidate_model"]).exists())

    def test_fresh_validation_checks_incumbent_non_regression(self):
        first = self.run_model()
        # Add (do not edit) training labels that pull repository medians towards
        # each other. The original model's labels remain valid.
        extra = selected("example/short", "train", 8, 30, start=500) + selected("example/long", "train", 8, 70, start=500)
        panel = selected("example/short", "validation", 2, 10, start=500) + selected("example/long", "validation", 2, 90, start=500)
        result = self.run_model([*self.rows, *extra, *panel])
        self.assertEqual(result["decision"], "rejected")
        self.assertTrue(result["gates"]["absolute_improvement"])
        self.assertTrue(result["gates"]["relative_improvement"])
        self.assertFalse(result["gates"]["no_incumbent_regression"])
        self.assertEqual(result["active_model"], first["active_model"])
        self.assertEqual(len(result["consumed_validation_tasks"]), 4)

    def test_rejected_panel_cannot_be_reused_after_training_changes(self):
        rows = [changed(row, declared_minutes=50) if row in self.rows[8:] else row for row in self.rows]
        first = self.run_model(rows)
        self.assertEqual(first["decision"], "rejected")
        newer = [*rows, *selected("example/short", "train", 1, 12, start=500)]
        second = self.run_model(newer)
        self.assertEqual(second["decision"], "validation_exhausted")
        self.assertIsNone(second["metrics"])

    def test_fixed_policy_and_protocol_bind_the_namespace(self):
        self.run_model()
        with self.assertRaisesRegex(ValueError, "settings are fixed"):
            improve(self.rows, self.state, {**POLICY, "min_train_tasks": 2}, synthetic=True)
        with self.assertRaises(ValueError):
            improve([], self.state, {**POLICY, "protocol_id": "another-protocol"}, synthetic=True)

    def test_live_synthetic_namespaces_separate_and_mixed_rows_rejected(self):
        self.run_model()
        live = deepcopy(self.rows)
        for row in live:
            row["observation"]["synthetic"] = False
        result = self.run_model(live, synthetic=False)
        self.assertFalse(result["active_model"]["synthetic"])
        self.assertIn("/live/", result["artifact_paths"]["evaluation"])
        with self.assertRaisesRegex(ValueError, "must not be mixed"):
            self.run_model([*self.rows, live[0]])

    def test_revision_identity_conflict_fails_closed(self):
        row = deepcopy(self.rows[0])
        row["observation"]["declared_minutes"] = 999
        with self.assertRaisesRegex(ValueError, "conflicting contents"):
            self.run_model([*self.rows, row])

    def test_concurrent_delivery_commits_only_one_evaluation(self):
        with ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(lambda _: self.run_model(), range(4)))
        self.assertTrue(all(result == results[0] for result in results))
        self.assertEqual(len(list((self.state/"model-learning/synthetic/evaluations").glob("*.json"))), 1)

    def test_interrupted_publication_cannot_activate_or_consume_partial_evaluation(self):
        def crash_before_commit(path, value):
            if Path(path).parent.name == "evaluations":
                raise OSError("manufactured storage interruption")
            write_json(path, value)
        with patch("observations.model.write_json", side_effect=crash_before_commit):
            with self.assertRaises(OSError):
                self.run_model()
        self.assertEqual(list((self.state/"model-learning/synthetic/evaluations").glob("*.json")), [])
        result = self.run_model()
        self.assertEqual(result["decision"], "promoted")
        self.assertEqual(result["sequence"], 1)

    def test_evaluation_tampering_fails_closed(self):
        result = self.run_model()
        path = Path(result["artifact_paths"]["evaluation"])
        path.write_text(path.read_text().replace('"decision": "promoted"', '"decision": "rejected"'))
        with self.assertRaisesRegex(ValueError, "corrupt"):
            self.run_model()


if __name__ == "__main__":
    unittest.main()
