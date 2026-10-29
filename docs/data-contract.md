# Data contract

Use [`examples/study.jsonl`](../examples/study.jsonl) as the executable synthetic reference. Import related records in one batch so references can be validated together. JSONL, a JSON record array, a single record, and this versioned envelope are accepted:

```json
{"schema":"observatory.bundle/1","records":[{"id":"source/example","kind":"source_record","data":{"source":"example"}}]}
```

Every record has `id`, `kind` and object `data`. IDs are nonempty, globally unique within a store and immutable: identical ID/content is idempotent; changed content under an existing ID is rejected. Supported kinds are listed below. Unknown kinds are rejected. Typed `data` objects reject unknown fields except `ai_usage` and `source_record`, which retain extension fields.

## Study records

Dates are RFC 3339 timestamps. Calculations use UTC and fixed 24-hour days. Fields marked optional may be omitted or null unless a default is specified. An omitted or null measurement does not mean zero.

| Kind | Required `data` fields | Optional fields and defaults |
|---|---|---|
| `protocol` | `name`, `frozen_at`, `acceptance_days`, `follow_up_days` | `assessment_tolerance_days=0`, `rubric_version="requirements-v1"`, `scope_change_rule="exclude_from_output"`; optional `attended_waiting_counts`, `eligibility_notes`, `role_convention_notes` |
| `task` | `protocol_id`, `requirement`, `task_class`, `scope_version`, `enrolled_at`, `score_frozen_at`, `frozen_score` | `count_output=true`, `repositories=[]`; optional `parent_task_id`, `assigned_policy`, `assignment_unit` |
| `lifecycle` | `task_id`, `event`, `occurred_at` | Optional `note`, `scope_version` |
| `effort` | `task_id`, `person_id`, `role`, `start`, `end` | `allocation_fraction` nullable; `method="direct"`, `response_status="observed"`; optional `inclusion_probability`, `missing_reason` |
| `coverage` | `task_id`, `domain`, `start`, `end`, `status` | `method="direct"`; optional `reason` |
| `assessment` | `task_id`, `phase`, `assessed_at`, `status`, `rubric_version` | `retained_fraction` nullable; `is_primary=true`; optional `revision`, `missing_reason`, `scope_version`, `reviewer_id`, `blinded`, `disagreement_notes`, `adjudication_notes`, `intentional_replacement_notes` |
| `ai_usage` | `purpose`, `observed_at`, `usage_status` | Optional `task_id`, `missing_reason`; provider-specific extension fields |
| `task_link` | `task_id`, `relation`, `observed_at`, `method` | Optional `source_record_id`, `repository`, `pull_request`, `commit`, `session_id`, `ambiguity` |
| `defect` | `task_id`, `detected_at`, `severity`, `evidence_ids` | `repair_effort_ids=[]`; optional `notes` |
| `task_scope_change` | `task_id`, `occurred_at`, `new_scope_version`, `reason` | Optional `previous_scope_version` |
| `correction` | `target_id`, `replacement_data`, `corrected_at`, `reason` | Optional `supersedes_correction_id` |
| `source_record` | Adapter-specific object | Source provenance, payloads and coverage remain evidence, without becoming labour or task output |

Use these exact enum values:

* Lifecycle `event`: `started`, `review`, `accepted`, `deployed`, `abandoned`, `lost_to_follow_up`.
* Effort `method`: `direct`, `sampled`. Observation status: `observed`, `missing`.
* Coverage `domain`: `effort`, `lifecycle`; `status`: `complete`, `partial`, `missing`.
* Assessment `phase`: `acceptance`, `follow_up`.
* AI `purpose`: `development`, `product_execution`, `synthetic`.
* Protocol `scope_change_rule`: `exclude_from_output`, `baseline_only`.

`acceptance_days` must be positive. Scores are finite and nonnegative; observed allocation and retention fractions lie in `[0,1]`. Protocol and score freeze times must not follow enrolment. Task references must exist in the imported graph.

Parent/child tasks share the original `enrolled_at` and `protocol_id`; implementation start belongs in a lifecycle event. A counted child cannot have a counted ancestor. Allocate labour once rather than duplicating it on parent and child. Parent/group records with `count_output=false` retain labour but do not enter the output or acceptance denominator.

There is at most one first-acceptance event per task. Acceptance and permanent abandonment cannot coexist. Deployment requires preceding acceptance. Missing effort/assessment records require a nonempty `missing_reason` and a null fraction. Sampled effort requires `0 < inclusion_probability <= 1`; direct effort has no probability or probability one. Overlapping allocations for a person sum to at most one. Overlapping sampled allocations must describe the same sampled interval and probability; overlapping direct and sampled observations are rejected.

Coverage attests a declared observation frame; it is not inferred from a few effort events. Partial/missing coverage requires `reason`. Lifecycle coverage uses `direct`. A conflicting partial interval is not silently overridden by a complete attestation. See the [measurement protocol](measurement-protocol.md) before asserting completeness.

Task links require a retained `source_record_id`, a `session_id`, or `repository` plus PR/commit. `relation` and `method` describe the linkage; `ambiguity` records uncertainty. Use full commit IDs for reproducible links. Defects require existing nonself evidence IDs. Repair IDs refer to effort on the same task starting no earlier than detection; linked effort is not added to the total a second time.

Scope versions form an ordered, unrepeated chain. `previous_scope_version`, when supplied, must match that chain. The frozen scope-change rule governs interpretation, not an edited original task.

## Corrections and versioning

Retain a new source version under a new ID when an external object changes. To correct a typed study record, append a new `correction` record rather than overwriting its original content. `target_id` identifies the existing study record; `replacement_data` is its **complete replacement data object**, not a partial patch. The target's ID and kind remain unchanged in the effective analysis view. Original records and correction envelopes remain in the evidence store and snapshots.

For example, after importing a missing effort record `study/effort-1`, an observed response can be recorded as:

```json
{"id":"study/effort-1/correction-1","kind":"correction","data":{"target_id":"study/effort-1","replacement_data":{"task_id":"study/task-1","person_id":"participant-01","role":"review","start":"2026-08-02T09:00:00Z","end":"2026-08-02T10:00:00Z","allocation_fraction":0.5,"method":"direct","response_status":"observed"},"corrected_at":"2026-08-04T10:00:00Z","reason":"Participant supplied the previously missing allocation"}}
```

The example depends on the referenced task and original effort record; it is not a standalone import. Corrections can target `protocol`, `task`, `lifecycle`, `effort`, `coverage`, `assessment`, `ai_usage`, `task_link`, `defect` and `task_scope_change`. They cannot target a `source_record`, another correction, or themselves. Correcting a source interpretation requires another source version or a study record linked to the evidence.

Each target has one linear correction chain. The first correction omits `supersedes_correction_id`. Later corrections name the immediately preceding correction and have strictly increasing `corrected_at` values. Branches, cycles, unknown targets and predecessors belonging to another target are rejected. Replacement schemas and the effective study graph are validated together; correcting allocation cannot bypass the person-time limit or other task constraints. There is no record-deletion operation.

Analyses use corrections dated by their `as_of` cut. Snapshot membership additionally limits corrections to those actually observed by that cut: a backdated correction imported today cannot change yesterday's frozen knowledge. Reports list applied IDs, targets, reasons, predecessor links and which correction is current. Preserve earlier snapshots. Correcting an inaccurate record is distinct from changing a scientific protocol: do not relabel a post-outcome choice as a frozen pre-intake decision.

For adjudication, reviewer observations can start as `is_primary=false` with one primary adjudicated result. If an existing primary must be superseded, correct its complete data to set `is_primary=false` and import the new adjudicated record in the same batch. Multiple eligible primaries leave retention unresolved rather than choosing one silently.

## EngArg and EAL imports

EngArg uses a separate manifest; see [`examples/import-manifest.json`](../examples/import-manifest.json). Required fields are `schema="observatory.engarg-import/1"`, `provenance_chain_id`, `session_id`, `attempt_id`, full 40-character `source_revision`, and `artifact_kind` (`report`, `audit`, `checkpoint`). Optional fields are `call_offset=0`, `call_ordinals`, `parent_attempt_id`, `parent_checkpoint_sha256`.

All representations of one execution share chain/session identity. A logical call is identified by chain, session and ordinal. Resumed invocations use distinct attempt IDs; inherited calls reuse their ordinals. Multi-call audits/checkpoints require explicit `call_ordinals`, a unique source-position-to-logical-ordinal mapping. Completion order can differ from a report's dependency order. Do not create identities from filenames or identical responses. Conflicting known representations are rejected; unknown optional metadata may be enriched.

Supported source formats are `engarg.run.v3`, explicitly synthetic `engarg.demo.v2`, `engarg.authoring-checkpoint/1`, and recognised audit events. Native provider metadata includes `cursor`, `augment` and `command`. Configured selection and observed model identity remain separate; null tokens and internal inference counts remain unknown. Imported calls are `product_execution` evidence. Prompts, response text and raw error messages remain in private source artifacts rather than transformed call records.

The native-provider source inspected for this adapter was [`876f720ec1abe951e391e8ff7bfc9a056a5e333b`](https://github.com/emmett08/engarg/tree/876f720ec1abe951e391e8ff7bfc9a056a5e333b), then PR #45; inspected main was `e8b21c929f2aa9e8b870131e534e354926354094`. Those are inspection identities, not a claim about the current branch head. Always record the artifact's actual producing revision. Schema support does not establish that a branch was merged.

EAL accepts the runtime SQLite `records` table (`id`, `kind`, `created_at`, `payload`) through a consistent read-only online backup, or `{"schema":"eal.records-export/1","records":[...]}` with those fields. `--namespace` distinguishes workspaces. EAL run IDs never become development-session IDs. Imports are bounded to supported formats; unknown schemas are rejected.

## Provenance and exports

The store adds `observed_at`, `source` and `raw_sha256` outside the immutable record. These are acquisition provenance, distinct from event timestamps inside `data`, including `ai_usage.observed_at`. Collection manifests retain response metadata and content hashes; generic source records preserve changed external versions and explicit endpoint coverage. Conditional cache revalidation retains the 304 receipt and the original cached body's provenance.

Snapshots use `observatory.snapshot/1`, sorted unique IDs, the observation-time cut, implementation/calculation identities and a content digest. Reports identify both snapshot producer and current analyser. A snapshot hash checks exported records; verifying original evidence also requires the state directory and its linked objects. Reports, CSV and snapshots are private research exports: they can contain requirements, participants and source comments. They are not anonymised publications.
