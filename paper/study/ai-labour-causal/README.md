# A causal comparison of AI-assisted software work

This is a specified design and planning analysis, not an executed or preregistered
trial. The principal study is not yet adequate for a benefit claim: neither
candidate meets the provisional joint-success target across the specified
planning scenarios. No workplace observations were supplied. The next step is a
separate measurement and feasibility pilot, followed by supported planning.

## Question and estimands

For a fixed set of software requirements, does providing one recorded AI toolset
and its working practices reduce all included human labour while preserving
required output at follow-up? The target is the effect of providing that complete
workflow, including any changes to review and repair. It is not the effect of an
extra minute of AI use, a particular model component, or organisation-wide adoption
that changes incoming work.

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
For a preselected subset of person-days, compare contemporaneous allocation
records with a second evidence collection procedure and retain disagreement and
its residual uncertainty. Neither reviewer agreement nor a stopwatch on coding
alone establishes complete true labour. The pilot must determine whether a
defensible bound on differential measurement bias can be obtained.

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
- Retained output may fall by no more than **5 percentage points**:
  `Delta_F > -0.05`. This is a maximum acceptable loss, not a prediction.
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
Non-significance does not establish quality preservation.

The conservative planning procedure permits no affirmative benefit claim if a
team-level endpoint remains missing. Keep the team and publish physical or
observation-based bounds and the missingness sensitivity, rather than silently
using available pairs. A future procedure allowing a benefit decision with
missing endpoints would need its own prespecified bounds and operating-characteristic
analysis; it is not silently included in the current power results.

In the current simulation an incomplete endpoint produces the full parameter-
domain confidence set and no component rejection. Reported unconditional coverage
includes those uninformative sets; the finite halfwidth and bias summaries are
explicitly restricted to simulated complete records. Missing generated values
are not used to report a finite interval that the actual procedure could not form.

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

`planning.py` compares **48 teams (24 pairs)** with **96 teams (48 pairs)**,
20 initial tasks per team. It simulates 20,000 experiments per design/scenario
cell, using actual within-pair coins, correlated team baselines, positive labour,
bounded true retained fractions, measurement noise, bias allowances, and the
prespecified conjunction/missing-endpoint rule. The latent distributions are
planning assumptions, not estimated workplace distributions. No treatment effect
or variance from another developer study is imported as if it applied here.

| Scenario | 48 teams: joint success | 96 teams: joint success |
| --- | ---: | ---: |
| Useful effect, central nuisance values | 44.79% | 77.25% |
| Useful effect, weaker matching and larger variation | 2.00% | 6.91% |
| Useful effect, 0.5% probability a pair lacks an endpoint | 39.77% | 60.66% |
| Bias exceeds the assumed allowance at both null boundaries | 47.81% false success | 76.21% false success |

The final row demonstrates a failure of a measurement assumption, not a valid
power improvement. Increasing sample size can make that biased decision more
confident. With bias at the declared allowance, observed false joint success is
0.01%/0.02%. With maximal favourable bias on the true-null component alone, labour-boundary rejection is 2.475%/2.560% and quality-boundary rejection is 2.580%/2.385%. The largest observed component rejection across the included valid-bias cases is 2.760%, rather than exactly the nominal 2.5%; the declared upper Monte Carlo tolerance is 3.5%. The partial-null cases and each endpoint's interval coverage are
retained in `planning-results.json`. Wilson intervals remain nondegenerate even
when no simulated successes occur. Monte Carlo error is much smaller than the
principal shortfall. The reported coverage concerns the t intervals expanded by the sensitivity allowances, not unexpanded t coverage or exact design-based coverage. Neither candidate qualifies, so no principal design is
selected. Recruitment, tool-version drift, rare severe failures, heterogeneous
treatment effects and generalisation are not established by these calculations.

Under the simulation's independent pair-loss model, the requirement for every
endpoint to be available also imposes an analytic ceiling. With pair-loss
probability `p` and `J` pairs, joint success cannot exceed `(1-p)^J`, even with
perfect statistical detection. At `p=0.005`, the ceilings are **88.67%** for 24
pairs and **78.62%** for 48 pairs. Increasing the number of teams alone cannot
reach the 90% target under that missingness scenario. Collection must achieve
lower endpoint loss, or a revised missing-data decision procedure must be
specified and its operating characteristics evaluated before selecting a
principal allocation.

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
python3 paper/study/ai-labour-causal/planning.py --check
python3 paper/study/ai-labour-causal/validate_protocol.py
```

`protocol.json` is **specified**. Its planning assessment is **evaluated under
provisional assumptions**, while the empirical pilot and principal trial remain
unrun. A structural pass or the successful simulated rehearsal does not promote
the trial to preregisterable, approved, running or complete status.
