# Precise prose in both current papers

The requested revision applies `write-for-experienced-engineers` and
`audit-engineering-wording` to the complete labour-identification and automatic
merge-count manuscripts. Comparison starts at forecasting commit
`1ef269334c774df4f6e46fe4a5bac52d52420ec3`. Both current PDFs are rebuilt.

## Changes that affect the reader

The labour paper now names the identified range and states that every included
value is attainable under its observations and allocation constraints. Its title
is *What software-work telemetry identifies: attainable labour bounds and
design-based estimation*. The experimental AI intervention is defined by the
available tools and versions, eligible tasks, autonomous actions, review
responsibilities and release requirements. Assignment and actual tool use remain
separate measurements.

The forecasting paper names the collection requirements, eligible complete
periods, stored forecasts and observed open-PR counts. Repeated qualifications
are consolidated where they serve the same argument. The past-to-future
assumption, two different training frames and failed recent-window comparison
retain their exact scope. Future congestion and causal resource constraints
remain unestablished.

Both papers replace metaphorical or incomplete references with the quantity,
rule, record or action intended. Figure labels identify expected totals,
conditional correctness probabilities and observed timing comparisons.
Captions and generated tables use the same terminology as the surrounding text.
The caption generator and its generated caption/finding strings are updated
together. Formulas, observations, curves and algorithm operations retain their
previous values and meanings.

The rendered papers contain no standalone `sharp`, `policy` or `policies`.
The exact published reference title *Probabilistic Forecasts, Calibration and
Sharpness* remains verbatim. Internal LaTeX identifiers and historical audit
records retain their original spelling so references and provenance remain valid.

## Two substantive clarifications

1. The productivity outer interval can be attained without pairing zero output
   with maximum labour. For example, admissible pairs satisfying
   `0 <= Q <= 1` and `1 <= H <= 1 + Q` have marginal ranges `[0,1]` and `[1,2]`.
   Their ratios fill `[0,1]` at `H=1`, while `(Q,H)=(0,2)` is inadmissible.
   The text now requires every ratio in the claimed exact range to be attainable
   from an admissible pair. Admissibility of the entire Cartesian product remains
   sufficient. The displayed ratio formulas are unchanged.
2. The proof-coverage table now separates the probability argument for first-pass
   composition from the independent organisational observations needed to assess
   the model. A probability proof cannot establish empirical accuracy.

## Review and verification

Six separate `gpt-6-astra` agents, each using `ultra` reasoning, handled the labour
revision, forecasting revision, precise terminology, mathematical fidelity,
paragraph flow and figures. Every agent exchanged proposals and challenges with
all five peers and consumed their replies, completing all 30 directed peer edges.
Each reviewer read both complete manuscripts in their actual include order.
Independent flow, fidelity and final rendered-wording reviews found no unresolved
material issue. These are development reviews rather than external peer review.

| Check | Result |
|---|---|
| Labour display mathematics | All 42 blocks preserved byte for byte in the six revised article files. |
| Labour references | All 163 label/reference/input/citation occurrences and ordered citations preserved in those files. |
| Forecast mathematics/references | All five equations, seven citation commands and nine labels preserved. |
| Caption symbol definitions | Existing symbols repeated locally to explain table headers. No formula changed. |
| Numerical outputs | Forecast table rows, the 11.03% adverse RPS comparison, 60 measured timing pairs and plot coordinates unchanged. |
| Bibliographies | Both current bibliography source files byte-identical to the comparison revision. |
| Generated wording | Generator logic unchanged. Caption/table text agrees with its generator. |
| Exact checks | Full `make -C paper check` passes, including all 130 reference scores and five corruption checks. |
| PDF builds | Both compile with resolved references and no overfull boxes. The labour log has two nonfatal underfull-box notices, inspected in the rendered pages. |
| Rendered review | All 29 labour pages and eight forecasting pages inspected. Changed figure labels and result tables also inspected at page resolution. |

The application source, tests, dependency files, reference data, original issued
forecast bundle, historical 17-page manuscript/proof package and measured Rust
kernels are unchanged. No benchmark observations were recollected and no new Lean
execution was performed. Existing figure audit receipts describe their earlier
revision. The present review records the wording-only figure changes separately.

Final PDF SHA-256 values:

- `paper/ai-assisted-software-engineering.pdf`: `12c957b83f804216e7caeed337578f05001a5f3968be2fcedeaa7e46923efcfb`
- `paper/forecasting/automatic-throughput-forecasting.pdf`: `3c4555f50e748a05adeadd6075289bf9c458b14cf7458695c58c45d0988252ab`
