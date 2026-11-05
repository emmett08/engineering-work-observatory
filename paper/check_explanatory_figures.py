#!/usr/bin/env python3
"""Check the quantities actually plotted in the paper's explanatory figures.

Run from any directory with Python 3. No third-party packages are required.
These checks concern the constructed examples, not workplace measurement.
"""
from fractions import Fraction
from itertools import combinations
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parent


class ExplanatoryFigureChecks(unittest.TestCase):
    def test_sampling_points_and_variances(self):
        source = (ROOT / "figures/sampling-spread.tex").read_text()
        lists = re.findall(
            r"\\foreach \\estimate/\\offset in \{([^}]+)\}", source
        )
        self.assertEqual(len(lists), 3)
        observed = [
            sorted(Fraction(pair.split("/")[0]) for pair in row.split(","))
            for row in lists
        ]
        labour = (0, 2, 4, 6)
        predictions = ((0, 0, 0, 0), (1, 3, 3, 5), (6, 4, 2, 0))
        variances = (Fraction(80, 3), Fraction(16, 3), Fraction(320, 3))
        for plotted, prediction, variance in zip(observed, predictions, variances):
            estimates = [
                Fraction(sum(prediction))
                + sum(Fraction(labour[m] - prediction[m], 1) / Fraction(1, 2)
                      for m in sample)
                for sample in combinations(range(4), 2)
            ]
            self.assertEqual(plotted, sorted(estimates))
            self.assertEqual(sum(estimates) / 6, 12)
            self.assertEqual(sum((value - 12) ** 2 for value in estimates) / 6,
                             variance)
        self.assertLess(observed[2][0], 0)

    def test_human_time_uses_non_overlapping_blocks(self):
        source = (ROOT / "figures/human-and-workflow-time.tex").read_text()
        blocks = re.findall(
            r"\\fill\[black!\d+\] \((\d+),34\) rectangle \((\d+),42\);",
            source,
        )
        self.assertEqual(len(blocks), 2)
        scale = Fraction(8, 5)  # millimetres per elapsed minute in the figure
        intervals = [(Fraction(left) - 43, Fraction(right) - 43)
                     for left, right in blocks]
        self.assertEqual([(r - l) / scale for l, r in intervals], [10, 20])
        self.assertLessEqual(intervals[0][1], intervals[1][0])
        self.assertEqual(sum((r - l) / scale for l, r in intervals) / 60,
                         Fraction(1, 2))
        self.assertEqual(Fraction(139 - 43) / scale, 60)

    def test_clock_endpoints(self):
        source = (ROOT / "figures/task-clocks.tex").read_text()
        follow_up = re.search(
            r"\\draw\[<->,line width=0.7pt\] \(([\d.]+),33\) -- \(([\d.]+),33\);",
            source,
        )
        labour = re.search(
            r"\\draw\[line width=1pt\] \(([\d.]+),17\) -- \(([\d.]+),17\);",
            source,
        )
        self.assertIsNotNone(follow_up)
        self.assertIsNotNone(labour)
        to_day = lambda coordinate: (Fraction(coordinate) - 42) / Fraction(17, 5)
        self.assertEqual(tuple(to_day(x) for x in follow_up.groups()), (3, 17))
        self.assertEqual(tuple(to_day(x) for x in labour.groups()), (0, 28))
        self.assertEqual(to_day("89.6"), 14)
        self.assertEqual(3 + 14, 17)
        self.assertEqual(14 + 14, 28)


if __name__ == "__main__":
    unittest.main(verbosity=2)
