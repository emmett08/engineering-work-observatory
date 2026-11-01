#!/usr/bin/env python3
"""Copy actually computed planning cells into a still-specified protocol.

This does not select a principal design or promote readiness. Review calculation
changes before using it; validate_protocol.py checks the resulting cross-links.
"""
from pathlib import Path
import json

ROOT=Path(__file__).resolve().parent

def main():
    p=json.loads((ROOT/'protocol.json').read_text());r=json.loads((ROOT/'planning-results.json').read_text())
    if p['status']!='specified':raise ValueError('Only a specified, unrun protocol may be synchronised by this helper')
    a=p['design_assessment'];a['results']=[]
    field={'T-JOINT':'joint_success','T-FALSE':'joint_success','T-COV-L':'labour_coverage','T-COV-Q':'quality_coverage','T-FALSE-L':'labour_true_null_rejection','T-FALSE-Q':'quality_true_null_rejection'}
    for rr in r['results']:
      for target in a['targets']:
        if rr['scenario_id'] not in target['required_scenario_ids']:continue
        v=rr[field[target['id']]]
        a['results'].append({'design_id':rr['design_id'],'target_id':target['id'],'scenario_id':rr['scenario_id'],'method':'simulation','value':v['estimate'],'replications':v['denominator'],'mcse':v['mcse'],'mc_interval':v['wilson_95'],'evidence':'planning-results.json; deterministic seeds and versions retained; Wilson95% interval describes MonteCarlo error. Incomplete endpoints return full parameter-domain sets; finite halfwidth/bias summaries use complete records only.'})
    p['analysis']['missing_data']='No primary complete-case redefinition. An unresolved team endpoint returns a full parameter-domain confidence set and no rejection/benefit; report available physical/observation bounds separately.'
    (ROOT/'protocol.json').write_text(json.dumps(p,indent=2,sort_keys=True)+'\n')
    print(json.dumps({'status':'synchronised','assessment_results':len(a['results']),'protocol_status':p['status'],'selected_principal_design':a['selected_design_id']}))

if __name__=='__main__':main()
