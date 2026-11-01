"""ActivityWatch boundary, completeness and allowlisting behaviour."""

import contextlib
import copy
import datetime as dt
import http.server
import json
import threading
import unittest
import urllib.error
import urllib.parse
from unittest import mock

from effort import activity
from effort.common import canonical_hash, parse_time


SECRET = "private-title-path-prompt-token-never-retain"


def config():
    return {
        "schema": "observatory.effort-config/1", "protocol_id": "protocol-1", "frame_id": "frame-1",
        "frames": [{"person_id": "person-1", "start": "2026-11-01T09:00:00Z", "end": "2026-11-01T10:00:00Z"}],
        "interval_minutes": 15,
        "sources": [{"person_id": "person-1", "device_id": "device-1", "afk_bucket": "afk", "context_buckets": ["editor"]}],
        "links": [{"task_id": "task-1", "repository": "example/repo", "issue_number": 1,
                   "branches": ["issue-1"], "pull_requests": [2], "projects": ["project-1"]}],
        "frame_complete": True, "attended_waiting_counts": False,
    }


def event(start="2026-11-01T09:00:00Z", duration=900, data=None, event_id=1):
    result = {"timestamp": start, "duration": duration,
              "data": data if data is not None else {"status": "not-afk"}}
    if event_id is not None:
        result["id"] = event_id
    return result


def export(afk=None, editor=None):
    return {"buckets": {
        "afk": {"hostname": SECRET, "events": [event()] if afk is None else afk},
        "editor": {"hostname": SECRET, "events": [event(data={"project": "project-1", "branch": "issue-1"})] if editor is None else editor},
        "unrequested-secret-bucket": {"events": [event(data={"title": SECRET})]},
    }}


@contextlib.contextmanager
def server(callback):
    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def do_GET(self):
            status, headers, payload = callback(self.path)
            self.send_response(status)
            for key, value in headers.items():
                self.send_header(key, value)
            self.end_headers()
            self.wfile.write(payload)

    httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{httpd.server_port}"
    finally:
        httpd.shutdown()
        httpd.server_close()
        thread.join(timeout=2)


class ActivityCollectorTests(unittest.TestCase):
    def test_export_sanitises_metadata_and_binds_config(self):
        raw = export(editor=[event(data={
            "project": "project-1", "branch": "issue-1", "repository": "example/repo",
            "title": SECRET, "file": "/home/" + SECRET, "environment": {"TOKEN": SECRET},
            "category": "coding", "url": "https://github.com/example/repo/pull/2/files?token=" + SECRET,
        })])
        result = activity.collect_export(config(), raw)
        self.assertNotIn(SECRET, json.dumps(result))
        self.assertEqual(result["config_sha256"], canonical_hash(config()))
        self.assertEqual(len(result["sources"]), 2)
        safe = result["sources"][1]["events"][0]["data"]
        self.assertEqual(safe["pr_url"], "https://github.com/example/repo/pull/2")
        self.assertEqual(safe["category"], "coding")
        self.assertEqual(set(safe), {"project", "branch", "repository", "category", "pr_url"})

    def test_unknown_linkage_is_not_erased_into_a_false_project_match(self):
        for field, value in [("branch", SECRET), ("repository", SECRET), ("project", SECRET),
                             ("url", "https://example.net/" + SECRET), ("category", SECRET)]:
            with self.subTest(field=field):
                raw = export(editor=[event(data={"project": "project-1", field: value})])
                result = activity.collect_export(config(), raw)
                safe = result["sources"][1]["events"][0]["data"]
                self.assertTrue(safe["linkage_unknown"])
                self.assertNotIn(SECRET, json.dumps(result))

    def test_explicit_cross_repository_branch_and_pr_links_are_allowlisted(self):
        cfg = copy.deepcopy(config())
        cfg["links"][0]["branch_links"] = [{"repository": "example/other", "branch": "shared-api"}]
        cfg["links"][0]["pull_request_urls"] = ["https://github.com/example/other/pull/7"]
        raw = export(editor=[event(data={"repository": "example/other", "branch": "shared-api",
                                         "url": "https://github.com/example/other/pull/7/files?private=" + SECRET})])
        result = activity.collect_export(cfg, raw)
        safe = result["sources"][1]["events"][0]["data"]
        self.assertEqual(safe, {"repository": "example/other", "branch": "shared-api",
                                "pr_url": "https://github.com/example/other/pull/7"})
        self.assertNotIn(SECRET, json.dumps(result))

    def test_github_urls_must_be_exact_known_origins_and_targets(self):
        for url in ["https://github.com.evil.invalid/example/repo/issues/1",
                    "https://user:password@github.com/example/repo/issues/1",
                    "http://github.com/example/repo/issues/1",
                    "https://github.com/example/repo/issues/99",
                    "https://github.com/example/repo/pull/2/unknown"]:
            result = activity.collect_export(config(), export(editor=[event(data={"url": url})]))
            self.assertEqual(result["sources"][1]["events"][0]["data"], {"linkage_unknown": True})

    def test_unknown_desktop_title_is_not_parsed_as_task_evidence(self):
        raw = export(editor=[event(data={"title": "issue-1 project-1 " + SECRET, "app": "VSCode"})])
        safe = activity.collect_export(config(), raw)["sources"][1]["events"][0]["data"]
        self.assertEqual(safe, {})

    def test_missing_bucket_and_empty_bucket_remain_distinct(self):
        raw = export(afk=[])
        del raw["buckets"]["editor"]
        sources = activity.collect_export(config(), raw)["sources"]
        self.assertEqual(sources[0]["status"], "retrieved")
        self.assertEqual(sources[0]["events"], [])
        self.assertEqual(sources[1]["status"], "missing")
        self.assertEqual(sources[1]["error"], "bucket_missing")

    def test_malformed_event_retains_other_events_and_marks_partial(self):
        bad_values = [None, {}, event(duration=-1), event(duration=True), event(duration=float("nan")),
                      event(duration=float("inf")), event(start="yesterday"), event(data={"status": "unknown"}),
                      event(event_id="secret-id"), event(data={"status": []})]
        for bad in bad_values:
            with self.subTest(bad=bad):
                source = activity.collect_export(config(), export(afk=[event(), bad]))["sources"][0]
                self.assertEqual(source["status"], "partial")
                self.assertEqual(source["error"], "malformed_event")
                self.assertEqual(len(source["events"]), 1)

    def test_zero_duration_is_not_padded_and_spans_are_clipped(self):
        raw = export(afk=[event(duration=0), event("2026-11-01T08:55:00Z", 600, event_id=2),
                          event("2026-11-01T09:55:00Z", 600, event_id=3),
                          event("2026-11-01T10:00:00Z", 30, event_id=4)])
        events = activity.collect_export(config(), raw)["sources"][0]["events"]
        self.assertEqual(len(events), 2)
        self.assertEqual(parse_time(events[0]["start"]), parse_time("2026-11-01T09:00:00Z"))
        self.assertEqual(parse_time(events[1]["end"]), parse_time("2026-11-01T10:00:00Z"))

    def test_exports_without_ids_have_stable_retained_content_ids(self):
        raw = export(afk=[event(event_id=None), event(event_id=None)])
        first = activity.collect_export(config(), raw)["sources"][0]
        second = activity.collect_export(config(), raw)["sources"][0]
        self.assertEqual(len(first["events"]), 1)
        self.assertEqual(first, second)
        self.assertTrue(first["events"][0]["event_id"].startswith("content-"))

    def test_conflicting_ids_do_not_keep_an_arbitrary_version(self):
        raw = export(afk=[event(), event(data={"status": "afk"})])
        source = activity.collect_export(config(), raw)["sources"][0]
        self.assertEqual(source["error"], "conflicting_event_id")
        self.assertEqual(source["events"], [])

    def test_export_limit_is_explicit(self):
        with mock.patch.object(activity, "MAX_EVENTS", 1):
            result = activity.collect_export(config(), export(afk=[event(), event(event_id=2)]))
        self.assertEqual(result["sources"][0]["error"], "event_limit")

    def test_export_shape_is_checked(self):
        for invalid in [None, [], {}, {"buckets": []}]:
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                activity.collect_export(config(), invalid)

    def test_server_origin_rejects_remote_dns_credentials_and_paths(self):
        for url in ["https://127.0.0.1:5600", "http://localhost:5600", "http://127.0.0.1.evil.invalid",
                    "http://example.com", "http://192.168.1.1", "http://user:secret@127.0.0.1",
                    "http://127.0.0.1/path", "http://127.0.0.1/?q=secret", "http://127.0.0.1/#secret",
                    "http://127.0.0.1:0", "http://[::ffff:192.168.1.1]:5600"]:
            with self.subTest(url=url), self.assertRaises(ValueError):
                activity._server_origin(url)
        self.assertEqual(activity._server_origin("http://127.0.0.1:5600/"), "http://127.0.0.1:5600")
        self.assertEqual(activity._server_origin("http://[::1]:5600"), "http://[::1]:5600")

    def test_live_reads_only_selected_buckets_and_rejects_redirects(self):
        paths = []

        def reply(path):
            paths.append(path)
            if "/afk/" in path:
                return 200, {}, json.dumps([event()]).encode()
            return 302, {"Location": "/secret-destination"}, b""

        with server(reply) as origin:
            result = activity.collect_live(config(), origin)
        self.assertEqual(len(paths), 2)
        self.assertTrue(all("/api/0/buckets/" in path for path in paths))
        self.assertEqual(result["sources"][0]["status"], "retrieved")
        self.assertEqual(result["sources"][1]["status"], "partial")
        self.assertEqual(result["sources"][1]["error"], "http_error")

    def test_live_strict_json_and_response_limit(self):
        for payload in [b'[{"id":1,"id":2}]', b'[NaN]', b"x" * 101]:
            with self.subTest(payload=payload), server(lambda _p: (200, {}, payload)) as origin:
                with mock.patch.object(activity, "MAX_RESPONSE_BYTES", 100):
                    result = activity.collect_live(config(), origin)
                self.assertTrue(all(s["status"] == "partial" for s in result["sources"]))
                self.assertTrue(all(s["error"] == "malformed_response" for s in result["sources"]))

    def test_live_limit_expansion_keeps_ties_and_clipped_events_once(self):
        original = [event("2026-11-01T09:00:00Z", 3600, event_id=1),
                    event("2026-11-01T09:05:00Z", 60, event_id=2),
                    event("2026-11-01T09:40:00Z", 60, event_id=3)]
        calls = []

        def fetch(_opener, url):
            calls.append(url)
            if "/editor/" in url:
                return []
            query = urllib.parse.parse_qs(urllib.parse.urlsplit(url).query)
            lo, hi = parse_time(query["start"][0]), parse_time(query["end"][0])
            found = [item for item in original
                     if parse_time(item["timestamp"]) < hi
                     and parse_time(item["timestamp"]) + dt.timedelta(seconds=item["duration"]) > lo]
            return found[:int(query["limit"][0])]

        with mock.patch.object(activity, "PAGE_SIZE", 2), mock.patch.object(activity, "_get_json", fetch):
            result = activity.collect_live(config())
        self.assertEqual(len(calls), 3)
        afk_queries = [urllib.parse.parse_qs(urllib.parse.urlsplit(url).query) for url in calls if "/afk/" in url]
        self.assertEqual([q["limit"][0] for q in afk_queries], ["3", "5"])
        self.assertEqual(afk_queries[0]["start"], afk_queries[1]["start"])
        self.assertEqual(afk_queries[0]["end"], afk_queries[1]["end"])
        self.assertEqual(result["sources"][0]["status"], "retrieved")
        self.assertEqual(len(result["sources"][0]["events"]), 3)

    def test_event_and_query_budgets_remain_partial(self):
        def fetch(_opener, url):
            if "/editor/" in url:
                return []
            return [event(event_id=1), event(event_id=2), event(event_id=3)]

        with mock.patch.object(activity, "PAGE_SIZE", 2), mock.patch.object(activity, "MAX_REQUESTS", 1), \
                mock.patch.object(activity, "_get_json", fetch):
            source = activity.collect_live(config())["sources"][0]
        self.assertEqual(source["status"], "partial")
        self.assertEqual(source["error"], "request_limit")
        with mock.patch.object(activity, "PAGE_SIZE", 2), mock.patch.object(activity, "MAX_EVENTS", 2), \
                mock.patch.object(activity, "_get_json", fetch):
            source = activity.collect_live(config())["sources"][0]
        self.assertEqual(source["error"], "event_limit")

    def test_later_page_failure_preserves_partial_evidence(self):
        calls = 0

        def fetch(_opener, url):
            nonlocal calls
            calls += 1
            if calls == 1:
                return [event(event_id=1), event(event_id=2), event(event_id=3)]
            raise urllib.error.URLError(SECRET)

        with mock.patch.object(activity, "PAGE_SIZE", 2), mock.patch.object(activity, "_get_json", fetch):
            result = activity.collect_live(config())
        self.assertEqual(result["sources"][0]["status"], "partial")
        self.assertEqual(len(result["sources"][0]["events"]), 3)
        self.assertEqual(result["sources"][1]["status"], "missing")
        self.assertNotIn(SECRET, json.dumps(result))


class ActivityMergeTests(unittest.TestCase):
    def collections(self):
        cfg = config()
        cfg["sources"].append({"person_id": "person-1", "device_id": "device-2",
                               "afk_bucket": "afk-other", "context_buckets": ["editor-other"]})
        first_export = export()
        second_export = {"buckets": {
            "afk-other": {"events": [event()]},
            "editor-other": {"events": [event(data={"project": "project-1", "branch": "issue-1"})]},
        }}
        first = activity.collect_export(cfg, first_export)
        second = activity.collect_export(cfg, second_export)
        first["collected_at"] = "2026-11-01T10:01:00Z"
        second["collected_at"] = "2026-11-01T10:02:00Z"
        return cfg, first, second

    def test_combines_disjoint_devices_with_provenance_and_max_acquisition(self):
        cfg, first, second = self.collections()
        merged = activity.merge_activity(cfg, [first, second])
        self.assertEqual(len(merged["sources"]), 4)
        self.assertTrue(all(source["status"] == "retrieved" for source in merged["sources"]))
        self.assertEqual(merged["merged_from"], sorted([canonical_hash(first), canonical_hash(second)]))
        self.assertEqual(parse_time(merged["collected_at"]), parse_time(second["collected_at"]))
        self.assertEqual(merged, activity.merge_activity(cfg, [second, first]))
        self.assertEqual(merged, activity.merge_activity(cfg, [first, second, first]))
        merged["sources"][0]["events"].clear()
        self.assertEqual(len(first["sources"][0]["events"]), 1)

    def test_conflicting_nonmissing_records_are_not_unioned_or_latest_wins(self):
        cfg, first, _second = self.collections()
        for change in ("event", "status", "empty"):
            with self.subTest(change=change):
                other = copy.deepcopy(first)
                other["collected_at"] = "2026-11-01T10:03:00Z"
                if change == "event":
                    other["sources"][0]["events"][0]["data"]["status"] = "afk"
                elif change == "status":
                    other["sources"][0].update(status="partial", error="request_limit")
                else:
                    other["sources"][0]["events"] = []
                with self.assertRaisesRegex(ValueError, "conflicting nonmissing"):
                    activity.merge_activity(cfg, [first, other])

    def test_partial_remains_partial_and_missing_remains_missing(self):
        cfg, first, second = self.collections()
        first["sources"][0].update(status="partial", error="malformed_event")
        second["sources"][3].update(status="missing", events=[], error="transport_error")
        merged = activity.merge_activity(cfg, [first, second])
        self.assertEqual(merged["sources"][0]["status"], "partial")
        self.assertEqual(merged["sources"][3]["status"], "missing")
        self.assertEqual(merged, activity.merge_activity(cfg, [second, first]))

    def test_mismatched_config_sources_and_invalid_states_are_rejected(self):
        cfg, first, second = self.collections()
        alterations = []
        bad = copy.deepcopy(second)
        bad["config_sha256"] = "0" * 64
        alterations.append(bad)
        bad = copy.deepcopy(second)
        bad["sources"].pop()
        alterations.append(bad)
        bad = copy.deepcopy(second)
        bad["sources"][1]["device_id"] = "device-2"
        alterations.append(bad)
        bad = copy.deepcopy(second)
        bad["sources"][1]["status"] = "unknown"
        alterations.append(bad)
        for bad in alterations:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                activity.merge_activity(cfg, [first, bad])
        for bad in ([], {}, None):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                activity.merge_activity(cfg, bad)

    def test_remerging_retains_immediate_artifact_ancestry(self):
        cfg, first, second = self.collections()
        merged = activity.merge_activity(cfg, [first, second])
        replay = activity.merge_activity(cfg, [merged, second])
        self.assertEqual(replay["sources"], merged["sources"])
        self.assertEqual(replay["merged_from"], sorted([canonical_hash(merged), canonical_hash(second)]))

    def test_afk_on_one_device_does_not_activate_another_devices_context(self):
        from effort.frame import build_frame

        cfg, first, second = self.collections()
        first["sources"][1].update(status="missing", events=[], error="bucket_missing")
        second["sources"][2].update(status="missing", events=[], error="bucket_missing")
        merged = activity.merge_activity(cfg, [first, second])
        frozen = build_frame(cfg, merged)
        self.assertEqual(sum(unit["predicted_person_hours"] for unit in frozen["units"]), 0)
        self.assertTrue(all("telemetry_gap" in unit["flags"] for unit in frozen["units"]))


if __name__ == "__main__":
    unittest.main()
