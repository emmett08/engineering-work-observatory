# Independent review of the capacity section

**Disposition.** The mathematical claims within the recorded proof scope agree
with the frozen target. This review binds the exact source of
`paper/article/partial-identification.tex` at
`804f07357511794493b5a9bd95675f60e0604f0e`.
The reviewer, `/root/capacity`, authored neither version of that file.
The separate examples in `article/interpreting-capacity.tex` fall outside
this review and the historical Lean evidence.

| Source | Git revision | SHA-256 of the section |
| --- | --- | --- |
| Frozen proof target | `38c4ce40b789d4c5420c98ff1d568b39c3b1e798` | `5d8884d6c0e4e4157d3bfae61d3c13d7c3fb3306f57c13013116d5bcf18ff1fb` |
| Previously reviewed article | `b754d67289c4edd6524e315bd05b5e9f0809b540` | `2f80edf947e643d906f3de38b5f1c3671c6c21b646c8f95eb5d7557ca45e55ff` |
| Article reviewed here | `804f07357511794493b5a9bd95675f60e0604f0e` | `199218ed24332a441cc8dafe504d4e5771fff77215ebf8dfd1dc7b2199712baa` |

Comparing the previously reviewed article with the present article gave
byte-for-byte equality, in order, for both proposition environments, both
proof environments, all nine displayed mathematics blocks and all eleven
labels. New inline expressions occur in explanatory prose and footnotes.
Every changed passage was read against the two article versions, with the
original target and the semantic map as references. The formal package had
no changes between these two commits.

| Present source lines | Findings |
| --- | --- |
| 4–47 | The first-hour bounds follow by applying the capacity inequality on each side of the cutoff. The construction treats either zero-length side explicitly. For a subinterval, the available duration of its complement gives the same lower bound. In the two-task example each separate maximum fits individually, while both cannot fit in the first hour. The change to figure placement and the description of its midpoint affect the presentation only. |
| 58–75 | Each calendar cell lies wholly inside or outside every observation and target interval, apart from shared endpoints of zero time measure. Labour fractions sum to at most one per person. Cell totals therefore have units of person-hours. The new explanations preserve the finite stream sets, target indicators, treatment of outside work and distinction between sampled and exact limits. |
| 77–114 | The reverse construction fits the stream masses in each cell and leaves any remaining time idle. Convexity supplies every intermediate total and compactness gives extrema for a nonempty feasible set. The purification footnote applies to the finite collection of integrals over an atomless time domain. Workflow restrictions can narrow the set, and the deduction about serial schedules concerns human allocations only. [Dvoretzky, Wald and Wolfowitz, Theorem 4](https://msp.org/pjm/1951/1-1/pjm-v1-n1-p06-s.pdf) |
| 118–140 | The dual argument uses nonnegative constraint multipliers and nonnegative cell masses. Applying the upper bound to the negative objective supplies the lower sign. A feasible primal allocation and a matching dual bound establish an optimum. Fixing the matrix and target preserves dual feasibility under a right-hand-side relaxation. The text attributes existence of matching multipliers to LP duality. |
| 142–148 | Simultaneous observation coverage includes the true cell vector in the feasible set, so projecting includes the true target. The union bound permits dependence and requires the sum of marginal failure probabilities to stay within the stated budget. Empty sets count as failures. The two paragraphs on Lean preserve its recorded scope. The existence theorems, the numerical block-count bound and implementation refinement have separate status. |
| 152–154 | The reciprocal bounds require positive minimum labour and fixed nonnegative output. Marginal output and labour bounds can combine unattainable pairs. The added footnote correctly places Sklar's theorem in a probability model. The actual optimisation uses feasible pairs with acceptance times and the retained fractions held fixed. [Haugh, Theorem 2](https://www.columbia.edu/~mh2078/QRM/Copulas.pdf) |

The frozen target uses a shorter cutoff proof. The present proof gives the
same allocations and handles the two endpoint cases by a constant allocation.
The frozen cell construction, signs of the two certificate inequalities,
relaxation and simultaneous-coverage implication match the present claims.
The repository's exact-arithmetic checker passed all 284 one-interval cases,
5,445 exact-observation cases and 1,500 bounded-observation cases. These are
finite checks. The historical Lean build, its toolchain, proof sources and
scope retain their existing evidence. This review establishes correspondence
for this section only.
