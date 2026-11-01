# Independent checker: initial evaluation

Run: 73f2f5977c5349f3a110c4d50d2acfcc. Read only checker-context.json and the seven baseline sources. Applied mathematical-thinking to delegated formal audit. No other worker files read; repository not edited. No machine-proof certification claimed.

## Main result

The existing finite-cell reduction is correct and already provides a simple exact representation for its declared model. I found no error in the stated reduction, scalar shortest-path result, fixed-prediction unbiasedness proof, or variance formulas. Replacing the general LP by a universal scalar graph or a fixed weighted average would lose required operations. Useful improvements are exact certificates, compression of indistinguishable streams, and a clear confidence-set bridge; these supplement rather than supersede the current LP.

## What is established, exactly

For finite stream sets, bounded calendar domain, measurable nonnegative fractional allocations, per-person capacity at most one almost everywhere, finitely many interval-sum bounds, and targets constant on cells, integration and cellwise-constant reconstruction give the same attainable observation and target vectors. Thus the whole finite feasible set is relevant, not only its extreme points. A nonempty feasible polytope is bounded because every coordinate is at most its cell length; each scalar linear target has a compact interval as its exact attainable image. The proof covers arbitrary real target weights. It does not establish physical measurability or completeness of the roster, validity of bounds, nonresponse assumptions, or output quality.

The endpoint partition must include every observation endpoint and every discontinuity of every target weight. The proof is an exact quotient under the stated model: histories with the same cell integrals cannot be distinguished by those observations and targets. It does not preserve intraceIl timing, switching counts, minimum work sessions, or nonconstant time weights. Those omissions are explicit in the baseline and should remain explicit.

## Simplification with no additional empirical assumptions

For each person and cell, merge streams having identical membership coefficients in every active observation and identical coefficients in every requested target. Replace the original stream variables by their sum in each equivalence class. Capacity depends only on this sum; observations and targets also depend only on it. Conversely, assign a class total to any one representative stream and assign zero to the others. This constructs an original feasible point with the same observations and target values. Hence the compressed LP is exact. Classes may differ between cells because no cross-cell continuity constraints exist. A task label or role label required as a separate output prevents merging across that distinction. This can reduce variables substantially but has no guaranteed reduction in the worst case.

## Exact certificate layer

Write all inequalities, including nonnegativity, as Bx <= b. If P is nonempty, an upper-bound certificate for w^T x consists of lambda >= 0 with B^T lambda = w; then w^T x <= b^T lambda for all feasible x. If x* is feasible and w^T x* = b^T lambda, both optimality and an attaining history are established. Lower bounds use -w. With rational coefficients the certificate equations and inequalities can be checked exactly. An infeasibility certificate is lambda >= 0, B^T lambda = 0 and b^T lambda < 0. Published LP strong duality/Farkas theorems establish existence under their usual finite-dimensional conditions; the paper's bounded feasible polytope satisfies the needed finite optimum condition. These certificates improve auditability, not the identification strength or validity of measurements.

Do not state that a solver success flag or small floating residual is an exact certificate. The existence theorem is not itself a benchmark for feasible runtime or small certificate bit length.

## Sampling-to-identification bridge

Suppose random observation bounds [L_nu,U_nu] satisfy joint coverage
P(all true interval totals lie in their respective bounds) >= 1-alpha
under a stated design/measurement process. On that event the true allocation gives a feasible LP point, so the projected LP interval contains the true target. Therefore the LP interval has coverage at least 1-alpha. This requires simultaneous coverage, not just marginal coverage for each observation. Bonferroni can obtain simultaneous coverage from marginal bounds with error probabilities summing to alpha. It supplies a valid confidence envelope, not an exact identified range conditional on realised random bounds. A Horvitz--Thompson point estimate alone does not satisfy these premises.

There is also a deterministic bridge: observed sampled unit values can be exact constraints, while unsampled units retain capacity bounds. This yields a potentially wide sharp range without extrapolation. Inclusion probabilities do not tighten that realised feasible set by themselves; they enable sampling-based inference.

## Ratio limits

The fixed positive-denominator result is correct. If exact Q>0, H_min=0 and H_max>0, restricting the ratio to feasible histories with H>0 gives [Q/H_max,infinity), with every finite value attained but no attained infinite endpoint. If Q=0, every defined ratio is zero; if H_max=0, no productivity ratio is defined. A blanket statement that a zero lower bound prevents any ratio analysis is too strong, although the baseline only says it cannot be used as a divisor.

Joint output-labour constraints matter. For example the feasible set Q=H in [0,1] has defined productivity identically one, whereas separate numerator and denominator intervals conceal that fact. A linear-fractional formulation over a joint polytope with H bounded strictly away from zero is the natural exact extension. State denominator positivity before invoking a Charnes--Cooper transformation. No such joint output model is currently supplied by the baseline, so this is an optional mathematical extension, not a justified measurement assumption.

## General graph/integrality obstruction

With three people, one one-hour cell, one stream for each person, and observation upper bounds x1+x2 <= 1, x2+x3 <= 1, x1+x3 <= 1, the target sum has maximum 3/2 at (1/2,1/2,1/2). The pair constraints form the 3-by-3 matrix [[1,1,0],[0,1,1],[1,0,1]] with determinant 2. Thus the general matrix is not totally unimodular, and a universal integral-flow replacement is unsupported. In this model fractional allocations are admissible, so the fractional optimum is valid. The scalar graph remains an exact special case, and the baseline correctly limits it.

## Baseline examples checked independently

Exhaustive rational calculation over the six size-two samples of y=(0,2,4,6) gives means 12 and variances 80/3 (HT), 16/3 for f=(1,3,3,5), and 320/3 for f=(6,4,2,0). This checks those examples only. Fixed predictions need not be accurate or nonnegative for unbiasedness. Accurate measured y and sample-independent f remain essential. In general the relevant precision condition is design variance of weighted residual totals; the S_r^2 criterion is special to SRSWOR.

## Recommendation for candidate evaluation

Retain the general finite LP, its elementary reconstruction proof, and the scalar appendix. Prefer concise improvements that add verifiable primal/dual and Farkas certificates, optional stream compression, and an explicitly simultaneous-coverage bridge. Do not adopt least-squares, smoothing, Bayesian imputation, network-flow, or prediction-power claims unless their new assumptions or changed targets are stated. Awaiting the coordinator's actual candidate statements and Rust-bus context for final independent evaluation.
