# Pinned Lean supplement

Lean 4.33.1 and mathlib v4.33.1 check 43 finite, algebraic and convergent-series
results. `claim-inventory.json` accounts for 54 entries, including deferred and
empirical claims. `semantic-map.md` and `evidence/reaudit-concordance.md` explain
the relationship between the actual Lean statements and the manuscript.

With the pinned toolchain on PATH, run from this directory:

```sh
lake exe cache get Mathlib.Data.Real.Basic Mathlib.Data.Nat.Choose.Basic Mathlib.Data.Fintype.BigOperators Mathlib.Tactic.NormNum Mathlib.Tactic.Ring Mathlib.Tactic.Linarith Mathlib.Tactic.FieldSimp Mathlib.Tactic.Positivity Mathlib.Analysis.SpecificLimits.Normed Mathlib.Data.Nat.Choose.Cast
lake build
lake env lean AxiomAudit.lean
python3 scripts/validate_formal_receipt.py formal-verification-receipt.json --base .
python3 scripts/check_inventory.py
```

Keep the committed dependency lock when replaying the evidence. If the toolchain
is unavailable, `bash scripts/bootstrap_lean.sh /absolute/install/path` installs
the prescribed release; then run
`export PATH="/absolute/install/path/bin:$PATH"`.
The bootstrap requires `curl`, `tar` and `sha256sum` on PATH. An existing pinned
elan installation is also suitable. Its narrow launcher workaround is documented in
`provenance.md`; ordinary Lean installations need no additional workaround.

All exported declarations use exactly the standard axioms `propext`,
`Classical.choice` and `Quot.sound`. The checked scopes exclude the full
stochastic process, causal identification, reliable empirical measurement and
source-level verification of the Python implementation. Python remains
`TESTED_ONLY`.

The receipt records source snapshot
`8043f2fdf5918545336632909140f48725d3fdf9` in the original standalone proof
repository. Its later receipt-bearing commit is
`6245517a1723a1e8a1ce71108fd33dab43dee612`. These are historical proof-repository
commits, not RevBench commits. To inspect or replay that exact original history,
clone the adjacent bundle into a separate directory:

```sh
git clone ../formal-proof-project.bundle /path/to/new/formal-history
```

Follow that checkout's README for its standalone replay script. The current
nested folder builds directly with `lake`; it intentionally does not include
the standalone repository-cloning or receipt-assembly scripts as working
commands. Regenerating the article or proofs requires a new semantic review and
receipt before making an updated verification claim.

The delivered positive boundary examples are in
`evidence/IndependentBoundaryTests.lean`. The deliberately false assertion in
`evidence/IndependentRejectedStatement.lean` is expected to fail and is not part
of the proof library. Recorded final receipt inspection is available in
`../reaudit/proof/receipt-review.md`.
