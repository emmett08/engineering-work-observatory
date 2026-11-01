# Lean supplement for attainable labour bounds

This new supplement is separate from the preserved historical proofs under
`paper/effort-speed-ai/formal`. It checks 54 new declarations with Lean 4.33.1
and mathlib v4.33.1. The source target is the immutable
`targets/partial-identification.tex` from commit
`38c4ce40b789d4c5420c98ff1d568b39c3b1e798`; current article citations and the
timeline panel do not change those mathematical statements.

## Checked component scope

- Derive interval integrability from measurable nonnegative allocations and
  almost-everywhere person-time capacity on the study domain.
- Derive nonnegative cell masses and integrated person-time capacity.
- Prove that actual interval integrals equal sums of adjacent cell integrals,
  including grouped observations and arbitrary real target weights.
- Construct measurable zero-or-one allocations, stitch them across the finite
  calendar grid, and preserve every required cell integral and aligned grouped
  interval observation. This allows unrestricted interruption.
- Prove upper and lower bounds from supplied feasible dual multipliers, prove
  attained optimality when feasible primal and dual values agree, and reuse
  certificates after right-hand-side changes.
- Prove the simultaneous-observation-coverage implication, including the signed
  observation constraints and empty-feasible-set failure. A specialised version
  obtains the enclosing target bounds from checked certificate premises.

`semantic-map.md` describes the exact symbol correspondence, generalisations and
limits. `evidence/semantic-review.md` records an independently dispatched review
of the final statement types and source hashes. The reviewer independently
repeated the complete axiom audit.

The actual exported axioms are exactly `propext`, `Classical.choice` and
`Quot.sound`, so the receipt uses `CHECKED_WITH_DECLARED_AXIOMS`. No project proof
uses an added axiom, proof hole, native decision shortcut or unsafe declaration.
The receipt distinguishes checked targets from deferred and empirical items.

## Deliberate boundaries

The new formal coverage does not include the general strong-duality existence
theorem, the full compactness/extrema-existence argument, or a separate numerical
bound on the number of schedule pieces. The explicit reconstruction is a finite
sum of interval indicators, but the separate `K*(n+1)` counting statement has no
new checked status. The original source supplies the mathematical arguments and
published references for claims outside this supplement.

The history construction is proved per person. It can be applied independently
to each person; grouped integral identities also permit cross-person sums once
each person's integrability is derived. The probability theorems concern the
fixed finite cell vector after that correspondence. No single additional theorem
packages every indexing choice and the whole continuous-history probability
application into one declaration.

The source statements are conditional on their explicit model, measurement and
coverage hypotheses. Lean establishes none of those empirical premises. The
Rust solver and Python calculations remain `TESTED_ONLY`: no source-language
semantics refinement, code generation or exhaustive implementation equivalence
is claimed. The module proves supplied certificate checks, not that the current
Rust command exports certificates or finds a particular one.

## Reproduce

Install the pinned toolchain into task-local storage when necessary, using the
repository's unchanged bootstrap:

```sh
bash ../effort-speed-ai/formal/scripts/bootstrap_lean.sh /absolute/toolchain/path
export PATH="/absolute/toolchain/path/bin:$PATH"
```

The three dependency records are committed. Do not run `lake update` during an
evidential replay. Obtain the pinned mathlib cache when network access permits,
or let Lake compile the pinned sources. The first source build can take much
longer than checking the new modules. In the recorded execution the official
Azure cache was denied by the proxy, so the required modules were built from
source without changing the pin.

From this directory:

```sh
make verify
```

This performs the actual pinned build, prints all 54 transitive axiom lists,
replays all imported and local constants into an empty Lean environment through
`leanchecker --fresh`, and validates the committed receipt. New replay logs are
saved below ignored `.lake/verification/`; the original evidence stays immutable.
The command does not manufacture a new semantic-review approval or rewrite the
committed receipt.

For a fast static check without a Lean installation:

```sh
make check-receipt
```

Static receipt validation checks structure, bound file hashes, pins, allowed
axioms and recorded evidence. It does not execute Lean or replace a fresh build.
The receipt binds the exact source, reviewed map, dependency records and original
checking logs. A change to any bound input invalidates that receipt and requires
the appropriate renewed proof/source review and checking before a new claim.

`evidence/toolchain-identity.json` also records the actual executed native Lean,
Lake and leanchecker binary hashes, launcher hashes, versions and host platform.
Those binary fingerprints describe this execution; another supported platform
has different release binaries. The portable static check verifies the retained
identity record's hash and all three local checker-script hashes against the
receipt, without requiring a Lean installation on that host.

`evidence/current-manuscript.json` separately binds the current canonical article
to the immutable target and an independent review of their differences. The
static check reads `paper/article/partial-identification.tex` through its fixed
repository path and rejects a changed article hash. Editorial changes require
renewed current-manuscript concordance and its hash binding; proof-source changes
require the appropriate renewed semantic review, build, audit and replay. The
current article's bytes are never presented as identical to the frozen target.
