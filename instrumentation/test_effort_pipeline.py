"""Adversarial frame, attribution and immutable CLI lifecycle checks."""
from copy import deepcopy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from effort.activity import collect_export
from effort.batch import prepare_batch
from effort.common import canonical_hash, parse_time, read_json, validate_config, validate_frame, write_json
from effort.frame import build_frame, resolve_task
from effort.report import build_report, markdown
from effort.sampling import draw_sample, response_template

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "instrumentation/effort.py"


def fixture():
    config = read_json(ROOT / "examples/effort/config.json")
    raw = read_json(ROOT / "examples/effort/activitywatch-export.json")
    return config, collect_export(config, raw)


class EffortPipelineTests(unittest.TestCase):
    def test_person_timeline_keeps_quiet_periods_and_unions_devices(self):
        config, activity = fixture()
        frame = build_frame(config, activity)
        self.assertEqual(len(frame["units"]), 8)
        first = next(u for u in frame["units"] if u["person_id"] == "participant-a" and parse_time(u["start"]).minute == 0)
        self.assertEqual(first["predicted_person_hours"], .25)
        self.assertEqual(sum(first["task_predictions"].values()), .25)
        quiet = next(u for u in frame["units"] if u["person_id"] == "participant-a" and parse_time(u["start"]).minute == 15)
        self.assertEqual(quiet["predicted_person_hours"], 0)
        missing = next(u for u in frame["units"] if u["person_id"] == "participant-b" and parse_time(u["start"]).minute == 45)
        self.assertEqual(missing["coverage"]["unknown_seconds"], 900)
        self.assertIn("telemetry_gap", missing["flags"])

    def test_afk_and_context_are_paired_on_same_device(self):
        config, activity = fixture()
        for source in activity["sources"]:
            if source["person_id"] != "participant-a":
                continue
            if (source["device_id"] == "laptop-a" and source["kind"] == "context") or (source["device_id"] == "desktop-a" and source["kind"] == "afk"):
                source["events"] = []
        frame = build_frame(config, activity)
        self.assertEqual(sum(u["predicted_person_hours"] for u in frame["units"] if u["person_id"] == "participant-a"), 0)

    def test_conflicting_active_device_tasks_are_unattributed(self):
        config, activity = fixture()
        for source in activity["sources"]:
            if source["device_id"] == "desktop-a" and source["kind"] == "context":
                for event in source["events"]:
                    event["data"] = {"project": "beta", "category": "coding"}
        frame = build_frame(config, activity)
        first = next(u for u in frame["units"] if u["person_id"] == "participant-a" and parse_time(u["start"]).minute == 0)
        self.assertEqual(first["predicted_person_hours"], .25)
        self.assertEqual(first["task_predictions"], {})
        self.assertEqual(first["unattributed_predicted_person_hours"], .25)

    def test_contradictory_afk_is_flagged_and_not_arbitrarily_chosen(self):
        config, activity = fixture()
        for source in activity["sources"]:
            if source["device_id"] == "desktop-a":
                source["events"] = []
            if source["device_id"] == "laptop-a" and source["kind"] == "afk":
                event = deepcopy(source["events"][0])
                event["event_id"] += "-conflict"
                event["data"]["status"] = "afk"
                source["events"].append(event)
        first = build_frame(config, activity)["units"][0]
        self.assertEqual(first["predicted_person_hours"], 0)
        self.assertIn("conflicting_afk", first["flags"])

    def test_waiting_convention_changes_building_proxy(self):
        config, activity = fixture()
        old = build_frame(config, activity)
        config["attended_waiting_counts"] = True
        activity["config_sha256"] = canonical_hash(config)
        new = build_frame(config, activity)
        self.assertGreater(sum(u["predicted_person_hours"] for u in new["units"]), sum(u["predicted_person_hours"] for u in old["units"]))

    def test_attribution_uses_all_constraints_and_preserves_ambiguity(self):
        config, _ = fixture()
        links = config["links"]
        self.assertEqual(resolve_task({"project": "alpha"}, links), "issue-alpha")
        self.assertIsNone(resolve_task({"project": "alpha", "linkage_unknown": True}, links))
        self.assertIsNone(resolve_task({"issue_url": "https://github.com/example/alpha/issues/101", "branch": "issue-202-check"}, links))
        links[0]["pull_requests"].append(22)
        links[1]["pull_request_urls"] = ["https://github.com/example/alpha/pull/22"]
        self.assertIsNone(resolve_task({"pr_url": "https://github.com/example/alpha/pull/22"}, links))
        links[0]["branch_links"] = [{"repository": "fork/project", "branch": "issue-101"}]
        self.assertEqual(resolve_task({"repository": "fork/project", "branch": "issue-101"}, links), "issue-alpha")

    def test_duplicate_bucket_ownership_is_rejected(self):
        config, _ = fixture()
        config["sources"][1]["afk_bucket"] = config["sources"][0]["afk_bucket"]
        with self.assertRaises(ValueError):
            validate_config(config)

    def test_completed_windows_precision_and_geometry_are_enforced(self):
        with self.assertRaises(ValueError):
            parse_time("2026-11-01T09:00:00.0000001Z")
        config, activity = fixture()
        frame = build_frame(config, activity)
        frame["units"][1]["start"] = frame["units"][0]["start"]
        with self.assertRaises(ValueError):
            validate_frame(frame)
        config["frames"][0]["end"] = "2026-11-02T10:00:00Z"
        config["interval_minutes"] = True
        with self.assertRaises(ValueError):
            validate_config(config)

    def test_proxy_and_missing_sample_never_become_measured_zero(self):
        config, activity = fixture()
        frame = build_frame(config, activity)
        proxy = build_report(frame)
        self.assertEqual(proxy["mode"], "activity_proxy")
        self.assertIsNone(proxy["estimate"])
        self.assertIn("not observed person-hours", markdown(proxy))
        sample = draw_sample(frame, 2, "0"*64)
        missing = build_report(frame, sample=sample, response_documents=[response_template(frame, sample)])
        self.assertEqual(missing["mode"], "estimate_unavailable")
        self.assertEqual(missing["sampling"]["missing_count"], 2)
        self.assertIsNone(missing["estimate"])
        self.assertTrue(missing["synthetic"])

    def test_immutable_json_and_duplicate_keys(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"record.json"
            write_json(path, {"value": 1})
            write_json(path, {"value": 1})
            with self.assertRaises(ValueError):
                write_json(path, {"value": 2})
            for content in ('{"a":1,"a":2}', '{"a":NaN}', '{"a":1e999}'):
                path.write_text(content)
                with self.assertRaises(ValueError):
                    read_json(path)

    def test_cli_draw_registry_prevents_accidental_resampling_and_scope_leak(self):
        config, activity = fixture()
        frame = build_frame(config, activity)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_json(root/"frame.json", frame)
            def command(*args):
                return subprocess.run([sys.executable, str(CLI), *map(str, args)], capture_output=True, text=True)
            first = command("sample", "--frame", root/"frame.json", "--size", 2, "--output", root/"sample.json")
            self.assertEqual(first.returncode, 0, first.stderr)
            second = command("sample", "--frame", root/"frame.json", "--size", 2, "--output", root/"sample-copy.json")
            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual((root/"sample.json").read_bytes(), (root/"sample-copy.json").read_bytes())
            redraw = command("sample", "--frame", root/"frame.json", "--size", 3, "--output", root/"redraw.json")
            self.assertNotEqual(redraw.returncode, 0)
            self.assertFalse((root/"redraw.json").exists())
            review = command("review", "--frame", root/"frame.json", "--sample", root/"sample.json", "--output", root/"review.html")
            self.assertNotEqual(review.returncode, 0)
            review = command("review", "--frame", root/"frame.json", "--sample", root/"sample.json", "--person", "participant-a", "--output", root/"review.html")
            self.assertEqual(review.returncode, 0, review.stderr)

    def test_daily_prepare_replays_same_frozen_draw_and_refuses_changed_configuration(self):
        config, _ = fixture()
        raw = read_json(ROOT/"examples/effort/activitywatch-export.json")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prepare_batch(config, root, "2026-11-01", 3, export_data=raw)
            before = (root/"sample.json").read_bytes()
            prepare_batch(config, root, "2026-11-01", 3, export_data=raw)
            self.assertEqual(before, (root/"sample.json").read_bytes())
            frame = read_json(root/"frame.json")
            self.assertTrue(frame["frame_id"].endswith("@2026-11-01"))
            self.assertEqual(len(read_json(root/"review-index.json")["reviews"]), 2)
            self.assertEqual(read_json(root/"proxy.json")["mode"], "activity_proxy")
            with self.assertRaises(ValueError):
                prepare_batch(config, root, "2026-11-01", 4, export_data=raw)
            changed = deepcopy(config)
            changed["attended_waiting_counts"] = not changed["attended_waiting_counts"]
            with self.assertRaises(ValueError):
                prepare_batch(changed, root, "2026-11-01", 3, export_data=raw)


if __name__ == "__main__":
    unittest.main()
