# Hosted demo verification — 1 November 2026

[Core run 41346681618](https://github.com/emmett08/engineering-work-observatory/actions/runs/41346681618)
passed on GitHub-hosted Ubuntu at implementation commit
`159f3de6345d1f787922a52e642e21b7373f4659`.
The pinned reusable caller also passed in
[instrumentation run 41346760938](https://github.com/emmett08/observatory-instrumentation/actions/runs/41346760938).
Neither used a private runner, workstation observations or a paid model API.

| Check | Observed result |
| --- | --- |
| Public observation acquisition | Complete; one current declaration and one retained revision |
| Closed issue support | EngArg #51 imported while closed |
| Declared human effort | 3 minutes for issue #49 / PR #50 |
| Training eligibility | Excluded: unknown method and coverage |
| Live learning in PR | Observe-only; no live fitting, validation scoring or promotion |
| Selected execution evidence | Two invocations, each with start and completion receipts; platform fields compared with retained API data |
| Reported tool runtime | 42.951 seconds in total; parallel activity can overlap, and this is not human labour |
| Synthetic software check | 83 training tasks, 45 held-out tasks; conditional promotion and idempotent replay passed |
| Synthetic mean absolute error | Baseline 20.289 minutes; candidate 1.770 minutes; fabricated populations only |
| Checkpoint verification | Local restore and uploaded/downloaded restore both passed |
| Independent verification | Download hashes matched GitHub digests; manifest validated; Rust verified 37 retained execution-store records |

The unmodified [report](report.md) and [machine-readable report](report.json)
are retained here with the imported [source revision](engarg-51-revision.json).
[Verification metadata](verification.json) records run, artifact and file hashes.
The archived report's absolute artifact paths describe the original hosted run;
they are not local paths in this checkout.

This proves the public import, execution collection, synthetic learning and
artifact delivery path. It does **not** prove improved real effort estimates.
The synthetic fixture was deliberately constructed with different repository
populations. The two workflow runs are repeated demonstrations over the same
source and fixture, not independent validation studies.

Trusted scheduled collection starts after merge with one manual
`bootstrap=true` run on the default branch; PR state cannot initialize it.
The ongoing-history protections have automated tests. Artifacts expire after
30 days; these committed snapshots preserve selected public results, not a
replacement for the collector's continuous state. See the
[operating guide](../automatic-observation-learning.md).
