# Semantic concordance map

Prepared by the original formal worker and reviewed at the expressly scoped theorem level by `/root/check_refine`; see the historical `evidence/independent-concordance.md`. The third-run review of the revised manuscript and unchanged proofs is recorded separately in `evidence/reaudit-concordance.md` by `/root/proof_reaudit`. The frozen final textual source is `manuscript/main.tex` (SHA-256 `5a4b472e247a69b8af113f1f7eb2eee81cf5ef28540064f941121843cfd34631`), with `manuscript/references.bib`. The inherited article remains separately marked historical provenance. Every theorem below is in namespace `EffortSpeed`. All 43 exported results use exactly the permitted standard logical axioms `propext`, `Classical.choice`, and `Quot.sound`; the exact output is retained in `evidence/axioms.log`. Their status is `CHECKED_WITH_DECLARED_AXIOMS`, not an empty-axiom claim. The independent report accepts the finite, algebraic and infinite-series subclaims described here; the final receipt binds the exact source, inventory and review hashes. Structural receipt validation does not replace that semantic review.

## Sign-rule correction — prop:binomial

`positiveSigns : (Fin 10 → Bool) → Nat` counts true entries. `rejectionEvent` filters the complete finite function space on counts 0, 1, 9, 10. `sign_space_cardinality` proves 1,024 total strings; `sign_event_cardinality` proves 22 rejecting strings with kernel evaluation (`decide`); `sign_rejection_probability` proves the uniform probability 11/512. This is a finite event cardinality computation, not merely substitution of the number 22. `binomial_correction` separately computes the equivalent sum of four natural binomial coefficients in rational arithmetic.

The measured signs are not modelled. Their fairness, independence and any second-level Binomial(300,p0) interpretation remain empirical/statistical assumptions. No new mechanisation of the 300-test tail approximation is claimed.

## Variance counterexample — ex:variance

The sample space is `Bool × Bool`, with each of its four points assigned probability 1/4 by `mean4`. `sign false=-1`, `sign true=1`; the first and second coordinates are the manuscript X and Z. `score4`, `duration4`, `efficiency4` are S=2+X, Y=10+4X+3Z and U=3X+3Z. `cov4 f g` is the expectation of the product of centred variables; `var4 f=cov4 f f`. Thus variances and covariance are computed from actual finite-space definitions, not hypotheses containing the desired moment values.

`four_point_normalized` checks unit total mass. `covariance_counterexample` computes Var(S)=1, Var(Y)=25, Cov(S,Y)=4, Var(U)=18, their squared-correlation expression 16/25, and Var(U)/Var(Y)>1−16/25. `counterexample_positive_and_decomposed` proves S>0, Y>0 and Y=10+X+U at every point. Rational moments are exact and canonically represent the real finite-space values. The square-root definition of Pearson correlation is not introduced; its square is represented by the standard positive-variance ratio Cov²/(Var S Var Y). Nothing identifies U as a causal component in real data.

## Hidden states and observations — prop:nonidentification

`bernoulliMass p b` is p for true and 1−p for false. `observedLaw π c` is the probability of a finite observed Boolean word under independent hidden states and independent conditional emissions. At each time it explicitly sums over both hidden states z, weighting by `bernoulliMass π z`; conditional emission probability is c for coding and zero otherwise. It multiplies that marginal with the recursively constructed law of the remaining word. Independence is therefore an explicit modelling definition, not an inference from parameter multiplication.

`marginal_emission` proves the one-step hidden-state marginal equals Bernoulli(πc). `observed_law_factorization` proves the entire finite-word law equals the Bernoulli-product law for every π,c and every `List Bool`; `observed_law_equal` instantiates (1/2,1/5) and (1/4,2/5) and proves equality for EVERY finite word. It is neither fixed-length enumeration nor only πc equality.

`historyExpectation π n f` separately evaluates the expectation of f over all 2^n hidden histories by recursive binary branching with weights 1−π and π. `history_mass_one` proves normalization; `history_expectation_nonnegative` proves positivity for 0≤π≤1 and nonnegative f. `activeCount` counts true/coding minutes. `expected_count_offset` is the general offset lemma; `latent_effort_mean` derives nπ; `latent_effort_unequal` proves strict inequality of the two active expectations for every natural n>0. `nonidentification_parameters_valid` proves the chosen probabilities lie strictly in (0,1) and the hidden probabilities differ.

The result checks all finite cylinder laws. It does not construct an infinite product probability space, prove measure uniqueness from cylinder equality, or make a generic identifiability assertion for persistent HMMs. Conversion from active minutes to hours divides by 60 and is not a separate exported theorem. The analytical interpretation of these latent states as human labour remains unverified.

## Work sampling — prop:sampling:finite

`sampling_unbiased_finite` is a theorem for arbitrary finite Ω and interval set M, real sample weights w, indicator values I, contributions a and inclusion probabilities π. Its hypothesis is `sum_ω wω * I_mω = π_m` for each interval, with π_m≠0. Its conclusion is the actual finite weighted expectation of the inverse-probability sum, equal to `sum_m a_m`.

For the manuscript instantiate I with the sampling inclusion indicator and a_m with d_m times the adjudicated interval-average allocation. A finite probability law supplies nonnegative normalized w, and known positive inclusion probabilities satisfy the weaker nonzero hypothesis. The theorem is stronger algebraically because normalization, positivity and Boolean values are not needed for the identity once the indicator means are supplied. No independence between indicators is assumed. Infinite or general measurable sample spaces, design variance, nonresponse and measurement correctness are not mechanised.

### Cluster-assignment estimator — eq:cluster-ht

The same `sampling_unbiased_finite` theorem also supplies the exact finite assignment calculation. Let Ω enumerate possible cluster assignments with weights w, let M be the enrolled clusters, set I_mω to the indicator that cluster m receives policy a, and set a_m to that cluster's fixed potential endpoint total divided by the fixed total task count. The hypothesis E[I_m]=π_m then gives unbiasedness of the assignment-weighted endpoint mean. No new Lean declaration is needed because this is a direct instantiation of the general finite theorem.

The causal consistency and no-interference premises connecting fixed potential endpoints to recorded outcomes are not algebraic premises established by Lean. The implementation of a concrete estimator and its data correspondence are also not checked. Dividing unbiased output and labour estimators does not prove an unbiased productivity ratio; that caveat remains explicit in `rem:ratios`.

## Capacity and comparison inequalities

`capacity_lower_bound` maps L,C,H,G,m,k to the manuscript symbols. Positive m,k and already established C≤L, H≤mL, G≤kL imply max(C,max(H/m,G/k))≤L. This is only the algebraic conclusion of prop:capacity: it does not prove the resource integrals, define feasible DAG schedules, or prove unlimited-capacity attainability.

`ratio_improves_iff` proves `(h1/p1<h0/p0) ↔ (h1/h0<p1/p0)` for h0,p0,p1>0. It allows arbitrary h1, so it includes the article's positive h1 case. Replace h with intrinsic attempt time t to obtain the time claim; unequal waits are not covered. `productivity_improves_iff` proves the stated productivity comparison iff h1/h0<(r1p1)/(r0p0), with S,r0,p0,h0,h1 positive; p1 and r1 need no extra algebraic positivity and hence may include zero. These are exact iff statements, not one-way sufficient conditions. `retry_reversal` checks the manuscript's rational cost reversal. Stochastic interpretations use the separately stated retry law.

## Infinite retry series — prop:retry:series and variance-series

The coordinator contributed `Geometric.lean`, delivered with its successful check log and hash before integration. Natural n is the number of failures; one-based attempt count N is n+1. The probability weights are `(1-p)^n*p` with 0<p≤1. `HasSum` means convergence of the infinite series and equality to the supplied sum, not equality of a finite truncation or an unqualified totalised `tsum`.

`retry_mass` proves total weight one. `retry_first_moment` proves the N-weighted sum equals 1/p. `retry_labour_moment` proves the hN-weighted sum equals h/p; `retry_elapsed_moment` proves the (w+tN)-weighted sum equals w+t/p. The latter encodes a one-off wait. `retry_second_moment` gives E[N²]'s weighted series (2−p)/p²; `retry_central_second_moment` centres at 1/p and gives (1−p)/p²; `retry_labour_variance` centres hN at h/p and yields h²(1−p)/p².

These are actual convergent, weighted, infinite first and second moments. Construction of an IID attempt process and translation of its induced geometric law into measure-theoretic expectation remain analytical. The strong law/renewal reward limit for IID task-cycle tuples is not proved, nor is a ratio of taskwise random rates substituted for it.

## Capped attempts — prop:capped

`min_eq_tail` proves pointwise, for arbitrary n,K in Nat, that min(n,K) equals the real finite sum of indicators j<n for j<K. `finite_tail_sum` multiplies by arbitrary real finite-space weights and sums, proving the finite weighted expectation identity. `capped_expectation_of_tails` identifies that expectation with `cappedTail p K` when the survival tails are explicitly supplied as `(1-p)^j`. This is a real finite expectation theorem, but the IID-to-geometric-survival argument is a premise rather than a constructed process proof.

`cappedTail` is the polynomial sum, defined for EVERY real p and natural K. `capped_tail_identity` proves p*m_K=1−(1−p)^K without division; `capped_tail_closed_form` derives the quotient when p≠0. `capped_zero_probability` proves m_K=K and q_K=0 at p=0. `capped_zero_cap` proves both zero at K=0. `capped_one_probability` proves both one at p=1,K>0. No use is made of Lean's totalised division at zero to represent the continuous extension. Continuity itself is not separately proved.

The cap intervention, success probability derived from IID failures, and reward S*r_K*q_K are analytical model statements. In particular, `r_K=E[R|N≤K]` need not equal uncapped r. At zero acceptance probability conditional retention is undefined; reward is defined directly as zero. A zero-cap output/effort ratio is undefined in the manuscript even though Lean field division is total.

## Detection — prop:detection

`detectionJoint d s defective passed` defines the four-cell table: correct/pass=1−d, correct/reject=0, defective/pass=d(1−s), defective/reject=ds. `detection_table` checks normalization, pass marginal 1−ds, and the correct cells. `detection_joint_nonnegative` proves nonnegativity for d,s in [0,1]. `detection_conditional_correct` identifies the finite-table conditional ratio with (1−d)/(1−ds); `detection_pass_positive` proves its denominator positive for 0≤d<1 and s≤1.

`detection_correct_rate` proves S*ρ*p/t=S*(1−d)/t (a field identity that is valid even for totalised zero t; the article only interprets rates for t>0). `detection_tradeoff` gives BOTH strict inequalities when S,t,d>0, d<1 and s0<s1≤1: recorded rate is lower at s1, and conditional correctness is higher at s1. The theorem allows s0 below zero algebraically; the article uses its physical subdomain 0≤s0. `zero_defect_no_detection_effect` proves the d=0 boundary. Fixed duration, no queue, no repair and IID repeated attempts are required for the article's long-run interpretation; the formal result does not prove the stopping composition or renewal step.

## Deferred scope and implementation boundary

The inventory explicitly retains deferred tower-property, full scheduling, stochastic law construction, renewal convergence, Jensen, restricted-time integral, regression/retransformation, joint-distribution and complexity claims. Accounting/causal-target formulas are definitions. Measurement, identification assumptions, published study estimates, transfer to a company and policy decisions have an empirical boundary. Python/plots/simulations/QR remain `TESTED_ONLY`; none of the three accepted code-refinement relations has been established.

The 54-entry inventory is bound to the frozen final article through explicit labels or paragraph locators. Superseded complexity, runtime and ATT-definition entries point to the retained historical article and are marked NOT_REQUIRED. Additional final-article qualifications about repair horizons, rejection-rate identification and mean-zero signs remain explicitly deferred. All 43 exported Lean declarations occur in the inventory; no unmatched theorem is silently promoted to full-model coverage. A changed article claim, proof source, dependency lock or implementation invalidates the corresponding hash-bound evidence until rechecked.

## Third-run prose repairs

Training on imputed labels is described as a training target rather than a calibration guarantee. Censoring inference now requires observation support through the endpoint in relevant at-risk strata. The study specifies sustained policy assignment and the treatment of reactive quality assessment. These are empirical or methodological qualifications in the existing inventory rows, not new Lean theorems. No exported declaration, numerical input or formalised mathematical premise changed. The fixed-outcome sampling identity does not itself justify extrapolation when audit inclusion changes the outcome.
