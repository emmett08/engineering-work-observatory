//! The standalone estimator command preserves design assumptions and signed results.
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn run(state: &Path, input: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_observatory"))
        .arg("--state")
        .arg(state)
        .arg("estimate-labour")
        .arg(input)
        .output()
        .unwrap()
}

fn input_value() -> Value {
    serde_json::from_str(include_str!("../examples/model-assisted-labour.json")).unwrap()
}

#[test]
fn estimator_example_runs_offline_without_creating_an_evidence_store() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("unused-state");
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/model-assisted-labour.json");
    let output = run(&state, &input);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["prediction_total_person_hours"], 4.0);
    assert_eq!(result["ht_total_person_hours"], 6.0);
    assert_eq!(result["difference_total_person_hours"], 5.5);
    assert_eq!(result["variance"]["status"], "estimated_srs");
    assert_eq!(
        result["variance"]["difference_variance_person_hours_squared"],
        0.125
    );
    assert_eq!(result["variance"]["ht_variance_person_hours_squared"], 2.0);
    assert!(!state.exists());
}

#[test]
fn estimator_command_keeps_a_negative_design_estimate() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("unused-state");
    let input = root.path().join("signed.json");
    let mut frame = input_value();
    frame["design"] = json!({"kind":"general"});
    frame["units"] = json!([
        {"id":"a", "predicted_person_hours":1.0, "inclusion_probability":0.5,
         "observation":{"status":"observed", "person_hours":0.0}},
        {"id":"b", "predicted_person_hours":0.0, "inclusion_probability":0.5,
         "observation":{"status":"unselected"}}
    ]);
    fs::write(&input, serde_json::to_vec(&frame).unwrap()).unwrap();
    let output = run(&state, &input);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["difference_total_person_hours"], -1.0);
    assert_eq!(result["ht_total_person_hours"], 0.0);
    assert_eq!(result["variance"]["status"], "unavailable");
    assert!(!state.exists());
}

#[test]
fn estimator_command_rejects_nonresponse_without_manufacturing_a_total() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("unused-state");
    let input = root.path().join("missing.json");
    let mut frame = input_value();
    frame["units"][0]["observation"] =
        json!({"status":"nonresponse", "reason":"No observation received"});
    fs::write(&input, serde_json::to_vec(&frame).unwrap()).unwrap();
    let output = run(&state, &input);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!state.exists());
}

#[test]
fn estimator_command_rejects_an_unknown_design_field() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("unused-state");
    let input = root.path().join("typo.json");
    let mut frame = input_value();
    frame["design"]["sample_szie"] = json!(2);
    fs::write(&input, serde_json::to_vec(&frame).unwrap()).unwrap();
    let output = run(&state, &input);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!state.exists());
}
