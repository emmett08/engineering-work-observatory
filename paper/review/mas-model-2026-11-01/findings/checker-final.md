# Independent final checker evaluation

Run: 73f2f5977c5349f3a110c4d50d2acfcc. Verdict: the coordinator's revision-1 candidate is mathematically sound under its stated finite-cell model. Retain the LP and estimator; adopt the serial-attainment clarification and concise dual-certificate statement. The simultaneous-coverage paragraph is a valid optional bridge. The recommendations below are written proof review, not machine certification or empirical validation.

Consumed actual Rust-bus inbox IDs:

- f5defb817f33f8f7bca33d33ba9ad503bc11f5ca8879836dce9682477a9f5bdb (formulator).
- a7787a3878505f1b4343d867c4dec4875d81b39c603baa8f08bf3146e8d0b821 (estimator).
- 5d77563e15926a0205dbaeacfc9fc032f7f16ba5fd46ca2753ffa16ee28108a7 (challenger).
- 3155d4299e2b7b1530a57d0d84b28ff340ac3950133ee7a0d91e6db6f0ecd737 (formulator peer review).

Also read coordinator-candidate.md and the three detailed findings files explicitly referenced in those incoming messages. After explicit coordinator disclosure, read only current repository paper/article/partial-identification.tex and the abstract in paper/article/main.tex, under /workspace/work/engineering-work-observatory. Read no other private worker outputs or current repository content and made no repository changes. The initial audit was saved before candidate disclosure.

## 1. Serial attainment and full range: valid

For each feasible x, person j and cell k, the nonnegative lengths x_sjk sum to at most d_k. Consecutive disjoint half-open blocks of those lengths fit inside the cell, leaving an idle block. Their indicators have exactly the required stream cell integrals and give exclusive allocation at each time. Repeating over finitely many persons and cells gives measurable binary histories preserving all original constraints and every cell-linear target simultaneously. The converse follows because binary histories are included among the baseline histories. This proves equality of feasible cell sets and target images, not only equality of extrema.

The full scalar interval remains attainable despite the exclusive-history class being nonconvex. Given minimising and maximising cell vectors x_minus,x_plus and theta in [0,1], first form the feasible cell vector (1-theta)x_minus+theta*x_plus. Serialise that vector afresh. Its target is the corresponding intermediate value. Do not take a convex combination of the binary histories and assert that it remains binary.

With S streams per person and K cells, there are at most K(S+1) nonempty blocks including idle blocks, hence at most K(S+1)-1 internal switches. Endpoint values do not affect any integral. For a literal piece-count statement choose half-open blocks consistently and assign the final domain endpoint the final block's value; setting it to zero can create a harmless isolated endpoint exception. This is a presentation qualification, not a failure of the result.

Exact hypotheses: finite stream and cell sets, positive cell lengths, only the baseline per-person pointwise capacity restriction, and observations/targets determined by cell integrals. No minimum sessions, restriction on returning to a stream, synchronisation, precedence, bounded switching budget, or additional pointwise restriction is being preserved. A serialisation can change pointwise labour at sampled instants. Finite interval-sampling observations are preserved when their endpoints enter the partition.

Therefore revise the baseline sentence implying simultaneous fractional attention is necessary. Suggested concise replacement:

> Every feasible cell vector can also be realised by assigning each person wholly to one stream at a time, with finitely many switches. This construction permits work to start, stop and change streams at arbitrary times; minimum session lengths, restrictions on interruption, smoothness requirements and switching costs require separate analysis.

Fractional person-hour cell totals remain possible and necessary in examples. This refinement changes neither target nor identified range.

## 2. Dual and perturbation signs: valid

Keep the coordinator's convention P={x>=0:Ax<=b}; do not silently put nonnegativity inside A and retain the same dual inequality rule. If lambda>=0 and A^T lambda>=w, nonnegativity of x gives

    w^T x <= (A^T lambda)^T x = lambda^T Ax <= lambda^T b.

Likewise mu>=0 and A^T mu>=-w give -mu^T b<=w^T x. For nonempty bounded P, both primal extrema are finite and attained; finite-dimensional LP strong duality supplies attaining dual vectors for maximising w and -w. Exact primal feasibility, dual feasibility and objective equality provide independently checkable arithmetic certificates. Strict feasibility is unnecessary, including when exact observation equalities are represented by opposing inequalities.

For any finite e>=0, P is a subset of P_e={x>=0:Ax<=b+e}. The same dual vectors remain feasible because A and w do not change. Thus

    -mu^T(b+e) <= w^T x <= lambda^T(b+e)  for every x in P_e.

When lambda and mu certify the original endpoints, the certified lower endpoint decreases by mu^T e and upper endpoint increases by lambda^T e. These are global one-sided guarantees. They are not necessarily the perturbed extrema, derivatives, or exact marginal values of an extra measurement. Original feasible points remain feasible, and for the cell model finite relaxed capacity right sides still bound all coordinates.

For measurement-error wording, set e=0 on deterministic capacity rows unless a change in capacity is intentionally part of the perturbation. For a lower observation row -a^T x<=-v_minus, adding e>=0 lowers the admissible lower limit to v_minus-e. For an upper row it raises the upper limit. This makes the signs and the intended meaning explicit.

Arithmetic certification is exact only if the coefficients and returned witnesses are represented exactly and all equations/inequalities are checked. A floating solver success flag does not establish it. For rational data rational optimal certificates exist; the candidate makes no unsupported scaling or time guarantee.

## 3. Simultaneous-coverage bridge: valid

Let x_true satisfy deterministic capacity and accounting assumptions, and let E be the event that every true observation total lies in its random supplied interval. If P(E)>=1-alpha, then on E the cell vector x_true belongs to random P_s, making P_s nonempty. Consequently the extrema over P_s bracket its true target. This proves unconditional repeated-design coverage at least 1-alpha. It does not establish coverage conditional on realised observations, nor empirical correctness of the sampling or measurement process.

Marginal intervals at level 1-alpha do not jointly deliver that coverage. Simultaneous coverage or an error allocation whose sum is at most alpha is required; independence is unnecessary for a union bound. Empty P_s may occur on the coverage failure event and must be reported as empty, never as a zero-width interval. For finite rectangular observation intervals, P_s remains a compact polytope when nonempty. A general nonpolyhedral or nonconvex confidence region requires appropriate optimisation and may have a noninterval exact target image; the enclosing min/max interval still supplies coverage when extrema exist.

This paragraph usefully turns the existing warning about treating estimates as exact totals into a constructive statement while leaving construction of valid simultaneous intervals design-specific. Do not silently apply it to an HT point estimate or call its output the sharp deterministic identified interval conditional on realised exact observations.

## 4. Other messages and counterexamples

Equal-column aggregation within the same person-cell is valid under the baseline: sum is the forward map; placing all mass on a representative is the lift. It preserves every nominated target and all retained constraints. It is optional preprocessing with no worst-case dimension reduction. Additional individual upper bounds or pointwise restrictions require a new lift proof; identical observation columns alone would not suffice.

The determinant-minus-two example is correct: pair equalities over one two-hour person-cell force x=(1/2,1/2,1/2), within capacity. It refutes automatic total unimodularity/integral cell totals of the original matrix, not every possible extended graph representation. Binary temporal scheduling does not make cell totals integral. The interrupted A=(1/2,0,1/2) example correctly refutes a one-session-per-task replacement.

The estimator's no-uniform-SRS-improvement argument is correct: constant admissible y gives HT variance zero, whereas nonconstant fixed f gives positive residual variance for 1<=n<M. The prediction-clipping and positive-denominator plug-in-ratio counterexamples are arithmetically correct. The challenger's unequal-probability example correctly gives HT variance 64/9 and difference-estimator variance 9 despite lower unweighted residual SSE. No universal precision gain follows.

The bounded measurement-error triangle inequality is valid under actually guaranteed error bounds; if those bounds hold only probabilistically, their failure probabilities must also be controlled. Its conclusion is coverage protection, not unbiasedness.

Deferring Charnes-Cooper is appropriate. For fixed Q and H_min>0 the reversed endpoint formula is exact and simpler. Jointly uncertain Q and H require an evidenced joint feasible set; marginal interval arithmetic does not establish sharpness. A zero lower labour bound alone does not imply an unbounded ratio when Q and H are coupled.

## 5. Independent source correspondence

I directly retrieved and inspected Boyd and Vandenberghe's author-hosted book PDF. Section 5.2.3, printed pp.226-227, explicitly states that the refined Slater condition reduces to feasibility for all-affine constraints with an open objective domain and that finite dual optima are attained. Section 5.2.4 expressly applies this to LPs. Representing x>=0 as affine inequalities supplies the open full-space objective domain, then eliminating their multipliers gives the coordinator's A^T lambda>=w convention. Section 5.6.2, printed p.250, gives global perturbation inequality (5.57); Section 5.6.3 adds differentiability for gradient interpretations. The candidate uses the supported global inequality and correctly avoids an unsupported derivative claim.

URL: https://web.stanford.edu/~boyd/cvxbook/bv_cvxbook.pdf . Local inspection copy: checker-boyd.pdf / checker-boyd.txt.

I also independently inspected the text of Khan, Rath and Sun's author-posted manuscript on ResearchGate. Section 2's Theorem [DWW] states that finite atomless measures and a finite measurable probability vector admit measurable binary components preserving all component integrals for all the measures. The paper's model fits by adding an idle action and taking cell-restricted Lebesgue measures, one person at a time, after harmless modification on null sets. This confirms the related purification attribution. The elementary serial proof establishes the required stronger finite-piece construction directly; the abstract theorem alone does not supply the stated finite-switch bound. No novelty claim is warranted.

URL: https://www.researchgate.net/publication/24058874_The_Dvoretzky-Wald-Wolfowitz_theorem_and_purification_in_atomless_finite-action_games . Local extracted inspection text: checker-krs.txt. I did not inspect the original 1951 DWW full proof.

I independently inspected Bardenet and Maillard's arXiv PDF, Corollary 2.5. Its one-sided sample-mean bound has exactly the estimator's rho factor. Applying it to fixed residuals and their negatives with delta=alpha/2 gives the stated two-sided halfwidth M R sqrt(rho log(2/alpha)/(2n)). A known global residual envelope is valid; observed sample extrema are not an automatic substitute. The census case is exact. This formula is correct but longer than needed for the coordinator's concise general projection paragraph and need not enter the manuscript.

URL: https://arxiv.org/pdf/1309.4029 . Local inspection copies: checker-bardenet.pdf / checker-bardenet.txt. I did not independently re-audit the other workers' optional scheduling and aggregation literature; neither is a dependency of the recommended elementary proofs.

## Final recommendation

Adopt the candidate with the minor endpoint and row-specific relaxation clarifications above. Retain the general LP, scalar graph special case and fixed-prediction estimator. Use direct finite proofs for serial attainment and aggregation, published LP duality for exact certificate existence, and explicit simultaneous coverage for the optional statistical bridge. These improve interpretation and auditability without changing the feasible labour totals or adding empirical assumptions. No mathematical discrepancy in the candidate remains unresolved; source and computation limits stated above remain in force.

## 6. Final authorised implementation correspondence

The later formulator peer message agrees with the checked results. Its simultaneous-confidence-set statement is valid, including Pr(P_s empty)<=alpha under the stated joint-coverage and correct-structural-model hypotheses. Its full-attention construction preserves all cell integrals, and its listed counterexamples and scope limits remain correct. All four actual inbox IDs above have now been consumed.

I inspected the explicitly authorised current manuscript partial-identification.tex and main.tex abstract. The implemented cell proof correctly constructs exclusive stream blocks and explicitly interpolates minimising/maximising **cell vectors**, then reconstructs each intermediate vector. It therefore preserves the entire claimed interval. The abstract qualifies arbitrary switching and does not infer uninterrupted tasks or integer cell totals. The dual subsection keeps nonnegativity outside A, uses the correct >= dual conditions, and correctly invokes LP strong duality for nonempty P. The perturbation subsection specifies zero epsilon on unchanged constraints and makes only an outer-bound claim, allowing reoptimisation to tighten it. The confidence paragraph requires simultaneous coverage, counts an empty random feasible set as failure, and explicitly leaves valid confidence-bound construction to the actual observation/sampling process. The ratio subsection retains its fixed-output assumptions and joint-feasibility caveat. No substantive theorem/prose mismatch was found.

At inspection the construction set the final domain endpoint to zero; this has no effect on any integral or capacity statement. The coordinator has stated it will use a matching terminal value or qualify the finite-block count almost everywhere to remove a literal singleton convention. This is an endpoint presentation matter only. No new sources or mathematical dependencies were needed for the implementation check.
