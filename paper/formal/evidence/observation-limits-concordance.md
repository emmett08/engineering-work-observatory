# Independent review of the wording on labour observations

I compared the revised capacity section with the preceding article and the archived mathematical target. Both proposition environments, all nine mathematical displays, eleven labels, twelve cross-references and seven citation commands match the preceding article byte for byte in their original order.

| Source | SHA-256 |
| --- | --- |
| Archived target, `targets/partial-identification.tex`, from `38c4ce40b789d4c5420c98ff1d568b39c3b1e798` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |
| Capacity section at `969f9acd1e0bff098085459b931c656ed47877ed` | `2a0a805ead9bdf6a24c5a9233d342034e76408819b0b199013c52b0155d0a294` |
| Revised `paper/article/partial-identification.tex` | `44e25dac263096215d61ebc7d57473b70dcd6b556d951c9421b680639e4da6cb` |
| Revised `paper/article/interpreting-capacity.tex` | `fd98c9289cf4a5f207d5c7a7dda1e3f43ae78e2184f864ede62dcbe8fc29765b` |
| Revised `paper/article/scalar-bounds.tex` | `41c7caec368c13150271c9740b27257955e8567fe32e364e07a8402968e7d33a` |

The revised account of rounding includes all underlying totals that could produce the reported value under the rounding rule. Additional measurement error can require wider limits. Each interval and its contributing streams determine which actual total the limits must cover. If all limits cover these totals together and the actual history respects each contributor's capacity, its cell vector is feasible. The target interval then contains the actual labour. The probability guarantee still depends on valid simultaneous coverage.

The revised proof gives a binary schedule for each feasible array of cell totals. A linear target takes every value between its minimum and maximum along the segment joining the two optimisers. Distinct schedules may therefore witness distinct target values. A schedule that obeys all supplied limits shows one possible timing of the work. If multiple sequences satisfy the same observations, those observations alone cannot identify the sequence that occurred.

In the scalar appendix, path sums cancel cumulative labour at intermediate calendar boundaries. The sign of the return distance and the allocation witnesses at both endpoints are unchanged. In the productivity example, every feasible pair gives two reference units per person-hour. With a positive denominator, minimising or maximising the ratio subject to linear constraints is a linear-fractional optimisation problem.

The [preceding independent review](revised-proofs-concordance.md) and the [correspondence map](../semantic-map.md) describe the archived Lean components. I ran the exact-rational identification and finite measurement checks, which passed. This comparison included no new Lean execution, solver proof or workplace observations.
