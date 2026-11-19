"""Exact finite checks of the labour sampling, forecast and continuous-bin examples.

These checks supplement the manuscript's analytical arguments; they do not
validate observed allocations or extend the historical Lean proof inventory.
"""
from fractions import Fraction as F
from itertools import combinations, product
import json


def mean(values):
    return sum(values, F(0)) / len(values)


def check_sampling_noise():
    allocations = (1, 1, 1, 1, 0, 0, 0, 0)
    labour = F(sum(allocations))
    estimates = [F(8 * value) for value in allocations]
    variance = mean([(estimate - labour) ** 2 for estimate in estimates])
    assert mean(estimates) == labour == 4
    for forecast in map(F, (0, 2, 4, 8)):
        assert mean([(estimate - forecast) ** 2 for estimate in estimates]) == (
            (labour - forecast) ** 2 + variance
        )
        assert mean([estimate - forecast for estimate in estimates]) == labour - forecast
    for first, second in product(map(F, (0, 2, 4, 8)), repeat=2):
        differences = [(first - estimate) ** 2 - (second - estimate) ** 2
                       for estimate in estimates]
        expected_difference = (first - labour) ** 2 - (second - labour) ** 2
        assert mean(differences) == expected_difference
        assert mean([(v - expected_difference) ** 2 for v in differences]) == (
            4 * (first - second) ** 2 * variance
        )
    assert mean([abs(estimate - labour) for estimate in estimates]) == 4
    assert F(7, 2) <= labour <= F(9, 2)
    assert not any(F(7, 2) <= estimate <= F(9, 2) for estimate in estimates)


def check_two_phase_and_pair_variance():
    """Enumerate two of four intervals, then audit one missed prompt uniformly."""
    hours = (0, 2, 4, 6)
    predictions = (6, 4, 2, 0)
    residuals = tuple(y - f for y, f in zip(hours, predictions))
    pi = F(1, 2)
    pi_distinct = F(1, 6)
    estimates, follow_up_means, variance_estimates = [], [], []
    for sample in combinations(range(len(hours)), 2):
        total = sum(predictions) + sum(residuals[m] / pi for m in sample)
        estimates.append(total)

        # Every high-workload unit misses its initial prompt.
        replies = [m for m in sample if hours[m] < 4]
        missed = [m for m in sample if hours[m] >= 4]
        initial_total = sum(predictions) + sum(residuals[m] / pi for m in replies)
        if missed:
            eta = F(1, len(missed))
            possible_totals = [initial_total + residuals[m] / (pi * eta) for m in missed]
        else:
            possible_totals = [initial_total]
        assert mean(possible_totals) == total
        follow_up_means.append(mean(possible_totals))

        variance_estimate = F(0)
        for m, k in product(sample, repeat=2):
            joint = pi if m == k else pi_distinct
            coefficient = (joint - pi ** 2) / joint
            variance_estimate += coefficient * residuals[m] * residuals[k] / pi ** 2
        variance_estimates.append(variance_estimate)

    target = sum(hours)
    assert mean(follow_up_means) == mean(estimates) == target
    design_variance = mean([(h - target) ** 2 for h in estimates])
    assert mean(variance_estimates) == design_variance == F(320, 3)
    forecast = F(11)
    corrected_errors = [
        (forecast - h) ** 2 - v for h, v in zip(estimates, variance_estimates)
    ]
    assert mean(corrected_errors) == (forecast - target) ** 2


def check_labour_forecast_and_ratio():
    # The task-class mean includes a genuinely zero-hour task.
    assert mean((0, 2, 4)) == 2
    hours = (1, 3)
    output = F(6)
    estimates = [2 * h for h in hours]
    assert mean(estimates) == sum(hours)
    assert output / sum(hours) == F(3, 2)
    assert mean([output / h for h in estimates]) == 2


def check_binning_bounds():
    checks = 0
    step = F(1, 4)
    for allocations in product((0, 1), repeat=8):
        bins = [(F(i), F(i + 1), step * sum(allocations[4 * i:4 * i + 4]))
                for i in range(2)]
        for cutoff_step in range(9):
            cutoff = step * cutoff_step
            true_labour = step * sum(allocations[:cutoff_step])
            lower = sum((mass for _, end, mass in bins if end <= cutoff), F(0))
            upper = sum((mass for start, _, mass in bins if start < cutoff), F(0))
            crossing = sum((mass for start, end, mass in bins
                            if start < cutoff < end), F(0))
            assert lower <= true_labour <= upper
            assert upper - lower == crossing
            if cutoff_step in (0, 4, 8):
                assert lower == true_labour == upper
            if lower > 0:
                for output_lower, output, output_upper in ((F(1), F(2), F(3)),
                                                          (F(0), F(0), F(0))):
                    assert output_lower / upper <= output / true_labour <= output_upper / lower
            checks += 1
    return checks


if __name__ == "__main__":
    check_sampling_noise()
    check_two_phase_and_pair_variance()
    check_labour_forecast_and_ratio()
    count = check_binning_bounds()
    print(json.dumps({"sampling_example": "PASS", "two_phase_and_pair_variance": "PASS",
                      "labour_forecast_and_ratio": "PASS", "exact_bin_cases": count,
                      "scope": "Finite manufactured checks; analytical proofs are in the paper."}))
