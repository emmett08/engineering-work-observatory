from pathlib import Path
import hashlib
import re

root = Path('paper/article')
expected = {
    'main.tex': '7620d7c37030dba1b723f426cac9dc3ed1aaee92',
    'partial-identification.tex': 'ec60c8dd992b9d189a5cb3aec392cb92e57389cf',
    'design-based-estimation.tex': 'caf1424fc3f7979f13e5ac8f5d519ed4f2e93699',
    'event-history.tex': '383a1fc233f6ff00d7c7f9e61019c2463883a6c0',
    'sampling-validation.tex': '70c87005437704dc868436711ac879e5c4241e03',
    'scalar-bounds.tex': '610a2c9fe3099028ea9e6925440d70d74601363b',
}

def blob_sha(data):
    return hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()

baseline = {}
for name, digest in expected.items():
    data = (root / name).read_bytes()
    if blob_sha(data) != digest:
        raise RuntimeError(f'Unexpected baseline for {name}')
    baseline[name] = data.decode('utf-8')

def change(name, before, after):
    p = root / name
    text = p.read_text()
    if text.count(before) != 1:
        raise ValueError(f'{name}: expected one match, found {text.count(before)}: {before[:80]}')
    p.write_text(text.replace(before, after))

change('main.tex', r'\date{2 November 2026}', r'\date{5 November 2026}')
change('main.tex', 'These two uses of observations provide different guarantees. An identification interval', 'What does each calculation establish? An identification interval')
change('main.tex', 'To examine what commits reveal about human activity, let a minute-indexed hidden variable', 'Why can the same commit distribution correspond to different amounts of activity? Commits in the model below depend on how often activity occurs and how often an active minute produces a commit. A higher activity probability can be offset by a lower commit probability during activity.\n\nTo express these two contributions, let a minute-indexed hidden variable')
change('main.tex', 'Within the model, different activity distributions can produce the same commit distribution, as the following construction shows.', 'The following construction makes this ambiguity explicit. A Bernoulli variable takes the value one with its stated probability and zero otherwise.')
change('main.tex', 'This construction belongs to the independent-state subfamily', r'''In Equation~\eqref{eq:event-law}, $e_t$ specifies whether minute $t$ contains a commit, so $\sum_t e_t$ counts the commit-bearing minutes in the specified sequence. Equation~\eqref{eq:activity-total} instead counts hidden active minutes and takes their expected total. The shared commit probability fixes the first calculation while the activity probability can change the second.

This construction belongs to the independent-state subfamily''')
change('main.tex', 'The next accounting question is how much human time each task receives.', 'How should human time be counted when several AI workflows run at once? Count the time each person allocates to the work, while recording workflow execution separately.')
change('main.tex', 'For each task accepted by the deadline, multiply its initial reference score by the fraction of its accepted requirement still satisfied at follow-up. This product is its retained output score.', 'What counts as output at follow-up? For each task accepted by the deadline, its retained output score is its initial reference score multiplied by the fraction of its accepted requirement still satisfied.')
change('main.tex', 'The labour total is half a person-hour. Record one hour of execution for each workflow in the computing-resource account described below.', r'''The labour total is half a person-hour. Record one hour of execution for each workflow in the computing-resource account described below. Figure~\ref{fig:human-and-workflow-time} places these quantities on the same elapsed-time axis.

\input{../figures/human-and-workflow-time}''')
change('main.tex', 'The preceding bounds describe every labour total permitted by given interval observations. Sampling raises the question of how selected person-intervals can support an estimate of total labour.', 'The preceding bounds describe every labour total permitted by given interval observations. What changes when labour is observed only in sampled person-intervals? We estimate the total, and assess how that estimate varies across possible samples.')
change('main.tex', 'The estimate varies with the selected intervals, so entering it as an exact observation can exclude the true allocation through sampling error.', 'Entering a sampled estimate as an exact interval observation can exclude the true allocation through sampling error.')
change('main.tex', 'Accurate labour and output measurements support a comparison between groups. Attributing a difference to AI access also requires a causal design that defines which work its conclusion concerns.', 'Accurate labour and output measurements support a comparison between groups. When can a difference be attributed to AI access? That conclusion requires a causal design defining which work is compared and how the comparison groups are formed.')

change('event-history.tex', 'Use the acceptance deadline', 'Why do output assessment and labour recording end at different task ages? Output is checked a fixed duration after acceptance, while labour is counted through a common task age. These choices determine which delivered requirements and which person-hours enter the productivity ratio.\n\nUse the acceptance deadline')
change('event-history.tex', 'Complete follow-up can still leave several labour allocations consistent with the records.', r'''Figure~\ref{fig:task-clocks} shows the two endpoints for an illustrative accepted task. Adding $s_i$ to each task age gives its calendar time, allowing the task's labour to be combined with other tasks sharing the same people.

\input{../figures/task-clocks}

Complete follow-up can still leave several labour allocations consistent with the records.''')

change('partial-identification.tex', "One person can contribute at most one person-hour in an elapsed hour. Knowing who supplied an interval total therefore limits how much of that labour could have occurred before or after a specified time. A total across several people must be compared with their combined available time.", r'''How much of a known interval total must lie before a chosen time? Whatever cannot fit after that time must have occurred before it. One person can contribute at most one person-hour in an elapsed hour, so each part of the interval has a known capacity. A total across several people must be compared with their combined available time.

In the proposition, $\ell$ and $u$ are the interval's start and end, $c$ is the time dividing it, and $v$ is the observed total labour. The unknown $x$ is the labour before $c$.''')
change('partial-identification.tex', 'Combining contributions requires distinguishing bounds on their sum from totals that can occur together.', 'What can be lost when separate task bounds are added? The sum can include totals whose component allocations cannot occur together.')
change('partial-identification.tex', 'Form the sorted distinct endpoints', 'How can a calculation consider all the possible activity histories? It is enough to keep the labour totals between successive observation and target boundaries.\n\nForm the sorted distinct endpoints')
change('partial-identification.tex', 'The following result shows that these cell totals preserve exactly the possible target values.', r'''Each $x_{ijrk}$ is a duration in person-hours, whereas $a_{ijr}(t)$ is an allocation fraction. The following result shows that the cell totals preserve exactly the possible target values. Its weights select or scale the contributions to the target, with weights of one including a contribution and weights of zero excluding it.''')
change('partial-identification.tex', 'To verify a proposed maximum, give a feasible allocation', 'How can a calculated endpoint be checked? Combine an allocation attaining it with a bound that applies to every feasible allocation.\n\nTo verify a proposed maximum, give a feasible allocation')
change('partial-identification.tex', 'Collect the cell totals and target weights into vectors $x$ and $w$, and write the capacity and observation inequalities as $Ax\\leq b$, with $x\\geq0$ kept separate.', r'''Collect the cell totals and target weights into vectors $x$ and $w$, and write the capacity and observation inequalities as $Ax\leq b$, with $x\geq0$ kept separate. Each row of $A$ contains the coefficients of one constraint, and the matching entry of $b$ is its limit. The superscript $\mathsf T$ denotes transpose, so $w^\mathsf T x$ is the weighted sum of cell totals.''')
change('partial-identification.tex', 'A numerical solver\'s success flag alone does not supply this check.', 'The allocation and multipliers supply the quantities needed to check a numerical solver\'s proposed endpoint.')
change('partial-identification.tex', 'With $Q\\geq0$ and $H_{\\min}>0$, dividing this output by the largest and smallest possible labour totals gives the identified productivity interval $[Q/H_{\\max},Q/H_{\\min}]$.', r'''With $Q\geq0$ and $H_{\min}>0$, dividing this output by the largest and smallest possible labour totals gives the identified productivity interval $[Q/H_{\max},Q/H_{\min}]$. The endpoints reverse because, for fixed nonnegative output, a larger positive labour total gives the same or a smaller output-to-labour ratio.''')

change('design-based-estimation.tex', 'Summing predictions for every person-interval gives a predicted labour total.', 'How can an inaccurate prediction still support unbiased labour estimation? Use sampled observations to estimate its total error, then correct the prediction total.\n\nSumming predictions for every person-interval gives a predicted labour total.')
change('design-based-estimation.tex', 'The correction therefore preserves design-unbiasedness even when the fixed prediction total differs from $H$.', r'''In the proof, $I_m$ is one when interval $m$ is sampled and zero otherwise. Weighting its residual by $1/\pi_m$ makes that contribution average to the interval's full residual over possible samples. The correction therefore preserves design-unbiasedness even when the fixed prediction total differs from $H$.''')
change('design-based-estimation.tex', 'The residuals and sampling design jointly determine precision.', 'Does unbiasedness guarantee a precise estimate? Unbiasedness describes the average over possible samples. Precision concerns how widely the estimates vary around that average, which depends jointly on the residuals and the sampling design.')
change('design-based-estimation.tex', 'Every size-$n$ subset has the same probability and $\\pi_m=n/M$. Write', r'''Every size-$n$ subset has the same probability and $\pi_m=n/M$. Let $\bar r_U$ and $\bar y_U$ be the means of the residuals and labour values over the complete frame. Their finite-population variances are''')
change('design-based-estimation.tex', 'For $2\\leq n\\leq M$, an unbiased estimator of the design variance is', r'''The factor $1-n/M$ accounts for sampling without replacement and becomes zero when every unit is measured. To estimate the variance from the sample, let $\bar r_s$ be its mean residual and $s_r^2$ its residual variance. For $2\leq n\leq M$, an unbiased estimator of the design variance is''')
change('design-based-estimation.tex', 'The residuals must vary less across person-intervals than the labour values themselves. For example,', 'The residuals must vary less across person-intervals than the labour values themselves.\n\nFor example,')
change('design-based-estimation.tex', 'Enumerating all six samples reproduces these variances', r'''Figure~\ref{fig:sampling-spread} shows the estimate from each of the six equally likely samples. All three sets of estimates average to the same true total, while their spreads differ.

\input{../figures/sampling-spread}

Enumerating all six samples reproduces these variances''')
change('design-based-estimation.tex', 'Fitting predictions to the correction sample can change the estimator\'s expectation.', 'Why must each prediction remain the same whichever correction sample is selected? The unbiasedness proof treats that prediction as fixed when averaging over samples. Fitting predictions to the correction sample can change the estimator\'s expectation.')

change('sampling-validation.tex', 'Comparing a prediction with sampled labour combines prediction error with sampling error.', 'Does disagreement with sampled labour measure prediction error alone? It combines the prediction\'s error with the labour estimate\'s sampling error.')
change('scalar-bounds.tex', 'Path weights then bound differences in cumulative labour.', r'''Why does a shortest path give a labour bound? Adding the inequalities along a path gives an upper limit on the difference between its final and initial cumulative totals. The smallest path-weight sum gives the tightest of these upper limits. A reverse path bounds the opposite difference and therefore supplies a lower limit on the target labour.''')

result_hashes = {
    'main.tex': 'c35f98d8642104bf8114cf6e742567849f4bf963',
    'partial-identification.tex': 'c632c66ee2adcfb13bc810ed30c7561fdb2a60a9',
    'design-based-estimation.tex': 'ac48ec5478a6331e40f888804011a9b91557e3a2',
    'event-history.tex': 'bfb5f3e81c2ea82ec6a3419fd025863cde5474e3',
    'sampling-validation.tex': 'cdfc17944d378434295e2f6c649cafda79cd3064',
    'scalar-bounds.tex': '9ad398e4a3891c52fccc94ca736ef887641c38ec',
}
for name, old in baseline.items():
    data = (root / name).read_bytes()
    if blob_sha(data) != result_hashes[name]:
        raise RuntimeError(f'Result differs from locally reviewed source: {name}')
    new = data.decode('utf-8')
    for env in ('equation', 'align', 'align*', 'proposition', 'proof'):
        pattern = r'\\begin\{' + re.escape(env) + r'\}.*?\\end\{' + re.escape(env) + r'\}'
        if re.findall(pattern, old, re.S) != re.findall(pattern, new, re.S):
            raise RuntimeError(f'Changed formal block: {name}/{env}')
    for pattern in (r'\\\[.*?\\\]', r'\\cite\w*\*?(?:\[[^\]]*\])*\{[^}]+\}', r'\\label\{([^}]+)\}'):
        if re.findall(pattern, old, re.S) != re.findall(pattern, new, re.S):
            raise RuntimeError(f'Changed protected content: {name}')
    print(f'PASS exact result and formal-content preservation: {name} {result_hashes[name]}')
