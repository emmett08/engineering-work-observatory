import csv
import contextlib
import io
import json
import pathlib
import shutil
import tempfile
import unittest
from unittest.mock import patch
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import collect_benchmarks as collector
import summarise_collection as summary


def artifact(path, package="performance-package", name="horizon-bench", kind="bin", fresh=False):
    return {"reason": "compiler-artifact", "package_id": package,
            "target": {"name": name, "kind": [kind]}, "executable": str(path), "fresh": fresh}


class ArtifactTests(unittest.TestCase):
    def test_configured_target_artifact_wins_over_stale_default_and_other_targets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            stale = root / "target/release/horizon-bench"
            selected = root / "external-target/host-triple/release/horizon-bench"
            messages = [artifact(stale, package="other-package"),
                        artifact(stale, name="other-bin"), artifact(stale, kind="lib"),
                        artifact(selected, fresh=True)]
            self.assertEqual(collector.select_executable(messages, "performance-package"), selected)

    def test_missing_or_ambiguous_executable_is_rejected(self):
        with self.assertRaises(ValueError):
            collector.select_executable([artifact("wrong", name="other-bin")], "performance-package")
        with self.assertRaises(ValueError):
            collector.select_executable([artifact("one"), artifact("two")], "performance-package")
        missing = artifact("unused")
        missing["executable"] = None
        with self.assertRaises(ValueError):
            collector.select_executable([missing], "performance-package")

    def test_build_uses_cargo_artifact_instead_of_a_guessed_target_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            binary = pathlib.Path(directory) / "external-target/triple/release/horizon-bench"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"built executable")
            metadata = {"packages": [{"id": "performance-package",
                                     "manifest_path": str(collector.ROOT / "Cargo.toml")} ]}
            result = type("BuildResult", (), {"stdout": json.dumps(artifact(binary, fresh=True))})()
            with patch.object(collector.subprocess, "check_output", return_value=json.dumps(metadata)), \
                    patch.object(collector.subprocess, "run", return_value=result) as build:
                selected, package, _ = collector.build_executable("cargo")
            self.assertEqual(selected, binary)
            self.assertEqual(package, "performance-package")
            self.assertIn("--message-format=json", build.call_args.args[0])
            self.assertEqual(collector.digest(selected), collector.digest(binary))


class CollectionIsolationTests(unittest.TestCase):
    def test_existing_output_and_retained_results_are_refused_before_build(self):
        retained = collector.ROOT / "results"
        before = collector.digest(retained / "raw.csv")
        with patch.object(collector, "build_executable") as build:
            with self.assertRaises(SystemExit), contextlib.redirect_stderr(io.StringIO()):
                collector.main(["--outputdir", str(retained)])
            build.assert_not_called()
        with self.assertRaises(ValueError):
            collector.prepare_output_directory(retained / "nested-collection")
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(FileExistsError):
                collector.prepare_output_directory(directory)
        self.assertEqual(collector.digest(retained / "raw.csv"), before)

    def test_new_summary_stays_in_the_selected_collection(self):
        retained = collector.ROOT / "results"
        before = {name: collector.digest(retained / name)
                  for name in ("raw.csv", "summary.csv", "table.tex", "environment.json")}
        with tempfile.TemporaryDirectory() as directory:
            output = pathlib.Path(directory) / "new-collection"
            collector.prepare_output_directory(output)
            shutil.copyfile(retained / "raw.csv", output / "raw.csv")
            generated = summary.write_summary(output)
            self.assertEqual(generated[0]["scan_median_ms"], 2.88929)
            self.assertTrue((output / "summary.csv").is_file())
            self.assertTrue((output / "table.tex").is_file())
            with (retained / "summary.csv").open() as stream:
                recorded = list(csv.DictReader(stream))
            for actual, expected in zip(generated, recorded):
                for key, value in actual.items():
                    self.assertEqual(float(value), float(expected[key]))
        self.assertEqual(before, {name: collector.digest(retained / name) for name in before})
        with self.assertRaises(ValueError):
            summary.write_summary(retained)


if __name__ == "__main__":
    unittest.main()
