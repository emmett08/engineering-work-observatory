"""Exact finite checks of the sampling-noise and continuous-bin examples.

These checks supplement the manuscript's analytical arguments; they do not
validate observed allocations or extend the historical Lean proof inventory.
"""
from fractions import Fraction as F
from itertools import product
import json


def check_sampling_noise():
    allocations = (1, 1, 1, 1, 0, 0, 0, 0)
    labour = F(sum(allocations))
    estimates = [F(8 * value) for value in allocations]
    mean = lambda values: sum(values, F(0)) / len(values)
    variance = mean([(estimate - labour) ** 2 for estimate in estimates])
    assert mean(estimates) == labour == 4
    for forecast in map(F, (0, 2, 4, 8)):
        assert mean([(estimate - forecast) ** 2 for estimate in estimates]) == (
            (labour - forecast) ** 2 + variance
        )
        assert mean([estimate - forecast for estimate in estimates]) == labour - forecast
    assert mean([abs(estimate - labour) for estimate in estimates]) == 4
    assert F(7, 2) <= labour <= F(9, 2)
    assert not any(F(7, 2) <= estimate <= F(9, 2) for estimate in estimates)


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
    count = check_binning_bounds()
    print(json.dumps({"sampling_example": "PASS", "exact_bin_cases": count,
                      "scope": "Finite manufactured checks; analytical proofs are in the paper."}))
