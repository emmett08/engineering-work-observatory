# Independent formulation review

Run: 73f2f5977c5349f3a110c4d50d2acfcc. Role: formulator. Only supplied context and the authorised baseline were read. No manuscript, runtime, contract, database or other worker material was changed. Mathematical statements below await independent checker review; no formal acceptance is claimed.

## Recommendation

Retain the joint cell LP as the general formulation. Its arbitrary attributed-stream sets already allow general overlapping linear observations, so a universal shortest-path or elementary interval formula is not established. The useful simplification is interpretive: **every feasible cell vector can also be attained by a finite schedule in which each person works on at most one task–role stream at a time.** Fractional *cell totals* need not mean simultaneous fractional work. The baseline's sentence saying that exact correspondence “depends on allowing measurable fractional allocations” is too restrictive if it is intended to exclude all-at-once allocations. What matters here is the freedom to place and interrupt work within cells, with no minimum session length, switching cost, synchronisation or within-cell restrictions.

An optional exact preprocessing step merges columns that have identical observation and target coefficients within the same person–cell capacity block. It can remove invisible role/task distinctions without assuming statistical similarity. It buys fewer variables when the records contain such duplicates, and no guaranteed reduction otherwise. It is not necessary for the present small research prototype.

## Exact statement A: serial attainment

Let S=|I||R|, let all cells have lengths d_k>0, and let x be any point of the baseline feasible set P. For each person j and cell k, fix an ordering s=1,...,S of the task–role streams and put

c_0=b_(k-1), c_s=b_(k-1)+sum_(t<=s) x_(t,j,k).

Define a_(s,j)(t)=1 on the half-open interval [c_(s-1),c_s), and zero elsewhere in this cell; set every stream to zero in the remaining interval [c_S,b_k) and at the domain endpoint. Zero-length intervals are empty. Because sum_s x_(s,j,k)<=d_k, all these intervals lie in the cell and are pairwise disjoint. Therefore a_(s,j) is measurable, takes values only in {0,1}, has sum_s a_(s,j)<=1 everywhere, and its integral in the cell is exactly x_(s,j,k).

Applying this independently in every person–cell block preserves every interval observation and every target with cell-constant coefficients. There are finitely many pieces, at most K(S+1) per person including idle pieces before removing empty or coalescing adjacent pieces. Conversely, every such exclusive schedule is an admissible fractional history, so its cell integrals lie in P.

**Conclusion.** The set of feasible cell totals, every vector of cell-linear targets, and in particular the attainable interval [H_min,H_max] are identical under (i) the baseline fractional histories and (ii) measurable exclusive {0,1} histories, even restricted to the finite-piece construction above. Thus the extrema and all intermediate values have exclusive attaining histories. Convexity of the exclusive-history class is not needed: each convexly interpolated feasible cell vector is serialised anew.

**Proof obligations for checker.** Check endpoint conventions, idle time, overlapping observations, preservation of all stream cell integrals, and the distinction between binary instantaneous allocations and integral cell totals. This is an elementary special case of atomless purification, with an explicit construction stronger for this finite partition than an abstract existence theorem. It does not require a computational claim.

**Limits.** This preserves the paper's integral observations, not the original pointwise allocation path, within-cell ordering, co-working synchronisation or a counterfactual sampling record at particular instants. A positive minimum session length or a ban on returning to a task can change the feasible set. For example, one stream with cell lengths 1 and exact consecutive-cell totals (1/2,0,1/2) is feasible under the baseline but cannot consist of a single uninterrupted session of total length 1. The construction does not enforce an additional per-task concurrency restriction; none is in the baseline. It does not change what labour means or validate its measurement.

## Exact statement B: safe column aggregation

Write the cell model as x>=0, Bx<=d for the person–cell capacity rows, and v^-<=Ax<=v^+ for observations. Let W collect every target that must be preserved (one row for labour, more rows if reporting several quantities). Within one fixed capacity block (j,k), partition variables into classes C having exactly equal columns in A and W. Replace their nonnegative coordinates by z_C=sum_(q in C)x_q. Use one common observation/target column per class and replace the capacity row by sum_C z_C<=d_k.

Every original feasible x maps to an aggregated feasible z with unchanged observations and targets. Conversely, for every feasible z choose one representative q_C in each nonempty class, set x_(q_C)=z_C, and set the other coordinates in C to zero. Because all members share their capacity block and observation/target columns, this lift is feasible and preserves every requested target. Therefore the attainable target set, infeasibility verdict, extrema and serial histories are all preserved exactly.

**Conditions are exact and target-specific.** Coefficients that merely look similar cannot be merged. Include all future target rows before merging. Do not pool people because they have similar occupations or mean predictions. Local unavailability or stream-specific upper limits must be included in the full signature or retained when lifting; the baseline has only its common capacity bound. For the unrestricted baseline, lifting a class entirely onto one member is permitted. If substantive attribution requirements prohibit this, those are extra constraints and must enter the model.

**Proof obligations.** Verify class signatures against the actual constructed matrices, then verify a lift and the target values. Merely matching two optimal values on examples is insufficient. Any implementation must preserve original-to-class provenance.

## Why the graph shortcut is not general

The scalar difference-constraint formulation is sound for its stated scalar scope. It is not justified simply by having interval endpoints once arbitrary overlapping stream groups are present.

A minimal obstruction to a blanket total-unimodularity claim is one person, one cell [0,2], three streams A,B,C, and exact observations A+B=1, A+C=1, B+C=1. The observation matrix is

    [1 1 0]
    [1 0 1]
    [0 1 1].

Its determinant is -2, and the observations uniquely force A=B=C=1/2. Their total is 3/2<=2, so the labour history is feasible. It has no integral cell-total point although capacities and observations are integral. The three half-hour blocks followed by a half-hour idle block give an exclusive instantaneous schedule, illustrating why statement A does not imply integral cell totals.

This refutes automatic TU/integral-flow reasoning for the original observation matrix. It does **not** prove that no extended network representation can ever exist. Any claimed network shortcut needs an explicit construction, a projection/lifting proof and a scope condition. For an actual two-stream shared-capacity model, even replacing each stream by independent shortest-path bounds loses their coupling; the baseline already supplies an exact counterexample to adding separate bounds.

## Comparison with published alternatives

1. **Atomless purification.** Khan, Rath and Sun, *The Dvoretzky–Wald–Wolfowitz theorem and purification in atomless finite-action games*, International Journal of Game Theory 34 (2006), 91–104, DOI [10.1007/s00182-005-0004-3](https://doi.org/10.1007/s00182-005-0004-3). Their author-posted August 2005 manuscript, Section 2, states the finite-action theorem preserving integrals under finitely many atomless measures. This is a serious structural alternative to taking fractional allocations as substantively necessary. Apply it per person, with an idle action and cell-restricted Lebesgue measures, to preserve each stream's cell total. The elementary serial proof above is shorter for the paper's step-function observations, supplies finite pieces, and needs no general vector-measure machinery. **Adopt the corollary and an optional literature sentence; reject replacing the LP by a general purification formalism.** Primary manuscript inspected: [author-uploaded text](https://www.researchgate.net/publication/24058874_The_Dvoretzky-Wald-Wolfowitz_theorem_and_purification_in_atomless_finite-action_games). Original 1950 paper metadata was found, but its full PDF access returned a CAPTCHA, so its exact theorem was not independently inspected.

2. **Scheduling theory.** McNaughton, *Scheduling with Deadlines and Loss Functions*, Management Science 6 (1959), 1–12, DOI [10.1287/mnsc.6.1.1](https://doi.org/10.1287/mnsc.6.1.1). The primary scan's Section 2, p.4, explicitly discusses splitting a task into separated intervals; its Theorem 2.2 removes splitting for a particular single-processor completion-loss optimisation. Those hypotheses do not cover fixed overlapping labour observations. **Reject a direct no-interruption simplification:** the (1/2,0,1/2) example above disproves it for this model. Keep a preemptive serial construction; do not replace identification targets by completion-time objectives. [Primary scan inspected](https://www.columbia.edu/~cs2035/courses/ieor6400.F07/mcn1.pdf).

3. **Exact aggregation.** Miller, *Sufficient Conditions for Exact Aggregation in Linear Programming Models*, Agricultural Economics Research 18(2) (1966), 52–57, gives aggregation conditions in terms of common matrices and compatible optimal activities; the same archival volume includes Lee's pp.58–61 discussion stressing their dependence on the objective and difficulties of defining groups before solving. This compares seriously with pooling similar people or intervals: equal mean prediction or occupation supplies none of these optimisation conditions. **Reject unproved representative-person pooling; adopt only explicit equal-column aggregation if useful.** Statement B proves the required stronger feasible-target-set correspondence directly, and does not rely on a “homogeneous solution” assumption. [Primary archival reprint inspected, pp.52–58](https://upload.wikimedia.org/wikipedia/commons/3/37/Exact_aggregation_in_linear_programming_models_%28microform%29._-_%28IA_exactaggregation300unse%29.pdf). The archive's journal pagination is more precise than index entries that list 1–6.

4. **Integral polyhedra/network arguments.** Hoffman–Kruskal's published integrality framework is a possible route only after proving the relevant matrix structure. The concrete determinant above is a paper proof against the required blanket TU premise. The 1956 article was located in a primary-paper reprint volume, but its scanned theorem text was not fully inspected in this turn; treat a detailed theorem citation as an outstanding source check, not a verified dependency. [Located reprint volume](https://sites.math.washington.edu/~thomas/teaching/m514_web/50YearsIP.pdf).

## Measurement uncertainty, sampling and ratios

Neither serialisation nor equal-column aggregation tightens the bounds; each preserves them exactly. Both remain valid for the baseline's finite lower/upper observation bounds. Widening justified bounds can only enlarge P, weakly decreasing H_min and increasing H_max; serialisation cannot manufacture better measurement evidence.

A realised sample that records exact person-interval integrals can be included as integral constraints, but the repeated-sampling law and unbiasedness of the difference estimator are separate. Replacing a path by an observationally equivalent serial schedule does not preserve its sampled values at arbitrary unsplit instants. To apply the constructive statement to a finite set of interval sampling units, include all their endpoints before forming cells. A probability estimate is still not an exact interval total. The existing fixed-prediction unbiasedness theorem and residual-variance comparison require no change from these structural observations.

The productivity range for fixed nonnegative Q and H_min>0 is unchanged. Joint output–labour constraints, zero denominators and sampling confidence coverage are not solved by either proposal. If adding targets for jointly uncertain output, include their coupling explicitly; equality of labour ranges alone cannot justify a Cartesian product of feasible Q and H values.

## Finite checks and open work

`formulator-check.py` uses exact fractions, verifies determinant -2 and the feasible three-stream point, and serialises all 165 quarter-hour grid points in a three-stream simplex of length 2. `formulator-check-results.json` records the results. These checks guard the example and construction coding; the general proofs are the arguments above. No runtime benchmark, production bound, empirical measurement validation, Lean result or independent acceptance is claimed.

Highest-value checker work: review statement A and its caveats, decide whether the baseline fractional-allocation sentence should be narrowed, test statement B's lifting assumptions, and challenge any inference that exclusive instantaneous work implies integer person-hour totals. Exact finite checks have no unresolved numeric gap; the independent proof-review obligation remains open.
