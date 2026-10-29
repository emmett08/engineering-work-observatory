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
    def test_summary_replaces_output_aliases_without_mutating_retained_targets(self):
        for link_kind in ("symlink", "hardlink"):
            with self.subTest(link_kind=link_kind), tempfile.TemporaryDirectory() as directory:
                root = pathlib.Path(directory)
                retained = root / "results"
                retained.mkdir()
                output = root / "new-collection"
                output.mkdir()
                shutil.copyfile(collector.ROOT / "results/raw.csv", output / "raw.csv")
                for name in ("summary.csv", "table.tex"):
                    target = retained / name
                    target.write_text(f"immutable {name}\n")
                    if link_kind == "symlink":
                        (output / name).symlink_to(target)
                    else:
                        (output / name).hardlink_to(target)
                with patch.object(summary, "ROOT", root):
                    summary.write_summary(output)
                for name in ("summary.csv", "table.tex"):
                    with self.subTest(output=name):
                        self.assertEqual((retained / name).read_text(), f"immutable {name}\n")
                        self.assertFalse((output / name).is_symlink())
                        self.assertNotEqual((output / name).read_text(), f"immutable {name}\n")

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


class ProtocolValidationTests(unittest.TestCase):
    def setUp(self):
        with (collector.ROOT / "results/raw.csv").open() as stream:
            self.raw = list(csv.DictReader(stream))

    def test_standalone_summary_rejects_each_invalid_protocol_or_timing_field(self):
        invalid = {
            "first": "invalid", "accepted_count": "1", "labour_count": "1",
            "delta_hours": "0", "scan_ns": "-1", "index_total_ns": "-1",
            "index_build_ns": "-1", "index_query_ns": "-1", "index_drop_ns": "-1",
            "checksum": "-1",
        }
        for field, value in invalid.items():
            with self.subTest(field=field):
                rows = [dict(row) for row in self.raw]
                rows[0][field] = value
                with self.assertRaises(ValueError):
                    summary.summarise(rows)
        for field in ("scan_ns", "index_total_ns"):
            with self.subTest(zero_total=field):
                rows = [dict(row) for row in self.raw]
                rows[0][field] = "0"
                with self.assertRaises(ValueError):
                    summary.summarise(rows)

    def test_summary_requires_complete_keys_and_declared_order(self):
        for rows in (self.raw[:-1], self.raw + [self.raw[0]], self.raw[1:] + [self.raw[1]]):
            with self.subTest(keys=len(rows)), self.assertRaises(ValueError):
                summary.summarise(rows)
        rows = [dict(row) for row in self.raw]
        rows[0]["first"] = "index" if rows[0]["first"] == "scan" else "scan"
        with self.assertRaises(ValueError):
            summary.summarise(rows)

    def test_collector_rejects_a_valid_row_for_the_wrong_requested_key(self):
        row = self.raw[0]
        line = ",".join(row[name] for name in collector.HEADER)
        with self.assertRaises(ValueError):
            collector.parse_row(line, 41, int(row["q"]), int(row["rep"]), row["first"])

    def test_collector_and_summary_share_row_checks_and_bound_the_helper_source(self):
        original = self.raw[0]
        for field, value in (("first", "invalid"), ("delta_hours", "0"),
                             ("scan_ns", "0"), ("index_build_ns", "-1"),
                             ("checksum", str(2**128)), ("scan_ns", "not-a-number")):
            with self.subTest(field=field, value=value):
                row = dict(original)
                row[field] = value
                line = ",".join(row[name] for name in collector.HEADER)
                with self.assertRaises(ValueError):
                    collector.parse_row(line, int(original["seed"]), int(original["q"]),
                                        int(original["rep"]), original["first"])
                with self.assertRaises(ValueError):
                    summary.summarise([row, *self.raw[1:]])
        self.assertEqual(collector.collector_hashes()["collection_protocol.py"],
                         collector.digest(collector.ROOT / "collection_protocol.py"))

    def test_invalid_raw_does_not_replace_existing_summaries(self):
        with tempfile.TemporaryDirectory() as directory:
            output = pathlib.Path(directory)
            self.raw[0]["scan_ns"] = "-1"
            with (output / "raw.csv").open("w", newline="") as stream:
                writer = csv.DictWriter(stream, fieldnames=collector.HEADER)
                writer.writeheader()
                writer.writerows(self.raw)
            for name in ("summary.csv", "table.tex"):
                (output / name).write_text("existing summary\n")
            with self.assertRaises(ValueError):
                summary.write_summary(output)
            for name in ("summary.csv", "table.tex"):
                self.assertEqual((output / name).read_text(), "existing summary\n")


if __name__ == "__main__":
    unittest.main()
