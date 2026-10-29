#!/usr/bin/env python3
"""Validate this project's review schema and current hashes; not the skill validator."""
import hashlib
import json
from pathlib import Path

import jsonschema

HERE = Path(__file__).resolve().parent
PAPER = HERE.parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check():
    review_path = HERE / "horizon-performance.review.json"
    spec_path = PAPER / "horizon-performance.spec.json"
    audit_path = HERE / "horizon-performance.audit.json"
    review = json.loads(review_path.read_text())
    audit = json.loads(audit_path.read_text())
    spec = json.loads(spec_path.read_text())
    schema = json.loads((HERE / "schemas/figure-review.schema.json").read_text())
    jsonschema.Draft202012Validator(schema).validate(review)
    assert review["spec_sha256"] == digest(spec_path), "Stale specification review"
    assert review["audit_sha256"] == digest(audit_path), "Stale audit review"
    assert audit["source_sha256"] == digest(PAPER / audit["source"]), "Stale source audit"
    assert audit["pdf_sha256"] == digest(PAPER / audit["pdf"]), "Stale PDF audit"
    assert audit["spec_sha256"] == digest(spec_path), "Stale specification audit"
    for item in review["artifacts"] + audit["inputs"]:
        assert item["sha256"] == digest(PAPER / item["path"]), f"Stale artifact: {item['path']}"
    for item in spec["inputs"]:
        if "path" in item and "sha256" in item:
            assert item["sha256"] == digest(PAPER / item["path"]), f"Stale input: {item['path']}"
    if review["decision"] == "accept":
        assert not audit["errors"], "Mechanical errors prevent acceptance"
        assert all(f["disposition"] != "unresolved" for f in review["comparison"]["findings"])
        expected = set(range(len(audit["warnings"]) + len(audit["typesetting_warnings"])))
        covered = {d["index"] for d in review["comparison"]["audit_warning_dispositions"]}
        assert expected == covered, "Every audit warning requires a disposition"
    print("Project-local review schema and current artifact/input hashes verified.")
    print("This is not validation by the unavailable bundled skill scripts.")


if __name__ == "__main__":
    check()
