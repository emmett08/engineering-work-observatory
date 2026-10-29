# Human effort, delivery speed and retained output

This folder contains the reviewed 13 October 2026 methodological draft extending
[Wright and Ziegler's Standard Coder paper](https://arxiv.org/abs/1903.02436).
It separates human person-hours, elapsed time to acceptance and output retained
after follow-up when AI changes how software is produced and verified.

Start with [the 17-page article](effort-speed-ai.pdf). The editable source is
[`article/main.tex`](article/main.tex), with its bibliography alongside it.
The examples are synthetic. The article proposes a measurement study; it does
not estimate an AI effect from RevBench observations.

The final re-audit added observation support to the censoring assumptions,
distinguished training targets from calibration, specified reactive audit
feedback, and maintained assigned policy throughout follow-up. The equations,
three figures and 43 Lean declarations were retained. [Review notes](review.md)
state what was checked and what remains open.

## Build and check

Run these commands from this folder:

```sh
make check
make article
```

`make check` uses Python 3.11+ and verifies the packaged file hashes, article-to-proof
source correspondence, complete claim inventory and formal receipt. It checks
retained evidence; it does not execute Lean. `make article` needs TeX Live,
BibTeX and latexmk, and writes `.build/article/main.pdf`. The reviewed PDF remains
at the top level so rebuilding does not silently replace it.

For a fresh proof build, install the pinned Lean 4.33.1 toolchain, put `lake` on
PATH and run:

```sh
make lean
```

See [`formal/README.md`](formal/README.md) for dependency-cache setup and the
verification boundary. The full source history underlying the recorded proof
commit is preserved in `formal-proof-project.bundle` (about 339 KiB).

## Reproduce the synthetic examples

In a Python environment with the pinned dependencies:

```sh
python3 -m pip install -r requirements.txt
make simulate
make figures
```

These targets run copies of the delivered scripts under `.build/`, preserving
the reviewed inputs and figures. Simulation outputs appear in
`.build/synthetic/evidence/`; regenerated figures appear in
`.build/figures/figures-v2/`. The figure script also produces alternative layouts
and inspection images. It checks its stipulated examples using exact rational
arithmetic. Neither simulation agreement nor those checks establish empirical
validity.

The three selected vector figures, their captions, alternative text and design
specifications are retained in `figures-v2/`. Figure regeneration uses the
recorded `evidence/results.json`; simulation reruns are a separate comparison
with that fixed input.

## Evidence and scope

The 54-entry inventory distinguishes the 43 checked Lean declarations from
deferred mathematical bridges, empirical conditions and definitions. Accepted
proofs use `propext`, `Classical.choice` and `Quot.sound`. The full stochastic
construction, strong-law interpretation, causal identification, measurement
validity and Python-to-Lean refinement remain outside the mechanical coverage.

Receipt-bound files retain their original bytes and paths within `formal/`.
This includes a duplicate of the current textual manuscript and one explicitly
historical manuscript used by three inventory entries. Historical audit records
contain their original workspace paths and earlier hashes; the Git bundle
preserves the corresponding source history. Current coverage is defined by
`formal/formal-verification-receipt.json` and the revised concordance it names.

This repository addition contains the paper, executable supplement and the
evidence needed to inspect that receipt. Transient agent checkouts, rendered
review pages and the full development message archive remain outside this
working package. No changes are made to the existing empirical manuscript or
the RevBench analysis pipeline.
