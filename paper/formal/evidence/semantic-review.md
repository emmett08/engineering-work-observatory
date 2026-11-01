# Final independent semantic review of the labour-bound Lean extension

**Disposition: PASS for the stated component scope.** No unresolved semantic-fidelity blocker remains for the new integral/cell correspondence, global full-attention reconstruction, supplied weak-dual and relaxed certificates, or finite-vector simultaneous-coverage implication. This is not approval of every mathematical claim in the manuscript, an external solver implementation, or the empirical premises.

Canonical reviewer agent ID: `/root/proof_fidelity`.

Review completed: 2026-11-01 15:39:53 UTC. This report records an actual review by that independently dispatched agent. Before inspecting formal sources or implementer messages, the reviewer read the requested manuscript section and `eq:labour` and wrote `independent-target-review.md`. The reviewer did not edit the proofs or manuscript. Intermediate findings were recorded in `in-progress-semantic-review.md`; this final report supersedes their pending dispositions.

## Exact reviewed objects

All paths in this table are relative to `paper/formal/` in `/workspace/work/engineering-work-observatory`.

| Object | SHA-256 |
|---|---|
| `LabourBounds/CellIntegrals.lean` | `5dc0ebd29f48fdbb024c2f4854b92ed9e6392572d1910c877a07752a4db63da4` |
| `LabourBounds/Certificates.lean` | `ce562fa2137a4170bb014e3cde5353679d38d151e3a812fb19955478ba73df75` |
| `semantic-map.md` | `c2b047284a65727fb2c0fd4ea4973cd2d5602f4ade577f3cf8cace87484fdbd1` |
| `targets/partial-identification.tex` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |
| `AxiomAudit.lean` | `ed29a1fea20d9e0dd0d72f4846dbe3e56a38f053cf7278e62de0816d9f66ae53` |

The target snapshot is identified with repository commit `38c4ce40b789d4c5420c98ff1d568b39c3b1e798`. The final source and map hashes were checked after reading the final declarations. This assessment applies to those exact bytes. Later changes require an updated review or a separate statement explaining their scope.

## Mathematical fidelity

| Requested component | Reviewed declarations and finding |
|---|---|
| Integrability and forward cell constraints | `allocation_integrable_of_capacity`, `history_to_cell_constraints`, the `zero_extension_*` lemmas, `localized_history_intervalIntegrable` and `localized_history_to_cell_constraints` derive the analytic prerequisites and integrate the actual history. Integrability is not assumed in place of the needed result. |
| Interval observations and cell sums | `cell_integrals_recover_interval`, `grouped_observation_from_cells`, `interval_observation_iff_cells`, `grid_cell_subset_domain` and `localized_grouped_observation_on_grid` connect actual interval integrals to finite cell sums. The observation splitting is proved rather than supplied as a premise. |
| Full-attention serial reconstruction | `serialAllocation` is a real indicator of a consecutive half-open block. The local and `stitchedAllocation_*` lemmas prove binaryness, measurability, capacity and exact integrals. `full_attention_reconstruction` gives one history over the whole finite grid, with every prescribed cell total and endpoint-aligned grouped observation. It is not merely a local construction or fractional cellwise constant history. |
| Target preservation | `reconstructed_cellMass`, `reconstructed_observation` and `reconstructed_weighted_target` use the same reconstructed history. Weighted target preservation permits arbitrary real weights, including negative weights. |
| Supplied certificates | `weak_dual_upper`, `weak_dual_lower` and `weak_dual_bounds` establish both directions with explicit nonnegative coordinates and multipliers, arbitrary real matrix entries and real objective weights. Matching-value extremality results require a feasible primal witness and an equal-valued valid certificate. |
| Observation relaxations | `relaxed_bounds` reuses the same multipliers for the changed RHS. `feasible_after_relaxation`, `relaxed_upper_ge` and `relaxed_lower_le` separately establish feasible-set inclusion and outward movement for nonnegative changes. No sharpness after relaxation is asserted. |
| Simultaneous coverage | The signed-row assembly and `true_vector_feasible_on_simultaneous_event` prove that the true fixed finite vector is feasible on simultaneous observation coverage. The event-inclusion and probability theorems then transfer coverage using either supplied valid enclosures or supplied valid certificates. Empty feasible sets explicitly fail the target-coverage event. |

The forward local theorems assume a.e. strong measurability under `volume.restrict D`, nonnegativity inside `D`, and the person's capacity inequality a.e. under that restricted measure. They impose no bound or measurability premise on raw function values outside the study domain. Zero extension and preservation of subinterval integrals close the earlier domain-local/global gap. This covers the manuscript's measurable nonnegative allocations on the bounded study interval; the constructed histories satisfy stronger pointwise properties.

`GridOrdered b K` constrains only the used finite endpoint prefix. The tail of `b : ℕ → ℝ` is unrestricted. Allowing equal endpoints generalizes the strictly increasing paper grid and does not add a premise: feasible masses in a zero-duration cell must vanish. Degenerate observations `p = q`, zero masses, idle cells and empty finite index sets are not excluded. Half-open blocks avoid overlap; the conversion between `Ico` and `Ioc` integrals handles endpoint-null differences. Instantaneous allocations are binary while their integrated cell totals remain arbitrary nonnegative real lengths.

The stream index in the reconstruction denotes one person's task/work-type pairs. The theorem can be applied independently to every person, preserving each person's distinct capacity. Arbitrary finite grouped-integral identities then sum preserved cell totals even when an observation groups people. This modular parameterization does not put a capacity-one constraint on all people together. The direct `localized_grouped_observation_on_grid` helper with one summed capacity is a one-person specialization; the generic grouped identity remains available for cross-person sums after deriving each stream's integrability separately.

The probability theorem has a fixed true finite vector, explicit deterministic feasibility, simultaneous observation coverage, measurable events and a probability measure. Its generic version explicitly requires valid enclosing bounds; the certificate version proves enclosure from its certificate premises. It does not assume target coverage itself. It is a proved finite-vector implication that applies after the integral-to-cell correspondence. There is no single newly declared theorem packaging every person-indexing choice and the continuous-history probability application into one statement. The semantic map's modular component claims are accurate; they must not be promoted to a claim that every conceivable sampling or optimization procedure has been proved correct.

## Actual checking and trusted assumptions

The pinned toolchain reports Lean `4.33.1`, commit `819816b2e0a3bf405af45ae5c7af2491d8f5bee6`. The manifest and installed mathlib checkout agree on commit `0df444a360eaa60ab8c11dca51a86af692955474` (`v4.33.1`). The reviewer inspected the completed project build log, which reports a successful 2,572-job build including both new modules.

The reviewer independently ran the actual pinned command `lake env lean AxiomAudit.lean` in the formal project, with `/workspace/work/lean-paper-20261101/toolchain/bin` on `PATH`. It exited 0. Its output covered all 54 named new theorems: 36 from `CellIntegrals.lean` and 18 from `Certificates.lean`. Every one reported exactly `[propext, Classical.choice, Quot.sound]`. No `sorryAx` or additional axiom appeared. These are Lean's standard logical axioms, not invented domain-specific assumptions. Source inspection found no `sorry`, `admit`, new `axiom`, `unsafe` proof bypass or `native_decide` in either reviewed module. Tactic elaboration is not substituted for inspection of what proposition was proved.

The reviewer did not repeat the full dependency compilation or the separate fresh-environment replay. Those checks belong in the build/replay receipt. The independently executed axiom audit and the semantic review are distinct evidence from that replay.

## Explicit limits and remaining counts

Remaining blockers within the requested, mapped component scope: **0**.

There are **3 explicitly excluded mathematical coverage classes** here:

1. General strong-duality or attaining-multiplier existence.
2. General compactness/extrema existence for the full feasible set, and hence a new formal proof of the complete closed identified-range proposition.
3. A separate minimal-piece or numerical switch-count theorem.

The explicit finite-indicator construction supports the measurable full-attention reconstruction request. Excluding a separate piece-count theorem does not weaken its proved binary/integral/capacity properties. Conditional primal/dual equality proves extremality when witnesses are supplied; it does not remove either existence exclusion above. Marginal-to-simultaneous coverage by a union bound is also not newly claimed in this extension.

No Rust source-semantics refinement, simplex implementation correctness, generic solver success guarantee, exact rational certificate extraction, Python test equivalence or empirical measurement/sampling adequacy is certified by this review. Historical proofs are outside the two reviewed extension modules and receive no new acceptance status from this report. The map states these boundaries honestly. The acceptable manuscript wording is that these mathematical components and their stated correspondence are checked, with the listed dependencies and exclusions retained.
