# Shared capacity and first-hour labour

This figure uses the constructed two-task example in Section 4.1: one person
spends one person-hour on each task during two hours. Separate first-hour bounds
are `[0,1]`; capacity in both hours forces their combined total to one person-hour.

## Source and design

- `tikz.tex`: one include-ready `tikzpicture`, without a document, float or caption.
  Compile from `paper/article/`, as the manuscript does.
- `coordinates.tex`: generated exact values; `caption.tex`: separate caption;
  `alt-text.txt`: accessible description.
- `figure.pdf`: selectable vector output at 152.4 × 80 mm, with Latin Modern fonts,
  labels of at least 8.5 pt and stroked rules of at least 0.45 pt.
- `../../shared-capacity.spec.json`: quantities, sets, encodings, input status,
  physical target and dependencies.
- `figure.audit.json` and `figure.review.json`: current source, inputs, caption
  and PDF hashes, bounded mechanical checks and separate agent review.

The side-by-side layout leads from the constraint geometry to the combined total.
The added panel (c) marks `P=(0.5,0.5)` on the same diagonal and supplies one
attaining schedule: A for the first half-hour, B for the next, then A and B for
another half-hour each. Every task receives one person-hour over the two hours,
including half a person-hour in the first hour. The timeline assumes arbitrary
interruption. It does not identify an observed or unique history.

`prototypes/compact-timeline.tikz.tex` and its PDF retain the compact composition;
`prototypes/bottom-timeline.tikz.tex` and its PDF retain a 112 mm-high alternative
with the timeline below both original panels. The compact composition keeps the
original 80 mm height and 52 mm-square plot, with a legible 54 mm time axis in the
right column. The bottom alternative gives the timeline 104 mm but adds 32 mm
of height without revealing another quantity. Both use the same point, intervals
and type sizes. The earlier `stacked` prototype remains a historical comparison
of the two-panel figure, rather than an alternative for the current three panels.

The square contains pairs permitted by separate bounds; the diagonal contains
jointly feasible pairs. The capped interval and diamond show the sum projections
`[0,2]` and `{1}`. These are deterministic sets, with no probability or
empirical-frequency meaning. A point alone gives interval totals. Panel (c)
adds a particular complete schedule whose integrals equal the marked totals.
Task identities remain directly labelled A and B in greyscale; the bars encode
elapsed time, not probability or the number of people.

## Build and check

TeX dependencies: pdfLaTeX, standalone, TikZ, AMS maths and Latin Modern.
Python dependencies are declared in `requirements.txt`. From the repository root:

```sh
python3 -m pip install -r paper/figures/shared-capacity/requirements.txt
make -C paper figure
make -C paper article
python3 paper/figures/shared-capacity/verify.py --review
make -C paper check
```

The manuscript adds `\usepackage{tikz}` to its existing AMS/Latin Modern preamble.
Its float inputs the picture and separate caption. The explicit short caption
protects the caption input from moving-argument expansion in amsart with hyperref.

`calculate.py --check` rejects stale or missing generated values. Its exact
rational reconstruction checks 1,089 grid pairs: 33 feasible, 528 violating
first-hour capacity and 528 violating second-hour capacity, plus three explicit
feasible schedules and four rejected pairs. The continuum result follows from
the two capacity inequalities; finite checks do not replace that argument.
The midpoint timeline is derived from its exact cell masses. Separate overlap
integration checks both whole-task totals, both first-hour totals and their
combined first-hour total. The four blocks are positive, disjoint and consecutive,
covering the observation interval at full attention. Their endpoints and task
identity are retained in `calculation.json`, with generated TeX interval lists.

`render.py` records PDF dimensions, text/stroke floors, vector content, expected
labels, actual local TeX inputs and hashes. `verify.py` checks schemas, typed
references and artifact/review freshness. The skill schemas were available but
its bundled scripts were unavailable. These are explicitly project-local bounded
checks adapted from the existing figure workflow, without claiming the unavailable
tools' full coverage.

The standalone class warns that shell escape is disabled. Ordinary TikZ rendering
needs no external commands; the warning is accepted without enabling shell escape.
Standalone date/trailer metadata are suppressed for stable exports on the same
toolchain. Different tool versions can change bytes and require renewed bindings.

The semantic review is agent screening. Human interpretation benefit remains
unmeasured by agent screening.
The current independent reader was `/root/timeline_panel/timeline_reader`.
Its image-and-caption-only reconstruction was recorded before specification
comparison; both stages are retained in `semantic-review-initial.md` and
`semantic-review-final.md`. It preferred the compact layout and accepted the
point-to-schedule relation, final caption and colour/greyscale reading at the
declared size. The exact calculation and the general theorem are outside that
visual review's authority. Final manuscript placement is inspected separately
when the article is rebuilt.
