# Formulator peer review, turn 2

Consumed actual Rust-bus messages:

- Estimator: `f115bd6606feb959e56d1a1470fe1e975102ec6d8fddbbf93ee91c495df6b2e4`.
- Challenger: `b4102ad9b8ee911093bf1d57e49ce3165939efa6810b39e6d657330cfd6499ca`.

Read the updated context only; the complete supplied payloads sufficed, so no additional peer file was opened. No coordinator/checker private material was read. No manuscript or runtime state was changed.

The peer counterexamples reinforce the initial recommendation: retain the joint cell LP, scalar graph special case and fixed-prediction estimator; add the exclusive-schedule clarification. The challenger independently supplies the same elementary serial construction and determinant obstruction. There is no substantive mathematical disagreement to resolve.

## Full-attention hypothesis boundary

The challenger states the right quantifier: **every feasible cell vector**, with every finite family of cell-linear targets preserved simultaneously. Each person has at most S+1 nonempty constant pieces in a cell, including idle time, hence at most K(S+1) pieces and K(S+1)-1 internal switching times over the domain. Choose half-open consecutive blocks and consistent endpoint values; point values never affect the integrals. This bound is for the constructed representative history, not a claim about the worker's actual history or a prespecified session budget.

The challenger's example A=(1/2,0,1/2), B=(0,1,0) over three unit cells is feasible but forces A to return after the middle cell. It defeats any inference from exclusive attention to one uninterrupted task session. A minimum positive session length, prescribed maximum number of sessions, switching cost, precedence or resumption delay needs separate feasibility analysis. Arbitrary target weights varying within a cell are also outside the stated cell-linear result unless the partition is refined to make them constant. The construction preserves integral records; it need not preserve observations at arbitrary instants. These boundaries agree with my initial statement and should accompany the correction to the baseline sentence.

Identical-column aggregation remains exact under the baseline conditions. I agree with the challenger's editorial caution: keep it optional preprocessing, since there is no worst-case reduction and adding quotient notation to the main paper may cost more than it explains. Do not pool distinct people or retain an old grouping after new targets distinguish its members.

## Sampling regions can use the same LP

The estimator's proposal has a short direct proof. Suppose the true fixed cell vector x* belongs to the deterministic structural set P0 and the random observation region C_s satisfies Pr(Ax* in C_s)>=1-alpha. On that event, x* belongs to P_s={x in P0:Ax in C_s}; therefore h(x*) belongs to h(P_s). Thus the projected labour region has at least the same coverage. Polyhedral C_s preserves the LP form. If P0 is compact, nonempty P_s is compact when C_s is closed, so scalar extrema are attained.

A useful explicit consequence is Pr(P_s is empty)<=alpha under these hypotheses. An empty sample-based set cannot be described as a logical contradiction of exact observations: it can occur on the statistical failure event. If structural or measurement restrictions in P0 are false, the coverage implication has no guarantee. Simultaneous component intervals with failure probabilities summing to alpha are valid by the union bound without independence. None of this turns an ordinary point estimate into an exact interval total.

The distinction between intervals bounding the realised correction estimator and intervals covering true labour is essential. Bounded errors may enlarge the former by direct arithmetic; coverage of the latter still needs its sampling argument.

## Precision and ratios

I independently recalculated the challenger's unequal-probability example using exact fractions. With probabilities (9/10,1/10), HT values (10/9,10) have mean 2 and variance 64/9; difference values (1,11) have mean 2 and variance 9. Prediction squared error decreases, yet design variance increases. Thus retain the design-specific variance criterion, including the baseline's valid SRS residual-variance comparison. An oracle optimal coefficient depending on unknown labour is not an implementable free improvement.

The coupled ratio examples are correct. Q=H in [1,2] yields only 1, not the marginal outer interval [1/2,2]. Q=H in [0,1], restricted to H>0, still yields only 1. Thus H_min=0 alone does not prove ratio unboundedness. Fixed Q=1 with H in (0,1] does yield [1,infinity), without an attained maximum. The baseline's narrower warning against dividing by zero is appropriate.

No recommendation changes: adopt the serial-attainment clarification; optionally explain justified confidence-region projection; retain the existing general formulation and limits. These are written proof reviews and exact finite arithmetic checks, not mechanical formalisation, empirical validation or a novelty claim.
