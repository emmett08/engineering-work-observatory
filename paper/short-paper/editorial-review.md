# Editorial review

The short paper contains 4,407 words on the counting basis below. The previous revision contained 4,339 at `1fd1c6e`.

In the revision at `aeefae3`, the word “may” fell from three uses to zero. Two occurrences concerned dependent selections and systematic prediction errors. In the revised sampling proof, both cases follow from the same calculation of the expected correction. [Sampling](sampling.tex)

## Terminology and notation

A requirement describes the behaviour to be checked. A task groups the activities needed to deliver one requirement in this comparison. Labour includes a contributor’s time spent implementing or reviewing that behaviour within the task’s labour window. [Accounting](main.tex) · [Bounds](bounds.tex)

The total $H$ sums labour over the included tasks’ full labour windows. The sampling total $H_{\mathcal I}$ covers the contributors and periods in the chosen sampling population. The dashboard example draws two of the four one-hour intervals available across the engineer and reviewer. [Accounting](main.tex) · [Sampling](sampling.tex)

A person can spend a sampled interval on other tasks or away from work. That interval can still be selected, and its contribution to the labour total is zero. The sampling proof needs each interval in the target population to have a known positive selection probability. [Sampling](sampling.tex)

The allocation calculation finds the smallest and largest labour totals allowed by the observations and each contributor’s available time. The confidence limits in the paired trial apply to the mean difference between teams for each outcome. A schedule specifies who works on which activity and when. In the numerical example, each of the 96 patterns places the engineer’s work and the colleague’s review in half-hour slots. [Bounds](bounds.tex) · [Numerical example](results.tex) · [Paired trial](workplace.tex)

## Measurement wording

When a task enters the comparison, the team specifies how to test its requirement and how much each passing check will count. Task age runs from entry. The follow-up date depends on first acceptance. The forecast notes use entry as the cutoff for information available to a predictor. [Accounting](main.tex) · [Forecasts](model-notes.md)

For an observation of work, the lower and upper values describe the hours that could have produced it, allowing for rounding or other known error. The capacity constraint comes from the time available to the contributor. The sampling section explains what the evidence must cover for the confidence guarantee to hold. [Bounds](bounds.tex) · [Sampling](sampling.tex)

The abstract identifies the break in availability that excludes some totals when a task must run continuously. If an engineer has an hour on each side of a break, a task that needs one continuous hour must fit entirely on one side. [Abstract](main.tex) · [Bounds](bounds.tex)

A developer acceleration team adds a namespace filter to its Kubernetes rollout dashboard and repairs the way it displays a deployed image tag. At follow-up, selecting staging returns both test deployments. Only the staging deployment should appear. The dashboard still shows its full image tag, `v2.4.1`, so two of the three checks pass. The example uses known durations to check the sampling calculation and a hypothetical test result to explain how much functionality the team retains. [Accounting](main.tex) · [Sampling](sampling.tex) · [Paired trial](workplace.tex)

## Negative Syndeton

None found.

## Rhetorical contrasts

Two activity rates imply the same distribution of commit sequences. The difference in expected minutes of work establishes why the commit times alone cannot identify hours. Across the 96 ways of allocating the engineer’s and reviewer’s time, omitting the reviewer leaves the mean range width at 0.5 person-hours and excludes the actual total in 26 cases. [Introduction](main.tex) · [Results](results.tex)

## Tetracolon

None found.

## Grammatical Parallelism

The earlier trial passage placed several short instructions between the definition of the pair effect and the formula for its uncertainty. The revision puts the fair draw with the team comparison it defines. The margin for each outcome follows from the variation over possible draws. [Paired trial](workplace.tex)

## Syndeton

“closed and bounded” identifies the two properties that guarantee the existence of extrema for the nonempty feasible set. The proof goes on to show which totals between them can occur when contributors can switch activities. [Bounds](bounds.tex)

## Over-elaboration

1. The accounting section defines each trial outcome using the same total of initial requirement weights. Both the worked example and the trial decision can use those units directly. [Accounting](main.tex)
2. The figure shows how either task can occupy the hour before release. The engineer works for that entire hour in either order. The review adds another half a person-hour. [Capacity example](bounds.tex) · [Figure](shared-time.tex)
3. The expected contribution of a sampled interval explains how the correction recovers total hours on average. The variance comparison then specifies exactly when predictions improve precision. For an interval drawn at follow-up, the calculation requires the time the contributor spent on the requirements during that interval. [Sampling](sampling.tex)
4. The reviewer's half-hour of work falls within one of that person's two intervals. Half the draws select that interval. When the reviewer supplies no work time for that interval, only the other 288 draws supply both sampled values. [Results](results.tex)
5. Setting ranges before allocation allows a pair with incomplete measurements to enter the analysis. In the 48-pair example, the permitted range of labour differences determines how large an observed saving must be to justify a claim. [Paired trial](workplace.tex)

## Word frequency

The counts include the title, keywords, headings, prose, captions and words in the figure and table. They exclude mathematical expressions, identifiers, pure numerals, citation markers, URLs and the bibliography. Hyphenated words count once. The ranking below omits common function words, ignores capitalisation and keeps inflections separate. Ties follow alphabetical order.

| Rank | Word | Count | Editorial decision |
| ---: | --- | ---: | --- |
| 1 | hours | 61 | Names measured duration. The contributor and period identify whose hours enter a total. |
| 2 | labour | 40 | Time spent by people on qualifying activities, measured in person-hours. |
| 3 | task | 36 | The unit of delivery followed from entry, whose labour and retained score enter the comparison. |
| 4 | time | 34 | Names elapsed time since entry or time available for work, depending on the calculation. |
| 5 | total | 30 | Denotes a sum across the specified tasks, contributors or intervals. |
| 6 | two | 29 | Specifies the number of people, units or periods in an example. |
| 7 | hour | 24 | Defines the period or unit in the examples. |
| 8 | filter | 21 | Limits the dashboard to deployments in the selected namespace. |
| 9 | release | 21 | Marks the end of the first hour in the dashboard example. The first labour comparison concerns the hours before it. |
| 10 | person-hours | 20 | The unit for labour aggregated across contributors. |
| 11 | tasks | 20 | Units of delivery included in the comparison. Their labour totals can be summed after allocating shared time once. |
| 12 | draw | 19 | Denotes selection of intervals or allocation within a pair. The object identifies which. |
| 13 | range | 18 | Names the values allowed for a specified quantity, such as an unknown duration or a difference between teams. |
| 14 | unit | 18 | Connects the score for a requirement to the behaviour tested at follow-up. |
| 15 | worked | 18 | Links hours to the person who spent them on the task. |
| 16 | contributor | 17 | The person whose time can count towards a task. |
| 17 | interval | 17 | A period with a start and an end, or a range of values in a confidence calculation. Name the quantity covered in either use. |
| 18 | dashboard | 16 | Names the tool whose namespace filter and image tag display the team tests. |
| 19 | initial | 16 | Identifies the weight set when a task enters or an estimate available before follow-up. |
| 20 | reviewer | 16 | The second contributor, whose half-hour of review counts towards dashboard labour. |
| 21 | work | 16 | General term for contributors’ effort. The paper counts time on the included tasks. |
| 22 | activity | 15 | An action a contributor performs on a task, such as reviewing the namespace filter. |
| 23 | gives | 15 | Names the result of a calculation. A specific verb such as “bounds” identifies its role where needed. |
| 24 | review | 15 | Names work on a change or a check of whether the requirement still holds. |
| 25 | engineer | 14 | Identifies the contributor whose tasks share one time budget. |
| 26 | zero | 14 | A measured amount or a mathematical boundary. An unknown amount needs its own range. |
| 27 | bound | 13 | Names a minimum, maximum or one-sided guarantee. Identify the quantity it constrains and any error probability. |
| 28 | duration | 13 | Time spent on an activity or length of a specified period. The text identifies which. |
| 29 | first | 13 | Identifies the first hour or the first acceptance of a task. |
| 30 | intervals | 13 | Periods sampled for labour measurement or ranges of uncertainty. Identify which calculation uses them. |
