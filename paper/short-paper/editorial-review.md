# Editorial review

The short paper contains 4,339 words on the counting basis below. The previous revision contained 3,794 at `34f153a`.

In the revision at `aeefae3`, the word “may” fell from three uses to zero. Two occurrences concerned dependent selections and systematic prediction errors. In the revised sampling proof, both cases follow from the same calculation of the expected correction. [Sampling](sampling.tex)

## Terminology and notation

A requirement describes the behaviour to be checked. The task is the unit of delivery whose time and score enter the comparison. Implementation and review are activities on a task. Labour is the time contributors spend on activities that fall within the accounting rules. [Accounting](main.tex) · [Bounds](bounds.tex)

The total $H$ sums labour over the included tasks’ full labour windows. The sampling total $H_{\mathcal I}$ covers the contributors and periods in the chosen sampling population. In the export example the latter covers a two-hour shift. [Accounting](main.tex) · [Sampling](sampling.tex)

A person can spend a sampled interval on other tasks or away from work. That interval can still be selected, and its contribution to the labour total is zero. The sampling proof needs each interval in the target population to have a known positive selection probability. [Sampling](sampling.tex)

The allocation calculation finds the smallest and largest labour totals allowed by the observations and each contributor’s available time. The confidence limits in the paired trial apply to the mean difference between teams for each outcome. A schedule specifies who works on which activity and when. In the numerical example, each of the 96 patterns places the engineer’s work and the colleague’s review in half-hour slots. [Bounds](bounds.tex) · [Numerical example](results.tex) · [Paired trial](workplace.tex)

## Measurement wording

The accounting section identifies when a task enters the comparison. The scoring weight of its requirement is fixed at entry. Task age is elapsed time from that event, while the follow-up date depends on first acceptance. The forecast notes use entry as the cutoff for information available to a predictor. [Accounting](main.tex) · [Forecasts](model-notes.md)

For an observation of work, the lower and upper values describe the hours that could have produced it, allowing for rounding or other known error. The capacity constraint comes from the time available to the contributor. The sampling section explains what the evidence must cover for the confidence guarantee to hold. [Bounds](bounds.tex) · [Sampling](sampling.tex)

The abstract now identifies the scheduling restriction that excludes some totals within the bounds. If an engineer has an hour on each side of a break, a task that needs one continuous hour must fit entirely on one side. [Abstract](main.tex) · [Bounds](bounds.tex)

The customer order export supplies a requirement readers can follow from acceptance to the later test of the date filter. The time spent implementing and reviewing it also enters the shift total. The shift supplies known work durations for the sampling example. The later test result is hypothetical. [Accounting](main.tex) · [Sampling](sampling.tex) · [Paired trial](workplace.tex)

## Negative Syndeton

None found.

## Rhetorical contrasts

Two activity rates imply the same distribution of commit sequences. The difference in expected minutes of work establishes why the commit times alone cannot identify hours. In the two-person shift example, omitting the reviewer leaves the mean range width at 0.5 person-hours and excludes the actual total in 26 cases. [Introduction](main.tex) · [Results](results.tex)

## Tetracolon

None found.

## Grammatical Parallelism

The earlier trial passage placed several short instructions between the definition of the pair effect and the formula for its uncertainty. The revision puts the fair draw with the team comparison it defines. The margin for each outcome follows from the variation over possible draws. [Paired trial](workplace.tex)

## Syndeton

“closed and bounded” identifies the two properties that guarantee the existence of extrema for the nonempty feasible set. The proof goes on to show which totals between them can occur when contributors can switch activities. [Bounds](bounds.tex)

## Over-elaboration

1. The accounting section defines each trial outcome using the same total of initial requirement weights. Both the worked example and the trial decision can use those units directly. [Accounting](main.tex)
2. The figure shows how either task can occupy the hour before release. In both orders the engineer works the full hour, so the engineer contributes one person-hour before the reviewer’s time is added. [Capacity example](bounds.tex) · [Figure](shared-time.tex)
3. The expected contribution of a sampled interval explains how the correction recovers total hours on average. The variance comparison then specifies exactly when predictions improve precision. For an interval drawn at follow-up, the calculation requires the time the contributor spent on the requirements during that interval. [Sampling](sampling.tex)
4. The reviewer's half-hour of work falls within one of that person's two intervals. Half the draws select that interval. When the reviewer supplies no work time for that interval, only the other 288 draws supply both sampled values. [Results](results.tex)
5. Setting ranges before allocation allows a pair with incomplete measurements to enter the analysis. In the 48-pair example, the permitted range of labour differences determines how large an observed saving must be to justify a claim. [Paired trial](workplace.tex)

## Word frequency

The counts include the title, keywords, headings, prose, captions and words in the figure and table. They exclude mathematical expressions, identifiers, pure numerals, citation markers, URLs and the bibliography. Hyphenated words count once. The ranking below omits common function words, ignores capitalisation and keeps inflections separate. Ties follow alphabetical order.

| Rank | Word | Count | Editorial decision |
| ---: | --- | ---: | --- |
| 1 | hours | 56 | Names measured duration. The contributor and period identify whose hours enter a total. |
| 2 | labour | 42 | Time spent by people on qualifying activities, measured in person-hours. |
| 3 | shift | 33 | The two-hour period over which the worked example counts labour. |
| 4 | total | 33 | Denotes a sum across the specified tasks, contributors or intervals. |
| 5 | task | 31 | The unit of delivery followed from entry, whose labour and retained score enter the comparison. |
| 6 | time | 26 | Qualify by its role, such as elapsed time or time available for work. |
| 7 | two | 26 | Specifies the number of people, units or periods in an example. |
| 8 | hour | 25 | Defines the period or unit in the examples. |
| 9 | person-hours | 20 | The unit for labour aggregated across contributors. |
| 10 | review | 20 | Names work on a change or a check of whether the requirement still holds. |
| 11 | worked | 20 | Connects a duration to the time a contributor spent on a task. |
| 12 | work | 19 | A general expression for what contributors do. The calculation counts time spent on qualifying activities. |
| 13 | draw | 18 | Denotes selection of intervals or allocation within a pair. The object identifies which. |
| 14 | first | 18 | Names the first interval or acceptance, depending on the calculation. |
| 15 | interval | 18 | A period with a start and an end, or a range of values in a confidence calculation. Name the quantity covered in either use. |
| 16 | range | 18 | Names the values allowed for a specified quantity, such as an unknown duration or a difference between teams. |
| 17 | unit | 18 | Connects the score for a requirement to the behaviour tested at follow-up. |
| 18 | contributor | 17 | The person whose time can count towards a task. |
| 19 | export | 16 | Names the feature that exports the customer orders selected by the month filter or the resulting file. |
| 20 | filter | 16 | Identifies the feature that restricts exported customer orders to the requested month. |
| 21 | initial | 16 | The weight fixed when a task enters the comparison or a duration supplied before follow-up. |
| 22 | release | 16 | Marks the midpoint of the example shift. The first labour comparison concerns the hours before it. |
| 23 | tasks | 16 | Units of delivery included in the comparison. Their labour totals can be summed after allocating shared time once. |
| 24 | activity | 15 | An action a contributor performs on a task, such as reviewing the date filter. |
| 25 | zero | 15 | A measured amount or a mathematical boundary. An unknown amount needs its own range. |
| 26 | gives | 14 | Names the result of a calculation. A specific verb such as “bounds” identifies its role where needed. |
| 27 | intervals | 14 | Periods sampled for labour measurement or ranges of uncertainty. Identify which calculation uses them. |
| 28 | team | 14 | Identifies the group whose labour and functionality enter the comparison. |
| 29 | engineer | 13 | Identifies the contributor whose tasks share one time budget. |
| 30 | bound | 12 | Names a minimum, maximum or one-sided guarantee. Identify the quantity it constrains and any error probability. |
