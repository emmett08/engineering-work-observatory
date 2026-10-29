use observatory::collectors::{Collection, collect_git, collect_github};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

struct Response {
    status: u16,
    body: Value,
    link: Option<String>,
}

impl Response {
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
    fn start(handler: impl Fn(&str, &str) -> Response + Send + Sync + 'static) -> Self {
        Self::start_headers(move |target, base, _request| handler(target, base))
    }

    fn start_headers(
        handler: impl Fn(&str, &str, &str) -> Response + Send + Sync + 'static,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let seen = requests.clone();
        let stopping = stop.clone();
        let server_base = base.clone();
        let handle = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(30);
            while !stopping.load(Ordering::SeqCst) && Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let request = read_request(&mut stream);
                        let target = request
                            .lines()
                            .next()
                            .unwrap_or("")
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or("");
                        seen.lock().unwrap().push(request.clone());
                        let reply = handler(target, &server_base, &request);
                        let body = if reply.status == 304 {
                            vec![]
                        } else {
                            serde_json::to_vec(&reply.body).unwrap()
                        };
                        let link = reply
                            .link
                            .map(|l| format!("Link: {l}\r\n"))
                            .unwrap_or_default();
                        let retry_after = if reply.status == 429 {
                            "Retry-After: 120\r\n"
                        } else {
                            ""
                        };
                        let etag = if reply.status == 200 {
                            "ETag: \"fixture-v1\"\r\n"
                        } else {
                            ""
                        };
                        let headers = format!(
                            "HTTP/1.1 {} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}{}{}\r\n",
                            reply.status,
                            body.len(),
                            link,
                            retry_after,
                            etag
                        );
                        let _ = stream.write_all(headers.as_bytes());
                        let _ = stream.write_all(&body);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2))
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
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut buffer = [0; 2048];
    while bytes.len() < 65_536 {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                bytes.extend_from_slice(&buffer[..n]);
                if bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn default_response(target: &str) -> Response {
    let path = target.split('?').next().unwrap();
    if path == "/repos/acme/demo/" || path == "/repos/acme/demo" {
        Response::ok(json!({"id": 100, "full_name": "acme/demo"}))
    } else if path.ends_with("/actions/runs") {
        Response::ok(json!({"total_count": 0, "workflow_runs": []}))
    } else if path.ends_with("/jobs") {
        Response::ok(json!({"total_count": 0, "jobs": []}))
    } else if path.ends_with("/check-runs") {
        Response::ok(json!({"total_count": 0, "check_runs": []}))
    } else {
        Response::ok(json!([]))
    }
}

#[test]
fn repository_metadata_endpoint_has_no_trailing_slash() {
    let server = Server::start(|target, _| {
        if target == "/repos/acme/demo/" {
            Response {
                status: 404,
                body: json!({"message":"Not Found"}),
                link: None,
            }
        } else {
            default_response(target)
        }
    });
    let collection = collect_github("acme/demo", None, &server.base).unwrap();
    assert_eq!(records(&collection, "github.repository").len(), 1);
    assert!(
        collection
            .coverage
            .iter()
            .any(|c| c.source == "github.repository" && c.state == "complete")
    );
}

fn records<'a>(
    collection: &'a Collection,
    source: &str,
) -> Vec<&'a observatory::collectors::CollectedRecord> {
    collection
        .records
        .iter()
        .filter(|r| r.data["source"] == source)
        .collect()
}

#[test]
fn paginated_comments_are_retained_with_complete_endpoint_coverage() {
    let server = Server::start(|target, base| {
        if target.starts_with("/repos/acme/demo/issues/comments") {
            if target.contains("page=2") {
                Response::ok(
                    json!([{"id": 2, "body": "second", "created_at": "2026-01-02T00:00:00Z"}]),
                )
            } else {
                Response {
                    status: 200,
                    body: json!([{"id": 1, "body": "first", "created_at": "2026-01-01T00:00:00Z"}]),
                    link: Some(format!(
                        "<{base}repos/acme/demo/issues/comments?page=2>; rel=\"next\""
                    )),
                }
            }
        } else {
            default_response(target)
        }
    });
    let collection = collect_github("acme/demo", None, &server.base).unwrap();
    assert_eq!(records(&collection, "github.issue_comment").len(), 2);
    let coverage = collection
        .coverage
        .iter()
        .find(|c| c.source == "github.issue_comment")
        .unwrap();
    assert_eq!(coverage.state, "complete");
    assert_eq!((coverage.pages, coverage.records), (2, 2));
    assert_eq!(
        collection
            .observations
            .iter()
            .filter(|r| r.source == "github.issue_comment")
            .count(),
        2
    );
    assert!(
        records(&collection, "github.issue_comment")
            .iter()
            .all(|r| r.id.starts_with("github:repository:100/"))
    );
    assert_eq!(
        records(&collection, "github.issue_comment")[0].data["payload"]["created_at"],
        "2026-01-01T00:00:00Z"
    );
}

#[test]
fn failed_middle_page_preserves_prefix_and_cannot_claim_completeness() {
    let server = Server::start(|target, base| {
        if target.starts_with("/repos/acme/demo/issues/comments") {
            if target.contains("page=2") {
                Response {
                    status: 418,
                    body: json!({"message": "unavailable"}),
                    link: None,
                }
            } else {
                Response {
                    status: 200,
                    body: json!([{"id": 1}]),
                    link: Some(format!(
                        "<{base}repos/acme/demo/issues/comments?page=2>; rel=\"next\""
                    )),
                }
            }
        } else {
            default_response(target)
        }
    });
    let collection = collect_github("acme/demo", None, &server.base).unwrap();
    assert_eq!(records(&collection, "github.issue_comment").len(), 1);
    let coverage = collection
        .coverage
        .iter()
        .find(|c| c.source == "github.issue_comment")
        .unwrap();
    assert_eq!(coverage.state, "incomplete");
    assert_eq!((coverage.pages, coverage.records), (1, 1));
    assert!(coverage.detail.as_deref().unwrap().contains("418"));
    assert!(
        collection
            .observations
            .iter()
            .any(|r| r.source == "github.issue_comment" && r.status == Some(418))
    );
    assert!(
        collection
            .coverage
            .iter()
            .any(|c| c.source == "github.workflow_run" && c.state == "complete")
    );
}

#[test]
fn cross_origin_pagination_is_rejected_without_transmitting_bearer_token() {
    let foreign = Server::start(|target, _| default_response(target));
    let foreign_base = foreign.base.clone();
    let server = Server::start(move |target, _| {
        if target.starts_with("/repos/acme/demo/issues/comments") {
            Response {
                status: 200,
                body: json!([{"id": 1}]),
                link: Some(format!("<{foreign_base}stolen>; rel=\"next\"")),
            }
        } else {
            default_response(target)
        }
    });
    let collection = collect_github("acme/demo", Some("test-only-token"), &server.base).unwrap();
    let coverage = collection
        .coverage
        .iter()
        .find(|c| c.source == "github.issue_comment")
        .unwrap();
    assert_eq!(coverage.state, "incomplete");
    assert!(coverage.detail.as_deref().unwrap().contains("origin"));
    assert!(foreign.requests.lock().unwrap().is_empty());
    assert!(server.requests.lock().unwrap().iter().all(|r| {
        r.to_ascii_lowercase()
            .contains("authorization: bearer test-only-token")
    }));
    let serialized = serde_json::to_string(&collection).unwrap();
    assert!(!serialized.contains("test-only-token"));
}

#[test]
fn workflow_reruns_keep_distinct_attempts_and_jobs() {
    let server = Server::start(|target, _| {
        let path = target.split('?').next().unwrap();
        if path.ends_with("/actions/runs") {
            Response::ok(
                json!({"total_count": 1, "workflow_runs": [{"id": 7, "run_attempt": 2, "head_sha": "abc", "event": "workflow_dispatch"}]}),
            )
        } else if path.ends_with("/attempts/1/jobs") || path.ends_with("/attempts/2/jobs") {
            Response::ok(json!({"total_count": 1, "jobs": [{"id": 42, "name": "verify"}]}))
        } else if path.ends_with("/attempts/1") {
            Response::ok(json!({"id": 7, "run_attempt": 1, "conclusion": "failure"}))
        } else if path.ends_with("/attempts/2") {
            Response::ok(json!({"id": 7, "run_attempt": 2, "conclusion": "success"}))
        } else {
            default_response(target)
        }
    });
    let collection = collect_github("acme/demo", None, &server.base).unwrap();
    let attempts = records(&collection, "github.workflow_attempt");
    assert_eq!(attempts.len(), 2);
    assert!(attempts.iter().any(|r| r.id.contains("run:7/attempt:1/")));
    assert!(attempts.iter().any(|r| r.id.contains("run:7/attempt:2/")));
    let jobs = records(&collection, "github.workflow_job");
    assert_eq!(jobs.len(), 2);
    assert_ne!(jobs[0].id, jobs[1].id);
    assert!(
        collection
            .coverage
            .iter()
            .any(|c| c.source == "github.workflow_logs" && c.state == "not_collected")
    );
}

#[test]
fn unchanged_github_content_has_stable_ids_across_collection_times() {
    let server = Server::start(|target, _| default_response(target));
    let first = collect_github("acme/demo", None, &server.base).unwrap();
    let second = collect_github("acme/demo", None, &server.base).unwrap();
    assert!(!first.records.is_empty());
    assert_eq!(
        first.records.iter().map(|r| &r.id).collect::<Vec<_>>(),
        second.records.iter().map(|r| &r.id).collect::<Vec<_>>()
    );
    // Exercise the actual immutable store: equal IDs must also have equal complete content.
    let directory = tempfile::tempdir().unwrap();
    let mut store = observatory::store::Store::open(directory.path()).unwrap();
    let first_records: Vec<observatory::model::Evidence> = first
        .records
        .into_iter()
        .map(|r| observatory::model::Evidence {
            id: r.id,
            kind: r.kind,
            data: r.data,
        })
        .collect();
    let second_records: Vec<observatory::model::Evidence> = second
        .records
        .into_iter()
        .map(|r| observatory::model::Evidence {
            id: r.id,
            kind: r.kind,
            data: r.data,
        })
        .collect();
    let bytes = serde_json::to_vec(&first_records).unwrap();
    let hash = store.retain_bytes(&bytes).unwrap();
    let first_time = chrono::Utc::now();
    assert_eq!(
        store
            .insert_batch(&first_records, "synthetic-github", first_time, &hash)
            .unwrap(),
        first_records.len()
    );
    assert_eq!(
        store
            .insert_batch(
                &second_records,
                "synthetic-github",
                first_time + chrono::Duration::seconds(1),
                &hash
            )
            .unwrap(),
        0
    );
    assert_eq!(store.records(None).unwrap().len(), first_records.len());
}

fn git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .env("GIT_AUTHOR_NAME", "Synthetic Tester")
        .env("GIT_AUTHOR_EMAIL", "tester@example.invalid")
        .env("GIT_COMMITTER_NAME", "Synthetic Tester")
        .env("GIT_COMMITTER_EMAIL", "tester@example.invalid")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn local_git_collects_graph_refs_and_dependency_content_without_running_hooks() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path();
    git(path, &["init", "--initial-branch=main"]);
    std::fs::write(
        path.join("Cargo.toml"),
        "[package]\nname = \"synthetic\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(path.join(".github/workflows")).unwrap();
    std::fs::write(
        path.join(".github/workflows/ci.yml"),
        "name: first-fixture\non: workflow_dispatch\n",
    )
    .unwrap();
    git(path, &["add", "Cargo.toml", ".github/workflows/ci.yml"]);
    git(path, &["commit", "-m", "first synthetic commit"]);
    let first_sha = git(path, &["rev-parse", "HEAD"]);
    git(path, &["branch", "baseline"]);
    std::fs::write(
        path.join("Cargo.lock"),
        "# Synthetic fixture\nversion = 4\n",
    )
    .unwrap();
    std::fs::write(
        path.join(".github/workflows/ci.yml"),
        "name: second-fixture\non: [push, pull_request]\n",
    )
    .unwrap();
    git(path, &["add", "Cargo.lock", ".github/workflows/ci.yml"]);
    git(path, &["commit", "-m", "second synthetic commit"]);
    let second_sha = git(path, &["rev-parse", "HEAD"]);
    let hook = path.join(".git/hooks/post-checkout");
    std::fs::write(&hook, "#!/bin/sh\ntouch hook-ran\nexit 99\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let collection = collect_git(path, "acme/demo").unwrap();
    let commits = records(&collection, "git.commit");
    assert_eq!(commits.len(), 2);
    let newest = commits
        .iter()
        .find(|r| r.data["payload"]["sha"] == second_sha)
        .unwrap();
    assert_eq!(newest.data["payload"]["parents"], json!([first_sha]));
    assert!(
        newest.data["payload"]["author"]
            .as_str()
            .unwrap()
            .contains("Synthetic Tester <tester@example.invalid>")
    );
    assert!(
        records(&collection, "git.ref")
            .iter()
            .any(|r| r.data["payload"]["ref"] == "refs/heads/baseline")
    );
    assert!(
        records(&collection, "git.dependency_manifest")
            .iter()
            .any(|r| r.data["payload"]["path"] == "Cargo.lock")
    );
    let diffs = collection
        .observations
        .iter()
        .filter(|r| r.source == "git.commit_diff")
        .map(|r| String::from_utf8_lossy(&r.body))
        .collect::<Vec<_>>();
    assert!(
        diffs
            .iter()
            .any(|patch| patch.contains("Cargo.lock") && patch.contains("+version = 4"))
    );
    let workflows =
        serde_json::to_string(&records(&collection, "git.workflow_definition")).unwrap();
    assert!(workflows.contains("first-fixture"));
    assert!(workflows.contains("second-fixture"));
    assert_eq!(git(path, &["rev-parse", "HEAD"]), second_sha);
    assert!(!path.join("hook-ran").exists());
    assert!(
        collection
            .coverage
            .iter()
            .any(|c| c.source == "git.graph" && c.state == "complete" && c.records == 2)
    );
    let repeated = collect_git(path, "acme/demo").unwrap();
    assert_eq!(
        collection.records.iter().map(|r| &r.id).collect::<Vec<_>>(),
        repeated.records.iter().map(|r| &r.id).collect::<Vec<_>>()
    );
}

#[test]
fn reported_inventory_total_cannot_be_silently_truncated() {
    let server = Server::start(|target, _| {
        if target.split('?').next().unwrap().ends_with("/actions/runs") {
            // A source reporting more records than it actually returns is an incomplete inventory.
            Response::ok(json!({"total_count": 2, "workflow_runs": []}))
        } else {
            default_response(target)
        }
    });
    let collection = collect_github("acme/demo", None, &server.base).unwrap();
    let coverage = collection
        .coverage
        .iter()
        .find(|c| c.source == "github.workflow_run")
        .unwrap();
    assert_eq!(coverage.state, "incomplete");
    assert!(coverage.detail.as_deref().unwrap().contains("total"));
}

#[test]
fn long_rate_limit_delay_stops_requests_and_reports_remaining_scopes() {
    let server = Server::start(|target, _| {
        if target.starts_with("/repos/acme/demo/issues/comments") {
            Response {
                status: 429,
                body: json!({"message":"synthetic rate limit"}),
                link: None,
            }
        } else {
            default_response(target)
        }
    });
    let collection = collect_github("acme/demo", None, &server.base).unwrap();
    assert!(
        collection
            .observations
            .iter()
            .any(|r| r.status == Some(429))
    );
    assert!(
        collection
            .coverage
            .iter()
            .any(|c| c.source == "github.request_budget" && c.state == "incomplete")
    );
    assert!(
        !server
            .requests
            .lock()
            .unwrap()
            .iter()
            .any(|r| r.contains("/actions/runs"))
    );
    assert_eq!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.contains("/issues/comments"))
            .count(),
        1
    );
}

#[test]
fn conditional_refresh_304_keeps_payload_provenance_and_follows_cached_pagination() {
    let server = Server::start_headers(|target, base, request| {
        if request
            .to_ascii_lowercase()
            .contains("if-none-match: \"fixture-v1\"")
        {
            Response {
                status: 304,
                body: Value::Null,
                link: None,
            }
        } else if target.starts_with("/repos/acme/demo/issues/comments") {
            if target.contains("page=2") {
                Response::ok(json!([{"id":2,"body":"second"}]))
            } else {
                Response {
                    status: 200,
                    body: json!([{"id":1,"body":"first"}]),
                    link: Some(format!(
                        "<{base}repos/acme/demo/issues/comments?page=2>; rel=\"next\""
                    )),
                }
            }
        } else {
            default_response(target)
        }
    });
    let cache = tempfile::tempdir().unwrap();
    let first = observatory::collectors::collect_github_cached(
        "acme/demo",
        None,
        &server.base,
        cache.path(),
    )
    .unwrap();
    let second = observatory::collectors::collect_github_cached(
        "acme/demo",
        None,
        &server.base,
        cache.path(),
    )
    .unwrap();
    assert_eq!(
        first.records.iter().map(|r| &r.id).collect::<Vec<_>>(),
        second.records.iter().map(|r| &r.id).collect::<Vec<_>>()
    );
    let cached = second
        .observations
        .iter()
        .filter(|o| o.source == "github.issue_comment.cached_body")
        .collect::<Vec<_>>();
    assert_eq!(cached.len(), 2);
    assert_eq!(
        cached[0].collected_at,
        first
            .observations
            .iter()
            .find(|o| o.source == "github.issue_comment")
            .unwrap()
            .collected_at
    );
    assert!(
        cached
            .iter()
            .all(|o| o.status.is_none() && !o.body.is_empty())
    );
    assert!(
        second
            .observations
            .iter()
            .filter(|o| o.status == Some(304))
            .all(|o| o.body.is_empty())
    );
    assert!(
        second
            .coverage
            .iter()
            .any(|c| c.source == "github.issue_comment" && c.state == "complete" && c.pages == 2)
    );
}

#[test]
fn corrupt_conditional_cache_is_refetched_without_validators() {
    let server = Server::start_headers(|target, _base, request| {
        if request
            .to_ascii_lowercase()
            .contains("if-none-match: \"fixture-v1\"")
        {
            Response {
                status: 304,
                body: Value::Null,
                link: None,
            }
        } else {
            default_response(target)
        }
    });
    let cache = tempfile::tempdir().unwrap();
    observatory::collectors::collect_github_cached("acme/demo", None, &server.base, cache.path())
        .unwrap();
    let mut corrupted_url = None;
    for file in std::fs::read_dir(cache.path()).unwrap() {
        let file = file.unwrap();
        let mut entry: Value =
            serde_json::from_slice(&std::fs::read(file.path()).unwrap()).unwrap();
        if entry["url"].as_str().unwrap().contains("issues/comments") {
            corrupted_url = entry["url"].as_str().map(str::to_owned);
            entry["body"] = json!([123]); // valid cache JSON, invalid/mismatched source bytes
            std::fs::write(file.path(), serde_json::to_vec(&entry).unwrap()).unwrap();
        }
    }
    let second = observatory::collectors::collect_github_cached(
        "acme/demo",
        None,
        &server.base,
        cache.path(),
    )
    .unwrap();
    let url = corrupted_url.unwrap();
    assert!(
        second
            .observations
            .iter()
            .any(|o| o.url == url && o.status == Some(200))
    );
    assert!(
        !second
            .observations
            .iter()
            .any(|o| o.url == url && o.status == Some(304))
    );
    assert!(
        second
            .coverage
            .iter()
            .any(|c| c.source == "github.issue_comment" && c.state == "complete")
    );
}

#[test]
fn conditional_cache_is_separate_for_different_tokens() {
    let server = Server::start_headers(|target, _base, request| {
        if request
            .to_ascii_lowercase()
            .contains("if-none-match: \"fixture-v1\"")
        {
            Response {
                status: 304,
                body: Value::Null,
                link: None,
            }
        } else {
            default_response(target)
        }
    });
    let cache = tempfile::tempdir().unwrap();
    observatory::collectors::collect_github_cached(
        "acme/demo",
        Some("synthetic-account-a"),
        &server.base,
        cache.path(),
    )
    .unwrap();
    let other = observatory::collectors::collect_github_cached(
        "acme/demo",
        Some("synthetic-account-b"),
        &server.base,
        cache.path(),
    )
    .unwrap();
    assert!(other.observations.iter().all(|o| o.status != Some(304)));
    for file in std::fs::read_dir(cache.path()).unwrap() {
        let content = std::fs::read_to_string(file.unwrap().path()).unwrap();
        assert!(
            !content.contains("synthetic-account-a") && !content.contains("synthetic-account-b")
        );
    }
}
