# Independent review of the revised labour bounds

The revised capacity section preserves both propositions and their assumptions. I compared its proofs with the previous article and the archived mathematical target. I also checked the examples on working conditions and the shortest-path argument in the appendix.

| Source | SHA-256 |
| --- | --- |
| Archived capacity target, `targets/partial-identification.tex`, from `38c4ce40b789d4c5420c98ff1d568b39c3b1e798` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |
| Capacity section at `6b89ad473d77d2b59edaf5d44045318284b4346a` | `348abe10110a0441b83866682cdcec0df49e00c8b4b721df8c789ba4860cdb87` |
| Revised `paper/article/partial-identification.tex` | `2a0a805ead9bdf6a24c5a9233d342034e76408819b0b199013c52b0155d0a294` |
| Revised `paper/article/interpreting-capacity.tex` | `9f68de01151f5920412d025ce9f752a5a7297a3a853858cdfeb00809367f2e7c` |
| Revised `paper/article/scalar-bounds.tex` | `f1032141003b78c4ee941861b1260ff113618add53238b3600ce850bc82176d3` |

All nine mathematical displays in the capacity section match the preceding article, as do its eleven labels, twelve cross-references and seven citation commands. The section includes all seven displays in the archived target. The first proposition is unchanged from the preceding article. The second has revised wording.

| Result | Check |
| --- | --- |
| Single cutoff | Capacity on each side gives both inequalities. For an interior cutoff the two allocation fractions give any admissible split. The constant fraction over the whole interval handles either boundary, including when the total is zero or equals the interval duration. |
| Ranges for several people or tasks | All intervals use the same finite partition. Each person's nonnegative cell contributions sum to at most the cell duration. Integration recovers the constraints. Consecutive blocks reproduce each feasible vector, including zero work or idle time, and preserve grouped observations and weighted targets. Adding separate task extrema gives the corresponding combined extremum precisely when one feasible schedule gives both. |
| Both endpoints and intermediate totals | For a nonempty set of feasible cell totals, compactness gives the extrema of the linear objective. The segment from a minimiser to a maximiser contains a vector for every intermediate value. Each vector has its own schedule with at most one activity per person at any instant. Added restrictions on scheduling can narrow the range. The purification remark retains the finite collection of atomless measures. |
| Certificates and coverage | Feasible primal and dual quantities with equal objective values establish an endpoint. Widening the observation limits at fixed coefficients preserves the inequality from each supplied multiplier. If all observation limits cover their true totals together, the true history is feasible under the allocation assumptions. The resulting target-coverage bound counts empty sets as failures. |
| Rates and working conditions | For fixed nonnegative output, a positive labour minimum bounds the rate from above. If labour ranges down to zero, positive output gives arbitrarily high rates on positive denominators. In the variable-output example each feasible pair gives two units per person-hour, inside the wider enclosure from the separate limits. The two-reviewer example places the first review wholly within the first hour by specifying the departure time. |
| Scalar appendix | Summing along a path bounds the cumulative difference. In the absence of negative cycles, shortest-path distances from a fixed vertex, shifted to give zero initial hours, yield feasible cell totals. Choosing either endpoint as the starting vertex gives the upper or lower bound with the sign shown. Convex interpolation covers the values between them. |

The exact-rational identification checks and finite measurement examples passed. The [archived correspondence map](../semantic-map.md) identifies the Lean results on cell integrals, serial schedules, supplied certificates and the transfer of coverage. Compactness, the existence of matching dual multipliers and the numerical block limit rely on arguments in the manuscript. The historical receipt covers its listed Lean components. This editorial comparison included no fresh kernel run, verification of the Rust solver or test on workplace measurements.
