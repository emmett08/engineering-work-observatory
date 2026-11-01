#!/usr/bin/env python3
"""Non-mutating project-local checks; not the unavailable bundled skill validator."""
from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

import jsonschema

HERE = Path(__file__).resolve().parent
PAPER = HERE.parents[1]
LIMITATION = "Human interpretation benefit remains unmeasured by agent screening."
module_spec = importlib.util.spec_from_file_location("shared_capacity_render", HERE / "render.py")
renderer = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(renderer)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def read(path: Path) -> dict:
    return json.loads(path.read_text())


def check_hash(relative: str, digest: str) -> None:
    path = renderer.local(relative)
    require(path.is_file(), f"Missing file: {relative}")
    require(renderer.sha(path) == digest, f"Stale hash: {relative}")


def unique_ids(items: list, group: str) -> set:
    ids = [item["id"] for item in items]
    require(len(ids) == len(set(ids)), f"Duplicate IDs in {group}")
    return set(ids)


def check_review(spec: dict, spec_path: Path, audit: dict, audit_path: Path) -> dict:
    review = read(renderer.local(spec["build"]["outputs"]["review"]))
    jsonschema.Draft202012Validator(read(HERE / "schemas/figure-review.schema.json")).validate(review)
    require(review["figure_id"] == spec["figure_id"], "Review figure ID mismatch")
    require(review["spec_sha256"] == renderer.sha(spec_path), "Review binds a stale specification")
    require(review.get("audit_sha256") == renderer.sha(audit_path), "Review binds a stale audit")
    pairs = [(a["role"], a["path"]) for a in review["artifacts"]]
    require(len(pairs) == len(set(pairs)), "Duplicate review artifact roles/paths")
    for artifact in review["artifacts"]:
        check_hash(artifact["path"], artifact["sha256"])
    finals = [a for a in review["artifacts"] if a["role"] == "final_pdf"]
    require(len(finals) == 1, "Review must bind exactly one final PDF")
    require(finals[0]["path"] == spec["build"]["outputs"]["pdf"], "Review final PDF differs from output")
    candidates = unique_ids(review["candidates"], "review candidates")
    require(review["selected_candidate"] in candidates, "Selected candidate is missing")
    prototypes = {a["path"] for a in review["artifacts"] if a["role"] == "prototype"}
    for candidate in review["candidates"]:
        require(candidate["prototype"] in prototypes, f"Unbound prototype: {candidate['prototype']}")
    reviewer = review["reviewer"]
    if reviewer["kind"] == "agent":
        require(reviewer["scope"] == "agent_screening", "Agent review must declare agent screening")
    if reviewer["scope"] == "agent_screening":
        require(LIMITATION in review["limitations"], "Required agent-screening limitation is missing")
    if reviewer["scope"] == "reader_test":
        require(reviewer["kind"] == "human", "Reader-test review requires a human reviewer")
        evidence = reviewer["test_evidence"]
        check_hash(evidence["path"], evidence["sha256"])
    warnings = audit.get("warnings", []) + audit.get("typesetting_warnings", [])
    dispositions = review["comparison"]["audit_warning_dispositions"]
    indices = [item["index"] for item in dispositions]
    require(len(indices) == len(set(indices)), "Duplicate warning dispositions")
    require(set(indices) == set(range(len(warnings))), "Warning dispositions do not cover the current audit")
    require(review["decision"] == "accept", "Review has not accepted the current figure")
    for finding in review["comparison"]["findings"]:
        require(finding["disposition"] != "unresolved", f"Unresolved review finding: {finding['id']}")
        if finding["severity"] == "blocking":
            require(finding["disposition"] == "resolved", f"Blocking finding is not resolved: {finding['id']}")
    return review


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, default=PAPER / "shared-capacity.spec.json")
    parser.add_argument("--review", action="store_true", help="Also require a current accepted review")
    args = parser.parse_args()
    spec_path = args.spec.resolve()
    require(spec_path.is_relative_to(PAPER), "Specification must be within the paper root")
    spec = renderer.validate_spec(spec_path)
    ids = {name: unique_ids(spec.get(name, []), name)
           for name in ("inputs", "quantities", "constructions", "encodings")}
    for name in ("quantities", "constructions"):
        for item in spec.get(name, []):
            require(set(item["input_ids"]) <= ids["inputs"], f"Invalid input reference in {item['id']}")
    for encoding in spec["encodings"]:
        for key, group in (("quantity_ids", "quantities"), ("construction_ids", "constructions")):
            require(set(encoding.get(key, [])) <= ids[group], f"Wrong reference type in {encoding['id']}")
    declared = set(spec["build"]["dependencies"])
    declared.update(item["path"] for item in spec["inputs"] if "path" in item)
    profiles = [spec["target"].get("manuscript_profile"), spec["build"].get("profile")]
    declared.update(path for path in profiles if path)
    if all(profiles):
        require(renderer.local(profiles[0]) == renderer.local(profiles[1]), "Conflicting manuscript profiles")
    for relative in declared:
        require(renderer.local(relative).is_file(), f"Missing declared file: {relative}")
    audit_path = renderer.local(spec["build"]["outputs"]["audit"])
    audit = read(audit_path)
    require(audit.get("errors") == [], "Mechanical audit is missing its successful errors list")
    require(audit.get("source") == spec["build"]["source"], "Audit source path mismatch")
    require(audit.get("pdf") == spec["build"]["outputs"]["pdf"], "Audit PDF path mismatch")
    check_hash(audit["source"], audit["source_sha256"])
    check_hash(audit["pdf"], audit["pdf_sha256"])
    require(audit.get("spec_sha256") == renderer.sha(spec_path), "Audit binds a stale specification")
    audited = [item["path"] for item in audit["inputs"]]
    require(len(audited) == len(set(audited)), "Duplicate audited inputs")
    require(set(audited) == declared, "Audit does not bind exactly the declared inputs/dependencies")
    for item in audit["inputs"]:
        check_hash(item["path"], item["sha256"])
    result = subprocess.run([sys.executable, str(HERE / "calculate.py"), "--check"],
                            capture_output=True, text=True, check=True)
    review = check_review(spec, spec_path, audit, audit_path) if args.review else None
    print(json.dumps({"status": "PASS", "scope": "project-local bounded checks",
                      "figure_id": spec["figure_id"], "declared_files": len(declared),
                      "calculation": json.loads(result.stdout),
                      "review": review["decision"] if review else "not requested"}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, jsonschema.ValidationError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"FAIL: {error}") from error
