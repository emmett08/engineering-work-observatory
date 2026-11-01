//! Exercise the Python producer and Rust import/store boundary together.
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn recorder(directory: &Path, command: &str, args: &[&str]) -> Output {
    Command::new("python3")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("instrumentation/receipts.py"))
        .arg(command)
        .arg("--directory")
        .arg(directory)
        .args(args)
        .env("GITHUB_REPOSITORY_ID", "123")
        .env("GITHUB_REPOSITORY", "example/project")
        .env(
            "GITHUB_WORKFLOW_REF",
            "example/project/.github/workflows/test.yml@refs/heads/main",
        )
        .env("GITHUB_WORKFLOW_SHA", "a".repeat(40))
        .env("GITHUB_RUN_ID", "456")
        .env("GITHUB_RUN_ATTEMPT", "2")
        .env("GITHUB_JOB", "test")
        .env("GITHUB_ACTOR", "credential-owner")
        .env("GITHUB_TRIGGERING_ACTOR", "rerun-owner")
        .env("GITHUB_EVENT_NAME", "workflow_dispatch")
        .env("UNRESTRICTED_ENV_SECRET", "never-retain-this-secret")
        .output()
        .unwrap()
}

fn observatory(state: &Path, args: &[&str]) -> Output {
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
fn real_wrapper_bundle_import_is_idempotent_and_keeps_account_controller_separate() {
    let root = tempfile::tempdir().unwrap();
    let recording = root.path().join("recording");
    let state = root.path().join("state");
    success(&recorder(
        &recording,
        "init",
        &[
            "--controller",
            "assistant",
            "--controller-provenance",
            "session-log-42",
            "--session-id",
            "declared-session",
            "--job-name",
            "Test (linux)",
            "--matrix",
            r#"{"os":"linux"}"#,
        ],
    ));
    let execution = recorder(
        &recording,
        "run",
        &[
            "--tool",
            "python",
            "--purpose",
            "development",
            "--logical-call-id",
            "check-17",
            "--check-name",
            "example-check",
            "--",
            "python3",
            "-c",
            "print('tool output is not evidence metadata'); raise SystemExit(7)",
        ],
    );
    assert_eq!(
        execution.status.code(),
        Some(7),
        "{}",
        String::from_utf8_lossy(&execution.stderr)
    );
    let published = recorder(&recording, "publish", &[]);
    success(&published);
    let publication: Value = serde_json::from_slice(&published.stdout).unwrap();
    let path = publication["path"].as_str().unwrap();
    let bytes = fs::read(path).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(!text.contains("never-retain-this-secret"));
    assert!(!text.contains("tool output is not evidence metadata"));
    success(&observatory(&state, &["import-executions", path]));
    let repeat = observatory(&state, &["import-executions", path]);
    success(&repeat);
    assert!(String::from_utf8_lossy(&repeat.stdout).contains("Imported 0"));
    success(&observatory(&state, &["verify"]));
    let store = observatory::store::Store::open(&state).unwrap();
    let records = store.records(None).unwrap();
    assert_eq!(records.len(), 3); // Bundle plus two immutable phases.
    assert!(records.iter().all(|r| r.kind == "source_record"));
    let end = &records
        .iter()
        .find(|r| r.id.ends_with("/completion"))
        .unwrap()
        .data["payload"];
    assert_eq!(end["platform"]["actor"], "credential-owner");
    assert_eq!(end["platform"]["triggering_actor"], "rerun-owner");
    assert_eq!(end["platform"]["run_attempt"], 2);
    assert_eq!(end["declared"]["controller"], "assistant");
    assert_eq!(end["declared"]["session_id"], "declared-session");
    assert_eq!(end["declared"]["logical_call_id"], "check-17");
    assert_eq!(end["result"]["status"], "failure");
    assert_eq!(end["result"]["exit_code"], 7);
    assert!(end["result"]["input_tokens"].is_null());
    assert!(end["result"]["charge_amount"].is_null());

    let mut tampered: Value = serde_json::from_slice(&bytes).unwrap();
    tampered["receipts"][0]["sha256"] = Value::String("0".repeat(64));
    let bad = root.path().join("bad.json");
    fs::write(&bad, serde_json::to_vec(&tampered).unwrap()).unwrap();
    assert!(
        !observatory(&state, &["import-executions", bad.to_str().unwrap()])
            .status
            .success()
    );
    assert_eq!(store.verify().unwrap(), 3);
}
