use chrono::{Duration as ChronoDuration, Utc};
use observatory::forecast::archive::IssuedForecast;
use observatory::store::Store;
use serde_json::{Value, json};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct Server {
    base: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Server {
    fn new(incomplete: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}/", listener.local_addr().unwrap());
        let now = Utc::now();
        let repository = json!({"id":42,"full_name":"acme/demo","created_at":(now-ChronoDuration::days(10)).to_rfc3339()});
        let pulls = json!([{"id":101,"number":1,"state":"closed","created_at":(now-ChronoDuration::days(3)).to_rfc3339(),"closed_at":(now-ChronoDuration::days(2)).to_rfc3339(),"merged_at":(now-ChronoDuration::days(2)).to_rfc3339(),"draft":false,"requested_reviewers":[],"requested_teams":[]}]);
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let observed = requests.clone();
        let stopped = stop.clone();
        let endpoint = base.clone();
        let handle = thread::spawn(move || {
            while !stopped.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let mut bytes = Vec::new();
                        let mut chunk = [0; 2048];
                        while !bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                            let count = stream.read(&mut chunk).unwrap();
                            assert!(count > 0 && bytes.len() < 65_536);
                            bytes.extend_from_slice(&chunk[..count]);
                        }
                        let request = String::from_utf8(bytes).unwrap();
                        let target = request
                            .lines()
                            .next()
                            .unwrap()
                            .split_whitespace()
                            .nth(1)
                            .unwrap();
                        observed.lock().unwrap().push(request.clone());
                        let (status, body, link) = if target == "/repos/acme/demo" {
                            (200, repository.clone(), String::new())
                        } else if target == "/repos/acme/demo/pulls?page=2" && incomplete {
                            (403, json!({"message":"Forbidden page"}), String::new())
                        } else {
                            assert_eq!(
                                target,
                                "/repos/acme/demo/pulls?state=all&sort=created&direction=asc&per_page=100"
                            );
                            let link = if incomplete {
                                format!(
                                    "Link: <{endpoint}repos/acme/demo/pulls?page=2>; rel=\"next\"\r\n"
                                )
                            } else {
                                String::new()
                            };
                            (200, pulls.clone(), link)
                        };
                        let body = serde_json::to_vec(&body).unwrap();
                        write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{link}\r\n", body.len()).unwrap();
                        stream.write_all(&body).unwrap();
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1))
                    }
                    Err(error) => panic!("accept failed: {error}"),
                }
            }
        });
        Self {
            base,
            requests,
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.handle.take().unwrap().join().unwrap();
    }
}

fn run(state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_observatory"))
        .arg("--state")
        .arg(state)
        .args(args)
        .env_remove("OBSERVATORY_TEST_UNUSED_TOKEN")
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

fn json_files(path: &Path) -> Vec<PathBuf> {
    if !path.exists() {
        return Vec::new();
    }
    let mut paths = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
fn forecast_command_collects_archives_exports_replays_and_scores_prior_receipts() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let export = root.path().join("issued.json");
    let server = Server::new(false);
    let args = [
        "forecast-team",
        "--repository",
        "acme/demo",
        "--api-base",
        &server.base,
        "--token-env",
        "OBSERVATORY_TEST_UNUSED_TOKEN",
    ];
    let mut first_args = args.to_vec();
    first_args.extend(["--output", export.to_str().unwrap()]);
    let output = run(&state, &first_args);
    success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["forecast"]["status"], "available");
    assert_eq!(report["forecast"]["period"], "day");
    assert_eq!(
        report["forecast"]["validation_status"],
        "unvalidated_operational_reference"
    );
    assert_eq!(report["current_pressure"]["open_pull_requests"], 0);
    assert!(
        report["previous_forecast_scores"]["entries"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(server.requests.lock().unwrap().len(), 2);
    let archives = json_files(&state.join("forecasts"));
    assert_eq!(archives.len(), 1);
    assert_eq!(json_files(&state.join("forecast-scores")).len(), 1);
    let archived_bytes = fs::read(&archives[0]).unwrap();
    assert_eq!(archived_bytes, fs::read(&export).unwrap());
    let original: IssuedForecast = serde_json::from_slice(&archived_bytes).unwrap();
    original.verify().unwrap();
    assert!(original.forecast.target_start >= original.created_at);
    assert!(
        original
            .forecast
            .training_bins
            .iter()
            .all(|bin| bin.end <= original.history.observed_cut)
    );

    let snapshot_path = root.path().join("snapshot.json");
    fs::write(
        &snapshot_path,
        serde_json::to_vec(&original.snapshot).unwrap(),
    )
    .unwrap();
    let replay_path = root.path().join("replay.json");
    let replay = run(
        &state,
        &[
            "forecast-snapshot",
            snapshot_path.to_str().unwrap(),
            "--repository",
            "acme/demo",
            "--output",
            replay_path.to_str().unwrap(),
        ],
    );
    success(&replay);
    let replayed: Value = serde_json::from_slice(&replay.stdout).unwrap();
    assert_eq!(replayed["schema"], "observatory.team-forecast-replay/1");
    assert_eq!(replayed["forecast"], report["forecast"]);
    assert_eq!(
        replayed,
        serde_json::from_slice::<Value>(&fs::read(&replay_path).unwrap()).unwrap()
    );
    assert_eq!(
        server.requests.lock().unwrap().len(),
        2,
        "offline replay made an HTTP request"
    );

    let repeated = run(&state, &args);
    success(&repeated);
    let repeated: Value = serde_json::from_slice(&repeated.stdout).unwrap();
    let previous = repeated["previous_forecast_scores"]["entries"]
        .as_array()
        .unwrap();
    assert_eq!(previous.len(), 1);
    assert!(previous[0]["error"].is_null());
    assert_eq!(previous[0]["score"]["status"], "pending");
    assert!(previous[0]["score"]["outcome"].is_null());
    assert_eq!(json_files(&state.join("forecasts")).len(), 2);
    assert_eq!(json_files(&state.join("forecast-scores")).len(), 2);
    assert_eq!(
        fs::read(&archives[0]).unwrap(),
        archived_bytes,
        "automatic scoring rewrote an old receipt"
    );
    assert_eq!(server.requests.lock().unwrap().len(), 4);
    success(&run(&state, &["verify"]));
}

#[test]
fn short_history_weekly_forecast_remains_unavailable_in_command_output() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let server = Server::new(false);
    let output = run(
        &state,
        &[
            "forecast-team",
            "--repository",
            "acme/demo",
            "--horizon",
            "week",
            "--api-base",
            &server.base,
            "--token-env",
            "OBSERVATORY_TEST_UNUSED_TOKEN",
        ],
    );
    success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["forecast"]["status"], "unavailable");
    assert!(report["forecast"]["distribution"].is_null());
    assert_eq!(report["forecast"]["required_training_bins"], 4);
    assert!(
        report["forecast"]["available_complete_bins"]
            .as_u64()
            .unwrap()
            < 4
    );
    assert_eq!(json_files(&state.join("forecasts")).len(), 1);
    success(&run(&state, &["verify"]));
}

#[test]
fn failed_inventory_retains_evidence_but_cannot_issue_forecast_or_export() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let export = root.path().join("issued.json");
    let server = Server::new(true);
    let output = run(
        &state,
        &[
            "forecast-team",
            "--repository",
            "acme/demo",
            "--api-base",
            &server.base,
            "--token-env",
            "OBSERVATORY_TEST_UNUSED_TOKEN",
            "--output",
            export.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("incomplete coverage"));
    assert!(!export.exists());
    assert!(json_files(&state.join("forecasts")).is_empty());
    assert!(json_files(&state.join("forecast-scores")).is_empty());
    assert_eq!(server.requests.lock().unwrap().len(), 3);
    let store = Store::open(&state).unwrap();
    assert!(store.verify().unwrap() > 0);
    let retained = store.retained_records(None).unwrap();
    let pull = retained
        .iter()
        .find(|record| record.record.data["source"] == "github.pull_request")
        .unwrap();
    let manifest: Value =
        serde_json::from_slice(&fs::read(store.object_path(&pull.raw_sha256).unwrap()).unwrap())
            .unwrap();
    assert!(
        manifest["responses"]
            .as_array()
            .unwrap()
            .iter()
            .any(|response| response["status"] == 403)
    );
    success(&run(&state, &["verify"]));
}

#[test]
fn existing_export_is_rejected_before_network_or_forecast_archive_changes() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let export = root.path().join("existing.json");
    fs::write(&export, b"keep this file").unwrap();
    let server = Server::new(false);
    let output = run(
        &state,
        &[
            "forecast-team",
            "--repository",
            "acme/demo",
            "--api-base",
            &server.base,
            "--token-env",
            "OBSERVATORY_TEST_UNUSED_TOKEN",
            "--output",
            export.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("refusing to overwrite"));
    assert_eq!(fs::read(&export).unwrap(), b"keep this file");
    assert!(server.requests.lock().unwrap().is_empty());
    assert!(json_files(&state.join("forecasts")).is_empty());
}
