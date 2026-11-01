#!/usr/bin/env python3
"""Explicit, job-local execution receipts. No environment, argv or transcript capture."""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import stat
import subprocess
import sys
import threading
import tempfile
import time
import uuid

MAX_RECEIPT_BYTES = 256 * 1024
MAX_BUNDLE_BYTES = 16 * 1024 * 1024
MAX_EVENT_BYTES = 256 * 1024
CONFIGURATION_KEYS = {
    "reasoning_effort", "temperature", "seed", "max_output_tokens", "configuration_sha256"
}
CONTROLLERS = ("human", "assistant", "automation", "unknown")
PURPOSES = ("development", "product_execution", "unknown")


def json_text(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False)


def load_json(text):
    def object_pairs(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("duplicate JSON object key")
            result[key] = value
        return result
    return json.loads(text, object_pairs_hook=object_pairs)


def validate_schema(value, schema, location="evidence", root=None):
    """Validate the deliberately small, closed JSON Schema vocabulary shipped here.

    Fail closed if a later schema introduces a keyword this validator does not
    understand. The Rust importer independently validates every delivered bundle.
    """
    root = schema if root is None else root
    supported = {"$ref", "$defs", "not", "$schema", "$id", "title", "description", "type", "const", "enum",
                 "properties", "required", "additionalProperties", "propertyNames",
                 "minLength", "maxLength", "pattern", "format", "minimum", "maximum",
                 "items", "maxItems", "uniqueItems", "maxProperties", "anyOf", "allOf", "if", "then", "else"}
    if set(schema) - supported:
        raise ValueError("unsupported schema validation keyword")
    def require(condition):
        if not condition:
            raise ValueError(f"schema validation failed at {location}")
    if "$ref" in schema:
        reference = schema["$ref"]
        require(isinstance(reference, str) and reference.startswith("#/"))
        target = root
        for part in reference[2:].split("/"):
            target = target[part.replace("~1", "/").replace("~0", "~")]
        validate_schema(value, target, location, root)
    if "not" in schema:
        matched = True
        try:
            validate_schema(value, schema["not"], location, root)
        except ValueError:
            matched = False
        require(not matched)
    if "anyOf" in schema:
        matched = False
        for option in schema["anyOf"]:
            try:
                validate_schema(value, option, location, root)
                matched = True
            except ValueError:
                pass
        require(matched)
    for option in schema.get("allOf", []):
        validate_schema(value, option, location, root)
    if "if" in schema:
        try:
            validate_schema(value, schema["if"], location, root)
            branch = "then"
        except ValueError:
            branch = "else"
        if branch in schema:
            validate_schema(value, schema[branch], location, root)
    if "type" in schema:
        kinds = schema["type"] if isinstance(schema["type"], list) else [schema["type"]]
        checks = {"null": value is None, "object": type(value) is dict,
                  "array": type(value) is list, "string": type(value) is str,
                  "boolean": type(value) is bool, "integer": type(value) is int,
                  "number": type(value) is int or (type(value) is float and math.isfinite(value))}
        require(any(checks.get(kind, False) for kind in kinds))
    if "const" in schema:
        require(value == schema["const"])
    if "enum" in schema:
        require(value in schema["enum"])
    if type(value) is dict:
        require(len(value) <= schema.get("maxProperties", len(value)))
        require(set(schema.get("required", [])) <= set(value))
        properties = schema.get("properties", {})
        extra = schema.get("additionalProperties", True)
        for key, item in value.items():
            if "propertyNames" in schema:
                validate_schema(key, schema["propertyNames"], location, root)
            if key in properties:
                validate_schema(item, properties[key], f"{location}.{key}", root)
            elif extra is False:
                require(False)
            elif isinstance(extra, dict):
                validate_schema(item, extra, location, root)
    if type(value) is list:
        require(len(value) <= schema.get("maxItems", len(value)))
        if schema.get("uniqueItems"):
            require(len({json_text(item) for item in value}) == len(value))
        if "items" in schema:
            for item in value:
                validate_schema(item, schema["items"], location, root)
    if type(value) is str:
        require(len(value) >= schema.get("minLength", 0))
        require(len(value) <= schema.get("maxLength", len(value)))
        if "pattern" in schema:
            require(re.search(schema["pattern"], value) is not None)
        if schema.get("format") == "date-time":
            require(re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]+)?(?:Z|[+-][0-9]{2}:[0-9]{2})", value) is not None)
            try:
                require(dt.datetime.fromisoformat(value.replace("Z", "+00:00")).tzinfo is not None)
            except ValueError:
                require(False)
    if type(value) in (int, float):
        require(type(value) is int or math.isfinite(value))
        require(value >= schema.get("minimum", value))
        require(value <= schema.get("maximum", value))


def read_schema(name):
    return load_json((Path(__file__).resolve().parent.parent / "schemas" / name).read_text(encoding="utf-8"))


def utc_now():
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def immutable_write(path, content):
    """Atomically publish complete bytes without replacing any previous object."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    data = content.encode("utf-8") if isinstance(content, str) else content
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".record-", delete=False) as handle:
            temporary = Path(handle.name)
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        try:
            os.link(temporary, path)
        except FileExistsError:
            if path.is_symlink() or path.read_bytes() != data:
                raise ValueError("immutable evidence already exists with different content")
        if os.name == "posix":
            descriptor = os.open(path.parent, os.O_RDONLY)
            try:
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def short_text(value, label, limit=256, nullable=False):
    if value is None and nullable:
        return None
    if not isinstance(value, str) or not value or len(value) > limit or any(ord(c) < 32 for c in value):
        raise ValueError(f"invalid {label}")
    return value


def env_text(name, limit=256):
    return short_text(os.environ.get(name) or "unknown", name, limit)


def configuration(text):
    value = load_json(text) if text else {}
    if not isinstance(value, dict) or set(value) - CONFIGURATION_KEYS:
        raise ValueError("configuration accepts only the documented allowlisted keys")
    result = {key: value.get(key) for key in sorted(CONFIGURATION_KEYS)}
    short_text(result["reasoning_effort"], "reasoning_effort", 64, nullable=True)
    for key in ("seed", "max_output_tokens"):
        item = result[key]
        if item is not None and (type(item) is not int or (key == "max_output_tokens" and item < 0)):
            raise ValueError(f"invalid {key}")
    item = result["temperature"]
    if item is not None and (type(item) not in (int, float) or not math.isfinite(item) or item < 0):
        raise ValueError("invalid temperature")
    item = result["configuration_sha256"]
    if item is not None and (not isinstance(item, str) or not re.fullmatch(r"[0-9a-f]{64}", item)):
        raise ValueError("invalid configuration_sha256")
    return result


def write_github(path, values):
    if path:
        with open(path, "a", encoding="utf-8") as handle:
            for key, value in values.items():
                if "\n" in str(value) or "\r" in str(value):
                    raise ValueError("newline in GitHub output")
                handle.write(f"{key}={value}\n")


def directory(args):
    value = args.directory or os.environ.get("OBSERVATORY_EVIDENCE_DIR")
    if not value:
        raise ValueError("pass --directory or run the action's setup step first")
    return Path(value).resolve()


def initialize(args):
    base = args.directory or os.environ.get("OBSERVATORY_EVIDENCE_DIR")
    if not base:
        base = str(Path(os.environ.get("RUNNER_TEMP", ".")) / f"observatory-evidence-{uuid.uuid4()}")
    root = Path(base).resolve()
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    matrix = load_json(args.matrix)
    if not isinstance(matrix, dict) or len(matrix) > 32:
        raise ValueError("matrix must be a flat object of at most 32 entries")
    for key, value in matrix.items():
        if not re.fullmatch(r"[A-Za-z0-9_.-]{1,64}", key):
            raise ValueError("invalid matrix key")
        if value is not None and type(value) not in (str, int, float, bool):
            raise ValueError("matrix values must be scalar")
        if isinstance(value, str):
            short_text(value, "matrix value", 512)
    attempt = os.environ.get("GITHUB_RUN_ATTEMPT")
    if attempt is not None and (not attempt.isdecimal() or int(attempt) < 1):
        raise ValueError("invalid GITHUB_RUN_ATTEMPT")
    platform = {
        "source": "runner_environment",
        "job_context_source": "caller",
        "repository_id": env_text("GITHUB_REPOSITORY_ID"),
        "repository": env_text("GITHUB_REPOSITORY"),
        "workflow_ref": env_text("GITHUB_WORKFLOW_REF", 512),
        "workflow_sha": env_text("GITHUB_WORKFLOW_SHA"),
        "run_id": env_text("GITHUB_RUN_ID"),
        "run_attempt": int(attempt) if attempt else None,
        "job_key": env_text("GITHUB_JOB"),
        "job_name": short_text(args.job_name or "unknown", "job name", 512),
        "job_id": args.job_id,
        "matrix": matrix,
        "actor": env_text("GITHUB_ACTOR"),
        "triggering_actor": env_text("GITHUB_TRIGGERING_ACTOR"),
        "event_name": env_text("GITHUB_EVENT_NAME"),
    }
    for key in ("repository_id", "run_id"):
        if platform[key] != "unknown" and not re.fullmatch(r"[0-9]+", platform[key]):
            raise ValueError(f"invalid {key}")
    if args.job_id is not None and not re.fullmatch(r"[0-9]+", args.job_id):
        raise ValueError("job ID must be a platform numeric ID")
    if platform["workflow_sha"] != "unknown" and not re.fullmatch(r"[0-9a-fA-F]{40}|[0-9a-fA-F]{64}", platform["workflow_sha"]):
        raise ValueError("invalid workflow SHA")
    declared = {"source": "caller", "controller": args.controller,
                "controller_provenance": args.controller_provenance,
                "protocol_id": args.protocol_id, "task_id": args.task_id,
                "session_id": args.session_id, "logical_call_id": None,
                "parent_invocation_id": None}
    for key in ("protocol_id", "task_id", "session_id"):
        short_text(declared[key], key, nullable=True)
    for value in declared["controller_provenance"]:
        short_text(value, "controller provenance", 512)
    context = {"platform": platform, "declared": declared}
    receipt_schema = read_schema("tool-receipt-v1.schema.json")
    validate_schema(platform, receipt_schema["properties"]["platform"], "platform")
    validate_schema(declared, receipt_schema["properties"]["declared"], "declared")
    immutable_write(root / "context.json", json_text(context))
    (root / "receipts").mkdir(exist_ok=True, mode=0o700)
    recorder = str(Path(__file__).resolve())
    write_github(args.github_env, {"OBSERVATORY_EVIDENCE_DIR": root, "OBSERVATORY_RECORDER": recorder})
    write_github(args.github_output, {"directory": root, "recorder": recorder})
    print(json_text({"directory": str(root), "recorder": recorder}))
    return 0


def artifact_spec(spec):
    name, separator, filename = spec.partition("=")
    if not separator:
        raise ValueError("artifacts use LABEL=PATH")
    short_text(name, "artifact label")
    if "/" in name or "\\" in name or name in (".", ".."):
        raise ValueError("artifact label must not be a path")
    return name, filename


def artifact(spec, role, missing):
    name, filename = artifact_spec(spec)
    path = Path(filename)
    if not path.exists() and role == "output":
        missing.append(f"artifacts.output.{name}" if re.fullmatch(r"[a-z][a-z0-9_.]{0,230}", name) else "artifacts.output")
        return None
    if not path.is_file():
        raise ValueError("designated artifact must be a regular file")
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
            size += len(chunk)
    return {"role": role, "name": name, "sha256": digest.hexdigest(), "size_bytes": size}


def find_missing(value, prefix=""):
    result = []
    if isinstance(value, dict):
        for key, item in value.items():
            name = f"{prefix}.{key}" if prefix else key
            if key not in ("result", "missing_fields") and (item is None or item == "unknown"):
                result.append(name)
            elif isinstance(item, dict):
                result.extend(find_missing(item, name))
    return result


def save_receipt(root, value):
    validate_schema(value, read_schema("tool-receipt-v1.schema.json"))
    serialized = json_text(value)
    if len(serialized.encode("utf-8")) > MAX_RECEIPT_BYTES:
        raise ValueError("receipt exceeds 256 KiB")
    immutable_write(root / "receipts" / f"{value['invocation_id']}.{value['phase']}.json", serialized)


class CodexUsage:
    # Verified against openai/codex commit 4b4a5cf25aa5b064c866218b405c848745fd280f,
    # codex-rs/exec/src/exec_events.rs (ThreadStartedEvent, TurnCompletedEvent, Usage).
    # Those events do not report model identity. Unknown fields (including a putative
    # model) remain ignored until a future adapter revision verifies their meaning.
    def __init__(self):
        self.turns = 0
        self.tokens = {"input_tokens": 0, "output_tokens": 0, "cached_input_tokens": 0}
        self.sessions = set()

    def observe(self, line):
        try:
            event = load_json(line)
        except (ValueError, UnicodeDecodeError):
            return
        if not isinstance(event, dict):
            return
        kind = event.get("type")
        if kind not in ("thread.started", "turn.completed"):
            return
        session = event.get("thread_id")
        if kind == "thread.started" and isinstance(session, str) and 0 < len(session) <= 256 and all(ord(c) >= 32 and not 127 <= ord(c) <= 159 for c in session):
            self.sessions.add(session)
        if kind == "turn.completed":
            self.turns += 1
            usage = event.get("usage")
            usage = usage if isinstance(usage, dict) else {}
            for key in self.tokens:
                value = usage.get(key)
                if type(value) is not int or not 0 <= value <= 2**64 - 1:
                    self.tokens[key] = None
                elif self.tokens[key] is not None:
                    total = self.tokens[key] + value
                    self.tokens[key] = total if total <= 2**64 - 1 else None
            observed_input = usage.get("input_tokens")
            observed_cached = usage.get("cached_input_tokens")
            if type(observed_input) is int and type(observed_cached) is int and observed_cached > observed_input:
                self.tokens["cached_input_tokens"] = None

    def result(self):
        return {**{key: value if self.turns else None for key, value in self.tokens.items()},
                "observed_model": None,
                "observed_session_id": next(iter(self.sessions)) if len(self.sessions) == 1 else None}


def run_tool(args):
    root = directory(args)
    context_path = root / "context.json"
    if context_path.stat().st_size > MAX_RECEIPT_BYTES:
        raise ValueError("context exceeds size limit")
    context = load_json(context_path.read_text(encoding="utf-8"))
    declared = context["declared"].copy()
    for key in ("protocol_id", "task_id", "session_id", "logical_call_id", "parent_invocation_id"):
        value = getattr(args, key)
        if value is not None:
            declared[key] = short_text(value, key)
    argv = args.argv[1:] if args.argv[:1] == ["--"] else args.argv
    if args.command == "codex":
        if any(arg in ("--model", "-m", "--json") or arg.startswith("--model=") for arg in argv):
            raise ValueError("use --configured-model; exec and --json are supplied by the adapter")
        argv = [args.executable, "exec", "--json"] + (["--model", args.configured_model] if args.configured_model else []) + argv
        tool_name = "codex"
        adapter = "codex_exec"
    else:
        if not argv:
            raise ValueError("a command is required after --")
        tool_name = args.tool
        adapter = "command"
    tool = {"adapter": adapter, "name": short_text(tool_name, "tool name"),
            "version": short_text(args.tool_version, "tool version", nullable=True),
            "purpose": args.purpose,
            "configured_model": short_text(args.configured_model, "configured model", nullable=True),
            "configuration": configuration(args.configuration)}
    short_text(args.check_name, "check name", nullable=True)
    for spec in args.output:
        artifact_spec(spec)
    receipt = {"schema": "observatory.tool-receipt/1", "invocation_id": str(uuid.uuid4()),
               "phase": "start", "timestamp": utc_now(), "platform": context["platform"],
               "declared": declared, "tool": tool, "result": None, "artifacts": [], "missing_fields": []}
    for spec in args.input:
        receipt["artifacts"].append(artifact(spec, "input", []))
    receipt["missing_fields"] = find_missing(receipt)
    save_receipt(root, receipt)
    started = time.monotonic_ns()
    process = None
    cancelled = None
    timer = None
    usage = CodexUsage()

    def forward(signum, _frame):
        nonlocal cancelled, timer
        cancelled = signum
        if process is not None:
            try:
                if os.name == "posix":
                    os.killpg(process.pid, signum)
                else:
                    process.terminate()
            except ProcessLookupError:
                pass
            if timer is None:
                def force_stop():
                    try:
                        if os.name == "posix":
                            os.killpg(process.pid, signal.SIGKILL)
                        elif process.poll() is None:
                            process.kill()
                    except ProcessLookupError:
                        pass
                timer = threading.Timer(5, force_stop)
                timer.daemon = True
                timer.start()

    previous = {sig: signal.signal(sig, forward) for sig in (signal.SIGINT, signal.SIGTERM)}
    try:
        try:
            process = subprocess.Popen(argv, stdout=subprocess.PIPE if adapter == "codex_exec" else None,
                                       start_new_session=os.name == "posix")
        except OSError:
            code = 127
        else:
            if cancelled is not None:
                forward(cancelled, None)
            if process.stdout is not None:
                with process.stdout:
                    while True:
                        line = process.stdout.readline(MAX_EVENT_BYTES + 1)
                        if not line:
                            break
                        if len(line) > MAX_EVENT_BYTES:
                            while line and not line.endswith(b"\n"):
                                line = process.stdout.readline(MAX_EVENT_BYTES + 1)
                            continue
                        usage.observe(line)
            code = process.wait()
    finally:
        if timer is not None:
            timer.cancel()
        if cancelled is not None and process is not None and os.name == "posix":
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        for sig, handler in previous.items():
            signal.signal(sig, handler)
    runtime_ms = max(0, (time.monotonic_ns() - started) // 1_000_000)
    status = "cancelled" if cancelled is not None or code in (-signal.SIGINT, -signal.SIGTERM) else ("success" if code == 0 else "failure")
    result = {"status": status, "exit_code": code, "runtime_ms": runtime_ms,
              "observed_model": None, "observed_session_id": None,
              "input_tokens": None, "output_tokens": None, "cached_input_tokens": None,
              "charge_amount": None, "charge_currency": None, "usage_source": "unknown", "checks": []}
    if adapter == "codex_exec":
        result.update(usage.result())
        if any(result[key] is not None for key in ("observed_model", "observed_session_id", "input_tokens", "output_tokens", "cached_input_tokens")):
            result["usage_source"] = "tool_reported"
    if args.check_name:
        result["checks"].append({"name": short_text(args.check_name, "check name"),
                                 "status": "passed" if status == "success" else "failed" if status == "failure" else "unknown"})
    receipt.update(phase="completion", timestamp=utc_now(), result=result)
    missing = []
    for spec in args.output:
        item = artifact(spec, "output", missing)
        if item is not None:
            receipt["artifacts"].append(item)
    receipt["missing_fields"] = sorted(set(find_missing(receipt) + find_missing(result, "result") + missing))
    save_receipt(root, receipt)
    return 128 + cancelled if cancelled is not None else (128 - code if code < 0 else code)


def publish(args):
    root = directory(args)
    records = []
    receipt_dir = root / "receipts"
    if not receipt_dir.is_dir():
        raise ValueError("no initialized receipt directory")
    for path in sorted(receipt_dir.iterdir()):
        if not re.fullmatch(r"[A-Za-z0-9._-]{1,128}\.(start|completion)\.json", path.name):
            raise ValueError("unexpected file in receipt directory")
        if path.is_symlink() or not stat.S_ISREG(path.stat().st_mode) or path.stat().st_size > MAX_RECEIPT_BYTES:
            raise ValueError("unsafe or oversized receipt file")
        data = path.read_bytes()
        receipt = load_json(data)
        validate_schema(receipt, read_schema("tool-receipt-v1.schema.json"))
        if receipt.get("schema") != "observatory.tool-receipt/1":
            raise ValueError("unsupported receipt schema")
        if path.name != f"{receipt.get('invocation_id')}.{receipt.get('phase')}.json":
            raise ValueError("receipt filename does not match identity")
        records.append({"sha256": hashlib.sha256(data).hexdigest(), "receipt_json": data.decode("utf-8")})
        if len(records) > 10000:
            raise ValueError("too many receipts")
    context = load_json((root / "context.json").read_text(encoding="utf-8"))
    for entry in records:
        if load_json(entry["receipt_json"])["platform"] != context["platform"]:
            raise ValueError("receipt platform differs from job context")
    value = {"schema": "observatory.execution-bundle/1", "platform": context["platform"], "receipts": records}
    validate_schema(value, read_schema("execution-bundle-v1.schema.json"))
    bundle = json_text(value).encode("utf-8")
    if len(bundle) > MAX_BUNDLE_BYTES:
        raise ValueError("bundle exceeds 16 MiB")
    digest = hashlib.sha256(bundle).hexdigest()
    output = Path(args.output).resolve() if args.output else root / "bundles" / digest / "bundle.json"
    immutable_write(output, bundle)
    # A fresh delivery name permits repeated uploads without overwriting old artifacts.
    # Receipt and bundle content identities remain stable for durable deduplication.
    artifact_name = f"observatory-executions-{digest}-{uuid.uuid4()}"
    write_github(args.github_output, {"path": output, "sha256": digest, "artifact-name": artifact_name})
    print(json_text({"path": str(output), "sha256": digest, "artifact_name": artifact_name, "receipts": len(records)}))
    return 0


def parser():
    main = argparse.ArgumentParser(description=__doc__)
    commands = main.add_subparsers(dest="command", required=True)
    setup = commands.add_parser("init", help="freeze allowlisted job context and caller declarations")
    setup.add_argument("--directory")
    setup.add_argument("--controller", choices=CONTROLLERS, default="unknown")
    setup.add_argument("--controller-provenance", action="append", default=[])
    for key in ("protocol-id", "task-id", "session-id", "job-id", "job-name", "github-output", "github-env"):
        setup.add_argument(f"--{key}")
    setup.add_argument("--matrix", default="{}")
    setup.set_defaults(function=initialize)
    for name in ("run", "codex"):
        command = commands.add_parser(name, help="execute one explicitly wrapped invocation")
        command.add_argument("--directory")
        if name == "run":
            command.add_argument("--tool", required=True)
        else:
            command.add_argument("--executable", default="codex")
        for key in ("tool-version", "configured-model", "protocol-id", "task-id", "session-id", "logical-call-id", "parent-invocation-id", "check-name"):
            command.add_argument(f"--{key}")
        command.add_argument("--purpose", choices=PURPOSES, default="unknown")
        command.add_argument("--configuration")
        command.add_argument("--input", action="append", default=[])
        command.add_argument("--output", action="append", default=[])
        command.add_argument("argv", nargs=argparse.REMAINDER)
        command.set_defaults(function=run_tool)
    upload = commands.add_parser("publish", help="package only immutable receipt JSON files")
    upload.add_argument("--directory")
    upload.add_argument("--output")
    upload.add_argument("--github-output")
    upload.set_defaults(function=publish)
    return main


def main():
    try:
        args = parser().parse_args()
        return args.function(args)
    except (ValueError, OSError, KeyError) as error:
        print(f"observatory recorder: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
