//! Exercise the model commands across JSON, CLI and side-effect boundaries.
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn run(command: &str, value: &Value) -> (std::process::Output, bool) {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("unused-state");
    let input = root.path().join("input.json");
    fs::write(&input, serde_json::to_vec(value).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_observatory"))
        .args(["--state", state.to_str().unwrap(), command])
        .arg(input)
        .output()
        .unwrap();
    (output, state.exists())
}

fn fixture(name: &str) -> Value {
    serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn joint_bounds_command_returns_exact_shared_capacity_and_witnesses_offline() {
    let (output, created) = run("bound-labour", &fixture("joint-labour-bounds.json"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!created);
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["schema"], "observatory.labour-bound/1");
    assert_eq!(result["status"], "feasible");
    assert_eq!(result["minimum"]["value"], "1");
    assert_eq!(result["maximum"]["value"], "1");
    assert_eq!(
        result["minimum"]["allocations"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        result["productivity"]["lower_reference_units_per_person_hour"],
        "3"
    );
}

#[test]
fn inconsistent_observations_return_no_numeric_extrema() {
    let mut value = fixture("joint-labour-bounds.json");
    value["observations"][0]["lower_person_hours"] = json!("3");
    value["observations"][0]["upper_person_hours"] = json!("3");
    let (output, created) = run("bound-labour", &value);
    assert!(output.status.success());
    assert!(!created);
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "inconsistent");
    assert!(result["minimum"].is_null() && result["maximum"].is_null());
    assert!(result["productivity"].is_null());
}

#[test]
fn exact_input_errors_do_not_print_approximate_results() {
    let baseline = fixture("joint-labour-bounds.json");
    let mut float = baseline.clone();
    float["observations"][0]["lower_person_hours"] = json!(1.0);
    let mut unknown = baseline.clone();
    unknown["estimated_person_hours"] = json!(1);
    let mut oversized = baseline;
    let target = oversized["target"][0].clone();
    oversized["target"] = json!(vec![target; 129]);
    for value in [float, unknown, oversized] {
        let (output, created) = run("bound-labour", &value);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!created);
    }
}

#[test]
fn estimator_v2_keeps_the_geometry_with_the_statistical_result() {
    let value = fixture("model-assisted-labour-v2.json");
    let (output, created) = run("estimate-labour", &value);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!created);
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["schema"], "observatory.labour-estimate/2");
    assert_eq!(result["frames"], value["frames"]);
    assert_eq!(result["geometry"], value["geometry"]);
    assert_eq!(
        result["estimation"]["schema"],
        "observatory.labour-estimate/1"
    );
}

#[test]
fn estimator_v2_rejects_physical_overlap_before_reporting_a_total() {
    let mut value = fixture("model-assisted-labour-v2.json");
    value["geometry"][1]["person_id"] = value["geometry"][0]["person_id"].clone();
    value["geometry"][1]["start"] = value["geometry"][0]["start"].clone();
    value["geometry"][1]["end"] = value["geometry"][0]["end"].clone();
    let (output, created) = run("estimate-labour", &value);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!created);
}
