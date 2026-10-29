#!/usr/bin/env python3
from pathlib import Path
import json,hashlib,re
root=Path(__file__).resolve().parents[1]
x=json.loads((root/'claim-inventory.json').read_text())
e=json.loads((root/'evidence/source-audit.json').read_text())['exports']
assert len(x['results'])==54
assert len({r['result_id'] for r in x['results']})==54
assert {n for r in x['results'] for n in r['declarations']}=={r['declaration'] for r in e}
for r in x['results']:
 s=r['source'];text=(root/s['path']).read_text()
 assert hashlib.sha256((root/s['path']).read_bytes()).hexdigest()==s['sha256']
 if s['path'].endswith('.tex'):
  labels=set(re.findall(r'\\label\{([^}]+)\}',text))
  tokens=set(re.findall(r'\b(?:eq|prop|cor|ex|rem|sec|app|fig|tab):[a-zA-Z0-9-]+',s['statement_locator']))
  assert tokens<=labels,(r['result_id'],tokens-labels)
 assert r['disposition'] in {'FORMALISE_NOW','DEFERRED_DEPENDENCY','EMPIRICAL_CONTEXTUAL','STATEMENT_REPAIR_REQUIRED','NOT_REQUIRED'}
print(json.dumps({'result_count':54,'unique_export_count':43,'hashes_and_label_locators':'MATCHED','dispositions':'COMPLETE','scope':'Inventory consistency only; not a mathematical or empirical proof.'},indent=2))
