#!/usr/bin/env python3
"""Sealed public demo state; GitHub run provenance is the trust boundary.

This is a bounded artifact checkpoint, not an archival database. Never restore
pull-request state into the default branch. Missing trusted history is an error,
including artifact expiry; bootstrap is permitted only without prior trusted runs.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import sys
import tempfile
from urllib.parse import quote, urlencode
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from effort.common import canonical_bytes, canonical_hash, now, read_json, require, write_json

SCHEMA = "observatory.observation-checkpoint/1"
RECEIPT_SCHEMA = "observatory.observation-checkpoint-restore/1"
ARTIFACT_PREFIX = "observatory-observation-state"
MAX_ARCHIVE = 50 * 1024 * 1024
MAX_EXPANDED = 100 * 1024 * 1024
MAX_FILES = 20000
MAX_MANIFEST = 8 * 1024 * 1024
MAX_PAGES = 20
TRUSTED_EVENTS = {"schedule", "workflow_dispatch"}
IMPORT_STEP = "Import public observations and execution evidence, then evaluate candidates"
RESTORE_STEP = "Restore trusted state or initialize an isolated demonstration"


def _decode(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result
    def invalid(_):
        raise ValueError("non-finite JSON")
    try:
        value = json.loads(raw, object_pairs_hook=pairs, parse_constant=invalid)
        canonical_bytes(value)
        return value
    except (RecursionError, UnicodeError) as error:
        raise ValueError("invalid JSON") from error


def _path(name):
    require(isinstance(name, str) and 0 < len(name) <= 1024, "invalid checkpoint path")
    require("\\" not in name and not any(ord(c) < 32 for c in name), "unsafe checkpoint path")
    path = PurePosixPath(name)
    require(not path.is_absolute() and all(p not in {"", ".", ".."} for p in name.split("/")), "unsafe checkpoint path")
    require(":" not in name, "unsafe checkpoint path")
    return name


def _members(archive):
    items = archive.infolist()
    require(0 < len(items) <= MAX_FILES + 1, "checkpoint file count exceeds limit")
    names, total = set(), 0
    for item in items:
        _path(item.filename)
        require(item.filename not in names, "duplicate checkpoint path")
        names.add(item.filename)
        mode = item.external_attr >> 16
        require(not item.is_dir() and stat.S_IFMT(mode) in {0, stat.S_IFREG}, "checkpoint contains a non-regular file")
        require(not item.flag_bits & 1, "encrypted checkpoint unsupported")
        require(item.compress_type in {zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED}, "unsupported checkpoint compression")
        require(0 <= item.file_size <= MAX_EXPANDED, "checkpoint member exceeds limit")
        total += item.file_size
        require(total <= MAX_EXPANDED, "expanded checkpoint exceeds limit")
    return {item.filename: item for item in items}


def _context(value):
    required = {"repository", "repository_id", "workflow", "run_id", "run_attempt", "event", "branch", "default_branch", "scope"}
    require(isinstance(value, dict) and set(value) == required, "invalid checkpoint context")
    require(isinstance(value["repository"], str) and re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", value["repository"]), "invalid repository")
    require(isinstance(value["workflow"], str) and re.fullmatch(r"\.github/workflows/[A-Za-z0-9_.-]+\.ya?ml", value["workflow"]), "invalid workflow path")
    for field in ("repository_id", "run_id", "run_attempt"):
        require(type(value[field]) is int and 0 < value[field] < 2**63, "invalid " + field)
    for field in ("event", "branch", "default_branch"):
        require(isinstance(value[field], str) and 0 < len(value[field]) <= 256 and not any(ord(c) < 32 for c in value[field]), "invalid " + field)
    require(value["scope"] in {"trusted_default_branch", "isolated_demo"}, "invalid checkpoint scope")
    if value["scope"] == "trusted_default_branch":
        require(value["event"] in TRUSTED_EVENTS and value["branch"] == value["default_branch"], "invalid trusted checkpoint context")
    return value


def seal(directory, output, context):
    """Snapshot only regular data files after all writers have exited."""
    context = _context(context)
    directory, output = Path(directory), Path(output)
    require(directory.is_dir() and not directory.is_symlink(), "state must be a regular directory")
    require(not output.exists() and not output.is_symlink(), "refusing to replace checkpoint")
    require(not output.resolve().is_relative_to(directory.resolve()), "checkpoint output must be outside state")
    files, contents, total = [], {}, 0
    for path in sorted(directory.rglob("*")):
        require(not path.is_symlink(), "state contains a symlink")
        if path.is_dir():
            continue
        require(path.is_file(), "state contains a non-regular file")
        name = _path(path.relative_to(directory).as_posix())
        require(path.stat().st_size <= MAX_EXPANDED, "state file exceeds limit")
        raw = path.read_bytes()
        total += len(raw)
        require(total <= MAX_EXPANDED and len(files) < MAX_FILES, "state exceeds checkpoint bounds")
        files.append({"path": name, "size": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
        contents["state/" + name] = raw
    manifest = {"schema": SCHEMA, "context": context, "files": files}
    manifest["manifest_sha256"] = canonical_hash(manifest)
    raw_manifest = canonical_bytes(manifest)
    require(len(raw_manifest) <= MAX_MANIFEST, "manifest exceeds limit")
    output.parent.mkdir(parents=True, exist_ok=True)
    try:
        with zipfile.ZipFile(output, "x", compression=zipfile.ZIP_DEFLATED) as archive:
            archive.writestr("manifest.json", raw_manifest)
            for name, raw in contents.items():
                archive.writestr(name, raw)
        require(output.stat().st_size <= MAX_ARCHIVE, "compressed checkpoint exceeds limit")
    except BaseException:
        output.unlink(missing_ok=True)
        raise
    return manifest


def extract(archive_path, directory, expected_context):
    """Verify everything before publishing a new directory; execute nothing."""
    expected_context = _context(expected_context)
    archive_path, directory = Path(archive_path), Path(directory)
    require(archive_path.is_file() and archive_path.stat().st_size <= MAX_ARCHIVE, "checkpoint archive exceeds limit")
    require(not directory.exists() and not directory.is_symlink(), "restore destination must not exist")
    with zipfile.ZipFile(archive_path) as archive:
        members = _members(archive)
        require("manifest.json" in members and members["manifest.json"].file_size <= MAX_MANIFEST, "missing or oversized checkpoint manifest")
        manifest = _decode(archive.read("manifest.json"))
        require(isinstance(manifest, dict) and set(manifest) == {"schema", "context", "files", "manifest_sha256"}, "invalid checkpoint manifest")
        require(manifest["schema"] == SCHEMA, "unsupported checkpoint schema")
        digest = manifest["manifest_sha256"]
        payload = {k: v for k, v in manifest.items() if k != "manifest_sha256"}
        require(digest == canonical_hash(payload), "checkpoint manifest hash mismatch")
        require(_context(manifest["context"]) == expected_context, "checkpoint provenance mismatch")
        require(isinstance(manifest["files"], list) and len(manifest["files"]) <= MAX_FILES, "invalid checkpoint file list")
        expected_names, content = {"manifest.json"}, {}
        for record in manifest["files"]:
            require(isinstance(record, dict) and set(record) == {"path", "size", "sha256"}, "invalid checkpoint file record")
            name = "state/" + _path(record["path"])
            require(name not in expected_names, "duplicate manifest path")
            expected_names.add(name)
            require(type(record["size"]) is int and 0 <= record["size"] <= MAX_EXPANDED, "invalid checkpoint size")
            require(isinstance(record["sha256"], str) and re.fullmatch(r"[0-9a-f]{64}", record["sha256"]), "invalid checkpoint hash")
            require(name in members and members[name].file_size == record["size"], "checkpoint file size mismatch")
            raw = archive.read(name)
            require(hashlib.sha256(raw).hexdigest() == record["sha256"], "checkpoint file hash mismatch")
            content[record["path"]] = raw
        require(set(members) == expected_names, "unexpected checkpoint file")
    directory.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix=".verified-checkpoint-", dir=directory.parent))
    try:
        for name, raw in content.items():
            target = temporary / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(raw)
            target.chmod(0o600)
        require(not directory.exists(), "restore destination appeared concurrently")
        temporary.rename(directory)
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)
    return manifest


class GitHub:
    """gh keeps credentials out of the checkpoint and follows artifact redirects."""
    def json(self, endpoint):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "response"
            self.download(endpoint, path, limit=16 * 1024 * 1024)
            return read_json(path)

    def download(self, endpoint, destination, limit=MAX_ARCHIVE):
        require(endpoint.startswith("repos/") and not any(c in endpoint for c in "\r\n"), "invalid API endpoint")
        with tempfile.TemporaryFile() as errors, Path(destination).open("xb") as output:
            process = subprocess.Popen(["gh", "api", endpoint], stdout=subprocess.PIPE, stderr=errors)
            size = 0
            try:
                while raw := process.stdout.read(65536):
                    size += len(raw)
                    require(size <= limit, "GitHub response exceeds limit")
                    output.write(raw)
                require(process.wait(timeout=60) == 0, "GitHub request failed: " + endpoint.split("?")[0])
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait()
                process.stdout.close()


def artifact_name(context):
    return f"{ARTIFACT_PREFIX}-{context['run_id']}-{context['run_attempt']}"


def context_from_environment(api, environment=None):
    env = os.environ if environment is None else environment
    repository = env.get("GITHUB_REPOSITORY", "")
    require(re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository), "missing repository context")
    metadata = api.json(f"repos/{repository}")
    require(metadata.get("private") is False and metadata.get("visibility", "public") == "public", "demo checkpoints require a verified public repository")
    require(metadata.get("full_name", "").lower() == repository.lower(), "repository identity mismatch")
    workflow_ref = env.get("GITHUB_WORKFLOW_REF", "")
    prefix = repository + "/"
    require(workflow_ref.startswith(prefix) and "@" in workflow_ref, "missing caller workflow identity")
    workflow = workflow_ref[len(prefix):].split("@", 1)[0]
    branch, event = env.get("GITHUB_REF_NAME", ""), env.get("GITHUB_EVENT_NAME", "")
    default_branch = metadata.get("default_branch")
    scope = "trusted_default_branch" if branch == default_branch and event in TRUSTED_EVENTS and env.get("GITHUB_REF_TYPE") == "branch" else "isolated_demo"
    return _context({"repository": metadata["full_name"], "repository_id": metadata["id"], "workflow": workflow,
                     "run_id": int(env.get("GITHUB_RUN_ID", "0")), "run_attempt": int(env.get("GITHUB_RUN_ATTEMPT", "0")),
                     "event": event, "branch": branch, "default_branch": default_branch, "scope": scope})


def _trusted_run(run, context, workflow_id):
    return (run.get("workflow_id") == workflow_id
            and run.get("path") == context["workflow"]
            and run.get("repository", {}).get("id") == context["repository_id"]
            and run.get("head_repository", {}).get("id") == context["repository_id"]
            and run.get("event") in TRUSTED_EVENTS
            and run.get("head_branch") == context["default_branch"]
            and type(run.get("id")) is int and 0 < run["id"] < 2**63
            and type(run.get("run_attempt")) is int and 0 < run["run_attempt"] < 1000)


def _prove_no_evolution(api, context, run):
    """A failed delivery may have exposed validation: never silently discard it.

Only platform job metadata proving that the import/evaluation step was skipped
allows traversal past a failed run. This also permits explicit first bootstrap
after an uninitialized scheduled run failed at restoration.
"""
    require(run.get("status") == "completed" and run.get("conclusion") in {"failure", "cancelled"},
            "trusted run lacks a completed checkpoint; explicit operator recovery required")
    matches = []
    for page in range(1, MAX_PAGES + 1):
        endpoint = f"repos/{context['repository']}/actions/runs/{run['id']}/attempts/{run['run_attempt']}/jobs?per_page=100&page={page}"
        jobs = api.json(endpoint).get("jobs")
        require(isinstance(jobs, list) and len(jobs) <= 100, "invalid job evidence page")
        for job in jobs:
            steps = job.get("steps", [])
            require(isinstance(steps, list), "invalid job step evidence")
            imports = [step for step in steps if step.get("name") == IMPORT_STEP]
            restores = [step for step in steps if step.get("name") == RESTORE_STEP]
            if imports or restores:
                require(len(imports) == 1 and len(restores) == 1, "ambiguous checkpoint stage evidence")
                matches.append((imports[0], restores[0]))
        if len(jobs) < 100:
            break
    else:
        raise ValueError("job evidence exceeds bounded lookup")
    require(len(matches) == 1 and matches[0][0].get("conclusion") == "skipped"
            and matches[0][1].get("conclusion") in {"success", "failure", "skipped", "cancelled"},
            "failed trusted run may have evolved observations or consumed validation; explicit operator recovery required")


def prior_run(api, context):
    """Select latest success without discarding failed state-evolving attempts."""
    repository, workflow = context["repository"], context["workflow"]
    metadata = api.json(f"repos/{repository}/actions/workflows/{quote(PurePosixPath(workflow).name, safe='')}")
    require(metadata.get("path") == workflow and type(metadata.get("id")) is int, "workflow identity mismatch")
    workflow_id, history = metadata["id"], {}
    for page in range(1, MAX_PAGES + 1):
        query = urlencode({"branch": context["default_branch"], "per_page": 100, "page": page})
        response = api.json(f"repos/{repository}/actions/workflows/{workflow_id}/runs?{query}")
        runs = response.get("workflow_runs")
        require(isinstance(runs, list) and len(runs) <= 100, "invalid workflow run page")
        for run in runs:
            if _trusted_run(run, context, workflow_id) and run["id"] != context["run_id"]:
                require(run["id"] not in history, "workflow history changed during lookup")
                history[run["id"]] = run
        if len(runs) < 100:
            break
    else:
        raise ValueError("workflow history exceeds bounded lookup; restore requires operator recovery")
    require(context["run_attempt"] <= 100, "too many attempts for bounded checkpoint lookup")
    if context["run_attempt"] > 1:
        attempt = context["run_attempt"] - 1
        history[context["run_id"]] = api.json(f"repos/{repository}/actions/runs/{context['run_id']}/attempts/{attempt}")
        require(history[context["run_id"]].get("run_attempt") == attempt, "prior attempt identity mismatch")
    lookups = 0
    for identity in sorted(history, reverse=True):
        latest = history[identity]
        require(_trusted_run(latest, context, workflow_id) and latest["id"] == identity, "prior run identity mismatch")
        require(latest["run_attempt"] <= 100, "too many attempts for bounded checkpoint lookup")
        for attempt in range(latest["run_attempt"], 0, -1):
            lookups += 1
            require(lookups <= 100, "too many failed attempts; explicit operator recovery required")
            run = latest if attempt == latest["run_attempt"] else api.json(f"repos/{repository}/actions/runs/{identity}/attempts/{attempt}")
            require(_trusted_run(run, context, workflow_id) and run["id"] == identity and run["run_attempt"] == attempt,
                    "prior attempt identity mismatch")
            if run.get("status") == "completed" and run.get("conclusion") == "success":
                require(identity <= context["run_id"], "newer trusted state exists; rerunning an older run would fork checkpoint history")
                return run
            _prove_no_evolution(api, context, run)
    return None


def _extract_artifact(api, artifact, context, directory):
    require(artifact.get("name") == artifact_name(context), "artifact name mismatch")
    require(artifact.get("workflow_run", {}).get("id") == context["run_id"], "artifact run identity mismatch")
    require(artifact.get("expired") is False, "latest trusted checkpoint expired; explicit operator recovery required")
    require(type(artifact.get("id")) is int and artifact["id"] > 0, "invalid artifact identity")
    require(type(artifact.get("size_in_bytes")) is int and 0 < artifact["size_in_bytes"] <= MAX_ARCHIVE, "artifact size exceeds limit")
    with tempfile.TemporaryDirectory() as temporary:
        outer, inner = Path(temporary) / "download.zip", Path(temporary) / "checkpoint.zip"
        api.download(f"repos/{context['repository']}/actions/artifacts/{artifact['id']}/zip", outer)
        require(outer.stat().st_size <= MAX_ARCHIVE, "artifact archive exceeds limit")
        digest = artifact.get("digest")
        if digest is not None:
            require(digest == "sha256:" + hashlib.sha256(outer.read_bytes()).hexdigest(), "GitHub artifact digest mismatch")
        with zipfile.ZipFile(outer) as archive:
            require(set(_members(archive)) == {"checkpoint.zip"}, "unexpected artifact contents")
            raw = archive.read("checkpoint.zip")
            require(len(raw) <= MAX_ARCHIVE, "checkpoint exceeds limit")
            inner.write_bytes(raw)
        return extract(inner, directory, context)


def restore(api, context, directory, receipt_path, bootstrap=False):
    context = _context(context)
    directory = Path(directory)
    require(not directory.exists() and not directory.is_symlink(), "restore destination must not exist")
    receipt = {"schema": RECEIPT_SCHEMA, "context": context, "restored_at": now(), "status": None, "source": None}
    if context["scope"] == "isolated_demo":
        directory.mkdir(parents=True)
        receipt["status"] = "isolated_empty_state"
    else:
        run = prior_run(api, context)
        if run is None:
            require(bootstrap, "first trusted run requires an explicit manual bootstrap=true; no checkpoint was restored")
            directory.mkdir(parents=True)
            receipt["status"] = "bootstrap_no_prior_trusted_state"
        else:
            expected = {**context, "run_id": run["id"], "run_attempt": run["run_attempt"], "event": run["event"]}
            name, matches = artifact_name(expected), []
            for page in range(1, MAX_PAGES + 1):
                response = api.json(f"repos/{context['repository']}/actions/runs/{run['id']}/artifacts?per_page=100&page={page}")
                artifacts = response.get("artifacts")
                require(isinstance(artifacts, list) and len(artifacts) <= 100, "invalid artifact page")
                matches.extend(artifact for artifact in artifacts if artifact.get("name") == name)
                if len(artifacts) < 100:
                    break
            else:
                raise ValueError("artifact listing exceeds bounded lookup")
            require(len(matches) == 1, "latest trusted checkpoint is missing or ambiguous; explicit operator recovery required")
            artifact = matches[0]
            manifest = _extract_artifact(api, artifact, expected, directory)
            receipt["status"] = "restored_verified_checkpoint"
            receipt["source"] = {"run_id": run["id"], "run_attempt": run["run_attempt"], "artifact_id": artifact["id"], "manifest_sha256": manifest["manifest_sha256"]}
    write_json(receipt_path, receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    restoring = commands.add_parser("restore")
    restoring.add_argument("--directory", required=True)
    restoring.add_argument("--receipt", required=True)
    restoring.add_argument("--bootstrap", action="store_true", help="Permit first trusted run only, never reset existing history")
    for name in ("seal", "verify"):
        command = commands.add_parser(name)
        command.add_argument("--directory", required=True)
        command.add_argument("--archive", required=True)
        command.add_argument("--receipt", required=True)
    delivered = commands.add_parser("verify-delivery", help="Verify only this run's uploaded state; never select trusted history")
    delivered.add_argument("--directory", required=True)
    delivered.add_argument("--receipt", required=True)
    delivered.add_argument("--artifact-id", required=True, type=int)
    arguments = parser.parse_args()
    if arguments.command == "restore":
        api = GitHub()
        result = restore(api, context_from_environment(api), arguments.directory, arguments.receipt, bootstrap=arguments.bootstrap)
        print(json.dumps({"checkpoint_status": result["status"], "scope": result["context"]["scope"]}))
    else:
        receipt = read_json(arguments.receipt)
        require(receipt.get("schema") == RECEIPT_SCHEMA, "unsupported restore receipt")
        if arguments.command == "verify-delivery":
            require(arguments.artifact_id > 0, "invalid artifact ID")
            api = GitHub()
            context = context_from_environment(api)
            require(receipt["context"] == context, "roundtrip receipt must belong to current run")
            artifact = api.json(f"repos/{context['repository']}/actions/artifacts/{arguments.artifact_id}")
            result = _extract_artifact(api, artifact, context, arguments.directory)
        else:
            operation = seal if arguments.command == "seal" else lambda directory, archive, context: extract(archive, directory, context)
            result = operation(arguments.directory, arguments.archive, receipt["context"])
        print(json.dumps({"manifest_sha256": result["manifest_sha256"], "files": len(result["files"])}))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, zipfile.BadZipFile, subprocess.SubprocessError) as error:
        print("Checkpoint rejected: " + str(error), file=sys.stderr)
        sys.exit(1)
