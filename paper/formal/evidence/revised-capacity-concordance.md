# Mathematical review of the revised capacity section

The revised section preserves the claims covered by the existing Lean proofs. I compared its exact text with the previously reviewed article, the frozen proof target and the supplement's semantic map. I reviewed the section independently of its writer.

| Source | SHA-256 |
| --- | --- |
| `article/partial-identification.tex` before this edit, Git `dffe5bedd88d5b31e858ed574e74b39d7e2a5514` | `199218ed24332a441cc8dafe504d4e5771fff77215ebf8dfd1dc7b2199712baa` |
| `article/partial-identification.tex` reviewed here | `87ef4d0968800e56c1cf0561c4371ea2a6c82ae803856fb7c133f6a4cb1adf58` |
| `formal/targets/partial-identification.tex` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |

Both proposition statements match the preceding article byte for byte, as do its nine displayed formulae and eleven labels. I also compared the revised proof paragraphs and their assumptions. The following observations record the grounds for the correspondence.

| Claim | Grounds in the revised text |
| --- | --- |
| Labour before a cutoff | The capacities on either side give the two endpoint inequalities. For an interior cutoff the piecewise constant rates integrate to the required totals. At either endpoint the constant rate across the whole interval avoids division by zero. |
| Reduction to a finite set of cell totals | The partition contains all observation and target boundaries in calendar time. Work streams share one capacity per person in each cell. Integrating an admissible history gives feasible cell totals. Conversely, consecutive blocks of the prescribed lengths realise each feasible vector and preserve fixed cell weights. |
| Attainment | Bounded coordinates and closed constraints make the nonempty feasible set compact. Convex combinations attain intermediate objective values and the block construction realises them as schedules. Arbitrary switching is a premise. Additional sequencing constraints may narrow the attainable range. |
| Supplied bounds and their relaxation | Nonnegative multipliers for the signed objective give an upper bound for the maximum and a lower bound for the minimum. A feasible allocation matching the relevant bound attains the endpoint. Widening observation limits changes the right-hand sides while preserving the constraint matrix and target weights. |
| Coverage | On the event that every observation limit covers its true total, the true cell vector is feasible. Projecting the feasible set then covers the target. The union bound permits dependent errors and the calculation counts empty sets as failures. |
| Productivity | Known nonnegative output and a positive lower labour bound justify reversing the labour endpoints in the ratio. With uncertain output, separate ranges give an enclosure. Exactness requires that each ratio in the enclosure is attainable. |

The historical proof target, Lean declarations, dependency pins and proof-run evidence retain their original bytes. The labour supplement establishes its recorded component results under their assumptions. The existence arguments for extrema and matching dual multipliers, the schedule-block count and correspondence with the Rust implementation retain their prior proof status. This review covers only the named section, including its rewritten proof explanations. OpenAI Codex conducted the independent review as an AI agent.
