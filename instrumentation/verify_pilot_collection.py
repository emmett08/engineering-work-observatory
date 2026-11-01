#!/usr/bin/env python3
"""Check the real GitHub collector against the two completed pilot jobs.

The current reconciliation job is still running. This check covers the completed
matrix jobs in this run attempt, not completion of the workflow or total tool use.
The Observatory CLI verifies each store before exporting these snapshots.
"""
import argparse
import json
from pathlib import Path

EXPECTED_JOBS = {"Execution pilot (engarg)", "Execution pilot (eal-mcp)"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def evidence(snapshot):
    require(snapshot.get("schema") == "observatory.snapshot/1", "unexpected snapshot schema")
    records = [item["record"] for item in snapshot["records"]]
    require(len({item["id"] for item in records}) == len(records), "duplicate record IDs")
    return records


def current_receipts(records, run_id, run_attempt):
    selected = {}
    for record in records:
        if record["data"].get("source") != "observatory.tool_receipt":
            continue
        receipt = record["data"]["payload"]
        platform = receipt["platform"]
        if platform["run_id"] == run_id and platform["run_attempt"] == run_attempt:
            selected[record["id"]] = record
    return selected


def verify(first, second, run_id, run_attempt):
    first_records, second_records = evidence(first), evidence(second)
    first_receipts = current_receipts(first_records, run_id, run_attempt)
    second_receipts = current_receipts(second_records, run_id, run_attempt)
    require(len(first_receipts) == 4, "first collection must retain four current-attempt receipts")
    require(first_receipts == second_receipts, "repeat collection changed or duplicated receipts")

    invocations = {}
    for record_id, record in second_receipts.items():
        receipt = record["data"]["payload"]
        scope = record_id.rsplit("/", 1)[0]
        invocations.setdefault(scope, {})[receipt["phase"]] = record
    require(len(invocations) == 2, "expected two distinct tool invocations")
    job_names = set()
    job_ids = set()
    for phases in invocations.values():
        require(set(phases) == {"start", "completion"}, "invocation is missing a lifecycle phase")
        start = phases["start"]["data"]["payload"]
        completion = phases["completion"]["data"]["payload"]
        require(start["platform"] == completion["platform"], "phase identities differ")
        require(completion["result"]["status"] == "success", "pilot tool did not succeed")
        job_names.add(start["platform"]["job_name"])
        expected_ids = {record["id"] for record in phases.values()}
        matches = [
            record["data"]["payload"] for record in second_records
            if record["data"].get("source") == "github.execution_reconciliation"
            and set(record["data"]["payload"]["receipt_ids"]) == expected_ids
        ]
        require(len(matches) == 1, "expected one independent invocation reconciliation")
        matched = matches[0]
        require(matched["run_id"] == int(run_id) and matched["run_attempt"] == run_attempt,
                "reconciliation points at another run attempt")
        require(matched["lifecycle"] == "start_and_completion_observed",
                "collector did not observe both lifecycle phases")
        require(matched["job_link"] == "unambiguous" and matched["job_id"] is not None,
                "collector could not link the invocation to one GitHub job")
        require(matched["job_status"] == "completed" and matched["job_conclusion"] == "success",
                "linked GitHub job did not complete successfully")
        job_ids.add(matched["job_id"])
        jobs = [
            record["data"]["payload"] for record in second_records
            if record["data"].get("source") == "github.workflow_job"
            and record["data"]["payload"].get("id") == matched["job_id"]
            and record["data"]["payload"].get("run_id") == int(run_id)
            and record["data"]["payload"].get("run_attempt") == run_attempt
        ]
        require(jobs and all(job["name"] == start["platform"]["job_name"] for job in jobs),
                "linked job metadata does not identify the expected matrix job")
    require(job_names == EXPECTED_JOBS and len(job_ids) == 2,
            "receipts must belong to both pilot matrix jobs, not the collector job")
    return {
        "schema": "observatory.execution-pilot-verification/1",
        "run_id": run_id,
        "run_attempt": run_attempt,
        "receipt_records": len(second_receipts),
        "completed_invocations": len(invocations),
        "linked_job_ids": sorted(job_ids),
        "job_names": sorted(job_names),
        "repeated_delivery_receipts_unchanged": True,
        "scope": "Completed pilot matrix jobs; reconciliation job remains active. No claim of complete tool coverage or workflow completion.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("first", type=Path)
    parser.add_argument("second", type=Path)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--run-attempt", required=True, type=int)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = verify(json.loads(args.first.read_text()), json.loads(args.second.read_text()),
                    args.run_id, args.run_attempt)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
