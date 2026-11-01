#!/usr/bin/env python3
"""Exact, dependency-free finite checks; not a generic LP solver or formal proof.

Run from any directory: python3 path/to/check_model.py
results.json contains deterministic mathematical results; scaling.json contains
measured certificate-verification/reconstruction timings and environment data.
"""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import platform
import resource
import sys
import time
import tracemalloc
from fractions import Fraction as F


HERE = Path(__file__).resolve().parent
ZERO = F(0)
ONE = F(1)


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def encode(value):
    if isinstance(value, F):
        return str(value)
    if isinstance(value, dict):
        return {str(k): encode(v) for k, v in value.items()}
    if isinstance(value, (tuple, list)):
        return [encode(v) for v in value]
    return value


def dot(a, b, counts=None):
    require(len(a) == len(b), "dot-product dimension mismatch")
    if counts is not None:
        counts["rational_multiplications"] += len(a)
        counts["rational_additions"] += len(a)
    return sum((x * y for x, y in zip(a, b)), ZERO)


def blank_counts():
    return {"rational_multiplications": 0, "rational_additions": 0,
            "order_comparisons": 0, "equality_comparisons": 0}


def feasible(A, b, x, counts=None):
    require(len(A) == len(b), "row-count mismatch")
    if counts is not None:
        counts["order_comparisons"] += len(x) + len(A)
    return all(v >= 0 for v in x) and all(dot(row, x, counts) <= bound
                                        for row, bound in zip(A, b))


def certificate(A, b, w, xlow, xhigh, mu, lam, counts=None):
    """Check every primal/dual inequality and both zero gaps independently."""
    require(feasible(A, b, xlow, counts), "lower primal is infeasible")
    require(feasible(A, b, xhigh, counts), "upper primal is infeasible")
    require(len(mu) == len(lam) == len(A), "dual dimension mismatch")
    if counts is not None:
        counts["order_comparisons"] += 2 * len(A) + 2 * len(w)
        counts["equality_comparisons"] += 2
    require(all(v >= 0 for v in mu + lam), "negative dual coefficient")
    for j, weight in enumerate(w):
        column = [row[j] for row in A]
        require(dot(column, lam, counts) >= weight, "upper dual inequality")
        require(dot(column, mu, counts) >= -weight, "lower dual inequality")
    primal_low, primal_high = dot(w, xlow, counts), dot(w, xhigh, counts)
    dual_low, dual_high = -dot(b, mu, counts), dot(b, lam, counts)
    require(primal_low == dual_low, "nonzero lower optimality gap")
    require(primal_high == dual_high, "nonzero upper optimality gap")
    return {"lower": primal_low, "upper": primal_high,
            "lower_gap": primal_low - dual_low,
            "upper_gap": dual_high - primal_high}


def reconstruct(endpoints, persons, masses, stats=None):
    """Consecutive full-attention blocks, with explicit residual idle blocks.

    masses[stream][cell] has person-hour units. A block is
    (person, stream-or-None, start, end). Input stream order fixes tie-breaking.
    The constructor visits each mass once after indexing streams by person.
    """
    K = len(endpoints) - 1
    require(K >= 1 and all(a < b for a, b in zip(endpoints, endpoints[1:])),
            "endpoints must be strictly increasing")
    if stats is not None:
        stats["order_comparisons"] += K
    require(len(persons) == len(masses), "stream/person count mismatch")
    by_person = {}
    for stream, person in enumerate(persons):
        require(len(masses[stream]) == K, "cell count mismatch")
        by_person.setdefault(person, []).append(stream)
    blocks = []
    for person, streams in by_person.items():
        for k, (left, right) in enumerate(zip(endpoints, endpoints[1:])):
            cursor = left
            if stats is not None:
                stats["person_cells_visited"] += 1
            for stream in streams:
                mass = masses[stream][k]
                require(mass >= 0, "negative stream-cell mass")
                if stats is not None:
                    stats["mass_entries_visited"] += 1
                    stats["rational_additions"] += 1
                    stats["order_comparisons"] += 2
                end = cursor + mass
                require(end <= right, "shared capacity exceeded")
                if mass:
                    blocks.append((person, stream, cursor, end))
                cursor = end
            if stats is not None:
                stats["order_comparisons"] += 1
            if cursor < right:
                blocks.append((person, None, cursor, right))
    return blocks


def overlap(left, right, lo, hi):
    return max(ZERO, min(right, hi) - max(left, lo))


def integral(blocks, streams, lo, hi):
    """Independently integrate actual time intervals, ignoring cell indices."""
    selected = set(streams)
    return sum((overlap(left, right, lo, hi)
                for _, stream, left, right in blocks
                if stream is not None and stream in selected), ZERO)


def verify_schedule(endpoints, persons, masses, blocks):
    """Check 0/1 capacity and cell integrals from interval geometry."""
    for person in dict.fromkeys(persons):
        person_blocks = [(s, l, r) for p, s, l, r in blocks if p == person]
        require(person_blocks, "person has no activity or idle blocks")
        cursor = endpoints[0]
        for stream, left, right in person_blocks:
            require(left == cursor and right > left, "blocks overlap or leave a gap")
            require(stream is None or persons[stream] == person,
                    "stream attributed to wrong person")
            cursor = right
        require(cursor == endpoints[-1], "schedule does not cover domain")
    checks = 0
    for stream, row in enumerate(masses):
        for k, mass in enumerate(row):
            require(integral(blocks, [stream], endpoints[k], endpoints[k + 1])
                    == mass, "reconstructed cell integral differs")
            checks += 1
    return checks


def cell_integral(endpoints, masses, streams, lo, hi):
    """Original piecewise-constant history, independent of block ordering."""
    return sum((masses[s][k] * overlap(left, right, lo, hi) / (right - left)
                for s in streams
                for k, (left, right) in enumerate(zip(endpoints, endpoints[1:]))), ZERO)


def query_check(endpoints, persons, masses, queries):
    blocks = reconstruct(endpoints, persons, masses)
    checks = verify_schedule(endpoints, persons, masses, blocks)
    for streams, lo, hi in queries:
        expected = cell_integral(endpoints, masses, streams, lo, hi)
        require(integral(blocks, streams, lo, hi) == expected,
                "observation/target interval integral differs")
        checks += 1
    return blocks, checks


def exhaustive_reconstruction():
    quarter_simplex = [tuple(F(v, 4) for v in values)
                       for values in itertools.product(range(5), repeat=3)
                       if sum(values) <= 4]
    require(len(quarter_simplex) == 35, "simplex enumeration count")
    endpoints = [F(0), F(1), F(2)]
    subsets = [tuple(s for s in range(3) if mask & (1 << s))
               for mask in range(1, 8)]
    queries = [(streams, endpoints[p], endpoints[q])
               for streams in subsets for p in range(2)
               for q in range(p + 1, 3)]
    histories = checks = block_count = idle_count = 0
    for first, second in itertools.product(quarter_simplex, repeat=2):
        masses = [[first[s], second[s]] for s in range(3)]
        blocks, checked = query_check(endpoints, [0, 0, 0], masses, queries)
        histories += 1
        checks += checked
        block_count += len(blocks)
        idle_count += sum(s is None for _, s, _, _ in blocks)
    return {"mass_grid": "multiples of 1/4 person-hour",
            "streams": 3, "people": 1, "unit_cells": 2,
            "feasible_single_cell_vectors": len(quarter_simplex),
            "exhaustive_two_cell_histories": histories,
            "all_nonempty_stream_subsets": len(subsets),
            "interval_queries_per_history": len(queries),
            "independent_integral_equalities_checked": checks,
            "constructed_activity_and_idle_blocks": block_count,
            "constructed_idle_blocks": idle_count}


def mixed_person_and_target_case():
    observations = [(F(0), F(3, 2)), (F(1, 2), F(2))]
    target = (F(3, 4), F(5, 4))
    endpoints = sorted({F(0), F(2), *[v for p in observations for v in p], *target})
    lengths = [b - a for a, b in zip(endpoints, endpoints[1:])]
    persons = [0, 0, 0, 1, 1]
    shares = [[F(1, 2), ZERO, F(1, 4), ZERO, ONE],
              [F(1, 4), ZERO, F(1, 4), ZERO, ZERO],
              [ZERO, ZERO, F(1, 2), ZERO, ZERO],
              [ZERO, ONE, ZERO, F(1, 3), ZERO],
              [ZERO, ZERO, ZERO, F(1, 3), F(1, 2)]]
    masses = [[share * duration for share, duration in zip(row, lengths)]
              for row in shares]
    queries = [(streams, lo, hi) for streams in [(0, 1), (1, 2, 3), tuple(range(5))]
               for lo, hi in observations + [target]]
    blocks, checks = query_check(endpoints, persons, masses, queries)
    weights = [F(1), F(-2), F(1, 3), ZERO, F(5, 4)]
    history_target = sum((weights[s] * integral(blocks, [s], *target)
                          for s in range(5)), ZERO)
    cell_target = sum((weights[s] * cell_integral(endpoints, masses, [s], *target)
                       for s in range(5)), ZERO)
    require(history_target == cell_target, "signed target changed")
    return {"endpoints": endpoints, "observation_intervals": observations,
            "target_interval": target, "target_only_boundaries": target,
            "people": 2, "streams": 5, "independent_equalities_checked": checks + 1,
            "signed_weight_target": history_target,
            "zero_mass_entries": sum(v == 0 for row in masses for v in row),
            "idle_blocks": sum(s is None for _, s, _, _ in blocks),
            "blocks": [{"person": p, "stream": s, "start": l, "end": r}
                       for p, s, l, r in blocks]}


def scalar_grid():
    cases = endpoint_cases = candidates = feasible_candidates = 0
    for duration_quarters in range(1, 9):
        end = F(duration_quarters, 4)
        for cut_quarters in range(duration_quarters + 1):
            cut = F(cut_quarters, 4)
            for volume_quarters in range(duration_quarters + 1):
                volume = F(volume_quarters, 4)
                low, high = max(ZERO, volume - (end - cut)), min(volume, cut)
                accepted = []
                for first_quarters in range(duration_quarters + 1):
                    first = F(first_quarters, 4)
                    second = volume - first
                    valid = ZERO <= first <= cut and ZERO <= second <= end - cut
                    require(valid == (low <= first <= high), "scalar inequalities differ")
                    candidates += 1
                    if not valid:
                        continue
                    endpoints = sorted({ZERO, cut, end})
                    masses = [[first, second]] if ZERO < cut < end else [[volume]]
                    blocks, _ = query_check(endpoints, [0], masses, [])
                    require(integral(blocks, [0], ZERO, cut) == first,
                            "scalar cutoff integral differs")
                    require(integral(blocks, [0], ZERO, end) == volume,
                            "scalar observation integral differs")
                    accepted.append(first)
                    feasible_candidates += 1
                require(min(accepted) == low and max(accepted) == high,
                        "enumerated scalar extrema differ")
                cases += 1
                endpoint_cases += int(cut == ZERO or cut == end)
    invalid_volumes = [F(-1, 4), F(9, 4)]
    for volume in invalid_volumes:
        require(not any(ZERO <= F(first, 4) <= ONE
                        and ZERO <= volume - F(first, 4) <= ONE
                        for first in range(5)), "invalid scalar total admitted")
    return {"duration_quarters": [1, 8], "parameter_cases": cases,
            "endpoint_cutoff_cases": endpoint_cases,
            "candidate_splits_checked": candidates,
            "feasible_schedules_integrated": feasible_candidates,
            "infeasible_volume_examples_on_length_two": invalid_volumes,
            "infeasible_volume_grid_checks": 2,
            "exact_infeasibility_certificates": [
                infeasibility_certificate([[ONE, ONE]], [-F(1, 4)], [ONE]),
                infeasibility_certificate([[ONE, ZERO], [ZERO, ONE], [-ONE, -ONE]],
                                           [ONE, ONE, -F(9, 4)], [ONE, ONE, ONE])]}


def infeasibility_certificate(A, b, multipliers):
    """For x>=0, y>=0, A^T y>=0 and b^T y<0 imply infeasibility."""
    require(len(A) == len(b) == len(multipliers), "infeasibility dimensions")
    require(all(y >= 0 for y in multipliers), "negative contradiction multiplier")
    combined_row = [dot([row[j] for row in A], multipliers)
                    for j in range(len(A[0]))]
    combined_rhs = dot(b, multipliers)
    require(all(v >= 0 for v in combined_row) and combined_rhs < 0,
            "arithmetic does not certify infeasibility")
    return {"A": A, "b": b, "nonnegative_row_multipliers": multipliers,
            "combined_row": combined_row, "combined_rhs": combined_rhs,
            "contradiction": "For x>=0 the combined left side is nonnegative, but must be at most the negative combined right side."}


def availability_and_infeasibility():
    # x=(A1,A2,A3,B1,B2,B3); no availability in the middle unit cell.
    A = [[ONE, ZERO, ZERO, ONE, ZERO, ZERO],
         [ZERO, ONE, ZERO, ZERO, ONE, ZERO],
         [ZERO, ZERO, ONE, ZERO, ZERO, ONE],
         [ZERO, ONE, ZERO, ZERO, ONE, ZERO],
         [ONE, ONE, ONE, ZERO, ZERO, ZERO],
         [-ONE, -ONE, -ONE, ZERO, ZERO, ZERO]]
    b = [ONE, ONE, ONE, ZERO, ONE, -ONE]
    witness = [F(1, 2), ZERO, F(1, 2), F(1, 4), ZERO, F(1, 4)]
    require(feasible(A, b, witness), "availability witness infeasible")
    blocks, checked = query_check([ZERO, ONE, F(2), F(3)], [0, 0],
                                  [witness[:3], witness[3:]],
                                  [((0, 1), ONE, F(2))])
    require(integral(blocks, [0, 1], ONE, F(2)) == ZERO,
            "work scheduled during unavailability")
    unavailable_contradiction = infeasibility_certificate(
        [[ONE, ONE], [ONE, ONE], [-ONE, ZERO]],
        [ONE, ZERO, -F(1, 4)], [ZERO, ONE, ONE])
    excessive_total_contradiction = infeasibility_certificate(
        [[ONE, ZERO], [ZERO, ONE], [-ONE, -ONE]],
        [ONE, ONE, -F(3)], [ONE, ONE, ONE])
    return {"feasible_unavailability_case": {"A": A, "b": b,
                "witness": witness, "unavailable_interval": [ONE, F(2)],
                "integrated_unavailable_labour": ZERO,
                "independent_integral_equalities_checked": checked},
            "infeasible_unavailability_case": unavailable_contradiction,
            "infeasible_observation_exceeding_capacity": excessive_total_contradiction,
            "scope": "Two explicit nonnegative row-combination certificates detect these inconsistent inputs; no generic feasibility solver is implemented."}


def figure_model():
    # x=(A1,A2,B1,B2); nonnegativity is implicit, observation lower rows negated.
    A = [[ONE, ZERO, ONE, ZERO], [ZERO, ONE, ZERO, ONE],
         [ONE, ONE, ZERO, ZERO], [-ONE, -ONE, ZERO, ZERO],
         [ZERO, ZERO, ONE, ONE], [ZERO, ZERO, -ONE, -ONE]]
    b = [ONE, ONE, ONE, -ONE, ONE, -ONE]
    return A, b


def figure_and_relaxation():
    A, b = figure_model()
    w = [ONE, ZERO, ONE, ZERO]
    lam = [ONE, ZERO, ZERO, ZERO, ZERO, ZERO]
    mu = [ZERO, ONE, ZERO, ONE, ZERO, ONE]
    original = certificate(A, b, w, [ZERO, ONE, ONE, ZERO],
                           [ONE, ZERO, ZERO, ONE], mu, lam)
    grid = [F(i, 4) for i in range(5)]
    accepted = [list(x) for x in itertools.product(grid, repeat=4) if feasible(A, b, x)]
    require(len(accepted) == 5, "figure grid count")
    values = [dot(w, x) for x in accepted]
    require(min(values) == original["lower"] and max(values) == original["upper"],
            "figure joint range differs")
    single_w = [ONE, ZERO, ZERO, ZERO]
    single = certificate(A, b, single_w, [ZERO, ONE, ONE, ZERO],
                         [ONE, ZERO, ZERO, ONE], [ZERO] * 6, lam)
    intermediate = []
    for step in range(9):
        t = F(step, 8)
        x = [t, ONE - t, ONE - t, t]
        require(feasible(A, b, x), "intermediate primal infeasible")
        masses = [x[:2], x[2:]]
        blocks, _ = query_check([ZERO, ONE, F(2)], [0, 0], masses,
                                [((0, 1), ZERO, ONE)])
        target = integral(blocks, [0], ZERO, ONE)
        require(target == t, "intermediate target not attained")
        intermediate.append(target)
    relaxed_cases = []
    for eps in [ZERO, F(1, 8), F(1, 4), F(1, 2)]:
        e = [ZERO, ZERO, eps, eps, eps, eps]
        relaxed_b = [base + extra for base, extra in zip(b, e)]
        low = [F(1, 2) - eps, F(1, 2), F(1, 2) - eps, F(1, 2)]
        high = [F(1, 2), F(1, 2) - eps, F(1, 2), F(1, 2) - eps]
        bounds = certificate(A, relaxed_b, w, low, high, mu, lam)
        for x in accepted:
            require(feasible(A, relaxed_b, x), "relaxation lost original point")
        for x in [low, high]:
            query_check([ZERO, ONE, F(2)], [0, 0], [x[:2], x[2:]],
                        [((0,), ZERO, F(2)), ((1,), ZERO, F(2)),
                         ((0, 1), ZERO, ONE)])
        relaxed_cases.append({"epsilon": eps, "rhs_increments": e,
                              "lower_witness": low, "upper_witness": high,
                              "bounds": bounds})
    # A valid reused certificate can be conservative after relaxation.
    duplicate_A = A + [list(A[3])]
    duplicate_b = b + [b[3]]
    e = [ZERO, ZERO, ZERO, F(1, 4), ZERO, ZERO, ZERO]
    relaxed_b = [base + extra for base, extra in zip(duplicate_b, e)]
    reused_mu = mu + [ZERO]
    sharp_mu = [ZERO, ONE, ZERO, ZERO, ZERO, ONE, ONE]
    sharp = certificate(duplicate_A, relaxed_b, w, accepted[0], accepted[-1],
                        sharp_mu, lam + [ZERO])
    for j in range(4):
        require(dot([row[j] for row in duplicate_A], reused_mu) >= -w[j],
                "reused lower certificate invalid")
    reused_bound = -dot(relaxed_b, reused_mu)
    require(reused_bound < sharp["lower"], "conservatism example is not strict")
    return {"variable_order": ["A1", "A2", "B1", "B2"], "A": A, "b": b,
            "joint_target_weights": w, "upper_dual": lam, "lower_dual": mu,
            "exact_joint_bounds": original, "single_task_bounds": single,
            "grid_candidates": 5 ** 4, "feasible_grid_vectors": accepted,
            "intermediate_single_task_targets_reconstructed": intermediate,
            "observation_only_relaxations": relaxed_cases,
            "conservative_reused_certificate_case": {
                "extra_row": duplicate_A[-1], "rhs_increments": e,
                "reused_lower_bound": reused_bound, "exact_bounds": sharp,
                "explanation": "The duplicate unrelaxed lower observation still binds."}}


def determinant(matrix):
    # Leibniz definition, independent of solving the corresponding equations.
    n = len(matrix)
    total = ZERO
    for permutation in itertools.permutations(range(n)):
        inversions = sum(permutation[i] > permutation[j]
                         for i in range(n) for j in range(i + 1, n))
        product = ONE
        for i, j in enumerate(permutation):
            product *= matrix[i][j]
        total += (-1 if inversions % 2 else 1) * product
    return total


def structural_counterexamples():
    matrix = [[ONE, ONE, ZERO], [ONE, ZERO, ONE], [ZERO, ONE, ONE]]
    point = [F(1, 2)] * 3
    det = determinant(matrix)
    require(abs(det) > 1, "TU witness determinant does not refute TU")
    require(all(dot(row, point) == ONE for row in matrix), "pair-sum point infeasible")
    blocks, _ = query_check([ZERO, F(2)], [0, 0, 0], [[v] for v in point],
                            [((0, 1), ZERO, F(2)), ((0, 2), ZERO, F(2)),
                             ((1, 2), ZERO, F(2))])
    require(sum(point) <= 2, "TU point violates shared capacity")
    # One uninterrupted full-attention session must be an interval [s,s+1].
    # First-cell mass 1/2 requires s=1/2 when the total length is 1; its
    # third-cell mass is then zero, so the required third-cell mass is impossible.
    masses = [[F(1, 2), ZERO, F(1, 2)]]
    nonpreemptive_blocks, _ = query_check([ZERO, ONE, F(2), F(3)], [0], masses, [])
    first_possible_start = F(1, 2)
    forced_session = (first_possible_start, first_possible_start + ONE)
    forced_integrals = [overlap(*forced_session, F(k), F(k + 1)) for k in range(3)]
    require(forced_integrals != masses[0], "nonpreemption counterexample failed")
    latest_possible_start = ONE - masses[0][0]
    earliest_possible_end = F(2) + masses[0][2]
    minimum_support_span = earliest_possible_end - latest_possible_start
    total_active_length = sum(masses[0], ZERO)
    require(minimum_support_span > total_active_length,
            "uninterrupted-session span does not contradict active length")
    interval_candidates = 0
    for left, right in itertools.combinations([F(i, 4) for i in range(13)], 2):
        interval_candidates += 1
        require([overlap(left, right, F(k), F(k + 1)) for k in range(3)] != masses[0],
                "a grid interval unexpectedly matches interrupted masses")
    # Coupled feasible line Q=H, 1<=H<=2, gives ratio identically one.
    samples = [ONE + F(i, 16) for i in range(17)]
    ratios = [h / h for h in samples]
    require(all(r == ONE for r in ratios), "coupled ratio arithmetic failed")
    marginal_box = [ONE / F(2), F(2) / ONE]
    return {"not_totally_unimodular": {"submatrix": matrix, "determinant": det,
             "integral_rhs": [ONE] * 3, "unique_solution": point,
             "uniqueness_checked_by_nonzero_determinant": det != 0,
             "cell_duration": F(2), "capacity_used": sum(point),
             "binary_activity_blocks": len(blocks)},
            "nonpreemption_failure": {"unit_cell_masses": masses[0],
             "meaning": "one uninterrupted full-attention session for the stream",
             "total_session_length": ONE,
             "start_forced_by_first_cell_mass": first_possible_start,
             "forced_session_cell_integrals": forced_integrals,
             "minimum_session_support_span": minimum_support_span,
             "total_active_length": total_active_length,
             "quarter_grid_intervals_ruled_out": interval_candidates,
             "preemptive_binary_active_blocks":
                 [[l, r] for _, s, l, r in nonpreemptive_blocks if s is not None],
             "argument": "A length-one interval with positive mass in cells 1 and 3 must cross the entire zero middle cell; this is impossible."},
            "coupled_output_labour_ratio": {"joint_set": "Q=H, 1<=H<=2",
             "identity": "Q/H=H/H=1 because H>0",
             "exact_ratio_range": [ONE, ONE], "finite_identity_checks": len(samples),
             "marginal_box_ratio_range": marginal_box,
             "counterexample_pairs_not_jointly_feasible": [[ONE, F(2)], [F(2), ONE]]}}


def build_scaling_case(N):
    require(N % 4 == 0, "scaling dimension must be divisible by four")
    m = N // 4
    A, b, w = [], [], [ZERO] * N
    xlow, xhigh = [ZERO] * N, [ZERO] * N
    mu, lam = [ZERO] * (6 * m), [ZERO] * (6 * m)
    persons, low_masses, high_masses = [], [], []
    local_A, _ = figure_model()
    for block in range(m):
        a = F(1 + (3 * block) % 7, 4)
        alpha = F(1 + (5 * block) % 31, 16)
        bounds = [ONE, ONE, a, -a, F(2) - a, a - F(2)]
        for local_row, bound in zip(local_A, bounds):
            row = [ZERO] * N
            row[4 * block:4 * block + 4] = local_row
            A.append(row)
            b.append(bound)
        low, high = max(ZERO, a - ONE), min(a, ONE)
        lo = [low, a - low, ONE - low, ONE - a + low]
        hi = [high, a - high, ONE - high, ONE - a + high]
        xlow[4 * block:4 * block + 4] = lo
        xhigh[4 * block:4 * block + 4] = hi
        w[4 * block] = alpha
        lam[6 * block + (2 if a < ONE else 0)] = alpha
        if a > ONE:
            mu[6 * block + 1] = alpha
            mu[6 * block + 3] = alpha
        persons.extend([block, block])
        low_masses.extend([lo[:2], lo[2:]])
        high_masses.extend([hi[:2], hi[2:]])
    return A, b, w, xlow, xhigh, mu, lam, persons, low_masses, high_masses


def max_bits(values):
    return max((max(abs(v.numerator).bit_length(), v.denominator.bit_length())
                for v in values), default=0)


def schedule_weighted_target(blocks, endpoints, weights, stats=None):
    """Evaluate one cell-constant linear target by chronological interval scan.

    Each block is contained in one cell. Two advancing cell pointers per person
    suffice; thus this scan is O(blocks+person*cells), after indexed ordering.
    """
    total = ZERO
    cell_for_person = {}
    for person, stream, left, right in blocks:
        k = cell_for_person.get(person, 0)
        while k + 1 < len(endpoints) - 1 and left >= endpoints[k + 1]:
            k += 1
            if stats is not None:
                stats["cell_pointer_advances"] += 1
        cell_for_person[person] = k
        require(endpoints[k] <= left < right <= endpoints[k + 1],
                "constructed interval crosses cell boundary")
        if stats is not None:
            stats["blocks_visited"] += 1
        if stream is not None:
            total += (right - left) * weights[(len(endpoints) - 1) * stream + k]
            if stats is not None:
                stats["active_blocks_visited"] += 1
                stats["rational_subtractions"] += 1
                stats["rational_multiplications"] += 1
                stats["rational_additions"] += 1
    return total


def scaling():
    cases, mathematical_cases = [], []
    total_start = time.perf_counter_ns()
    for N in [4, 8, 16, 32, 48]:
        tracemalloc.start()
        case_start = time.perf_counter_ns()
        start = time.perf_counter_ns()
        data = build_scaling_case(N)
        (A, b, w, xlow, xhigh, mu, lam, persons, low_masses, high_masses) = data
        # A real sorting step is recorded, although this family has only 3 times.
        endpoints = sorted(set([F(2), ZERO, ONE, ONE, F(2), ZERO]))
        assembly_ns = time.perf_counter_ns() - start
        counts = blank_counts()
        start = time.perf_counter_ns()
        bounds = certificate(A, b, w, xlow, xhigh, mu, lam, counts)
        certificate_ns = time.perf_counter_ns() - start
        reconstruction_counts = {"person_cells_visited": 0, "mass_entries_visited": 0,
                                 "rational_additions": 0, "order_comparisons": 0}
        start = time.perf_counter_ns()
        low_blocks = reconstruct(endpoints, persons, low_masses, reconstruction_counts)
        high_blocks = reconstruct(endpoints, persons, high_masses, reconstruction_counts)
        reconstruction_ns = time.perf_counter_ns() - start
        target_counts = {"blocks_visited": 0, "active_blocks_visited": 0,
                         "cell_pointer_advances": 0, "rational_subtractions": 0,
                         "rational_multiplications": 0, "rational_additions": 0}
        start = time.perf_counter_ns()
        schedule_low = schedule_weighted_target(low_blocks, endpoints, w, target_counts)
        schedule_high = schedule_weighted_target(high_blocks, endpoints, w, target_counts)
        require(schedule_low == bounds["lower"] and schedule_high == bounds["upper"],
                "scaling schedule target differs from primal/dual bound")
        target_ns = time.perf_counter_ns() - start
        start = time.perf_counter_ns()
        independent_checks = (verify_schedule(endpoints, persons, low_masses, low_blocks)
                              + verify_schedule(endpoints, persons, high_masses, high_blocks))
        independent_ns = time.perf_counter_ns() - start
        peak_bytes = tracemalloc.get_traced_memory()[1]
        tracemalloc.stop()
        case_ns = time.perf_counter_ns() - case_start
        rss_raw = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        # macOS reports bytes; Linux reports KiB. Keep the memory limit in KiB.
        rss_kib = (rss_raw + 1023) // 1024 if sys.platform == "darwin" else rss_raw
        input_values = [v for row in A for v in row] + b + w
        input_bits = max_bits(input_values)
        require(len(A) + N <= 144, "registered inequality limit exceeded")
        require(input_bits <= 16, "registered rational bit limit exceeded")
        require(case_ns <= 15 * 10**9, "registered per-case time exceeded")
        require(rss_kib <= 1024 * 1024, "registered process memory exceeded")
        # Two primal products, two transposed dual products, objective products:
        # 4*rows*N + 2*N + 2*rows products/additions, including explicit zeros.
        expected_arithmetic = 4 * len(A) * N + 2 * N + 2 * len(A)
        require(counts["rational_multiplications"] == expected_arithmetic,
                "dense arithmetic count differs from loops")
        require(counts["rational_additions"] == expected_arithmetic,
                "dense addition count differs from loops")
        require(reconstruction_counts["mass_entries_visited"] == 2 * N,
                "reconstruction did not make one visit per endpoint mass")
        mathematical_cases.append({"variables": N, "explicit_rows": len(A),
            "inequalities_including_nonnegativity": len(A) + N,
            "maximum_input_numerator_or_denominator_bits": input_bits,
            "bounds": bounds, "certificate_operation_counts": counts,
            "reconstruction_operation_counts": reconstruction_counts,
            "target_operation_counts": target_counts,
            "independently_integrated_cell_masses": independent_checks,
            "constructed_blocks_both_endpoints": len(low_blocks) + len(high_blocks)})
        cases.append({"variables": N, "explicit_rows": len(A),
            "inequalities_including_nonnegativity": len(A) + N,
            "nanoseconds": {"assembly_and_endpoint_sorting": assembly_ns,
                "exact_primal_dual_certificate_checks": certificate_ns,
                "binary_reconstruction": reconstruction_ns,
                "schedule_target_evaluation": target_ns,
                "independent_interval_integration_checks": independent_ns,
                "whole_case": case_ns},
            "tracemalloc_peak_bytes": peak_bytes,
            "process_cumulative_peak_rss_KiB": rss_kib})
    total_ns = time.perf_counter_ns() - total_start
    require(total_ns <= 180 * 10**9, "registered total computation time exceeded")
    timing = {"benchmark_scope": "Exact certificate verification and reconstruction for a structured family; no generic LP solve is timed.",
        "family": "Independent two-task/two-cell/person blocks, rational task totals and target weights; matching primal and dual certificates are constructed.",
        "environment": {"platform": platform.platform(), "python": sys.version,
            "python_executable": sys.executable, "implementation": platform.python_implementation(),
            "machine": platform.machine(), "logical_cpus": os.cpu_count(),
            "clock": "time.perf_counter_ns", "memory": "tracemalloc plus ru_maxrss normalised to KiB (macOS bytes, Linux KiB), cumulative for this process"},
        "registered_limits": {"dimensions": [4, 8, 16, 32, 48],
            "maximum_inequalities_including_nonnegativity": 144,
            "maximum_input_numerator_or_denominator_bits": 16,
            "per_case_seconds": 15, "total_seconds": 180, "peak_memory_MiB": 1024},
        "all_registered_limits_passed": True, "total_scaling_nanoseconds": total_ns,
        "cases": cases,
        "complexity": {"dense_certificate_dot_product_arithmetic": "4*R*N+2*N+2*R products and the same number of additions in the checked dot products: O(R*N), rational bit costs excluded. Counter records dot-product products/additions and rational comparisons, not Python bookkeeping or unary negation.",
            "endpoint_sorting": "O(E log E) comparisons for E supplied endpoints; this structured family has 6 supplied, 3 distinct endpoints.",
            "reconstruction": "After ordering/indexing: one visit to each mass and each person-cell, O(N+P*K); 2*N mass visits for two endpoint histories.",
            "target_scan": "O(B+P*K), B constructed activity/idle blocks, using advancing cell pointers.",
            "independent_integral_check": "Separate geometric overlap checks, O(N*B) worst case; not included in the linear reconstruction claim.",
            "assembly_storage": "Dense matrix has R*N entries; assembly and storage O(R*N).",
            "scope_limit": "Five structured cases do not establish generic LP-solver scaling, arbitrary-input tractability or empirical applicability."}}
    return mathematical_cases, timing


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true",
                      help="recompute exact cases, compare results.json, write no files")
    mode.add_argument("--record", action="store_true",
                      help="record deterministic results and fresh measurements (default)")
    args = parser.parse_args()
    overall_start = time.perf_counter_ns()
    result = {"schema": "exact-cell-model-checks/1", "status": "passed",
        "arithmetic": "fractions.Fraction, exact rational comparisons; Python standard library only",
        "authority": "Finite checks support the stated proofs; they are neither a formal proof nor empirical validation.",
        "reconstruction": exhaustive_reconstruction(),
        "overlapping_observations_multiple_people_and_target_boundaries": mixed_person_and_target_case(),
        "scalar_cutoff_grid": scalar_grid(),
        "availability_and_infeasibility": availability_and_infeasibility(),
        "figure_and_dual_robustness": figure_and_relaxation(),
        "counterexamples": structural_counterexamples()}
    math_scaling, timings = scaling()
    result["structured_scaling_exact_results"] = math_scaling
    timings["whole_check_nanoseconds"] = time.perf_counter_ns() - overall_start
    require(timings["whole_check_nanoseconds"] <= 180 * 10**9,
            "whole finite check exceeds registered total time")
    timings["script_sha256"] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    if args.check:
        require(json.loads((HERE / "results.json").read_text()) == encode(result),
                "recomputed exact results differ from recorded results.json; review and --record deliberately")
    else:
        for name, content in [("results.json", result), ("scaling.json", timings)]:
            (HERE / name).write_text(json.dumps(encode(content), indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": "passed", "mode": "check" if args.check else "record",
                      "histories": result["reconstruction"]["exhaustive_two_cell_histories"],
                      "scalar_parameter_cases": result["scalar_cutoff_grid"]["parameter_cases"],
                      "scaling_dimensions": [r["variables"] for r in math_scaling],
                      "outputs": [str(HERE / name) for name in ["results.json", "scaling.json"]]}))


if __name__ == "__main__":
    main()
