# Coordinator candidate, revision 1

This candidate is additional disclosed input, not an agent-approved result.

Retain the finite-cell LP, strengthen its reconstruction with per-person, per-cell consecutive 0/1 activity intervals, and use published LP duality to expose independently checkable endpoint bounds.

Let P={x>=0: A x<=b} be the existing nonempty bounded cell polytope and h(x)=w^T x. All observation lower inequalities are multiplied by -1 in A,b. Nonnegative vectors lambda and mu satisfying A^T lambda>=w and A^T mu>=-w give -b^T mu<=w^T x<=b^T lambda for every x in P. LP strong duality gives vectors attaining the exact endpoints. Matching these bounds to primal attaining vectors supplies exact optimality certificates by arithmetic alone.

For a relaxation b -> b+e with e>=0, the same vectors certify [-mu^T(b+e),lambda^T(b+e)] for every newly feasible x; extrema may be tighter. Original feasible points remain feasible. Do not claim derivatives or exact marginal measurement values from this global one-sided inequality.

Proof: w^T x <= lambda^T A x <=lambda^T b, and the same argument for -w. Relaxed right hand sides repeat that argument. Existence of optimal dual vectors uses standard feasible bounded LP duality, without strict feasibility. Boyd and Vandenberghe (2004), sections 5.2.3–5.2.4 and 5.6 inspected at https://web.stanford.edu/~boyd/cvxbook/bv_cvxbook.pdf.

Optional short statistical statement: if random observation intervals jointly contain all true observation totals with probability at least 1-alpha, and true history satisfies deterministic allocation assumptions, the true history belongs to the resulting random P on that event; optimizing then covers the true target total with at least 1-alpha. Empty P is not automatically a zero-width interval. Constructing those simultaneous intervals remains sampling-design-specific. No assertion of empirical validity or implementing a generic confidence interval.

Potential linear-fractional extension: preserve as an optional reviewed alternative unless a joint linear feasible set for uncertain retained output and labour is actually supplied. Existing paper's fixed-output quotient is already exact. Do not invent such output/labour constraints to justify an extra theorem.

Disclosure: challenger native preliminary message suggested the binary reconstruction; coordinator independently checked its interval scheduling construction. Other candidates here are coordinator contributions. Initial literature search had nominated strong duality and Charnes–Cooper; no mathematical candidate was supplied to initial independent workers.
