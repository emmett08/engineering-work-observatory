use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_observatory"))
        .arg("--state")
        .arg(state)
        .args(args)
        .output()
        .unwrap()
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn study_import_freeze_report_and_replay_work_from_the_command_line() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/study.jsonl");
    success(&run(&state, &["import", example.to_str().unwrap()]));
    let repeat = run(&state, &["import", example.to_str().unwrap()]);
    success(&repeat);
    assert!(String::from_utf8_lossy(&repeat.stdout).contains("Imported 0"));
    let cut = root.path().join("cut.json");
    let report = root.path().join("report");
    success(&run(
        &state,
        &["snapshot", "--output", cut.to_str().unwrap()],
    ));
    success(&run(
        &state,
        &[
            "analyse",
            cut.to_str().unwrap(),
            "--output",
            report.to_str().unwrap(),
        ],
    ));
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(report.join("report.json")).unwrap()).unwrap();
    assert_eq!(
        value["analysis"]["cohorts"][0]["total_retained_output"],
        0.75
    );
    assert_eq!(value["analysis"]["cohorts"][0]["total_person_hours"], 5.0);
    assert_eq!(
        value["analysis"]["cohorts"][0]["retained_output_per_person_hour"],
        0.15
    );
    let html = fs::read_to_string(report.join("index.html")).unwrap();
    assert!(html.contains("<svg"));
    assert!(html.contains("<table"));
    assert!(
        !run(&state, &["snapshot", "--output", cut.to_str().unwrap()])
            .status
            .success()
    );
    assert!(
        !run(
            &state,
            &[
                "analyse",
                cut.to_str().unwrap(),
                "--output",
                report.to_str().unwrap()
            ]
        )
        .status
        .success()
    );
    success(&run(&state, &["verify"]));
}

#[test]
fn cli_eal_import_uses_a_nonexistent_backup_destination() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let source = root.path().join("source.sqlite3");
    let db = rusqlite::Connection::open(&source).unwrap();
    db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE records(id TEXT PRIMARY KEY,kind TEXT,created_at TEXT,payload TEXT); INSERT INTO records VALUES('observation1','observation','2026-01-01T00:00:00Z','{\"schema\":\"EAL/observation-record/1\",\"status\":\"error\"}');").unwrap();
    success(&run(
        &state,
        &[
            "import-eal",
            source.to_str().unwrap(),
            "--namespace",
            "synthetic",
        ],
    ));
    success(&run(&state, &["verify"]));
    assert_eq!(
        db.query_row("SELECT count(*) FROM records", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn cli_engarg_fixture_retains_missing_tokens_and_synthetic_classification() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    success(&run(
        &state,
        &[
            "import-engarg",
            examples.join("engarg-cursor-report.json").to_str().unwrap(),
            "--manifest",
            examples.join("import-manifest.json").to_str().unwrap(),
        ],
    ));
    let cut = root.path().join("cut.json");
    let report = root.path().join("report");
    success(&run(
        &state,
        &["snapshot", "--output", cut.to_str().unwrap()],
    ));
    success(&run(
        &state,
        &[
            "analyse",
            cut.to_str().unwrap(),
            "--output",
            report.to_str().unwrap(),
        ],
    ));
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(report.join("report.json")).unwrap()).unwrap();
    assert_eq!(value["sources"]["logical_product_calls_observed"], 0);
    assert_eq!(value["sources"]["synthetic_logical_calls"], 1);
}

#[test]
fn oversized_input_is_rejected_before_retaining_objects() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let input = root.path().join("oversized.json");
    fs::File::create(&input)
        .unwrap()
        .set_len(128 * 1024 * 1024 + 1)
        .unwrap();
    let output = run(
        &state,
        &[
            "import-engarg",
            input.to_str().unwrap(),
            "--manifest",
            "does-not-exist.json",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("128 MiB"));
    assert_eq!(fs::read_dir(state.join("objects")).unwrap().count(), 0);
}
