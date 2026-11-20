# Editorial review

The short paper contains 3,222 words on the counting basis below. The previous revisions contained 3,278 words at `6f270af` and 2,993 at `aeefae3`.

In the revision at `aeefae3`, the word “may” fell from three uses to zero. Two occurrences concerned dependent selections and systematic prediction errors. In the revised sampling proof, both cases follow from the same calculation of the expected correction. [Sampling](sampling.tex)

## Measurement wording

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
| 1 | hours | 50 | Names measured duration. The contributor and period identify whose hours enter a total. |
| 2 | total | 28 | Denotes a sum across the specified tasks, contributors or intervals. |
| 3 | labour | 26 | Names the human effort counted for the included requirements. |
| 4 | interval | 21 | Denotes either a period sampled or the limits on a quantity. Its endpoints identify which. |
| 5 | shift | 19 | Defines the observation period in the examples. |
| 6 | time | 18 | Qualify by its role, such as elapsed time or time available for work. |
| 7 | limit | 17 | Identifies a capacity restriction or a confidence limit. The surrounding noun distinguishes them. |
| 8 | hour | 16 | Defines the period or unit in the examples. |
| 9 | two | 16 | Specifies the number of people, schedules, units or time periods in an example. |
| 10 | gives | 14 | Names the result of a calculation. A specific verb such as “bounds” identifies its role where needed. |
| 11 | probability | 14 | Refers to a defined event, such as inclusion in a sample or exclusion of a true difference. |
| 12 | work | 14 | Names eligible activity. Retain the activity name where the example distinguishes implementation from repair. |
| 13 | worked | 14 | Connects a measured duration to the time someone spent on the requirements. |
| 14 | review | 13 | Names work on a change or a check of whether the requirement still holds. |
| 15 | mean | 12 | Names an average. Specify whether it is over draws, schedules or pairs. |
| 16 | person-hour | 12 | The singular unit used at a decision threshold. |
| 17 | person-hours | 12 | The unit for labour aggregated across contributors. |
| 18 | task | 12 | Identifies the requirement or activity to which the hours belong. |
| 19 | zero | 12 | A measured amount or a mathematical boundary. An unknown amount needs its own range. |
| 20 | first | 11 | Distinguishes first acceptance, the first interval or the first hour. |
| 21 | observations | 11 | Names the evidence that constrains the possible schedules. |
| 22 | range | 11 | Names the possible values permitted by observations or fixed before allocation. |
| 23 | schedule | 11 | Specifies which contributor worked in each half-hour slot. |
| 24 | activity | 10 | The indexed quantity carries both contributor and task. |
| 25 | draw | 10 | Denotes selection of intervals or allocation within a pair. The object identifies which. |
| 26 | duration | 10 | Identifies the time a contributor worked within a sampled interval. The interval itself has a fixed length. |
| 27 | engineer | 10 | Identifies the contributor with a shared time budget. |
| 28 | initial | 10 | Identifies the intake score or the first round of replies. |
| 29 | sampled | 10 | Identifies the intervals chosen for measurement or the work time in those intervals. |
| 30 | sampling | 10 | Identifies the source of variation across draws. |
