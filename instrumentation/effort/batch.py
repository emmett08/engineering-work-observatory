"""Repeatable daily preparation, suitable for a local OS scheduler."""
from copy import deepcopy
import datetime as dt
import hashlib
import html
from pathlib import Path

from .activity import collect_export, collect_live
from .common import canonical_hash, format_time, parse_time, read_json, require, validate_config, validate_frame, write_json, write_text
from .frame import build_frame, _validate_activity
from .report import build_report, markdown
from .sampling import draw_sample, render_review, response_template, validate_sample


def prepare_batch(template, directory, day, sample_size, server=None, export_data=None):
    """Shift the declared UTC windows, never infer work windows from activity."""
    validate_config(template)
    require(type(sample_size) is int and sample_size > 0, "sample size must be positive")
    try:
        date = dt.date.fromisoformat(day)
    except (TypeError, ValueError) as error:
        raise ValueError("--day must be a YYYY-MM-DD UTC date") from error
    require(date.isoformat() == day, "--day must be a YYYY-MM-DD UTC date")
    config = deepcopy(template)
    original_date = min(parse_time(f["start"]).date() for f in config["frames"])
    shift = date-original_date
    for frame in config["frames"]:
        frame["start"] = format_time(parse_time(frame["start"]) + shift)
        frame["end"] = format_time(parse_time(frame["end"]) + shift)
    config["frame_id"] += "@" + day
    validate_config(config)
    require(all(parse_time(f["end"]) <= dt.datetime.now(dt.timezone.utc) for f in config["frames"]), "daily batch windows have not ended")
    directory = Path(directory)
    write_json(directory/"config.json", config)
    activity_path = directory/"activity.json"
    if activity_path.exists():
        activity = read_json(activity_path)
        _validate_activity(config, activity)
    else:
        activity = collect_export(config, export_data) if export_data is not None else collect_live(config, server or "http://127.0.0.1:5600")
        write_json(activity_path, activity)
    frame_path = directory/"frame.json"
    if frame_path.exists():
        frame = validate_frame(read_json(frame_path))
        require(frame["config_sha256"] == canonical_hash(config) and frame["activity_sha256"] == canonical_hash(activity), "batch frame source mismatch")
    else:
        frame = build_frame(config, activity)
        write_json(frame_path, frame)
    registry = directory/"samples"/(canonical_hash(frame)+".json")
    if registry.exists():
        sample = read_json(registry)
        validate_sample(frame, sample)
        require(sample["design"]["sample_size"] == sample_size, "batch already has a different sample size")
    else:
        sample = draw_sample(frame, sample_size)
        write_json(registry, sample)
    write_json(directory/"sample.json", sample)
    reviews = []
    for person in frame["frames"]:
        person_id = person["person_id"]
        name = "review-" + hashlib.sha256(person_id.encode()).hexdigest()[:16]
        write_text(directory/(name+".html"), render_review(frame, sample, person_id))
        write_json(directory/(name+"-template.json"), response_template(frame, sample, person_id))
        reviews.append({"person_id": person_id, "form": name+".html", "response_template": name+"-template.json"})
    write_json(directory/"review-index.json", {"schema": "observatory.effort-review-index/1", "frame_sha256": canonical_hash(frame), "reviews": reviews})
    entries = "".join(f'<li><a href="{item["form"]}">{html.escape(item["person_id"])}</a></li>' for item in reviews)
    write_text(directory/"review-index.html", '<!doctype html><html lang="en"><meta charset="utf-8"><title>Sampled interval checks</title><body><h1>Sampled interval checks</h1><p>Distribute only each participant’s own form. These forms send nothing automatically.</p><ul>'+entries+'</ul></body></html>')
    proxy = build_report(frame)
    write_json(directory/"proxy.json", proxy)
    write_text(directory/"proxy.md", markdown(proxy))
    return directory
