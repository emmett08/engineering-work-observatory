# Measuring labour in known work histories

This finite experiment asks when a shared engineer's time narrows a labour bound and when a sampled account of work reproduces the total on average. Both calculations start from observations derived from the same synthetic activity histories. The detailed activities remain hidden from the production commands. The histories supply independent truth for comparison with their answers.

## Reproduction

From the repository root, run the following command with Python 3.9 or later and the pinned Rust toolchain.

```sh
python3 paper/study/labour-measurement/check.py --check
```

The script builds the locked `observatory` executable, including when Cargo uses a custom target directory. To use a binary already built from this revision, append `--binary /path/to/observatory`. The script exercises the public `bound-labour` and `estimate-labour` commands with JSON inputs. It checks the returned totals against the timelines and compares the complete numerical report with [results.json](results.json). Use `--write` to regenerate the report after an intentional change to the design.

## Study population

One engineer implements task A and repairs task B. A second person reviews A. Each works within the same two-hour window, split into four half hours. The engineer spends two slots on A and zero, one or two on B. The reviewer spends exactly one slot on A. The remaining slots contain no cohort work. All distinct orders are included. There are 6, 12 and 6 orders at the three levels of repair work, respectively. Each combines with the reviewer's four possible start times, giving 96 histories. Every summary weights each of the 96 histories equally. Distinct histories can yield identical observable inputs.

The target for the bounds is the two people's combined labour on A and B in the **first hour**. For the sampling experiment it is their combined labour over the **whole two-hour window**. Truth comes from counting occupied slots in the relevant window and multiplying by half an hour.

## Which observation narrows the first-hour range?

The initial observations give the whole-window total separately for each person's work on each task, together with the reviewer's first-hour total. For the relaxed calculation, each of the engineer's two activity streams receives its own capacity of one hour per elapsed hour. For the shared-capacity calculation they draw on the engineer's single time budget. Both versions keep each stream's individual upper limit, every observation and the same target. One additional condition supplies the observed first-hour total across the engineer's two activities while keeping both original stream totals.

A decision concerns whether first-hour labour exceeded one person-hour. A lower bound greater than one establishes that it did. An upper bound of one or less establishes that it did not. A range straddling that boundary leaves the decision unresolved. Counts below use the true labour of both people even when a contributor is hidden.

| Observations for first-hour bounds | Capacity across the engineer's tasks | Histories containing truth | Mean width in person-hours | Correct / wrong / unresolved decisions |
|---|---|---:|---:|---:|
| Complete | Separate per stream | 96/96 | 1.5 | 12 / 0 / 84 |
| Complete | Shared | 96/96 | 0.5 | 60 / 0 / 36 |
| Additional first-hour total across engineer's tasks | Either | 96/96 | 0 | 96 / 0 / 0 |
| Reviewer omitted from the supplied problem | Shared | 70/96 | 0.5 | 70 / 26 / 0 |
| Reviewer's first-hour work entered as zero | Shared | 70/96 | 0.5 | 70 / 26 / 0 |
| Reviewer's first-hour work allowed to occupy zero to half an hour | Shared | 96/96 | 1 | 0 / 0 / 96 |

In the omitted-reviewer case the input contains only the engineer. In the zero-report case the reviewer remains visible and the whole-window review total is still known. A zero in the first hour can always be explained by placing the review in the second hour. Both conditions therefore yield the same first-hour bounds here. The error-bound condition allows every time compatible with the reviewer's known half-hour total. That restores containment at the expense of a decision in all 96 histories. These deterministic fractions describe the enumerated cases. They supply no estimate of how often teams omit reviewers.

## Recovering whole-window labour from sampled intervals

Each person's shift contains two one-hour intervals. The script supplies both person-windows and all four interval boundaries to version 2 of the estimation command. It evaluates all six ways to draw two intervals uniformly without replacement. Each interval has inclusion probability 1/2. A fixed role-based rule predicts 0.75 hours per engineer interval and 0.25 per reviewer interval. No sampled outcome enters these predictions.

| What happens when a selected interval includes review work | Draws yielding estimates | Mean signed error in person-hours, HT / difference | Mean within-history variance in squared person-hours, HT / difference |
|---|---:|---:|---:|
| The reviewer reports their time accurately | 576/576 | 0 / 0 | 49/72 / 25/72 |
| The reviewer reports zero work | 576/576 | −1/2 / −1/2 | 67/72 / 19/72 |
| The reviewer leaves this prompt unanswered | 288/576 | 0 / −1/3 | 2/3 / 2/9 |

HT denotes the Horvitz–Thompson estimator. For each history, the script calculates the mean error against known labour and the variance across equiprobable samples. It averages those summaries across histories. In the final row the averages use only draws for which the command returns an estimate. The Rust implementation explicitly rejects an unanswered selected interval. The other three draws per history have different selection probabilities once analysis is restricted to successful draws. The zero average error for HT in this row is specific to this finite mixture. It supplies no unbiasedness guarantee after non-response.

The benchmark establishes how the implementations behave under these input conditions. Wider variations in shift length, simultaneous reviews, measurement error and response require new cases. Workplace checks must establish how often the conditions arise and whether the recorded inputs cover the people and hours the study intends to measure.
