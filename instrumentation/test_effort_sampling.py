"""Sampling integrity, observation conservation and offline review regression tests."""
import copy
from datetime import datetime, timedelta, timezone
from fractions import Fraction
import json
import math
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from effort.common import canonical_hash, format_time, validate_frame
from effort.sampling import (
    _hours,
    draw_sample,
    export_estimator,
    merge_responses,
    render_review,
    response_template,
    validate_responses,
    validate_sample,
)


def fixture_frame():
    start = datetime(2020, 1, 1, 9, tzinfo=timezone.utc)
    predictions = [0.1, 0.2, 0.0, 0.0]
    units = []
    for i, prediction in enumerate(predictions):
        units.append({
            "unit_id": f"unit-{i}", "person_id": "person-a",
            "start": format_time(start + timedelta(minutes=15*i)),
            "end": format_time(start + timedelta(minutes=15*(i+1))),
            "predicted_person_hours": prediction,
            "task_predictions": {"task-a": prediction},
            "unattributed_predicted_person_hours": 0.0,
            "coverage": {"afk_seconds": 900 if i < 3 else 0, "context_seconds": 900 if i < 2 else 0, "unknown_seconds": 900 if i == 3 else 0},
            "flags": ["missing_telemetry"] if i == 3 else [],
        })
    frame = {
        "schema": "observatory.effort-frame/1", "protocol_id": "protocol-1", "frame_id": "four-intervals",
        "config_sha256": "a" * 64, "activity_sha256": "b" * 64,
        "frozen_at": "2020-01-01T10:01:00Z", "frame_complete": True, "synthetic": False, "attended_waiting_counts": False,
        "frames": [{"person_id": "person-a", "start": format_time(start), "end": format_time(start + timedelta(hours=1))}],
        "links": [{"task_id": task, "repository": "example/project", "issue_number": n,
                   "branches": [task], "pull_requests": [], "projects": []} for n, task in enumerate(["task-a", "task-b"], 1)],
        "model": {"id": "activity-proxy-v1", "description": "Synthetic test predictor; no empirical observations"},
        "units": units,
    }
    validate_frame(frame)
    return frame


def observed_response(frame, sample, task="task-a", minutes=10):
    response = response_template(frame, sample)
    for record in response["responses"]:
        record.update(status="observed", allocations=[{"task_id": task, "role": "implementation", "minutes": minutes}],
                      non_work_minutes=15-minutes, reason=None)
    return response


class SamplingTests(unittest.TestCase):
    def setUp(self):
        self.frame = fixture_frame()
        self.sample = draw_sample(self.frame, 2, "01" * 32)

    def test_population_includes_quiet_and_missing_telemetry(self):
        sample = draw_sample(self.frame, 4, "02" * 32)
        self.assertEqual(sample["selected_unit_ids"], ["unit-0", "unit-1", "unit-2", "unit-3"])
        self.assertEqual(sample["inclusion_probability"], 1)
        validate_sample(self.frame, sample)

    def test_production_seed_is_generated_after_frame_validation(self):
        with patch("effort.sampling.secrets.token_hex", return_value="ab" * 32) as generate:
            sample = draw_sample(self.frame, 2)
        generate.assert_called_once_with(32)
        self.assertFalse(sample["synthetic"])
        self.assertEqual(sample["seed"], "ab" * 32)
        self.assertTrue(self.sample["synthetic"])
        validate_sample(self.frame, sample)

    def test_synthetic_frame_stays_synthetic_with_random_sample(self):
        frame = copy.deepcopy(self.frame)
        frame["synthetic"] = True
        sample = draw_sample(frame, 2)
        self.assertTrue(sample["synthetic"])
        sample["synthetic"] = False
        with self.assertRaisesRegex(ValueError, "synthetic marker"):
            validate_sample(frame, sample)

    def test_retained_selection_replays_independent_of_unit_order(self):
        reversed_frame = copy.deepcopy(self.frame)
        reversed_frame["units"].reverse()
        replay = draw_sample(reversed_frame, 2, "01" * 32)
        self.assertEqual(replay["selected_unit_ids"], self.sample["selected_unit_ids"])
        self.assertNotEqual(replay["frame_sha256"], self.sample["frame_sha256"])
        with self.assertRaisesRegex(ValueError, "bind"):
            validate_sample(reversed_frame, self.sample)

    def test_sample_tampering_and_unknown_fields_are_rejected(self):
        cases = []
        changed = copy.deepcopy(self.sample)
        changed["selected_unit_ids"] = ["unit-0", "unit-0"]
        cases.append(changed)
        changed = copy.deepcopy(self.sample)
        changed["inclusion_probability"] = 1.0
        cases.append(changed)
        changed = copy.deepcopy(self.sample)
        changed["design"]["population_size"] = 5
        cases.append(changed)
        changed = copy.deepcopy(self.sample)
        changed["created_at"] = "2020-01-01T09:00:00Z"
        cases.append(changed)
        changed = copy.deepcopy(self.sample)
        changed["extra"] = True
        cases.append(changed)
        for changed in cases:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                validate_sample(self.frame, changed)

    def test_invalid_sample_sizes_and_seeds_are_rejected(self):
        for size in [0, 5, True, 1.5]:
            with self.subTest(size=size), self.assertRaises(ValueError):
                draw_sample(self.frame, size)
        for seed in ["0", "AB"*32, 0, "g"*64]:
            with self.subTest(seed=seed), self.assertRaises(ValueError):
                draw_sample(self.frame, 2, seed)

    def test_templates_default_to_nonresponse_and_absence_is_not_zero(self):
        responses = response_template(self.frame, self.sample)
        self.assertTrue(all(r["status"] == "nonresponse" and r["non_work_minutes"] is None for r in responses["responses"]))
        with self.assertRaisesRegex(ValueError, "nonresponse"):
            export_estimator(self.frame, self.sample, responses)
        responses = observed_response(self.frame, self.sample)
        responses["responses"].pop()
        with self.assertRaisesRegex(ValueError, "nonresponse"):
            export_estimator(self.frame, self.sample, responses)

    def test_explicit_zero_is_an_observation(self):
        responses = observed_response(self.frame, self.sample)
        for record in responses["responses"]:
            record.update(allocations=[], non_work_minutes=15)
        exported = export_estimator(self.frame, self.sample, responses)
        observed = [unit["observation"] for unit in exported["estimation"]["units"] if unit["observation"]["status"] == "observed"]
        self.assertEqual(observed, [{"status": "observed", "person_hours": 0.0}] * 2)

    def test_responses_bind_to_both_frame_and_sample(self):
        responses = observed_response(self.frame, self.sample)
        for name in ["frame_sha256", "sample_sha256"]:
            changed = copy.deepcopy(responses)
            changed[name] = "f" * 64
            with self.assertRaisesRegex(ValueError, "bind"):
                export_estimator(self.frame, self.sample, changed)

    def test_unknown_duplicate_and_unselected_labels_fail(self):
        responses = observed_response(self.frame, self.sample)
        cases = []
        changed = copy.deepcopy(responses)
        changed["responses"][0]["allocations"][0]["task_id"] = "absent"
        cases.append(changed)
        changed = copy.deepcopy(responses)
        changed["responses"][0]["allocations"][0]["role"] = "invented"
        cases.append(changed)
        changed = copy.deepcopy(responses)
        changed["responses"][0]["unit_id"] = "unselected"
        cases.append(changed)
        changed = copy.deepcopy(responses)
        changed["responses"][1]["unit_id"] = changed["responses"][0]["unit_id"]
        cases.append(changed)
        changed = copy.deepcopy(responses)
        changed["responses"][0]["allocations"].append(copy.deepcopy(changed["responses"][0]["allocations"][0]))
        cases.append(changed)
        for changed in cases:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                export_estimator(self.frame, self.sample, changed)

    def test_overcapacity_incomplete_and_nonfinite_allocations_fail(self):
        responses = observed_response(self.frame, self.sample)
        for minutes in [11, 9, -1, float("nan"), float("inf"), True]:
            changed = copy.deepcopy(responses)
            changed["responses"][0]["allocations"][0]["minutes"] = minutes
            with self.subTest(minutes=minutes), self.assertRaises(ValueError):
                export_estimator(self.frame, self.sample, changed)

    def test_multiple_roles_and_targets_conserve_person_time(self):
        responses = observed_response(self.frame, self.sample)
        for record in responses["responses"]:
            record.update(allocations=[
                {"task_id": "task-a", "role": "implementation", "minutes": 6},
                {"task_id": "task-a", "role": "review", "minutes": 3},
                {"task_id": "task-b", "role": "coordination", "minutes": 2},
                {"task_id": "unattributed", "role": "other", "minutes": 1},
            ], non_work_minutes=3)
        expected = {"total": 12, "task-a": 9, "task-b": 2, "unattributed": 1}
        for target, minutes in expected.items():
            exported = export_estimator(self.frame, self.sample, responses, target)
            self.assertEqual(exported["schema"], "observatory.labour-estimate-input/2")
            self.assertEqual(exported["frames"], self.frame["frames"])
            self.assertEqual(len(exported["geometry"]), 4)
            for unit in exported["estimation"]["units"]:
                if unit["observation"]["status"] == "observed":
                    self.assertAlmostEqual(unit["observation"]["person_hours"], minutes/60)
            self.assertTrue(json.loads(exported["estimation"]["predictions"]["provenance"])["synthetic"])

    def test_incomplete_declared_roster_or_window_prevents_total(self):
        frame = copy.deepcopy(self.frame)
        frame["frame_complete"] = False
        sample = draw_sample(frame, 2, "00" * 32)
        with self.assertRaisesRegex(ValueError, "roster"):
            export_estimator(frame, sample, observed_response(frame, sample))

    def test_minute_resolution_and_float_export_never_exceed_observation(self):
        # One second is a repeating decimal number of hours. Its binary value
        # must not accidentally exceed the Rust exact capacity comparison.
        for micros in [1, 1_000_000, 60_000_000, 120_000_000, 900_000_000]:
            hours = _hours(micros)
            self.assertLessEqual(Fraction.from_float(hours), Fraction(micros, 3_600_000_000))
            self.assertLess(float(Fraction(micros, 3_600_000_000)) - hours, 1e-15)
        responses = observed_response(self.frame, self.sample)
        for record in responses["responses"]:
            record["allocations"][0]["minutes"] = 9.9
            record["non_work_minutes"] = 5.1
        self.assertTrue(validate_responses(self.frame, self.sample, responses))

    def test_review_payload_hides_predictions_and_escapes_untrusted_identifiers(self):
        frame = copy.deepcopy(self.frame)
        frame["frame_id"] = '</script><img src=x onerror=alert(1)>'
        sample = draw_sample(frame, 2, "00" * 32)
        document = render_review(frame, sample)
        self.assertNotIn(frame["frame_id"], document)
        self.assertNotIn("predicted_person_hours", document)
        self.assertNotIn("task_predictions", document)
        self.assertIn("default-src 'none'", document)
        self.assertNotRegex(document, r'<script[^>]+src=')
        match = re.search(r'<script id="payload" type="application/json">(.*?)</script>', document, re.S)
        payload = json.loads(match.group(1))
        self.assertEqual(payload["frame_id"], frame["frame_id"])
        self.assertFalse(payload["attended_waiting_counts"])
        self.assertTrue(payload["synthetic"])
        self.assertEqual(len(payload["units"]), 2)
        self.assertTrue(all(r["status"] == "nonresponse" for r in payload["template"]["responses"]))

    def test_person_forms_partition_selected_units_and_merge_without_double_counting(self):
        frame = copy.deepcopy(self.frame)
        second = copy.deepcopy(frame["frames"][0])
        second["person_id"] = "person-b"
        frame["frames"].append(second)
        for original in list(frame["units"]):
            unit = copy.deepcopy(original)
            unit["unit_id"] += "-b"
            unit["person_id"] = "person-b"
            frame["units"].append(unit)
        sample = draw_sample(frame, 8, "ff" * 32)
        forms = [response_template(frame, sample, person) for person in ["person-a", "person-b"]]
        for person, form in zip(["person-a", "person-b"], forms):
            self.assertEqual(len(form["responses"]), 4)
            document = render_review(frame, sample, person)
            payload = json.loads(re.search(r'<script id="payload" type="application/json">(.*?)</script>', document, re.S).group(1))
            self.assertEqual({u["person_id"] for u in payload["units"]}, {person})
            for record in form["responses"]:
                record.update(status="observed", non_work_minutes=15, reason=None)
        merged = merge_responses(frame, sample, forms + [forms[0]])
        self.assertEqual(len(merged["responses"]), 8)
        self.assertEqual(merged["responses"], merge_responses(frame, sample, list(reversed(forms)))["responses"])
        export_estimator(frame, sample, merged)
        with self.assertRaisesRegex(ValueError, "person"):
            render_review(frame, sample, "absent")
        with self.assertRaisesRegex(ValueError, "nonresponse"):
            export_estimator(frame, sample, merge_responses(frame, sample, forms[:1]))

    def test_response_merge_rejects_conflicting_revisions(self):
        first = observed_response(self.frame, self.sample, minutes=10)
        second = observed_response(self.frame, self.sample, minutes=11)
        with self.assertRaisesRegex(ValueError, "conflicting duplicate"):
            merge_responses(self.frame, self.sample, [first, second])
        with self.assertRaises(ValueError):
            merge_responses(self.frame, self.sample, [])

    @unittest.skipUnless(shutil.which("node"), "Node.js is optional for offline review script syntax checking")
    def test_offline_review_script_has_valid_javascript(self):
        document = render_review(self.frame, self.sample)
        scripts = re.findall(r'<script>(.*?)</script>', document, re.S)
        self.assertEqual(len(scripts), 1)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"review.js"
            path.write_text(scripts[0])
            result = subprocess.run([shutil.which("node"), "--check", str(path)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
