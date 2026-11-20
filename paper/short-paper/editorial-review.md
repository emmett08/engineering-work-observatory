# Editorial review

The short paper contains 3,484 words on the counting basis below. Earlier counts were 3,278 at `6f270af`, 2,993 at `aeefae3` and 3,222 at `2d572b1`.

In the revision at `aeefae3`, the word “may” fell from three uses to zero. Two occurrences concerned dependent selections and systematic prediction errors. In the revised sampling proof, both cases follow from the same calculation of the expected correction. [Sampling](sampling.tex)

## Measurement wording

The accounting section identifies when a requirement enters the comparison. Its weight is fixed at entry. Task age is elapsed time from that event, while the follow-up date depends on first acceptance. The forecast notes use entry as the cutoff for information available to a predictor. [Accounting](main.tex) · [Forecasts](model-notes.md)

For an observation of work, the lower and upper values describe the hours that could have produced it, allowing for rounding or other known error. The capacity constraint comes from the time available to the contributor. The sampling section explains what the evidence must cover for the confidence guarantee to hold. [Bounds](bounds.tex) · [Sampling](sampling.tex)

The abstract now identifies the scheduling restriction that excludes some totals within the bounds. If an engineer has an hour on each side of a break, a task that needs one continuous hour must fit entirely on one side. [Abstract](main.tex) · [Bounds](bounds.tex)

The sampling example identifies whose time enters each estimate. We know from the schedule how long each contributor worked during the selected interval. This allows us to compare each estimate with the two contributors’ combined hours over the shift. [Sampling](sampling.tex) · [Results](results.tex)

## Negative Syndeton

None found.

## Contrastive Framing

Two activity rates imply the same distribution of commit sequences. The difference in expected minutes of work establishes why the commit times alone cannot identify hours. In the schedule comparison, omitting the reviewer leaves the mean range width at 0.5 person-hours yet excludes the actual total in 26 schedules. [Introduction](main.tex) · [Results](results.tex)

## Tetracolon

None found.

## Grammatical Parallelism

The earlier trial passage placed several short instructions between the definition of the pair effect and the formula for its uncertainty. The revision puts the fair draw with the team comparison it defines. The radius can then follow from the variation over possible draws. [Paired trial](workplace.tex)

## Syndeton

“closed and bounded” identifies the two properties that guarantee the existence of extrema for the nonempty feasible set. The proof goes on to show which totals between them a schedule can produce. [Bounds](bounds.tex)

## Over-elaboration

1. The accounting section defines each trial outcome using the same total of initial requirement weights. Both the worked example and the trial decision can use those units directly. [Accounting](main.tex)
2. The figure shows which job occupies the first hour under each schedule. The surrounding paragraph explains why adding separate task maxima would count the engineer’s time twice. [Capacity example](bounds.tex) · [Figure](shared-time.tex)
3. The expected contribution of a sampled interval explains how the correction recovers total hours on average. The variance comparison then specifies exactly when predictions improve precision. For an interval drawn at follow-up, the calculation requires the time the contributor spent on the requirements during that interval. [Sampling](sampling.tex)
4. The reviewer's half-hour of work falls within one of that person's two intervals. Half the draws select that interval. When the reviewer supplies no work time for that interval, only the other 288 draws supply both sampled values. [Results](results.tex)
5. Setting ranges before allocation allows a pair with incomplete measurements to enter the analysis. In the 48-pair example, the permitted range of labour differences determines how large an observed saving must be to justify a claim. [Paired trial](workplace.tex)

## Word frequency

The counts include the title, keywords, headings, prose, captions and words in the figure and table. They exclude mathematical expressions, identifiers, pure numerals, citation markers, URLs and the bibliography. Hyphenated words count once. The ranking below omits common function words, ignores capitalisation and keeps inflections separate. Ties follow alphabetical order.

| Rank | Word | Count | Editorial decision |
| ---: | --- | ---: | --- |
| 1 | hours | 53 | Names measured duration. The contributor and period identify whose hours enter a total. |
| 2 | total | 32 | Denotes a sum across the specified tasks, contributors or intervals. |
| 3 | labour | 26 | Names the human effort counted for the included requirements. |
| 4 | interval | 21 | For time, names a period with a start and an end. For hours worked, names the values between a lower and an upper bound. |
| 5 | shift | 21 | Defines the observation period in the examples. |
| 6 | hour | 20 | Defines the period or unit in the examples. |
| 7 | time | 20 | Qualify by its role, such as elapsed time or time available for work. |
| 8 | two | 20 | Specifies the number of people, schedules, units or time periods in an example. |
| 9 | worked | 20 | Connects a duration to the time a contributor spent on the requirements. |
| 10 | range | 18 | Keep the endpoints with the quantity they bound, such as hours worked within a shift or the difference between two teams. |
| 11 | gives | 17 | Names the result of a calculation. A specific verb such as “bounds” identifies its role where needed. |
| 12 | mean | 17 | Names an average. Specify whether it is over draws, schedules or pairs. |
| 13 | work | 16 | Names eligible activity. Retain the activity name where the example distinguishes implementation from repair. |
| 14 | limit | 15 | Names what restricts the hours or the effect. A contributor’s available time caps labour. A one-sided confidence limit accounts for variation across random draws. |
| 15 | task | 15 | Identifies the requirement or activity to which the hours belong. |
| 16 | review | 14 | Names work on a change or a check of whether the requirement still holds. |
| 17 | first | 13 | Distinguishes first acceptance, the first interval or the first hour. |
| 18 | person-hours | 13 | The unit for labour aggregated across contributors. |
| 19 | probability | 13 | Refers to a defined event, such as inclusion in a sample or exclusion of a true difference. |
| 20 | draw | 12 | Denotes selection of intervals or allocation within a pair. The object identifies which. |
| 21 | initial | 12 | Names the weight fixed when a requirement enters the comparison or a duration supplied before follow-up. |
| 22 | person-hour | 12 | The singular unit used at a decision threshold. |
| 23 | sampled | 12 | Names the intervals selected for measurement or the hours worked in them. |
| 24 | schedule | 12 | Specifies which contributor worked in each half-hour slot. |
| 25 | zero | 12 | A measured amount or a mathematical boundary. An unknown amount needs its own range. |
| 26 | duration | 11 | Names the time a contributor spent on the requirements within an interval. |
| 27 | estimate | 11 | Names the quantity calculated from a sample. Identify the total or rate it estimates. |
| 28 | observations | 11 | Names the evidence that constrains the possible schedules. |
| 29 | requirement | 11 | Names the requested behaviour whose acceptance and later operation the team checks. |
| 30 | activity | 10 | The indexed quantity carries both contributor and task. |
