//! Opt-in receipt acquisition and independent workflow reconciliation.
use super::{GithubCollector, MAX_ATTEMPTS, MAX_BODY, MAX_REQUESTS, new_collector};
use crate::collectors::{CollectedRecord, Collection, Coverage, RawResponse, digest, push_record};
use crate::importers::{ImportRecord, executions_import, validate_execution_records};
use crate::model::Evidence;
use anyhow::{Context, Result, bail, ensure};
use chrono::Utc;
use reqwest::Url;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

const ARTIFACT_PREFIX: &str = "observatory-executions-";

fn coverage(c: &mut GithubCollector, source: &str, scope: String, state: &str, detail: &str) {
    c.output.coverage.push(Coverage {
        source: source.into(),
        scope,
        state: state.into(),
        pages: 0,
        records: 0,
        detail: Some(detail.into()),
    });
}

fn loopback(url: &Url) -> bool {
    matches!(
        url.host_str(),
        Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
    )
}

impl GithubCollector {
    /// Only the known API endpoint gets credentials. Signed redirects are neither
    /// authenticated with the GitHub token nor written to retained metadata.
    fn archive(&mut self, artifact_id: u64) -> Result<Vec<u8>> {
        let api_url = self.url(&format!("actions/artifacts/{artifact_id}/zip"))?;
        let mut url = api_url.clone();
        for hop in 0..=3 {
            if self.requests >= MAX_REQUESTS || self.stopped.is_some() {
                self.stopped = Some(format!("request cap ({MAX_REQUESTS}) reached"));
                bail!("artifact request budget exhausted");
            }
            self.requests += 1;
            let mut request = self.client.get(url.clone());
            if hop == 0 {
                request = request
                    .header("Accept", "application/vnd.github+json")
                    .header("X-GitHub-Api-Version", "2022-11-28");
                if let Some(token) = &self.token {
                    request = request.bearer_auth(token);
                }
            }
            let collected_at = Utc::now().to_rfc3339();
            let mut response = match request.send() {
                Ok(response) => response,
                Err(error) => {
                    // reqwest error Display normally includes the signed URL.
                    let message = format!("archive request failed: {}", error.without_url());
                    self.output.observations.push(RawResponse {
                        source: "github.execution_artifact_archive".into(),
                        url: api_url.to_string(),
                        collected_at,
                        status: None,
                        headers: BTreeMap::new(),
                        body: vec![],
                        sha256: digest(&[]),
                        error: Some(message.clone()),
                    });
                    bail!(message);
                }
            };
            let status = response.status();
            let mut headers = BTreeMap::new();
            headers.insert("x-observatory-download-hop".into(), hop.to_string());
            if let Some(content_type) = response.headers().get("content-type")
                && let Ok(content_type) = content_type.to_str()
            {
                headers.insert("content-type".into(), content_type.into());
            }
            let location = response.headers().get("location").cloned();
            let mut body = Vec::new();
            let mut error = None;
            if status.is_success() {
                if response
                    .by_ref()
                    .take(MAX_BODY + 1)
                    .read_to_end(&mut body)
                    .is_err()
                {
                    error = Some("archive response body could not be read completely".into());
                }
                if body.len() as u64 > MAX_BODY {
                    body.truncate(MAX_BODY as usize);
                    error = Some("archive exceeds 16 MiB; retained bytes are truncated".into());
                }
            } else if !status.is_redirection() {
                error = Some(format!(
                    "archive HTTP {}; unavailable, not a measured zero",
                    status.as_u16()
                ));
            }
            self.output.observations.push(RawResponse {
                source: "github.execution_artifact_archive".into(),
                url: api_url.to_string(),
                collected_at,
                status: Some(status.as_u16()),
                headers,
                sha256: digest(&body),
                body: body.clone(),
                error: error.clone(),
            });
            if let Some(error) = error {
                bail!(error);
            }
            if status.is_success() {
                return Ok(body);
            }
            ensure!(
                status.is_redirection() && hop < 3,
                "archive redirect limit reached"
            );
            let target = location
                .as_ref()
                .and_then(|value| value.to_str().ok())
                .context("archive redirect missing location")?;
            // Never include the target in an error, even if URL parsing fails.
            url = url
                .join(target)
                .map_err(|_| anyhow::anyhow!("invalid archive redirect"))?;
            ensure!(
                url.username().is_empty()
                    && url.password().is_none()
                    && (url.scheme() == "https"
                        || (url.scheme() == "http" && loopback(&url) && loopback(&self.base))),
                "archive redirect must use HTTPS without credentials"
            );
        }
        unreachable!()
    }
}

#[derive(Clone)]
struct Job {
    run_id: u64,
    attempt: u64,
    payload: Value,
}

fn job_matches<'a>(jobs: &'a [Job], platform: &Value) -> Vec<&'a Job> {
    let attempt = platform["run_attempt"].as_u64();
    let run_id = platform["run_id"]
        .as_str()
        .and_then(|id| id.parse::<u64>().ok());
    let candidates = jobs
        .iter()
        .filter(|job| Some(job.attempt) == attempt && Some(job.run_id) == run_id);
    if let Some(id) = platform["job_id"].as_str() {
        candidates
            .filter(|job| job.payload["id"].as_u64().map(|v| v.to_string()).as_deref() == Some(id))
            .collect()
    } else if let Some(name) = platform["job_name"]
        .as_str()
        .filter(|name| *name != "unknown")
    {
        candidates
            .filter(|job| job.payload["name"].as_str() == Some(name))
            .collect()
    } else {
        vec![]
    }
}

/// Validate identities against independently downloaded metadata, not artifact
/// names or the credential's account. Unavailable identities remain unresolved.
fn validate_archive_identity(
    records: &[ImportRecord],
    repository_id: Option<u64>,
    run_id: u64,
    latest_attempt: u64,
    attempts: &BTreeMap<u64, Value>,
) -> Result<()> {
    for record in records {
        let platform = &record.data["payload"]["platform"];
        if platform["repository_id"] != "unknown"
            && let Some(repository_id) = repository_id
        {
            ensure!(
                platform["repository_id"].as_str() == Some(repository_id.to_string().as_str()),
                "receipt repository identity does not match independent GitHub metadata"
            );
        }
        if platform["run_id"] != "unknown" {
            ensure!(
                platform["run_id"].as_str() == Some(run_id.to_string().as_str()),
                "receipt run identity does not match independent GitHub metadata"
            );
        }
        // Names can change while repository IDs remain stable. Retain both labels;
        // acquisition provenance below records the current API repository name.
        let Some(attempt) = platform["run_attempt"].as_u64() else {
            continue;
        };
        ensure!(
            attempt <= latest_attempt,
            "receipt attempt exceeds independently observed run attempt count"
        );
        let Some(metadata) = attempts.get(&attempt) else {
            continue;
        };
        for (receipt_field, api_value) in [
            ("actor", &metadata["actor"]["login"]),
            ("triggering_actor", &metadata["triggering_actor"]["login"]),
            ("event_name", &metadata["event"]),
        ] {
            if let Some(observed) = api_value.as_str()
                && platform[receipt_field] != "unknown"
            {
                ensure!(
                    platform[receipt_field].as_str() == Some(observed),
                    "receipt {receipt_field} conflicts with independent GitHub metadata"
                );
            }
        }
    }
    Ok(())
}

fn reconcile(c: &mut GithubCollector, run_id: u64, jobs: &[Job], records: &[ImportRecord]) {
    let mut invocations: BTreeMap<String, Vec<&ImportRecord>> = BTreeMap::new();
    let mut instrumented_jobs = BTreeSet::new();
    for record in records {
        if record.data["source"] == "observatory.execution_bundle" {
            let matches = job_matches(jobs, &record.data["payload"]["platform"]);
            if matches.len() == 1
                && let Some(id) = matches[0].payload["id"].as_u64()
            {
                instrumented_jobs.insert((matches[0].attempt, id));
            }
            continue;
        }
        // Use exactly the immutable receipt scope. Unknown platform identities
        // must not be silently joined to a later, more specific declaration.
        let key = record
            .id
            .rsplit_once('/')
            .map_or(record.id.as_str(), |(scope, _)| scope)
            .to_owned();
        invocations.entry(key).or_default().push(record);
    }
    let mut matched_jobs: BTreeMap<(u64, u64), BTreeSet<String>> = BTreeMap::new();
    let mut unresolved_attempts = BTreeSet::new();
    let mut unresolved_run = false;
    for (key, records) in invocations {
        let receipt = &records[0].data["payload"];
        let platform = &receipt["platform"];
        let matches = job_matches(jobs, platform);
        let phases: BTreeSet<_> = records
            .iter()
            .filter_map(|r| r.data["payload"]["phase"].as_str())
            .collect();
        let lifecycle = match (phases.contains("start"), phases.contains("completion")) {
            (true, true) => "start_and_completion_observed",
            (true, false) => "start_only",
            (false, true) => "completion_only",
            _ => "unknown",
        };
        let linked_job = if matches.len() == 1 {
            Some(matches[0])
        } else {
            None
        };
        let job_id = linked_job.and_then(|job| job.payload["id"].as_u64());
        if let (Some(job), Some(id)) = (linked_job, job_id) {
            matched_jobs
                .entry((job.attempt, id))
                .or_default()
                .insert(key.clone());
        } else if let Some(attempt) = platform["run_attempt"].as_u64() {
            unresolved_attempts.insert(attempt);
        } else {
            unresolved_run = true;
        }
        push_record(
            &mut c.output,
            &c.namespace,
            "github.execution_reconciliation",
            &format!("run:{run_id}/invocation:{key}"),
            json!({
                "schema":"observatory.execution-reconciliation/1",
                "source":"github_api_comparison",
                "run_id":run_id,
                "run_attempt":platform["run_attempt"],
                "invocation_id":receipt["invocation_id"],
                "receipt_ids":records.iter().map(|record| &record.id).collect::<BTreeSet<_>>(),
                "lifecycle":lifecycle,
                "job_link":if job_id.is_some() {"unambiguous"} else {"unresolved"},
                "job_id":job_id,
                "job_status":linked_job.map(|job| &job.payload["status"]),
                "job_conclusion":linked_job.map(|job| &job.payload["conclusion"]),
                "limitations":"Schema and hash validation do not authenticate caller declarations. Missing phases do not establish tool outcome."
            }),
        );
    }
    for job in jobs {
        let id = job.payload["id"].as_u64();
        let count = id
            .and_then(|id| matched_jobs.get(&(job.attempt, id)))
            .map_or(0, BTreeSet::len);
        let observation = if count > 0 {
            "receipts_observed"
        } else if id.is_some_and(|id| instrumented_jobs.contains(&(job.attempt, id))) {
            "instrumented_no_receipts"
        } else if unresolved_run || unresolved_attempts.contains(&job.attempt) {
            "unresolved_receipts_present"
        } else {
            "no_receipts_observed"
        };
        push_record(
            &mut c.output,
            &c.namespace,
            "github.execution_job_reconciliation",
            &format!(
                "run:{}/attempt:{}/job:{}",
                job.run_id,
                job.attempt,
                id.map_or("unknown".into(), |id| id.to_string())
            ),
            json!({
                "schema":"observatory.execution-job-reconciliation/1",
                "source":"github_api_comparison",
                "run_id":job.run_id,"run_attempt":job.attempt,"job_id":id,
                "job_name":job.payload["name"],"status":job.payload["status"],"conclusion":job.payload["conclusion"],
                "receipt_observation":observation,"linked_invocations":count,
                "limitations":"No receipts can mean uninstrumented work or a missing upload. Observed receipts do not establish complete tool coverage, human effort, acceptance or retained quality."
            }),
        );
    }
}

/// Download allowlisted evidence artifacts and retain all visible run attempts
/// and jobs, including failed/cancelled/skipped executions. Use `run_id` for a
/// workflow_run observer; omit it for durable historical inventory collection.
/// Collection is read-only and safe to repeat; receipt identity survives upload.
pub fn collect_github_executions(
    repo: &str,
    token: Option<&str>,
    api_base: &str,
    run_id: Option<u64>,
) -> Result<Collection> {
    ensure!(run_id != Some(0), "run ID must be positive");
    let mut c = new_collector(repo, token, api_base, None)?;
    let mut repository_id = None;
    let mut repository_name = repo.to_owned();
    if let Some(metadata) = c.singleton("github.repository", "") {
        repository_id = metadata["id"].as_u64();
        if let Some(id) = repository_id {
            c.namespace = format!("github:repository:{id}");
        }
        if let Some(name) = metadata["full_name"].as_str() {
            repository_name = name.into();
        }
        push_record(
            &mut c.output,
            &c.namespace,
            "github.repository",
            "repository",
            metadata,
        );
    }
    let runs = if let Some(run_id) = run_id {
        c.singleton("github.workflow_run", &format!("actions/runs/{run_id}"))
            .into_iter()
            .collect()
    } else {
        c.list("github.workflow_run", "actions/runs", Some("workflow_runs"))
    };
    c.retain("github.workflow_run", "", &runs);
    for run in runs {
        if c.stopped.is_some() {
            break;
        }
        let Some(id) = run["id"].as_u64().filter(|id| *id > 0) else {
            coverage(
                &mut c,
                "github.workflow_run",
                repo.into(),
                "incomplete",
                "run has no valid numeric identity",
            );
            continue;
        };
        if run_id.is_some_and(|expected| expected != id) {
            coverage(
                &mut c,
                "github.workflow_run",
                repo.into(),
                "incomplete",
                "requested run ID differs from returned metadata",
            );
            continue;
        }
        let Some(latest_attempt) = run["run_attempt"].as_u64().filter(|attempt| *attempt > 0)
        else {
            coverage(
                &mut c,
                "github.workflow_attempt",
                format!("run:{id}"),
                "incomplete",
                "run has no valid attempt count",
            );
            continue;
        };
        let mut attempts = BTreeMap::new();
        let mut jobs = vec![];
        for attempt in 1..=latest_attempt.min(MAX_ATTEMPTS) {
            let prefix = format!("run:{id}/attempt:{attempt}/");
            if let Some(detail) = c.singleton(
                "github.workflow_attempt",
                &format!("actions/runs/{id}/attempts/{attempt}"),
            ) {
                if detail["id"].as_u64() == Some(id)
                    && detail["run_attempt"].as_u64() == Some(attempt)
                {
                    attempts.insert(attempt, detail.clone());
                } else {
                    coverage(
                        &mut c,
                        "github.workflow_attempt",
                        prefix.clone(),
                        "incomplete",
                        "attempt metadata identity does not match its endpoint",
                    );
                }
                push_record(
                    &mut c.output,
                    &c.namespace,
                    "github.workflow_attempt",
                    &prefix,
                    detail,
                );
            }
            let values = c.list(
                "github.workflow_job",
                &format!("actions/runs/{id}/attempts/{attempt}/jobs"),
                Some("jobs"),
            );
            c.retain("github.workflow_job", &prefix, &values);
            for payload in values {
                if payload["id"].as_u64().is_some_and(|id| id > 0)
                    && payload["run_id"].as_u64() == Some(id)
                    && payload["run_attempt"].as_u64() == Some(attempt)
                {
                    jobs.push(Job {
                        run_id: id,
                        attempt,
                        payload,
                    });
                } else {
                    coverage(
                        &mut c,
                        "github.workflow_job",
                        prefix.clone(),
                        "incomplete",
                        "job metadata identity does not match its attempt endpoint",
                    );
                }
            }
        }
        if latest_attempt > MAX_ATTEMPTS {
            coverage(
                &mut c,
                "github.workflow_attempt",
                format!("run:{id}"),
                "incomplete",
                "attempt collection cap reached",
            );
        }
        let artifacts = c.list(
            "github.execution_artifact",
            &format!("actions/runs/{id}/artifacts"),
            Some("artifacts"),
        );
        // Inventory metadata is retained, but arbitrary binary artifacts are never fetched.
        c.retain(
            "github.execution_artifact",
            &format!("run:{id}/"),
            &artifacts,
        );
        let mut records = Vec::new();
        for artifact in artifacts {
            if !artifact["name"]
                .as_str()
                .is_some_and(|name| name.starts_with(ARTIFACT_PREFIX))
            {
                continue;
            }
            let Some(artifact_id) = artifact["id"].as_u64().filter(|id| *id > 0) else {
                coverage(
                    &mut c,
                    "github.execution_artifact_bundle",
                    format!("run:{id}"),
                    "incomplete",
                    "allowlisted artifact lacks a valid ID",
                );
                continue;
            };
            let scope = format!("run:{id}/artifact:{artifact_id}");
            let result = (|| -> Result<Vec<ImportRecord>> {
                ensure!(
                    artifact["expired"] == false,
                    "artifact is expired or availability is unknown"
                );
                ensure!(
                    artifact["size_in_bytes"]
                        .as_u64()
                        .is_some_and(|size| size <= MAX_BODY),
                    "artifact size missing or above 16 MiB limit"
                );
                if let Some(artifact_run) = artifact["workflow_run"]["id"].as_u64() {
                    ensure!(
                        artifact_run == id,
                        "artifact inventory run identity mismatch"
                    );
                }
                let archive = c.archive(artifact_id)?;
                if let Some(expected) = artifact["digest"].as_str() {
                    ensure!(
                        expected == format!("sha256:{}", digest(&archive)),
                        "archive digest differs from GitHub artifact metadata"
                    );
                }
                let imported = executions_import(&archive)?;
                validate_archive_identity(&imported, repository_id, id, latest_attempt, &attempts)?;
                let combined = records
                    .iter()
                    .chain(imported.iter())
                    .map(|record: &ImportRecord| Evidence {
                        id: record.id.clone(),
                        kind: record.kind.clone(),
                        data: record.data.clone(),
                    })
                    .collect::<Vec<_>>();
                validate_execution_records(&combined)?;
                Ok(imported)
            })();
            match result {
                Ok(imported) => {
                    c.output.coverage.push(Coverage {source:"github.execution_artifact_bundle".into(),scope,state:"complete".into(),pages:1,records:imported.len(),detail:Some("Validated schema and receipt hashes; known platform fields compared with available API metadata. Unknown provenance and controller declarations remain unverified".into())});
                    push_record(
                        &mut c.output,
                        &c.namespace,
                        "github.execution_artifact_delivery",
                        &format!("run:{id}/artifact:{artifact_id}"),
                        json!({
                            "schema":"observatory.execution-artifact-delivery/1", "source":"github_api",
                            "repository_id":repository_id,"repository":repository_name,"run_id":id,"artifact_id":artifact_id,
                            "record_ids":imported.iter().map(|record| &record.id).collect::<Vec<_>>(),
                            "identity_comparison":"Known receipt repository/run/actor fields checked when API metadata was available; unknown fields remain unresolved. Acquisition context does not fill missing receipt provenance."
                        }),
                    );
                    records.extend(imported);
                }
                Err(error) => coverage(
                    &mut c,
                    "github.execution_artifact_bundle",
                    scope,
                    "incomplete",
                    &error.to_string(),
                ),
            }
        }
        reconcile(&mut c, id, &jobs, &records);
        c.output
            .records
            .extend(records.into_iter().map(|record| CollectedRecord {
                id: record.id,
                kind: record.kind,
                data: record.data,
            }));
    }
    if let Some(reason) = c.stopped.clone() {
        coverage(
            &mut c,
            "github.request_budget",
            repo.into(),
            "incomplete",
            &reason,
        );
    }
    coverage(
        &mut c,
        "github.history",
        repo.into(),
        "not_collected",
        "Deleted runs and expired artifacts cannot be reconstructed. No receipts do not establish zero executions; runner loss may prevent every upload.",
    );
    Ok(c.output)
}
