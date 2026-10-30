# Measurement protocol

This release implements descriptive task accounting and direct-observation labour bounds from the paper's accounting and empirical measurement sections, with the declared scope-policy adjustment described below. A separate `estimate-labour` command implements specified probability-sampling designs. Neither path infers human effort from commits, identifies a causal AI effect, or forecasts staffing. The example records are synthetic.

## Freeze before intake

Create a `protocol` before enrolling tasks. State eligibility, all-role measurement boundaries, whether attended waiting counts, the acceptance rubric, scope-change rule and observation horizons. Use `eligibility_notes`, `role_convention_notes` and `attended_waiting_counts` for these choices. Do not change them after seeing outcomes; start a separately identified protocol/cohort when the design changes.

Register every eligible requirement, including postponed or abandoned work. Freeze its scope and reference score before implementation. A defensible first score is one unit per eligible requirement, reported by task class; it is a task-equivalent index, not calibrated human hours or customer value. Connect PRs, commits, sessions and cross-repository changes with `task_link` records. Split implementation work without resetting the original enrolment clock or counting both parent and child output.

`assigned_policy` and `assignment_unit` retain policy metadata. Actual use belongs in `ai_usage` or linked source evidence. Tool availability, assigned policy, branch names, workflow actors and observed assistance are different observations. EngArg and EAL are coupled components; two repositories and many commits do not supply independent experimental assignments.

## Record human effort

Include specification, prompting, implementation, review, testing, integration, coordination and repair according to the frozen convention. Record people pseudonymously. Unattended agent execution is separate resource use. Two people pairing for one hour supply two person-hours; one person supervising two agents for an hour cannot supply two human hours.

For direct recording, an interval contributes duration in hours times `allocation_fraction`. The fraction describes average allocation over the recorded interval; it does not assert a constant allocation at every instant. Separately observed allocations on each side of the fixed labour boundary identify the within-window contribution. A whole-interval average strictly between zero and one does not: the software keeps `person_hours` unavailable and retains contributions from other identified intervals in `recorded_person_hours`. It additionally reports the justified range for each observed direct interval that crosses the boundary.

For total observed interval mass `m`, duration `u` inside the labour window, and duration `v` outside it, the range is `[max(0, m-v), min(m, u)]` person-hours. The assumption is `0 <= allocation <= 1` at every time, up to sets of measure zero. Filling the outside or inside portion first attains each endpoint using this interval's datum alone. These are sharp marginal bounds; other observations and shared person capacity can tighten them. Literal direct averages of zero and one identify zero or full allocation almost everywhere, so their ranges collapse and the existing point result remains available. No near-zero or near-one tolerance substitutes for these exact endpoints. Calculations retain subsecond duration before ordinary floating-point accounting; they do not implement certified outward rounding. Study records still require effort to start at or after task enrolment.

The optional task `person_hours_bounds` and cohort `total_person_hours_bounds` sum direct interval ranges. They are conservative enclosures, not jointly sharp extrema: overlapping task or role allocations can couple the marginal endpoints. The separate `bound-labour` command solves the joint interval model from explicit constraint inputs. It does not reinterpret these additive study records. See [exact labour identification](labour-identification.md). Finite complete-total bounds require a complete, method-matched direct frame, observed responses, a closed horizon, and completed intersecting observations. Missing, partial, conflicting or sampled frames leave those total bounds unavailable. Individual direct boundary bounds remain visible even when the whole frame is incomplete. Use `coverage` records for the lifecycle and effort frames. Complete direct coverage with no positive effort attests a measured zero; missing coverage does not.

Sampled intervals crossing the boundary remain unsupported under the sampling-frame contract. A complete census needs an actual declared frame and sufficiently complete recording, not simply a few intervals. Deterministic direct bounds are neither confidence intervals nor a replacement for sampling uncertainty.

Supply one allocation observation per task, person and role on each interval cell. Two observations in that same category cannot overlap for a positive duration, even when their interval boundaries differ; touching endpoints are allowed. Different tasks or roles can overlap, subject to the method rules and a conservative screen requiring the sum of overlapping interval averages to be at most one. This is a sufficient input-format restriction, not a diagnosis of actual instantaneous over-allocation: averages over different intervals may fail the screen even when the underlying activity was feasible. Supply a common observed partition to resolve such a rejection; do not invent within-interval allocations.

Complete coverage attestations with the same measurement method may jointly cover an effort interval. Adjacent or overlapping attestations do not split the observation, interpolate its allocation, or change a sampled unit or its inclusion probability. Conflicting partial or missing coverage remains unresolved.

For probability sampling, declare the work frame and randomly selected person-intervals before examining their allocations. A sampled interval contributes:

```text
duration_hours * allocation_fraction / inclusion_probability
```

Retain selected zero allocations as observed zeros. Retain nonresponses as `response_status="missing"`, null allocation and a reason. A sampled frame needs actual sampled units. Crossing intervals need separately observed within-boundary allocations under the declared sampling design; multiplying an interval average by a clipped duration does not supply that observation. Do not mix direct and sampled observations of the same person-time or add diary totals to estimates for that same period.

Task/cohort reports calculate a Horvitz–Thompson point estimate, without design variance or a confidence interval: those effort records do not supply a complete joint sampling design. Such point estimates never become exact population-mass constraints for the direct labour bounds, including in a mixed direct/sampled frame. The separate `estimate-labour` path accepts its own explicit design contract and can report design-specific variance, including finite-population variance under simple random sampling without replacement; it does not silently enrich generic cohort effort records. Sampling weights do not repair missing responses, inaccurate allocations or work outside the frame. Nonresponse, incomplete frames and method mismatches leave complete labour unavailable while showing recorded contributions. Pilot response burden and independently check a subset of allocations before interpreting estimates.

## Record acceptance and retained functionality

Record first acceptance explicitly with its evidence and revision links; merging and deployment are separate events. Record permanent abandonment as an outcome. `lost_to_follow_up` means observation was lost; it is not abandonment. Unaccepted tasks with complete observation through the deadline contribute the full restricted time without acceptance. Immature or lost observations remain unavailable.

This implementation requires an assessment for **every counted accepted requirement**. Assess against the same frozen rubric at acceptance and follow-up, using independent reviewers where feasible. Record `reviewer_id`, `blinded`, disagreements, adjudication and intentional replacement notes. Keep reviewer observations nonprimary and designate one adjudicated `is_primary=true` result. Record inability to blind rather than assuming it succeeded.

Passing tests, surviving lines and no reported incidents do not establish `retained_fraction=1`. Record defects and severity independently of the retained fraction; link repair effort rather than counting it again. Use a common audit/feedback procedure and record audit-triggered repair in the applicable labour boundary. Independent quality subsampling, audit-exposure adjustment and quality sampling weights are outside this release.

## Apply the three timing rules

The synthetic example uses `acceptance_days=30` and `follow_up_days=30`. The paper prescribes no universal values. Days are fixed 24-hour UTC durations. Register another choice before that cohort begins when local calendar boundaries or other horizons are needed.

| Quantity | Boundary |
|---|---|
| Acceptance deadline | Enrolment plus 30 days |
| Retention assessment due | First acceptance plus 30 days |
| Cohort labour end | Enrolment plus 60 days for every task |

A day-2 acceptance is assessed on day 32, but labour remains open until day 60. A day-40 acceptance receives zero output for the day-30 deadline while labour still counts through day 60. Follow-up scores must come from one observed primary assessment in `[due, due + assessment_tolerance_days]`; tolerance defaults to zero. Outside-window assessments are retained but do not silently become the target outcome.

For the paper's frozen-requirement endpoint `Q`, timely acceptance gives frozen score times retained fraction. Late acceptance or permanent abandonment gives zero deadline output. Missing retention for a timely accepted task is unknown. The reported `retained_output` also follows the frozen `scope_change_rule`.

The default `exclude_from_output` defines a different, protocol-credited endpoint over the completed observation window: `Q_tilde = 0` if a material scope change occurs by the fixed labour end; otherwise `Q_tilde = Q`. It retains all applicable labour. Thus a scope change after an already completed retention assessment can remove protocol credit without changing what was observed at that earlier assessment. Before the full window ends, current protocol credit can remain provisional even when the retention assessment is complete.

Read `scope_treatment`, `lifecycle_status` and `scope_change_ids` with the reported endpoints. Under exclusion, `accepted_by_deadline` and restricted time are suppressed as `None`, even when first acceptance is known; `accepted_at` preserves that recorded event. The qualified status denotes the scope rule, not evidence that acceptance was unobserved. For the paper's original-scope endpoint, prespecify `baseline_only` and explicitly identify the original `scope_version` in acceptance and follow-up records. Neither rule rescales the frozen score, and the rule must not be changed after inspecting outcomes.

Cohort productivity uses `sum(retained_output) / sum(person_hours)`, including unsuccessful work, with positive total labour required. Its numerator therefore inherits the chosen scope endpoint. Report both totals, the scope qualification, acceptance by deadline and restricted time alongside the ratio. Do not average task-specific ratios.

When retained output `Q` is known and complete direct labour is bounded by `0 < H_lower <= H <= H_upper`, the additional `retained_output_per_person_hour_bounds` field reports `[Q/H_upper,Q/H_lower]` reference units per person-hour. This is a conservative conditional range and inherits the labour bounds' floating-point limitations; shared constraints can make it wider than the sharp range. Known zero output yields `[0,0]` with positive lower labour. Unknown output, zero lower labour, sampled or incomplete labour, no counted tasks, or numerical overflow leaves the finite range unavailable. The existing point ratio and its status are unchanged; a finite range does not identify a point estimate or a causal productivity effect.

## Freeze, inspect and interpret

Collecting an old event today does not put it into yesterday's knowledge cut. Freeze a snapshot after the required observations are available, inspect coverage and unresolved issues, then generate the report. A later observation or correction produces a new snapshot; preserve earlier results. Back up raw state and snapshots together. Follow the [data contract](data-contract.md) for immutable-record corrections and supported import schemas.

Start by evaluating coverage, feasibility, allocation reliability and follow-up completeness. Current reports do not estimate assignment-weighted policy contrasts or uncertainty from a causal design. Retention, labour and speed remain separate endpoints; a favourable ratio alone does not support a staffing or quality claim. Source collection, completed computations and passing software tests establish their own bounded properties rather than validating the observations about human work.
