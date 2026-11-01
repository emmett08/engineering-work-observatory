# Live linked pilot evidence

[Collector run 41339863127](https://github.com/emmett08/engineering-work-observatory/actions/runs/41339863127)
downloaded the two pilot repositories' actual bundles, validated schemas and
hashes, reconciled their platform identities and collected them twice. The
verified store contains 55 records. Both deliveries retained the same six
receipt records: three invocations with start and completion phases.

| Task | Selected check | Observed tool runtime |
| --- | --- | --- |
| EngArg, PR #50 | Python authoring tests, Ubuntu | 14.583 seconds |
| EngArg, PR #50 | Python authoring tests, macOS | 28.368 seconds |
| EAL, PR #37 | Rust context-experiment tests, Ubuntu | 17.050 seconds |

All three selected commands succeeded. EAL's pilot workflow had completed
successfully at collection. EngArg's broader workflow was still running; its
five other jobs had no matched receipt for this task. These records do not
establish complete tool coverage or the final result of every EngArg check.

Human effort is **unavailable**, with no participant activity or sampled
responses. GitHub actor and triggering actor are `emmett08`; the declared
controller remains `unknown`, and no assistant session is invented. These
runtimes are not estimates of human work.

- [EngArg report](task-f8608a061be71b1e.md)
- [EAL report](task-4e8873fa577efb0c.md)
- [Public collector snapshot](collected-snapshot.json)
- [Repeated-delivery result](deduplication.json)
- [File hashes and collector provenance](evidence-index.json)

The aggregate reports were published to both task issues and implementing PRs:
[EngArg issue](https://github.com/emmett08/engarg/issues/49#issuecomment-6562521151),
[EngArg PR](https://github.com/emmett08/engarg/pull/50#issuecomment-6562521373),
[EAL issue](https://github.com/emmett08/eal-mcp/issues/36#issuecomment-6562520686),
and [EAL PR](https://github.com/emmett08/eal-mcp/pull/37#issuecomment-6562520875).
Running the publisher again reused those comments without duplicates.

The three `*-bundle.json` files preserve the original unmodified receipt bundles.
The two `links-*.json` files retain the actual GitHub closing-issue relationships.
Report JSON/Markdown files are copied unchanged from the collector artifact.
These public files contain no participant activity, sampled responses or private
runner configuration. The full verified collector database is also retained in
the workflow artifact; collect that archive before its 30-day expiry if needed.

The snapshot is an observation at collection time. Later workflow completion or
reruns require another collection, not edits to these historical records. The
[private runner guide](../linked-pilot.md) describes ongoing collection and the
additional observations needed for effort estimation.
