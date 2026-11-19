# Independent comparison of the two capacity propositions

The revised propositions describe the same sets of possible labour totals as the archived proof target. Both propositions have new wording. Their hypotheses, displayed equations and conclusions agree with the previous manuscript.

| Source compared | SHA-256 |
| --- | --- |
| Archived target, `targets/partial-identification.tex`, from commit `38c4ce40b789d4c5420c98ff1d568b39c3b1e798` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |
| Capacity section at pull request commit `ec2491e72fc5f9755ea7a48f41d4cf25019a1719` | `b21a50c8e10a4b5d0822af0b8193fdf919411a8571d5fd3b26f63eee35a42064` |
| Revised `paper/article/partial-identification.tex` | `348abe10110a0441b83866682cdcec0df49e00c8b4b721df8c789ba4860cdb87` |

I compared the proofs independently of the author of the edits. All nine displays, eleven labels, twelve cross-references and seven citation commands match the preceding manuscript in order. The seven displays in the archived target also occur in the revised text.

| Claim | Reason it follows |
| --- | --- |
| Hours before a cutoff | Each side must have enough time for its share of the given total. For an interior cutoff, the two constant fractions in the proof give every split within the resulting limits. At either boundary, the constant fraction over the whole interval gives the unique split and avoids division by zero. |
| Finite cell totals | The task, contributor and activity sets are finite. Every observation and target starts and ends at a calendar boundary. Integrating each person's capacity constraint gives the cell inequalities. Summing cell contributions over the specified streams reproduces observations across tasks or people. Conversely, consecutive blocks of the prescribed lengths fit in each person's cell and reproduce its totals. Different people can work at the same time. |
| Minimum, maximum and all intermediate totals | The nonempty feasible set is closed and bounded in finite dimensions. A continuous linear target therefore has a minimiser and a maximiser. Convexity keeps the segment between them feasible, and linearity gives every intermediate total. At each point on the segment, a binary schedule reproduces those cell totals. A single-point range satisfies the same argument. An empty set indicates inconsistent constraints. |
| Limits imposed by shared time | The sum of two individual maxima equals the maximum combined total exactly when one feasible schedule gives both maxima. The corresponding equality for minima requires a schedule giving both minima. Further workplace restrictions may exclude all schedules at a proposed endpoint. |
| Endpoint evidence and observation error | A schedule gives the proposed maximum or minimum when its objective equals a valid upper or lower bound, respectively. Nonnegative dual multipliers give these bounds. They still give an enclosure after observation limits widen at fixed coefficients and weights. Coverage transfers from simultaneous coverage of the true observations when the true history satisfies the allocation assumptions. An empty feasible set counts as a coverage failure. |
| Output per person-hour | For fixed nonnegative output and positive minimum labour, continuity gives the whole ratio interval, including its endpoints. If output may vary with labour, the separate limits enclose every rate. Equality with that interval requires a feasible pair at every rate. |

I checked the distinction between a schedule obeying a lower bound and a schedule whose objective equals that bound, and the requirement that one schedule give both task minima or both maxima. The revised text makes both conditions explicit.

The [formal correspondence map](../semantic-map.md) identifies the archived Lean results on cell integrals, binary schedules, supplied dual multipliers and the transfer of coverage. The manuscript supplies the compactness argument, the appeal to linear programming duality for existence of matching multipliers and the count of blocks per cell. The Rust solver's correspondence with the Lean definitions remains an open question. I ran the repository's exact-rational identification examples and finite measurement examples. Both passed. This review compares manuscript meaning with the archived proof scope and includes no new Lean run or field measurements.
