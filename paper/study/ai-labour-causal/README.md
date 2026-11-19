# A causal comparison of AI-assisted software work

This proposed trial has reached the planning stage. Both candidate designs
miss the provisional target for detecting the stipulated benefit. A workplace
measurement pilot would supply evidence for choosing a main design.

## Question and estimands

For the participating teams and their fixed requirements, does offering the
specified AI workflow save more than one post-intake person-hour per initial
reference unit while the loss in retained score stays below five percentage
points? The effect of the offer includes changes in review and repair within
the observation window. Uptake describes how often each team uses the workflow.
An effect on which requests reach intake would require following incoming work.

Before choosing comparison groups, each team records its initial task set, the
requirements and acceptance criteria, reference scores, contributors and study
boundaries. Teams are paired using repository domain, previous workload and
experience. A fair independent coin within each pair selects which team receives
AI access. The other team uses the same ordinary development, test and review
process without generative AI. Use parallel groups throughout; switching groups
would introduce learned skills, shared code and unfinished tasks into a carryover
comparison. Exact tool/model versions, permissions, prompts, concurrency limits
and resource budgets must be recorded in a common manifest before the pilot.
That manifest is an outstanding design input, not a product claim.

Let `C_g` be team g's initial task set and `B_g=sum_i S_i^0>0` its reference total.
Under group condition z define:

- `L_g(z)=sum_(i in C_g) H_i(tau+Delta;z)/B_g`, in person-hours per reference unit.
- `F_g(z)=sum_(i in C_g) Q_i(tau,Delta;z)/B_g`, the fraction of initial reference
  output accepted by the deadline and retained at follow-up.

For this concrete planning comparison, one recorded initial requirement-task has
one reference unit (`S_i^0=1`); the principal workloads therefore have 20 fixed
units per team. Complexity differences remain in the labour outcomes and in
the pretreatment matching. This does not assert that all tasks require equal work.

The two primary causal quantities are the equal-team mean differences
`Delta_L=mean_g[L_g(1)-L_g(0)]` and
`Delta_F=mean_g[F_g(1)-F_g(0)]` for the participating teams and their initial work.
Their denominators are fixed before group selection. Abandoned, failed and
unfinished tasks remain in both initial task sets and labour accounting; they
contribute zero retained output if not accepted by `tau`. No analysis selects
only completers or divides by successful tasks. The team-weighted estimands
differ from a ratio of pooled output and labour; report that ratio secondarily.

For each pair, subtract the no-AI team's outcome from the AI team's outcome and
average the pair differences. Its randomisation expectation equals the average
effect over the two teams in every pair. Exact unbiasedness of this point
estimator does not make its Student-t interval exact. The t procedure is a
small-sample approximation whose coverage is examined in the planning simulation.
The approximation will be reassessed with pilot-supported outcome distributions.

## Comparison, clocks and measurement

The provisional principal horizon is `tau=Delta=336` hours (14 calendar days each), with a 672-hour labour window.
Every task is followed for 28 days from the recorded start. Quality is checked
14 days after first acceptance, for tasks accepted by day 14. Labour continues
through day 28 for all tasks, including prompting, coordination, reviews,
unsuccessful attempts and repairs. Delayed acceptance and loss of observation
remain different records. The labour and quality clocks therefore match the
paper's definitions; repair triggered by a follow-up check is included if it
occurs within the labour window.

The pilot uses a complete person-time ledger with 15-minute intervals and
participant-recorded fractions attributed to the recorded tasks and work types.
The ledger must include reasoning away from the keyboard, shared review,
coordination and work outside normal hours. A separately recorded list of people
who contributed prevents the telemetry event stream from defining the labour
frame. Each person's fractions must sum to at most one across all tasks and teams. Record shared review and platform time once, using a rule fixed before group selection to divide it among contributing tasks; never credit the full same interval to several tasks or teams. Retain ambiguous attribution as an interval or unresolved record, without replacing it by an exact total. Records not attributable
to individual tasks remain separate and visible. Diaries and reviewers can still
miss work or misclassify attention; a consistent ledger is not proof of accuracy.

Use independently checked calibration cases with known scripted durations,
deliberately overlapping records, off-keyboard work and missing contributors.
Randomly select audit shifts in both groups, including on-call and shared review
work. Collect evidence independently of the diary, for example by observing a
consenting engineer during randomly chosen periods and checking incident or
review attendance against a separate participant's account. A meeting's elapsed
time alone gives no allocation to a requirement. Document what each source
could establish about the original interval, how observation might have changed
the work, and bounds on attributable hours both sources could have missed.
Compare the resulting measurement-error ranges by group and report how sampling
audit shifts affects their uncertainty. If the plausible difference in group
errors exceeds an allowance, repeat planning over the wider range. If the source
coverage cannot justify any useful error limit, report which activities remain
unmeasured and leave the benefit claim unresolved.

Two reviewers, unaware of comparison groups as far as the material permits,
check acceptance criteria and retained functionality. Check all pilot outputs
twice and a preselected principal-study subset twice. Include known correct,
incorrect and ambiguous examples to test the reviewers' ability to detect the
specified defects. Log any unblinding and feedback that triggers repair. Record
severe security or reliability failures separately; the retained fraction is not
a complete consequence measure.

Do not share reviewers, developers or scarce AI execution resources across
comparison groups where this can affect work. Examine these links before
randomisation. The present design has one team per cluster. If the links require
clusters containing multiple teams, stop and revise the pairing, estimator weights,
independent-unit count and planning while preserving the equal-team target before
randomisation. Record later cross-group reuse
and borrowing without dropping those teams. A material violation defeats the
simple no-between-team-interference interpretation; adjusting for observed tool
use does not repair it.

## Decision and analysis

The following are provisional scientific planning choices requiring confirmation
before the study is preregistered:

- A worthwhile benefit saves more than **1 person-hour per reference unit**:
  `Delta_L < -1`. With a planning comparator mean of 10 hours per reference unit,
  that boundary corresponds to a 10% reduction; the natural-unit boundary is the
  actual decision rule.
- A loss of **5 percentage points or more** in retained score must be ruled
  out (`Delta_F > -0.05`). A precisely measured four-point loss can pass.
- The simulated useful effect is **-2 hours per reference unit with no quality
  loss**, which lies beyond both decision boundaries. Boundary scenarios are
  used to assess false success, not to demand high power at the null boundary.

This is a sensitivity analysis at nominated effect sizes; no minimum detectable
effect has been estimated or substituted for the decision boundary.

Use one-sided 97.5% Student-t bounds from independent pair differences. Expand
the labour interval by a provisional 0.2 hours per reference unit and the
retained-output interval by 0.01 to allow for a bounded difference in measurement
bias. These are sensitivity assumptions, not validated error limits. Declare
benefit only when the expanded labour upper bound is below -1 **and** the
expanded quality lower bound is above -0.05. This conjunction is a claim about labour and retained required output, not an overall safety or adoption decision. Severe defects remain separately reported and can defeat an overall-benefit interpretation even when these two conditions pass; their incidence is not simulated. A false joint claim implies rejection
of at least one true component null, so separate level-0.025 component procedures
control the conjunction at 0.025 if each component's assumptions hold. No
independence of the two outcomes is needed for that logical implication. The
logic does not make the approximate t procedures exactly level 0.025. Report
each true-null component's rejection rate as well as joint false success.

Retain original groups regardless of tool uptake. Actual use is a diagnostic,
not a basis for selecting the primary analysis population. Report both primary
effects and intervals even when the conjunction fails. Distinguish evidence of
harm, exclusion of a worthwhile labour saving and unresolved intervals. Excluding
a worthwhile saving does not establish a practically small effect: that would
require a declared equivalence region and an interval wholly within it.
The lower confidence limit for the retained fraction must exceed the prespecified loss margin.

The conservative planning procedure permits no affirmative benefit claim if a
team-level endpoint remains missing. Keep the team and publish physical or
observation-based bounds and the missingness sensitivity, rather than silently
using available pairs. A future procedure allowing a benefit decision with
missing endpoints would need its own prespecified bounds and operating-characteristic
analysis; it is not silently included in the current power results.

An incomplete experiment returns the full parameter range for each effect and
cannot declare a benefit. The reported unconditional coverage counts every such
range as covering. In the 96-team loss scenario, 4,212 of 20,000 experiments
returned these uninformative ranges. Of the 15,788 experiments that supplied
finite intervals, 15,676 covered the labour effect (99.29%) and 15,777 covered
the retained-fraction effect (99.93%). These counts follow by subtracting the
4,212 incomplete experiments from each unconditional coverage count.
The finite-width and bias summaries also use complete experiments.

## Pilot and conditional principal-study rule

The specified pilot contains **six team pairs, six initial tasks per team**, with
the same 672-hour (28-day) clocks. Its purpose is to rehearse collection, reviewer checks,
tool delivery, cross-team separation and analysis, and estimate uncertain
variability and cost. It is not an efficacy test. Six pair differences give only
five variance degrees of freedom; retain the broad uncertainty in nuisance
estimates. The 72 tasks and repeated person-intervals are not 72 independent
randomised units. Pilot tasks and outcome records do not enter the later
confirmatory comparison.

The pilot's coverage plan is two pairs from each of three repository/work types
within the intended principal population, chosen before outcomes. This count
supports a bounded rehearsal across those settings; it does not establish precise
variance or rare-missingness rates. If those settings or complete records cannot
be obtained, revise the pilot's coverage and purpose before collecting data.

Before preregistration, justify the natural-unit margins from the intended
decision and domain evidence, and record the toolset manifest. The pilot then determines ranges for
between-team variability, the effectiveness of matching, outcome dependence,
tail behaviour, ledger completeness, differential error, follow-up loss and
collection/review cost. Do not use its favourable treatment effect to choose a
principal sample size. Re-run the specified planning calculation using those
supported nuisance ranges and externally justified useful effects. Select the
least costly feasible allocation only if:

1. the lower 95% Monte Carlo bound on joint success is at least 0.90 in every
   required useful-effect scenario;
2. the upper Monte Carlo bound on false joint success is at most 0.035 in each
   component-null/boundary scenario, with at least 0.94 coverage for each nominal
   sensitivity-expanded effect interval, and the same false-rejection tolerance
   for each true component null;
3. the required measurement/error and completeness premises are supportable;
4. the actual available team and observation budgets cover all tasks, failed
   attempts, checks and follow-up.

These planning tolerances were specified before these simulations. They cannot
be loosened merely because a design fails. If neither allocation qualifies,
improve measurement or matching, find more independent teams/resources, or
report a precision-focused pilot with narrower permissible claims. Do not change
the target workload or quality margin to manufacture adequacy.

## Numerical comparison actually executed

`planning.py` compares **48 teams (24 pairs)** with **96 teams (48 pairs)**.
It generates team-level baselines, adds fixed differences in the two outcomes
and uses a fair coin in each pair to determine which team receives AI access.
The generated pair contrasts receive measurement noise and, where specified, a
fixed difference in reporting bias. Each cell contains 20,000 simulated
experiments. Twenty initial tasks per team determines the budget and reported
task count only. This simulation therefore provides no estimate of the gain in
precision from observing additional tasks per team. A larger trial would need
evidence about between-task as well as between-team variability to estimate
that gain.

A separate [controlled measurement experiment](measurement_bridge.py)
passes differences in measured labour and retained fraction between paired teams
through the same decision rule. The constructed work histories supply the true
differences, so the experiment can trace the effect of incomplete measurements
on the decision. It imports effort, acceptance and assessment observations into
the Rust store for twelve teams and passes the cohort reports to the trial
analysis. With accurate reports the mean differences reach the two decision
thresholds exactly. Omitting review time and overlooking failed checks in the
AI group moves both measured means beyond their thresholds and produces a
false declaration. One missing follow-up assessment then prevents a declaration.
These three checks hold the 24 requirements fixed. The following simulations
draw new team outcomes in every replication.

| Scenario | 48 teams: joint success | 96 teams: joint success |
| --- | ---: | ---: |
| Useful effect, central nuisance values | 44.79% | 77.25% |
| Useful effect, weaker matching and larger variation | 2.00% | 6.91% |
| Useful effect, 0.5% probability a pair lacks an endpoint | 39.77% | 60.66% |
| Bias exceeds the assumed allowance at both null boundaries | 47.81% false success | 76.21% false success |

In the final row, the AI group's labour is understated by one hour per unit
relative to the comparison group and its retained fraction is overstated by
six percentage points. A larger sample narrows intervals around those biased
differences. With bias at the declared allowance, observed false joint success is
0.01%/0.02%. With maximal favourable bias on the true-null component alone, labour-boundary rejection is 2.475%/2.560% and quality-boundary rejection is 2.580%/2.385%. The largest observed component rejection across the included valid-bias cases is 2.760%, rather than exactly the nominal 2.5%; the declared upper Monte Carlo tolerance is 3.5%. The partial-null cases and each endpoint's interval coverage are
retained in `planning-results.json`. Wilson intervals remain nondegenerate even
when no simulated successes occur. Monte Carlo error is much smaller than the
principal shortfall. The reported coverage concerns the t intervals expanded by the sensitivity allowances, not unexpanded t coverage or exact design-based coverage. Neither candidate qualifies, so no principal design is
selected. The simulation does not measure recruitment or address tool-version
drift, rare severe failures or effects that vary across teams.

The adverse scenario changes several assumptions together. After the original
design assessment, we reran the central scenario, replacing one parameter at
a time by its adverse value. Each comparison uses the same latent draws,
treatment coins and noise draws as the central scenario for that team count.
The two-hour saving, unchanged retention and decision thresholds stay fixed.
At 96 teams, reducing latent within-pair correlation from 0.7 to 0.2 produced
6,930 benefit declarations in 20,000 experiments. Increasing only the standard
deviation of the lognormal labour term on the log scale from 0.35 to 0.65
produced 5,677. Increasing only the labour measurement-noise standard deviation
from 0.2 to 0.6 hours per unit produced 14,183. The central count was 15,450.
These contrasts isolate each parameter *with the others at their central
settings*. Their differences need not add up to the total adverse effect.
The [supplemental table](planning-sensitivity.csv) also reports the two
retention-parameter comparisons, both component pass counts, and the Wilson
interval for each success probability. The original design assessment and its
decision thresholds use only the original scenarios.

Under the simulation's independent pair-loss model, the requirement for every
endpoint to be available also imposes an analytic ceiling. With pair-loss
probability `p` and `J` pairs, joint success cannot exceed `(1-p)^J`, even with
perfect statistical detection. At `p=0.005`, the ceilings are **88.67%** for 24
pairs and **78.62%** for 48 pairs. Increasing the number of teams alone cannot
reach the 90% target under that missingness scenario. Collection must achieve
lower endpoint loss, or a revised missing-data decision procedure must be
specified and its operating characteristics evaluated before selecting a
principal allocation.

The independence assumption matters for that ceiling. For example, if one
common outage erased all pairs with probability 0.005 and data were otherwise
complete, each pair would still have marginal loss probability 0.005. All pairs
would then be complete with probability 0.995 at either sample size. The pilot
should identify shared causes of loss and examine missingness by workload and
group before adopting a loss model.

The compared observation budgets are provisional **1,056** and **2,112**
collection/review person-hours (`22` per team: 12 collection hours plus 0.5 hours
per initial task). They exclude the unknown development labour being measured,
hardware charges and local setup costs; actual affordability remains an input.
The pilot has a provisional cap of 200 collection/review hours. No money has
been committed and no human study has started.

## Sources and verification

The source audit in `references.md` records methodological consequences and
proposed BibTeX. The protocol follows schema 1.2 of the invoked
`design-scientific-investigations` skill. Its template and contract resources
were readable; the exact advertised `scripts/validate_investigation.py` resource
returned “failed to read skill resource”. The retained `validate_protocol.py`
therefore performs explicitly bounded local structural/reference/numeric checks;
it is not presented as the bundled skill validator or as scientific validation.

```sh
python3 paper/study/ai-labour-causal/planning_sensitivity.py --check
python3 paper/study/ai-labour-causal/measurement_bridge.py
python3 paper/study/ai-labour-causal/validate_protocol.py
```

The first command recomputes the original 24 cells and the supplemental
one-parameter comparisons. It checks every original numeric result, source
hash and scenario parameter, with Python 3.12 and the NumPy and SciPy versions
pinned in `requirements.txt`. A different Python patch release is permitted
when those results reproduce exactly. To verify the original environment
metadata as well, run `planning.py --check` with Python 3.12.14. Both checks
leave the retained results and their original provenance untouched.

`protocol.json` is **specified**. Its planning assessment is **evaluated under
provisional assumptions**, while the empirical pilot and principal trial remain
unrun. A structural pass or the successful simulated rehearsal does not promote
the trial to preregisterable, approved, running or complete status.
