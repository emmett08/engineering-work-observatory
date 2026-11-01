# Timeline, formal proofs, causal design and citation completion

This revision adds the requested timeline to Figure 1, a new checked Lean
supplement, and a specified causal-study protocol. It preserves the mathematical
targets and the earlier removal of the Reproducibility section.

## Figure

The marked point `P=(0.5,0.5)` on Figure 1's diagonal is linked to one two-hour
schedule: A, B, A and B each occupy consecutive half-hour intervals. Each task
receives half an hour in each hour and one person-hour overall. The schedule is
constructed, permits arbitrary interruption, and does not identify the actual
unobserved history.

The selected three-panel layout retains the original 152.4 by 80 mm dimensions.
An alternative bottom-panel layout is 112 mm high. The figure package includes
both prototypes, exact block/overlap calculations, updated caption and alternative
text, vector export, mechanical audit and independent blind reading followed by
specification comparison. The complete manuscript placement was inspected.

## New formal coverage

`paper/formal/` is separate from the historical `paper/effort-speed-ai/formal/`
package. Lean 4.33.1 and mathlib v4.33.1 check 54 declarations covering:

- Integrability from domain-local measurability, nonnegativity and almost-everywhere
  person-time capacity, with a checked zero extension outside the study domain.
- Nonnegative cell integrals and integrated capacity constraints.
- Actual interval-integral and adjacent-cell-sum identities.
- Binary histories stitched across a finite ordered grid, preserving cell masses,
  endpoint-aligned grouped observations and arbitrary real weighted targets.
- Bounds from supplied primal and dual vectors, attained optimality when their
  feasible objective values agree, and reuse after observation relaxations.
- The conditional simultaneous-coverage implication, with empty-set failure.

The pinned project build, complete axiom audit and fresh kernel replay passed.
Every exported theorem uses exactly `propext`, `Classical.choice` and `Quot.sound`.
Fresh replay checked local and transitively imported constants from an empty
environment in 320.446 seconds; recorded peak child RSS was 3,099,096 KiB.
The official mathlib cache was unavailable in this environment, so the pinned
dependency sources were built without changing versions.

An independent reviewer checked the target before inspecting the implementation,
identified and resolved domain/localisation and target-preservation gaps, then
reviewed the final source/map hashes and independently repeated the axiom audit.
A second review, tied to the current manuscript hash, confirms that it differs from the
immutable target only by the timeline callout, attribution citations and formal
coverage paragraph. The receipt records that current-source binding separately
from its original target commit. A source-change test confirms that the freshness
guard rejects an altered article.

The receipt also binds dependency records, proof sources, checker scripts,
executable identities, review and checking logs. Static receipt validation checks
those records; it does not replace a Lean execution. `make -C paper formal`
performs the actual build, audit and fresh replay. The dedicated formal workflow
repeats it for the relevant source changes.

General strong-duality existence, the full extrema-existence proof and the
separate numerical block-count theorem remain outside the new formal coverage.
No Rust/Python source refinement or empirical measurement validity is claimed.
The existing numerical solver is unchanged. Its descriptions now explain the
two compatible reconstructions and distinguish cell-total output from optional
schedule/certificate export, which is not implemented by this revision.

## Causal-study design

The invoked design-scientific-investigations skill guided the companion at
`paper/study/ai-labour-causal/`. A concise account is included in the article.
The proposed comparison uses parallel, matched-pair randomisation of teams with
requirements and scores recorded before group selection. Primary quantities are
equal-team differences in labour per initial reference unit and retained output
per initial reference unit. The analysis retains original groups, unsuccessful
work, abandoned tasks and repairs. The clocks are 336 hours to acceptance and
336 hours of follow-up, with labour recorded through 672 hours for every task.

The protocol specifies a separate six-pair measurement/feasibility pilot and a
conditional rule for choosing a principal allocation. It compares 48 and 96 teams
in 24 design--scenario cells, each with 20,000 simulated experiments. Planning
effects, meaningful decision boundaries, quality noninferiority margins, bias
allowances and budgets are explicitly provisional. Neither candidate meets the
joint-success requirement across the required assumptions; no principal design
is selected and no field experiment has started.

The numerical assessment includes component-null boundaries, maximum allowed
measurement bias, failed bias assumptions, missing endpoints and Monte Carlo
uncertainty. Independent review resolved changes of cluster unit/weights, severe
defect interpretation and missing-endpoint interval reporting. The companion
also explains the missingness ceiling imposed by requiring every endpoint:
under the simulated independent pair-loss model, `(1-p)^J` bounds joint success.
Larger allocations alone cannot overcome that particular completeness rule.

The retained protocol has schema 1.2 and status `specified`. The advertised
bundled validator resource was unavailable; the project-local validator reports
its bounded structural/reference/numeric coverage explicitly. Successful record
checks and simulated planning do not establish empirical or causal validity.

## Citation audit and validation

Ten used references are added: Dvoretzky--Wald--Wolfowitz and Khan--Rath--Sun for
the reconstruction's mathematical context; Allman--Matias--Rhodes for the generic
HMM-identifiability qualification; Lean and mathlib; and five sources for the
causal design, interference, noninferiority, simulation planning and pilot-effect
bias. Primary-source inspection and bibliographic metadata are retained in the
citation and study records. Conditional comparison sources not used in the paper
are not added merely to enlarge the bibliography. Cui et al. remains online-first
without invented pagination.

Validation includes all 216 Rust tests, formatting and Clippy; the complete paper
check; exact figure and model calculations; 24-cell planning reproduction and
protocol record checks; and the Lean checks described above. The rebuilt PDF has
16 pages. Every page was rendered, with detailed inspection of the figure,
formal-scope paragraph and causal-study section. Citations and labels resolve;
there are no overfull/underfull boxes. The existing BibTeX missing-pages warning
for the online-first Cui reference remains. Historical proof-package bytes are
unchanged.
