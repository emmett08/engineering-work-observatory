"""Probability samples, bounded effort observations and an offline review form.

The sample is a reproducible pseudorandom SRS draw; a retained seed and hash do
not establish that somebody did not shop among draws. Response minutes are
resolved to the nearest microsecond, then conserved against the interval exactly.
Only validated observations enter the existing Rust estimator's outcome fields.
"""
from __future__ import annotations

from datetime import datetime, timezone
from decimal import Decimal, InvalidOperation, ROUND_HALF_EVEN
from fractions import Fraction
import hashlib
import json
import math
import re
import secrets

from .common import canonical_hash, format_time, parse_time, validate_frame

SAMPLER = "sha256-fisher-yates-v1"
ROLES = ("specification", "implementation", "review", "testing", "coordination", "repair", "other")
REASONS = ("not_answered", "cannot_recall", "allocation_uncertain", "outside_declared_scope")
MICROS_PER_MINUTE = 60_000_000
MICROS_PER_HOUR = 3_600_000_000


def _keys(value, required, label):
    if not isinstance(value, dict) or set(value) != set(required):
        raise ValueError(f"{label} requires exactly: {', '.join(sorted(required))}")


def _integer(value, label, minimum=1):
    if type(value) is not int or value < minimum:
        raise ValueError(f"{label} must be an integer >= {minimum}")
    return value


def _duration_micros(unit):
    delta = parse_time(unit["end"]) - parse_time(unit["start"])
    result = (delta.days * 86_400 + delta.seconds) * 1_000_000 + delta.microseconds
    if result <= 0:
        raise ValueError("interval must have positive duration")
    return result


def _minutes_micros(value, label):
    try:
        valid = type(value) in (int, float) and math.isfinite(value) and value >= 0
    except OverflowError:
        valid = False
    if not valid:
        raise ValueError(f"{label} must be a finite nonnegative number of minutes")
    try:
        exact = Decimal(str(value)) * MICROS_PER_MINUTE
        return int(exact.to_integral_value(rounding=ROUND_HALF_EVEN))
    except (InvalidOperation, OverflowError) as exc:
        raise ValueError(f"invalid {label}") from exc


def _hours(micros):
    """Represent observed time without rounding above its exact physical value.

    This is float representation handling, not clipping an estimated total.
    """
    exact = Fraction(micros, MICROS_PER_HOUR)
    result = float(exact)
    if Fraction.from_float(result) > exact:
        result = math.nextafter(result, -math.inf)
    return result


def _seed_bytes(seed):
    if not isinstance(seed, str) or not re.fullmatch(r"[0-9a-f]{64}", seed):
        raise ValueError("sample seed must be 64 lowercase hexadecimal characters")
    return bytes.fromhex(seed)


def _selected_ids(frame, size, seed):
    """Versioned Fisher--Yates using SHA-256 counter blocks and rejection draws.

    No Python random implementation details or input order affect replay.
    """
    source = _seed_bytes(seed)
    population = sorted(unit["unit_id"] for unit in frame["units"])
    counter = 0
    for i in range(size):
        bound = len(population) - i
        limit = (1 << 256) - (1 << 256) % bound
        while True:
            digest = hashlib.sha256(b"observatory-effort-sample-v1\0" + source + counter.to_bytes(16, "big")).digest()
            counter += 1
            draw = int.from_bytes(digest, "big")
            if draw < limit:
                j = i + draw % bound
                break
        population[i], population[j] = population[j], population[i]
    return sorted(population[:size])


def draw_sample(frame, sample_size, seed=None):
    validate_frame(frame)
    size = _integer(sample_size, "sample_size")
    population = len(frame["units"])
    if size > population:
        raise ValueError("sample_size exceeds the complete declared frame")
    synthetic = frame.get("synthetic", False) or seed is not None
    seed = secrets.token_hex(32) if seed is None else seed
    _seed_bytes(seed)
    created_at = format_time(datetime.now(timezone.utc))
    if parse_time(created_at) < parse_time(frame["frozen_at"]):
        raise ValueError("cannot sample before frame predictions were frozen")
    return {
        "schema": "observatory.effort-sample/1",
        "frame_sha256": canonical_hash(frame),
        "frame_id": frame["frame_id"],
        "created_at": created_at,
        "sampler": SAMPLER,
        "synthetic": synthetic,
        "design": {"kind": "simple_random_without_replacement", "population_size": population, "sample_size": size},
        "seed": seed,
        "selected_unit_ids": _selected_ids(frame, size, seed),
        "inclusion_probability": size / population,
    }


def validate_sample(frame, sample):
    validate_frame(frame)
    _keys(sample, {"schema", "frame_sha256", "frame_id", "created_at", "sampler", "synthetic", "design", "seed", "selected_unit_ids", "inclusion_probability"}, "sample")
    if sample["schema"] != "observatory.effort-sample/1" or sample["sampler"] != SAMPLER:
        raise ValueError("unsupported sample schema or sampler")
    if type(sample["synthetic"]) is not bool:
        raise ValueError("sample synthetic marker must be boolean")
    if frame.get("synthetic", False) and not sample["synthetic"]:
        raise ValueError("sample cannot remove the frozen frame synthetic marker")
    if sample["frame_sha256"] != canonical_hash(frame) or sample["frame_id"] != frame["frame_id"]:
        raise ValueError("sample does not bind to this frozen frame")
    if parse_time(sample["created_at"]) < parse_time(frame["frozen_at"]):
        raise ValueError("sample predates frozen predictions")
    _keys(sample["design"], {"kind", "population_size", "sample_size"}, "sample design")
    design = sample["design"]
    if design["kind"] != "simple_random_without_replacement":
        raise ValueError("only simple random sampling without replacement is supported")
    population = _integer(design["population_size"], "population_size")
    size = _integer(design["sample_size"], "sample_size")
    if population != len(frame["units"]) or size > population:
        raise ValueError("sample design does not match frame population")
    probability = sample["inclusion_probability"]
    if type(probability) not in (float, int) or probability != size / population:
        raise ValueError("sample inclusion probability does not match actual draw")
    if sample["selected_unit_ids"] != _selected_ids(frame, size, sample["seed"]):
        raise ValueError("selected units do not replay from the retained sample seed")
    return sample


def _person_selection(frame, sample, person_id=None):
    if person_id is not None and person_id not in {person["person_id"] for person in frame["frames"]}:
        raise ValueError("review person is not in the frozen frame")
    allowed = {unit["unit_id"] for unit in frame["units"] if person_id is None or unit["person_id"] == person_id}
    return [unit_id for unit_id in sample["selected_unit_ids"] if unit_id in allowed]


def response_template(frame, sample, person_id=None):
    validate_sample(frame, sample)
    selected = _person_selection(frame, sample, person_id)
    return {
        "schema": "observatory.effort-responses/1",
        "frame_sha256": canonical_hash(frame),
        "sample_sha256": canonical_hash(sample),
        "responses": [
            {"unit_id": unit_id, "status": "nonresponse", "allocations": [], "non_work_minutes": None, "reason": "not_answered"}
            for unit_id in selected
        ],
    }


def validate_responses(frame, sample, responses):
    """Validate labels while retaining explicit and absent nonresponse.

    Returns observed allocations expressed in integer microseconds. Missing
    selected units remain absent; only export_estimator requires every response.
    """
    validate_sample(frame, sample)
    _keys(responses, {"schema", "frame_sha256", "sample_sha256", "responses"}, "responses")
    if responses["schema"] != "observatory.effort-responses/1":
        raise ValueError("unsupported response schema")
    if responses["frame_sha256"] != canonical_hash(frame) or responses["sample_sha256"] != canonical_hash(sample):
        raise ValueError("responses do not bind to this frame and sample")
    records = responses["responses"]
    if not isinstance(records, list) or len(records) > sample["design"]["sample_size"]:
        raise ValueError("responses must be a bounded list of sampled units")
    units = {unit["unit_id"]: unit for unit in frame["units"]}
    selected = set(sample["selected_unit_ids"])
    tasks = {link["task_id"] for link in frame["links"]} | {"unattributed"}
    result = {}
    for record in records:
        _keys(record, {"unit_id", "status", "allocations", "non_work_minutes", "reason"}, "response")
        unit_id = record["unit_id"]
        if not isinstance(unit_id, str) or unit_id not in selected or unit_id in result:
            raise ValueError("response has a duplicate or unselected unit_id")
        if not isinstance(record["allocations"], list) or len(record["allocations"]) > len(tasks) * len(ROLES):
            raise ValueError("allocations must be a bounded list")
        if record["status"] == "nonresponse":
            if record["allocations"] or record["non_work_minutes"] is not None:
                raise ValueError("nonresponse cannot contain observed allocation or non-work time")
            if not isinstance(record["reason"], str) or not re.fullmatch(r"[a-z0-9_-]{1,64}", record["reason"]):
                raise ValueError("nonresponse requires a short machine-readable reason")
            result[unit_id] = None
            continue
        if record["status"] != "observed" or record["reason"] is not None:
            raise ValueError("observed responses require status observed and a null reason")
        allocations = []
        categories = set()
        total = _minutes_micros(record["non_work_minutes"], "non_work_minutes")
        for allocation in record["allocations"]:
            _keys(allocation, {"task_id", "role", "minutes"}, "allocation")
            if not isinstance(allocation["task_id"], str) or allocation["task_id"] not in tasks:
                raise ValueError("allocation task is not in the frozen issue map")
            if allocation["role"] not in ROLES:
                raise ValueError("unknown allocation role")
            category = (allocation["task_id"], allocation["role"])
            if category in categories:
                raise ValueError("duplicate task/role allocation within a person interval")
            categories.add(category)
            micros = _minutes_micros(allocation["minutes"], "allocation minutes")
            total += micros
            allocations.append((allocation["task_id"], allocation["role"], micros))
        if total != _duration_micros(units[unit_id]):
            raise ValueError("allocations plus non-work must equal the whole sampled interval (microsecond resolution)")
        result[unit_id] = allocations
    return result


def merge_responses(frame, sample, response_documents):
    """Merge independently completed person forms without resolving conflicts silently."""
    validate_sample(frame, sample)
    if not isinstance(response_documents, list) or not response_documents:
        raise ValueError("supply one or more response documents")
    merged = {}
    for document in response_documents:
        validate_responses(frame, sample, document)
        for response in document["responses"]:
            unit_id = response["unit_id"]
            if unit_id in merged and canonical_hash(merged[unit_id]) != canonical_hash(response):
                raise ValueError("conflicting duplicate response for selected interval")
            merged[unit_id] = response
    result = response_template(frame, sample)
    result["responses"] = [merged[unit_id] for unit_id in sample["selected_unit_ids"] if unit_id in merged]
    return result


def export_estimator(frame, sample, responses, target="total"):
    observations = validate_responses(frame, sample, responses)
    if not frame["frame_complete"]:
        raise ValueError("complete target roster and person windows must be declared before total estimation")
    tasks = {link["task_id"] for link in frame["links"]}
    if target not in tasks | {"total", "unattributed"}:
        raise ValueError("unknown estimation target")
    missing = [unit_id for unit_id in sample["selected_unit_ids"] if observations.get(unit_id) is None]
    if missing:
        raise ValueError(f"sample nonresponse prevents complete estimation ({len(missing)} selected intervals)")
    units = []
    for unit in frame["units"]:
        if target == "total":
            prediction = unit["predicted_person_hours"]
        elif target == "unattributed":
            prediction = unit["unattributed_predicted_person_hours"]
        else:
            prediction = unit["task_predictions"].get(target, 0.0)
        observed = observations.get(unit["unit_id"])
        outcome = {"status": "unselected"}
        if observed is not None:
            micros = sum(value for task, _role, value in observed if target == "total" or task == target)
            outcome = {"status": "observed", "person_hours": _hours(micros)}
        units.append({"id": unit["unit_id"], "predicted_person_hours": prediction,
                      "inclusion_probability": sample["inclusion_probability"], "observation": outcome})
    provenance = {
        "frame_sha256": canonical_hash(frame), "sample_sha256": canonical_hash(sample),
        "responses_sha256": canonical_hash(responses), "model": frame["model"], "target": target,
        "synthetic": sample["synthetic"], "observation_resolution": "nearest_microsecond",
        "float_representation": "observed hours rounded down only if binary representation exceeds exact supplied duration",
    }
    return {
        "schema": "observatory.labour-estimate-input/2",
        "estimation": {
            "schema": "observatory.labour-estimate-input/1", "frame_id": frame["frame_id"] + ":" + target,
            "frame_complete": True,
            "predictions": {"frozen_before_sample": True, "provenance": json.dumps(provenance, sort_keys=True, separators=(",", ":"))},
            "design": {"kind": "simple_random_without_replacement", "sample_size": sample["design"]["sample_size"]},
            "units": units,
        },
        "frames": frame["frames"],
        "geometry": [{"unit_id": unit["unit_id"], "person_id": unit["person_id"], "start": unit["start"], "end": unit["end"]} for unit in frame["units"]],
    }


def render_review(frame, sample, person_id=None):
    """Return an offline HTML response form without prediction anchoring."""
    validate_sample(frame, sample)
    selected = set(_person_selection(frame, sample, person_id))
    payload = {
        "template": response_template(frame, sample, person_id),
        "person_id": person_id,
        "frame_id": frame["frame_id"], "protocol_id": frame["protocol_id"],
        "synthetic": sample["synthetic"], "attended_waiting_counts": frame["attended_waiting_counts"],
        "units": [{"unit_id": unit["unit_id"], "person_id": unit["person_id"], "start": unit["start"], "end": unit["end"],
                   "minutes": _duration_micros(unit) / MICROS_PER_MINUTE}
                  for unit in frame["units"] if unit["unit_id"] in selected],
        "tasks": [{"id": link["task_id"], "label": f'{link["repository"]}#{link["issue_number"]} — {link["task_id"]}'} for link in frame["links"]]
                 + [{"id": "unattributed", "label": "Engineering work — task attribution unresolved"}],
        "roles": list(ROLES), "reasons": list(REASONS),
    }
    encoded = json.dumps(payload, ensure_ascii=False, separators=(",", ":"))
    for character, replacement in (("&", "\\u0026"), ("<", "\\u003c"), (">", "\\u003e"), ("\u2028", "\\u2028"), ("\u2029", "\\u2029")):
        encoded = encoded.replace(character, replacement)
    return _HTML.replace("__PAYLOAD__", encoded)


_HTML = r'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'">
<title>Observatory sampled effort review</title>
<style>
:root { color-scheme:light; font-family:system-ui,sans-serif; color:#183244; background:#f4f7f6; }
* { box-sizing:border-box; }
body { max-width:1050px; margin:auto; padding:28px 18px 70px; line-height:1.5; overflow-wrap:anywhere; }
h1 { margin-bottom:8px; } h2 { font-size:1.2rem; margin:0 0 8px; }
.notice { padding:15px; background:#e1efeb; border-left:4px solid #007e70; }
.card { background:white; border:1px solid #cbd9d6; border-radius:8px; padding:20px; margin:20px 0; }
fieldset { border:0; margin:12px 0; padding:0; min-width:0; } fieldset:disabled { opacity:.55; }
label { display:inline-flex; flex-direction:column; gap:4px; margin:8px 14px 8px 0; min-width:0; max-width:100%; }
input,select,button { font:inherit; padding:7px; max-width:100%; }
select { min-width:0; width:100%; }
input[type=number] { width:105px; } button { border:1px solid #8ca6a0; border-radius:4px; cursor:pointer; background:white; }
.row { display:flex; gap:10px; flex-wrap:wrap; align-items:end; border-bottom:1px solid #e0e8e5; padding:4px 0; min-width:0; }
.row label { margin-right:0; flex:1 1 130px; } .row .task-field { flex-basis:100%; }
.row button { margin-bottom:8px; }
.add-activity { display:block; margin:14px 0 4px; }
.nonwork-field { display:flex; width:fit-content; margin:14px 0; }
details { margin:8px 0 16px; } details p { margin:4px 0; } summary { cursor:pointer; }
.interval-times { margin:6px 0; }
.primary { color:white; background:#006b60; font-weight:650; padding:12px 20px; }
.small { color:#536773; font-size:.92rem; } .error { color:#a22622; font-weight:600; }
#result { white-space:pre-wrap; } a { color:#006b60; }
</style></head><body>
<h1>Review a few sampled intervals</h1>
<p id="identity" class="small"></p>
<div class="notice"><strong id="study-kind"></strong>
<p>Record what you remember doing within each complete interval. Include specification, implementation, review, testing, coordination and repair. Use “task attribution unresolved” when you cannot reliably assign engineering work to an issue; do not guess an issue.</p>
<p id="waiting"></p>
<p>Reading, thinking and meetings may count even without keyboard activity. Non-work includes breaks and time outside this study's engineering-work scope. If you cannot recall the interval reliably, keep it as nonresponse. Predictions are deliberately hidden.</p>
</div>
<p class="small">This file works offline. It sends nothing to a server. Download saves a response JSON file for the collector. Your entries are not saved automatically; keep the downloaded file. Self-reports remain observations to validate, not certified facts.</p>
<div id="units"></div>
<button id="download" class="primary" type="button">Download responses</button>
<p id="result" role="status" aria-live="polite"></p>
<script id="payload" type="application/json">__PAYLOAD__</script>
<script>
"use strict";
const data=JSON.parse(document.getElementById("payload").textContent);
const states=[];
const node=(tag,text)=>{const e=document.createElement(tag); if(text!==undefined)e.textContent=text; return e;};
const option=(value,label)=>{const e=node("option",label);e.value=value;return e;};
const labelled=(text,control)=>{const e=node("label",text);e.append(control);return e;};
function numeric(){const e=node("input");e.type="number";e.min="0";e.step="any";e.inputMode="decimal";return e;}
const localTime=new Intl.DateTimeFormat(undefined,{year:"numeric",month:"short",day:"numeric",weekday:"short",hour:"2-digit",minute:"2-digit",second:"2-digit",timeZoneName:"short"});
const displayZone=Intl.DateTimeFormat().resolvedOptions().timeZone;
document.getElementById("identity").textContent="Protocol: "+data.protocol_id+" · Frame: "+data.frame_id+(data.person_id?" · Person: "+data.person_id:"");
document.getElementById("study-kind").textContent=data.synthetic?"SYNTHETIC REPLAY — not an empirical sample":(data.person_id?"Sampled validation — complete only your own observed intervals":"Observer view — complete only intervals supported by your observation method");
document.getElementById("waiting").textContent=data.attended_waiting_counts?"Frozen convention: attended waiting counts as engineering work. Unattended tool execution does not count as human work.":"Frozen convention: waiting for a tool without other included work is excluded from engineering effort. Unattended tool execution does not count as human work.";
for(const unit of data.units){
  const card=node("section");card.className="card";
  card.append(node("h2",unit.person_id+" · "+(unit.minutes>=.001?unit.minutes.toFixed(3).replace(/\.?0+$/,""):unit.minutes.toPrecision(3))+" minutes"));
  const zone=node("p","Times shown in "+displayZone+" (this browser's time zone)");zone.className="small";card.append(zone);
  const start=node("p","Start: "+localTime.format(new Date(unit.start)));start.className="interval-times";
  const end=node("p","End: "+localTime.format(new Date(unit.end)));end.className="interval-times";card.append(start,end);
  const exact=node("details");exact.className="small";
  exact.append(node("summary","Exact UTC timestamps"),node("p","Start: "+unit.start),node("p","End: "+unit.end));card.append(exact);
  const status=node("select");status.append(option("nonresponse","Cannot supply a reliable observation"),option("observed","I can account for this interval"));
  card.append(labelled("Observation status",status));
  const reason=node("select");for(const r of data.reasons)reason.append(option(r,r.replaceAll("_"," ")));
  const reasonLabel=labelled("Missing-response reason",reason);card.append(reasonLabel);
  const fields=node("fieldset");fields.disabled=true;
  const rows=node("div");fields.append(rows);
  const add=node("button","Add task / activity");add.type="button";add.className="add-activity";fields.append(add);
  const nonwork=numeric();const nonworkLabel=labelled("Non-work minutes",nonwork);nonworkLabel.className="nonwork-field";fields.append(nonworkLabel);
  fields.append(node("p","Work allocations + non-work must account for all "+unit.minutes+" minutes. An observed zero requires no work allocations and the full interval as non-work."));
  card.append(fields);document.getElementById("units").append(card);
  const state={unit,status,reason,nonwork,allocations:[]};states.push(state);
  function addRow(){
    const row=node("div");row.className="row";
    const task=node("select");for(const t of data.tasks)task.append(option(t.id,t.label));
    const role=node("select");for(const r of data.roles)role.append(option(r,r));
    const minutes=numeric();const remove=node("button","Remove");remove.type="button";
    const taskLabel=labelled("Task",task);taskLabel.className="task-field";
    row.append(taskLabel,labelled("Activity",role),labelled("Minutes",minutes),remove);
    const allocation={task,role,minutes};state.allocations.push(allocation);
    remove.onclick=()=>{state.allocations=state.allocations.filter(x=>x!==allocation);row.remove();};rows.append(row);
  }
  add.onclick=addRow;
  status.onchange=()=>{fields.disabled=status.value!=="observed";reason.disabled=status.value==="observed";};
}
function readNumber(input,label){if(input.value.trim()==="")throw new Error(label+" is required");const n=Number(input.value);if(!Number.isFinite(n)||n<0)throw new Error(label+" must be nonnegative");return n;}
document.getElementById("download").onclick=()=>{
  const output=JSON.parse(JSON.stringify(data.template));const result=document.getElementById("result");result.className="";
  try{
    output.responses=states.map(s=>{
      if(s.status.value!=="observed")return {unit_id:s.unit.unit_id,status:"nonresponse",allocations:[],non_work_minutes:null,reason:s.reason.value};
      const categories=new Set();
      const allocations=s.allocations.map(a=>{const key=JSON.stringify([a.task.value,a.role.value]);if(categories.has(key))throw new Error("Duplicate task/activity in "+s.unit.unit_id);categories.add(key);return {task_id:a.task.value,role:a.role.value,minutes:readNumber(a.minutes,"Work minutes")};});
      const nonwork=readNumber(s.nonwork,"Non-work minutes");
      const sum=allocations.reduce((v,a)=>v+a.minutes,nonwork);
      if(Math.abs(sum-s.unit.minutes)>1/60000000)throw new Error("Account for exactly "+s.unit.minutes+" minutes in "+s.unit.unit_id+"; entered "+sum);
      return {unit_id:s.unit.unit_id,status:"observed",allocations,non_work_minutes:nonwork,reason:null};
    });
    const blob=new Blob([JSON.stringify(output,null,2)+"\n"],{type:"application/json"});
    const url=URL.createObjectURL(blob);const a=node("a");a.href=url;a.download="effort-responses.json";document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);
    const missing=output.responses.filter(r=>r.status==="nonresponse").length;
    result.textContent="Response file downloaded. "+missing+" interval(s) remain nonresponse.";
  }catch(error){result.className="error";result.textContent=error.message;}
};
</script></body></html>
'''
