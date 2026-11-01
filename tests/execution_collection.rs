use observatory::{
    collectors::{Collection, collect_github_executions},
    ingestion::{CollectionRequirements, persist_collection},
    store::{Store, digest},
};
use serde_json::{Value, json};
use std::{
    io::{Cursor, Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

struct Response {
    status: u16,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
}
impl Response {
    fn json(value: Value) -> Self {
        Self {
            status: 200,
            body: serde_json::to_vec(&value).unwrap(),
            headers: vec![],
        }
    }
    fn bytes(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            body,
            headers: vec![],
        }
    }
}
struct Server {
    base: String,
    seen: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}
impl Server {
    fn start(handler: impl Fn(&str, &str) -> Response + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}/", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(vec![]));
        let stop = Arc::new(AtomicBool::new(false));
        let requests = seen.clone();
        let stopping = stop.clone();
        let server_base = base.clone();
        let handle = thread::spawn(move || {
            while !stopping.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let mut bytes = vec![];
                        let mut buffer = [0; 1024];
                        while !bytes.windows(4).any(|v| v == b"\r\n\r\n") && bytes.len() < 65536 {
                            let n = stream.read(&mut buffer).unwrap();
                            if n == 0 {
                                break;
                            }
                            bytes.extend_from_slice(&buffer[..n]);
                        }
                        let request = String::from_utf8(bytes).unwrap();
                        let target = request
                            .lines()
                            .next()
                            .unwrap()
                            .split_whitespace()
                            .nth(1)
                            .unwrap();
                        let response = handler(target, &server_base);
                        requests.lock().unwrap().push(request);
                        let headers = response
                            .headers
                            .iter()
                            .map(|(key, value)| format!("{key}: {value}\r\n"))
                            .collect::<String>();
                        let _ = write!(
                            stream,
                            "HTTP/1.1 {} Test\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n",
                            response.status,
                            response.body.len(),
                            headers
                        );
                        let _ = stream.write_all(&response.body);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2))
                    }
                    Err(error) => panic!("mock server failed: {error}"),
                }
            }
        });
        Self {
            base,
            seen,
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
fn platform(attempt: u64, name: &str) -> Value {
    json!({"source":"runner_environment","job_context_source":"caller","repository_id":"100","repository":"acme/demo","workflow_ref":"acme/demo/.github/workflows/build.yml@refs/heads/main","workflow_sha":"a".repeat(40),"run_id":"7","run_attempt":attempt,"job_key":"verify","job_name":name,"job_id":null,"matrix":{"os":"linux"},"actor":"original","triggering_actor":if attempt == 1 {"original"}else{"rerunner"},"event_name":"workflow_dispatch"})
}
fn receipt(attempt: u64, invocation: &str, phase: &str, name: &str) -> Value {
    json!({"schema":"observatory.tool-receipt/1","invocation_id":invocation,"phase":phase,"timestamp":if phase == "start" {"2026-11-01T10:00:00Z"} else {"2026-11-01T10:00:01Z"},"platform":platform(attempt,name),"declared":{"source":"caller","controller":"assistant","controller_provenance":["study:declaration"],"protocol_id":"protocol-1","task_id":"task-1","session_id":"session-1","logical_call_id":"call-1","parent_invocation_id":null},"tool":{"adapter":"command","name":"cargo","version":null,"purpose":"development","configured_model":null,"configuration":{"reasoning_effort":null,"temperature":null,"seed":null,"max_output_tokens":null,"configuration_sha256":null}},"result":if phase == "start" {Value::Null} else {json!({"status":"failure","exit_code":1,"runtime_ms":1000,"observed_model":null,"observed_session_id":null,"input_tokens":null,"output_tokens":null,"cached_input_tokens":null,"charge_amount":null,"charge_currency":null,"usage_source":"unknown","checks":[]})},"artifacts":[],"missing_fields":["tool.version"]})
}
fn bundle(platform: Value, receipts: &[Value]) -> Value {
    json!({"schema":"observatory.execution-bundle/1","platform":platform,"receipts":receipts.iter().map(|receipt| {let text=serde_json::to_string(receipt).unwrap();json!({"sha256":digest(text.as_bytes()),"receipt_json":text})}).collect::<Vec<_>>()})
}
fn zip(value: &Value) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(vec![]));
    writer
        .start_file("bundle.json", zip::write::SimpleFileOptions::default())
        .unwrap();
    writer
        .write_all(&serde_json::to_vec(value).unwrap())
        .unwrap();
    writer.finish().unwrap().into_inner()
}
fn default_response(target: &str) -> Response {
    let path = target.split('?').next().unwrap();
    if path == "/repos/acme/demo" {
        return Response::json(json!({"id":100,"full_name":"acme/demo"}));
    }
    let run = json!({"id":7,"run_attempt":2,"status":"completed","conclusion":"cancelled"});
    if path == "/repos/acme/demo/actions/runs" {
        return Response::json(json!({"total_count":1,"workflow_runs":[run]}));
    }
    if path == "/repos/acme/demo/actions/runs/7" {
        return Response::json(run);
    }
    for attempt in 1..=2 {
        if path == format!("/repos/acme/demo/actions/runs/7/attempts/{attempt}") {
            return Response::json(
                json!({"id":7,"run_attempt":attempt,"actor":{"login":"original"},"triggering_actor":{"login":if attempt == 1 {"original"}else{"rerunner"}},"event":"workflow_dispatch","status":"completed","conclusion":if attempt == 1 {"failure"} else {"cancelled"}}),
            );
        }
        if path == format!("/repos/acme/demo/actions/runs/7/attempts/{attempt}/jobs") {
            return Response::json(json!({"total_count":3,"jobs":[
                {"id":attempt*10+1,"run_id":7,"run_attempt":attempt,"name":"verify (linux)","status":"completed","conclusion":if attempt == 1 {"failure"} else {"cancelled"}},
                {"id":attempt*10+2,"run_id":7,"run_attempt":attempt,"name":"verify (macos)","status":"completed","conclusion":"success"},
                {"id":attempt*10+3,"run_id":7,"run_attempt":attempt,"name":"deploy","status":"completed","conclusion":"skipped"}
            ]}));
        }
    }
    if path.ends_with("/artifacts") {
        return Response::json(json!({"total_count":0,"artifacts":[]}));
    }
    Response {
        status: 404,
        body: vec![],
        headers: vec![],
    }
}
fn artifact(id: u64, bytes: &[u8]) -> Value {
    json!({"id":id,"name":format!("observatory-executions-{id}"),"size_in_bytes":bytes.len(),"expired":false,"digest":format!("sha256:{}",digest(bytes)),"workflow_run":{"id":7}})
}
fn source<'a>(collection: &'a Collection, name: &str) -> Vec<&'a Value> {
    collection
        .records
        .iter()
        .filter(|record| record.data["source"] == name)
        .map(|record| &record.data["payload"])
        .collect()
}

#[test]
fn missing_uploads_preserve_failed_cancelled_and_skipped_jobs_and_original_rerun_actors() {
    let server = Server::start(|target, _| default_response(target));
    let collection = collect_github_executions("acme/demo", None, &server.base, Some(7)).unwrap();
    assert_eq!(source(&collection, "github.workflow_job").len(), 6);
    let attempts = source(&collection, "github.workflow_attempt");
    assert_eq!(attempts[1]["actor"]["login"], "original");
    assert_eq!(attempts[1]["triggering_actor"]["login"], "rerunner");
    let jobs = source(&collection, "github.execution_job_reconciliation");
    assert!(
        jobs.iter()
            .all(|job| job["receipt_observation"] == "no_receipts_observed")
    );
    assert!(jobs.iter().any(|job| job["conclusion"] == "skipped"));
    assert!(jobs.iter().any(|job| job["conclusion"] == "cancelled"));
    assert!(jobs.iter().any(|job| job["conclusion"] == "failure"));
    assert!(source(&collection, "observatory.tool_receipt").is_empty());
}

#[test]
fn repeated_artifact_deliveries_deduplicate_and_separate_reruns_matrix_and_partial_lifecycles() {
    let start = receipt(1, "first", "start", "verify (linux)");
    let completion = receipt(1, "first", "completion", "verify (linux)");
    let later = receipt(2, "retry", "start", "verify (linux)");
    let archives = [
        zip(&bundle(
            start["platform"].clone(),
            &[start.clone(), completion],
        )),
        zip(&bundle(start["platform"].clone(), &[start])),
        zip(&bundle(later["platform"].clone(), &[later])),
        zip(&bundle(platform(2, "verify (macos)"), &[])),
    ];
    let server = Server::start(move |target, _| {
        if target.starts_with("/repos/acme/demo/actions/runs/7/artifacts") {
            return Response::json(
                json!({"total_count":archives.len(),"artifacts":archives.iter().enumerate().map(|(index,bytes)|artifact(index as u64+1,bytes)).collect::<Vec<_>>()}),
            );
        }
        for (index, archive) in archives.iter().enumerate() {
            if target == format!("/repos/acme/demo/actions/artifacts/{}/zip", index + 1) {
                return Response::bytes(archive.clone());
            }
        }
        default_response(target)
    });
    let first = collect_github_executions("acme/demo", None, &server.base, None).unwrap();
    let invocations = source(&first, "github.execution_reconciliation");
    assert_eq!(invocations.len(), 2);
    assert!(
        invocations
            .iter()
            .any(|v| v["lifecycle"] == "start_only" && v["job_conclusion"] == "cancelled")
    );
    assert!(
        invocations
            .iter()
            .any(|v| v["lifecycle"] == "start_and_completion_observed")
    );
    assert!(
        source(&first, "github.execution_job_reconciliation")
            .iter()
            .any(|v| v["job_id"] == 22 && v["receipt_observation"] == "instrumented_no_receipts")
    );
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let first_count = persist_collection(&mut store, first, "test", CollectionRequirements::Full)
        .unwrap()
        .new_records;
    assert!(first_count > 0);
    let second = collect_github_executions("acme/demo", None, &server.base, None).unwrap();
    assert_eq!(
        persist_collection(&mut store, second, "test", CollectionRequirements::Full)
            .unwrap()
            .new_records,
        0
    );
    assert_eq!(
        store
            .records(None)
            .unwrap()
            .iter()
            .filter(|r| r.data["source"] == "observatory.tool_receipt")
            .count(),
        3
    );
    store.verify().unwrap();
}

#[test]
fn signed_redirects_never_receive_github_credentials_or_enter_retained_metadata() {
    let archive = zip(&bundle(platform(1, "verify (linux)"), &[]));
    let server = Server::start(move |target, base| {
        if target.starts_with("/repos/acme/demo/actions/runs/7/artifacts") {
            return Response::json(json!({"total_count":1,"artifacts":[artifact(1,&archive)]}));
        }
        if target == "/repos/acme/demo/actions/artifacts/1/zip" {
            return Response {
                status: 302,
                body: vec![],
                headers: vec![(
                    "Location".into(),
                    format!("{base}signed?credential=do-not-retain"),
                )],
            };
        }
        if target == "/signed?credential=do-not-retain" {
            return Response::bytes(archive.clone());
        }
        default_response(target)
    });
    let collection = collect_github_executions(
        "acme/demo",
        Some("private-test-token"),
        &server.base,
        Some(7),
    )
    .unwrap();
    let requests = server.seen.lock().unwrap();
    assert!(
        requests
            .iter()
            .find(|r| r.starts_with("GET /repos/acme/demo/actions/artifacts/1/zip "))
            .unwrap()
            .to_lowercase()
            .contains("authorization: bearer private-test-token")
    );
    assert!(
        !requests
            .iter()
            .find(|r| r.starts_with("GET /signed?"))
            .unwrap()
            .to_lowercase()
            .contains("authorization:")
    );
    let retained = serde_json::to_string(&collection).unwrap();
    assert!(!retained.contains("do-not-retain"));
    assert!(!retained.contains("private-test-token"));
    assert_eq!(source(&collection, "observatory.execution_bundle").len(), 1);
}

#[test]
fn invalid_expired_and_wrong_run_artifacts_retain_coverage_and_do_not_erase_jobs() {
    let mut wrong = platform(1, "verify (linux)");
    wrong["run_id"] = json!("9");
    let archives = [
        zip(&bundle(wrong, &[])),
        zip(&json!({"schema":"unknown"})),
        zip(&bundle(platform(1, "verify (linux)"), &[])),
    ];
    let server = Server::start(move |target, _| {
        if target.starts_with("/repos/acme/demo/actions/runs/7/artifacts") {
            let mut expired = artifact(3, &archives[2]);
            expired["expired"] = json!(true);
            return Response::json(
                json!({"total_count":3,"artifacts":[artifact(1,&archives[0]),artifact(2,&archives[1]),expired]}),
            );
        }
        for (index, archive) in archives.iter().enumerate() {
            if target == format!("/repos/acme/demo/actions/artifacts/{}/zip", index + 1) {
                return Response::bytes(archive.clone());
            }
        }
        default_response(target)
    });
    let collection = collect_github_executions("acme/demo", None, &server.base, Some(7)).unwrap();
    assert_eq!(source(&collection, "github.workflow_job").len(), 6);
    assert!(source(&collection, "observatory.execution_bundle").is_empty());
    assert_eq!(
        collection
            .coverage
            .iter()
            .filter(|v| v.source == "github.execution_artifact_bundle" && v.state == "incomplete")
            .count(),
        3
    );
    assert!(
        !server
            .seen
            .lock()
            .unwrap()
            .iter()
            .any(|r| r.starts_with("GET /repos/acme/demo/actions/artifacts/3/zip "))
    );
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    assert!(
        persist_collection(&mut store, collection, "test", CollectionRequirements::Full).is_err()
    );
    assert!(
        store
            .records(None)
            .unwrap()
            .iter()
            .any(|r| r.data["source"] == "github.workflow_job")
    );
    store.verify().unwrap();
}

#[test]
fn ambiguous_names_remain_unresolved_and_completion_only_is_not_a_start() {
    let receipt = receipt(1, "only-completion", "completion", "verify (linux)");
    let archive = zip(&bundle(receipt["platform"].clone(), &[receipt]));
    let server = Server::start(move |target, _| {
        if target.starts_with("/repos/acme/demo/actions/runs/7/artifacts") {
            return Response::json(json!({"total_count":1,"artifacts":[artifact(1,&archive)]}));
        }
        if target == "/repos/acme/demo/actions/artifacts/1/zip" {
            return Response::bytes(archive.clone());
        }
        let mut response = default_response(target);
        if target.starts_with("/repos/acme/demo/actions/runs/7/attempts/1/jobs") {
            let mut value: Value = serde_json::from_slice(&response.body).unwrap();
            value["jobs"][1]["name"] = json!("verify (linux)");
            response = Response::json(value);
        }
        response
    });
    let collection = collect_github_executions("acme/demo", None, &server.base, Some(7)).unwrap();
    let reconciled = source(&collection, "github.execution_reconciliation");
    assert_eq!(reconciled[0]["lifecycle"], "completion_only");
    assert_eq!(reconciled[0]["job_link"], "unresolved");
    assert_eq!(reconciled[0]["job_id"], Value::Null);
}

#[test]
fn unknown_platform_provenance_is_retained_without_inventing_a_job_link() {
    let mut receipt = receipt(1, "unknown-origin", "start", "verify (linux)");
    for field in [
        "repository_id",
        "repository",
        "run_id",
        "actor",
        "triggering_actor",
        "event_name",
    ] {
        receipt["platform"][field] = json!("unknown");
    }
    receipt["platform"]["run_attempt"] = Value::Null;
    receipt["declared"]["controller"] = json!("unknown");
    let archive = zip(&bundle(receipt["platform"].clone(), &[receipt]));
    let server = Server::start(move |target, _| {
        if target.starts_with("/repos/acme/demo/actions/runs/7/artifacts") {
            return Response::json(json!({"total_count":1,"artifacts":[artifact(1,&archive)]}));
        }
        if target == "/repos/acme/demo/actions/artifacts/1/zip" {
            return Response::bytes(archive.clone());
        }
        default_response(target)
    });
    let collection = collect_github_executions("acme/demo", None, &server.base, Some(7)).unwrap();
    let receipts = source(&collection, "observatory.tool_receipt");
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0]["platform"]["run_id"], "unknown");
    assert_eq!(receipts[0]["declared"]["controller"], "unknown");
    assert_eq!(
        source(&collection, "github.execution_reconciliation")[0]["job_link"],
        "unresolved"
    );
    assert!(
        source(&collection, "github.execution_job_reconciliation")
            .iter()
            .all(|job| job["receipt_observation"] == "unresolved_receipts_present")
    );
}

#[test]
fn paginated_inventory_quarantines_conflicting_later_delivery_and_retains_prior_receipt() {
    let initial = receipt(1, "stable", "start", "verify (linux)");
    let mut conflict = initial.clone();
    conflict["declared"]["controller"] = json!("human");
    let archives = [
        zip(&bundle(initial["platform"].clone(), &[initial])),
        zip(&bundle(conflict["platform"].clone(), &[conflict])),
    ];
    let server = Server::start(move |target, base| {
        if target.starts_with("/repos/acme/demo/actions/runs/7/artifacts") {
            if target.contains("page=2") {
                return Response::json(
                    json!({"total_count":2,"artifacts":[artifact(2,&archives[1])]}),
                );
            }
            let mut response =
                Response::json(json!({"total_count":2,"artifacts":[artifact(1,&archives[0])]}));
            response.headers.push((
                "Link".into(),
                format!("<{base}repos/acme/demo/actions/runs/7/artifacts?page=2>; rel=\"next\""),
            ));
            return response;
        }
        for (index, archive) in archives.iter().enumerate() {
            if target == format!("/repos/acme/demo/actions/artifacts/{}/zip", index + 1) {
                return Response::bytes(archive.clone());
            }
        }
        default_response(target)
    });
    let collection = collect_github_executions("acme/demo", None, &server.base, Some(7)).unwrap();
    assert_eq!(source(&collection, "observatory.tool_receipt").len(), 1);
    assert_eq!(
        source(&collection, "observatory.tool_receipt")[0]["declared"]["controller"],
        "assistant"
    );
    assert!(
        collection
            .coverage
            .iter()
            .any(|v| v.source == "github.execution_artifact"
                && v.pages == 2
                && v.state == "complete")
    );
    assert!(
        collection
            .coverage
            .iter()
            .any(|v| v.source == "github.execution_artifact_bundle" && v.state == "incomplete")
    );
    assert_eq!(
        collection
            .observations
            .iter()
            .filter(|o| o.source == "github.execution_artifact_archive" && o.status == Some(200))
            .count(),
        2
    );
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    assert!(
        persist_collection(&mut store, collection, "test", CollectionRequirements::Full).is_err()
    );
    assert_eq!(
        store
            .records(None)
            .unwrap()
            .iter()
            .filter(|record| record.data["source"] == "observatory.tool_receipt")
            .count(),
        1
    );
    store.verify().unwrap();
}

#[test]
fn incomplete_provenance_cannot_join_a_different_immutable_invocation_scope() {
    let mut start = receipt(1, "shared-label", "start", "verify (linux)");
    start["platform"]["repository_id"] = json!("unknown");
    start["platform"]["run_id"] = json!("unknown");
    let end = receipt(1, "shared-label", "completion", "verify (linux)");
    let archives = [
        zip(&bundle(start["platform"].clone(), &[start])),
        zip(&bundle(end["platform"].clone(), &[end])),
    ];
    let server = Server::start(move |target, _| {
        if target.starts_with("/repos/acme/demo/actions/runs/7/artifacts") {
            return Response::json(
                json!({"total_count":2,"artifacts":[artifact(1,&archives[0]),artifact(2,&archives[1])]}),
            );
        }
        for (index, archive) in archives.iter().enumerate() {
            if target == format!("/repos/acme/demo/actions/artifacts/{}/zip", index + 1) {
                return Response::bytes(archive.clone());
            }
        }
        default_response(target)
    });
    let collection = collect_github_executions("acme/demo", None, &server.base, Some(7)).unwrap();
    let invocations = source(&collection, "github.execution_reconciliation");
    assert_eq!(invocations.len(), 2);
    assert!(
        invocations
            .iter()
            .any(|v| v["lifecycle"] == "start_only" && v["job_link"] == "unresolved")
    );
    assert!(
        invocations
            .iter()
            .any(|v| v["lifecycle"] == "completion_only" && v["job_link"] == "unambiguous")
    );
}
