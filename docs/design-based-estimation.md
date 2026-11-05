# Model-assisted labour estimation

`observatory estimate-labour FILE` reads a standalone labour-estimation JSON
document. Version 1 retains the original statistical input and output. Version 2
adds person-window and interval geometry checks and retains that geometry in its
result. Neither version imports records, creates a state directory, or changes a
frozen cohort report. Executable examples are
[`model-assisted-labour.json`](../examples/model-assisted-labour.json) and
[`model-assisted-labour-v2.json`](../examples/model-assisted-labour-v2.json).

The sampling frame lists every time interval for every person included in the
labour total. Randomly select intervals from this list using a design that gives
each interval a known positive selection probability. Let `f_m` be its predicted
person-hours, `y_m` its observed person-hours if selected, and `pi_m` its selection
probability. Predictions are calculated before sampling and remain unchanged
whichever intervals are selected. The difference estimate is
`sum_all(f_m) + sum_observed((y_m-f_m)/pi_m)`. Its comparator is the
Horvitz–Thompson (HT) estimate `sum_observed(y_m/pi_m)`. Both estimate the total
labour across the same frame.

The difference estimate averages to the true total across repeated samples
when the frame is complete, selection probabilities describe the actual design,
sampled labour measurements are accurate and predictions are fixed before
sampling. This property is called design-unbiasedness.

Frame units must partition each person's target observation period so that the total counts each contribution
once. Version 1 accepts that geometry as a caller declaration. Version 2 checks
the supplied partition against declared person windows. Neither version can
establish that those windows and people cover all actual target work.

## Version 1 statistical input

| Field | Meaning |
|---|---|
| `schema` | Exactly `observatory.labour-estimate-input/1` |
| `frame_id` | Nonempty identity for the fixed person-interval frame |
| `frame_complete` | Must be `true`; unrecorded work outside the frame is not estimated |
| `predictions.frozen_before_sample` | Must be `true` |
| `predictions.provenance` | Nonempty description of the frozen model or prediction source |
| `design` | An explicit supported sampling design |
| `units` | Every unit in the frame, with distinct nonempty IDs |

Each unit requires finite `predicted_person_hours` (signed predictions are allowed), finite
`inclusion_probability` in `(0,1]`, and an `observation` object. Status `observed`
requires nonnegative finite `person_hours`, including explicit zeros. Status
`unselected` supplies no outcome. Status `nonresponse` records a reason and causes
the estimator to reject the incomplete input. Unknown fields are rejected.

The design object is either `{"kind":"general"}` or
`{"kind":"simple_random_without_replacement","sample_size":n}`. General designs
need genuine first-order inclusion probabilities and permit dependent inclusion;
they provide no design-variance calculation. For simple random sampling, `n`
must agree with the number of observed units, and every inclusion probability
must equal `n/M` for frame size `M`, with a relative tolerance of eight `f64`
machine epsilons for representation differences. A realised sample alone cannot
verify random selection. A declaration
of simple random sampling must describe the sampling procedure actually used.

## Version 2 geometry

The typed API is `estimate_labour_v2(&LabourEstimateInputV2)`. Its input has four
fields, with unknown fields rejected at every level:

| Field | Meaning |
|---|---|
| `schema` | Exactly `observatory.labour-estimate-input/2` |
| `estimation` | The complete version 1 document above, retaining its `/1` schema |
| `frames` | One declared observation window for each person |
| `geometry` | Exactly one interval record for every `estimation.units` ID |

Each person window supplies a distinct nonempty `person_id` and RFC 3339 `start`
and `end` timestamps, parsed into `chrono::DateTime<Utc>`. Each geometry record
supplies `unit_id`, `person_id`, `start` and `end`. Intervals are half-open and
must have positive duration. Every unit ID must exist in the nested statistical
frame, and every person ID must have a declared window. Duplicate geometry,
missing units, duplicate person windows and windows without units are errors.

For each person, intervals must exactly partition that person's window. Adjacent
intervals are accepted. Overlaps, gaps and out-of-window intervals are rejected,
including overlaps between different unit IDs. Different people can have
concurrent intervals. Records can arrive in any order. Validation sorts references
for its checks and retains the supplied record order in the result.

Observed `person_hours` must not exceed their interval's elapsed hours. The check
compares the exact rational value of the supplied `f64` observation with elapsed
nanoseconds divided by `3_600_000_000_000`. It uses no capacity tolerance and
preserves subsecond intervals. A floating-point hour value rounded above the
interval's exact duration is rejected, even if the difference is one floating-point
step. Predictions remain signed auxiliary values and need not satisfy this
physical capacity bound.

Version 2 reports `observatory.labour-estimate/2`, containing the retained `frames`
and `geometry`, an `estimation` field with the original `/1` statistical result,
and geometry-specific limitations. The statistical calculations, missingness
states, signed corrections and variance conditions are the same in both versions.
The nested statistical result still lists the assumptions that the supplied
records cannot establish. Geometry validates coverage of declared windows. It
does not validate the roster, observation accuracy, actual random selection or
prediction provenance.

## Output and uncertainty

The result records the frame identity, prediction source, sampling design,
frame size and observed sample size. It reports the predicted total and the HT
and difference estimates of total labour. It also records limitations and the
variance status:

| `variance.status` | Interpretation |
|---|---|
| `estimated_srs` | Variance under simple random sampling, estimated from the observed sample with the finite-population correction |
| `known_zero_census` | The declared simple-random-sampling design observes the entire frame, so sampling variance is zero |
| `unavailable` | The declared design or sample size does not support this variance calculation |

For simple random sampling (SRS) without replacement with `n>=2` and fewer
observations than the full frame, the estimated variance of the total is
`M^2 * (1-n/M) * s_e^2/n`, where `s_e^2` is the sample variance of residuals
`y_m-f_m`. The HT comparator replaces residuals with observed labour. Values
have units of squared person-hours; standard errors have units of person-hours.
A small observed residual variance does not establish accurate labour measurements, complete
coverage, or useful predictions outside that sample.

The implementation uses checked floating-point arithmetic and compensated sums.
SRS calculations subtract one fixed reference prediction before multiplying
the sample totals by `M/n`, so a large common prediction offset cancels before
weighting; a census returns the observed total directly. General designs retain
their supplied inclusion weights. These numerical choices preserve the
mathematical formula while reducing cancellation; they do not supply an exact
arithmetic guarantee at arbitrary scales or recover distinctions already lost
when input numbers were rounded.

The result estimates total person-hours across the sampling frame. It cannot
serve as an exact observed interval total in the labour-bound calculation. Signed
residuals and negative realised totals are preserved: clipping changes the
estimator and generally introduces bias. Variance can exceed HT variance when
predictions are unhelpful. Known inclusion probabilities do not correct erroneous
allocations or nonresponse. This command computes neither assignment-weighted
causal effects nor ratio confidence intervals; those require their own designs,
dependence information and estimands.
