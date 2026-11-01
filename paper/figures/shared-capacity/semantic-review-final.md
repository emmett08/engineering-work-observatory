# Independent semantic review: final

Reviewer: `/root/timeline_panel/timeline_reader`.

Initial image-and-caption-only reconstruction was saved in `/workspace/work/shared-capacity/timeline-reader-initial.md` before reading the specification. This final review compares that reconstruction with `shared-capacity.spec.json`, the revised caption, and the mechanical audit. I did not read or execute the mathematical source code, and this review does not independently certify its arithmetic or general feasibility proof.

## Comparison with the specification

The initial reading agrees with the specified reader question and claim: separate bounds give a square and a sum interval [0,2], both hourly capacity constraints leave the diagonal and combined total {1}, and the marked midpoint has the displayed one-person, one-activity-at-a-time schedule. I correctly reconstructed the intended task totals, first-hour quantities, units, two-hour interval, midpoint, four half-hour blocks, and first-hour divider without access to the specification.

The square, diagonal, interval, diamond, timeline lengths, and A/B labels were read in their specified roles. Neither grey fill nor the capped interval was read probabilistically. The two panel-(b) rows were read as categories, without a quantitative vertical interpretation. The timeline was read as elapsed time and full-attention task duration, rather than a second labour axis or a diagram of causal states.

The caption's constructed-example statement and final limit prevent the principal prohibited readings: an observed work history, a unique schedule, a measured workplace result, or a confidence interval. The example's fixed aggregate is bounded by the particular saturated two-hour case. Nothing in the rendering claims that every shared-capacity system must identify its aggregate. Panel (c) supplies a witness for P only; my initial reconstruction did not mistake this single witness for a proof that every point on the diagonal is feasible.

The revised final sentence, 'The example allows the person to switch tasks at arbitrary times; the schedule is neither unique nor observed,' resolves the subject-precision caution in the initial review. It makes arbitrary switching a model permission and retains the evidence limit. 'Separate bounds added' still relies on the accompanying sentence for a precise explanation, but the caption supplies it, so I do not regard this as an unresolved issue.

## Candidate observations and preference

The compact candidate at 152.4 by 80 mm is preferred. Its timeline is visibly smaller than the bottom candidate but every task label, half-hour endpoint, and the dashed one-hour divider remains readable. The explicit 'One schedule for P' title and the caption connect the schedule to the marked point. No cross-panel connector is needed to recover that relation. Both colour and greyscale preserve the connection and all task identities.

The 152.4 by 112 mm bottom candidate is also semantically clear and legible. Its longer timeline adds no extra quantities, distinctions, or evidence; panel (b) gains substantial empty vertical space. It is a reasonable alternative where a full-width timeline is required, but is less suitable for the requested small witness panel.

The greyscale compact rendering retains dark A segments with white letters and pale B segments with black letters, a dark feasible line and diamond, visible interval caps, and direct region labels. I found no material overlap, clipping, or illegible mark in the supplied compact or bottom renders. This is a visual review of the supplied renderings assessed against the stated physical dimensions; it is not a paper-print inspection.

## Mechanical evidence and bounded verdict

The audit reports 152.3993 by 79.9994 mm for the compact output, minimum text size approximately 8.5 pt, minimum stroke approximately 0.45 pt, vector output with no raster images, and no mechanical errors. The approximately 0.00013 pt difference below the nominal 8.5 pt text floor is consistent with measurement precision and is not a visible shortfall. The audit explicitly limits its coverage to project-local mechanical checks; I retain that limit. The shell-escape-disabled package warning does not indicate a visual or semantic defect in the ordinary TikZ figure reviewed here.

**Verdict: accept the compact candidate for the stated bounded claim and final size.** No remaining concrete change is required by this independent semantic review. Acceptance covers legibility, intended reading, witness linkage, colour/greyscale interpretation, and caption scope. It does not substitute for the separate calculation verification, prove the general theorem, or identify the actual work history.
