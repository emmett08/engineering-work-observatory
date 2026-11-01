# From activity to evidence — speaker notes

Audience: engineers without advanced mathematics. Suggested delivery: 45–60 minutes plus discussion. Slides 1–32 form the main walkthrough; later slides are reference material.

SLIDE 1: From activity to evidence

Open with a practical question: when a pipeline completes, what do we actually know about the engineering work? This tutorial connects PR #9's instrumentation to the repository's measurement and forecasting components. They answer different questions and must retain distinct evidence contracts. No calculus, probability course or optimisation background is assumed. Use the diagrams first; formulas and fuller assumptions are in the appendix. Suggested duration is 45–60 minutes with the two short audience exercises.

Evidence status: Model definition.

Sources
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 2: A tool ran for 20 minutes. How much human work?

Ask the audience for an answer before revealing the three possibilities. The same tool runtime is compatible with unattended execution, active supervision or several people working together. The numbers are illustrative possibilities, not measurements of the pilot. Even zero attended minutes during execution would not establish zero labour for the whole requirement: specification, prompting, review and repair may occur outside that interval. Attended waiting is handled by a prespecified protocol convention.

Evidence status: Illustrative thought experiment.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 3: Three questions need three kinds of evidence

Explain the architecture as three related contracts, not one all-purpose productivity score. Execution evidence establishes observed tool events and resources. The study-accounting path needs direct or sampled human allocations, acceptance and retention observations. The automatic forecasting path uses complete historical PR-count bins. PR #9 strengthens execution evidence, and its records can be linked to requirements. It does not supply the missing measurements needed by the other paths.

Evidence status: Model definition.

Sources
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[F] Automatic forecasting contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/automatic-forecasting.md
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 4: Follow one requirement through many activities

Use this fictional requirement throughout the explanation. A single requirement can span several PRs, repositories, assistant sessions and test runs. A failed attempt still consumed resources and may have used human effort. PRs and tool calls therefore act as evidence about the requirement, rather than replacing the accounting unit. The frozen requirement and its score must precede implementation. The example does not assert any actual customer task from the public pilot.

Evidence status: Illustrative engineering example.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md

---

SLIDE 5: Keep the source, the interpretation and the cut

Walk left to right. Raw source bytes are retained by content hash. Validated records add explicit identity, observations and study meaning. A snapshot freezes the evidence available at a particular cut, so later observations or corrections do not rewrite an old result. Event time says when the work happened; observation time says when the Observatory learned about it. An old event collected today does not belong to yesterday's knowledge cut. Hash verification protects retained bytes; it does not establish that the original observation was true.

Evidence status: Model definition.

Sources
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md

---

SLIDE 6: The GitHub account is not the controller

This is the central provenance distinction in PR #9. GitHub actor records the account associated with the original run; triggering_actor records the account that initiated the current run attempt, which can differ on a rerun. Neither field tells us whether a human, assistant or automation actually controlled the work. Controller and session provenance are caller declarations and remain labelled as such. Even runner-environment values inside an uploaded bundle are assertions until independently compared with the API.

Evidence status: Model definition.

Sources
[G] GitHub actor and triggering-actor contexts: https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#github-context
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 7: The same account can hide two different stories

Both rows are hypothetical. In the first, an assistant uses Alice's credentials to run an ordinary test tool. Alice remains the GitHub actor, while assistant is a declaration supported by session provenance. In the second, Alice directly launches a workflow that uses Codex; human is the declared controller, while the tool is an assistant tool. A workflow containing AI does not by itself establish an assistant controller, and an assistant controller need not run an AI tool in every invocation. If evidence is missing, the controller stays unknown.

Evidence status: Illustrative provenance scenarios.

Sources
[G] GitHub actor and triggering-actor contexts: https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#github-context
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 8: Identify an execution at the right level

Explain the nested platform context first: stable repository ID, workflow revision, run ID, run attempt, job and matrix identity. A numeric job ID differs from the workflow job key. In this action, expanded job name, optional numeric ID and matrix metadata are caller supplied and marked job_context_source=caller. Within that context, each actual tool invocation receives a fresh UUID. Start and completion share that invocation ID. A logical-call link can connect retries, but does not collapse them into one physical execution.

Evidence status: Model definition.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md
[R] Receipt schema: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/schemas/tool-receipt-v1.schema.json

---

SLIDE 9: Write a start before the tool can fail

The start record is written before launch. Completion records observed exit status and elapsed runtime after the process finishes. These are separate immutable files, not updates to one evolving row. The example shows a failed attempt followed by a new invocation for a retry. Both can retain the same logical-call ID; a declared parent link connects them. A workflow rerun changes the run attempt as well. Cancellation handling is best effort: runner loss can prevent completion and publication, so the independent job record remains essential.

Evidence status: Illustrative execution timeline.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 10: A wrapper observes only the boundary you wrap

The ordinary adapter launches an explicit command without adding its argv, stdout or unrestricted environment to receipts. Ordinary command output may still appear in normal Actions logs. The wrapper observes the process boundary, not every internal operation. The Codex adapter parses a small allowlist of documented JSON events, not the transcript. Separate jobs need their own setup and publication. Work outside Actions needs its own adapter or observation. There is no magic collector step that sees all tools, all jobs and all work. The coloured region is the declared instrumentation boundary, not an assertion of total coverage.

Evidence status: Model definition.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 11: Three components cooperate

The action configures recording, captures explicit tool receipts and publishes a separate immutable bundle for each instrumented job. A durable collector downloads designated bundles, validates schema and hashes, deduplicates repeated deliveries and imports records. Independent reconciliation retrieves GitHub runs, attempts and jobs, including unsuccessful conclusions. This comparison is implemented within the collector command; independent refers to its GitHub metadata source, not a mandatory separate service. Parallel runners do not share an evolving SQLite database. The workflow_run completion observer can run regardless of the preceding conclusion, but still requires its own successful acquisition and durable retention.

Evidence status: Model definition.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md
[W] GitHub workflow_run completion event: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run

---

SLIDE 12: A missing receipt is an observation gap

Read across the table rather than treating receipt count as tool count. Start plus completion supports a recorded process outcome; it does not prove complete job instrumentation. Start alone leaves completion unknown. Completion alone preserves the reported result while identifying a missing start record. A valid empty bundle reports setup context with zero receipt entries. Schema validity alone does not prove that recording was configured or that no tools ran. No bundle can reflect uninstrumented work or lost delivery. Independent failed, cancelled and skipped job records are retained in every case. A skipped platform job is not manufactured into a tool invocation.

Evidence status: Model definition.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 13: A receipt answers six practical questions

This is a human-readable map of the v1 receipt, not literal JSON. Repository/run/job context and caller declarations remain explicitly distinguishable. Tool version and configured model are supplied labels unless separately observed. Tool-reported usage remains a report rather than an invoice. Requirements and sessions are links; they do not certify acceptance or attention. Designated input and output files contribute hashes and sizes, without copying contents or paths into the receipt. The full field dictionary is in the appendix and schema.

Evidence status: Model definition.

Sources
[R] Receipt schema: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/schemas/tool-receipt-v1.schema.json
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 14: Configured, reported and unknown stay separate

The Codex adapter invokes codex exec --json and recognises thread.started and turn.completed events. In the verified event definitions, these report a thread ID and token usage, but no model identity. Therefore the observed model remains null even if the caller selected a model. The tool-reported session ID is separate from the declared research session. Token usage is not automatically complete billed usage or invoiced cost. Both initial adapters leave charge_amount and charge_currency null; they neither read invoices nor estimate charges. Product execution is also different from development assistance.

Evidence status: Model definition.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md
[R] Receipt schema: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/schemas/tool-receipt-v1.schema.json

---

SLIDE 15: A fingerprint checks bytes. Evidence checks claims.

A SHA-256 hash is a content fingerprint: change the bytes and the fingerprint changes with overwhelming probability. It does not identify who truthfully observed the event. Schema validation checks permitted structure and values; it cannot certify the truth of those values. Independent API comparison can corroborate supported repository, run, attempt, actor and job fields, while caller declarations remain declarations. During collection, conflicting stable receipt identities are quarantined rather than overwritten, while independent GitHub evidence and raw acquisition records remain retained. Direct import rejects conflicting receipts.

Evidence status: Model definition.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md

---

SLIDE 16: Now link the execution to a requirement

Change the question from what ran to what engineering work and retained output occurred. The requirement is the accounting unit. Link its executions, PRs, sessions and revisions with explicit provenance. Human effort still needs an observation method; acceptance needs an explicit event and evidence; retained functionality needs an assessment against the frozen rubric. A passing CI receipt is evidence that a particular check passed, not that the requirement was accepted or retained in full. None of these study records is automatically created from a workflow duration.

Evidence status: Model definition.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 17: Three clocks answer different questions

The illustrative values need not move together. Elapsed acceptance time measures from enrolment to first acceptance, with the deadline and observation rules. Person-hours sum human contributions across people and included roles. Tool runtime is process elapsed time. A tool may run while nobody attends it, while a person works on something else, or while multiple people collaborate. Do not subtract machine time from elapsed time and label the remainder human labour. The protocol explicitly decides whether attended waiting counts.

Evidence status: Illustrative units.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md

---

SLIDE 18: One person has one hour of capacity per hour

Pairing consumes time from two people, so two people pairing for one hour contribute two person-hours. One person supervising two agents for one elapsed hour has at most one person-hour to allocate across the included tasks. This is an attention-capacity assumption, not a way to infer the fractions from agent activity. All included roles—specification, prompting, implementation, review, testing, coordination and repair—need the frozen accounting convention. Fractions describe interval-average allocation, not necessarily constant attention at every instant.

Evidence status: Illustrative capacity arithmetic.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[L] Exact labour identification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/labour-identification.md

---

SLIDE 19: Freeze the deadlines before observing outcomes

This slide uses the repository's synthetic 30-day acceptance and 30-day retention-follow-up choices. They are protocol parameters, not universal defaults for every study. An acceptance on day 2 is assessed for retention on day 32; cohort labour continues through day 60. Acceptance on day 40 misses the day-30 output deadline, but its applicable labour still counts through day 60. Recording acceptance, assessing retention and closing the labour window are separate operations. Late or abandoned work must not disappear from the denominator.

Evidence status: Illustrative protocol: 30 + 30 days.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md

---

SLIDE 20: Count retained requirement credit, then divide

These are illustrative direct observations, not the repository's mixed-method synthetic demo and not the pilot's empirical results. Assume three eligible tasks, complete direct labour observation, closed horizons, no material scope changes and a frozen score of one per task. Task A is accepted on time with retained fraction 0.75; B is permanently abandoned; C is accepted late. Their output totals 0.75 reference units and their labour totals five person-hours. The cohort ratio is 0.15 task-equivalent units per person-hour. A retained fraction is an assessment against the rubric, not a probability or a calibrated customer-value score. Use the ratio of totals, not an average of task ratios.

Evidence status: Illustrative complete direct observations.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md

---

SLIDE 21: Zero, unknown and pending are different states

A zero requires the relevant observation and rule: for example, permanent abandonment establishes zero deadline output, whereas missing follow-up for a timely accepted task leaves retained output unknown. Pending means the required horizon has not yet arrived. A complete direct frame can attest zero labour; missing coverage cannot. The default scope-change policy can remove protocol output credit after a material change by the fixed labour end, while preserving the observed acceptance event and applicable labour. The alternate baseline-only rule follows original scope explicitly. These endpoint rules must be fixed before outcomes are inspected.

Evidence status: Model definition.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md

---

SLIDE 22: An interval total may leave its timing unknown

Here is the smallest useful bound example. We know one person devoted exactly 40 minutes to the target work somewhere in a 60-minute interval. We ask how much occurred in the first 30 minutes, without assuming attention was spread uniformly. At most 30 minutes can fit outside the target, so at least 10 must be inside. At most 30 can fit inside, so the upper endpoint is 30. Both endpoint schedules are possible under allocation between zero and one, so the range is sharp for this datum alone. Multiplying 40 by one half to get 20 would add an unsupported within-interval assumption.

Evidence status: Illustrative exact interval observation.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[L] Exact labour identification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/labour-identification.md

---

SLIDE 23: Shared capacity can tighten the answer

Use the existing exact joint-labour example. One person spends exactly one hour on task A and one hour on task B within the same two-hour interval. Each task alone could contribute between zero and one hour to the first hour, so summing the marginal ranges would give zero to two. That sum ignores shared capacity. Two hours of total work exhaust the person's two-hour capacity; neither hour can contain more than one person-hour, so each hour must contain exactly one combined person-hour. The coloured schedules are two compatible witnesses, not reconstructed actual histories. The solver splits time at every boundary, constrains cell totals by observations and shared capacity, then minimises and maximises the target. Incompatible constraints yield an inconsistent result, with no bounds. It returns cell allocations and rechecks constraints with exact rational arithmetic.

Evidence status: Repository synthetic example, verified.

Sources
[L] Exact labour identification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/labour-identification.md
[J] Joint-capacity example input: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/examples/joint-labour-bounds.json

---

SLIDE 24: Use a random sample to correct frozen predictions

This is a manufactured four-unit example, in minutes. The four units are nonoverlapping 30-minute person-intervals. Each has a frozen prediction of ten minutes of included labour, giving forty minutes across the frame. Select two of four units by simple random sampling without replacement, so each has inclusion probability one half. Each sampled error therefore represents two units on average; this is why the correction is multiplied by two. Suppose the observed contributions are fourteen and eight minutes. Their prediction errors are plus four and minus two. Expand each error by two: plus eight minus four. Add the net correction of four to the prediction total, giving a difference estimate of forty-four minutes. It is a frame-total estimate, not a known true total. The frame, selection and accurate labels are substantive assumptions. The next slide explains the uncertainty.

Evidence status: Illustrative probability sample.

Sources
[E] Model-assisted estimation: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/design-based-estimation.md

---

SLIDE 25: A bound and a sampling estimate answer different questions

A deterministic bound lists values compatible with declared observations and capacity assumptions. It is not a confidence interval. A sampling estimate uses a genuine selection design to estimate a finite-frame total, with sampling uncertainty where the design supports a variance calculation. In the four-unit example, residuals are four and minus two minutes; their sample variance is eighteen square minutes. The supported simple-random-sampling formula gives estimated total variance seventy-two square minutes, or standard error about 8.49 minutes. Standard error estimates the sample-to-sample variation in the total estimate. It is not an estimate of the remaining physical capacity, and this is not a ready-made 95% interval. General designs in the implementation do not receive an invented variance. No method repairs missing responses, inaccurate labels or work outside the declared frame.

Evidence status: Model definition.

Sources
[L] Exact labour identification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/labour-identification.md
[E] Model-assisted estimation: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/design-based-estimation.md

---

SLIDE 26: A separate model forecasts merged-PR counts

Return to the third question from the opening map. This is a retained observed example, not a new live forecast. On 30 October 2026, complete daily bins from 24–29 October were 17, 11, 13, 12, 8 and 5 combined merges across EngArg and EAL. The expanding empirical method assigns one sixth probability to each distinct observed count. Its inverse-CDF median is eleven, not the conventional midpoint median of 11.5. Its mean is also eleven. Partial days are excluded, complete zero bins must be supported, and unknown periods are not filled with zeros. Forecasting has no requirement for task labels or labour diaries because it targets PR counts, not labour or functionality. Using these past days as a reference assumes they remain relevant to the next day: launch bursts, policy changes and seasonality can break that assumption.

Evidence status: Observed public history: 24–29 October 2026.

Sources
[F] Automatic forecasting contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/automatic-forecasting.md
[D] Retained public forecast case, 30 October 2026: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/use-cases/engarg-eal-daily-2026-10-30/README.md

---

SLIDE 27: Preserve the awkward results

The retained forecast case reports adverse checks, and the presentation must keep them. The empirical tenth-to-ninetieth percentile endpoints are five and seventeen, but that inclusive range contains all training mass because the distribution is discrete and small. It does not establish eighty-percent future coverage. At the two retrospective origins, the ranges missed both outcomes; mean absolute error of the medians was 5.5 PRs. Mean ranked probability score was 4.83125 versus 3.5 for the previous-day point comparator, with lower being better. Two retrospective origins cannot establish skill or justify promoting a comparator. There were zero complete Monday-based weeks, so weekly forecasting was unavailable, with four required.

Evidence status: Retained public case; retrospective checks.

Sources
[D] Retained public forecast case, 30 October 2026: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/use-cases/engarg-eal-daily-2026-10-30/README.md
[F] Automatic forecasting contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/automatic-forecasting.md

---

SLIDE 28: More evidence does not automatically identify an AI effect

Execution receipts, descriptive labour accounting and forecasting each supply bounded evidence. A causal question asks what would have happened under another policy or treatment. That requires a comparison or assignment design, reliable outcomes and assumptions about dependencies and interference. EngArg and EAL are coupled components; two repositories do not create independent experimental units. A different GitHub actor or a workflow containing AI is not a treatment assignment. A successful receipt pipeline validates the instrumentation path, not an AI productivity effect or staffing rule.

Evidence status: Model definition.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 29: The public pilot verified the evidence path

These are observed verification results retained by PR #9. The pilot replayed existing pinned EngArg and EAL checks inside Observatory's own workflow; it did not instrument the original repositories' workflows. The direct collection run produced four lifecycle receipts for two invocations, each matched to its corresponding completed matrix job. Repeating collection left receipt identity and content unchanged. Twenty-nine retained records include API and coverage records; that is not twenty-nine tool calls. The downloaded store and snapshot assertions replayed locally. The live pilot used ordinary checks, while Codex validation used fixtures and verified upstream event definitions. Neither pilot test counts nor runtimes measure engineering productivity.

Evidence status: Observed live pilot: run 41329655162.

Sources
[T] Public execution pilot and retained verification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-pilot.md
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 30: Adopt one explicit boundary at a time

Start with a selected tool command and a named job, not an ambition to capture everything implicitly. Configure the action in each instrumented job, use the explicit wrapper and publish under an always condition. Establish scheduled durable collection before artifact retention expires. Add an independent workflow_run observer from trusted code for completion metadata. Verify schema failures, missing uploads, retries, matrix identity and reruns before interpreting a larger rollout. Human work and quality observations belong to their own protocol, linked through stable task and session identifiers.

Evidence status: Model definition.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md
[W] GitHub workflow_run completion event: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run

---

SLIDE 31: Which claim can we defend?

Pause and invite a reason for each answer. A: yes, within scope, because an observed successful completion and independently matched job support the wrapped-process claim; they do not establish total tool coverage. B: no, because tool runtime does not measure human attention, and a saved-work comparison also needs a valid counterfactual. C: no, because unknown reported usage is not a measured zero. D: only conditionally: the joint bound is exact under the declared observations, streams and per-person capacity assumptions; those inputs still need evidence. Use the answers to return to the opening three-question map.

Evidence status: Audience exercise; answers in speaker notes.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[L] Exact labour identification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/labour-identification.md

---

SLIDE 32: Make the evidence strong enough for the claim

Close by returning to the practical engineering discipline rather than promising a universal score. Capture what the system actually observes. Keep identity, provenance and missingness explicit. Choose the measurement model for the question: tool events, human labour and retained output, or future PR counts. Check its assumptions and preserve adverse evidence. The benefit is a defensible chain from evidence to a bounded conclusion that another engineer can inspect and replay. Point readers to the appendix for equations, field names and source contracts.

Evidence status: Model definition.

Sources
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[F] Automatic forecasting contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/automatic-forecasting.md

---

SLIDE 33: Receipt fields and their provenance

This reference slide names the concrete schema groups. Platform values are copied from the runner environment, while job name, optional ID and matrix are caller supplied and separately labelled. Independent API comparison corroborates only supported fields; it does not verify every platform value, such as workflow_sha. Declared controller, protocol and session fields are not inferred from GitHub. Tool labels, purpose and configuration are declarations. Result values describe wrapper observations and recognised tool reports, with missing fields retained. Hashes describe designated artifact bytes, not their semantic quality. See the schema for types, enum values, constraints and nullable fields.

Evidence status: Model definition.

Sources
[R] Receipt schema: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/schemas/tool-receipt-v1.schema.json
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 34: The implementation rejects ambiguous substitutions

This is an operational reference for reviewers and implementers. The JSON or ZIP bundle is bounded at sixteen MiB and contains at most ten thousand receipt entries, each at most 256 KiB. The ZIP must contain exactly one bundle.json file; directories and symlinks are rejected. Unknown schema fields and duplicate JSON keys are rejected; hashes are checked over exact receipt JSON bytes. Same-phase content changes under a stable identity fail. Phase-invariant platform, declared and tool identity must agree, and completion cannot precede start. Incomplete phases remain admissible evidence. During collection, cross-delivery conflicts are quarantined without erasing independently retrieved job metadata. Direct import rejects conflicting receipts. Numeric limits bound the implementation; they are not completeness claims.

Evidence status: Model definition.

Sources
[R] Receipt schema: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/schemas/tool-receipt-v1.schema.json
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md

---

SLIDE 35: The small set of equations behind the pictures

Translate each expression aloud. Allocation times duration gives one person's contribution when the required within-window allocation is observed. Retained credit follows the frozen score, timely-acceptance rule and observed retained fraction, plus the prespecified scope policy. Under the default exclusion rule, a material scope change can set protocol credit to zero. Cohort productivity divides summed output by summed labour, with positive labour required. The interval bound moves known mass outside or inside the target as far as capacity allows. These are conditional accounting or identification statements, not equations that infer unobserved people from repository activity.

Evidence status: Model definition.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[L] Exact labour identification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/labour-identification.md

---

SLIDE 36: Correct prediction errors with the actual sampling design

Here M is the number of units in a complete declared frame; n is the sample size for simple random sampling without replacement. For each unit, f is a prediction frozen before evaluation sampling, y is accurately observed labour, and pi is its genuine inclusion probability. The difference estimate sums all predictions and adds weighted observed residuals. Under the stated design assumptions it is design-unbiased across repeated samples. That does not make a realised estimate exact or force it into the physical range. The repository's v2 synthetic example gives 5.5 hours despite five hours of declared frame capacity; clipping changes the estimator. The command preserves signed results and reports variance only when supported. It constructs neither ratio confidence intervals nor causal effects.

Evidence status: Model definition.

Sources
[E] Model-assisted estimation: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/design-based-estimation.md

---

SLIDE 37: The forecast is a reproducible reference, not a capacity model

The automatic model forms complete UTC bins for the chosen horizon and portfolio, summing repositories before fitting. At least four complete target-width bins are required. The expanding empirical method uses all supported complete bins and reports discrete empirical quantiles. Its relevance assumption is that past complete periods provide a useful reference for the next period; launch bursts, policy changes or seasonality may violate it. Forecast bundles are immutable and bound to an evidence snapshot. A later successful collection must cover the entire target period before an original forecast can be scored; merely taking a newer snapshot of stale sources does not extend observation. Retrospective reconstructions are not original issuance receipts. Queue counts describe current visible inventory, not causal bottlenecks or a maximum future merge count. Preserve the adverse two-origin results rather than selecting only a flattering statistic.

Evidence status: Model definition.

Sources
[F] Automatic forecasting contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/automatic-forecasting.md
[D] Retained public forecast case, 30 October 2026: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/use-cases/engarg-eal-daily-2026-10-30/README.md

---

SLIDE 38: Sources for execution evidence

Each title on this slide is a hyperlink. Repository sources are pinned to the implementation revision recorded in the deck manifest. The external GitHub documentation describes platform semantics and may evolve. All illustrative examples are explicitly labelled; observed pilot and forecast examples retain their source scope and dates. The editable PowerPoint, PDF handout, full speaker notes and build source are stored together in the PR.

Evidence status: Model definition.

Sources
[X] Execution-evidence contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-evidence.md
[R] Receipt schema: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/schemas/tool-receipt-v1.schema.json
[T] Public execution pilot and retained verification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/execution-pilot.md
[G] GitHub actor and triggering-actor contexts: https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#github-context
[W] GitHub workflow_run completion event: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run
[A] Architecture and evidence cuts: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/architecture.md

---

SLIDE 39: Sources for measurement and forecasting

Each title on this slide is a hyperlink. Repository sources are pinned to the implementation revision recorded in the deck manifest. The external GitHub documentation describes platform semantics and may evolve. All illustrative examples are explicitly labelled; observed pilot and forecast examples retain their source scope and dates. The editable PowerPoint, PDF handout, full speaker notes and build source are stored together in the PR.

Evidence status: Model definition.

Sources
[P] Measurement protocol: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/measurement-protocol.md
[L] Exact labour identification: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/labour-identification.md
[J] Joint-capacity example input: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/examples/joint-labour-bounds.json
[E] Model-assisted estimation: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/design-based-estimation.md
[F] Automatic forecasting contract: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/automatic-forecasting.md
[D] Retained public forecast case, 30 October 2026: https://github.com/emmett08/engineering-work-observatory/blob/e252eae943a2cc5de27c2db9f60ed9c73a790536/docs/use-cases/engarg-eal-daily-2026-10-30/README.md