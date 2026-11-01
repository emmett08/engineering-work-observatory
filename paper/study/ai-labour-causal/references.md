# Methodological sources and their consequences

The following primary texts were inspected on 1 November 2026. The companion
`references.bib` proposes entries for the canonical article bibliography; it is
not an independently maintained replacement bibliography.

| Source | Inspected material | Design consequence and limits |
| --- | --- | --- |
| Imai, King and Nall (2009), *Statistical Science* 24(1), 29–53, DOI 10.1214/08-STS274 | [Author-hosted full paper](https://imai.fas.harvard.edu/research/files/cluster.pdf), especially Sections 3–4 and discussion of SATE variance | Pair similar clusters using pretreatment information; analyse at the randomised-cluster level and state weights/estimand. The present equal-team mean has its own elementary unbiasedness argument. Their discussion does not make an arbitrary Student-t interval exact or justify changing cluster sizes without changing weights. Cite at the pairing/randomisation choice. |
| Hudgens and Halloran (2008), *JASA* 103(482), 832–842, DOI 10.1198/016214508000000292 | [Full publisher-form PDF](https://www.treatment-effects.com/Hudgens-Halloran-2008.pdf), Section 2.1 | Potential outcomes allowing within-group effects still assume no effects between distinct groups. Shared developers/reviewers/code/resources therefore matter to identifying a simple team-level effect. Their two-stage vaccination design is not claimed as the present implementation. Cite at the interference boundary. |
| Piaggio et al. (2012), *JAMA* 308(24), 2594–2604, DOI 10.1001/jama.2012.87802 | [Full journal article](https://jamanetwork.com/journals/jama/fullarticle/1487502), objectives, margin rationale and inference sections | State and justify the quality-loss margin before outcomes, and compare the appropriate confidence bound with that margin. Failure to find a difference is insufficient. Clinical margin examples are not evidence that five percentage points is acceptable for software; that value remains provisional. Cite at the separate noninferiority criterion. |
| Morris, White and Crowther (2019), *Statistics in Medicine* 38(11), 2074–2102, DOI 10.1002/sim.8086 | [Full journal text](https://onlinelibrary.wiley.com/doi/10.1002/sim.8086), ADEMP and Monte Carlo uncertainty guidance | Record aims, data-generating assumptions, estimands, methods and performance measures. Evaluate partial nulls, joint decisions and calibration; retain Monte Carlo SE and binomial intervals. Simulation under assumed team baselines does not establish workplace power or generalisation. Cite at the planning simulation. |
| Albers and Lakens (2018), *JESP* 74, 187–195, DOI 10.1016/j.jesp.2017.09.004 | [Author-hosted full publisher PDF](https://www.casperalbers.nl/files/powerJESP2018.pdf), abstract and Sections 3–4 | Small pilot effects and selective follow-up can mislead principal-study power. Use the pilot for measurement/nuisance ranges and keep the useful effect grounded in the intended decision; do not size the main trial from a favourable pilot effect. Cite at pilot/confirmation separation. |

No source supplies this study's true variance, matching correlation, tool effect,
measurement-bias bound, quality margin or available budget. The numerical
scenarios and margins in the protocol are explicitly author-specified planning
assumptions. No methodological reference turns them into empirical estimates.

The probability control for a conjunction of two component decisions follows
directly from event inclusion: a false conjunction entails a false rejection of
at least one true component null. It requires valid component tests; the
conjunction argument itself does not validate the approximate t calculation.
