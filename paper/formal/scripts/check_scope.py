"""Check the proof-scope inventory; this does not execute or certify a proof."""
from pathlib import Path
import json
import hashlib

root = Path(__file__).resolve().parents[1]
scope = json.loads((root / "scope.json").read_text())
receipt = json.loads((root / "formal-verification-receipt.json").read_text())
checker_scripts = receipt.get("checker_scripts", [])
assert {item["path"] for item in checker_scripts} == {
    "scripts/check_scope.py", "scripts/verify.sh", "scripts/validate_formal_receipt.py"
}, "The checker-script inventory is incomplete"
for item in checker_scripts:
    assert hashlib.sha256((root / item["path"]).read_bytes()).hexdigest() == item["sha256"], item["path"]
identity = receipt["recorded_toolchain_identity"]
assert hashlib.sha256((root / identity["path"]).read_bytes()).hexdigest() == identity["sha256"], identity["path"]
manuscript_binding = receipt["current_manuscript"]
binding_path = root / manuscript_binding["path"]
assert hashlib.sha256(binding_path.read_bytes()).hexdigest() == manuscript_binding["sha256"], manuscript_binding["path"]
binding = json.loads(binding_path.read_text())
assert binding["repository_relative_path"] == "paper/article/partial-identification.tex"
canonical_article = root.parent / "article" / "partial-identification.tex"
assert hashlib.sha256(canonical_article.read_bytes()).hexdigest() == binding["sha256"], "Current manuscript changed; renew its independent concordance binding."
assert hashlib.sha256((root / "targets/partial-identification.tex").read_bytes()).hexdigest() == binding["frozen_target"]["sha256"], "Frozen target changed."
assert binding["independent_review"]["outcome"] == "MATHEMATICAL_STATEMENTS_UNCHANGED"
current_review = binding["independent_review"]["report"]
assert hashlib.sha256((root / current_review["path"]).read_bytes()).hexdigest() == current_review["sha256"], current_review["path"]
expected = {item["id"]: item["disposition"] for item in scope["targets"]}
actual = {item["result_id"]: item for item in receipt["results"]}
assert set(expected) == set(actual), "Scope and receipt inventories differ"
for ident, disposition in expected.items():
    item = actual[ident]
    assert item["disposition"] == disposition, ident
    status = item["formal_statement"]["status"]
    if disposition == "FORMALISE_NOW":
        assert status in {"KERNEL_CHECKED", "CHECKED_WITH_DECLARED_AXIOMS"}, ident
    elif disposition == "NOT_REQUIRED":
        assert status == "NOT_REQUIRED", ident
    else:
        assert status == "NOT_FORMALISED", ident
print(json.dumps({"status": "PASS", "inventory_items": len(actual),
    "checked_component_groups": sum(v == "FORMALISE_NOW" for v in expected.values()),
    "checker_scripts_sha256": "MATCHED",
    "recorded_toolchain_identity_sha256": "MATCHED",
    "current_manuscript_sha256": "MATCHED",
    "other_dispositions": {k: v for k, v in expected.items() if v != "FORMALISE_NOW"},
    "scope": "Inventory and exclusion consistency only; not a proof of excluded or empirical claims."}, indent=2))
