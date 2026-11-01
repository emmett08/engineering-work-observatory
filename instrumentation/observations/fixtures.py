"""Deterministic fabricated task totals for software demonstration, never evidence.

The two repository populations deliberately differ. A repository-based predictor
should beat a pooled median here; that demonstrates wiring, not real accuracy.
"""
from effort.common import canonical_hash

PROTOCOL = "synthetic-task-total-demo-v1"


def synthetic_revisions():
    records = []
    for repository, offset in (("synthetic/short", 5), ("synthetic/long", 45)):
        for number in range(1, 65):
            observation = {
                "schema": "observatory.effort-observation/1", "protocol_id": PROTOCOL,
                "task": {"repository": repository, "issue_number": number,
                         "task_id": f"fabricated-{repository.split('/')[1]}-{number}", "pull_requests": []},
                "declared_minutes": offset + number % 5,
                "method": "timer", "coverage": "complete", "scope": "task_total",
                "completed_at": "2026-01-01T00:00:00Z", "synthetic": True,
            }
            source = {"repository": repository, "issue_number": number,
                      "updated_at": "2026-01-01T00:00:00Z", "state": "closed"}
            body = {"schema": "observatory.observation-revision/1",
                    "observation_id": canonical_hash(["fabricated", repository, number]),
                    "source": source, "status": "accepted", "observation": observation,
                    "errors": [], "fixture": "fabricated-repository-populations/1"}
            records.append({**body, "revision_id": canonical_hash(body),
                            "observed_at": "2026-01-01T00:00:00Z"})
    return records
