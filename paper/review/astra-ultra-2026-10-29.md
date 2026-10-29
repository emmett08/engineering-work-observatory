# Astra review and three editorial passes

Baseline: `e63e023b94ab67db53defc5c79ea34d22c72b4b9`, the first revision in
PR #2. Six separate live workers were configured as `gpt-6-astra` with `ultra`
reasoning effort. They reviewed independently before exchanging challenges in
three pairs. The coordinator adjudicated the findings and checked the repairs.
The native runtime does not expose monetary accounting here.

## Debate and decisions

| Pair | Challenge resolved | Decision |
| --- | --- | --- |
| Mathematics ↔ statistics | Whether known-error convolution can identify latent accuracy, and whether later scope changes contradict the paper's earlier retention endpoint | Keep the identifying-model qualification. Define the application's predeclared scope penalty as a distinct endpoint rather than changing its numerical policy silently. |
| Representation ↔ implementation | Interval averages versus instantaneous capacity, repeated observations of one allocation, and fragmented coverage | Retain the documented conservative overlap screen with an accurate diagnostic. Reject overlapping observations of the same task/person/role. Compose complete same-method coverage without changing sampling units or inclusion probabilities. |
| Evidence ↔ editor | Which repeated qualifications can be consolidated and which need to remain local | Keep the eventual-acceptance bridge local to retry cycles. Name the accompanying application explicitly. Preserve local synthetic-workload, timing, book-attribution, censoring, and formal-coverage conditions. |

All three pairs exchanged actual addressed messages and recorded the peer
findings they consumed. The main propositions survived the reviewed challenges
under their stated assumptions. Shared sources and model families can still
produce correlated errors. Agreement supplies review, not empirical replication
or new machine-checked proofs.

### Implemented repairs

- A complete effort frame can now be formed from adjoining same-method
  attestations. An administrative coverage split no longer makes an unchanged
  labour observation unavailable. Gaps, wrong-method bridges and conflicting
  partial/missing attestations retain their previous force.
- Overlapping observations of one task/person/role are rejected. The schema
  represents one allocation for that combination, rather than distinct additive
  work items. Other concurrent categories remain subject to the declared
  conservative interval-average screen.
- The successor summary path validates every timing row before arithmetic and
  replaces generated files atomically. Existing symbolic or hard links no
  longer cause truncation of another file's contents.
- The application's default scope exclusion is documented as protocol credit
  over the full labour window. Its qualified status fields are distinguished
  from the recorded acceptance event and the paper's earlier retention target.
  Regression tests retain explicit original-scope acceptance and assessment
  evidence while exercising that existing policy.
- For two predictions frozen before the same labour sample, the paper now states
  that sampling variance cancels from their expected squared-loss difference.
  The absolute-accuracy and dependence-aware uncertainty qualifications remain.
- The test HTTP server explicitly restores blocking reads on accepted sockets.
  This removes platform-dependent inheritance of the nonblocking listener's
  mode. The earlier macOS failure was consistent with routing an incomplete
  metadata request. Local collector tests pass, with macOS CI providing the
  platform confirmation.

### Findings not expanded into new machinery

The capacity screen is sufficient for the supported record format rather than
a complete feasibility test for arbitrary interval averages. An EDF solver and
stronger capacity-based bin bounds were deferred. They would broaden the model
and require their own validation. The task-granularity clarification retains the
acceptance unit and assessment clock as parts of the estimand. Additive scores
alone do not guarantee invariance when components are regrouped.

No new mandatory citation correction was found. The supplied Van Der Post
metadata remains further reading, and the mutable author-manuscript URL remains
a source-version limitation rather than a new result. Original source ownership,
the targeted Standard Coder corrections and historical Lean coverage are retained.

## Three applications of `write-for-experienced-engineers`

Each pass applied the skill to all four canonical article inputs, using the
previous pass as its baseline. Each included a whole-paper claim/function map,
consolidation and ordering checks, a recursive prose review, and a comparison of
conditions, quantifiers, modality and citation scope. The emphases below describe
the changes, rather than replacing the complete workflow with three partial steps.

| Pass | Main effect | Revision |
| --- | --- | --- |
| 1 | Reordered study analysis around assignment, measurement validation and the decision. Front-loaded the retry boundary, consolidated equivalent runtime qualifications, clarified scenario labels, and removed the forced appendix break. | `163a8cf` |
| 2 | Connected the definitions to their observation procedures and decisions, explained relationships hidden in inventories, and removed narrative semicolons outside protected formal material. | `5a990ed` |
| 3 | Standalone clarity and fidelity review. Narrowed the simulation claim to this paper's calculations, made the record referent explicit, and distinguished relative squared-loss comparison from absolute accuracy. | `a585089` |

Mechanical comparisons preserve the normalised contents of propositions,
corollaries, examples, proofs, algorithms and displayed mathematics, together
with labels, citation-key occurrences and reference occurrences. The new
squared-loss identity was added before these editorial baselines were frozen.
These bounded checks support the separate semantic review, rather than proving
the correctness of every rewritten sentence. Readability and engagement remain
editorial judgements until assessed with representative readers.

All three pass guards and builds passed. Citation calls, labels and reference
occurrences remained unchanged. The final paper has 22 pages and no unresolved
references or overfull boxes. Its extracted text matches the editor's inspected
preview. Appendix spacing is natural after the revisions, so no ragged-bottom
override was added. Separate Astra mathematics and evidence reviewers accepted
the final source, including the paired-loss identity and its qualifications.

The original 83-file package, seven measured source hashes, 60 paired benchmark
observations, bibliography and figure inputs remain unchanged. No new performance
experiment or Lean execution was performed. See [validation notes](../VALIDATION.md)
for executable checks and rendered-paper inspection.
