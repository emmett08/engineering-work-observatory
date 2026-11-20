# Editorial review

The short paper contains 4,900 words on the counting basis below. The previous revision contained 4,585 at `a7134cc`.

In the revision at `aeefae3`, the word “may” fell from three uses to zero. Two occurrences concerned dependent selections and systematic prediction errors. In the revised sampling proof, both cases follow from the same calculation of the expected correction. [Sampling](sampling.tex)

## Terminology and notation

A requirement describes the behaviour the team wants. In this comparison, each task covers the work needed to deliver one requirement. The team can describe it in a GitHub issue using its own Task template. Labour counts time spent implementing, reviewing or repairing the change within the task’s labour window. [Accounting](main.tex) · [Bounds](bounds.tex)

The total $H$ sums labour over the included tasks’ full labour windows. The sampling total $H_{\mathcal I}$ covers the contributors and periods in the chosen sampling population. The dashboard example draws two of the four one-hour intervals available across the implementer and reviewer. [Accounting](main.tex) · [Sampling](sampling.tex)

A person can spend an entire hour on unrelated work. If that hour belongs to the sampling population, it can still be selected and contributes zero task hours. The sampling proof needs each interval in the target population to have a known positive selection probability. [Sampling](sampling.tex)

The allocation calculation finds the smallest and largest labour totals allowed by the observations and each contributor’s available time. The confidence limits in the paired trial apply to the mean effect of AI access on each outcome under the trial assumptions. A schedule specifies who works on which activity and when. In the numerical example, each of the 96 patterns places the implementer’s work and the reviewer’s half-hour in half-hour slots. [Bounds](bounds.tex) · [Numerical example](results.tex) · [Paired trial](workplace.tex)

## Measurement wording

A fair draw determines which team in each pair can use the AI tools. A team with access can complete its tasks without using the tools. Keeping it in its original group preserves the comparison of access to the tools. [Paired trial](workplace.tex)

In the dashboard example, the developer acceleration team labels an issue Ready when its task qualifies and the team has agreed the checks and their weights. That moment sets the start of labour measurement and the cutoff for information used in a forecast. With first acceptance on day three, a 14-day delay puts the follow-up on day 17. A repair on day 18 still falls within the 28 days set aside for counting hours. [Accounting](main.tex) · [Forecasts](model-notes.md)

For an observation of work, the lower and upper values describe the hours that could have produced it, allowing for rounding or other known error. The capacity constraint comes from the time available to the contributor. The sampling section explains what the evidence must cover for the confidence guarantee to hold. [Bounds](bounds.tex) · [Sampling](sampling.tex)

The abstract identifies the break in availability that excludes some totals when a task must run continuously. If an engineer has an hour on each side of a break, a task that needs one continuous hour must fit entirely on one side. [Abstract](main.tex) · [Bounds](bounds.tex)

Two engineers on the developer acceleration team contribute to the dashboard tasks. The implementer builds the namespace filter and repairs the image tag display. The reviewer spends half an hour checking the filter’s test cases against the agreed requirement. That review can take place before the filter is ready to run. First acceptance requires evidence that the implemented filter passes both checks. At follow-up, selecting staging shows both deployments, so only the check for the staging deployment passes. The image tag still reads `v2.4.1`, giving two passing checks out of three. [Accounting](main.tex) · [Sampling](sampling.tex) · [Paired trial](workplace.tex)

## Negative Syndeton

None found.

## Rhetorical contrasts

Two combinations of working time and commit probability during a working minute give the same distribution of commit sequences. The difference in expected minutes of work establishes why the commit times alone cannot identify hours. Across the 96 ways of allocating the implementer’s and reviewer’s time, omitting the reviewer leaves the mean range width at 0.5 person-hours and excludes the actual total in 26 cases. [Introduction](main.tex) · [Results](results.tex)

## Tetracolon

None found.

## Grammatical Parallelism

The earlier trial passage placed several short instructions between the definition of the pair effect and the formula for its uncertainty. The revision puts the fair draw with the team comparison it defines. The margin for each outcome uses the permitted range of differences in each pair. [Paired trial](workplace.tex)

## Syndeton

“closed and bounded” identifies the two properties that guarantee the existence of extrema for the nonempty feasible set. The proof goes on to show which totals between them can occur when contributors can switch activities. [Bounds](bounds.tex)

## Over-elaboration

1. The accounting section defines each trial outcome using the same total of initial requirement weights. Dividing each team’s labour by that total gives hours per initial unit. Dividing its retained score by the same total gives the fraction retained. [Accounting](main.tex)
2. The figure shows how either task can occupy the hour before release. The implementer works for that entire hour in either order. The review adds another half a person-hour. [Capacity example](bounds.tex) · [Figure](shared-time.tex)
3. The expected contribution of a sampled interval explains how the correction recovers total hours on average. The variance comparison then specifies exactly when predictions improve precision. When contributors supply only some of the sampled durations, the second draw selects from the intervals whose hours are still unknown. [Sampling](sampling.tex)
4. The reviewer works half an hour in one of the two hours available for selection. Suppose the duration is known for every interval except the one containing the review. Of the 576 possible draws, 288 omit that interval and supply both durations needed for the estimate. [Results](results.tex)
5. Setting ranges before allocation lets the calculation include a pair with incomplete measurements. In the 48-pair example, the permitted range of labour differences determines how large an observed saving must be to justify a claim. [Paired trial](workplace.tex)

## Word frequency

The counts include the title, keywords, headings, body text, footnotes, captions and words in the figure and table. They exclude mathematical expressions, identifiers, pure numerals, citation markers, URLs and the bibliography. Hyphenated words count once. The ranking below omits common function words, ignores capitalisation and keeps inflections separate. Ties follow alphabetical order.

| Rank | Word | Count | Editorial decision |
| ---: | --- | ---: | --- |
| 1 | hours | 63 | Names measured duration. The contributor and period identify which hours to include in a total. |
| 2 | labour | 42 | Time spent by people on qualifying activities, measured in person-hours. |
| 3 | task | 41 | The work needed to deliver one original requirement in this comparison. |
| 4 | time | 40 | Names elapsed time since the team included the task, or time available for work. |
| 5 | two | 37 | Specifies the number of people, units or periods in an example. |
| 6 | total | 35 | Denotes a sum across the specified tasks, contributors or intervals. |
| 7 | tasks | 26 | Units of delivery included in the comparison. Their labour totals can be summed after allocating shared time once. |
| 8 | hour | 24 | Defines the period or unit in the examples. |
| 9 | filter | 23 | Limits the dashboard to deployments in the selected namespace. |
| 10 | release | 22 | Marks the end of the first hour in the dashboard example. The first labour comparison concerns the hours before it. |
| 11 | reviewer | 22 | In the worked example, the teammate who checks the namespace test cases. The independent reviewers in the trial verify functionality at follow-up. |
| 12 | draw | 20 | Denotes selection of intervals or allocation within a pair. The object identifies which. |
| 13 | person-hours | 20 | The unit for labour aggregated across contributors. |
| 14 | review | 20 | For the first-hour calculation, the time the reviewer spends checking the namespace test cases. |
| 15 | spent | 19 | Links a person’s hours to the activity and period being measured. |
| 16 | team | 19 | Identifies the developer acceleration team in the example or one of the two teams in a randomised pair. |
| 17 | work | 19 | General term for contributors’ effort. The paper counts time on the included tasks. |
| 18 | worked | 19 | Links hours to the person who spent them on the task. |
| 19 | gives | 18 | Names the result of a calculation. A specific verb such as “bounds” identifies its role where needed. |
| 20 | implementer | 18 | The engineer who builds the namespace filter and repairs the image tag display. |
| 21 | initial | 18 | Identifies the requirement weight agreed when the team includes a task or an estimate available before follow-up. |
| 22 | interval | 18 | A period with a start and an end, or a range of values in a confidence calculation. Name the quantity covered in either use. |
| 23 | range | 18 | Names the values allowed for a specified quantity, such as an unknown duration or a difference between teams. |
| 24 | zero | 17 | A measured amount or a mathematical boundary. An unknown amount needs its own range. |
| 25 | dashboard | 16 | Names the tool whose namespace filter and image tag display the team tests. |
| 26 | unit | 16 | Connects the score for a requirement to the behaviour tested at follow-up. |
| 27 | use | 16 | May describe how a team works with its tools or introduce a quantity in a calculation. |
| 28 | contributor | 15 | The person whose time can count towards a task. |
| 29 | count | 15 | Identifies contributions to a labour total, a requirement score or the sample. The object names which quantity is being calculated. |
| 30 | activity | 14 | An action a contributor performs on a task, such as reviewing the namespace filter. |
