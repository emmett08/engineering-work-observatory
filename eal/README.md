# Engineering Work Observatory arguments

Open [the main argument](00-measuring-human-labour.eal) to follow the claims and their dependencies together. Open a file numbered 01–19 to assess one argument independently. Each file contains its own definitions, grounds, examples or derivations, and conditions of application. The main file embeds the supporting declarations so it can run from a directory containing that file alone.

| Toulmin role | EAL/4 declaration |
| --- | --- |
| Claim | `claim finding` |
| Grounds | `evidence grounds` |
| Warrant | `reasoning warrant` |
| Backing | `evidence backing_material` |
| Qualifier | `assumption applicability` |
| Rebuttal | `objection challenge`, which can activate if `claim exception_applies` gains support |

The embedded material distinguishes derivations, synthetic examples and proposed comparisons. The `structured/1` assessment checks the argument against that material. Mathematical validity and applicability at work require examining the stated reasoning and conditions. The exception claims describe conditions that would block a conclusion. They need case-specific evidence before they activate.

## Validate and assess

Use the EAL/4 CLI tested at commit `690b0080afac7d4b8847382e050cdb71f54cdbf2`. Run from within this directory.

```sh
for argument in *.eal; do
  eal-core --workspace "$PWD" validate "$argument" || exit 1
done

eal --workspace "$PWD" collect 00-measuring-human-labour.eal --context '{}'
```

Pass the returned collection identifier to the assessment command. The same commands work with any numbered file in isolation.

```sh
eal --workspace "$PWD" reason 00-measuring-human-labour.eal \
  --collection COLLECTION_ID --context '{}'
```

## Argument index

| File | Claim |
| --- | --- |
| [00-measuring-human-labour.eal](00-measuring-human-labour.eal) | A team can bound human labour from time observations and capacity limits or estimate it from a planned random sample of measured hours. A claim of benefit from offering AI also requires independently checked labour and retained functionality. |
| [01-commits-and-human-labour.eal](01-commits-and-human-labour.eal) | Commit observations alone do not identify mean active human time across the activity models in the counterexample. |
| [02-human-workflow-and-elapsed-time.eal](02-human-workflow-and-elapsed-time.eal) | Account for each person’s labour once across concurrent activities, and report workflow duration and elapsed time until acceptance as separate quantities. |
| [03-intake-scope-and-retained-output.eal](03-intake-scope-and-retained-output.eal) | Fix the scoring rule and original work scope at intake, then credit timely acceptance by the fraction of that scope still satisfied at the scheduled follow-up. |
| [04-shared-capacity-in-96-histories.eal](04-shared-capacity-in-96-histories.eal) | Enforcing a shared time budget can narrow labour ranges and resolve threshold decisions when several activities draw on the same person. |
| [05-exact-range-under-preemption.eal](05-exact-range-under-preemption.eal) | For a non-empty feasible allocation model, the minimum and maximum of its linear programme delimit exactly the attainable values of a cellwise linear labour target. |
| [06-endpoint-certificates-and-coverage.eal](06-endpoint-certificates-and-coverage.eal) | Feasible allocations and matching dual bounds certify labour endpoints. The resulting interval inherits a simultaneous coverage guarantee for the observation limits when the structural constraints also hold. |
| [07-single-stream-shortest-paths.eal](07-single-stream-shortest-paths.eal) | For one allocation stream, a difference-constraint graph detects infeasibility and gives the exact attainable labour range over any interval whose endpoints are graph vertices. |
| [08-coupled-productivity.eal](08-coupled-productivity.eal) | Sharp deterministic productivity bounds must respect jointly feasible output and positive labour totals. |
| [09-design-unbiased-labour.eal](09-design-unbiased-labour.eal) | Inverse-probability weighting estimates finite-frame labour without design bias, with or without fixed predictions. |
| [10-precision-from-predictions.eal](10-precision-from-predictions.eal) | Under simple random sampling without replacement, prediction correction improves precision when residuals vary less than actual interval hours. |
| [11-missing-sampled-allocations.eal](11-missing-sampled-allocations.eal) | Random follow-up with known probabilities can correct initial non-response when follow-up measurements are accurate. Remaining unknown hours can be bounded. |
| [12-sampled-productivity-contrast.eal](12-sampled-productivity-contrast.eal) | An unbiased labour estimate may bias a plug-in productivity ratio. A valid output-minus-rate-times-labour contrast supports inference on productivity. |
| [13-historical-labour-benchmark.eal](13-historical-labour-benchmark.eal) | The mean measured labour of earlier comparable tasks provides a transparent reference forecast at intake. |
| [14-forecast-validation-sampling-noise.eal](14-forecast-validation-sampling-noise.eal) | Two fixed forecasts can be compared by their expected squared-error difference against a common unbiased sampled reference, while other accuracy measures may need an audit. |
| [15-paired-offer.eal](15-paired-offer.eal) | The mean within-pair difference estimates the equally weighted effect of offering the specified AI toolset on the enrolled teams’ fixed workloads. |
| [16-benefit-criteria.eal](16-benefit-criteria.eal) | A benefit declaration requires evidence of a saving greater than one person-hour per initial unit and a loss smaller than five percentage points in the retained fraction. |
| [17-conditional-detection.eal](17-conditional-detection.eal) | Neither candidate trial size qualified under the stated planning criteria. The simulation estimates detection probabilities conditional on a stipulated effect and specified measurement assumptions. |
| [18-differential-measurement-bias.eal](18-differential-measurement-bias.eal) | Increasing trial size can increase false declarations when systematic measurement differences favour AI beyond the allowed errors. |
| [19-completeness-and-pilot.eal](19-completeness-and-pilot.eal) | Requiring complete outcomes from every pair caps the probability of declaring benefit, and a six-pair pilot cannot establish that incomplete pairs are rare. |
