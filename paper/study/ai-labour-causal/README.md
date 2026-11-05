# A causal comparison of AI-assisted software work

The study design is specified, and its planning simulations have been run. The
trial awaits preregistration and workplace data collection. Neither proposed team count
meets the provisional target for demonstrating both a worthwhile labour saving
and an acceptable output loss across the required planning scenarios. The next
step is a separate pilot to test measurement and feasibility, then repeat the
planning analysis using ranges supported by the pilot.

## Study question and target effects

For fixed software requirements, does providing a specified AI workflow reduce
total person-hours while keeping any loss of requirement fulfilment within an
agreed limit? Fulfilment is assessed a specified time after first acceptance.
The target is the effect of providing that complete workflow, including any
changes to review and repair.
Effects of the amount of AI use, individual model components and organisation-wide
changes to incoming work would require different comparisons.

Before choosing comparison groups, each team records its initial task set, the
requirements and acceptance criteria, reference scores, contributors and study
boundaries. Teams are paired using repository domain, previous workload and
experience. A fair independent coin within each pair selects which team receives
AI access. The other team uses the same ordinary development, test and review
process without generative AI. Keep the same groups throughout the study. Switching groups
would introduce learned skills, shared code and unfinished tasks into a carryover
comparison. Exact tool/model versions, permissions, prompts, concurrency limits
and resource budgets must be recorded in a common manifest before the pilot.
That manifest remains to be supplied.

Let `C_g` be team g's initial task set and `B_g=sum_i S_i^0>0` the sum of its initial
reference scores. Group condition `z=1` denotes AI access, and `z=0` denotes the comparator.
Define:

- `L_g(z)=sum_(i in C_g) H_i(tau+Delta;z)/B_g`, in person-hours per reference unit.
- `F_g(z)=sum_(i in C_g) Q_i(tau,Delta;z)/B_g`, the output score per initial
  reference unit.

For a task accepted by the deadline `tau`, its output score is its reference score
multiplied by the fraction of its accepted requirement still satisfied at the
assessment conducted `Delta` hours after first acceptance. A task that misses the deadline has an
output score of zero. Both team outcomes use the same fixed initial reference
total as their denominator.

For this planning comparison, each initial task has one reference unit
(`S_i^0=1`). The main study's workloads therefore have 20 fixed units per team.
Differences in task complexity are reflected in labour outcomes and in the
matching performed before group assignment. Equal reference scores therefore
allow tasks to require different amounts of work.

The two primary causal quantities are the equal-team mean differences
`Delta_L=mean_g[L_g(1)-L_g(0)]` and
`Delta_F=mean_g[F_g(1)-F_g(0)]` for the participating teams and their initial work.
Their denominators are fixed before group selection. Each team's abandoned,
failed and unfinished tasks remain in its initial task set and labour accounting.
They contribute an output score of zero if not accepted by `tau`. The primary analysis
includes every initial task and keeps the initial reference total as its
denominator. Each team receives equal weight in the target effects. Report the
ratio of pooled output scores to pooled labour as a separate secondary quantity.

For each pair, subtract the no-AI team's outcome from the AI team's outcome and
average the pair differences. Averaged over the possible coin-toss assignments,
this estimator equals the average effect over the two teams in every pair. Its
Student-t interval uses a small-sample approximation whose coverage is examined
in the planning simulation. The approximation will be reassessed using outcome
distributions supported by pilot observations.

## Acceptance deadline, assessment timing and labour measurement

The provisional acceptance deadline and assessment delay are `tau=Delta=336`
hours (14 calendar days each), with a 672-hour labour window. Every task is
followed for 28 days from its recorded start. For tasks accepted by day 14,
reviewers assess how much of the accepted requirement is still satisfied 14 days
after first acceptance. Labour is counted through day 28 for every task, including
prompting, coordination, reviews, unsuccessful attempts and repairs. Thus an early
acceptance has an earlier assessment, while the labour cutoff remains day 28.
Acceptance after the deadline contributes an output score of zero. Loss of
observation leaves the outcome unresolved. Repair triggered by an assessment is
included when it occurs within the labour window.

The pilot uses a time-allocation ledger for each person, divided into 15-minute
intervals. Participants record the fraction of each interval spent on each task
and type of work. The ledger must include reasoning away from the keyboard, shared review,
coordination and work outside normal hours. A separate list of contributors
identifies whose work must be measured, including people absent from development
tool records. Each person's fractions must sum to at most one across all tasks
and teams. Record shared review and platform time once, using a rule fixed before
group selection to divide it among contributing tasks. When attribution is
uncertain, save bounds on attributable time or an unresolved record. Work that
cannot be assigned to individual tasks remains separate and visible. Accuracy
checks must investigate missed work and misclassified attention even when the
ledger satisfies its arithmetic constraints.

Use independently checked calibration cases with known scripted durations,
deliberately overlapping records, off-keyboard work and missing contributors.
For a preselected subset of person-days, compare contemporaneous allocation
records with a second evidence collection procedure and retain disagreement and
its remaining uncertainty. Reviewer agreement and timed coding exercises provide
partial checks on measurement accuracy. The pilot must assess coverage of all
included work and determine whether a defensible bound can be placed on the
difference in measurement bias between groups.

Two reviewers, unaware of comparison groups as far as the material permits,
check acceptance criteria and assess requirement fulfilment. Check all pilot outputs
twice and a preselected principal-study subset twice. Include known correct,
incorrect and ambiguous examples to test the reviewers' ability to detect the
specified defects. Log any unblinding and feedback that triggers repair. Record
severe security or reliability failures separately. Their consequences require
assessment alongside the fraction of an accepted requirement still satisfied.

Keep reviewers, developers and scarce AI execution resources separate across
comparison groups when sharing could change their outcomes. Examine these links
before randomisation. The present design assigns whole teams to groups. If the links require
clusters containing multiple teams, stop and revise the pairing, estimator weights,
independent-unit count and planning while preserving the equal-team target before
randomisation. Record later cross-group reuse
and borrowing while retaining those teams in the analysis. Material transmission
of effects between groups invalidates the assumption that each team's outcome
depends only on its own group assignment. Adjusting for observed tool use alone
leaves that cross-group transmission unresolved.

## Decision and analysis

The following are provisional scientific planning choices requiring confirmation
before the study is preregistered:

- A worthwhile benefit saves more than **1 person-hour per reference unit**:
  `Delta_L < -1`. With an assumed no-AI group mean of 10 hours per reference unit,
  that boundary corresponds to a 10% reduction. The saving in person-hours per
  reference unit is the decision rule.
- An acceptable loss in output score per reference unit is less than **5 percentage points**:
  `Delta_F > -0.05`. This sets the maximum permitted loss.
- The simulated useful effect is **-2 hours per reference unit with no reduction
  in output score per reference unit**, which lies beyond both decision boundaries.
  Scenarios at either decision boundary assess the frequency of false benefit claims.

This sensitivity analysis evaluates nominated effect sizes. The decision boundary
is fixed independently of any minimum detectable effect, which has not been estimated.

Use one-sided 97.5% Student-t bounds from independent pair differences. Expand
the labour interval by a provisional 0.2 hours per reference unit and the
interval for output score per reference unit by 0.01 to allow for a bounded
difference in measurement bias between groups. These allowances are sensitivity
assumptions requiring empirical validation. Declare
benefit only when the expanded labour upper bound is below -1 **and** the
expanded output lower bound is above -0.05. Meeting both criteria supports a claim
about labour and output scores. A decision to adopt the workflow also requires
assessment of severe defects and other consequences. Severe defects can rule out
adoption even when the two numerical criteria pass. They are reported separately,
and their incidence is outside the simulation.

A false joint claim necessarily rejects at least one true component null
hypothesis, meaning that at least one benefit criterion is actually unmet.
Separate level-0.025 component procedures therefore control the probability of a
false joint claim at 0.025 if each component's assumptions hold. This implication
holds regardless of dependence between the two outcomes. The approximate t
procedures still require calibration. Report each true-null component's rejection
rate as well as the rate of false joint claims.

Analyse teams in their original groups regardless of tool uptake. Use actual
tool-use records to describe implementation while retaining every initial team
in the primary analysis. Report both primary effects and intervals even when
the two benefit criteria are not both met. Distinguish evidence of
harm, exclusion of a worthwhile labour saving and unresolved intervals. Excluding
a worthwhile saving does not establish a practically small effect: that would
require a declared equivalence region and an interval wholly within it.
Non-significance does not establish quality preservation.

Under the conservative planning procedure, a benefit claim requires both outcomes
to be resolved for every team. When a team outcome is missing, retain the team
and publish bounds justified by available time or observations, together with an
analysis of how the missing values could change the result. A future procedure
allowing a benefit decision with missing outcomes would need prespecified bounds
and an evaluation of its false-claim rate and probability of detecting a useful
effect. The current planning results apply to the procedure requiring complete outcomes.

In the current simulation, an incomplete team outcome produces a confidence set
containing every value allowed by the model, so neither component null is
rejected. Reported coverage across all simulations includes these uninformative
sets. Finite interval half-widths and bias summaries use only simulated complete
records. The reported intervals therefore use only values the procedure could
observe.

## Pilot and conditional principal-study rule

The specified pilot contains **six team pairs, six initial tasks per team**, with
the same 672-hour (28-day) labour window and the acceptance and assessment rules
above. Its purpose is to rehearse collection, reviewer checks,
tool delivery, cross-team separation and analysis, and estimate uncertain
variability and cost. It is not an efficacy test. Six pair differences give only
five variance degrees of freedom, so estimates of variability remain uncertain.
Assignment takes place within six independent team pairs. The 72 tasks
and repeated observations of each person's time supply measurements within those
pairs. Pilot tasks and outcome records are kept separate from the later
confirmatory comparison.

The pilot's coverage plan is two pairs from each of three repository/work types
within the intended principal population, chosen before outcomes. This count
supports a rehearsal across those settings. Estimating variance or rates of rare
missing outcomes precisely would require more observations. If those settings or complete records cannot
be obtained, revise the pilot's coverage and purpose before collecting data.

Before preregistration, justify the natural-unit margins from the intended
decision and domain evidence, and record the toolset manifest. The pilot then determines ranges for
between-team variability, the effectiveness of matching, outcome dependence,
tail behaviour, ledger completeness, differential error, follow-up loss and
collection/review cost. Do not use its favourable treatment effect to choose a
principal sample size. Re-run the specified planning calculation using those
supported ranges for variability and measurement error and externally justified
useful effects. Select the
least costly feasible allocation only if:

1. the lower 95% Monte Carlo bound on joint success is at least 0.90 in every
   required useful-effect scenario.
2. the upper Monte Carlo bound on false joint success is at most 0.035 in each
   component-null/boundary scenario. Coverage of each sensitivity-expanded effect
   interval must be at least 0.94. Apply the same false-rejection tolerance to
   each true component null.
3. the required measurement/error and completeness premises are supportable.
4. the actual available team and observation budgets cover all tasks, failed
   attempts, checks and follow-up.

These planning tolerances were specified before these simulations. They cannot
be loosened merely because a design fails. If neither allocation qualifies,
improve measurement or matching, find more independent teams/resources, or
report a precision-focused pilot with narrower permissible claims. Do not change
the target workload or quality margin to manufacture adequacy.

## Numerical comparison actually executed

`planning.py` compares **48 teams (24 pairs)** with **96 teams (48 pairs)**,
20 initial tasks per team. For each team count and scenario, it simulates 20,000
experiments with fair coin assignments within pairs, correlated team baselines
and positive labour. Hypothetical output scores per reference unit lie between
zero and one. The simulation adds measurement noise and bias allowances, then
applies the rule requiring both benefit criteria and complete team outcomes.
These distributions are planning assumptions. Treatment effects and variances
from other developer studies have not been imported as workplace measurements
for this population.

| Scenario | 48 teams: joint success | 96 teams: joint success |
| --- | ---: | ---: |
| Useful effect, central variability and measurement assumptions | 44.79% | 77.25% |
| Useful effect, weaker matching and larger variation | 2.00% | 6.91% |
| Useful effect, 0.5% probability a pair lacks an outcome | 39.77% | 60.66% |
| Bias exceeds the assumed allowance at both null boundaries | 47.81% false success | 76.21% false success |

The final row shows how exceeding the assumed bound on measurement bias produces
false benefit claims. Increasing sample size can make that biased decision more
confident. With bias at the declared allowance, simulated false joint success is
0.01%/0.02%. With maximal favourable bias on the true-null component alone,
labour-boundary rejection is 2.475%/2.560% and quality-boundary rejection is
2.580%/2.385%. The largest simulated component rejection rate across the included
valid-bias cases is 2.760%, compared with the nominal 2.5%. The specified upper
Monte Carlo tolerance is 3.5%. Cases in which only one component null is true, and
each outcome's interval coverage, are saved in `planning-results.json`.

Wilson intervals retain positive width even when no simulated successes occur.
Monte Carlo error is much smaller than the shortfall in joint success. The
reported coverage concerns t intervals widened by the sensitivity allowances.
It describes their performance across the simulated experiments and supplies no
exact finite-sample guarantee for workplace observations. Neither candidate
qualifies, so no main-study design is selected. Recruitment, tool-version drift,
rare severe failures, heterogeneous treatment effects and generalisation require
further evidence.

Under the simulation's independent pair-loss model, requiring both outcomes for
every team also imposes an analytic ceiling. With pair-loss
probability `p` and `J` pairs, joint success cannot exceed `(1-p)^J`, even if the
statistical criteria are always met when records are complete. At `p=0.005`, the ceilings are **88.67%** for 24
pairs and **78.62%** for 48 pairs. Increasing the number of teams alone cannot
reach the 90% target under that missingness scenario. Collection must achieve
fewer missing team outcomes, or a revised missing-data decision procedure must be
specified and its operating characteristics evaluated before selecting a
principal allocation.

The compared observation budgets are provisional **1,056** and **2,112**
collection/review person-hours (`22` per team: 12 collection hours plus 0.5 hours
per initial task). They exclude the unknown development labour being measured,
hardware charges and local setup costs. Actual affordability remains an input.
The pilot has a provisional cap of 200 collection/review hours. No money has
been committed and no human study has started.

## Sources and verification

The source audit in `references.md` explains how the methodological sources inform
the design and records proposed bibliography entries. `protocol.json` follows
investigation schema 1.2. The local `validate_protocol.py` script checks file
structure, referenced evidence and numerical consistency. Empirical measurement
validity requires the pilot and subsequent workplace observations.

```sh
python3 paper/study/ai-labour-causal/planning.py --check
python3 paper/study/ai-labour-causal/validate_protocol.py
```

`protocol.json` is **specified**. Its planning assessment is **evaluated under
provisional assumptions**, while the empirical pilot and principal trial remain
unrun. A structural pass or the successful simulated rehearsal does not promote
the trial to preregisterable, approved, running or complete status.
