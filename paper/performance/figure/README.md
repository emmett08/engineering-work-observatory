# Repeated-query data story

The figure asks when preprocessing repays its cost for repeated cohort-horizon
queries. Its inputs are **observed machine timings on generated records**, not
observed engineering labour. Every indexed total includes input copies, index
construction, queries and disposal. The benchmark's environment and exclusion
boundary are retained in `../results/environment.json`.

`prepare.py` checks the 60 retained rows, fixture sizes, positive timings, included
costs, complete pairing and three seeds with five repetitions each. It writes
`horizon-data.csv`, generated labels, the separate caption, calculation checks,
and current input hashes in `../../horizon-performance.spec.json`. No observations
are dropped or imputed. Whiskers are observed minima and maxima, not confidence
intervals. The alternative's speed ratio is calculated within each pair before
aggregation; it is not a ratio of marginal medians.

All one-query pairs favoured scanning. Ten-query pairs varied in direction.
All 100-query and 1,000-query pairs favoured indexing. Only those four query
counts were measured: the figure does not locate an exact break-even count.

Two alternatives were rendered at 152.4 mm × 85 mm. The selected paired-time
chart retains the absolute elapsed-cost comparison and shows why index
construction dominates short batches. `ratio-prototype.tikz.tex` and
`prototypes/ratio.pdf` retain the alternative, which makes the equal-time reference
and paired variation more immediate but hides absolute elapsed cost. See the
independent agent's review for the comparison and limitations.

From `paper/`, regenerate the calculation and figure:

```sh
python3 performance/figure/prepare.py
python3 performance/figure/render.py \
  --source figures/horizon-performance.tikz.tex \
  --output figures/horizon-performance.pdf \
  --spec horizon-performance.spec.json \
  --audit performance/figure/horizon-performance.audit.json \
  --preview-dir /tmp/horizon-performance-preview
python3 performance/figure/render.py \
  --source performance/figure/ratio-prototype.tikz.tex \
  --output performance/figure/prototypes/ratio.pdf \
  --preview-dir /tmp/horizon-performance-ratio-preview
```

The calculation uses the Python standard library. Rendering requires pdfLaTeX,
`standalone`, Latin Modern and PGFPlots 1.18; the Python checks require `jsonschema`,
PyMuPDF and Pillow. The manuscript includes the picture from its `article/`
directory and needs `\usepackage{pgfplots}` and `\pgfplotsset{compat=1.18}`.

The two requested skills' reference material and JSON schemas were available,
but the referenced `figure_spec.py`, `render_tikz.py` and `figure_review.py` scripts
could not be retrieved and were absent locally. `render.py` is an explicitly
project-local replacement with bounded checks: JSON schema and input identity,
reference resolution, compilation, one-page output, physical width, extracted
font and stroke floors, expected labels, text bounds, and current hashes.
It also produces colour, greyscale and thumbnail inspection images. These checks
do **not** claim the unavailable skill renderer's full coverage. The review is
agent screening; human interpretation benefit remains unmeasured.

Changing data, source or caption requires rerunning affected checks and renewing
the review hashes. Recompilation can change PDF metadata and consequently its
hash even when the visible figure is unchanged.
