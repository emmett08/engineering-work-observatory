---
title: "Effort, delivery speed and retained output in AI-assisted software engineering"
subtitle: "A corrected methodological extension of the Standard Coder approach"
date: "12 October 2026"
fontsize: 11pt
papersize: a4
geometry: margin=24mm
colorlinks: true
linkcolor: blue
urlcolor: blue
---

## Abstract

Code-change size, human labour and delivery speed are different quantities. AI assistance can change their relationship by automating production, moving work into verification, introducing parallel execution and altering the probability that defects are detected. This article retains Standard Coding Hours as a possible fixed reference index, corrects identifiable problems in the original validation argument, and constructs a task-level model of human effort, elapsed delivery time and retained accepted output. An exact hidden-state counterexample shows why commit observations alone cannot guarantee identification of human work. A dependency graph separates work from elapsed time, while a bounded retry model supplies explicit conditions for effort and speed improvements. A detection model shows how easier acceptance can improve recorded throughput without improving correct output. All numerical examples are synthetic. The contribution is an inspectable measurement and analysis framework, with elementary proofs and executable examples; company-specific calibration, causal effects and empirical superiority remain to be established.

## 1. The question and the correction

Wright and Ziegler [1] estimate coding time from commit events, then predict that inferred time from code-change features. Their Standard Coding Hours (SCH) supply a common reference scale. The extension needed for AI is a separation of measurements and observation processes, followed by an empirical test of their changing relationship.

The audit identified the following issues in [1]. Page numbers refer to the seven-page PDF.

| Location | Finding | Required correction |
|:--|:--|:--|
| p.4, III.B | The stated binomial rejection rule gives 2.1484375%, not 5.52%. | Recalculate the null and justify dependence assumptions. |
| p.5, Figure 6 | The caption reverses timestamp and code-diff provenance. | Correct the prediction and target labels. |
| p.5, IV.A | Correlation does not establish the stated 36% efficiency-variance bound. | Remove the bound or supply a structural variance decomposition. |
| pp.4--5 | Latent-model consistency and short-gap proxies are not independent labour observations. | Validate against externally sampled activity. |
| pp.6--7 | Feature substitutions establish predictor sensitivity. | Reserve causal interpretations for identified interventions. |
| p.5, Figure 6 | Bin counts imply about 1.2 million examples, against about 500,000 reported commits. | Reconcile counting units and provide the split manifest. |

The last discrepancy is unresolved reporting, not evidence that the underlying observations are invalid. Raw study data and fitted models were unavailable here, so the published empirical correlations cannot be reproduced. The following calculations correct arguments rather than reconstructing that experiment.

### 1.1 Exact arithmetic and its inferential limit

For ten independent fair signs, let $K\sim\operatorname{Binomial}(10,1/2)$. Rejecting for $K\in\{0,1,9,10\}$ has probability

$$
p_0=\frac{\binom{10}{0}+\binom{10}{1}+\binom{10}{9}+\binom{10}{10}}{2^{10}}
=\frac{11}{512}=0.021484375. \tag{1}
$$

**Proof.** The four outcomes are disjoint; add their binomial masses. The numerator is $1+10+10+1=22$, and $22/1024=11/512$.

A second binomial calculation for 24 positives among 300 tests gives approximately $5.42\times10^{-8}$ with this null. That is a *conditional arithmetic result*. Dependence between time blocks, developers and probability bins can invalidate that second binomial model. Moreover, a mean-zero correction need not have equally probable positive and negative signs. Neither the original non-rejection nor this recalculation establishes the true calibration of latent coding states.

### 1.2 A correlation cannot isolate efficiency

Let $X,Z$ be independent variables taking $-1,+1$ with equal probability. Define a positive reference score $S=2+X$, duration $Y=10+4X+3Z$, and a putative efficiency component $U=3X+3Z$. Then $Y=10+X+U$,

$$
\operatorname{Corr}(S,Y)^2=\frac{16}{25},\qquad
\frac{\operatorname{Var}(U)}{\operatorname{Var}(Y)}=\frac{18}{25}
>1-\frac{16}{25}. \tag{2}
$$

**Proof.** Independence gives $\operatorname{Var}(S)=1$, $\operatorname{Var}(Y)=25$, $\operatorname{Cov}(S,Y)=4$ and $\operatorname{Var}(U)=18$. Substitution proves the inequalities. The score absorbs part of the designated efficiency component. Its residual therefore cannot bound that component's variance.

This variance ratio is a counterexample to the claimed bound, not an identified causal variance share. An upper bound would need an explicitly justified additive model, the relevant orthogonality conditions, and a specified aggregation level. Project-level averages also cannot directly bound individual variation. Similarly, correlation one can coexist with a calibration error: predicting twice the true quantity preserves correlation.

## 2. Why the AI extension needs separate endpoints

Published AI studies estimate different outcomes under different conditions. Peng et al. [2] report a 55.8% reduction in completion time among completers of a specified programming task. Becker et al. [3] report a 19% increase in time for their experienced open-source developers using early-2025 tools. Cui et al. [4] report 26.08% more completed tasks across three field experiments, with a standard error of 10.3 percentage points. These quantities are neither the same endpoint nor interchangeable estimates of a universal effect.

METR's February 2026 update [5] describes selection problems and difficulty measuring time when developers use concurrent agents. It does not supply a reliable replacement effect size. These findings motivate measuring human attention, elapsed time, task selection and quality separately. They do not determine any parameter in the model below.

Define an **AI policy** as a specified combination of available tools and model versions, permitted autonomy, task eligibility, human review and release rules. Randomised access to that policy differs from actual usage. Usage remains a measured behaviour and may depend on task difficulty. Policy comparisons must identify which components change and which acceptance requirements remain fixed.

“Effort” below means allocated human labour time in person-hours. Cognitive strain, satisfaction and organisational value require additional observations; they are not inferred from a time measure.

## 3. Measurement model

### 3.1 Tasks, labour and resource use

Enrol task $i$ when its scope and acceptance criteria are recorded. The task is a requested change in behaviour or capability, with explicit rules for splits, mergers and later scope changes. A commit or pull request can link to a task but does not define its boundaries.

Let $a_{ijr}(t)$ be the fraction of person $j$'s time allocated to task $i$ in role $r$. Time is measured in hours. Impose

$$
a_{ijr}(t)\geq0,\qquad \sum_{i,r}a_{ijr}(t)\leq1,\qquad
H_i(u)=\sum_{j,r}\int_{s_i}^{s_i+u}a_{ijr}(t)\,dt. \tag{3}
$$

Here $s_i$ is task enrolment and $u$ is a stated follow-up horizon. Roles include specification, implementation, prompting, review, testing, integration and repair. This is a definition of accounted labour, not a claim that attention fractions are observed without error. Pair work contributes two person-hours per elapsed hour. One person's simultaneous open sessions cannot each receive a full labour hour. State whether attended waiting is counted; unattended agent execution belongs in a separate resource measure.

Record agent instance-hours by model/resource class, tokens and billed cost separately. They cannot be added to person-hours without an explicit conversion. Total economic cost can be defined using stated labour prices and compute charges, but it answers a different question.

Let $T_i$ be elapsed hours from enrolment to first acceptance. Deployment is a separate endpoint when the intended outcome is delivery to users. Do not substitute first-commit time for actual task start. For a task permanently abandoned without acceptance, define $T_i=\infty$. Administrative loss of follow-up is censoring, not abandonment.

### 3.2 A fixed reference and retained output

A reference score $S_i^0\geq0$ may be one task, independently assessed functional units, or a frozen SCH model. Specify the reference policy's stopping rule, unsuccessful work, quality bar and finite expectation. If calibrated from actual labour in that declared reference population, its conceptual target is $\mathbb E[H_i^{\mathrm{reference}}\mid x_i]$. A model trained only on imputed coding labels remains an index of those labels. Calling it a human-hour standard does not supply the missing calibration.

Fix the score version, task representation and acceptance bar. A score based on final code differences is retrospective and affected by AI's implementation choices. Prefer intake requirements for fixed-scope causal comparisons. Also test invariance to commit splitting and squashing: a nonlinear score $f$ need not satisfy $f(x_1)+f(x_2)=f(x_1+x_2)$. Score a canonical task-level change once, or use an explicitly additive requirement measure. Retrying or reverting a change must not create duplicate retained output.

For an acceptance horizon $\tau$ and subsequent follow-up $\Delta$, define

$$
Q_i(\tau,\Delta)=S_i^0\,\mathbf1\{T_i\leq\tau\}\,R_i(\Delta),
\qquad 0\leq R_i(\Delta)\leq1. \tag{4}
$$

$R_i$ is the fraction of the accepted requirement that still satisfies a specified assessment at $T_i+\Delta$. Fractional assessment needs a declared rubric. Intentional replacement and defect-driven removal must be distinguished. Line survival and green CI are useful observations, but neither establishes maintained functionality, freedom from undetected defects or business value.

Observe each enrolment cohort for at least $\tau+\Delta$. Two distinct aggregate quantities are

$$
P_H=\frac{\sum_i Q_i(\tau,\Delta)}{\sum_i H_i(\tau+\Delta)},
\qquad V_W=\frac{\sum_{i:\,s_i+T_i\in W}S_i^0R_i(\Delta)}{|W|}. \tag{5}
$$

$P_H$ is retained reference output per person-hour for the specified cohort and horizon. Its denominator includes unsuccessful work and follow-up repairs. $V_W$ is retained output accepted per calendar hour in window $W$, assessed after equal follow-up. Report initial and final work in progress because an output window can contain work begun earlier. These two ratios use different accounting boundaries. Do not divide a portfolio's output by the sum of overlapping task durations and call the result calendar throughput. Undefined individual ratios at zero effort should remain undefined; aggregate positive denominators and counts of autonomous tasks are informative.

### 3.3 Measuring rather than merely assuming human activity

Use sampled task diaries and independently adjudicated person-time observations to anchor event telemetry. Include off-keyboard reasoning, review and coordination. Random sampling with known inclusion probabilities provides an elementary estimator. For interval $m$ of length $d_m$, true adjudicated allocation $a_m$ and sampling probability $\pi_m>0$,

$$
\widehat H=\sum_{m\in\mathrm{sample}}\frac{d_ma_m}{\pi_m},
\qquad \mathbb E[\widehat H]=\sum_m d_ma_m. \tag{6}
$$

**Proof.** Write each sampled term using its inclusion indicator $I_m$. Linearity and $\mathbb E[I_m]=\pi_m$ yield the equality; independent sampling is unnecessary for unbiasedness. Variance estimation does depend on the sampling design. Nonresponse, incorrect labels and uncertain allocations require further correction or sensitivity bounds.

This design can calibrate a small activity model before and after AI adoption. It does not require minute-by-minute neural-HMM training for every developer. Missing usage telemetry must remain missing rather than being encoded as no AI use.

## 4. What commit histories cannot identify

**Proposition 1 (failure of unrestricted global identification).** In the two-state commit model, let the next hidden coding state have probability $\pi$ regardless of the current state, initialise at that distribution, and emit a commit with probability $c$ only while coding. Then distinct pairs $(\pi,c)$ can produce the same complete observed process and different expected coding hours.

**Proof.** The hidden states $Z_t$ are independent Bernoulli$(\pi)$. With independent emissions, observed commits are independent Bernoulli$(\pi c)$. Both $(\pi,c)=(1/2,1/5)$ and $(1/4,2/5)$ give probability $1/10$ at every minute, with identical finite joint distributions. Over $n$ minutes the expected active times are $n/2$ and $n/4$ minutes. No estimator using only that observed law can distinguish them.

This construction is an exceptional independent-state subfamily. It does not claim that every persistent hidden Markov model is unidentified. It establishes that a guarantee requires additional restrictions, and that identifying a statistical state does not by itself identify human labour. AI-generated events add further reasons to inspect the observation mechanism.

**Proposition 2 (internal consistency is insufficient).** For information sets $\mathcal F\subseteq\mathcal G$ and integrable activity $Z$, correctly specified probabilities $p=\mathbb E[Z\mid\mathcal F]$ and $q=\mathbb E[Z\mid\mathcal G]$ obey $\mathbb E[q-p\mid\mathcal F]=0$.

**Proof.** Apply iterated conditional expectation. The converse fails: a misspecified procedure that always returns $p=q=c$ has zero correction whatever the true activity process. Hence a filtering/smoothing consistency diagnostic can reveal discrepancies but cannot supply independent ground truth. Prospective validation also requires fitting parameters only on information available before prediction.

## 5. A tractable model of production, verification and retries

### 5.1 Work and elapsed duration

Represent a specified execution by a directed acyclic graph of work and waiting stages. Each node has a nonnegative duration. The longest path gives the earliest completion time when the graph includes all prerequisites, capacity is unlimited and stages begin immediately when ready. For finite resources, let $C$ be that path duration, $m>0$ the available human capacity and $k>0$ the available homogeneous-agent capacity, for each resource class present. Every feasible schedule satisfies

$$
L\geq\max\{C,H/m,G/k\}, \tag{7}
$$

omitting a resource term when that class is absent.

**Proof.** Each dependency path must execute in order. Integrating capacity limits gives $H\leq mL$ and $G\leq kL$. Under unlimited capacity, the earliest-finish recursion achieves $C$. With finite capacity the maximum is only a lower bound, not a complete scheduling algorithm.

For the numerical examples, a human production stage of duration $c_h$ runs independently alongside an agent stage of duration $g$, followed by human verification $v$. Thus

$$
h=c_h+v,\qquad t=\max(c_h,g)+v. \tag{8}
$$

$h$ is person-hours per attempt and $t$ is elapsed hours per attempt. The maximum is justified only for independent branches. If the human stage needs the agent result, replace the parallel branches by the actual serial dependencies.

### 5.2 Retry accounting

Assume independent, identically distributed attempts, pass probability $p\in(0,1]$, constant costs $h>0$ and $t>0$, and sequential retries until the first pass. Let $N$ count attempts. Let $w\geq0$ be a one-off initial wait and $r\in[0,1]$ the expected retained fraction conditional on final acceptance. Then

$$
\mathbb E[N]=\frac1p,\quad
\mathbb E[H^{A}]=\frac hp,\quad
\mathbb E[L]=w+\frac tp,\quad
\operatorname{Var}(H^{A})=\frac{h^2(1-p)}{p^2}. \tag{9}
$$

**Proof.** $\Pr(N\geq n)=(1-p)^{n-1}$. Summing this series gives the mean; the geometric variance is $(1-p)/p^2$. Multiplication by constant per-attempt costs gives the displayed moments. A wait incurred on *every* attempt belongs inside the factor $1/p$.

Across independent comparable task cycles with finite means, the strong law gives

$$
P_H^{A}=\frac{S^0r}{h/p}=\frac{S^0rp}{h},\qquad
V_{\mathrm{serial}}=\frac{S^0r}{w+t/p}. \tag{10}
$$

These are ratios of total rewards to total resources, expressed as ratios of expectations. They are not expectations of individual ratios. $H^A$ ends at first acceptance, so $P_H^A$ excludes subsequent repair. For lifecycle productivity use $\mathbb E[H^A+B_\Delta]$ in its denominator, with follow-up labour $B_\Delta$ measured rather than silently assumed zero. The serial rate describes one repeated task stream; it is not a capacity formula for a concurrent team. This is a specialised renewal-reward calculation [6], not a new general theorem.

**Corollary (separate improvement conditions).** For the same task and fixed reference, policies 0 and 1 reduce expected labour and intrinsic delivery time respectively exactly when

$$
\frac{h_1}{h_0}<\frac{p_1}{p_0},\qquad
\frac{t_1}{t_0}<\frac{p_1}{p_0}. \tag{11}
$$

If $S^0>0$ and $r_0>0$, retained-output productivity before follow-up repair improves exactly when $h_1/h_0<(r_1p_1)/(r_0p_0)$. These follow by cross-multiplication of positive denominators in (9)--(10). Unequal waiting requires comparing $w_1+t_1/p_1$ with $w_0+t_0/p_0$ directly. Smaller drafting time alone proves none of these inequalities.

For example, $h_0=1,p_0=0.9$ and $h_1=0.5,p_1=0.2$ halve effort per attempt while increasing expected total effort from $1.11$ to $2.5$ person-hours. This refutes a universal claim that halving production effort halves total effort.

### 5.3 Bounded retries and heterogeneous tasks

Allow at most $K\geq1$ attempts, count exhaustion as failure, and set

$$
q_K=1-(1-p)^K,\qquad
m_K=\mathbb E[\min(N,K)]=\frac{1-(1-p)^K}{p}. \tag{12}
$$

**Proof.** Acceptance within the cap is the complement of $K$ failures. The expected attempt count is the finite tail sum $\sum_{j=0}^{K-1}(1-p)^j$. Mean labour is $hm_K$ and mean retained output is $S^0r q_K$, including unsuccessful-task labour. At $p=0$, the continuous extension gives $m_K=K,q_K=0$. With $K=0$, no attempt occurs and an output/effort ratio is undefined.

Attempts often learn from prior failures. In that case the constant-$p$ model is a falsifiable simplification; use observed totals or attempt-specific conditional probabilities. Pooling task difficulty can also understate effort: with fixed $h$ and random task pass probability $P>0$, Jensen's inequality gives $h\mathbb E[1/P]\geq h/\mathbb E[P]$ when finite. This does not automatically hold after also allowing arbitrary dependence between per-attempt effort and $P$.

## 6. A detection effect that can reverse the interpretation

Assume each independent attempt is defective with probability $d\in[0,1)$, a defect is detected with sensitivity $s\in[0,1]$, and there are no false-positive rejections. Let the gate stop at the first pass. Then

$$
p=1-ds,\qquad \rho=\Pr(\text{correct}\mid\text{pass})=\frac{1-d}{1-ds}. \tag{13}
$$

**Proof.** Rejection requires a defect and its detection, with probability $ds$. Every correct attempt passes, so Bayes' rule gives the conditional correctness. Independent retries preserve that composition for the final accepted attempt.

For fixed attempt costs, no queue and no later repairs,

$$
V_{\mathrm{recorded}}=\frac{S^0(1-ds)}{t},\qquad
V_{\mathrm{correct}}=\frac{S^0\rho p}{t}
=\frac{S^0(1-d)}{t}. \tag{14}
$$

The proof is substitution into (10). Reducing detection sensitivity can increase recorded accepted output per hour while lowering the probability that accepted output is correct. Correct-output throughput remains unchanged in this particular independent-attempt model. The cancellation is not a recommendation to weaken tests: escaped defects, repair costs, security losses and task dependence lie outside these simplified rates. At $d=0$ there is no detection effect. General retention $r$ and conditional correctness $\rho$ are different quantities and must not be conflated.

![Under fixed synthetic defect probability 0.4, two-hour attempts and ten reference units per task, a weaker gate raises recorded throughput. These are model consequences, not observations of a company.](figures/detection-and-speed.png){width=100%}

Observed CI failures alone identify neither defect prevalence nor detection sensitivity. An independent, stable audit sample and an explicit follow-up horizon are needed to assess whether AI improves the delivered work or merely its chance of passing assessment.

## 7. Executed examples and computational scope

Four synthetic workflows illustrate the distinction. Every task has ten reference units; the one-off queue is two hours. No post-acceptance repair is included. The input rows are: $(c_h,g,v,p,r)=(6,0,1,.95,.98)$, $(2,1,1.5,.90,.98)$, $(2,8,1.5,.90,.98)$ and $(1,.5,7,.75,.90)$.

| Synthetic workflow | Mean human hours | Mean elapsed hours | Retained units/person-hour |
|:--|--:|--:|--:|
| Human baseline | 7.37 | 9.37 | 1.330 |
| AI saves effort and time | 3.89 | 5.89 | 2.520 |
| AI saves effort; slow agent | 3.89 | 12.56 | 2.520 |
| AI shifts work into review | 10.67 | 12.67 | 0.844 |

The second and third workflows consume the same human effort, but only the second finishes sooner than the baseline. The fourth reduces human production time while increasing verification enough to lose both advantages.

![Synthetic expected labour and elapsed time, each normalised to the human baseline. The shared effort coordinate of the two left-hand cases makes the role of agent duration visible.](figures/effort-versus-time.png){width=90%}

The executable supplement checks the binomial arithmetic, HMM observational equivalence, variance counterexample, retry reversal, twelve rational detection cases and twenty-five capped-retry boundary cases. Three seeds (17, 41, 93) each simulate 100,000 tasks per workflow. Exact geometric means and variances accompany Monte Carlo errors. Chebyshev's inequality supplies a conservative 99% absolute-error bound of ten analytic standard errors for each simulated mean under the generating assumptions. It is not a joint confidence guarantee across all experiments.

For a deliberately small complete-data prediction baseline, the supplement fits two log outcomes using a shared design matrix and QR least squares at $n=1{,}000$, $10{,}000$ and $100{,}000$, with twelve features. This demonstrates executable calibration, prediction and analytic uncertainty calculations on well-conditioned synthetic inputs. The fitted outcomes are generic synthetic log endpoints, not measurements from the four workflow scenarios. The computed 1.96-standard-error bands concern approximate pointwise fitted log means under independent homoscedastic Gaussian errors. They are neither prediction intervals nor simultaneous two-outcome or policy-dominance confidence bounds. The coefficient on AI in an empirical regression would remain associational without a separate identification design.

| Operation | Algorithm and bound | Executed scope |
|:--|:--|:--|
| Scenario means and bounds | Constant arithmetic per case; $O(M)$ for $M$ cases | Four workflows; 10,001-point detection grid |
| Retry simulation | Geometric draws; $O(n)$ sampled tasks, $O(n)$ stored output | Three seeds, 100,000 tasks per case |
| Complete-data calibration | QR least squares; $O(np^2+p^3)$ time, $O(np)$ space | $n\leq100{,}000$, $p=12$ |
| Prediction and mean uncertainty | $O(np)$ and $O(np^2)$ respectively | Same synthetic workload |
| Pointwise threshold screening | Compare two log-mean upper bounds with zero; $O(n)$ | Illustrative arithmetic; no policy inference |

The host used Python 3.12.14 and an AMD EPYC 9V74 processor, with one BLAS thread. The recorded numerical results give per-operation timings, resident-memory high-water marks, conditioning and residual checks. The 100,000-row fit and uncertainty calculation each took less than 0.1 seconds in the recorded run; process memory remained below 200 MiB. These are measurements on a shared host, not worst-case guarantees.

Float64 is used for plots and regression, with exact rational arithmetic for the bounded checks. The reported regression residual is below $10^{-10}$ and the generated design is well conditioned. These checks do not certify every admissible numerical input. Near-zero probabilities require the finite-cap formula and stable endpoint handling; near-zero ratio denominators require reporting an undefined or unstable quantity. Monte Carlo samples human effort; elapsed-time and retained-output values use analytical formulae. Stochastic quality follow-up, shared queues and missingness are not simulated. General censor-aware fitting, causal estimation, clustered uncertainty and empirical calibration were not executed. Consequently the full model is a research specification, not an accepted production forecasting system. Analytical propositions have explicit proofs and independent review; no theorem-prover certificate is claimed.

## 8. Estimation and validation with company data

### 8.1 Delivery without selecting only successful tasks

For enrolled tasks define the acceptance cumulative incidence $F_A(u)=\Pr(T_i\leq u)$ and restricted expected nonacceptance time

$$
M_A(\tau)=\mathbb E[\min(T_i,\tau)]
=\int_0^\tau [1-F_A(u)]\,du. \tag{15}
$$

**Proof.** Pointwise, $\min(T_i,\tau)=\int_0^\tau\mathbf1\{T_i>u\}\,du$, including $T_i=\infty$. Interchange a nonnegative integral and expectation. Smaller $M_A$ means less time spent without acceptance within the fixed horizon; report $F_A(\tau)$ alongside it.

When all task outcomes are followed to $\tau$, empirical averages estimate these quantities directly. With unconditional independent administrative censoring and no terminal competitor, an ordinary survival estimator is appropriate. Independence only conditional on covariates requires conditional, stratified or justified weighted estimation before marginal standardisation. With permanent abandonment, use acceptance cumulative incidence with abandonment as a competing event. Treating abandonment as if it were independent loss of observation changes the estimand. Sparse data favour stratified nonparametric summaries over a large state model. The required censoring and competing-event assumptions must be checked in the actual dataset before deploying a fitted estimator.

### 8.2 Prediction and causal attribution answer different questions

Start with direct task accounting and a simple task-class baseline. For positive complete effort, a small Gamma model with a log link is a candidate; zero-human-effort tasks require an explicit zero component. A censored time model is a candidate for delivery prediction where its event assumptions hold. Features must be available at the declared prediction time. Exponentiating an estimated conditional mean of log labour gives a conditional geometric mean. It also gives a median when the log residual has median zero, as in a correctly specified lognormal model; the arithmetic mean requires retransformation using the residual distribution. A Gamma model with a log link instead specifies the log of the arithmetic conditional mean directly. Marginal effort and time models do not identify the probability of meeting both a budget and a deadline without dependence information.

Reserve later periods and distinct developers or teams for validation. Compare absolute errors, calibration, interval coverage, acceptance probabilities and retained-output assessment. Propagate uncertainty from activity sampling, missingness and reference-score fitting. Binned correlation is a supplementary diagnostic. Include rejected, abandoned and unfinished work in the cohort, and retain the original task identity before any augmentation.

Randomise access or staged rollout where feasible. Analyse all assigned tasks, with uncertainty at the assignment level. Shared reviewers and queues may require team-level assignment. The primary contrast is the effect of the assigned policy; actual usage is a post-assignment choice. Tools, versions and permitted autonomy belong in the intervention record.

Where adoption already varies by engineer, cohort-time effects provide an established alternative to a single pooled before/after coefficient [7]:

$$
\operatorname{ATT}(g,t)=
\mathbb E[Y_{it}(g)-Y_{it}(\infty)\mid G_i=g]. \tag{16}
$$

$G_i=g$ denotes a documented persistent adoption/access cohort; $Y_{it}(g)$ is its potential outcome after adoption at $g$, and $Y_{it}(\infty)$ the outcome without adoption. This defines the causal quantity, not its identification. Estimation requires suitable untreated or not-yet-treated comparisons, no anticipation, conditional parallel trends, overlap and a defensible treatment of spillovers. When everyone has adopted, later untreated counterfactuals are not identified by that design alone. Reversible usage and changing intensity require a different treatment model. A step detected in SCH must not itself define AI adoption: doing so selects treatment timing from the outcome.

## 9. Decision rule and immediate study plan

For comparable scope, estimate the changes in all-role labour, elapsed delivery and quality separately. Prefer a policy only when the intended improvement is supported and its quality constraint is met at a domain-justified margin. If labour improves while delivery worsens, report that trade-off and investigate the critical path or queue. If all plausible measurement models rank the policies in the same direction, the decision may be robust despite uncertainty in absolute hours; if rankings disagree, collect the missing anchor before claiming a reliable choice.

A practical pilot has five steps:

1. Fix the task population, intake timestamp, acceptance rule, reference-score version, follow-up and AI policy.
2. Collect all-role labour samples, task lifecycle events and actual AI exposure. Preserve missingness, failures and abandoned tasks.
3. Assess a blinded quality sample under a common test and review procedure; follow accepted functionality and repair effort for the same horizon.
4. Compare simple accounting and low-dimensional predictive baselines on held-out groups and periods. Use a separately justified randomised or staggered-adoption design for causality.
5. Report labour, acceptance probability, elapsed time, retained-output rates, quality and resource cost together, with uncertainty and task mix.

This retains the useful ambition of a common reference scale while making the AI-era question testable: how much human work is required, when does acceptable functionality arrive, and how much remains acceptable after follow-up? No single change-size score answers all three.

## References

[1] Wright, I. and Ziegler, A. (2019). *The standard coder: a machine learning approach to measuring the effort required to produce source code change*. arXiv:1903.02436v1. <https://arxiv.org/abs/1903.02436>.

[2] Peng, S., Kalliamvakou, E., Cihon, P. and Demirer, M. (2023). *The Impact of AI on Developer Productivity: Evidence from GitHub Copilot*. arXiv:2302.06590. Section “Results”; completion-time estimate conditions on completion. <https://arxiv.org/abs/2302.06590>.

[3] Becker, J., Rush, N., Barnes, E. and Rein, D. (2025). *Measuring the Impact of Early-2025 AI on Experienced Open-Source Developer Productivity*. arXiv:2507.09089v2. <https://arxiv.org/abs/2507.09089>.

[4] Cui, K. Z., Demirer, M., Jaffe, S., Musolff, L., Peng, S. and Salz, T. (2026). *The Effects of Generative AI on High-Skilled Work: Evidence from Three Field Experiments with Software Developers*. Management Science, published online 27 February 2026. doi:10.1287/mnsc.2025.00535. <https://doi.org/10.1287/mnsc.2025.00535>. Published abstract checked; the full empirical analysis was not reproduced.

[5] Becker, J., Rush, N., Cunningham, T., Rein, D. and Mahamud, K. (2026). *We are Changing our Developer Productivity Experiment Design*. METR, 24 February. <https://metr.org/blog/2026-02-24-uplift-update/>. Research update, not a validated new effect estimate.

[6] Gallager, R. (2011). *Discrete Stochastic Processes*, Lecture 12: Renewal Rewards, Stopping Trials, and Wald's Inequality. MIT OpenCourseWare. <https://ocw.mit.edu/courses/6-262-discrete-stochastic-processes-spring-2011/resources/lecture-12-renewal-rewards-stopping-trials-and-walds-inequality/>.

[7] Callaway, B. and Sant'Anna, P. H. C. (2021). *Difference-in-Differences with Multiple Time Periods*. Journal of Econometrics 225(2), 200--230. Author manuscript: arXiv:1803.09015v4. <https://arxiv.org/abs/1803.09015>.

Evidence search date: 12 October 2026. The contribution combines established accounting, dependency-path, renewal and statistical ideas; no claim of an exhaustive literature review or a new general mathematical theory is made.
