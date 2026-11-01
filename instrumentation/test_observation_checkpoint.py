"""Trust, delivery and archive-boundary tests for hosted checkpoint persistence."""
import hashlib
import io
import json
from pathlib import Path
import re
import stat
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from observations import checkpoint as cp


CONTEXT = {"repository": "owner/public", "repository_id": 42, "workflow": ".github/workflows/demo.yml",
           "run_id": 100, "run_attempt": 1, "event": "workflow_dispatch", "branch": "main",
           "default_branch": "main", "scope": "trusted_default_branch"}


def run(identity, **updates):
    result = {"id": identity, "run_attempt": 1, "workflow_id": 7, "path": CONTEXT["workflow"],
              "repository": {"id": 42}, "head_repository": {"id": 42}, "event": "workflow_dispatch",
              "head_branch": "main", "status": "completed", "conclusion": "success"}
    result.update(updates)
    return result


class API:
    def __init__(self, runs=(), artifacts=(), data=None, attempts=None, skipped=()):
        self.runs, self.artifacts, self.data = list(runs), list(artifacts), data
        self.attempts, self.calls = attempts or {}, []
        self.skipped = set(skipped)

    def json(self, endpoint):
        self.calls.append(endpoint)
        if endpoint == "repos/owner/public":
            return {"id": 42, "full_name": "owner/public", "private": False, "visibility": "public", "default_branch": "main"}
        if endpoint.endswith("/actions/workflows/demo.yml"):
            return {"id": 7, "path": CONTEXT["workflow"]}
        if "/jobs?" in endpoint:
            identity, attempt = map(int, re.search(r"/runs/(\d+)/attempts/(\d+)/jobs", endpoint).groups())
            return {"jobs": [{"steps": [
                {"name": cp.IMPORT_STEP, "conclusion": "skipped" if (identity, attempt) in self.skipped else "success"},
                {"name": cp.RESTORE_STEP, "conclusion": "failure"}]}]}
        if "/attempts/" in endpoint:
            return self.attempts[int(endpoint.rsplit("/", 1)[1])]
        if "/actions/workflows/7/runs?" in endpoint:
            return {"workflow_runs": self.runs}
        if "/artifacts?" in endpoint:
            return {"artifacts": self.artifacts}
        raise AssertionError("unexpected endpoint " + endpoint)

    def download(self, endpoint, destination):
        self.calls.append(endpoint)
        Path(destination).write_bytes(self.data)


class Checkpoints(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.state = self.root / "state"
        self.state.mkdir()
        (self.state / "model.json").write_text('{"active":"v1","consumed_validation":["a"]}')
        (self.state / "revisions").mkdir()
        (self.state / "revisions/one.json").write_text('{"minutes":3}')
        self.archive, self.destination = self.root / "checkpoint.zip", self.root / "restored"

    def rewrite(self, transform):
        with zipfile.ZipFile(self.archive) as archive:
            contents = [(member, archive.read(member)) for member in archive.infolist()]
        with zipfile.ZipFile(self.archive, "w") as archive:
            for member, raw in transform(contents):
                archive.writestr(member, raw)

    def published(self, context=None):
        context = context or {**CONTEXT, "run_id": 99}
        cp.seal(self.state, self.archive, context)
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("checkpoint.zip", self.archive.read_bytes())
        raw = buffer.getvalue()
        artifact = {"id": 91, "name": cp.artifact_name(context), "expired": False, "size_in_bytes": len(raw),
                    "workflow_run": {"id": context["run_id"]}, "digest": "sha256:" + hashlib.sha256(raw).hexdigest()}
        return artifact, raw

    def test_seal_roundtrip_preserves_revisions_and_consumed_holdouts(self):
        original = cp.seal(self.state, self.archive, CONTEXT)
        result = cp.extract(self.archive, self.destination, CONTEXT)
        self.assertEqual(original, result)
        self.assertEqual((self.destination / "model.json").read_bytes(), (self.state / "model.json").read_bytes())
        self.assertEqual((self.destination / "revisions/one.json").read_bytes(), (self.state / "revisions/one.json").read_bytes())
        self.assertFalse((self.destination / "model.json").stat().st_mode & 0o111)

    def test_tampering_is_detected_before_any_state_is_published(self):
        cp.seal(self.state, self.archive, CONTEXT)
        self.rewrite(lambda contents: [(member, raw.replace(b'"minutes":3', b'"minutes":9')) for member, raw in contents])
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            cp.extract(self.archive, self.destination, CONTEXT)
        self.assertFalse(self.destination.exists())

    def test_manifest_hash_is_checked(self):
        cp.seal(self.state, self.archive, CONTEXT)
        def change(contents):
            for member, raw in contents:
                if member.filename == "manifest.json":
                    manifest = json.loads(raw)
                    manifest["context"]["run_id"] = 98
                    raw = json.dumps(manifest).encode()
                yield member, raw
        self.rewrite(change)
        with self.assertRaisesRegex(ValueError, "manifest hash mismatch"):
            cp.extract(self.archive, self.destination, CONTEXT)

    def test_archive_cannot_claim_different_origin(self):
        cp.seal(self.state, self.archive, {**CONTEXT, "scope": "isolated_demo", "event": "pull_request"})
        with self.assertRaisesRegex(ValueError, "provenance mismatch"):
            cp.extract(self.archive, self.destination, CONTEXT)

    def test_rejects_traversal_absolute_duplicates_and_symlinks(self):
        for bad_path in ("../escape", "/absolute", "state/../escape", "state\\escape", "state/a//b", "C:/escape"):
            with self.subTest(path=bad_path):
                self.archive.unlink(missing_ok=True)
                cp.seal(self.state, self.archive, CONTEXT)
                self.rewrite(lambda contents: contents + [(zipfile.ZipInfo(bad_path), b"evil")])
                with self.assertRaisesRegex(ValueError, "path"):
                    cp.extract(self.archive, self.destination, CONTEXT)
        self.archive.unlink()
        cp.seal(self.state, self.archive, CONTEXT)
        symlink = zipfile.ZipInfo("state/link")
        symlink.create_system = 3
        symlink.external_attr = (stat.S_IFLNK | 0o777) << 16
        self.rewrite(lambda contents: contents + [(symlink, b"/etc/passwd")])
        with self.assertRaisesRegex(ValueError, "non-regular"):
            cp.extract(self.archive, self.destination, CONTEXT)

    def test_rejects_duplicate_zip_members(self):
        cp.seal(self.state, self.archive, CONTEXT)
        import warnings
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            self.rewrite(lambda contents: contents + [contents[0]])
        with self.assertRaisesRegex(ValueError, "duplicate checkpoint path"):
            cp.extract(self.archive, self.destination, CONTEXT)

    def test_rejects_unlisted_extra_file_and_existing_destination(self):
        cp.seal(self.state, self.archive, CONTEXT)
        self.rewrite(lambda contents: contents + [(zipfile.ZipInfo("state/extra"), b"evil")])
        with self.assertRaisesRegex(ValueError, "unexpected checkpoint file"):
            cp.extract(self.archive, self.destination, CONTEXT)
        self.destination.mkdir()
        with self.assertRaisesRegex(ValueError, "must not exist"):
            cp.extract(self.archive, self.destination, CONTEXT)

    def test_bounds_apply_to_compressed_and_expanded_archives(self):
        cp.seal(self.state, self.archive, CONTEXT)
        with patch.object(cp, "MAX_ARCHIVE", 10):
            with self.assertRaisesRegex(ValueError, "archive exceeds"):
                cp.extract(self.archive, self.destination, CONTEXT)
        with patch.object(cp, "MAX_EXPANDED", 10):
            with self.assertRaisesRegex(ValueError, "exceeds limit"):
                cp.extract(self.archive, self.destination, CONTEXT)
        with patch.object(cp, "MAX_FILES", 1):
            with self.assertRaisesRegex(ValueError, "file count"):
                cp.extract(self.archive, self.destination, CONTEXT)

    def test_seal_rejects_local_symlink_and_in_state_output(self):
        (self.state / "link").symlink_to(self.state / "model.json")
        with self.assertRaisesRegex(ValueError, "symlink"):
            cp.seal(self.state, self.archive, CONTEXT)
        (self.state / "link").unlink()
        with self.assertRaisesRegex(ValueError, "outside state"):
            cp.seal(self.state, self.state / "checkpoint.zip", CONTEXT)

    def test_pr_state_is_isolated_and_never_reads_trusted_artifacts(self):
        api = API(runs=[run(99)])
        result = cp.restore(api, {**CONTEXT, "scope": "isolated_demo", "event": "pull_request", "branch": "10/merge"},
                            self.destination, self.root / "receipt.json")
        self.assertEqual(result["status"], "isolated_empty_state")
        self.assertEqual(api.calls, [])

    def test_no_history_requires_explicit_bootstrap(self):
        with self.assertRaisesRegex(ValueError, "explicit manual bootstrap"):
            cp.restore(API(), CONTEXT, self.destination, self.root / "receipt.json")
        result = cp.restore(API(), CONTEXT, self.destination, self.root / "receipt.json", bootstrap=True)
        self.assertEqual(result["status"], "bootstrap_no_prior_trusted_state")

    def test_prior_failed_history_cannot_be_silently_reset(self):
        with self.assertRaisesRegex(ValueError, "may have evolved observations"):
            cp.restore(API(runs=[run(99, conclusion="failure")]), CONTEXT, self.destination,
                       self.root / "receipt.json", bootstrap=True)

    def test_first_schedule_failed_before_import_can_explicitly_bootstrap(self):
        api = API(runs=[run(99, conclusion="failure")], skipped=[(99, 1)])
        result = cp.restore(api, CONTEXT, self.destination, self.root / "receipt.json", bootstrap=True)
        self.assertEqual(result["status"], "bootstrap_no_prior_trusted_state")

    def test_failed_validation_cannot_be_discarded_by_returning_to_older_success(self):
        api = API(runs=[run(98), run(99, conclusion="failure")])
        with self.assertRaisesRegex(ValueError, "consumed validation"):
            cp.prior_run(api, CONTEXT)

    def test_proven_skipped_import_can_continue_from_previous_success(self):
        api = API(runs=[run(98), run(99, conclusion="failure")], skipped=[(99, 1)])
        self.assertEqual(cp.prior_run(api, CONTEXT)["id"], 98)

    def test_failed_earlier_run_attempt_cannot_hide_previous_success(self):
        api = API(runs=[run(99, run_attempt=2, conclusion="failure")], attempts={1: run(99)}, skipped=[(99, 2)])
        result = cp.prior_run(api, CONTEXT)
        self.assertEqual((result["id"], result["run_attempt"]), (99, 1))

    def test_latest_success_missing_artifact_never_falls_back(self):
        api = API(runs=[run(98), run(99)], artifacts=[{"name": "observatory-observation-state-98-1"}])
        with self.assertRaisesRegex(ValueError, "missing or ambiguous"):
            cp.restore(api, CONTEXT, self.destination, self.root / "receipt.json", bootstrap=True)
        self.assertTrue(any("/runs/99/artifacts" in call for call in api.calls))
        self.assertFalse(any("/runs/98/artifacts" in call for call in api.calls))

    def test_trust_excludes_pr_branch_workflow_and_foreign_repository(self):
        cases = [run(99, event="pull_request"), run(99, head_branch="feature"), run(99, workflow_id=8),
                 run(99, path=".github/workflows/other.yml"), run(99, repository={"id": 80}),
                 run(99, head_repository={"id": 80})]
        for untrusted in cases:
            with self.subTest(run=untrusted):
                self.assertIsNone(cp.prior_run(API(runs=[untrusted]), CONTEXT))

    def test_previous_successful_attempt_is_restored_for_rerun(self):
        context = {**CONTEXT, "run_attempt": 3}
        api = API(runs=[run(99)], attempts={2: run(100, run_attempt=2, conclusion="failure"), 1: run(100)}, skipped=[(100, 2)])
        self.assertEqual(cp.prior_run(api, context)["id"], 100)
        self.assertTrue(any("/workflows/7/runs?" in call for call in api.calls))

    def test_old_run_rerun_cannot_roll_back_newer_state(self):
        with self.assertRaisesRegex(ValueError, "newer trusted state exists"):
            cp.prior_run(API(runs=[run(101)], attempts={1: run(100)}), {**CONTEXT, "run_attempt": 2})

    def test_full_workflow_pagination_fails_closed(self):
        with patch.object(cp, "MAX_PAGES", 1):
            with self.assertRaisesRegex(ValueError, "history exceeds bounded"):
                cp.prior_run(API(runs=[run(i + 1) for i in range(100)]), CONTEXT)

    def test_download_restore_verifies_source_hashes_and_retains_state(self):
        artifact, raw = self.published()
        result = cp.restore(API(runs=[run(99)], artifacts=[artifact], data=raw), CONTEXT,
                            self.destination, self.root / "receipt.json")
        self.assertEqual(result["status"], "restored_verified_checkpoint")
        self.assertEqual(result["source"]["run_id"], 99)
        self.assertEqual((self.destination / "model.json").read_bytes(), (self.state / "model.json").read_bytes())

    def test_expired_wrong_run_and_corrupt_artifacts_fail_closed(self):
        artifact, raw = self.published()
        for change, message in (({"expired": True}, "expired"), ({"workflow_run": {"id": 98}}, "identity mismatch"),
                                ({"digest": "sha256:" + "0" * 64}, "digest mismatch")):
            with self.subTest(change=change):
                with self.assertRaisesRegex(ValueError, message):
                    cp.restore(API(runs=[run(99)], artifacts=[{**artifact, **change}], data=raw), CONTEXT,
                               self.destination, self.root / "receipt.json", bootstrap=True)
                self.assertFalse(self.destination.exists())

    def test_repository_must_be_verified_public_and_ref_is_not_assumed_trusted(self):
        environment = {"GITHUB_REPOSITORY": "owner/public", "GITHUB_WORKFLOW_REF": "owner/public/.github/workflows/demo.yml@refs/heads/main",
                       "GITHUB_REF_NAME": "main", "GITHUB_REF_TYPE": "branch", "GITHUB_EVENT_NAME": "workflow_dispatch",
                       "GITHUB_RUN_ID": "100", "GITHUB_RUN_ATTEMPT": "1"}
        self.assertEqual(cp.context_from_environment(API(), environment), CONTEXT)
        for change in ({"GITHUB_EVENT_NAME": "pull_request"}, {"GITHUB_REF_NAME": "feature"}, {"GITHUB_REF_TYPE": "tag"}):
            self.assertEqual(cp.context_from_environment(API(), {**environment, **change})["scope"], "isolated_demo")
        api = API()
        with patch.object(api, "json", return_value={"private": True}):
            with self.assertRaisesRegex(ValueError, "verified public repository"):
                cp.context_from_environment(api, environment)


if __name__ == "__main__":
    unittest.main()
