# Validating the labour manuscript

Build the [article](article/main.tex) with `make -C paper article`. The finished [PDF](ai-assisted-software-engineering.pdf) has the same title and includes the study design and scalar-bound appendix. The article can be built using pdfLaTeX, BibTeX and `latexmk` with the standard AMS, Latin Modern, TikZ, `microtype`, `natbib` and `xurl` packages.

## Examples, figures and proposed study

Run the following from the repository root. All counts and probabilities in the examples concern constructed data or simulated trial designs. Workplace measurement requires the proposed observations.

| Command | What it checks |
| --- | --- |
| `python3 paper/check_measurement_examples.py` | Exact finite calculations for sampled-labour estimates and binned labour totals. |
| `python3 paper/check_sharp_identification.py` | Exact rational bounds, feasibility and allocations in the cutoff examples. |
| `python3 paper/review/mas-model-2026-11-01/calculations/check_model.py --check` | Recomputes the constructed cases used to check reconstruction, endpoint certificates and simultaneous coverage. |
| `python3 paper/check_explanatory_figures.py` | Reconstructs the human and workflow timelines, task clocks and sampling spreads illustrated in the article. |
| `python3 paper/figures/shared-capacity/verify.py --review` | Validates the shared-capacity figure against its inputs and the retained review hashes. Install the Python packages in `paper/figures/shared-capacity/requirements.txt` for the figure checks. |
| `python3 paper/study/ai-labour-causal/validate_protocol.py` | Checks the specified estimands, simulation cells, assessment decisions and acknowledged shortfalls in the proposed trial. |

The [study directory](study/ai-labour-causal/README.md) contains the planning model, simulation results and instructions for repeating the simulation. The validation command checks the recorded results against the protocol. It does not recruit participants or estimate an effect of AI on a team.

## Proofs and implementation

`make -C paper/formal check-receipt` checks the recorded proof inventory, evidence hashes and the binding between the article's capacity section and the archived proof target. The [formal supplement](formal/README.md) identifies exactly which statements the 54 Lean declarations cover. For a fresh execution of Lean, run `make -C paper formal` with the pinned tools on `PATH`.

`make -C paper check` runs the manuscript checks above and tests the [discrete horizon-query implementation](performance/README.md) in Rust and its results collector in Python. The benchmark crate specifies its Rust toolchain in `paper/performance/rust-toolchain.toml`. Run `make -C paper performance` to collect a fresh benchmark separately from the retained measurements.

A valid observation constraint can bound labour under the paper's assumptions. Testing whether those constraints cover the work that people actually do requires independent field observations, including time spent away from development tools.
