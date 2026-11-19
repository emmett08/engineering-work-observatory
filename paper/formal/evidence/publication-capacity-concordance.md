# Independent review of the capacity section for publication

**Outcome.** The revised section preserves both propositions and the qualifications that connect the engineering interpretation to the archived Lean proofs. An independent OpenAI Codex review agent compared the following three sources.

| Source | SHA-256 |
| --- | --- |
| Archived proof target, `paper/formal/targets/partial-identification.tex`, originating at `38c4ce40b789d4c5420c98ff1d568b39c3b1e798` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |
| Preceding manuscript, `paper/article/partial-identification.tex` at `07df68c8990b9cdb7c7cd2af133918bf60f1e916` | `87ef4d0968800e56c1cf0561c4371ea2a6c82ae803856fb7c133f6a4cb1adf58` |
| Revised manuscript, `paper/article/partial-identification.tex` | `b21a50c8e10a4b5d0822af0b8193fdf919411a8571d5fd3b26f63eee35a42064` |

The two proposition environments are byte-identical in all three sources. Against the preceding manuscript, all nine displayed mathematics environments, eleven labels, twelve cross-references and seven citation commands match in order. The archived target's seven displayed environments appear unchanged in the revised section. The two additional displays already appeared in the preceding manuscript and work through the interior-cutoff construction.

| Mathematical point | Independent assessment of the revised proof and interpretation |
| --- | --- |
| Time before a cutoff | Nonnegativity and the elapsed time on either side give the two inequalities. The piecewise constant allocation attains any admissible split for an interior cutoff. Using a constant allocation over the whole period handles either endpoint without division by zero. |
| Calendar, observations and targets | All time boundaries share one calendar. Summing over whole cells and the specified streams recovers each observation, including observations spanning people. Each individual has a separate capacity constraint. Computing the target requires the contributors and attribution rules for the population of interest. |
| Attainable schedules | Each feasible cell vector admits a schedule of consecutive blocks with the prescribed lengths. Distinct people may work simultaneously. For each person the constructed schedule uses at most one activity at a time. Compactness gives extrema when the set is nonempty, and linearity on the segment between the optimisers gives every intermediate value. Additional workplace constraints can narrow that range. |
| Endpoint and sensitivity checks | A feasible schedule at the proposed value proves attainability. A matching bound obtained from feasible dual multipliers proves optimality. Relaxing only the observation limits preserves dual feasibility, so the original multipliers still provide an enclosure. |
| Sampling and productivity | Simultaneous coverage of the true observation totals implies coverage of the target under the allocation constraints. The union bound permits dependent observation errors and an empty feasible set counts as a miss. The fixed-output ratio requires positive minimum labour. For uncertain output, attaining every rate requires compatible output and labour values. |
| Scope of verification | The historical Lean checks cover integral and cell identities, the per-person schedule construction, supplied multipliers and the simultaneous-coverage implication. The manuscript gives the existence arguments for extrema and matching multipliers and an upper bound on the number of blocks. The refinement of the Rust implementation against the Lean definitions remains open. |

I inspected the revised proof explanations and the [formal correspondence map](../semantic-map.md) independently of the writer. I asked the writer to clarify three points in an earlier draft: the population attached to overlapping observations, equality between a proposed endpoint and the objective of its witness schedule, and the distinction between per-person capacity and the upper bound on block count. I checked those changes in the revised source identified above.

The exact identification examples and the separate finite model checks passed. The archived target, Lean declarations, dependency pins and verification scripts match the preceding commit. This assessment concerns manuscript concordance. It makes no claim of a fresh Lean execution or of workplace measurement accuracy.
