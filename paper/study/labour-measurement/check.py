"""Exercise the labour CLIs on every timeline in a small, declared finite population."""

import argparse
import json
import subprocess
import tempfile
from fractions import Fraction
from functools import cache
from itertools import combinations, permutations
from pathlib import Path
from statistics import mean, pvariance

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[2]
HALF = Fraction(1, 2)
ESTIMATORS = ("ht", "difference")
SAMPLES = tuple(combinations(range(4), 2))
STREAMS = (
    ("build", "engineer", "feature", "implementation"),
    ("repair", "engineer", "defect", "repair"),
    ("review", "reviewer", "feature", "review"),
)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def work_hours(ticks):
    return sum(tick != "-" for tick in ticks) * HALF


def histories():
    # Each person has four half-hour slots. A dash denotes time outside the cohort.
    for pattern in ("AA--", "AAB-", "AABB"):
        for engineer in sorted(set(permutations(pattern))):
            for start in range(4):
                reviewer = tuple("R" if index == start else "-" for index in range(4))
                yield engineer, reviewer


def build_binary():
    build = subprocess.run(
        ["cargo", "build", "--locked", "--bin", "observatory", "--message-format=json-render-diagnostics"],
        cwd=REPOSITORY, check=True, stdout=subprocess.PIPE, text=True,
    )
    for line in build.stdout.splitlines():
        artifact = json.loads(line)
        if artifact.get("executable") and artifact["target"]["name"] == "observatory":
            return Path(artifact["executable"])
    raise RuntimeError("Cargo did not report the observatory executable.")


def cli(binary, working_file):
    # Several distinct timelines produce identical observations. Reuse their CLI result.
    @cache
    def run(command, document):
        working_file.write_text(document)
        result = subprocess.run(
            [binary, command, str(working_file)], capture_output=True, text=True,
        )
        if result.returncode:
            if command == "estimate-labour" and "has nonresponse;" in result.stderr:
                return None
            raise RuntimeError(result.stderr or f"{command} exited {result.returncode}")
        return json.loads(result.stdout)

    def execute(command, document):
        return run(command, json.dumps(document, sort_keys=True))

    return execute


def observation(name, streams, end, lower, upper):
    return {"id": name, "start": "0", "end": str(end), "stream_ids": list(streams),
            "lower_person_hours": str(lower), "upper_person_hours": str(upper)}


def bounds(execute, history, condition, shared):
    engineer, reviewer = history
    observed_streams = STREAMS[:2] if condition == "reviewer omitted" else STREAMS
    streams = []
    for identifier, person, task, role in observed_streams:
        # Giving each stream its own capacity removes only cross-stream competition.
        streams.append({"id": identifier, "person_id": person if shared else identifier,
                        "task_id": task, "role": role})
    observations = []
    for stream, letter in (("build", "A"), ("repair", "B")):
        amount = engineer.count(letter) * HALF
        observations.append(observation(stream, [stream], 2, amount, amount))
    if condition != "reviewer omitted":
        first_hour = work_hours(reviewer[:2])
        low, high = first_hour, first_hour
        if condition == "review counted as zero":
            low, high = 0, 0
        elif condition == "review given an error bound":
            low, high = 0, HALF
        observations.append(observation("shift review", ["review"], 2, HALF, HALF))
        observations.append(observation("first hour review", ["review"], 1, low, high))
    if condition == "grouped first-hour observation":
        total = work_hours(engineer[:2])
        observations.append(observation("engineer first hour", ["build", "repair"], 1, total, total))
    target = [{"stream_id": row["id"], "start": "0", "end": "1", "weight": "1"}
              for row in streams]
    result = execute("bound-labour", {"schema": "observatory.labour-bound-input/1",
                     "problem_id": "known-timelines", "domain": {"start": "0", "end": "2"},
                     "streams": streams, "observations": observations, "target": target})
    require(result["status"] == "feasible", "Unexpected inconsistent observations")
    return Fraction(result["minimum"]["value"]), Fraction(result["maximum"]["value"])


def bound_results(execute, timelines):
    results = {}
    for condition in ("complete observations", "grouped first-hour observation", "reviewer omitted",
                      "review counted as zero", "review given an error bound"):
        models = {}
        for shared in (False, True):
            widths, contains, correct, wrong = [], 0, 0, 0
            for history in timelines:
                truth = sum(work_hours(person[:2]) for person in history)
                low, high = bounds(execute, history, condition, shared)
                widths.append(high - low)
                contains += low <= truth <= high
                if condition in ("complete observations", "review given an error bound"):
                    require(low <= truth <= high, "Valid observations excluded known first-hour labour")
                elif condition == "grouped first-hour observation":
                    require(low == high == truth, "Measured first-hour totals should identify first-hour labour")
                # Can the data settle whether first-hour cohort effort exceeded one person-hour?
                if high <= 1 or low > 1:
                    if (low > 1) == (truth > 1):
                        correct += 1
                    else:
                        wrong += 1
            models["shared" if shared else "separate"] = {
                "contains_truth": contains, "mean_width_hours": str(mean(widths)),
                "correct_decisions": correct, "wrong_decisions": wrong,
                "unresolved_decisions": len(timelines) - correct - wrong,
            }
        results[condition] = models
    return results


def estimate(execute, measured, sample, condition):
    units, geometry, frames = [], [], []
    hours = [f"2000-01-01T{hour:02d}:00:00Z" for hour in range(3)]
    for person_index, (person, prediction) in enumerate((("engineer", .75), ("reviewer", .25))):
        frames.append({"person_id": person, "start": hours[0], "end": hours[2]})
        for hour in range(2):
            index = 2 * person_index + hour
            unit = f"{person}-{hour}"
            geometry.append({"unit_id": unit, "person_id": person,
                             "start": hours[hour], "end": hours[hour + 1]})
            response = {"status": "unselected"}
            if index in sample:
                value = measured[index]
                response = {"status": "observed", "person_hours": float(value)}
                if person == "reviewer" and value:
                    if condition == "review counted as zero":
                        response["person_hours"] = 0
                    elif condition == "review prompt unanswered":
                        response = {"status": "nonresponse", "reason": "busy reviewing"}
            units.append({"id": unit, "predicted_person_hours": prediction,
                          "inclusion_probability": .5, "observation": response})
    document = {"schema": "observatory.labour-estimate-input/2", "geometry": geometry,
                "frames": frames, "estimation": {"schema": "observatory.labour-estimate-input/1",
                "frame_id": "four-person-hours", "frame_complete": True,
                "design": {"kind": "simple_random_without_replacement", "sample_size": 2},
                "predictions": {"frozen_before_sample": True,
                                "provenance": "Fixed role-specific values for this synthetic design."},
                "units": units}}
    result = execute("estimate-labour", document)
    if result is None:
        return None
    stats = result["estimation"]
    return {method: Fraction(stats[f"{method}_total_person_hours"]) for method in ESTIMATORS}


def sampling_results(execute, timelines):
    results = {}
    for condition in ("complete observations", "review counted as zero", "review prompt unanswered"):
        errors = {method: [] for method in ESTIMATORS}
        variances = {method: [] for method in ESTIMATORS}
        unavailable = 0
        for history in timelines:
            units = tuple(work_hours(person[start:start + 2]) for person in history for start in (0, 2))
            truth = sum(units)
            draws = []
            for sample in SAMPLES:
                estimated = estimate(execute, units, sample, condition)
                missing = condition == "review prompt unanswered" and any(i >= 2 and units[i] for i in sample)
                require((estimated is None) == missing, "Unexpected estimator response to a selected interval")
                if estimated is not None:
                    draws.append(estimated)
            unavailable += len(SAMPLES) - len(draws)
            for method in ESTIMATORS:
                values = [draw[method] for draw in draws]
                error = mean(values) - truth
                if condition == "complete observations":
                    require(error == 0, f"{method} should recover the shift total across the six samples")
                elif condition == "review counted as zero":
                    require(error == -HALF, f"{method} should miss the reviewer's half hour")
                errors[method].append(error)
                variances[method].append(pvariance(values))
        results[condition] = {
            "unavailable_draws": unavailable,
            "available_draws": len(timelines) * len(SAMPLES) - unavailable,
            "mean_error_hours_among_available": {m: str(mean(errors[m])) for m in ESTIMATORS},
            "mean_within_history_variance_among_available": {m: str(mean(variances[m])) for m in ESTIMATORS},
        }
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="compare the results with the committed values")
    mode.add_argument("--write", action="store_true", help="replace results.json after an intentional design change")
    parser.add_argument("--binary", type=Path, help="use this prebuilt observatory executable from this revision")
    args = parser.parse_args()
    binary = args.binary.resolve() if args.binary else build_binary()
    timelines = list(histories())
    require(len(timelines) == len(set(timelines)) == 96, "The generated history population changed")
    with tempfile.TemporaryDirectory() as directory:
        execute = cli(binary, Path(directory) / "input.json")
        result = {"histories": len(timelines), "samples_per_history": len(SAMPLES),
                  "bounds": bound_results(execute, timelines), "sampling": sampling_results(execute, timelines)}
    output = HERE / "results.json"
    if args.write:
        output.write_text(json.dumps(result, indent=2) + "\n")
    else:
        require(result == json.loads(output.read_text()), "Benchmark differs from results.json. Inspect the changes.")
    print(f"Verified {len(timelines)} work histories, {len(SAMPLES)} samples per history.")


if __name__ == "__main__":
    main()
