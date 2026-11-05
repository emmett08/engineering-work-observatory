#!/usr/bin/env python3
"""Check the quantities actually plotted in the paper's explanatory figures.

Run from any directory with Python 3. No third-party packages are required.
These checks concern the illustrative examples and their time order.
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

    def test_human_work_and_execution_follow_the_same_timeline(self):
        source = (ROOT / "figures/human-and-workflow-time.tex").read_text()
        times = {
            name: Fraction(value)
            for name, value in re.findall(r"\\def\\HT(\w+)\{([\d.]+)\}", source)
        }
        self.assertEqual(len(times), 12)
        self.assertEqual((times["WindowStart"], times["WindowEnd"]), (0, 60))
        human = ("PromptA", "PromptB", "ReviewB")
        runs = ("RunA", "RunB")
        for name in human + runs:
            self.assertLess(times[name + "Start"], times[name + "End"])
            self.assertLessEqual(times["WindowStart"], times[name + "Start"])
            self.assertLessEqual(times[name + "End"], times["WindowEnd"])
        for before, after in zip(human, human[1:]):
            self.assertLessEqual(times[before + "End"], times[after + "Start"])

        # A run starts when the corresponding prompt is submitted.
        for run, prompt in (("RunA", "PromptA"), ("RunB", "PromptB")):
            self.assertEqual(times[run + "Start"], times[prompt + "End"])
        # B's completed result is available before its review begins.
        self.assertGreaterEqual(times["ReviewBStart"], times["RunBEnd"])

        # Check the endpoints used by the plotted rectangles, not only comments.
        for name, bottom, top in (
            ("PromptA", 34, 42), ("PromptB", 34, 42), ("ReviewB", 34, 42),
            ("RunA", 22, 28), ("RunB", 11, 17),
        ):
            self.assertIn(
                "({40+1.75*\\HT" + name + "Start}," + str(bottom) + ")", source
            )
            self.assertIn(
                "rectangle ({40+1.75*\\HT" + name + "End}," + str(top) + ")",
                source,
            )
        labour_minutes = sum(times[n + "End"] - times[n + "Start"] for n in human)
        run_minutes = sum(times[n + "End"] - times[n + "Start"] for n in runs)
        overlap_minutes = (
            min(times["RunAEnd"], times["RunBEnd"])
            - max(times["RunAStart"], times["RunBStart"])
        )
        self.assertEqual(labour_minutes, 40)
        self.assertEqual(run_minutes, 70)
        self.assertEqual(overlap_minutes, 20)
        self.assertEqual(labour_minutes / 60, Fraction(2, 3))
        self.assertEqual(run_minutes / 60, Fraction(7, 6))
        caption = source.split(r"\caption{", 1)[1]
        self.assertIn("$2/3$", caption)
        self.assertIn("$7/6$", caption)
        prose = (ROOT / "article/main.tex").read_text()
        self.assertIn("Human labour totals forty minutes", prose)
        self.assertIn("seventy minutes", prose)

    def test_single_panel_figures_have_no_overall_heading(self):
        for name in ("human-and-workflow-time", "task-clocks", "sampling-spread"):
            source = (ROOT / f"figures/{name}.tex").read_text()
            picture = source.split(r"\begin{tikzpicture}", 1)[1].split(
                r"\end{tikzpicture}", 1
            )[0]
            self.assertNotIn(r"\bfseries", picture)
            self.assertNotIn("Same average, different sampling uncertainty", picture)
            self.assertNotIn("Concurrent execution, separate human time", picture)

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
