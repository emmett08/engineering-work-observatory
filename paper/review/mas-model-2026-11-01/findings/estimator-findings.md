# Estimator's independent findings

Authorised material: estimator-context.json and its seven frozen baseline files only. I did not inspect another worker's private work or current repository edits. This is a mathematical recommendation, not empirical validation or formal acceptance.

## Recommendation

Retain the fixed-prediction difference estimator and the distinction between deterministic labour identification and sampling inference. Both are already simple and target-preserving. No published estimator inspected establishes that a replacement would improve precision for every finite labour population allowed here. The useful optional extension is a short confidence-set bridge: construct a design-valid simultaneous confidence region for unknown observation totals, then project that region through the existing capacity LP. This gives sampling coverage rather than an exact-data identification claim. It does not require replacing the LP or adding a model relating code to labour.

The paper can also state more explicitly that bounded measurement error does not require a zero-mean measurement model: it can be carried into conservative confidence bounds. Neither extension is necessary to justify the current main results.

## 1. Published comparison

Denis and Haddou, *Prediction-Powered Estimation: Unbiased Model-Assisted Estimation*, Journal of Official Statistics 42(3), 371–394 (2026), DOI 10.1177/0282423X261451320, Section 2.3 equations (2)–(4), gives exactly the fixed-auxiliary difference estimator used in the baseline, up to dividing totals by population size. Its sampled variance estimate additionally needs positive joint inclusion probabilities for the pairs entering that estimate. Its Section 2.2 distinguishes fitted GREG's asymptotic design properties from exact finite-population unbiasedness; Section 2.4 describes subsampling Rao–Blackwellisation as a different, more computationally involved construction. Therefore generic PPI, cross-fitting, GREG, or fitted calibration is not a drop-in simplification preserving this paper's exact design-unbiasedness claim. Source inspected directly: https://journals.sagepub.com/doi/full/10.1177/0282423X261451320 .

Bardenet and Maillard, *Concentration inequalities for sampling without replacement*, Bernoulli 21(3), 1361–1385 (2015), DOI 10.3150/14-BEJ605, Corollary 2.5, supplies a finite-population Hoeffding–Serfling bound under uniform sampling without replacement. Its finite-population factor vanishes at a census. This provides coverage without normality or independent person-interval outcomes. The bounded fixed residuals are the finite population to which the theorem applies. Source inspected directly, version 2: https://arxiv.org/pdf/1309.4029 .

Fieller's classical ratio-interval work is relevant background but does not itself license a normal approximation for this finite-population sampling design. The direct set mapping below avoids adding such an approximation. Bibliographic primary page: https://academic.oup.com/jrsssb/article/16/2/175/7026708 .

## 2. What precision improvement actually requires

Let T_y = sum_s y_m/pi_m and T_f = sum_s f_m/pi_m, with all y and f fixed under the design. The difference estimator equals T_y + F - T_f, where F=sum_U f_m. Define V_y=Var(T_y), V_f=Var(T_f), C=Cov(T_y,T_f). Then its variance is V_y+V_f-2C. It improves on HT exactly when V_f<2C. This is just the variance identity, with no superpopulation model. Under SRS it is equivalent to the baseline's S_(y-f)^2<S_y^2.

More generally, for any coefficient beta fixed whichever sample is drawn,

    Hhat_beta = beta F + sum_s (y_m-beta f_m)/pi_m
    E_s Hhat_beta = H
    Var_s Hhat_beta = V_y - 2 beta C + beta^2 V_f.

If V_f>0 the oracle minimiser is beta*=C/V_f and the minimum is V_y-C^2/V_f. If V_f=0 then C=0 by Cauchy–Schwarz and the coefficient makes no variance difference. This is a useful comparison with a simpler HT baseline, but not an operational guarantee: C depends on unknown labour. Selecting beta from an independent pilot is allowed if the coefficient is then fixed, conditional on that pilot. It may still be poor for the current population. Selecting beta on the correction sample requires another proof. Thus the optional coefficient does not justify changing the paper merely to claim improvement.

No nonconstant fixed prediction can uniformly improve SRS variance over all admissible equal-duration labour populations. Take y_m=c with 0<=c<=d. HT has zero variance under fixed-size SRS; a nonconstant f has nonconstant residuals and strictly positive difference-estimator variance whenever 1<=n<M. This refutes a universal improvement claim without needing unverified properties of workplace data.

Clipping predictions into physically admissible ranges preserves frozen-prediction unbiasedness but need not reduce design variance. Exact example: M=2, n=1, d=(1,1), y=(0,1), f=(-1,0). The two difference estimates are both 1, so variance is 0. Clipping f to (0,0) gives HT estimates 0 and 2, mean 1 and variance 1. With already bounded predictions, y=(1,1), f=(0,1) gives estimates 3 and 1, variance 1; HT is always 2. These rational calculations were independently enumerated using Python Fraction arithmetic. They are counterexamples, not empirical predictions.

## 3. Optional finite-sample confidence interval under SRS

Assume precisely the baseline SRS design: fixed U of size M>=2, fixed y_m in [0,d_m], fixed finite f_m, and uniform size-n sampling without replacement, 1<=n<=M. Set

    L = min_m (-f_m), U_r = max_m (d_m-f_m), R = U_r-L.

Every true residual r_m=y_m-f_m lies in [L,U_r]. These are justified bounds from capacity and fixed predictions, not bounds estimated from the selected sample. Let

    rho_(M,n) = 1-(n-1)/M                         if n<=M/2,
                (1-n/M)(1+1/n)                  if n>M/2.

For 0<alpha<1, the two-sided Corollary 2.5 bound, applied to r and -r and combined by the union bound, gives

    Pr_s(|Hhat_diff-H| <= eps) >= 1-alpha,
    eps = M R sqrt(rho_(M,n) log(2/alpha)/(2n)).

Derivation: Hhat_diff-H = M times the difference between sample and population residual means. At n=M, eps=0 and the estimator equals H; for M=1 use the same census identity directly. If R=0 the residual mean is known and the estimator is exact. Intersect the confidence interval with [0,sum_m d_m], or stronger deterministic bounds already justified for H. This intersection preserves coverage of the true H. It may be empty on an error event.

This is a conservative alternative to treating an estimated standard error as a justified confidence interval. It works even at n=1, for which the baseline correctly supplies no sample variance estimator. It does not require iid allocations, normal labour, a correct predictor, or stochastic measurement error. It is restricted to the declared uniform SRS design; the same formula is not valid for arbitrary unequal inclusion probabilities. Confidence width can be uninformative, and global residual-range bounds can be wider than the HT labour-range bound. One must not infer smaller confidence intervals merely from using predictions. Published Bernstein variants can use residual variance more efficiently but add formulas and hypotheses; they are not needed for the simplest defensible bridge.

## 4. Bounded measurement error remains separate

Suppose a selected interval has reported labour ytilde_m with a justified guarantee |ytilde_m-y_m|<=e_m. No assumption that the errors average to zero is needed for the following bound. Let Hhat_tilde use ytilde in the difference formula and B_s=sum_(m in s) e_m/pi_m. Pointwise for the realised sample,

    |Hhat_tilde-Hhat_diff| <= B_s.

Therefore any valid sampling bound Pr(|Hhat_diff-H|<=eps)>=1-alpha gives

    Pr(|Hhat_tilde-H|<=eps+B_s)>=1-alpha.

This follows from the triangle inequality on the sampling coverage event. B_s can depend on the realised sample. The measurement guarantees must actually hold for its selected records; if they instead hold only with probability 1-beta, use joint coverage at least 1-alpha-beta. This protects coverage, not unbiasedness of Hhat_tilde. In a census sampling error vanishes but B_s remains.

For asymmetric certified bounds ell_m<=y_m<=u_m, the attainable value of the hypothetical accurate-data difference estimate for a fixed sample is

    [F+sum_s(ell_m-f_m)/pi_m, F+sum_s(u_m-f_m)/pi_m]

when the sampled bounds vary independently; with linked measurement restrictions, optimise jointly. This interval contains a sample estimator, not necessarily true population labour. Confusing these two objects is a serious error. Deterministic physical bounds on unsampled intervals instead give ordinary missing-data labour bounds, potentially tightened by the existing shared-capacity LP.

## 5. General confidence-set projection through the same LP

Let P0 contain the deterministic capacity and other justified restrictions, let A x be the vector of latent observation totals, and suppose a random region C_s satisfies Pr_s(A x_true in C_s)>=1-alpha for every admissible fixed true history x_true. Define

    P_s = {x in P0 : A x in C_s}.

On the coverage event x_true belongs to P_s. Hence the projected interval [min_(P_s) h, max_(P_s) h] covers true labour with probability at least 1-alpha. The same event contains the entire deterministic identified set based on the true observation vector. If C_s is polyhedral, construction, consistency checks, extrema and attaining histories still use the existing LP machinery. Rectangular simultaneous bounds are sufficient; marginal 1-alpha bounds separately on many observations are not simultaneous 1-alpha bounds. A union bound with sum_nu alpha_nu<=alpha works without assuming their independence.

An empty P_s is possible with probability at most alpha under all stated model and coverage assumptions. It is then a statistical inconsistency flag, not deterministic proof that the true records violate capacity. The resulting interval is an outer confidence set with the stated sampling coverage, not a sharp deterministic identification interval conditioned on exact observed totals. A confidence region may be conservative. This distinction preserves the paper's current targets and prevents treating HT estimates as exact LP equalities.

## 6. Productivity ratios

For exactly known Q>=0 and deterministic 0<L<=H<=U, the baseline inversion [Q/U,Q/L] is already simplest and exact whenever every labour value is attainable. The same inversion applied to a valid labour confidence set preserves its coverage for fixed Q and positive H. If the confidence set contains arbitrarily small positive H and Q>0, the resulting productivity set is unbounded above. A finite upper endpoint cannot be manufactured by dividing by zero or clipping a denominator away from zero without justification. If Q=0 and H>0, productivity is 0.

Design-unbiased labour estimation does not imply unbiased productivity estimation, even when estimated labour is always positive. Exact example: M=2, n=1, d=(3,3), y=(1,3), f=0, Q=1. True H=4 and productivity=1/4. HT is 2 or 6; E[Q/Hhat_HT]=(1/2+1/6)/2=1/3. This was checked with rational arithmetic. Jensen's inequality explains the direction whenever the random denominator is strictly positive: reciprocal is strictly convex.

When Q and H are jointly uncertain, report or optimise over their joint feasible/confidence set. Marginal intervals can lose dependence. Example: permitted pairs satisfy Q=H in [1,2]. The ratio is exactly 1, while separately inverted marginal bounds give [1/2,2]. Joint polyhedral constraints with a denominator bounded strictly above zero can use a linear-fractional optimisation transformation, but such joint constraints need evidence; nothing in the current record implies them. For sampling uncertainty in both Q and H, a valid joint region mapped by (Q,H)->Q/H preserves coverage without normality. A delta-method or Fieller approximation needs an independently justified distributional or asymptotic argument.

## Limits and proposed use

The baseline already meets the main measurement and sampling obligations. Keep its LP for overlapping, attributed, shared-capacity observations and keep its fixed-prediction estimator. The confidence-set paragraph is the most useful optional addition because it turns the warning about treating estimates as exact totals into a constructive procedure. Present it as a design-specific extension, not a universal confidence method or a claim that predictions improve precision. No source inspected establishes empirical accuracy, frame coverage, valid labour-error magnitudes, or better residuals for software telemetry.
