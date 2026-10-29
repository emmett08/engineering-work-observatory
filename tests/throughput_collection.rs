use observatory::collectors::{
    Collection, collect_github_throughput, collect_github_throughput_cached,
};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct Reply {
    status: u16,
    body: Value,
    link: Option<String>,
}

impl Reply {
    fn ok(body: Value) -> Self {
        Self {
            status: 200,
            body,
            link: None,
        }
    }
}

struct Server {
    base: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Server {
    fn new(handler: impl Fn(&str, &str, &str) -> Reply + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}/", listener.local_addr().unwrap());
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
                        let reply = handler(target, &endpoint, &request);
                        let body = if reply.status == 304 {
                            vec![]
                        } else {
                            serde_json::to_vec(&reply.body).unwrap()
                        };
                        let link = reply
                            .link
                            .map(|link| format!("Link: {link}\r\n"))
                            .unwrap_or_default();
                        write!(stream, "HTTP/1.1 {} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nETag: \"throughput-v1\"\r\n{}\r\n", reply.status, body.len(), link).unwrap();
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

fn repository() -> Value {
    json!({"id": 42, "full_name": "acme/demo", "created_at": "2026-01-01T12:00:00Z"})
}

fn pull(number: u64) -> Value {
    json!({"id": 1000 + number, "number": number, "state": "closed", "created_at": "2026-01-02T00:00:00Z", "closed_at": "2026-01-03T00:00:00Z", "merged_at": "2026-01-03T00:00:00Z"})
}

fn summary(collection: &Collection) -> &Value {
    &collection
        .records
        .iter()
        .find(|r| r.data["source"] == "github.throughput_collection")
        .unwrap()
        .data["payload"]
}

fn inventory_state(collection: &Collection) -> &str {
    &collection
        .coverage
        .iter()
        .find(|c| c.source == "github.pull_request")
        .unwrap()
        .state
}

#[test]
fn lightweight_profile_collects_only_identity_and_pull_inventory() {
    let server = Server::new(|target, _, _| {
        if target == "/repos/acme/demo" {
            Reply::ok(repository())
        } else {
            assert_eq!(
                target,
                "/repos/acme/demo/pulls?state=all&sort=created&direction=asc&per_page=100"
            );
            Reply::ok(json!([pull(1)]))
        }
    });
    let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
    assert_eq!(server.requests.lock().unwrap().len(), 2);
    assert_eq!(collection.records.len(), 3);
    assert_eq!(inventory_state(&collection), "complete");
    assert!(
        collection
            .records
            .iter()
            .all(|r| r.data["repository"] == "github:repository:42")
    );
    assert_eq!(summary(&collection)["profile"], "throughput");
    assert_eq!(summary(&collection)["unique_pull_requests"], 1);
    assert_eq!(summary(&collection)["duplicate_pull_request_rows"], 0);
    assert_eq!(
        summary(&collection)["coverage"].as_array().unwrap().len(),
        2
    );
    assert!(
        collection
            .coverage
            .iter()
            .filter(|c| !["github.repository", "github.pull_request"].contains(&c.source.as_str()))
            .all(|c| c.state == "not_collected")
    );
    assert!(
        summary(&collection)["started_at"].as_str().unwrap()
            <= summary(&collection)["completed_at"].as_str().unwrap()
    );
}

#[test]
fn paginated_inventory_keeps_every_page_without_nested_requests() {
    let server = Server::new(|target, base, _| {
        if target == "/repos/acme/demo" {
            Reply::ok(repository())
        } else if target.contains("page=2") {
            Reply::ok(json!([pull(2)]))
        } else {
            Reply {
                status: 200,
                body: json!([pull(1)]),
                link: Some(format!(
                    "<{base}repos/acme/demo/pulls?page=2>; rel=\"next\""
                )),
            }
        }
    });
    let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
    let coverage = collection
        .coverage
        .iter()
        .find(|c| c.source == "github.pull_request")
        .unwrap();
    assert_eq!((coverage.pages, coverage.records), (2, 2));
    assert_eq!(inventory_state(&collection), "complete");
    assert_eq!(server.requests.lock().unwrap().len(), 3);
    assert_eq!(
        collection
            .records
            .iter()
            .filter(|r| r.data["source"] == "github.pull_request")
            .count(),
        2
    );
}

#[test]
fn failed_later_page_retains_prefix_and_marks_summary_incomplete() {
    let server = Server::new(|target, base, _| {
        if target == "/repos/acme/demo" {
            Reply::ok(repository())
        } else if target.contains("page=2") {
            Reply {
                status: 404,
                body: json!({"message": "Not found"}),
                link: None,
            }
        } else {
            Reply {
                status: 200,
                body: json!([pull(1)]),
                link: Some(format!(
                    "<{base}repos/acme/demo/pulls?page=2>; rel=\"next\""
                )),
            }
        }
    });
    let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
    assert_eq!(inventory_state(&collection), "incomplete");
    assert_eq!(summary(&collection)["coverage"][1]["state"], "incomplete");
    assert_eq!(summary(&collection)["coverage"][1]["records"], 1);
    assert!(
        collection
            .observations
            .iter()
            .any(|r| r.status == Some(404))
    );
}

#[test]
fn malformed_http_success_is_not_a_complete_inventory() {
    let server = Server::new(|target, _, _| {
        Reply::ok(if target == "/repos/acme/demo" {
            repository()
        } else {
            json!({"unexpected": []})
        })
    });
    let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
    assert_eq!(inventory_state(&collection), "request_failed");
    assert_eq!(
        summary(&collection)["coverage"][1]["state"],
        "request_failed"
    );
    assert_eq!(collection.observations.len(), 2);
}

#[test]
fn malformed_timestamp_identity_and_state_fields_keep_raw_evidence_but_fail_coverage() {
    for (field, bad) in [
        ("id", json!(0)),
        ("number", json!(0)),
        ("state", json!("unknown")),
        ("created_at", json!("yesterday")),
        ("closed_at", json!(123)),
        ("merged_at", json!("not-a-date")),
    ] {
        let mut row = pull(1);
        row[field] = bad;
        let server = Server::new(move |target, _, _| {
            Reply::ok(if target == "/repos/acme/demo" {
                repository()
            } else {
                json!([row])
            })
        });
        let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
        assert_eq!(inventory_state(&collection), "incomplete", "field {field}");
        assert_eq!(summary(&collection)["coverage"][1]["state"], "incomplete");
        assert_eq!(
            collection
                .records
                .iter()
                .filter(|r| r.data["source"] == "github.pull_request")
                .count(),
            1
        );
        assert!(!collection.observations[1].body.is_empty());
    }
}

#[test]
fn missing_nullable_timestamps_and_malformed_repository_are_not_silently_accepted() {
    let server = Server::new(|target, _, _| {
        if target == "/repos/acme/demo" {
            Reply::ok(json!({"id": 42}))
        } else {
            let mut row = pull(1);
            row.as_object_mut().unwrap().remove("merged_at");
            Reply::ok(json!([row]))
        }
    });
    let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
    assert_eq!(summary(&collection)["coverage"][0]["state"], "incomplete");
    assert_eq!(summary(&collection)["coverage"][1]["state"], "incomplete");
}

#[test]
fn successful_empty_inventory_and_open_null_timestamps_are_valid() {
    for empty in [false, true] {
        let server = Server::new(move |target, _, _| {
            if target == "/repos/acme/demo" {
                Reply::ok(repository())
            } else if empty {
                Reply::ok(json!([]))
            } else {
                let mut row = pull(1);
                row["state"] = json!("open");
                row["merged_at"] = Value::Null;
                row["closed_at"] = Value::Null;
                Reply::ok(json!([row]))
            }
        });
        let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
        assert_eq!(inventory_state(&collection), "complete");
        assert_eq!(
            summary(&collection)["coverage"][1]["records"],
            if empty { 0 } else { 1 }
        );
    }
}

#[test]
fn cached_profile_revalidates_inventory_and_preserves_original_payload_identity() {
    let server = Server::new(|target, _, request| {
        if request.to_ascii_lowercase().contains("if-none-match:") {
            Reply {
                status: 304,
                body: Value::Null,
                link: None,
            }
        } else {
            Reply::ok(if target == "/repos/acme/demo" {
                repository()
            } else {
                json!([pull(1)])
            })
        }
    });
    let cache = tempfile::tempdir().unwrap();
    let first =
        collect_github_throughput_cached("acme/demo", None, &server.base, cache.path()).unwrap();
    let second =
        collect_github_throughput_cached("acme/demo", None, &server.base, cache.path()).unwrap();
    assert_eq!(inventory_state(&second), "complete");
    let source_ids = |c: &Collection| {
        c.records
            .iter()
            .filter(|r| r.data["source"] != "github.throughput_collection")
            .map(|r| r.id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(source_ids(&first), source_ids(&second));
    assert_eq!(
        second
            .observations
            .iter()
            .filter(|r| r.status == Some(304))
            .count(),
        2
    );
    assert_eq!(
        second
            .observations
            .iter()
            .filter(|r| r.source.ends_with(".cached_body"))
            .count(),
        2
    );
    let started = summary(&second)["started_at"].as_str().unwrap();
    assert!(
        second
            .observations
            .iter()
            .any(|r| r.status == Some(304) && r.collected_at == started)
    );
}

#[test]
fn impossible_or_future_event_chronology_cannot_establish_complete_coverage() {
    for (field, bad) in [
        ("merged_at", json!("2026-01-01T00:00:00Z")),
        ("closed_at", json!("2026-01-01T00:00:00Z")),
        ("created_at", json!("2999-01-01T00:00:00Z")),
        ("merged_at", json!("2999-01-01T00:00:00Z")),
        ("merged_at", json!("2026-01-04T00:00:00Z")),
        ("closed_at", Value::Null),
        ("state", json!("open")),
    ] {
        let mut row = pull(1);
        row[field] = bad;
        let server = Server::new(move |target, _, _| {
            Reply::ok(if target == "/repos/acme/demo" {
                repository()
            } else {
                json!([row])
            })
        });
        let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
        assert_eq!(inventory_state(&collection), "incomplete", "field {field}");
    }
}

#[test]
fn conflicting_duplicate_pages_are_not_ordered_by_content_hash() {
    for conflict in [false, true] {
        let server = Server::new(move |target, base, _| {
            if target == "/repos/acme/demo" {
                Reply::ok(repository())
            } else if target.contains("page=2") {
                let mut row = pull(1);
                if conflict {
                    row["merged_at"] = json!("2026-01-02T23:00:00Z");
                }
                Reply::ok(json!([row]))
            } else {
                Reply {
                    status: 200,
                    body: json!([pull(1)]),
                    link: Some(format!(
                        "<{base}repos/acme/demo/pulls?page=2>; rel=\"next\""
                    )),
                }
            }
        });
        let collection = collect_github_throughput("acme/demo", None, &server.base).unwrap();
        assert_eq!(
            inventory_state(&collection),
            if conflict { "incomplete" } else { "complete" }
        );
        assert_eq!(summary(&collection)["unique_pull_requests"], 1);
        assert_eq!(summary(&collection)["duplicate_pull_request_rows"], 1);
        assert_eq!(summary(&collection)["coverage"][1]["records"], 2);
        if conflict {
            assert!(
                collection
                    .coverage
                    .iter()
                    .find(|c| c.source == "github.pull_request")
                    .unwrap()
                    .detail
                    .as_ref()
                    .unwrap()
                    .contains("conflicting duplicate")
            );
        }
    }
}
