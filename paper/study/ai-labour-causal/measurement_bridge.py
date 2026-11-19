"""Carry the same constructed task histories through cohort accounting and the trial rule."""

import argparse
import json
import subprocess
from datetime import datetime, timedelta, timezone
from pathlib import Path
from tempfile import TemporaryDirectory

import numpy as np

from planning import analyse


REPO = Path(__file__).resolve().parents[3]
INTAKE = datetime(2000, 1, 1, tzinfo=timezone.utc)
PAIR_OFFSETS = (-2, -1, 0, 0, 1, 2)
REGIMES = ("complete", "favourable-errors", "missing-audit")


def stamp(day, minutes=0):
    return (INTAKE + timedelta(days=day, minutes=minutes)).isoformat()


def fixed_teams():
    """Two requirements of one and two initial units for each of twelve teams."""
    teams = {}
    allocation = np.random.default_rng(19).integers(2, size=len(PAIR_OFFSETS))
    for pair, offset in enumerate(PAIR_OFFSETS):
        for side in (0, 1):
            ai = side == allocation[pair]
            team = f"pair-{pair}-team-{side}"
            tasks = []
            for index, units in enumerate((1, 2)):
                build = 180 + 10 * (pair % 3) + 15 * index
                passed = 94 - 2 * index - pair % 3
                if ai:
                    build += -60 + 6 * offset
                    passed += -5 + offset
                tasks.append((units, units * build, units * 60, passed))
            teams[team] = (pair, ai, tasks)
    return teams


def evidence(teams, regime):
    rows = []

    def add(identifier, kind, **data):
        rows.append(dict(id=identifier, kind=kind, data=data))

    for team, (pair, ai, tasks) in teams.items():
        # Rust groups by protocol_id. All team identifiers use identical rules.
        add(team, "protocol", name="Constructed measurement comparison", frozen_at=stamp(-1),
            acceptance_days=14, follow_up_days=14, rubric_version="100-checks")
        for index, (units, build, review, passed) in enumerate(tasks):
            task = f"{team}/task-{index}"
            add(task, "task", protocol_id=team, requirement=f"Requirement {index}",
                task_class="maintenance", scope_version="1", enrolled_at=stamp(0),
                score_frozen_at=stamp(0), frozen_score=units, assignment_unit=team,
                assigned_policy="AI access" if ai else "comparator")
            for domain in ("effort", "lifecycle"):
                # The erroneous regimes preserve the false claim of full effort coverage.
                add(f"{task}/{domain}", "coverage", task_id=task, domain=domain,
                    start=stamp(0), end=stamp(28), status="complete", method="direct")
            work_day = 1 + 2 * index
            biased = ai and regime != "complete"
            for role, minutes, day in (("implementation", build, work_day),
                                       ("review", review, work_day + 1)):
                if biased and role == "review":
                    continue
                add(f"{task}/{role}", "effort", task_id=task, person_id=f"{team}/{role}",
                    role=role, start=stamp(day, 540), end=stamp(day, 540 + minutes),
                    allocation_fraction=1, method="direct")
            add(f"{task}/accepted", "lifecycle", task_id=task, event="accepted",
                occurred_at=stamp(5), scope_version="1")
            if regime == "missing-audit" and ai and pair == 0 and index == 1:
                continue
            # Each requirement has 100 equally weighted checks; six failed checks
            # are overlooked in the favourable reporting regime.
            observed_passes = passed + (6 if biased else 0)
            add(f"{task}/follow-up", "assessment", task_id=task, phase="follow_up",
                assessed_at=stamp(19), status="observed", retained_fraction=observed_passes / 100,
                rubric_version="100-checks", scope_version="1")
    return rows


def cohort_endpoints(command, rows):
    with TemporaryDirectory() as tmp:
        directory = Path(tmp)
        source = directory / "study.jsonl"
        source.write_text("".join(json.dumps(row) + "\n" for row in rows))
        snapshot = directory / "snapshot.json"
        report = directory / "report"
        for arguments in (["import", source], ["snapshot", "--output", snapshot],
                          ["analyse", snapshot, "--output", report]):
            subprocess.run([*command, "--state", directory / "store", *arguments],
                           cwd=REPO, check=True, stdout=subprocess.DEVNULL)
        analysis = json.loads((report / "report.json").read_text())["analysis"]
    initial = {}
    for task in analysis["tasks"]:
        team = task["protocol_id"]
        initial[team] = initial.get(team, 0) + task["frozen_score"]
    endpoints = {}
    for row in analysis["cohorts"]:
        team = row["protocol_id"]
        assert row["counted_output_tasks"] == 2 and initial[team] == 3
        score = row["total_retained_output"]
        endpoints[team] = (row["total_person_hours"] / initial[team],
                           None if score is None else score / initial[team])
    return endpoints


def paired_differences(teams, endpoints):
    """A missing score passes through as NaN; no team or pair is dropped."""
    differences = np.empty((len(PAIR_OFFSETS), 2))
    for pair in range(len(PAIR_OFFSETS)):
        arms = {ai: endpoints[team] for team, (p, ai, _) in teams.items() if p == pair}
        differences[pair] = np.asarray(arms[True], dtype=float) - np.asarray(arms[False], dtype=float)
    return differences


def decision(differences):
    complete_pairs = np.isfinite(differences).all(axis=1)
    hours, quality = differences.T
    result = analyse(hours[None, :], quality[None, :], np.array([complete_pairs.all()]))
    return int(complete_pairs.sum()), bool(result[-1][0])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, help="use a prebuilt observatory from this revision")
    args = parser.parse_args()
    command = ([args.binary.resolve()] if args.binary
               else ["cargo", "run", "--locked", "--quiet", "--bin", "observatory", "--"])
    teams = fixed_teams()
    truth = {}
    for team, (_, _, tasks) in teams.items():
        initial = sum(s for s, _, _, _ in tasks)
        truth[team] = (sum(b + r for _, b, r, _ in tasks) / (60 * initial),
                       sum(s * passed for s, _, _, passed in tasks) / (100 * initial))
    latent = paired_differences(teams, truth)
    np.testing.assert_allclose(latent.mean(axis=0), [-1, -.05])
    assert np.all(latent.std(axis=0) > 0) and decision(latent) == (6, False)
    for regime in REGIMES:
        observed = cohort_endpoints(command, evidence(teams, regime))
        assert observed.keys() == truth.keys()
        for team, (pair, ai, _) in teams.items():
            hours, score = truth[team]
            biased = ai and regime != "complete"
            np.testing.assert_allclose(observed[team][0], hours - int(biased))
            if regime == "missing-audit" and ai and pair == 0:
                assert observed[team][1] is None
            else:
                np.testing.assert_allclose(observed[team][1], score + (.06 if biased else 0))
        differences = paired_differences(teams, observed)
        complete, benefit = decision(differences)
        assert (complete, benefit) == {
            "complete": (6, False), "favourable-errors": (6, True), "missing-audit": (5, False)
        }[regime]
        quality = f"{100 * differences[:, 1].mean():+.2f} pp" if complete == 6 else "unknown"
        print(f"{regime}: labour {differences[:, 0].mean():+.2f} h/unit; retained {quality}; "
              f"complete pairs {complete}/6; benefit {benefit}")


if __name__ == "__main__":
    main()
