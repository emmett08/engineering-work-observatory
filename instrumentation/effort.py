#!/usr/bin/env python3
"""Collect local activity, freeze predictions, sample observations and report effort."""
import argparse
from pathlib import Path
import subprocess
import sys

from effort.activity import collect_export, collect_live
from effort.common import canonical_hash, read_json, require, validate_config, validate_frame, write_json, write_text
from effort.frame import build_frame
from effort.report import build_report, markdown
from effort.sampling import draw_sample, render_review, response_template, validate_sample


def parser():
    cli = argparse.ArgumentParser(description=__doc__)
    sub = cli.add_subparsers(dest="command", required=True)
    prepare = sub.add_parser("prepare", help="Prepare a complete daily local batch and participant review forms")
    prepare.add_argument("--config", required=True, type=Path, help="Template with explicit UTC person windows")
    prepare.add_argument("--day", required=True, help="YYYY-MM-DD; shifts every template window by the same UTC date offset")
    prepare.add_argument("--size", required=True, type=int)
    prepare.add_argument("--directory", required=True, type=Path)
    prepare_source = prepare.add_mutually_exclusive_group()
    prepare_source.add_argument("--export", type=Path)
    prepare_source.add_argument("--server", default="http://127.0.0.1:5600")
    collect = sub.add_parser("collect", help="Sanitise explicitly selected ActivityWatch buckets")
    collect.add_argument("--config", required=True, type=Path)
    source = collect.add_mutually_exclusive_group(required=True)
    source.add_argument("--export", type=Path)
    source.add_argument("--server", help="Loopback ActivityWatch server, e.g. http://127.0.0.1:5600")
    collect.add_argument("--output", required=True, type=Path)
    merge = sub.add_parser("merge-activity", help="Combine devices collected separately under the same config")
    merge.add_argument("--config", required=True, type=Path)
    merge.add_argument("--activity", required=True, action="append", type=Path)
    merge.add_argument("--output", required=True, type=Path)
    freeze = sub.add_parser("freeze", help="Freeze a complete partition and activity proxy predictions")
    freeze.add_argument("--config", required=True, type=Path)
    freeze.add_argument("--activity", required=True, type=Path)
    freeze.add_argument("--output", required=True, type=Path)
    sample = sub.add_parser("sample", help="Retain one probability draw beside the frozen frame")
    sample.add_argument("--frame", required=True, type=Path)
    sample.add_argument("--size", required=True, type=int)
    sample.add_argument("--synthetic-seed", help="64 hex characters; labels the sample and report synthetic")
    sample.add_argument("--output", required=True, type=Path)
    for name in ("review", "template"):
        review = sub.add_parser(name, help="Create an offline review form" if name == "review" else "Create unanswered response JSON")
        review.add_argument("--frame", required=True, type=Path)
        review.add_argument("--sample", required=True, type=Path)
        person = review.add_mutually_exclusive_group()
        person.add_argument("--person", help="Respondent's pseudonymous person ID")
        person.add_argument("--all-people", action="store_true", help="Explicitly prepare an observer form for everyone")
        review.add_argument("--output", required=True, type=Path)
    for name in ("estimate", "proxy"):
        report = sub.add_parser(name, help="Produce an issue-linked aggregate report")
        report.add_argument("--frame", required=True, type=Path)
        report.add_argument("--target", default="total", help="total, unattributed, or a configured task ID")
        report.add_argument("--output", required=True, type=Path)
        report.add_argument("--markdown", type=Path)
        if name == "estimate":
            report.add_argument("--sample", required=True, type=Path)
            report.add_argument("--responses", required=True, action="append", type=Path, help="Repeat for each person's response file")
            report.add_argument("--observatory", default="target/release/observatory", type=Path)
    links = sub.add_parser("sync-links", help="Read actual GitHub issue/PR relationships into a new config")
    links.add_argument("--config", required=True, type=Path)
    links.add_argument("--repository", required=True)
    links.add_argument("--pr", required=True, action="append", type=int)
    links.add_argument("--output", required=True, type=Path)
    links.add_argument("--evidence", required=True, type=Path)
    return cli


def main(argv=None):
    args = parser().parse_args(argv)
    if args.command == "prepare":
        from effort.batch import prepare_batch
        directory = prepare_batch(read_json(args.config), args.directory, args.day, args.size,
                                  args.server, read_json(args.export) if args.export else None)
        print(str(directory))
        return 0
    elif args.command == "collect":
        config = validate_config(read_json(args.config))
        activity = collect_export(config, read_json(args.export)) if args.export else collect_live(config, args.server)
        write_json(args.output, activity)
    elif args.command == "merge-activity":
        from effort.activity import merge_activity
        write_json(args.output, merge_activity(read_json(args.config), [read_json(p) for p in args.activity]))
    elif args.command == "freeze":
        write_json(args.output, build_frame(read_json(args.config), read_json(args.activity)))
    elif args.command == "sample":
        frame = validate_frame(read_json(args.frame))
        registry = args.frame.resolve().parent / "samples" / (canonical_hash(frame) + ".json")
        if registry.exists():
            sample = read_json(registry)
            validate_sample(frame, sample)
            require(sample["design"]["sample_size"] == args.size, "frame already has a draw with another sample size")
            if args.synthetic_seed is not None:
                require(sample["synthetic"] and sample["seed"] == args.synthetic_seed, "frame already has another draw")
        else:
            sample = draw_sample(frame, args.size, args.synthetic_seed)
            write_json(registry, sample)
        write_json(args.output, sample)
    elif args.command in {"review", "template"}:
        frame, sample = read_json(args.frame), read_json(args.sample)
        validate_sample(frame, sample)
        require(args.person is not None or args.all_people or len(frame["frames"]) == 1,
                "choose --person for a participant, or --all-people for an explicit observer form")
        if args.command == "review":
            write_text(args.output, render_review(frame, sample, args.person))
        else:
            write_json(args.output, response_template(frame, sample, args.person))
    elif args.command in {"estimate", "proxy"}:
        frame = read_json(args.frame)
        extra = {}
        if args.command == "estimate":
            extra = {"sample": read_json(args.sample), "response_documents": [read_json(p) for p in args.responses],
                     "observatory": args.observatory, "input_path": args.output.with_suffix(".estimator-input.json")}
        report = build_report(frame, args.target, **extra)
        write_json(args.output, report)
        if args.markdown:
            write_text(args.markdown, markdown(report))
    elif args.command == "sync-links":
        from effort.linking import sync_links
        config, evidence = sync_links(read_json(args.config), args.repository, args.pr)
        write_json(args.evidence, evidence)
        write_json(args.output, config)
    print(str(args.output))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError, TypeError, OverflowError, subprocess.SubprocessError) as error:
        print("effort: " + str(error), file=sys.stderr)
        sys.exit(2)
