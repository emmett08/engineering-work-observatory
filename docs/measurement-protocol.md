# Measurement protocol

The study-accounting commands calculate labour totals, acceptance times and weighted scores from explicit task observations. They also bound labour when a directly observed interval crosses the end of the period being measured. The scope rule below can change the weighted score reported for a task. A separate `estimate-labour` command estimates labour from randomly selected time intervals for individual people, using known selection probabilities. These calculations require observations of human work. Comparing AI's effects requires a separate causal study design. The example records are synthetic.

## Set measurement rules before enrolling tasks

Create a `protocol` before enrolling tasks. Define which tasks qualify, whose work is included and which types of work count. State whether time spent attending a running process counts as labour. Set the acceptance criteria, the rule for scope changes and the periods for observing labour and reassessing requirements. Record these choices in `eligibility_notes`, `role_convention_notes` and `attended_waiting_counts`. Keep them unchanged after observing outcomes. A revised design requires a separately identified protocol and task cohort.

Register every eligible requirement, including postponed or abandoned work. Freeze its scope and reference score before implementation. A defensible first score is one unit per eligible requirement, reported by task class. Its unit is a task equivalent. Interpreting it as human hours or customer value would require separate calibration. Connect PRs, commits, sessions and cross-repository changes with `task_link` records. Split implementation work without resetting the original enrolment clock or counting the same requirement in both parent and child weighted scores.

`assigned_policy` and `assignment_unit` retain policy metadata. Actual use belongs in `ai_usage` or linked source evidence. Tool availability, assigned policy, branch names, workflow actors and observed assistance are different observations. EngArg and EAL are coupled components; two repositories and many commits do not supply independent experimental assignments.

## Record human effort

Include specification, prompting, implementation, review, testing, integration, coordination and repair according to the frozen convention. Record people pseudonymously. Unattended agent execution is separate resource use. Two people pairing for one hour supply two person-hours. A person dividing one hour between two agents can contribute at most one person-hour.

A directly observed interval contributes its duration in hours multiplied by `allocation_fraction`. That fraction is the average share of the person's time allocated to the task and type of work across the whole interval. The person may have switched activities within it. When the observation interval extends beyond the labour period, separate observations on each side of the boundary can determine the contribution inside the period. A whole-interval average strictly between zero and one leaves that contribution unresolved. The software then keeps `person_hours` unavailable and reports contributions from other identified intervals in `recorded_person_hours`. It also reports the range permitted by the crossing interval's observed average and the person's available time.

Let `m` be the labour observed over the complete interval, `u` its duration inside the labour period and `v` its duration outside. The permitted range inside the period is `[max(0, m-v), min(m, u)]` person-hours. It assumes `0 <= allocation <= 1` at every time except sets of total duration zero. Allocating the observed labour outside the period first achieves the minimum. Allocating it inside first achieves the maximum. Both limits are therefore attainable for this interval alone. Other observations and tasks sharing the same person's time can narrow the range. An observed average of exactly zero or one determines zero labour or the full duration inside the labour period, respectively. Values merely close to zero or one retain a range. Calculations preserve subsecond duration before floating-point accounting, whose rounding can move a reported limit inward. Study records still require labour to start at or after task enrolment.

The optional task `person_hours_bounds` and cohort `total_person_hours_bounds` sum the direct interval ranges. The summed minimum or maximum may require allocations that cannot occur together when tasks or types of work share a person's time. The separate `bound-labour` command enforces these shared constraints in a joint model. It accepts explicit constraints rather than reinterpreting additive study records. See [exact labour identification](labour-identification.md).

Bounds on the complete labour total require a frame covering all target work with direct observations under the stated measurement method. Every required response must be observed, the labour period must have ended and all observations intersecting it must be complete. Missing, partial, conflicting or sampled frames leave complete-total bounds unavailable. Ranges from individual crossing intervals remain visible even when the whole frame is incomplete. Use `coverage` records to document the lifecycle and labour observation frames. Complete direct coverage with no positive labour observations attests a measured zero. Missing coverage leaves the total unknown.

Sampled intervals crossing the boundary remain unsupported under the sampling-frame contract. A complete census requires a declared frame and observations of every unit in it. Deterministic direct bounds are neither confidence intervals nor a replacement for sampling uncertainty.

Supply one allocation observation per task, person and role on each interval cell. Two observations in that same category cannot overlap for a positive duration, even when their interval boundaries differ. The end of one observation can coincide with the start of another. Different tasks or roles can overlap, subject to the method rules and a conservative screen requiring the sum of overlapping interval averages to be at most one. This is a sufficient input-format restriction, not a diagnosis of actual instantaneous over-allocation: averages over different intervals may fail the screen even when the underlying activity was feasible. Supply a common partition supported by observations to resolve such a rejection.

Complete coverage attestations with the same measurement method may jointly cover an effort interval. Adjacent or overlapping attestations do not split the observation, interpolate its allocation, or change a sampled unit or its inclusion probability. Conflicting partial or missing coverage remains unresolved.

For probability sampling, list the people and time intervals that cover the target work. Select intervals randomly, using a design that gives each interval a known positive selection probability, before examining their allocations. A selected interval contributes:

```text
duration_hours * allocation_fraction / inclusion_probability
```

Retain selected zero allocations as observed zeros. Retain nonresponses as `response_status="missing"`, null allocation and a reason. A sampled frame needs actual sampled units. Crossing intervals need separately observed within-boundary allocations under the declared sampling design. Multiplying the whole-interval average by a clipped duration leaves that within-period allocation unresolved. Do not mix direct and sampled observations of the same person-time or add diary totals to estimates for that same period.

Task and cohort reports calculate a Horvitz–Thompson estimate of total labour. These effort records lack the complete joint sampling design needed to calculate variance or a confidence interval. An estimated total remains subject to sampling uncertainty and cannot serve as an exact observed total in the direct labour-bound calculation, including when a frame mixes direct and sampled observations. The separate `estimate-labour` path accepts its own explicit design contract and can report design-specific variance, including finite-population variance under simple random sampling without replacement. These calculations use the standalone design input and leave generic cohort effort records unchanged. Sampling weights do not repair missing responses, inaccurate allocations or work outside the frame. Nonresponse, incomplete frames and method mismatches leave complete labour unavailable while showing recorded contributions. Measure the effort required to supply responses in a pilot and independently check a subset of allocations before interpreting estimates.

## Record acceptance and reassess requirement fulfilment

Record first acceptance explicitly with its evidence and revision links. Record merging and deployment as separate events. Record permanent abandonment as an outcome. `lost_to_follow_up` records loss of observation and leaves the later outcome unknown. For a task observed throughout the acceptance period without acceptance, elapsed time capped at the deadline equals the full period. A task observed for less than that period, or lost from observation, has an unresolved outcome.

This implementation requires an assessment for **every counted accepted requirement**. At first acceptance and again after the chosen time since acceptance, assess how much of the requirement is satisfied. Use the same assessment criteria and independent reviewers where feasible. Record `reviewer_id`, `blinded`, disagreements, adjudication and intentional replacement notes. Keep reviewer observations nonprimary and designate one adjudicated `is_primary=true` result. Record inability to blind rather than assuming it succeeded.

Passing tests, surviving lines and no reported incidents do not establish `retained_fraction=1`. Record defects and severity independently of that fraction. Link repair work to its existing labour record so it is counted once. Use a common audit/feedback procedure and record audit-triggered repair in the applicable labour boundary. Independent quality subsampling, audit-exposure adjustment and quality sampling weights are outside this release.

## Apply the three timing rules

The synthetic example uses `acceptance_days=30` and `follow_up_days=30`. The paper prescribes no universal values. Days are fixed 24-hour UTC durations. Register another choice before that cohort begins when local calendar boundaries or other horizons are needed.

| Quantity | Boundary |
|---|---|
| Acceptance deadline | Enrolment plus 30 days |
| Requirement reassessment due | First acceptance plus 30 days |
| Cohort labour end | Enrolment plus 60 days for every task |

A task accepted on day 2 is reassessed on day 32 and contributes labour through day 60. A task accepted on day 40 receives a weighted score of zero for the day-30 deadline and still contributes labour through day 60. The fraction used in the weighted score must come from one observed primary assessment in `[due, due + assessment_tolerance_days]`. Tolerance defaults to zero. Outside-window assessments are retained but do not silently become the target outcome.

For the paper's weighted score `Q`, a task accepted by the deadline contributes its initial reference score multiplied by the fraction of its accepted requirement still satisfied at reassessment. Late acceptance or permanent abandonment gives a score of zero. An absent reassessment for a task accepted by the deadline leaves its score unknown. The reported `retained_output` also follows the protocol's `scope_change_rule`.

The default `exclude_from_output` rule further adjusts the score over the complete observation window. It sets `Q_tilde = 0` if a material scope change occurs by the fixed end of labour observation. Otherwise `Q_tilde = Q`. All applicable labour still counts. A scope change after reassessment can therefore remove credit without changing what was observed at that assessment. The reported score can remain provisional until the full observation window ends.

Read `scope_treatment`, `lifecycle_status` and `scope_change_ids` with the reported outcomes. Under exclusion, `accepted_by_deadline` and restricted time are suppressed as `None`, even when first acceptance is known. `accepted_at` preserves that recorded event. The qualified status denotes the scope rule, not evidence that acceptance was unobserved. For the paper's weighted score for the original requirement, prespecify `baseline_only` and explicitly identify the original `scope_version` in acceptance and follow-up records. Neither rule rescales the frozen score, and the rule must not be changed after inspecting outcomes.

Cohort productivity uses `sum(retained_output) / sum(person_hours)`, including unsuccessful work, with positive total labour required. The sum of weighted scores therefore follows the chosen scope rule. Report both totals, the scope qualification, acceptance by deadline and restricted time alongside the ratio. Do not average task-specific ratios.

When the sum of weighted scores `Q` is known and complete direct labour is bounded by `0 < H_lower <= H <= H_upper`, the additional `retained_output_per_person_hour_bounds` field reports `[Q/H_upper,Q/H_lower]` reference units per person-hour. This is a conservative conditional range and inherits the labour bounds' floating-point limitations. Shared constraints can make it wider than the jointly attainable range. A known zero sum yields `[0,0]` with positive lower labour. Unknown weighted scores, zero lower labour, sampled or incomplete labour, no counted tasks, or numerical overflow leaves the finite range unavailable. The existing point ratio and its status are unchanged. A finite range still leaves the exact ratio unresolved, and causal interpretation requires a separate comparison design.

## Freeze, inspect and interpret

An event collected after a knowledge cut remains outside that cut even when the event itself occurred earlier. Freeze a snapshot after the required observations are available, inspect coverage and unresolved issues, then generate the report. Preserve earlier results when a later observation or correction produces a new snapshot. Back up raw state and snapshots together. Follow the [data contract](data-contract.md) for immutable-record corrections and supported import schemas.

Start by evaluating coverage, feasibility, allocation reliability and follow-up completeness. Current reports do not estimate assignment-weighted policy contrasts or uncertainty from a causal design. Weighted score, labour and elapsed time to acceptance are separate outcomes. Their separate results are needed to assess a staffing or quality claim. Source collection, completed computations and passing software tests establish their own bounded properties rather than validating the observations about human work.
