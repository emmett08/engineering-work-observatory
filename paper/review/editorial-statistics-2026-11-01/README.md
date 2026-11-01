# Labour-paper editorial and statistical review

Baseline: `5936f2c28fd075217b679c0b2ad3aba57b9801f3` (merged PR #7).
Scope: all seven text files compiled by `paper/article/main.tex`, plus the
unchanged Figure 1 caption and the linked causal-study protocol. The historical
paper, unused article fragments and empirical data collection are outside this revision.
The intended reader is an experienced software engineer who can follow elementary
mathematical arguments, without assumed expertise in sampling theory.

## Sequence and changes

1. **Engineering statistics and wording audit, before rewriting.** Independent
   reviews checked the quantities, denominators, assumptions, source-code
   implementation and terminology. The statistical review found no defect in
   the estimator or variance formulas. The wording review distinguished useful
   mathematical terms from phrases that concealed their referents.
2. **Experienced-engineer writing, then technical-prose rewriting.** Revise the
   explanation in dependency order, then repair sentences. The text now names
   the three timing rules, observations included in each stream group, the
   weighted sample correction and the particular claims checked in Lean.
   The conclusion points to the pilot already specified earlier in the paper.
3. **Prose refinement.** Freeze 71 source-grounded questions before applying manuscript edits.
   Review paragraph purposes, referents and transitions, then use a fresh
   candidate-only agent to answer the questions without the original text or
   answer key. A separate reviewer compared those answers with the key.
4. **Prose Workbench.** Analyse all seven exact baseline files and compare the
   candidates under the same format and known-term configuration. Inspect
   protected-content differences and linguistic risk flags against the full
   native sources. Repeat the comparisons after the final feasibility-condition
   and Rust-source-reference clarifications.

The source questions and answer comparison are retained in
[comprehension-review.json](comprehension-review.json). The separate
[discourse review](discourse-review.md) records paragraph and reference checks.
[Wording](wording-review.md) and [statistics](statistics-review.md) reviews
explain the retained terminology and statistical conditions.

## Substantive correction and protocol clarifications

Protocol decision DM-003 previously called an interval excluding a worthwhile
labour saving a "practically small labour effect". That conclusion does not
follow. For example, an interval from 3 to 5 additional person-hours per reference
unit excludes a saving beyond one hour while indicating increased labour.
The revised decision reports exclusion of worthwhile savings and any increase
or unresolved harm. Practical equivalence would require a declared equivalence
region and a suitable interval contained in it. No such result is asserted.

The protocol advances to 1.0.1. Its numerical design, estimator, planning code
and simulated results are unchanged. Descriptions of this unregistered study's
planning now say "specified". The article exposes existing protocol conditions:
approximate one-sided 97.5% paired t bounds, provisional bias allowances of
0.2 person-hours per reference unit and 0.01 retained fraction, and failure to
claim benefit when a team endpoint is unresolved. It names both tested team
counts and the 90% joint-success target. Neither count qualifies across the
required hypothetical useful-effect scenarios. The experiment remains unrun.

The abstract and introduction now explicitly condition attained extrema on
compatible observations. This restores the nonempty-feasible-set condition
already present in the unchanged proposition, rather than adding a result.

## Checks and limits

- All mathematics, citation commands, labels and input commands are byte-identical
  and in the same order. Original cross-references are retained, with one added
  reference to the existing causal-experiment subsection. See
  [source-protection.json](source-protection.json).
- The candidate-only reader answered 71 questions with 150 verbatim source-line
  evidence items. Separate review found 70 preserved answers and one disclosed
  clarification of the already specified study design, with no uncertain or
  unanswerable questions. These are agent editorial judgements, not observed
  human comprehension or proof of semantic equivalence.
- Workbench flags intentional changes to whole proof/proposition environments
  when their prose changes, the added cross-reference and percentage escapes.
  Those are reviewed differences, not an automatic preservation pass. The
  [diagnostic record](linguistic-checks.json) retains all returned risk flags and
  their dispositions. Scores were not optimised or treated as reader outcomes.
- An independent [formal concordance review](../../formal/evidence/current-manuscript-editorial-review.md)
  binds the changed interval-identification prose to the frozen proof targets.
  Static receipt validation passes, and an isolated changed-byte test rejects
  a stale binding. Lean sources, checker scripts and original proof evidence
  are unchanged. No new Lean execution is claimed.
- `make -C paper check` and `make -C paper article` pass. The paper check includes
  preserved-package checks, exact measurement/identification examples, model
  checks, protocol consistency, formal receipt validation, forecasting checks,
  seven performance Rust tests and eleven Python tests. The protocol validator
  retains all six planning-adequacy failures. It establishes record consistency,
  not empirical adequacy.
- The rebuilt PDF has 17 pages. All pages were rendered and inspected in a
  contact sheet, with detailed figure, experiment and bibliography review.
  Citations and cross-references resolve and there are no overfull boxes.
  One bibliography underfull-vbox warning remains on page 16, with the last
  two references on page 17. The existing online-first Cui missing-pages
  BibTeX warning is retained. See [build-checks.json](build-checks.json).

## Tool provenance

The user identified the Prose Workbench package as **1.0.2**. Its deployed
capabilities response and deterministic analysis engine report **1.0.0**.
These are recorded separately, without asserting a package downgrade or that
the reported engine implements features from a different version.

The advertised wording scanner and refinement `text_lab.py`/`linguistic_core.py`
resources could not be read. Full contextual review and the available hosted
Workbench diagnostics were used. No bundled scanner, native refinement CLI,
automatic freeze or native map-validation run is claimed. Hosted extraction
omits some formal environments and limits returned observation lists, so
manual review covered the complete native sources. No empirical benefit,
human-reader improvement or new calibration result follows from this editing.
