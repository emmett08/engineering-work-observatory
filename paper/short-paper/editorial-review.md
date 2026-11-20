# Editorial review

The short paper contains 3,794 words on the counting basis below. The previous revision contained 3,484 at `0fe0769`.

In the revision at `aeefae3`, the word “may” fell from three uses to zero. Two occurrences concerned dependent selections and systematic prediction errors. In the revised sampling proof, both cases follow from the same calculation of the expected correction. [Sampling](sampling.tex)

## Measurement wording

The accounting section identifies when a requirement enters the comparison. Its weight is fixed at entry. Task age is elapsed time from that event, while the follow-up date depends on first acceptance. The forecast notes use entry as the cutoff for information available to a predictor. [Accounting](main.tex) · [Forecasts](model-notes.md)

For an observation of work, the lower and upper values describe the hours that could have produced it, allowing for rounding or other known error. The capacity constraint comes from the time available to the contributor. The sampling section explains what the evidence must cover for the confidence guarantee to hold. [Bounds](bounds.tex) · [Sampling](sampling.tex)

The abstract now identifies the scheduling restriction that excludes some totals within the bounds. If an engineer has an hour on each side of a break, a task that needs one continuous hour must fit entirely on one side. [Abstract](main.tex) · [Bounds](bounds.tex)

The date filter gives the reader a requirement to follow from the first acceptance check to the later test. The time spent implementing and reviewing it also enters the shift total. The shift supplies known work durations for the sampling example. The later test result is hypothetical. [Accounting](main.tex) · [Sampling](sampling.tex) · [Paired trial](workplace.tex)

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
2. The figure shows how either task can occupy the hour before release. Both schedules use that hour in full, so the engineer contributes one person-hour before the reviewer’s time is added. [Capacity example](bounds.tex) · [Figure](shared-time.tex)
3. The expected contribution of a sampled interval explains how the correction recovers total hours on average. The variance comparison then specifies exactly when predictions improve precision. For an interval drawn at follow-up, the calculation requires the time the contributor spent on the requirements during that interval. [Sampling](sampling.tex)
4. The reviewer's half-hour of work falls within one of that person's two intervals. Half the draws select that interval. When the reviewer supplies no work time for that interval, only the other 288 draws supply both sampled values. [Results](results.tex)
5. Setting ranges before allocation allows a pair with incomplete measurements to enter the analysis. In the 48-pair example, the permitted range of labour differences determines how large an observed saving must be to justify a claim. [Paired trial](workplace.tex)

## Word frequency

The counts include the title, keywords, headings, prose, captions and words in the figure and table. They exclude mathematical expressions, identifiers, pure numerals, citation markers, URLs and the bibliography. Hyphenated words count once. The ranking below omits common function words, ignores capitalisation and keeps inflections separate. Ties follow alphabetical order.

| Rank | Word | Count | Editorial decision |
| ---: | --- | ---: | --- |
| 1 | hours | 57 | Names measured duration. The contributor and period identify whose hours enter a total. |
| 2 | total | 32 | Denotes a sum across the specified tasks, contributors or intervals. |
| 3 | labour | 28 | Names the human effort counted for the included requirements. |
| 4 | hour | 25 | Defines the period or unit in the examples. |
| 5 | two | 23 | Specifies the number of people, schedules, units or time periods in an example. |
| 6 | shift | 22 | Defines the observation period in the examples. |
| 7 | worked | 21 | Connects a duration to the time a contributor spent on the requirements. |
| 8 | time | 20 | Qualify by its role, such as elapsed time or time available for work. |
| 9 | work | 20 | Names eligible activity. Retain the activity name where the example distinguishes implementation from repair. |
| 10 | interval | 19 | For time, names a period with a start and an end. For hours worked, names the values between a lower and an upper bound. |
| 11 | mean | 18 | Names an average. Specify whether it is over draws, schedules or pairs. |
| 12 | person-hours | 18 | The unit for labour aggregated across contributors. |
| 13 | range | 18 | Keep the endpoints with the quantity they bound, such as hours worked within a shift or the difference between two teams. |
| 14 | review | 17 | Names work on a change or a check of whether the requirement still holds. |
| 15 | task | 16 | Identifies the requirement or activity to which the hours belong. |
| 16 | limit | 15 | Names what restricts the hours or the effect. A contributor’s available time caps labour. A one-sided confidence limit accounts for variation across random draws. |
| 17 | export | 14 | Names the file containing the invoices selected by the filter or the operation that produces it. |
| 18 | gives | 14 | Names the result of a calculation. A specific verb such as “bounds” identifies its role where needed. |
| 19 | team | 14 | Identifies the group whose labour and functionality enter the comparison. |
| 20 | draw | 13 | Denotes selection of intervals or allocation within a pair. The object identifies which. |
| 21 | engineer | 13 | Identifies the contributor whose tasks share one time budget. |
| 22 | first | 13 | Distinguishes first acceptance, the first interval or the first hour. |
| 23 | initial | 13 | Names the weight fixed when a requirement enters the comparison or a duration supplied before follow-up. |
| 24 | probability | 13 | Refers to a defined event, such as inclusion in a sample or exclusion of a true difference. |
| 25 | schedule | 13 | Specifies which contributor worked in each half-hour slot. |
| 26 | zero | 13 | A measured amount or a mathematical boundary. An unknown amount needs its own range. |
| 27 | release | 12 | Marks the midpoint of the shift in the invoice example. The first labour comparison concerns the time before it. |
| 28 | unit | 12 | Connects the score for a requirement to the behaviour tested at follow-up. |
| 29 | duration | 11 | Names the time a contributor spent on the requirements within an interval. |
| 30 | observations | 11 | Names the evidence that constrains the possible schedules. |
