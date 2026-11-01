#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"
if ! command -v lake >/dev/null 2>&1; then
  printf 'Pinned Lean tools are missing from PATH. Install Lean 4.33.1 using the repository bootstrap and retry.\n' >&2
  exit 2
fi
run_evidence="$project_root/.lake/verification/$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$run_evidence"
lake env lean --version > "$run_evidence/toolchain.log"
IFS= read -r version_line < "$run_evidence/toolchain.log"
if [[ "$version_line" != *'version 4.33.1,'* ]]; then
  printf 'Expected Lean 4.33.1; inspect %s\n' "$run_evidence/toolchain.log" >&2
  exit 2
fi
printf 'Building the pinned Lean project.\n'
lake build > "$run_evidence/lake-build.log" 2>&1
printf 'Auditing every exported theorem axiom list.\n'
lake env lean AxiomAudit.lean > "$run_evidence/axiom-audit.log" 2>&1
printf 'Replaying local and imported constants from an empty Lean environment.\n'
lake env leanchecker --fresh -v LabourBounds > "$run_evidence/fresh-replay.log" 2>&1
python3 scripts/check_scope.py > "$run_evidence/scope-check.json"
python3 scripts/validate_formal_receipt.py formal-verification-receipt.json --base . \
  > "$run_evidence/receipt-validation.json"
printf 'Fresh build, axiom audit and fresh kernel replay passed. Evidence: %s\n' "$run_evidence"
