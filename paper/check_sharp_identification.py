"""Exact rational reference checks for the paper's interval-identification results.

Run with ``python3 paper/check_sharp_identification.py``.  No external package is
required.  The scalar solver is a difference-constraints reference, not a solver
for the general shared-person, multiple-task linear programme.  Enumeration
checks manufactured finite cases; the manuscript supplies the general proofs.
"""

from dataclasses import dataclass
from fractions import Fraction as F
from itertools import combinations_with_replacement, product
import json


def rational(value):
    """Accept exact inputs only: silently approximated floating point is excluded."""
    if not isinstance(value, (int, F)):
        raise TypeError("Use an int or Fraction for an exact input")
    return F(value)


class InconsistentObservations(ValueError):
    """The stated interval information and scalar capacity have no joint history."""


@dataclass(frozen=True)
class IntervalObservation:
    start: F
    end: F
    lower: F
    upper: F

    @classmethod
    def exact(cls, start, end, mass):
        return cls(start, end, mass, mass)


@dataclass(frozen=True)
class GraphBounds:
    points: tuple
    lower: F
    upper: F
    lower_masses: tuple
    upper_masses: tuple


def interval_cutoff_bounds(start, end, mass, cutoff):
    """Sharp cumulative contribution of one exact single-person interval."""
    start, end, mass, cutoff = map(rational, (start, end, mass, cutoff))
    if not start < end:
        raise ValueError("An observation interval must have positive length")
    if not 0 <= mass <= end - start:
        raise InconsistentObservations("Mass violates single-person capacity")
    cutoff = min(end, max(start, cutoff))
    return max(F(0), mass - (end - cutoff)), min(mass, cutoff - start)


def sharp_stream_bounds(domain_start, domain_end, observations,
                        target_start, target_end):
    """Return sharp scalar interval bounds and exact attainable cell masses.

    Histories are measurable allocations 0 <= a <= 1 on the given domain.
    Every observation bounds an interval integral.  A target may be a point
    (zero mass), and all observation intervals must have positive length.
    Bounds may be negative or exceed capacity: infeasibility is detected from
    the combined constraints rather than repaired or clipped silently.
    """
    domain_start, domain_end, target_start, target_end = map(
        rational, (domain_start, domain_end, target_start, target_end))
    if not domain_start < domain_end:
        raise ValueError("The domain must have positive length")
    if not domain_start <= target_start <= target_end <= domain_end:
        raise ValueError("The target must lie within the domain")
    normalised = []
    endpoints = {domain_start, domain_end, target_start, target_end}
    for observation in observations:
        start, end, lower, upper = map(rational, (
            observation.start, observation.end,
            observation.lower, observation.upper))
        if not domain_start <= start < end <= domain_end:
            raise ValueError("Every observation must lie within the domain")
        if lower > upper:
            raise ValueError("An observation's lower bound exceeds its upper bound")
        normalised.append(IntervalObservation(start, end, lower, upper))
        endpoints.update((start, end))
    points = tuple(sorted(endpoints))
    indices = {point: index for index, point in enumerate(points)}
    size = len(points)
    distances = [[None] * size for _ in range(size)]
    for index in range(size):
        distances[index][index] = F(0)

    def edge(source, destination, weight):
        current = distances[source][destination]
        if current is None or weight < current:
            distances[source][destination] = weight

    for index in range(1, size):
        edge(index - 1, index, points[index] - points[index - 1])
        edge(index, index - 1, F(0))
    for observation in normalised:
        left, right = indices[observation.start], indices[observation.end]
        edge(left, right, observation.upper)
        edge(right, left, -observation.lower)

    # Floyd--Warshall: None represents an absent path; all finite weights are
    # exact rationals.  Adjacent edges eventually make every pair reachable.
    for via in range(size):
        for source in range(size):
            if distances[source][via] is None:
                continue
            for destination in range(size):
                if distances[via][destination] is None:
                    continue
                candidate = distances[source][via] + distances[via][destination]
                current = distances[source][destination]
                if current is None or candidate < current:
                    distances[source][destination] = candidate
    if any(distances[index][index] < 0 for index in range(size)):
        raise InconsistentObservations("A negative cycle certifies inconsistency")

    left, right = indices[target_start], indices[target_end]

    def witness(root):
        return tuple(distances[root][index] - distances[root][index - 1]
                     for index in range(1, size))

    return GraphBounds(points, -distances[right][left], distances[left][right],
                       witness(right), witness(left))


def integrated_mass(points, masses, start, end):
    """Independently integrate the piecewise-constant witness on any subinterval."""
    return sum((mass * max(F(0), min(right, end) - max(left, start)) / (right-left)
                for left, right, mass in zip(points, points[1:], masses)), F(0))


def check_witness(result, observations, target_start, target_end):
    for masses, endpoint in ((result.lower_masses, result.lower),
                             (result.upper_masses, result.upper)):
        assert all(0 <= mass <= right-left for left, right, mass in
                   zip(result.points, result.points[1:], masses))
        for observation in observations:
            observed = integrated_mass(result.points, masses,
                                       observation.start, observation.end)
            assert observation.lower <= observed <= observation.upper
        assert integrated_mass(result.points, masses, target_start, target_end) == endpoint


def check_single_interval():
    checks = 0
    for duration_units in range(1, 9):
        end = F(duration_units, 4)
        for mass_units in range(duration_units + 1):
            mass = F(mass_units, 4)
            observation = IntervalObservation.exact(0, end, mass)
            for cutoff_units in range(duration_units + 1):
                cutoff = F(cutoff_units, 4)
                expected = interval_cutoff_bounds(0, end, mass, cutoff)
                result = sharp_stream_bounds(0, end, (observation,), 0, cutoff)
                assert (result.lower, result.upper) == expected
                check_witness(result, (observation,), 0, cutoff)
                checks += 1
    assert interval_cutoff_bounds(0, 2, F(3, 2), 1) == (F(1, 2), F(1))
    assert interval_cutoff_bounds(0, 2, 1, -1) == (0, 0)
    assert interval_cutoff_bounds(0, 2, 1, 3) == (1, 1)
    return checks


def check_exact_observations_by_enumeration():
    """Group independently enumerated histories by their observation signatures.

    The grids include empty, overlapping, nested, disjoint and repeated
    observations, asymmetric cell lengths, and fractional attention.  The
    expected extrema are obtained only by enumeration, not by graph distances.
    """
    families = ((), ((0, 4),), ((0, 3), (1, 4)), ((0, 4), (1, 3)),
                ((0, 2), (2, 4)), ((0, 1), (1, 3), (3, 4)),
                ((0, 3), (1, 4), (1, 3)), ((0, 4), (0, 4)))
    grids = ((F(0), F(1, 2), F(1), F(3, 2), F(2)),
             (F(0), F(1, 4), F(3, 4), F(1), F(2)))
    quantum = F(1, 4)
    checks = 0
    histories = 0
    for points in grids:
        capacities = [int((right-left)/quantum) for left, right in zip(points, points[1:])]
        allocations = list(product(*(range(capacity+1) for capacity in capacities)))
        histories += len(allocations)
        for family in families:
            groups = {}
            for allocation in allocations:
                signature = tuple(sum(allocation[left:right]) for left, right in family)
                groups.setdefault(signature, []).append(allocation)
            for signature, compatible in groups.items():
                observations = tuple(IntervalObservation.exact(
                    points[left], points[right], quantum * mass)
                    for (left, right), mass in zip(family, signature))
                for left, right in combinations_with_replacement(range(len(points)), 2):
                    values = [quantum * sum(allocation[left:right]) for allocation in compatible]
                    result = sharp_stream_bounds(points[0], points[-1], observations,
                                                 points[left], points[right])
                    assert (result.lower, result.upper) == (min(values), max(values))
                    check_witness(result, observations, points[left], points[right])
                    checks += 1
    return {"enumerated_histories": histories, "target_cases": checks}


def check_bounded_observations_by_enumeration():
    points = tuple(map(F, range(5)))
    allocations = tuple(product((0, 1), repeat=4))
    possible_ranges = tuple(combinations_with_replacement(range(4), 2))
    checks = infeasible = 0
    for first, second in product(possible_ranges, repeat=2):
        observations = (IntervalObservation(0, 3, *first),
                        IntervalObservation(1, 4, *second))
        compatible = [allocation for allocation in allocations
                      if first[0] <= sum(allocation[:3]) <= first[1]
                      and second[0] <= sum(allocation[1:]) <= second[1]]
        for left, right in combinations_with_replacement(range(5), 2):
            try:
                result = sharp_stream_bounds(0, 4, observations, points[left], points[right])
            except InconsistentObservations:
                assert not compatible
                infeasible += 1
            else:
                assert compatible
                values = [sum(allocation[left:right]) for allocation in compatible]
                assert (result.lower, result.upper) == (min(values), max(values))
                check_witness(result, observations, points[left], points[right])
            checks += 1
    return {"target_cases": checks, "infeasible_cases": infeasible}


def check_capacity_and_window_boundaries():
    # The observations span both sides of the target.  Dropping outside cells
    # would incorrectly impose the complete mass inside the clipped window.
    observation = IntervalObservation.exact(0, 3, 2)
    result = sharp_stream_bounds(0, 3, (observation,), 1, 2)
    assert (result.lower, result.upper) == (0, 1)
    check_witness(result, (observation,), 1, 2)

    # Two tasks share one person.  Separately sharp marginal ranges need not
    # yield jointly sharp endpoints when summed.
    feasible = [allocation for allocation in product((0, 1, 2), repeat=2)
                if allocation.count(1) == allocation.count(2) == 1]
    task_a = [int(allocation[0] == 1) for allocation in feasible]
    task_b = [int(allocation[0] == 2) for allocation in feasible]
    total = [a+b for a, b in zip(task_a, task_b)]
    assert (min(task_a), max(task_a)) == (0, 1)
    assert (min(task_b), max(task_b)) == (0, 1)
    assert (min(total), max(total)) == (1, 1)

    # Reconstruct fractional task/role cell masses.  Both role rows of each
    # person share that person's cell capacity; distinct people do not.
    durations = (F(1, 2), F(3, 2))
    rows = ((0, (F(1, 4), F(1, 2))),
            (0, (F(1, 4), F(1))),
            (1, (F(1, 2), F(3, 4))))
    for person in (0, 1):
        for cell, duration in enumerate(durations):
            allocated = sum((masses[cell] / duration for owner, masses in rows
                             if owner == person), F(0))
            assert 0 <= allocated <= 1
    for _, masses in rows:
        assert sum((mass / duration * duration for mass, duration in
                    zip(masses, durations)), F(0)) == sum(masses)


def check_inconsistent_inputs():
    inconsistent = (
        (IntervalObservation.exact(0, 1, 2),),
        (IntervalObservation.exact(0, 1, -1),),
        (IntervalObservation.exact(0, 2, 1), IntervalObservation.exact(0, 2, 2)),
        (IntervalObservation.exact(0, 2, 1), IntervalObservation.exact(0, 1, 1),
         IntervalObservation.exact(1, 2, 1)),
        (IntervalObservation(0, 1, F(3, 2), 2),),
    )
    for observations in inconsistent:
        try:
            sharp_stream_bounds(0, 2, observations, 0, 1)
        except InconsistentObservations:
            pass
        else:
            raise AssertionError("Inconsistent information was accepted")
    for mass in (F(-1, 2), F(5, 2)):
        try:
            interval_cutoff_bounds(0, 2, mass, 1)
        except InconsistentObservations:
            pass
        else:
            raise AssertionError("Invalid scalar mass was accepted")
    return len(inconsistent) + 2


def main():
    single = check_single_interval()
    exact = check_exact_observations_by_enumeration()
    bounded = check_bounded_observations_by_enumeration()
    check_capacity_and_window_boundaries()
    inconsistent = check_inconsistent_inputs()
    print(json.dumps({
        "status": "PASS", "single_interval_cases": single,
        "exact_observation_enumeration": exact,
        "bounded_observation_enumeration": bounded,
        "explicit_inconsistency_cases": inconsistent,
        "capacity_and_window_boundaries": "PASS",
        "arithmetic": "fractions.Fraction; no floating-point optimisation",
        "scope": "Manufactured finite checks and witness validation; general proofs are in the paper."
    }, sort_keys=True))


if __name__ == "__main__":
    main()
