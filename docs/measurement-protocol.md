# Measurement protocol

This release implements descriptive task accounting from §§3 and 6 of *Human Effort, Delivery Speed and Retained Output in AI-Assisted Software Engineering*. It does not infer human effort from commits, identify a causal AI effect, or forecast staffing. The example records are synthetic.

## Freeze before intake

Create a `protocol` before enrolling tasks. State eligibility, all-role measurement boundaries, whether attended waiting counts, the acceptance rubric, scope-change rule and observation horizons. Use `eligibility_notes`, `role_convention_notes` and `attended_waiting_counts` for these choices. Do not change them after seeing outcomes; start a separately identified protocol/cohort when the design changes.

Register every eligible requirement, including postponed or abandoned work. Freeze its scope and reference score before implementation. A defensible first score is one unit per eligible requirement, reported by task class; it is a task-equivalent index, not calibrated human hours or customer value. Connect PRs, commits, sessions and cross-repository changes with `task_link` records. Split implementation work without resetting the original enrolment clock or counting both parent and child output.

`assigned_policy` and `assignment_unit` retain policy metadata. Actual use belongs in `ai_usage` or linked source evidence. Tool availability, assigned policy, branch names, workflow actors and observed assistance are different observations. EngArg and EAL are coupled components; two repositories and many commits do not supply independent experimental assignments.

## Record human effort

Include specification, prompting, implementation, review, testing, integration, coordination and repair according to the frozen convention. Record people pseudonymously. Unattended agent execution is separate resource use. Two people pairing for one hour supply two person-hours; one person supervising two agents for an hour cannot supply two human hours.

For direct recording, an interval contributes duration in hours times `allocation_fraction`. The fraction describes average allocation over the recorded interval; it does not assert a constant allocation at every instant. Split observations at the fixed enrolment labour boundary and record each part's observed allocation separately. A direct whole-interval average strictly between zero and one does not identify a partial contribution: the software withholds complete labour, retains contributions from other identified intervals, and reports the unresolved interval. Literal direct averages of zero and one are exceptions: the attention bound identifies zero or full allocation almost everywhere, so their clipped contribution is known. No near-zero or near-one tolerance substitutes for these exact endpoints. Sampled intervals crossing the boundary remain unsupported under the sampling-frame contract. A complete census needs an actual declared frame and sufficiently complete recording, not simply a few intervals. Use `coverage` records for the lifecycle and effort frames. Complete direct coverage with no positive effort attests a measured zero; missing coverage leaves the total unknown.

For probability sampling, declare the work frame and randomly selected person-intervals before examining their allocations. A sampled interval contributes:

```text
duration_hours * allocation_fraction / inclusion_probability
```

Retain selected zero allocations as observed zeros. Retain nonresponses as `response_status="missing"`, null allocation and a reason. A sampled frame needs actual sampled units. Crossing intervals need separately observed within-boundary allocations under the declared sampling design; multiplying an interval average by a clipped duration does not supply that observation. Do not mix direct and sampled observations of the same person-time or add diary totals to estimates for that same period.

The software calculates a Horvitz–Thompson point estimate, not design variance or a confidence interval: joint inclusion probabilities/design information are not implemented. Sampling weights do not repair missing responses, inaccurate allocations or work outside the frame. Nonresponse, incomplete frames and method mismatches leave complete labour unavailable while showing recorded contributions. Pilot response burden and independently check a subset of allocations before interpreting estimates.

## Record acceptance and retained functionality

Record first acceptance explicitly with its evidence and revision links; merging and deployment are separate events. Record permanent abandonment as an outcome. `lost_to_follow_up` means observation was lost; it is not abandonment. Unaccepted tasks with complete observation through the deadline contribute the full restricted time without acceptance. Immature or lost observations remain unavailable.

This implementation requires an assessment for **every counted accepted requirement**. Assess against the same frozen rubric at acceptance and follow-up, using independent reviewers where feasible. Record `reviewer_id`, `blinded`, disagreements, adjudication and intentional replacement notes. Keep reviewer observations nonprimary and designate one adjudicated `is_primary=true` result. Record inability to blind rather than assuming it succeeded.

Passing tests, surviving lines and no reported incidents do not establish `retained_fraction=1`. Record defects and severity independently of the retained fraction; link repair effort rather than counting it again. Use a common audit/feedback procedure and record audit-triggered repair in the applicable labour boundary. Independent quality subsampling, audit-exposure adjustment and quality sampling weights are outside this release.

## Apply the two clocks

The suggested pilot uses `acceptance_days=30` and `follow_up_days=30`. Days are fixed 24-hour UTC durations. Register another choice before that cohort begins when local calendar boundaries or other horizons are needed.

| Quantity | Boundary |
|---|---|
| Acceptance deadline | Enrolment plus 30 days |
| Retention assessment due | First acceptance plus 30 days |
| Cohort labour end | Enrolment plus 60 days for every task |

A day-2 acceptance is assessed on day 32, but labour remains open until day 60. A day-40 acceptance receives zero output for the day-30 deadline while labour still counts through day 60. Follow-up scores must come from one observed primary assessment in `[due, due + assessment_tolerance_days]`; tolerance defaults to zero. Outside-window assessments are retained but do not silently become the target outcome.

For timely acceptance, retained output is frozen score times retained fraction. Late acceptance or permanent abandonment gives zero deadline output. Missing retention for a timely accepted task is unknown. Cohort productivity is `sum(retained_output) / sum(person_hours)`, including unsuccessful work, with positive total labour required. Report both totals, acceptance by deadline and restricted time alongside the ratio. Do not average task-specific ratios.

Material scope changes follow `scope_change_rule`. The default `exclude_from_output` assigns zero output while retaining labour and leaves original-scope acceptance unresolved. `baseline_only` requires acceptance and follow-up explicitly identifying the original `scope_version`. Neither rule rescales the frozen task score retrospectively.

## Freeze, inspect and interpret

Collecting an old event today does not put it into yesterday's knowledge cut. Freeze a snapshot after the required observations are available, inspect coverage and unresolved issues, then generate the report. A later observation or correction produces a new snapshot; preserve earlier results. Back up raw state and snapshots together. Follow the [data contract](data-contract.md) for immutable-record corrections and supported import schemas.

Start by evaluating coverage, feasibility, allocation reliability and follow-up completeness. Current reports do not estimate assignment-weighted policy contrasts or uncertainty from a causal design. Retention, labour and speed remain separate endpoints; a favourable ratio alone does not support a staffing or quality claim. Source collection, completed computations and passing software tests establish their own bounded properties rather than validating the observations about human work.
