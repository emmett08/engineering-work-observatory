# Sharp identification and design-based estimation revision

The user authorised implementation of the preceding three-approach analysis in a
new PR using a multi-agent topology configured as `gpt-6-astra` with `ultra`
reasoning effort. Work started from PR #2 head
`b89a2584ef34637b71f5036a67673250b382d4e2` on a separate branch. PR #2 had advanced
since the original analysis and already contained the paired squared-loss
identity; this revision preserves that result rather than claiming to add it.

## Topology and exchange

Six separate native agents were configured with the requested model and effort.
Their initial contexts contained the task and fixed base, without the earlier
chat history. The host retained integration, CLI wiring, documentation, baseline
reproduction, build gates and publication responsibility.

| Agent | Responsibility | Consumed review or coordination |
|---|---|---|
| `astra_mathematics` | Allocation model, sharpness proofs, exact finite reduction, rational reference checker | Independent proof challenges; shared-capacity and implementation-scope exchange with the bounds author |
| `astra_bounds` | Application labour/productivity bounds, eligibility, report propagation | Independent coverage/cutoff tests, float-semantics challenge and host acceptance of the demonstrated old-zero repair |
| `astra_sampling` | Difference estimator, explicit frame/design validation, SRS variance and manuscript argument | Independent numerical stress tests, source attribution and CLI contract exchange with the host |
| `astra_manuscript` | Whole-paper argument, compact path representation, appendices and typesetting | Mathematical/source contracts, implementation limitations and independent rendered inspection |
| `astra_sources` | Primary-source verification, bibliography and attribution | Citation-key and passage-specific exchanges with the mathematical and manuscript authors |
| `astra_independent` | Independent initial derivations and adversarial implementation/manuscript checks | Direct cross-examination of the three technical authors and final host evidence |

Agents exchanged addressed messages that were consumed by their peers; these were
separate executions. All used the shared repository and mathematical specification,
so agreement is review rather than independent empirical replication. Monetary and
token use are not available from this workflow; no efficiency claim is made.

## Accepted decisions

The manuscript now leads with observation, task endpoints, sharp identification
and design-based estimation. The exact continuous-to-finite reduction retains all
observation/window cells and shared capacity on calendar time. The single-stream
graph is an executable reference specialisation; the application does not claim
to implement a general multi-task LP solver. Existing execution mechanisms and
the unchanged measured query comparison remain in appendices.

The application adds sharp marginal formulas for direct interval observations
and conservative task/cohort sums. Complete method-matched direct frames and
responses are required for finite complete-total ranges. Conditional productivity
ranges additionally require known output and a positive lower labour denominator.
Sampled estimates are never imposed as exact masses. Point fields retain their
meaning; no midpoint is substituted. Every range identifies its floating-point
semantics, which do not certify outward enclosure of exact real values.

Independent review exposed an old complete-zero result when the only intersecting
effort record ended after the evidence cut. The host reproduced that result on
the exact base in an isolated worktree. Both complete points and bounds now
remain unavailable in that case, with an explicit diagnostic.

The separate estimator uses frozen predictions on a complete frame and
sampling-weighted observed errors. It preserves signed results, requires explicit
selected/unselected/nonresponse states, and rejects nonresponse rather than
inventing zero. General designs provide points only; SRS variance has a separately
validated sample-count/probability contract. There is no default confidence
interval, automatic clipping, causal effect or ratio-unbiasedness claim.

Independent stress checks found cancellation under large common prediction
offsets. Numerical review required stable algebra and regression checks before
acceptance. The mathematical unbiasedness statement concerns exact design
expectations; the Rust implementation uses checked finite floating arithmetic.

## Evidence and acceptance boundary

The dependency-free rational checker compares graph extrema and reconstructed
witnesses against independent finite allocation enumeration. The Rust tests
enumerate both beneficial and harmful predictors and include malformed designs,
nonresponse, signed estimates, boundaries and report export. Additional independent
corpora and implementation stress requests were prepared outside the production
code; their scope is recorded separately by the reviewer.

The source audit verifies the relevant primary-source passages and metadata in
`sharp-identification-sources-2026-10-29.md`. Established difference estimation,
partial identification and graph algorithms are attributed as such. No broad
mathematical novelty or empirical superiority is established.

Final executed gates and rendered-manuscript inspection are recorded in
`../VALIDATION.md`. The mathematical additions are analytical proofs with finite
checks, outside the retained 43-theorem Lean receipt. The original 83-file package,
seven measured source hashes, retained 60 measurement pairs and figure bindings
are protected. This work does not recollect performance data or execute Lean.
