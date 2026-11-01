#!/usr/bin/env python3
"""Derive and check the shared-capacity figure with exact rational arithmetic.

The construction is analytical: summing the two task totals uses both available
person-hours. Each hour must therefore contain exactly one person-hour. Finite
checks below independently reconstruct allocations and test the drawn sets;
they are checks of the construction, not a numerical proof of the continuum.
"""
from __future__ import annotations

import argparse
from fractions import Fraction
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def rational(value: int | str) -> Fraction:
    if isinstance(value, bool) or not isinstance(value, (int, str)):
        raise ValueError("Exact inputs must be integers or rational strings")
    return Fraction(value)


def exact(value: Fraction) -> str:
    return str(value)


def tex_number(value: Fraction) -> str:
    """The plotted coordinates in this construction are integers or halves."""
    if value.denominator == 1:
        return str(value.numerator)
    if value.denominator == 2:
        sign = "-" if value < 0 else ""
        numerator = abs(value.numerator)
        return f"{sign}{numerator // 2}.5"
    raise ValueError(f"Unexpected non-integer/half plotted coordinate: {value}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Reject missing or stale generated files")
    args = parser.parse_args()
    source = ROOT / "inputs.json"
    inputs = json.loads(source.read_text())
    start, end = map(rational, inputs["observation_interval"])
    boundary = rational(inputs["target_boundary"])
    totals = {name: rational(value) for name, value in inputs["task_totals"].items()}
    if (
        inputs["schema_version"] != "1.0"
        or inputs["status"] != "illustrative"
        or inputs["time_unit"] != "hours"
        or inputs["labour_unit"] != "person-hours"
        or inputs["people"] != 1
        or inputs["fractional_allocations"] is not True
        or (start, boundary, end) != (0, 1, 2)
        or totals != {"A": Fraction(1), "B": Fraction(1)}
    ):
        raise ValueError("This figure requires its declared one-person, two-hour, two-task example")
    denominator = inputs["grid_denominator"]
    if isinstance(denominator, bool) or not isinstance(denominator, int) or not 1 <= denominator <= 256:
        raise ValueError("grid_denominator must be an integer from 1 to 256")

    durations = (boundary - start, end - boundary)
    capacity = durations  # One person's time in person-hours.
    marginal = {
        task: (max(Fraction(0), total - capacity[1]), min(total, capacity[0]))
        for task, total in totals.items()
    }
    separate_total = (sum(bounds[0] for bounds in marginal.values()),
                      sum(bounds[1] for bounds in marginal.values()))
    all_labour = sum(totals.values())
    joint_total = (max(separate_total[0], all_labour - capacity[1]),
                   min(separate_total[1], capacity[0]))
    assert joint_total[0] == joint_total[1]
    fixed_sum = joint_total[0]
    xmin = max(marginal["A"][0], fixed_sum - marginal["B"][1])
    xmax = min(marginal["A"][1], fixed_sum - marginal["B"][0])
    endpoints = [(xmin, fixed_sum - xmin), (xmax, fixed_sum - xmax)]
    midpoint = tuple((endpoints[0][k] + endpoints[1][k]) / 2 for k in (0, 1))
    timeline_point = tuple(map(rational, inputs["timeline_point"]))
    if timeline_point != midpoint or inputs["timeline_order_in_each_hour"] != ["A", "B"]:
        raise ValueError("The timeline must realise the derived midpoint, with A then B in each hour")

    def reconstruct(x: Fraction, y: Fraction) -> dict:
        """Evaluate complete piecewise-constant histories, independently of x+y=1."""
        allocations = {
            "A": (x / durations[0], (totals["A"] - x) / durations[1]),
            "B": (y / durations[0], (totals["B"] - y) / durations[1]),
        }
        nonnegative = all(a >= 0 for values in allocations.values() for a in values)
        own_capacity = all(a <= 1 for values in allocations.values() for a in values)
        first_capacity = sum(values[0] for values in allocations.values()) <= 1
        second_capacity = sum(values[1] for values in allocations.values()) <= 1
        observed_totals = all(
            sum(rate * duration for rate, duration in zip(values, durations)) == totals[name]
            for name, values in allocations.items()
        )
        return {
            "feasible": nonnegative and own_capacity and first_capacity and second_capacity and observed_totals,
            "nonnegative": nonnegative,
            "individual_capacity": own_capacity,
            "first_hour_capacity": first_capacity,
            "second_hour_capacity": second_capacity,
            "observed_totals": observed_totals,
            "rates": {name: list(map(exact, values)) for name, values in allocations.items()},
        }

    def drawn_membership(x: Fraction, y: Fraction) -> bool:
        return (
            marginal["A"][0] <= x <= marginal["A"][1]
            and marginal["B"][0] <= y <= marginal["B"][1]
            and x + y == fixed_sum
        )

    grid_counts = {"jointly_feasible": 0, "exceeds_first_hour_capacity": 0,
                   "exceeds_second_hour_capacity": 0}
    for i in range(denominator + 1):
        for j in range(denominator + 1):
            x, y = Fraction(i, denominator), Fraction(j, denominator)
            history = reconstruct(x, y)
            assert history["observed_totals"]
            assert history["feasible"] == drawn_membership(x, y)
            if history["feasible"]:
                grid_counts["jointly_feasible"] += 1
            elif not history["first_hour_capacity"]:
                assert history["second_hour_capacity"]
                grid_counts["exceeds_first_hour_capacity"] += 1
            else:
                assert not history["second_hour_capacity"]
                grid_counts["exceeds_second_hour_capacity"] += 1
    assert grid_counts == {
        "jointly_feasible": denominator + 1,
        "exceeds_first_hour_capacity": denominator * (denominator + 1) // 2,
        "exceeds_second_hour_capacity": denominator * (denominator + 1) // 2,
    }

    witnesses = []
    for point in [*endpoints, midpoint]:
        history = reconstruct(*point)
        assert history["feasible"] and drawn_membership(*point)
        witnesses.append({"first_hour_totals": list(map(exact, point)), **history})
    rejection_points = [(Fraction(1), Fraction(1)), (Fraction(0), Fraction(0)),
                        (Fraction(-1, 2), Fraction(3, 2)), (Fraction(3, 2), Fraction(-1, 2))]
    rejections = []
    for point in rejection_points:
        history = reconstruct(*point)
        assert not history["feasible"] and not drawn_membership(*point)
        rejections.append({"first_hour_totals": list(map(exact, point)), **history})

    # Realise the same point by consecutive full-attention blocks. Independently
    # integrate geometric overlaps rather than infer totals from block labels.
    timeline = []
    for left, right, amount_a, amount_b in (
        (start, boundary, timeline_point[0], timeline_point[1]),
        (boundary, end, totals["A"] - timeline_point[0], totals["B"] - timeline_point[1]),
    ):
        split = left + amount_a
        assert split + amount_b == right
        timeline.extend((("A", left, split), ("B", split, right)))
    def timeline_integral(task: str, left: Fraction, right: Fraction) -> Fraction:
        return sum((max(Fraction(0), min(stop, right) - max(begin, left))
                    for name, begin, stop in timeline if name == task), Fraction(0))
    assert all(begin < stop for _, begin, stop in timeline)
    assert all(a[2] == b[1] for a, b in zip(timeline, timeline[1:]))
    assert timeline[0][1] == start and timeline[-1][2] == end
    for index, name in enumerate(("A", "B")):
        assert timeline_integral(name, start, end) == totals[name]
        assert timeline_integral(name, start, boundary) == timeline_point[index]
    assert sum(timeline_integral(name, start, boundary) for name in totals) == fixed_sum

    calculation = {
        "schema_version": "1.0",
        "status": "derived",
        "input_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "arithmetic": "fractions.Fraction; all reported rational values are exact strings",
        "units": {"time": "hours", "labour": "person-hours", "allocation_rate": "dimensionless"},
        "coordinates": {"x": "Task A labour during the first hour",
                        "y": "Task B labour during the first hour"},
        "marginal_bounds": {name: list(map(exact, bounds)) for name, bounds in marginal.items()},
        "joint_constraints": ["0 <= x <= 1", "0 <= y <= 1", "x + y <= 1",
                              "(1 - x) + (1 - y) <= 1"],
        "joint_segment": {"equation": "x + y = 1", "boundary_included": True,
                          "endpoints": [list(map(exact, p)) for p in endpoints]},
        "combined_first_hour_labour": {"separate_bounds_added": list(map(exact, separate_total)),
                                       "joint_bounds": list(map(exact, joint_total))},
        "continuum_construction": "For each real x in [0,1], set y=1-x. Use constant rates (A,B)=(x,1-x) in the first hour and (1-x,x) in the second. Rates are nonnegative, sum to one in each hour, and give one person-hour per task.",
        "timeline": {
            "point_label": "P", "first_hour_totals": list(map(exact, timeline_point)),
            "allocation_rate_when_active": "1",
            "blocks": [{"task": name, "start": exact(begin), "end": exact(stop)}
                       for name, begin, stop in timeline],
            "interval_convention": "Half-open blocks; the final endpoint takes the final block's value. Endpoint values do not change labour integrals.",
            "checks": {"positive_disjoint_consecutive_blocks": True,
                       "full_interval_totals": {name: exact(timeline_integral(name, start, end)) for name in totals},
                       "first_hour_totals": {name: exact(timeline_integral(name, start, boundary)) for name in totals}},
            "scope": "One constructed attaining schedule for P. Neither the true schedule nor a unique schedule is inferred. Work may switch at arbitrary times.",
        },
        "verification": {"grid_denominator": denominator,
                         "grid_pairs": (denominator + 1) ** 2, **grid_counts,
                         "explicit_feasible_witnesses": witnesses,
                         "explicit_rejections": rejections,
                         "scope": "Finite checks of the derived sets and reconstructed histories; the continuum argument is stated separately."},
    }
    macros = {
        "SCMarginalMinimum": tex_number(marginal["A"][0]),
        "SCMarginalMaximum": tex_number(marginal["A"][1]),
        "SCTargetBoundary": tex_number(boundary),
        "SCObservationEnd": tex_number(end),
        "SCTaskTotal": tex_number(totals["A"]),
        "SCSeparateTotalMinimum": tex_number(separate_total[0]),
        "SCSeparateTotalMaximum": tex_number(separate_total[1]),
        "SCJointTotal": tex_number(fixed_sum),
        "SCFeasibleStart": "(" + ",".join(map(tex_number, endpoints[0])) + ")",
        "SCFeasibleEnd": "(" + ",".join(map(tex_number, endpoints[1])) + ")",
        "SCFeasibleMidpoint": "(" + ",".join(map(tex_number, midpoint)) + ")",
        "SCTimelineX": tex_number(timeline_point[0]),
        "SCTimelineY": tex_number(timeline_point[1]),
        "SCTaskABlocks": ",".join(tex_number(begin) + "/" + tex_number(stop)
                                   for name, begin, stop in timeline if name == "A"),
        "SCTaskBBlocks": ",".join(tex_number(begin) + "/" + tex_number(stop)
                                   for name, begin, stop in timeline if name == "B"),
    }
    coordinates = "% Generated by calculate.py; exact derived values, no graphical styling.\n"
    coordinates += "".join(f"\\def\\{name}{{{value}}}\n" for name, value in macros.items())
    outputs = {"calculation.json": json.dumps(calculation, indent=2) + "\n",
               "coordinates.tex": coordinates}
    stale = []
    for name, content in outputs.items():
        path = ROOT / name
        if args.check:
            if not path.is_file() or path.read_text() != content:
                stale.append(name)
        else:
            path.write_text(content)
    if stale:
        raise SystemExit("Missing or stale generated files: " + ", ".join(stale))
    print(json.dumps({"status": "PASS", "mode": "check" if args.check else "write",
                      "grid_pairs": (denominator + 1) ** 2, **grid_counts,
                      "explicit_feasible_witnesses": len(witnesses),
                      "explicit_rejections": len(rejections)}, sort_keys=True))


if __name__ == "__main__":
    main()
