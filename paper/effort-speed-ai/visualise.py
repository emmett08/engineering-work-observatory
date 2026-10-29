#!/usr/bin/env python3
"""Publication figures for the synthetic effort/time methodological examples.

Run: python visualise.py
Dependencies: Python 3.10+, numpy, matplotlib, Pillow, PyMuPDF.
Reads evidence/results.json; changes neither those results nor simulate.py.
All PDFs are exactly 6 inches wide. All displayed ordinary text is >= 8 pt.
"""
from pathlib import Path
from fractions import Fraction as F
import csv
import hashlib
import json
import platform

import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import PercentFormatter
from PIL import Image
import fitz

ROOT = Path(__file__).resolve().parent
OUT = ROOT / "figures-v2"
OUT.mkdir(exist_ok=True)
QA = OUT / "inspection"
QA.mkdir(exist_ok=True)
RESULTS = ROOT / "evidence" / "results.json"
DATA = json.loads(RESULTS.read_text())
INK, MUTED, GRID = "#203038", "#5b666a", "#c7cdd0"
HUMAN, AGENT, ADVERSE = "#007c83", "#626b75", "#aa4b22"
plt.rcParams.update({
    "font.family": "DejaVu Sans", "font.size": 9,
    "axes.labelsize": 9, "axes.titlesize": 10,
    "xtick.labelsize": 8.5, "ytick.labelsize": 8.5,
    "text.color": INK, "axes.labelcolor": INK,
    "xtick.color": INK, "ytick.color": INK,
    "axes.spines.top": False, "axes.spines.right": False,
    "axes.linewidth": .6, "axes.edgecolor": MUTED,
    "xtick.major.width": .6, "ytick.major.width": .6,
    "pdf.fonttype": 42, "ps.fonttype": 42,
    "savefig.facecolor": "white", "figure.facecolor": "white",
})
ARTIFACTS = []


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dump(name, value):
    (OUT / name).write_text(json.dumps(value, indent=2) + "\n")


def title(fig, heading, qualifier="Synthetic construction · no company observations"):
    fig.text(.105, .964, heading, fontsize=11, weight="bold", va="top")
    fig.text(.105, .909, qualifier, fontsize=8.5, color=MUTED, va="top")


def axis_style(ax, grid="y"):
    ax.set_axisbelow(True)
    ax.grid(axis=grid, color=GRID, linewidth=.45, alpha=.7)
    ax.tick_params(length=3)


def export(fig, name, selected=False):
    assert abs(fig.get_figwidth() - 6) < 1e-10
    fig.canvas.draw()
    minimum_font = min(t.get_fontsize() for t in fig.findobj(matplotlib.text.Text)
                       if t.get_visible() and t.get_text())
    assert minimum_font >= 8
    path = OUT / f"{name}.pdf"
    fig.savefig(path)
    fig.savefig(OUT / f"{name}.png", dpi=240)
    # Inspect actual vector exports, not a different plotting path.
    doc = fitz.open(path)
    pg = doc[0]
    actual_width = pg.rect.width / 72
    assert abs(actual_width - 6) < .001
    pix = pg.get_pixmap(matrix=fitz.Matrix(1.5, 1.5), alpha=False)
    pix.save(QA / f"{name}-108dpi.png")
    img = Image.open(QA / f"{name}-108dpi.png")
    img.convert("L").save(QA / f"{name}-greyscale.png")
    img.resize((324, round(img.height * 324 / img.width))).save(QA / f"{name}-thumbnail.png")
    blocks = pg.get_text("dict")["blocks"]
    spans = [sp for b in blocks if b["type"] == 0 for line in b["lines"] for sp in line["spans"]]
    small = [sp["text"] for sp in spans if sp["size"] < 7.99]
    outside = [sp["text"] for sp in spans if not pg.rect.contains(fitz.Rect(sp["bbox"]))]
    assert not small, small
    assert not outside, outside
    ARTIFACTS.append({"name":name,"selected":selected,
                      "pdf":str(path.relative_to(ROOT)),"pdf_sha256":sha(path),
                      "width_inches":actual_width,"height_inches":pg.rect.height/72,
                      "minimum_source_font_pt":minimum_font,
                      "minimum_extracted_font_pt":min(s["size"] for s in spans),
                      "outside_page_text":outside, "pdf_images":len(pg.get_images()),
                      "png_dpi":240})
    plt.close(fig)


def numerical_checks():
    """Independent rational re-evaluation; no import from simulate.py."""
    out=[]
    for row in DATA["scenarios"]:
        c,g,v,p,r,w=[F(str(row[k])) for k in
          ["production_h","agent_h_per_attempt","review_h","pass_probability","retention","queue_h"]]
        h=c+v; t=max(c,g)+v; H=h/p; L=w+t/p
        P=10*r/H; V=10*r/L
        for key,value in [("human_h",H),("elapsed_h",L),("effort_productivity",P),("serial_delivery_rate",V)]:
            assert abs(float(value)-row[key]) < 1e-12
        out.append({"scenario":row["scenario"],"h":str(h),"t":str(t),"H":str(H),"L":str(L),"P":str(P),"V":str(V)})
    H0,L0=F(out[0]["H"]),F(out[0]["L"])
    for q in out:
        q["H_over_H0"]=str(F(q["H"])/H0)
        q["L_over_L0"]=str(F(q["L"])/L0)
    detection=[]
    for s in [F(0),F(1,4),F(1,2),F(3,4),F(1)]:
        correct=F(3,5)
        p=correct+(1-correct)*(1-s)
        rho=correct/p
        recorded=10*p/2
        rate_correct=10*p*rho/2
        assert recorded == 5-2*s and rate_correct == 3
        detection.append({"s":str(s),"pass":str(p),"rho":str(rho),"recorded":str(recorded),"correct":str(rate_correct)})
    # Independent rectangle-area and dependency-path checks for one attempt.
    labour_area=sum((b-a)*allocation for a,b,allocation in [(F(0),F(2),F(1)),(F(2),F(8),F(0)),(F(8),F(19,2),F(1))])
    assert labour_area == F(7,2)
    paths=[F(2)+F(3,2),F(8)+F(3,2)]
    assert max(paths)==F(19,2)
    report={"status":"pass", "method":"Exact Fraction arithmetic, independent of simulate.py; comparison against retained inputs.",
            "scope":"Finite example checks, not universal proofs or empirical validation.",
            "scenario_checks":out, "detection_checks":detection,
            "schedule_area_person_hours":str(labour_area),"schedule_longest_path_hours":str(max(paths)),
            "grid_monotonicity": "recorded decreases; correct rate constant; accepted correctness increases for d=0.4"}
    dump("numerical-checks.json",report)
    return out


def schedule():
    fig=plt.figure(figsize=(6,4.15))
    title(fig,"Elapsed time includes unattended agent work")
    ax=fig.add_axes([.145,.555,.805,.27])
    ax.set_xlim(0,10);ax.set_ylim(-.45,1.75)
    ax.hlines(1,0,2,color=HUMAN,lw=5)
    ax.hlines(1,8,9.5,color=HUMAN,lw=5)
    ax.hlines(0,0,8,color=AGENT,lw=4,linestyle=(0,(4,2)))
    ax.text(1,1.26,"Production\n2 h",ha="center",va="bottom",fontsize=8.5)
    ax.text(8.75,1.26,"Verification\n1.5 h",ha="center",va="bottom",fontsize=8.5)
    ax.text(4,.20,"Unattended execution: 8 h",ha="center",fontsize=8.5,color=AGENT)
    ax.set_yticks([0,1],["Agent","Human"])
    ax.tick_params(axis="y",length=0,pad=10)
    ax.tick_params(axis="x",bottom=False,labelbottom=False)
    for sp in ax.spines.values():sp.set_visible(False)
    ax.axvline(8,color=GRID,lw=.7,linestyle=(0,(2,3)))
    ax.vlines(9.5,-.42,1.06,color=MUTED,lw=.7)
    ax2=fig.add_axes([.145,.185,.805,.285],sharex=ax)
    for a,b in [(0,2),(8,9.5)]:
        ax2.fill_between([a,b],[1,1],0,color=HUMAN,alpha=.16,linewidth=0)
        ax2.hlines(1,a,b,color=HUMAN,lw=1.5)
        ax2.vlines([a,b],0,1,color=HUMAN,lw=.8)
    ax2.hlines(0,2,8,color=HUMAN,lw=1.5)
    ax2.set_ylim(0,1.3);ax2.set_yticks([0,1])
    ax2.set_ylabel("Human allocation\n(persons)",labelpad=8)
    ax2.set_xticks([0,2,4,6,8,9.5],["0","2","4","6","8","9.5"])
    ax2.set_xlabel("Elapsed time within one attempt (hours)",labelpad=8)
    ax2.text(5,.65,"Area = 3.5 person-hours",ha="center",color=HUMAN,fontsize=9)
    ax2.axvline(9.5,color=MUTED,lw=.7)
    fig.text(.145,.042,"Independent production branches; verification follows both. Finish: 9.5 h.",fontsize=8.5)
    export(fig,"01-work-and-time",True)

    # Encoding alternative: cumulative labour makes pauses clear but suppresses
    # agent duration unless supplied verbally; retained for honest comparison.
    fig=plt.figure(figsize=(6,3.5))
    title(fig,"Human labour can pause while elapsed time grows")
    a=fig.add_axes([.13,.19,.81,.63])
    a.plot([0,2,8,9.5],[0,2,2,3.5],color=HUMAN,lw=2)
    a.axvline(9.5,color=MUTED,lw=.7,linestyle="--")
    a.text(4.7,2.16,"Agent continues; no human labour",ha="center",fontsize=8.5)
    a.text(9.25,3.58,"Finish",ha="right",fontsize=8.5)
    a.set(xlim=(0,10),ylim=(0,4.1),xlabel="Elapsed time within one attempt (hours)",ylabel="Accumulated human labour (person-hours)")
    a.set_xticks([0,2,4,6,8,9.5]);a.set_yticks([0,1,2,3,4]);axis_style(a)
    export(fig,"alternative-01-cumulative-labour")


def scenarios():
    rows=DATA["scenarios"]
    x=np.array([r["human_h"]/rows[0]["human_h"] for r in rows])
    y=np.array([r["elapsed_h"]/rows[0]["elapsed_h"] for r in rows])
    labels=["Human baseline","Fast agent","Slow agent","Heavy review"]
    colours=[INK,HUMAN,HUMAN,ADVERSE]
    markers=["D","o","o","^"]
    fig=plt.figure(figsize=(6,4.35))
    title(fig,"Less labour need not mean earlier acceptance",
          "Synthetic expected totals · first acceptance · human baseline = 1")
    ax=fig.add_axes([.14,.17,.82,.665])
    ax.axvline(1,color=MUTED,lw=.7,linestyle=(0,(3,3)))
    ax.axhline(1,color=MUTED,lw=.7,linestyle=(0,(3,3)))
    offsets=[(8,-16),(8,-16),(8,10),(-8,-18)]
    for i in range(4):
        ax.scatter(x[i],y[i],marker=markers[i],s=44,color=colours[i],zorder=3)
        ax.annotate(labels[i],(x[i],y[i]),xytext=offsets[i],textcoords="offset points",
                    ha="right" if i==3 else "left",fontsize=9)
    ax.set(xlim=(0,1.68),ylim=(0,1.68),xlabel="Expected human labour / baseline labour",ylabel="Expected elapsed time / baseline time")
    ax.set_xticks([0,.5,1,1.5]);ax.set_yticks([0,.5,1,1.5])
    ax.text(.03,.975,"Longer elapsed time",transform=ax.transAxes,va="top",fontsize=8.5,color=MUTED)
    fig.text(.14,.037,"Same 2-hour initial wait in every case; no follow-up repair labour.",fontsize=8.5)
    export(fig,"02-effort-versus-time",True)

    fig=plt.figure(figsize=(6,3.45))
    title(fig,"Labour and elapsed time can move apart",
          "Synthetic expected totals · first acceptance · human baseline = 1")
    ax=fig.add_axes([.255,.25,.685,.55])
    for i in range(4):
        ax.hlines(3-i,min(x[i],y[i]),max(x[i],y[i]),color=GRID,lw=2)
    ax.scatter(x,np.arange(3,-1,-1),s=42,facecolors="white",edgecolors=HUMAN,label="Human labour",zorder=3)
    ax.scatter(y,np.arange(3,-1,-1),s=42,marker="x",color=INK,label="Elapsed time",zorder=4)
    ax.axvline(1,color=MUTED,lw=.7,linestyle=(0,(3,3)))
    ax.set_yticks(range(4),labels[::-1]);ax.tick_params(axis="y",length=0,pad=7)
    ax.set(xlim=(0,1.65),ylim=(-.65,3.45),xlabel="Expected total / human baseline")
    ax.set_xticks([0,.5,1,1.5]);ax.spines["left"].set_visible(False)
    fig.legend(*ax.get_legend_handles_labels(),loc="lower center",bbox_to_anchor=(.58,.005),frameon=False,ncols=2,fontsize=8.5)
    export(fig,"alternative-02-paired-dots")


def detection():
    d=DATA["detection"]["defect_probability"]
    t=DATA["detection"]["attempt_hours"]
    S=DATA["detection"]["reference_units"]
    s=np.linspace(0,1,10001);p=1-d*s;rho=(1-d)/p
    recorded=S*p/t;correct=S*(1-d)/t*np.ones_like(s)
    assert np.all(np.diff(recorded)<0) and np.all(np.diff(rho)>0)
    assert np.allclose(S*p*rho/t,correct,rtol=0,atol=1e-14)
    fig=plt.figure(figsize=(6,3.6))
    title(fig,"A weaker gate can raise recorded throughput",
          "Synthetic derivation · defect probability 0.4 · no queue or repair")
    axes=[fig.add_axes([.12,.20,.345,.59]),fig.add_axes([.62,.20,.345,.59])]
    a,b=axes
    a.plot(s,recorded,color=ADVERSE,lw=1.8)
    a.plot(s,correct,color=HUMAN,lw=1.8,linestyle=(0,(4,2)))
    a.text(.07,4.93,"Recorded",color=ADVERSE,fontsize=9)
    a.text(.07,2.57,"Correct",color=HUMAN,fontsize=9)
    a.set(xlim=(0,1),ylim=(0,5.5),ylabel="Reference units per elapsed hour")
    a.set_yticks([0,1,2,3,4,5]);a.set_title("a  Output rate",loc="left",pad=9)
    b.plot(s,rho,color=INK,lw=1.8)
    b.set(xlim=(0,1),ylim=(0,1.1),ylabel="Correctness among accepted tasks")
    b.set_yticks([0,.2,.4,.6,.8,1]);b.yaxis.set_major_formatter(PercentFormatter(1,decimals=0))
    b.set_title("b  Accepted correctness",loc="left",pad=9)
    for ax in axes:
        ax.set_xticks([0,.5,1]);ax.set_xlabel("Detection sensitivity",labelpad=7);axis_style(ax)
    fig.text(.12,.035,"Ten reference units per task; two-hour attempts; no false-positive rejection.",fontsize=8.5)
    export(fig,"03-detection-and-speed",True)

    # Alternative: derived endpoint pairs, avoiding a curve where only the two
    # extreme gates matter. This discards the interior sensitivity shape.
    fig=plt.figure(figsize=(6,3.6))
    title(fig,"The gate changes acceptance and accepted quality",
          "Synthetic endpoint comparison · sensitivity 0 versus sensitivity 1")
    a=fig.add_axes([.13,.20,.34,.59]);b=fig.add_axes([.62,.20,.34,.59])
    a.plot([0,1],[recorded[0],recorded[-1]],"o-",color=ADVERSE,lw=1.5)
    a.plot([0,1],[correct[0],correct[-1]],"x--",color=HUMAN,lw=1.5)
    a.text(.1,5.07,"Recorded",fontsize=9,color=ADVERSE)
    a.text(.1,2.55,"Correct",fontsize=9,color=HUMAN)
    b.plot([0,1],[rho[0],rho[-1]],"o-",color=INK,lw=1.5)
    a.set(ylabel="Reference units per elapsed hour",ylim=(0,5.6));a.set_yticks([0,1,2,3,4,5])
    b.set(ylabel="Correctness among accepted tasks",ylim=(0,1.1));b.set_yticks([0,.2,.4,.6,.8,1]);b.yaxis.set_major_formatter(PercentFormatter(1,decimals=0))
    for ax in (a,b):
        ax.set_xlim(-.12,1.12);ax.set_xticks([0,1],["0","1"]);ax.set_xlabel("Detection sensitivity");axis_style(ax)
    fig.text(.13,.035,"Points are exact endpoints; connectors show direction, not the interior function.",fontsize=8)
    export(fig,"alternative-03-endpoint-pairs")


CAPTIONS={
"01-work-and-time": "One synthetic attempt with independent human and agent production branches. Human production takes 2 hours, unattended agent execution takes 8 hours, and human verification takes 1.5 hours after both finish. The lower panel integrates human allocation: h = 2 + 1.5 = 3.5 person-hours. The elapsed critical path is t = max(2, 8) + 1.5 = 9.5 hours. Agent execution adds no human labour during the unattended interval. This drawing excludes the initial queue, retries and follow-up repair; it is a constructed schedule, not an observed workflow.",
"02-effort-versus-time": "Four synthetic workflows, with expected human labour to first acceptance and expected elapsed time to first acceptance divided by their respective human-baseline means (7.3684 person-hours and 9.3684 hours). Sequential independent attempts continue to the first pass; each scenario has the same one-off 2-hour initial wait. Fast and slow agents require equal expected human labour, but the slow agent increases elapsed time above baseline. Heavy review increases both totals. Values are deterministic model expectations, not estimates or confidence intervals; follow-up repair labour is excluded. Scenario parameters and exact totals are in the accompanying table.",
"03-detection-and-speed": "Detection sensitivity s varies from 0 to 1 in a synthetic model with independent attempts, defect probability d = 0.4, no false-positive rejection, ten reference units per task, and two elapsed hours per attempt. Sequential retries stop at the first pass; queueing and subsequent repair are excluded. Recorded accepted throughput is 5 − 2s reference units per hour, correct-output throughput is 3 units per hour, and correctness conditional on acceptance is 0.6/(1 − 0.4s). Weakening detection increases recorded throughput and reduces accepted correctness, while correct-output throughput is unchanged under these particular assumptions. Curves are derived functions, not observed effects; correctness is distinct from the article’s retained-functionality fraction."
}
ALTTEXT={
"01-work-and-time":"Human production occupies hours zero to two; agent work runs concurrently from zero to eight. Human verification follows from eight to nine and a half. A separate human-allocation trace has area three and a half person-hours despite nine and a half elapsed hours.",
"02-effort-versus-time":"A scatter of four synthetic cases indexed to human baseline one on both axes. Fast agent: labour 0.528, time 0.629. Slow agent: the same labour 0.528, time 1.340. Heavy review: labour 1.448, time 1.352. The slow agent saves labour but delays acceptance.",
"03-detection-and-speed":"As detection sensitivity rises from zero to one, recorded throughput falls from five to three reference units per hour, correct-output throughput stays at three, and the probability an accepted task is correct rises from sixty to one hundred percent."
}


def records(exact):
    for name,caption in CAPTIONS.items():
        (OUT/f"{name}.caption.txt").write_text(caption+"\n")
        (OUT/f"{name}.alt.txt").write_text(ALTTEXT[name]+"\n")
    specs=[
        {"id":"01-work-and-time","reader_question":"Why can elapsed duration exceed accounted human labour?",
         "reading_task":"Compare a lawful concurrent schedule with the integral of human allocation.",
         "source_status":"illustrative construction derived from the retained slow-agent scenario",
         "quantities":{"x":"elapsed hours within one attempt","top_y":"resource category, not numerical magnitude","bottom_y":"human allocation, persons","area":"human labour, person-hours"},
         "domain":{"x":[0,10],"allocation":[0,1.3]},
         "claim":"3.5 person-hours and 9.5 elapsed hours describe the same attempt.",
         "conditions":["one human","independent production branches","unattended agent execution","verification follows both","no queue, retry or follow-up repair in this drawing"],
         "alternative":"alternative-01-cumulative-labour",
         "selection":"Schedule plus allocation retains concurrency and the area interpretation; cumulative labour clearly shows a plateau but requires prose to recover agent duration and the prerequisite for verification.",
         "encoding":"Segment position and length encode start and duration; direct resource labels and dashed agent strokes preserve greyscale; filled area has explicit integral semantics, not a diagram container."},
        {"id":"02-effort-versus-time","reader_question":"Can expected human labour fall while expected acceptance time rises?",
         "reading_task":"Locate four paired model expectations relative to the human baseline on both outcomes.",
         "source_status":"derived expectations from synthetic input parameters",
         "quantities":{"x":"H/H0, dimensionless human-labour ratio","y":"L/L0, dimensionless elapsed-time ratio","point":"one stipulated workflow"},
         "domain":{"x":[0,1.68],"y":[0,1.68]},
         "reference":{"H0_person_hours":"140/19","L0_hours":"178/19","baseline_ratio":1},
         "claim":"The two agent-duration cases have equal labour but straddle baseline elapsed time; heavy review worsens both.",
         "conditions":["independent identical sequential attempts until first pass within each case","constant per-attempt costs","one-off initial wait 2 hours","no follow-up repair labour"],
         "alternative":"alternative-02-paired-dots",
         "selection":"Scatter exposes shared labour coordinates and policy trade-offs directly; paired dots support easier separate magnitude lookup but make the two-dimensional comparison less immediate. Both keep baseline and adverse heavy-review case.",
         "encoding":"Paired position on two linear zero-origin ratio axes; dashed reference lines at one; category identity direct-labelled. No joining line implies causal or chronological progression."},
        {"id":"03-detection-and-speed","reader_question":"How can detection change recorded throughput without changing correct-output throughput?",
         "reading_task":"Read throughput and accepted correctness as functions of detection sensitivity under one fixed synthetic model.",
         "source_status":"derived mathematical functions with stipulated parameters",
         "quantities":{"x":"sensitivity s, probability","left_y":"reference units per elapsed hour","right_y":"P(correct | accepted), percentage"},
         "domain":{"s":[0,1],"rate":[0,5.5],"correctness":[0,1.1]},
         "fixed":{"d":0.4,"attempt_elapsed_hours":2,"reference_units_per_task":10,"queue":0,"followup_repair":0},
         "formulae":{"recorded":"5 - 2*s","correct":"3","accepted_correctness":"0.6 / (1 - 0.4*s)"},
         "claim":"Weakening detection raises recorded throughput and lowers accepted correctness; correct-output throughput stays constant in this model.",
         "conditions":["independent attempts","fixed costs","no false-positive rejection","retry to first pass"],
         "alternative":"alternative-03-endpoint-pairs",
         "selection":"Curves retain the known nonlinear correctness function over the full valid domain. Endpoint pairs reduce decoding but omit the interior sensitivity shape and their connector must not be mistaken for interpolation.",
         "encoding":"Common sensitivity domain in aligned panels; separate y scales for different units; direct curve labels and dashed correct-throughput curve preserve greyscale."}
    ]
    for spec in specs:
        name=spec["id"]
        spec.update({"route":"Python / Matplotlib; not TeX-schema validation",
          "uncertainty":"Not applicable to deterministic synthetic expectations; parameter uncertainty and model inadequacy are not quantified.",
          "missingness":"None; no empirical observations supplied, removed or imputed.",
          "permitted_inference":"Consequences of the stipulated model only; no estimated company effect, causal identification or empirical calibration.",
          "target":{"width_inches":6,"minimum_text_pt":8,"font":"DejaVu Sans","minimum_nonzero_stroke_pt":0.45},
          "source":{"path":"evidence/results.json","sha256":sha(RESULTS)},
          "caption":{"path":f"figures-v2/{name}.caption.txt","sha256":sha(OUT/f"{name}.caption.txt")},
          "build":["python","visualise.py"]})
        dump(f"{name}.spec.json",spec)
    with (OUT/"exact-values.csv").open("w",newline="") as f:
        w=csv.DictWriter(f,fieldnames=list(exact[0]));w.writeheader();w.writerows(exact)
    dump("manifest.json",{"source_input":{"path":"evidence/results.json","sha256":sha(RESULTS),"status":"synthetic"},
         "script":{"path":"visualise.py","sha256":sha(Path(__file__))},
         "dependencies":{"python":platform.python_version(),"numpy":np.__version__,"matplotlib":matplotlib.__version__},
         "artifacts":ARTIFACTS,"recipe":["python","visualise.py"],
         "mechanical_status":"pass: exact six-inch widths; vector PDFs; fonts embedded; extracted text within page bounds and >=8pt",
         "mechanical_limit":"Bounds and font checks do not establish semantic validity; independent rendered review is recorded separately."})
    (OUT/"README.md").write_text("# Reproducing the figures\n\nRun `python visualise.py` from the project directory. Required packages: numpy, matplotlib, Pillow, PyMuPDF. The script reads `evidence/results.json`, checks the examples using independent exact-rational arithmetic, and exports six-inch vector PDFs and PNGs. Files beginning `01`, `02` and `03` are selected; `alternative-` files retain compared encodings. Captions and alternative text are separate. `inspection/` contains actual PDF renders at 108 dpi (six inches = 648 pixels), greyscale and thumbnails.\n\nAll examples are synthetic. Formula checks cover the supplied finite cases; no figure is an empirical estimate. The PDF inspection is a Python-specific equivalent audit, not the skill’s TeX schema/audit workflow. Final manuscript integration remains the coordinator’s responsibility.\n")


def main():
    assert DATA["synthetic_only"] is True
    exact=numerical_checks()
    schedule();scenarios();detection();records(exact)
    print(json.dumps({"output":str(OUT),"figures":len(ARTIFACTS),"mechanical_checks":"pass","numerical_checks":"pass"}))


if __name__=="__main__":main()
