# Original automatic forecast example

`issued-20261029.json.gz` contains an actual `forecast-team` bundle from the two
public user repositories. The gzip archive retains the original bytes; the manifest
records both file hashes and the verified snapshot digest. It includes public API
payloads so the forecast can be recomputed from its source inventory. It is not a
synthetic fixture or a retrospective replay. Its October 30 daily target had not
begun when the local command recorded creation.

The declared forecast uses five complete daily counts, `[17,11,13,12,8]`.
Its mean is 12.2 and predictive median 12. Neither this short history nor the saved
receipt establishes predictive accuracy. The receipt times are declarations, not
independent timestamp certification. The separate manuscript tables use an earlier
retained snapshot with the same five counts; the snapshot identifiers differ.

To verify the unchanged original and its pending outcome with the current CLI:

```sh
mkdir -p work/forecast-example
python3 - <<'PYCODE'
import gzip, json
from pathlib import Path
root = Path('paper/forecasting/examples')
raw = gzip.decompress((root / 'issued-20261029.json.gz').read_bytes())
Path('work/forecast-example/issued.json').write_bytes(raw)
snapshot = json.loads(raw)['snapshot']
Path('work/forecast-example/source.json').write_text(json.dumps(snapshot))
PYCODE
cargo run --locked -- score-forecast work/forecast-example/issued.json \
  work/forecast-example/source.json
```

The supplied pending-score JSON has no outcome or loss. A later sufficiently fresh
collection is required to score the entire target. The example does not supply one.
Do not publish private deployment bundles merely because this public example is
retained in the repository.
