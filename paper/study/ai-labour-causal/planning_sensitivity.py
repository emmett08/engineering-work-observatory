"""Vary one assumption at a time in the central team-level planning scenario."""

import argparse
import csv
import io
import json
from pathlib import Path

from planning import DESIGNS, SCENARIOS, compute, simulate


ROOT = Path(__file__).resolve().parent
PARAMETERS = ("rho", "labour_log_sd", "quality_logit_sd", "noise_l", "noise_q")


def comparable_result(result):
    """Keep the reported numbers and producer identity, allowing Python patch updates."""
    versions = result["versions"].copy()
    versions["python"] = ".".join(versions["python"].split(".")[:2])
    return {**result, "versions": versions}


def sensitivity_table(retained):
    scenarios = {scenario["id"]: scenario for scenario in SCENARIOS}
    central = scenarios["central-benefit"]
    adverse = scenarios["adverse-benefit"]
    central_seed_index = SCENARIOS.index(central)
    reference = {
        row["design_id"]: row
        for row in retained["results"]
        if row["scenario_id"] == central["id"]
    }

    output = io.StringIO()
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(("teams", "parameter", "central_value", "tested_value", "experiments",
                     "labour_pass", "retained_pass", "both_pass", "wilson_95_low", "wilson_95_high"))
    for design in DESIGNS:
        # The seed stays fixed within each design. Each comparison changes only
        # the named parameter and shares the latent draws, treatment coins and noise draws.
        for parameter in ("central", *PARAMETERS):
            scenario = central.copy()
            if parameter != "central":
                scenario[parameter] = adverse[parameter]
            row = simulate(design, scenario, central_seed_index)
            if parameter == "central" and row != reference[design]:
                raise ValueError(f"Central scenario does not reproduce for {design}")
            both = row["joint_success"]
            writer.writerow((row["teams"], parameter, central.get(parameter, ""),
                             scenario.get(parameter, ""), both["denominator"],
                             row["labour_pass"]["count"], row["quality_pass"]["count"],
                             both["count"], *both["wilson_95"]))
    return output.getvalue()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Reproduce the original and supplemental results")
    args = parser.parse_args()
    table = ROOT / "planning-sensitivity.csv"
    with (ROOT / "planning-results.json").open() as source:
        retained = json.load(source)
    result = sensitivity_table(retained)
    if args.check:
        if comparable_result(compute()) != comparable_result(retained):
            parser.exit(1, "Original planning results, inputs or producer differ from the retained analysis.\n")
        if table.read_text() != result:
            parser.exit(1, "Planning sensitivity results differ from the retained table.\n")
    else:
        table.write_text(result)


if __name__ == "__main__":
    main()
