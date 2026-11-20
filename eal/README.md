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

The embedded derivations justify the mathematical claims. Enumerated examples supply known outcomes for checking the calculations. Proposed comparisons specify how teams would test the effect of AI access. The `structured/1` assessment checks the argument against that material. Mathematical validity and applicability at work require examining the stated reasoning and conditions. The exception claims describe conditions that would block a conclusion. They need case-specific evidence before they activate.

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
| [01-commits-and-human-labour.eal](01-commits-and-human-labour.eal) | Commit observations alone leave expected human working time undetermined across the two activity models. |
| [02-human-workflow-and-elapsed-time.eal](02-human-workflow-and-elapsed-time.eal) | Count each person’s labour once across concurrent activities. Measure workflow duration and elapsed time to first acceptance separately. |
| [03-intake-scope-and-retained-output.eal](03-intake-scope-and-retained-output.eal) | Fix the scoring rule and original requirements at intake. For timely acceptances, base the retained score on the fraction of the original requirement still satisfied at follow-up. |
| [04-shared-capacity-in-96-schedules.eal](04-shared-capacity-in-96-schedules.eal) | A shared time budget can reduce the range of possible person-hours and resolve threshold decisions when several activities draw on the same contributor. |
| [05-exact-range-under-preemption.eal](05-exact-range-under-preemption.eal) | In a nonempty feasible allocation model, the minimum and maximum of the linear programme give the exact endpoints of the attainable range of the weighted labour total. |
| [06-endpoint-certificates-and-coverage.eal](06-endpoint-certificates-and-coverage.eal) | A feasible allocation that matches a dual bound verifies the corresponding labour endpoint. Valid structural constraints transfer simultaneous coverage of the input ranges to coverage of the target range. |
| [07-single-stream-shortest-paths.eal](07-single-stream-shortest-paths.eal) | For one allocation stream, a graph of differences detects inconsistent constraints and gives the exact attainable labour range for a period whose endpoints are vertices. |
| [08-coupled-productivity.eal](08-coupled-productivity.eal) | Sharp deterministic productivity bounds come from optimising the ratio over feasible pairs of retained output and positive labour. |
| [09-design-unbiased-labour.eal](09-design-unbiased-labour.eal) | Inverse-probability weighting gives a design-unbiased estimate of total labour in the stated frame. Fixed predictions can supply the starting total before residuals are added. |
| [10-precision-from-predictions.eal](10-precision-from-predictions.eal) | Under simple random sampling without replacement, prediction correction improves precision when residuals vary less than actual interval hours. |
| [11-intervals-without-replies.eal](11-intervals-without-replies.eal) | Known follow-up probabilities and accurate replies allow inverse-probability weighting to correct initial non-response. Interval durations bound any remaining unknown labour. |
| [12-sampled-productivity-contrast.eal](12-sampled-productivity-contrast.eal) | Dividing output by an unbiased labour estimate can yield a biased productivity rate. Confidence intervals for output minus rate times labour can identify a confidence set for the rate. |
| [13-labour-benchmark-from-prior-tasks.eal](13-labour-benchmark-from-prior-tasks.eal) | The mean measured labour of earlier comparable tasks provides a transparent reference forecast at intake. |
| [14-forecast-validation-sampling-noise.eal](14-forecast-validation-sampling-noise.eal) | Scoring two fixed forecasts against the same unbiased estimate of labour preserves their expected difference in squared errors. Measures such as absolute error and interval coverage may require independently checked hours. |
| [15-paired-offer.eal](15-paired-offer.eal) | The mean difference within pairs estimates the equally weighted effect of offering the specified AI toolset on the enrolled teams’ fixed workloads. |
| [16-benefit-criteria.eal](16-benefit-criteria.eal) | Declaring a benefit requires evidence of a saving greater than one person-hour per initial unit and a loss smaller than five percentage points in the retained fraction. |
| [17-conditional-detection.eal](17-conditional-detection.eal) | Both trial sizes fell short of the planning criteria. Their simulated detection probabilities depend on the stipulated effect and the assumed variation in outcomes and measurement errors. |
| [18-differential-measurement-bias.eal](18-differential-measurement-bias.eal) | A larger trial can declare a benefit falsely more often when systematic measurement errors favour AI by more than the analysis allows. |
| [19-completeness-and-measurement-feasibility.eal](19-completeness-and-measurement-feasibility.eal) | Requiring both outcomes from every pair limits the chance of establishing a benefit. Six pairs provide too little evidence to establish that losses are rare. |
