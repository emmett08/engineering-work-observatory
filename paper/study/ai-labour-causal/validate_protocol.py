#!/usr/bin/env python3
"""Bounded project-local checks, not the unavailable bundled skill validator.

Checks record/reference/metric consistency and its retained calculation evidence.
It cannot validate measurement premises, margin relevance or causal assumptions.
"""
from pathlib import Path
import hashlib,json,math

ROOT=Path(__file__).resolve().parent

def require(ok,message):
    if not ok:raise ValueError(message)

def index(items):
    ids=[x['id'] for x in items];require(len(ids)==len(set(ids)),'duplicate IDs');return {x['id']:x for x in items}

def validate(p,r):
    required={'schema_version','investigation_id','title','version','status','question','primary_claim_id','scope','claims','assumptions','hypotheses','predictions','constructs','measurements','design','units','interventions_or_exposures','controls','confounds','estimands','identification','sampling','analysis','decision_matrix','bias_assessment','ethics_and_safety','feasibility','reproducibility','execution','provenance','method_selection','implementation','design_assessment'}
    require(required<=p.keys(),'missing protocol root fields')
    require(p['schema_version']=='1.2','wrong protocol schema')
    require(p['status']=='specified','this unrun design must remain specified')
    require(p['execution']['started_at'] is None and not p['execution']['observed_results'],'field experiment must not contain fabricated execution')
    claims=index(p['claims']);estimands=index(p['estimands']);measurements=index(p['measurements']);assumptions=index(p['assumptions'])
    require(p['primary_claim_id'] in claims,'unknown primary claim')
    require([x['id'] for x in p['claims'] if x['role']=='primary']==[p['primary_claim_id']],'primary claim count')
    require(set(p['design_assessment']['primary_estimand_ids'])=={'EST-L','EST-Q'},'both benefit conditions must be principal')
    require(p['time_axis']['tau']==p['time_axis']['Delta']==336 and p['time_axis']['labour_followup']==672,'hour/day conversion')
    for e in estimands.values():
        require(set(e['claim_ids'])<=claims.keys(),'unknown estimand claim')
        require(set(e['outcome_measurement_ids'])<=measurements.keys(),'unknown measurement')
    require(set(p['identification']['assumption_ids'])<=assumptions.keys(),'unknown identification premise')
    checks=index(p['implementation']['verification'])
    require({'assignment','collection','scoring','analysis','pipeline'}<={c['stage'] for c in checks.values()},'missing implementation stage checks')
    require(p['implementation']['status']=='planned','field collection remains planned despite completed simulation')
    for tr in p['implementation']['traceability']:
        require(set(tr['check_ids'])<=checks.keys(),'unknown traceability check')
        item=p
        for part in tr['protocol_ref'].strip('/').split('/'):
            item=item[int(part)] if isinstance(item,list) else item[part]
        require(item is not None,'unresolved traceability pointer')
    a=p['design_assessment'];designs=index(a['candidate_designs']);scenarios=index(a['scenarios']);targets=index(a['targets'])
    require(a['status']=='evaluated' and a['selected_design_id'] is None,'failed candidates must not be selected')
    require(r['adequate_candidates_in_registered_scenarios']==[],'registered scenario decision changed; scientific review needed')
    require(r['source_sha256']==hashlib.sha256((ROOT/'planning.py').read_bytes()).hexdigest(),'planning source hash stale')
    require(len(r['results'])==24 and r['replications_per_cell']==20000,'retained execution dimensions')
    require(len(scenarios)==12 and len(designs)==2,'design/scenario inventory')
    rows={(x['design_id'],x['scenario_id']):x for x in r['results']}
    require(len(rows)==24,'duplicated simulation cells')
    field={'T-JOINT':'joint_success','T-FALSE':'joint_success','T-COV-L':'labour_coverage','T-COV-Q':'quality_coverage','T-FALSE-L':'labour_true_null_rejection','T-FALSE-Q':'quality_true_null_rejection'}
    seen=set();failures=[]
    for v in a['results']:
        key=(v['design_id'],v['target_id'],v['scenario_id']);require(key not in seen,'duplicate assessment result');seen.add(key)
        require(v['design_id'] in designs and v['scenario_id'] in scenarios and v['target_id'] in targets,'unknown assessment reference')
        t=targets[v['target_id']];require(v['scenario_id'] in t['required_scenario_ids'],'unexpected assessment scenario')
        q=rows[v['design_id'],v['scenario_id']][field[v['target_id']]]
        require(q is not None and v['value']==q['estimate'] and v['mc_interval']==q['wilson_95'] and v['replications']==q['denominator'],'assessment evidence mismatch')
        require(0<=v['mc_interval'][0]<=v['value']<=v['mc_interval'][1]<=1,'Monte Carlo probability bounds')
        require(math.isclose(v['mcse'],math.sqrt(v['value']*(1-v['value'])/v['replications'])),'Monte Carlo SE')
        bound=v['mc_interval'][0] if t['direction']=='at-least' else v['mc_interval'][1]
        passed=bound>=t['threshold'] if t['direction']=='at-least' else bound<=t['threshold']
        if not passed:failures.append(key)
    expected={(d,t,s) for d in designs for t,obj in targets.items() for s in obj['required_scenario_ids']}
    require(seen==expected,'missing required assessment cells')
    require(any(t=='T-JOINT' for _,t,_ in failures),'shortfall must remain visible')
    require({'labour-boundary-bias','quality-boundary-bias'}<=scenarios.keys(),'partial-null favourable-bias cases missing')
    for d in designs:
        require((d,'T-FALSE-L','labour-boundary-bias') in seen and (d,'T-FALSE-Q','quality-boundary-bias') in seen,'component calibration omitted')
    return {'status':'passed','validator':'bounded project-local structural/reference/numeric checks','bundled_skill_validator':'unavailable: exact advertised skill resource failed to read','protocol_status':p['status'],'field_execution':'not-started','designs':len(designs),'scenarios':len(scenarios),'simulation_cells':len(rows),'replications_per_cell':r['replications_per_cell'],'assessment_results':len(seen),'failed_adequacy_cells':[list(x) for x in failures],'principal_adequacy':'unestablished','scope':'Record/calculation consistency only; not empirical or causal validation'}

def main():
    p=json.loads((ROOT/'protocol.json').read_text());r=json.loads((ROOT/'planning-results.json').read_text())
    result=validate(p,r)
    (ROOT/'validation.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps(result))

if __name__=='__main__':main()
