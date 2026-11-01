#!/usr/bin/env python3
"""Planning sensitivity for a specified, unrun paired-team experiment.

The generated populations are assumptions, not workplace observations. Student-t
interval calibration is measured here, not guaranteed for every finite population.
Run --check to recompute mathematical results without changing retained files.
"""
from pathlib import Path
import argparse, csv, hashlib, json, math, platform, sys
import numpy as np
import scipy
from scipy.special import expit
from scipy.stats import t

ROOT = Path(__file__).resolve().parent
REPS = 20000
BASE_SEED = 20261101
ALPHA = .025
LABOUR_BOUNDARY = -1.0  # person-hours per recorded reference unit
QUALITY_BOUNDARY = -.05  # retained fraction, five percentage points
LABOUR_BIAS_ALLOWANCE = .2  # difference of team measurement biases
QUALITY_BIAS_ALLOWANCE = .01
DESIGNS = {"D48": 24, "D96": 48}
SCENARIOS = [
 {"id":"central-benefit", "effect_l":-2.,"effect_q":0.,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":0.,"bias_q":0.,"pair_loss":0.,"purpose":"information","required":True},
 {"id":"adverse-benefit", "effect_l":-2.,"effect_q":0.,"rho":.2,"labour_log_sd":.65,"quality_logit_sd":2.5,"noise_l":.6,"noise_q":.03,"bias_l":0.,"bias_q":0.,"pair_loss":0.,"purpose":"information","required":True},
 {"id":"labour-boundary", "effect_l":-1.,"effect_q":0.,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":0.,"bias_q":0.,"pair_loss":0.,"purpose":"calibration","required":True},
 {"id":"quality-boundary", "effect_l":-2.,"effect_q":-.05,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":0.,"bias_q":0.,"pair_loss":0.,"purpose":"calibration","required":True},
 {"id":"both-boundaries", "effect_l":-1.,"effect_q":-.05,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":0.,"bias_q":0.,"pair_loss":0.,"purpose":"calibration","required":True},
 {"id":"no-effect", "effect_l":0.,"effect_q":0.,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":0.,"bias_q":0.,"pair_loss":0.,"purpose":"calibration","required":True},
 {"id":"quality-harm", "effect_l":-2.,"effect_q":-.10,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":0.,"bias_q":0.,"pair_loss":0.,"purpose":"harm","required":True},
 {"id":"bias-at-allowance", "effect_l":-1.,"effect_q":-.05,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":-.2,"bias_q":.01,"pair_loss":0.,"purpose":"calibration","required":True},
 {"id":"labour-boundary-bias", "effect_l":-1.,"effect_q":0.,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":-.2,"bias_q":0.,"pair_loss":0.,"purpose":"calibration","required":True},
 {"id":"quality-boundary-bias", "effect_l":-2.,"effect_q":-.05,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":0.,"bias_q":.01,"pair_loss":0.,"purpose":"calibration","required":True},
 {"id":"bias-exceeds-allowance", "effect_l":-1.,"effect_q":-.05,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":-1.,"bias_q":.06,"pair_loss":0.,"purpose":"assumption-failure","required":False},
 {"id":"incomplete-pairs", "effect_l":-2.,"effect_q":0.,"rho":.7,"labour_log_sd":.35,"quality_logit_sd":1.4,"noise_l":.2,"noise_q":.005,"bias_l":0.,"bias_q":0.,"pair_loss":.005,"purpose":"missingness","required":True},
]

def probability(values):
    count=int(np.count_nonzero(values)); n=len(values); p=count/n; z=1.959963984540054
    den=1+z*z/n; centre=(p+z*z/(2*n))/den
    half=z*math.sqrt(p*(1-p)/n+z*z/(4*n*n))/den
    return {"count":count,"denominator":n,"estimate":p,"mcse":math.sqrt(p*(1-p)/n),"wilson_95":[centre-half,centre+half]}

def latent(rng, repetitions, pairs, rho):
    # Correlation between labour and retained-output baselines is 0.3;
    # between teams of a pair it is rho for each latent outcome.
    c=rng.normal(size=(repetitions,pairs,1,2)); e=rng.normal(size=(repetitions,pairs,2,2))
    c[...,1]=.3*c[...,0]+math.sqrt(1-.3**2)*c[...,1]
    e[...,1]=.3*e[...,0]+math.sqrt(1-.3**2)*e[...,1]
    return math.sqrt(rho)*c+math.sqrt(1-rho)*e

def analyse(d_l,d_q,complete):
    pairs=d_l.shape[1]; critical=float(t.ppf(1-ALPHA,pairs-1))
    means_l=d_l.mean(axis=1); means_q=d_q.mean(axis=1)
    half_l=critical*d_l.std(axis=1,ddof=1)/math.sqrt(pairs)+LABOUR_BIAS_ALLOWANCE
    half_q=critical*d_q.std(axis=1,ddof=1)/math.sqrt(pairs)+QUALITY_BIAS_ALLOWANCE
    # Incomplete endpoints produce an uninformative full-parameter-domain set,
    # never a hidden complete-case estimate or a test on latent missing values.
    labour=(means_l+half_l<LABOUR_BOUNDARY)&complete
    quality=(means_q-half_q>QUALITY_BOUNDARY)&complete
    return means_l,means_q,half_l,half_q,labour,quality,labour & quality & complete

def simulate(design,scenario,index):
    pairs=DESIGNS[design]; rng=np.random.default_rng(np.random.SeedSequence([BASE_SEED,index,pairs]))
    z=latent(rng,REPS,pairs,scenario['rho'])
    s=scenario['labour_log_sd']; l0=3+7*np.exp(s*z[...,0]-.5*s*s)
    q0=.72+.25*expit(scenario['quality_logit_sd']*z[...,1])
    # Fixed additive potential-outcome contrasts. All true values remain physical.
    assert np.all(l0+scenario['effect_l']>0)
    assert np.all((q0+scenario['effect_q']>=0)&(q0+scenario['effect_q']<=1))
    coin=rng.integers(0,2,size=(REPS,pairs)); sign=2*coin-1
    d_l=sign*(l0[...,0]-l0[...,1])+scenario['effect_l']+scenario['bias_l']
    d_q=sign*(q0[...,0]-q0[...,1])+scenario['effect_q']+scenario['bias_q']
    d_l+=rng.normal(scale=math.sqrt(2)*scenario['noise_l'],size=d_l.shape)
    d_q+=rng.normal(scale=math.sqrt(2)*scenario['noise_q'],size=d_q.shape)
    complete=(rng.random(size=(REPS,pairs))>=scenario['pair_loss']).all(axis=1)
    ml,mq,hl,hq,pass_l,pass_q,joint=analyse(d_l,d_q,complete)
    return {'design_id':design,'scenario_id':scenario['id'],'pairs':pairs,'teams':2*pairs,
      'labour_bias_complete_data':float(ml[complete].mean()-scenario['effect_l']),'quality_bias_complete_data':float(mq[complete].mean()-scenario['effect_q']),
      'labour_coverage':probability((~complete)|(np.abs(ml-scenario['effect_l'])<=hl)),
      'quality_coverage':probability((~complete)|(np.abs(mq-scenario['effect_q'])<=hq)),
      'labour_pass':probability(pass_l),'quality_pass':probability(pass_q),'joint_success':probability(joint),
      'labour_true_null_rejection':probability(pass_l) if scenario['effect_l']>=LABOUR_BOUNDARY else None,
      'quality_true_null_rejection':probability(pass_q) if scenario['effect_q']<=QUALITY_BOUNDARY else None,
      'complete_data':probability(complete),'mean_labour_halfwidth':float(hl[complete].mean()),'mean_quality_halfwidth':float(hq[complete].mean()),
      'width_scope':'Mean finite sensitivity-expanded halfwidth among complete simulated records only; incomplete records return the full parameter domain and no component rejection',
      'planned_task_count':2*pairs*20,'collection_review_hours':2*pairs*(12+20*.5),
      'scope':'Hypothetical normal latent pair baselines with lognormal labour, bounded retained fractions, actual pair coin flips and the planned t/bias-allowance decision; not empirical power.'}

def verification():
    # Exact algebraic pairing expectation, independent of generated populations.
    from fractions import Fraction as F
    l00,l01,l10,l11=map(F,[8,7,12,9])
    observed=[l01-l10,l11-l00]
    assert sum(observed)/2==((l01-l00)+(l11-l10))/2
    dl=np.full((3,24),-2.); dq=np.zeros((3,24)); complete=np.array([True,True,False])
    ans=analyse(dl,dq,complete); assert ans[-1].tolist()==[True,True,False]
    dq[:]=-.1; assert not analyse(dl,dq,complete)[-1].any()
    return {'pair_randomisation_expectation':'exact Fraction check passed','known_benefit':'passed','quality_harm_override':'passed','missing_endpoint_override':'passed'}

def compute():
    results=[simulate(d,s,i) for d in DESIGNS for i,s in enumerate(SCENARIOS)]
    feasible=[]
    for d in DESIGNS:
      rows={r['scenario_id']:r for r in results if r['design_id']==d}
      information=all(rows[s]['joint_success']['wilson_95'][0]>=.90 for s in ['central-benefit','adverse-benefit','incomplete-pairs'])
      calibration=all(rows[s]['joint_success']['wilson_95'][1]<=.035 for s in ['labour-boundary','quality-boundary','both-boundaries','no-effect','quality-harm','bias-at-allowance','labour-boundary-bias','quality-boundary-bias'])
      component_calibration=all(value['wilson_95'][1]<=.035 for s in SCENARIOS if s['required'] for field in ['labour_true_null_rejection','quality_true_null_rejection'] if (value:=rows[s['id']][field]) is not None)
      coverage=all(rows[s['id']][metric]['wilson_95'][0]>=.94 for s in SCENARIOS if s['required'] for metric in ['labour_coverage','quality_coverage'])
      if information and calibration and component_calibration and coverage:feasible.append(d)
    return {'schema':'causal-study-planning/1','status':'evaluated-under-provisional-assumptions','replications_per_cell':REPS,'base_seed':BASE_SEED,
      'versions':{'python':platform.python_version(),'numpy':np.__version__,'scipy':scipy.__version__},'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
      'designs':DESIGNS,'scenarios':SCENARIOS,'analysis':{'alpha_one_sided':ALPHA,'labour_boundary':LABOUR_BOUNDARY,'quality_boundary':QUALITY_BOUNDARY,'difference_bias_allowance_l':LABOUR_BIAS_ALLOWANCE,'difference_bias_allowance_q':QUALITY_BIAS_ALLOWANCE,'reported_coverage':'sensitivity-expanded intervals; raw t intervals are not claimed to have exact finite-sample coverage','success':'labour upper<boundary AND quality lower>boundary AND all team endpoints available'},
      'verification':verification(),'results':results,'adequate_candidates_in_registered_scenarios':feasible,
      'selected_principal_design':feasible[0] if feasible else None,
      'decision':'Principal-study adequacy unestablished; run specified measurement pilot and repeat planning with supported nuisance ranges. Never size from favourable pilot treatment effect.'}

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
    result=compute()
    if args.check:
      assert result==json.loads((ROOT/'planning-results.json').read_text()),'Planning results changed; inspect before recording.'
    else:
      (ROOT/'planning-results.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
      with (ROOT/'planning-table.csv').open('w',newline='') as f:
        writer=csv.writer(f);writer.writerow(['design','scenario','joint_success','joint_mcse','joint_wilson_low','joint_wilson_high','labour_coverage','quality_coverage','labour_halfwidth','quality_halfwidth','planned_tasks','collection_review_hours'])
        for r in result['results']:
          j=r['joint_success'];writer.writerow([r['design_id'],r['scenario_id'],j['estimate'],j['mcse'],*j['wilson_95'],r['labour_coverage']['estimate'],r['quality_coverage']['estimate'],r['mean_labour_halfwidth'],r['mean_quality_halfwidth'],r['planned_task_count'],r['collection_review_hours']])
    print(json.dumps({'status':'passed','mode':'check' if args.check else 'record','cells':len(result['results']),'replications_per_cell':REPS,'adequate_candidates':result['adequate_candidates_in_registered_scenarios']}))

if __name__=='__main__':main()
