#!/usr/bin/env python3
"""Validate a hash-bound Lean proof and algorithm-refinement receipt."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import tomllib
from pathlib import Path
from typing import Any


SCHEMA = "formal-verification-receipt/1.0"
LEAN_TOOLCHAIN = "leanprover/lean4:v4.33.1"
MATHLIB_VERSION = "v4.33.1"
MATHLIB_REVISION = "0df444a360eaa60ab8c11dca51a86af692955474"
HEX_40 = re.compile(r"^[0-9a-f]{40}$")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
CLASSIFICATIONS = {
    "PURE_THEOREM",
    "FINITE_EXECUTABLE",
    "NUMERICAL_APPROXIMATE",
    "EMPIRICAL_CONTEXTUAL",
    "MIXED",
}
DISPOSITIONS = {
    "FORMALISE_NOW",
    "DEFERRED_DEPENDENCY",
    "EMPIRICAL_CONTEXTUAL",
    "STATEMENT_REPAIR_REQUIRED",
    "NOT_REQUIRED",
}
PROOF_STATUSES = {
    "KERNEL_CHECKED",
    "CHECKED_WITH_DECLARED_AXIOMS",
    "STATEMENT_MISMATCH",
    "NOT_FORMALISED",
    "NOT_REQUIRED",
}
RELATIONSHIPS = {
    "LEAN_CANONICAL",
    "FINITE_EXHAUSTIVE_DIFFERENTIAL",
    "SEMANTICS_REFINEMENT_PROVED",
    "TESTED_ONLY",
    "NONE",
}
ALGORITHM_STATUSES = {
    "LEAN_CANONICAL_CHECKED",
    "FINITE_REFINEMENT_CHECKED",
    "SEMANTICS_REFINEMENT_PROVED",
    "TESTED_ONLY",
    "NOT_REQUIRED",
}
EXPECTED_ALGORITHM_STATUS = {
    "LEAN_CANONICAL": "LEAN_CANONICAL_CHECKED",
    "FINITE_EXHAUSTIVE_DIFFERENTIAL": "FINITE_REFINEMENT_CHECKED",
    "SEMANTICS_REFINEMENT_PROVED": "SEMANTICS_REFINEMENT_PROVED",
    "TESTED_ONLY": "TESTED_ONLY",
    "NONE": "NOT_REQUIRED",
}
PERMITTED_AXIOMS = {"propext", "Classical.choice", "Quot.sound"}
CONCORDANCE_OUTCOMES = {"MATCHED", "MISMATCHED", "NOT_APPLICABLE"}
FORBIDDEN_LEAN = re.compile(
    r"\b(sorry|admit|sorryAx|axiom|constant|partial|unsafe|"
    r"implemented_by|extern|native_decide)\b"
)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def strip_lean_comments_and_strings(text: str) -> str:
    output: list[str] = []
    index = 0
    block_depth = 0
    in_string = False
    escaped = False
    while index < len(text):
        pair = text[index : index + 2]
        char = text[index]
        if block_depth:
            if pair == "/-":
                block_depth += 1
                output.extend("  ")
                index += 2
            elif pair == "-/":
                block_depth -= 1
                output.extend("  ")
                index += 2
            else:
                output.append("\n" if char == "\n" else " ")
                index += 1
            continue
        if in_string:
            output.append("\n" if char == "\n" else " ")
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            index += 1
            continue
        if pair == "--":
            newline = text.find("\n", index)
            if newline == -1:
                output.extend(" " * (len(text) - index))
                break
            output.extend(" " * (newline - index))
            index = newline
        elif pair == "/-":
            block_depth = 1
            output.extend("  ")
            index += 2
        elif char == '"':
            in_string = True
            output.append(" ")
            index += 1
        else:
            output.append(char)
            index += 1
    return "".join(output)


def require_string(value: Any, field: str, errors: list[str]) -> str:
    if not isinstance(value, str) or not value.strip():
        errors.append(f"{field}: expected a non-empty string")
        return ""
    return value


def resolve_project_file(
    base: Path, relative: str, field: str, errors: list[str]
) -> Path | None:
    candidate = Path(relative)
    if candidate.is_absolute():
        errors.append(f"{field}: expected a project-relative path")
        return None
    try:
        resolved = (base / candidate).resolve(strict=True)
    except OSError:
        errors.append(f"{field}: missing file {relative}")
        return None
    try:
        resolved.relative_to(base)
    except ValueError:
        errors.append(f"{field}: path escapes the project root")
        return None
    if not resolved.is_file():
        errors.append(f"{field}: missing file {relative}")
        return None
    return resolved


def check_path_hash(
    base: Path, record: dict[str, Any], prefix: str, errors: list[str]
) -> Path | None:
    relative = require_string(record.get("path"), f"{prefix}.path", errors)
    expected = require_string(record.get("sha256"), f"{prefix}.sha256", errors)
    if expected and not HEX_64.fullmatch(expected):
        errors.append(f"{prefix}.sha256: expected 64 lowercase hexadecimal characters")
    if not relative:
        return None
    path = resolve_project_file(base, relative, f"{prefix}.path", errors)
    if path is None:
        return None
    if expected and HEX_64.fullmatch(expected) and sha256(path) != expected:
        errors.append(f"{prefix}: hash mismatch for {relative}")
    return path


def validate(receipt: dict[str, Any], base: Path) -> list[str]:
    errors: list[str] = []
    if receipt.get("schema_version") != SCHEMA:
        errors.append(f"schema_version: expected {SCHEMA}")

    snapshot = receipt.get("source_snapshot")
    if not isinstance(snapshot, dict):
        errors.append("source_snapshot: expected an object")
        snapshot = {}
    require_string(snapshot.get("repository"), "source_snapshot.repository", errors)
    commit = require_string(snapshot.get("commit"), "source_snapshot.commit", errors)
    if commit and not HEX_40.fullmatch(commit):
        errors.append("source_snapshot.commit: expected a 40-character lowercase Git SHA")

    toolchain = receipt.get("toolchain")
    if not isinstance(toolchain, dict):
        errors.append("toolchain: expected an object")
        toolchain = {}
    if toolchain.get("lean") != LEAN_TOOLCHAIN:
        errors.append(f"toolchain.lean: expected {LEAN_TOOLCHAIN}")
    if toolchain.get("mathlib") != MATHLIB_VERSION:
        errors.append(f"toolchain.mathlib: expected {MATHLIB_VERSION}")
    lakefile_hash = require_string(
        toolchain.get("lakefile_sha256"),
        "toolchain.lakefile_sha256",
        errors,
    )
    if lakefile_hash and not HEX_64.fullmatch(lakefile_hash):
        errors.append(
            "toolchain.lakefile_sha256: expected 64 lowercase hexadecimal characters"
        )
    manifest_hash = require_string(
        toolchain.get("lake_manifest_sha256"),
        "toolchain.lake_manifest_sha256",
        errors,
    )
    if manifest_hash and not HEX_64.fullmatch(manifest_hash):
        errors.append(
            "toolchain.lake_manifest_sha256: expected 64 lowercase "
            "hexadecimal characters"
        )

    lakefile = resolve_project_file(base, "lakefile.toml", "lakefile.toml", errors)
    if lakefile is not None:
        if (
            lakefile_hash
            and HEX_64.fullmatch(lakefile_hash)
            and sha256(lakefile) != lakefile_hash
        ):
            errors.append("lakefile.toml hash mismatch")
        try:
            lake_config = tomllib.loads(lakefile.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError) as error:
            errors.append(f"lakefile.toml: cannot parse: {error}")
            lake_config = {}
        requirements = lake_config.get("require", [])
        if not isinstance(requirements, list):
            requirements = []
        mathlib_requirements = [
            item
            for item in requirements
            if isinstance(item, dict) and item.get("name") == "mathlib"
        ]
        if len(mathlib_requirements) != 1:
            errors.append("lakefile.toml: expected exactly one mathlib requirement")
        else:
            requirement = mathlib_requirements[0]
            if requirement.get("scope") != "leanprover-community":
                errors.append("lakefile.toml: mathlib scope must be leanprover-community")
            if requirement.get("rev") != MATHLIB_VERSION:
                errors.append(f"lakefile.toml: mathlib rev must be {MATHLIB_VERSION}")

    manifest = resolve_project_file(
        base, "lake-manifest.json", "lake-manifest.json", errors
    )
    if manifest is not None:
        if (
            manifest_hash
            and HEX_64.fullmatch(manifest_hash)
            and sha256(manifest) != manifest_hash
        ):
            errors.append("lake-manifest.json hash mismatch")
        try:
            manifest_data = json.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            errors.append(f"lake-manifest.json: cannot parse: {error}")
            manifest_data = {}
        if not isinstance(manifest_data, dict):
            errors.append("lake-manifest.json: root must be an object")
            manifest_data = {}
        packages = manifest_data.get("packages", [])
        if not isinstance(packages, list):
            packages = []
        mathlib_packages = [
            item
            for item in packages
            if isinstance(item, dict) and item.get("name") == "mathlib"
        ]
        if len(mathlib_packages) != 1:
            errors.append("lake-manifest.json: expected exactly one mathlib package")
        else:
            package = mathlib_packages[0]
            if package.get("scope") != "leanprover-community":
                errors.append("lake-manifest.json: mathlib scope must be leanprover-community")
            if package.get("inputRev") != MATHLIB_VERSION:
                errors.append(
                    f"lake-manifest.json: mathlib inputRev must be {MATHLIB_VERSION}"
                )
            if package.get("rev") != MATHLIB_REVISION:
                errors.append(
                    f"lake-manifest.json: mathlib revision must be {MATHLIB_REVISION}"
                )

    lean_toolchain = resolve_project_file(
        base, "lean-toolchain", "lean-toolchain", errors
    )
    if (
        lean_toolchain is not None
        and lean_toolchain.read_text(encoding="utf-8").strip() != LEAN_TOOLCHAIN
    ):
        errors.append(f"lean-toolchain: expected {LEAN_TOOLCHAIN}")

    formal_sources = receipt.get("formal_sources")
    if not isinstance(formal_sources, list) or not formal_sources:
        errors.append("formal_sources: expected a non-empty array")
        formal_sources = []
    resolved_formal_paths: set[str] = set()
    for index, source in enumerate(formal_sources):
        prefix = f"formal_sources[{index}]"
        if not isinstance(source, dict):
            errors.append(f"{prefix}: expected an object")
            continue
        relative = require_string(source.get("path"), f"{prefix}.path", errors)
        if not relative:
            continue
        if relative in resolved_formal_paths:
            errors.append(f"formal_sources: duplicate path {relative}")
            continue
        resolved_formal_paths.add(relative)
        path = check_path_hash(base, source, prefix, errors)
        if path is None:
            continue
        stripped = strip_lean_comments_and_strings(path.read_text(encoding="utf-8"))
        match = FORBIDDEN_LEAN.search(stripped)
        if match:
            line = stripped.count("\n", 0, match.start()) + 1
            errors.append(f"{relative}:{line}: forbidden Lean escape {match.group(1)}")

    inventory = receipt.get("inventory_ids")
    if not isinstance(inventory, list) or not inventory:
        errors.append("inventory_ids: expected a non-empty array")
        inventory = []
    if any(not isinstance(item, str) or not item for item in inventory):
        errors.append("inventory_ids: every identifier must be a non-empty string")
    if len(inventory) != len(set(inventory)):
        errors.append("inventory_ids: duplicate identifier")

    results = receipt.get("results")
    if not isinstance(results, list):
        errors.append("results: expected an array")
        results = []
    result_ids: list[str] = []
    for index, result in enumerate(results):
        prefix = f"results[{index}]"
        if not isinstance(result, dict):
            errors.append(f"{prefix}: expected an object")
            continue
        result_id = require_string(result.get("result_id"), f"{prefix}.result_id", errors)
        if result_id:
            result_ids.append(result_id)
        if result.get("classification") not in CLASSIFICATIONS:
            errors.append(f"{prefix}.classification: invalid value")
        if result.get("disposition") not in DISPOSITIONS:
            errors.append(f"{prefix}.disposition: invalid value")

        source = result.get("source")
        if not isinstance(source, dict):
            errors.append(f"{prefix}.source: expected an object")
        else:
            check_path_hash(base, source, f"{prefix}.source", errors)
            require_string(
                source.get("statement_locator"),
                f"{prefix}.source.statement_locator",
                errors,
            )

        formal = result.get("formal_statement")
        if not isinstance(formal, dict):
            errors.append(f"{prefix}.formal_statement: expected an object")
            formal = {}
        proof_status = formal.get("status")
        if proof_status not in PROOF_STATUSES:
            errors.append(f"{prefix}.formal_statement.status: invalid value")
        disposition = result.get("disposition")
        required_proof_statuses = {
            "FORMALISE_NOW": {"KERNEL_CHECKED", "CHECKED_WITH_DECLARED_AXIOMS"},
            "DEFERRED_DEPENDENCY": {"NOT_FORMALISED"},
            "STATEMENT_REPAIR_REQUIRED": {"STATEMENT_MISMATCH"},
            "NOT_REQUIRED": {"NOT_REQUIRED"},
        }
        if (
            disposition in required_proof_statuses
            and proof_status not in required_proof_statuses[disposition]
        ):
            expected = ", ".join(sorted(required_proof_statuses[disposition]))
            errors.append(
                f"{prefix}.formal_statement.status: disposition {disposition} "
                f"requires one of {expected}"
            )
        formal_path = formal.get("path")
        checked = proof_status in {"KERNEL_CHECKED", "CHECKED_WITH_DECLARED_AXIOMS"}
        if checked:
            require_string(
                formal.get("declaration"),
                f"{prefix}.formal_statement.declaration",
                errors,
            )
            if formal_path not in resolved_formal_paths:
                errors.append(f"{prefix}.formal_statement.path: not in formal_sources")
        axioms = formal.get("axioms")
        if not isinstance(axioms, list) or any(not isinstance(item, str) for item in axioms):
            errors.append(f"{prefix}.formal_statement.axioms: expected an array of strings")
            axioms = []
        unexpected_axioms = set(axioms) - PERMITTED_AXIOMS
        if unexpected_axioms:
            errors.append(
                f"{prefix}.formal_statement.axioms: unpermitted axioms "
                + ", ".join(sorted(unexpected_axioms))
            )
        if proof_status == "KERNEL_CHECKED" and axioms:
            errors.append(f"{prefix}: KERNEL_CHECKED must have an empty axiom list")
        if proof_status == "CHECKED_WITH_DECLARED_AXIOMS" and not axioms:
            errors.append(f"{prefix}: CHECKED_WITH_DECLARED_AXIOMS requires listed axioms")

        supporting = result.get("supporting_formal_statements", [])
        if not isinstance(supporting, list):
            errors.append(
                f"{prefix}.supporting_formal_statements: expected an array"
            )
            supporting = []
        for support_index, support in enumerate(supporting):
            support_prefix = (
                f"{prefix}.supporting_formal_statements[{support_index}]"
            )
            if not isinstance(support, dict):
                errors.append(f"{support_prefix}: expected an object")
                continue
            require_string(
                support.get("declaration"),
                f"{support_prefix}.declaration",
                errors,
            )
            support_status = support.get("status")
            if support_status not in {
                "KERNEL_CHECKED",
                "CHECKED_WITH_DECLARED_AXIOMS",
            }:
                errors.append(f"{support_prefix}.status: invalid checked status")
            support_axioms = support.get("axioms")
            if not isinstance(support_axioms, list) or any(
                not isinstance(item, str) for item in support_axioms
            ):
                errors.append(f"{support_prefix}.axioms: expected an array of strings")
                support_axioms = []
            unexpected_support_axioms = set(support_axioms) - PERMITTED_AXIOMS
            if unexpected_support_axioms:
                errors.append(
                    f"{support_prefix}.axioms: unpermitted axioms "
                    + ", ".join(sorted(unexpected_support_axioms))
                )
            if support_status == "KERNEL_CHECKED" and support_axioms:
                errors.append(
                    f"{support_prefix}: KERNEL_CHECKED must have an empty axiom list"
                )
            if (
                support_status == "CHECKED_WITH_DECLARED_AXIOMS"
                and not support_axioms
            ):
                errors.append(
                    f"{support_prefix}: CHECKED_WITH_DECLARED_AXIOMS "
                    "requires listed axioms"
                )

        concordance = result.get("semantic_concordance")
        if not isinstance(concordance, dict) or concordance.get("status") != "REVIEWED":
            errors.append(f"{prefix}.semantic_concordance.status: expected REVIEWED")
        else:
            concordance_outcome = concordance.get("outcome")
            if concordance_outcome not in CONCORDANCE_OUTCOMES:
                errors.append(
                    f"{prefix}.semantic_concordance.outcome: invalid value"
                )
            elif checked and concordance_outcome != "MATCHED":
                errors.append(
                    f"{prefix}.semantic_concordance.outcome: a checked statement "
                    "requires MATCHED"
                )
            elif (
                proof_status == "STATEMENT_MISMATCH"
                and concordance_outcome != "MISMATCHED"
            ):
                errors.append(
                    f"{prefix}.semantic_concordance.outcome: "
                    "STATEMENT_MISMATCH requires MISMATCHED"
                )
            require_string(
                concordance.get("reviewer"),
                f"{prefix}.semantic_concordance.reviewer",
                errors,
            )
            mapping = concordance.get("mapping")
            if not isinstance(mapping, dict):
                errors.append(
                    f"{prefix}.semantic_concordance.mapping: expected an object"
                )
            else:
                check_path_hash(
                    base,
                    mapping,
                    f"{prefix}.semantic_concordance.mapping",
                    errors,
                )
            differences = concordance.get("differences")
            if not isinstance(differences, list) or any(
                not isinstance(item, str) for item in differences
            ):
                errors.append(
                    f"{prefix}.semantic_concordance.differences: "
                    "expected an array of strings"
                )

        algorithm = result.get("algorithm")
        if algorithm is not None:
            if not isinstance(algorithm, dict):
                errors.append(f"{prefix}.algorithm: expected an object or null")
            else:
                require_string(
                    algorithm.get("algorithm_id"),
                    f"{prefix}.algorithm.algorithm_id",
                    errors,
                )
                relationship = algorithm.get("relationship")
                if relationship not in RELATIONSHIPS:
                    errors.append(f"{prefix}.algorithm.relationship: invalid value")
                algorithm_status = algorithm.get("status")
                if algorithm_status not in ALGORITHM_STATUSES:
                    errors.append(f"{prefix}.algorithm.status: invalid value")
                elif (
                    relationship in EXPECTED_ALGORITHM_STATUS
                    and algorithm_status != EXPECTED_ALGORITHM_STATUS[relationship]
                ):
                    errors.append(
                        f"{prefix}.algorithm.status: relationship {relationship} "
                        f"requires {EXPECTED_ALGORITHM_STATUS[relationship]}"
                    )
                sources = algorithm.get("implementation_sources")
                if not isinstance(sources, list):
                    errors.append(
                        f"{prefix}.algorithm.implementation_sources: expected an array"
                    )
                    sources = []
                if relationship != "NONE" and not sources:
                    errors.append(
                        f"{prefix}.algorithm.implementation_sources: expected at least one source"
                    )
                implementation_paths: set[str] = set()
                for source_index, source in enumerate(sources):
                    source_prefix = (
                        f"{prefix}.algorithm.implementation_sources[{source_index}]"
                    )
                    if not isinstance(source, dict):
                        errors.append(f"{source_prefix}: expected an object")
                        continue
                    relative = source.get("path")
                    if isinstance(relative, str) and relative in implementation_paths:
                        errors.append(f"{source_prefix}.path: duplicate path {relative}")
                    elif isinstance(relative, str):
                        implementation_paths.add(relative)
                    check_path_hash(base, source, source_prefix, errors)
                if relationship not in {"TESTED_ONLY", "NONE"}:
                    require_string(
                        algorithm.get("lean_definition"),
                        f"{prefix}.algorithm.lean_definition",
                        errors,
                    )
                    require_string(
                        algorithm.get("domain_coverage"),
                        f"{prefix}.algorithm.domain_coverage",
                        errors,
                    )

        checks = result.get("checks")
        if not isinstance(checks, list) or not checks:
            errors.append(f"{prefix}.checks: expected a non-empty array")
            checks = []
        for check_index, check in enumerate(checks):
            check_prefix = f"{prefix}.checks[{check_index}]"
            if not isinstance(check, dict):
                errors.append(f"{check_prefix}: expected an object")
                continue
            require_string(check.get("command"), f"{check_prefix}.command", errors)
            if check.get("exit_status") != 0:
                errors.append(f"{check_prefix}.exit_status: expected 0")
            check_path_hash(
                base,
                {
                    "path": check.get("output_path"),
                    "sha256": check.get("output_sha256"),
                },
                f"{check_prefix}.output",
                errors,
            )

        require_string(result.get("scope_boundary"), f"{prefix}.scope_boundary", errors)

    if result_ids != inventory:
        errors.append("results identifiers and order must match inventory_ids exactly")
    require_string(receipt.get("scope_boundary"), "scope_boundary", errors)
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("receipt", type=Path)
    parser.add_argument("--base", type=Path, default=Path.cwd())
    args = parser.parse_args()

    try:
        receipt = json.loads(args.receipt.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        print(json.dumps({"status": "ERROR", "errors": [str(error)]}, indent=2))
        return 2
    if not isinstance(receipt, dict):
        print(
            json.dumps(
                {"status": "ERROR", "errors": ["receipt root must be an object"]},
                indent=2,
            )
        )
        return 2

    errors = validate(receipt, args.base.resolve())
    print(json.dumps({"status": "PASS" if not errors else "FAIL", "errors": errors}, indent=2))
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
