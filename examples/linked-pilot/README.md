# Linked EngArg/EAL pilot

[`manifest.json`](manifest.json) identifies the selected public issues, PRs and
CI runs. [`config-template.json`](config-template.json) uses the same protocol
and task IDs for private effort collection. Its person, device, bucket and time
window are **placeholders, not observations**. `frame_complete` is deliberately
false. Do not run it unchanged as a human study.

[`private-runner-workflow.yml`](private-runner-workflow.yml) is an adoption
template for a private control repository and a trusted, dedicated runner. It
does not register or deploy a runner. It uploads no participant artifacts and
publishes only on an explicit dispatch. The protocol's sampled checks still
require actual responses.

Follow the [setup and demo guide](../../docs/linked-pilot.md). The public demo
can collect real CI receipts and report human effort as unavailable before any
participant machine is configured.
