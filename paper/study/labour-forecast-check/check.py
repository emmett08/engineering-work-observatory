"""Replay the paper's task-labour reference forecast against known synthetic hours."""

import csv
import sys
from dataclasses import dataclass
from fractions import Fraction
from statistics import mean


@dataclass(frozen=True)
class Task:
    day: int
    kind: str
    hours: int
    reporting_delay: int = 0

    @property
    def hours_available_on(self):
        return self.day + 2 + self.reporting_delay


# One new requirement per intake; hours cover the two days after entry.
# The verified zero on day 4 is an abandoned fix, reported two days late.
TASKS = (
    Task(0, "fix", 2), Task(1, "fix", 4),
    Task(2, "incident", 8), Task(3, "incident", 10),
    Task(4, "fix", 0, reporting_delay=2),
    Task(6, "fix", 2), Task(7, "incident", 9), Task(8, "migration", 12),
    Task(9, "fix", 2), Task(10, "fix", 6), Task(11, "incident", 9),
    Task(12, "fix", 6), Task(13, "incident", 9),
)


def historical_mean(hours):
    if len(hours) < 2:
        return None
    return Fraction(sum(hours), len(hours))


def issue_forecasts(day, kind, tasks):
    available = [t for t in tasks if t.hours_available_on <= day]
    same_kind = [t.hours for t in available if t.kind == kind]
    pooled = [t.hours for t in available]
    return len(same_kind), historical_mean(same_kind), historical_mean(pooled)


def main():
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(("intake_day", "kind", "eligible_in_class", "class_forecast_h",
                     "pooled_forecast_h", "true_h"))
    class_forecasts = []
    pooled_forecasts = []
    class_errors = []
    pooled_errors_on_common_tasks = []
    for task in TASKS:
        if task.day < 6:
            continue
        count, class_hours, pooled_hours = issue_forecasts(task.day, task.kind, TASKS)
        writer.writerow((task.day, task.kind, count, class_hours, pooled_hours, task.hours))
        class_forecasts.append(class_hours)
        pooled_forecasts.append(pooled_hours)
        if class_hours is not None and pooled_hours is not None:
            class_errors.append(abs(class_hours - task.hours))
            pooled_errors_on_common_tasks.append(abs(pooled_hours - task.hours))

    # Hand-calculated intake forecasts catch training on future or unreleased hours
    # and treating a measured zero as a missing value.
    assert class_forecasts == [3, 9, None, 2, 2, 9, Fraction(8, 3), 9]
    assert pooled_forecasts == [
        6, 6, Fraction(13, 3), 5, Fraction(47, 8), Fraction(49, 9),
        Fraction(11, 2), Fraction(64, 11),
    ]
    class_mae = mean(class_errors)
    pooled_mae = mean(pooled_errors_on_common_tasks)
    assert (class_mae, pooled_mae) == (Fraction(25, 21), Fraction(13751, 5544))
    for name, forecasts in (("class", class_forecasts), ("pooled", pooled_forecasts)):
        issued = sum(value is not None for value in forecasts)
        print(f"{name} forecasts issued: {issued}/{len(forecasts)}")
    print(f"Mean absolute error on {len(class_errors)} common tasks (person-hours): "
          f"class {float(class_mae):.2f}, pooled {float(pooled_mae):.2f}")


if __name__ == "__main__":
    main()
