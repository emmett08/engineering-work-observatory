# Review of the 13 October 2026 draft

The third MAS run used the 39 delivered messages behind the earlier 16-page
draft. Fresh mathematical, measurement, source, proof and adjudication reviews
were followed by peer challenge and inspection of the actual repairs.

Four changes were justified:

| Finding | Repair |
|---|---|
| Independent censoring alone does not identify outcomes beyond observation support. | Require positive observation support through the target horizon and endpoint, including relevant conditional strata. |
| Training on imputed coding labels does not guarantee calibration. | Distinguish the training target from a separate empirical labour-calibration test. |
| Quality audits can prompt repairs and alter the outcome being sampled. | Specify feedback and repair accounting, define the assessment regime, and require appropriate assumptions and design when extrapolating. |
| Six-week intake did not specify policy duration during later follow-up. | Maintain assignment through final follow-up and register baseline timing and prior work. |

The core equations, three selected figures and all 43 Lean declarations were
unchanged. The added qualifications made the revised PDF 17 pages. Each page
was individually inspected. Five final review perspectives covered contribution,
mathematics, exposition, literature alignment and typesetting; these were
perspectives taken by one adjudicator, not five independent reviewers.

A fresh independent project build and a separate replay of the revised source
snapshot passed. The final independent receipt check verified 302 hash records,
all 54 inventory entries and all 43 axiom sets. Its retained report is
[`reaudit/proof/receipt-review.md`](reaudit/proof/receipt-review.md). The allowed
axioms and limited theorem coverage are explicit in the formal supplement.

No unresolved material finding remained within the reviewed scope. Empirical
measurement reliability, company-specific AI effects, statistical power and
external peer review remain open. Optional missingness bounds, retention
trajectories and further stochastic formalisation were deferred because none
was needed to repair an accepted finding.

The source review retained explicit access limits: detailed checking of Cui's
estimator used the separately cited June 2025 manuscript and publication
metadata, while the full published text was inaccessible. Full-text access to
the original Horvitz–Thompson and SPACE articles was also limited. The elementary
sampling argument is supplied in the manuscript.

The work used MAS Math Lab 0.3.0's existing structured Rust/nom 8 codec. It did
not create a new protocol or issue a `TheorySynthesis` certificate. Reviewers
shared inherited evidence; their agreement is correlated internal evidence.
The source reviewer used one disclosed auxiliary review and claimed no
independence from it. This provenance concerns the derivative draft, not the
authorship of the 2019 paper.
