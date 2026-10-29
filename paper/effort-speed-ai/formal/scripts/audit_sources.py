"""Static trust scan and exhaustive exported-theorem audit manifest."""
from pathlib import Path
import json,re,hashlib,sys
root=Path(__file__).resolve().parents[1]
names=['Basic','Binomial'] + (['Geometric'] if '--include-geometric' in sys.argv else [])
files=[root/'FormalVerification'/f'{name}.lean' for name in names]
exports=[];hits=[]
for f in files:
 s=f.read_text()
 # Comments are retained in scan for human review; avoid documenting escape hatches in sources.
 for pattern in [r'\bsorry\b',r'\badmit\b',r'\bsorryAx\b',r'\baxiom\b',r'\bconstant\b',r'\bpartial\b',r'\bunsafe\b',r'\bimplemented_by\b',r'\bextern\b',r'\bnative_decide\b']:
  for m in re.finditer(pattern,s):hits.append({'path':str(f.relative_to(root)),'line':s.count('\n',0,m.start())+1,'match':m.group()})
 for m in re.finditer(r'^theorem\s+(\w+)',s,re.M):
  exports.append({'declaration':'EffortSpeed.'+m.group(1),'path':str(f.relative_to(root)),'sha256':hashlib.sha256(f.read_bytes()).hexdigest()})
audit=''.join('import '+str(f.relative_to(root)).removesuffix('.lean').replace('/','.')+'\n' for f in files)+'\n'+''.join('#print axioms '+x['declaration']+'\n' for x in exports)
(root/'AxiomAudit.lean').write_text(audit)
(root/'evidence/source-audit.json').write_text(json.dumps({'exports':exports,'scan_hits':hits},indent=2)+'\n')
print(json.dumps({'export_count':len(exports),'scan_hits':hits}))
if hits: raise SystemExit('Trust scan requires adjudication')
