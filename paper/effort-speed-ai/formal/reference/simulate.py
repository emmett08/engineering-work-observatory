#!/usr/bin/env python3
"""Synthetic mechanisms only. No company observations or estimated AI effects.
Run with Python 3.12, numpy, scipy and matplotlib. Outputs are reproducible.
"""
from pathlib import Path
import csv, json, math, os, platform, resource, time
from fractions import Fraction as F
import numpy as np
from scipy.stats import binomtest
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
OUT=Path(__file__).resolve().parent
FIG=OUT/'figures'; FIG.mkdir(exist_ok=True)
EV=OUT/'evidence'; EV.mkdir(exist_ok=True)
plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False,'savefig.dpi':180})

def exact_checks():
    null=sum((F(math.comb(10,k),2**10) for k in [0,1,9,10]),F(0))
    assert null==F(11,512)
    # Identical full finite observed laws, distinct latent expected effort.
    a,b=(F(1,2),F(1,5)),(F(1,4),F(2,5))
    assert a[0]*a[1]==b[0]*b[1]==F(1,10)
    # Positive raw effort: S=2+X, Y=10+4X+3Z, efficiency term U=3X+3Z.
    # X,Z independent symmetric +-1, variance=1 each. Corr(S,Y)^2=16/25.
    var_y,var_u=F(25),F(18)
    assert var_u/var_y>1-F(16,25)
    # Failed conjecture: a 50% smaller per-attempt effort guarantees lower total effort.
    assert F(1,2)/F(1,5)>F(1)/F(9,10)
    # Renewal/reliability identities for a finite rational grid, no universal proof claimed.
    rational_cases=0
    for d in [F(0),F(1,5),F(3,5),F(9,10)]:
        for sens in [F(0),F(1,3),F(1)]:
            p=1-d*sens; rho=(1-d)/p
            assert rho*p==1-d
            rational_cases+=1
    return {'binomial_null_exact':str(null),'binomial_null_probability':float(null),
      'iid_second_stage_p_value_only':float(binomtest(24,300,float(null)).pvalue),
      'hmm_same_event_probability':str(a[0]*a[1]),'hmm_occupancy_ratio':str(a[0]/b[0]),
      'variance_counterexample_r_squared':.64,'efficiency_component_variance_fraction':float(var_u/var_y),
      'unexplained_fraction':.36,'rational_detection_cases':rational_cases,
      'capped_retry_checks':capped_retry_checks(),
      'formal_proof_status':'Exact arithmetic and analytical derivations; no Lean certificate'}

def capped_retry_checks():
    cases=0
    for p in [F(0), F(1,1000), F(1,5), F(1,2), F(1)]:
      for k in [0,1,2,10,100]:
        m=sum(((1-p)**j for j in range(k)),F(0))
        q=1-(1-p)**k
        if p: assert m==q/p
        else: assert m==k and q==0
        cases+=1
    return cases

def make_scenarios():
    # Two fully parallel branches: one human production branch, one agent branch;
    # human review follows both, then a one-off queue delay. All figures stipulated.
    cases=[('Human baseline',6.,0.,1.,.95,.98,2.),
      ('AI saves effort and time',2.,1.,1.5,.90,.98,2.),
      ('AI saves effort; slow agent',2.,8.,1.5,.90,.98,2.),
      ('AI shifts work into review',1.,.5,7.,.75,.90,2.)]
    rows=[]
    for name,c,g,v,p,r,w in cases:
        h=c+v; tau=max(c,g)+v
        H=h/p; L=tau/p+w; S=10.
        rows.append(dict(scenario=name,production_h=c,agent_h_per_attempt=g,agent_h_total=g/p,review_h=v,pass_probability=p,
          retention=r,queue_h=w,human_h=H,elapsed_h=L,retained_reference_units=S*r,
          effort_productivity=S*r/H,serial_delivery_rate=S*r/L))
    with (OUT/'synthetic-scenarios.csv').open('w',newline='') as f:
        writer=csv.DictWriter(f,fieldnames=list(rows[0]));writer.writeheader();writer.writerows(rows)
    fig,ax=plt.subplots(figsize=(7.1,4.4))
    for i,row in enumerate(rows):
        x=row['human_h']/rows[0]['human_h']; y=row['elapsed_h']/rows[0]['elapsed_h']
        ax.scatter(x,y,s=90,zorder=3,color=['#283e51','#087e8b','#a85d10','#a53543'][i])
        offsets=[(8,-25),(8,-25),(8,12),(-156,12)]
        ax.annotate(row['scenario'],(x,y),textcoords='offset points',xytext=offsets[i],fontsize=9)
    ax.axvline(1,color='gray',lw=.8);ax.axhline(1,color='gray',lw=.8)
    ax.set(xlabel='Human effort / baseline effort',ylabel='Elapsed delivery time / baseline time',
           xlim=(.35,1.65),ylim=(.45,1.65),title='Less human work does not guarantee earlier delivery')
    ax.grid(alpha=.15);fig.text(.99,.01,'Synthetic assumptions; not company estimates',ha='right',fontsize=8,color='gray');fig.tight_layout(rect=(0,.025,1,1));fig.savefig(FIG/'effort-versus-time.png');fig.savefig(FIG/'effort-versus-time.pdf');plt.close(fig)
    return rows

def detection_plot():
    sens=np.linspace(0,1,10001);d=.4;h=2.;tau=2.;S=10.
    p=1-d*sens;rho=(1-d)/p
    fig,axs=plt.subplots(1,2,figsize=(8.3,3.7))
    axs[0].plot(sens,S*p/tau,label='Recorded accepted output',color='#a53543')
    axs[0].plot(sens,S*rho*p/tau,label='Actually correct output',color='#087e8b')
    axs[0].set(xlabel='Defect detection sensitivity',ylabel='Reference units per elapsed hour',title='A weaker gate can look faster')
    axs[0].legend(fontsize=8)
    axs[1].plot(sens,rho,color='#283e51')
    axs[1].set(xlabel='Defect detection sensitivity',ylabel='P(correct | passed)',ylim=(.5,1.03),title='The accepted work becomes less reliable')
    for ax in axs:ax.grid(alpha=.2)
    fig.text(.99,.01,'Synthetic model: d=0.4; two-hour attempts; no queue',ha='right',fontsize=8,color='gray');fig.tight_layout(rect=(0,.035,1,1));fig.savefig(FIG/'detection-and-speed.png');fig.savefig(FIG/'detection-and-speed.pdf');plt.close(fig)
    return {'defect_probability':d,'attempt_hours':tau,'reference_units':S,'grid_size':len(sens),
      'sensitivity_1':{'pass':.6,'correct_given_pass':1.,'recorded_rate':3.,'correct_rate':3.},
      'sensitivity_0':{'pass':1.,'correct_given_pass':.6,'recorded_rate':5.,'correct_rate':3.}}

def retries(rows):
    result=[]
    for seed in [17,41,93]:
      rng=np.random.default_rng(seed)
      for r in rows:
        p=r['pass_probability'];h=r['production_h']+r['review_h'];tau=max(r['production_h'],r['agent_h_per_attempt'])+r['review_h']
        n=100000; attempts=rng.geometric(p,n)
        mc=float(np.mean(h*attempts));truth=h/p
        se=h*math.sqrt((1-p)/(p*p*n))
        result.append({'seed':seed,'scenario':r['scenario'],'n':n,'mean_h':mc,'exact_mean_h':truth,
          'mc_error_h':mc-truth,'analytic_standard_error_h':se,
          'chebyshev_abs_error_bound_at_99_percent_h':10*se})
    return result

def benchmark():
    # Optional complete-case prediction demonstrator, not a causal estimator.
    # Fixed 12 features (includes intercept); both log outcomes share X.
    result=[]
    for n in [1000,10000,100000]:
        rng=np.random.default_rng(17);tim={};t=time.perf_counter()
        X=rng.normal(size=(n,12));X[:,0]=1
        coef=np.linspace(-.2,.2,24).reshape(12,2)
        Y=X@coef+rng.normal(scale=.25,size=(n,2));tim['assembly']=time.perf_counter()-t
        t=time.perf_counter();Q,R=np.linalg.qr(X,mode='reduced');B=np.linalg.solve(R,Q.T@Y);tim['calibration']=time.perf_counter()-t
        t=time.perf_counter();pred=X@B;tim['prediction']=time.perf_counter()-t
        t=time.perf_counter();res=Y-pred;sigma=(res.T@res)/(n-12)
        Ri=np.linalg.solve(R,np.eye(12));vc=Ri@Ri.T
        mean_variance=np.einsum('ij,jk,ik->i',X,vc,X)[:,None]*np.diag(sigma)[None,:]
        tim['uncertainty']=time.perf_counter()-t
        t=time.perf_counter();lo=pred-1.96*np.sqrt(mean_variance);hi=pred+1.96*np.sqrt(mean_variance)
        both_log_mean_upper_bounds_below_zero=(hi[:,0]<0)&(hi[:,1]<0);tim['decision_scan']=time.perf_counter()-t
        condition=float(np.linalg.cond(R));normal_res=float(np.linalg.norm(X.T@res)/max(1,np.linalg.norm(X.T@Y)))
        assert condition<1e8 and normal_res<1e-10
        result.append({'n':n,'p':12,'seconds':tim,'peak_rss_mb':resource.getrusage(resource.RUSAGE_SELF).ru_maxrss/1024,
          'condition_R':condition,'normal_equation_relative_residual':normal_res,
          'max_coefficient_error':float(np.max(np.abs(B-coef))),
          'status':'Synthetic complete uncensored data only; intervals assume iid homoscedastic Gaussian errors; no causal interpretation'})
    return result

def cpu_name():
    path=Path('/proc/cpuinfo')
    if path.exists():
        for line in path.read_text().splitlines():
            if line.startswith('model name'):return line.split(':',1)[1].strip()
    return platform.processor() or 'unavailable'

def main():
    start=time.perf_counter();checks=exact_checks();rows=make_scenarios();detect=detection_plot();mc=retries(rows);scaling=benchmark()
    res={'synthetic_only':True,'checks':checks,'scenarios':rows,'detection':detect,'monte_carlo':mc,'scaling':scaling,
         'environment':{'python':platform.python_version(),'numpy':np.__version__,'machine':platform.machine(),'platform':platform.platform(),'cpu':cpu_name(),'declared_thread_environment':{k:os.environ.get(k,'unset') for k in ['OPENBLAS_NUM_THREADS','OMP_NUM_THREADS','MKL_NUM_THREADS']}},
         'total_seconds':time.perf_counter()-start}
    (EV/'results.json').write_text(json.dumps(res,indent=2))
    print(json.dumps({'checks':checks,'scenarios':rows,'scaling':scaling,'total_seconds':res['total_seconds']},indent=2))
if __name__=='__main__':main()
