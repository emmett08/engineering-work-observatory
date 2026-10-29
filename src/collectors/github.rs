//! GitHub HTTP enumeration, pagination, conditional refresh and retrieval coverage.
use super::{Collection, Coverage, RawResponse, digest, push_record};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::{Url, redirect::Policy};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_PAGES: usize = 250;
const MAX_REQUESTS: usize = 8_000;
const MAX_BODY: u64 = 16 * 1024 * 1024;
const MAX_ATTEMPTS: u64 = 50;

struct GithubCollector {
    client: Client,
    base: Url,
    token: Option<String>,
    repository: String,
    namespace: String,
    requests: usize,
    stopped: Option<String>,
    cache: Option<HttpCache>,
    output: Collection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    version: u32,
    url: String,
    collected_at: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
    sha256: String,
}

struct HttpCache {
    root: PathBuf,
    scope: String,
}

impl HttpCache {
    fn path(&self, url: &Url) -> PathBuf {
        self.root.join(format!(
            "{}.json",
            digest(format!("{}\n{url}", self.scope).as_bytes())
        ))
    }

    fn load(&self, url: &Url) -> Option<CacheEntry> {
        let path = self.path(url);
        // Reject oversized or malformed entries instead of trusting stale/truncated bytes.
        if std::fs::metadata(&path).ok()?.len() > MAX_BODY * 5 + 16_384 {
            return None;
        }
        let entry: CacheEntry = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
        if entry.version != 1
            || entry.url != url.as_str()
            || entry.body.len() as u64 > MAX_BODY
            || digest(&entry.body) != entry.sha256
            || serde_json::from_slice::<Value>(&entry.body).is_err()
        {
            return None;
        }
        Some(entry)
    }

    fn save(&self, url: &Url, entry: &CacheEntry) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root)?;
        serde_json::to_writer(&mut temporary, entry)?;
        temporary.flush()?;
        temporary.as_file().sync_all()?;
        temporary.persist(self.path(url))?;
        Ok(())
    }
}

fn same_origin(a: &Url, b: &Url) -> bool {
    a.scheme() == b.scheme()
        && a.host_str() == b.host_str()
        && a.port_or_known_default() == b.port_or_known_default()
        && b.username().is_empty()
        && b.password().is_none()
}

fn next_link(header: &str) -> Result<Option<String>> {
    for part in header.split(',') {
        let mut fields = part.trim().split(';');
        let target = fields.next().unwrap_or("").trim();
        if fields.any(|f| {
            f.trim()
                .strip_prefix("rel=")
                .map(|v| v.trim_matches('"').split_whitespace().any(|r| r == "next"))
                .unwrap_or(false)
        }) {
            if !target.starts_with('<') || !target.ends_with('>') {
                bail!("malformed next pagination link");
            }
            return Ok(Some(target[1..target.len() - 1].to_owned()));
        }
    }
    Ok(None)
}

impl GithubCollector {
    fn url(&self, suffix: &str) -> Result<Url> {
        let path = if suffix.is_empty() {
            format!("repos/{}", self.repository)
        } else {
            format!("repos/{}/{}", self.repository, suffix)
        };
        self.base.join(&path).context("endpoint URL")
    }

    fn request(&mut self, source: &str, url: &Url) -> Result<(Value, Option<String>)> {
        if let Some(reason) = &self.stopped {
            bail!("collection stopped: {reason}");
        }
        if !same_origin(&self.base, url) {
            bail!("pagination target has an untrusted origin");
        }
        let cached = self.cache.as_ref().and_then(|cache| cache.load(url));
        for attempt in 0..3 {
            if self.requests >= MAX_REQUESTS {
                self.stopped = Some(format!("request cap ({MAX_REQUESTS}) reached"));
                bail!(
                    "collection request cap reached ({MAX_REQUESTS}); remaining scopes were not collected"
                );
            }
            self.requests += 1;
            let collected_at = Utc::now().to_rfc3339();
            let mut req = self
                .client
                .get(url.clone())
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28");
            if let Some(token) = &self.token {
                req = req.bearer_auth(token);
            }
            let mut conditional = false;
            if let Some(entry) = &cached {
                for (cached_name, request_name) in [
                    ("etag", "If-None-Match"),
                    ("last-modified", "If-Modified-Since"),
                ] {
                    if let Some(value) = entry.headers.get(cached_name) {
                        req = req.header(request_name, value);
                        conditional = true;
                    }
                }
            }
            let mut response = match req.send() {
                Ok(response) => response,
                Err(error) => {
                    let message = format!("request failed: {error}");
                    self.output.observations.push(RawResponse {
                        source: source.into(),
                        url: url.to_string(),
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
            let status = response.status().as_u16();
            let mut headers = BTreeMap::new();
            for name in [
                "link",
                "etag",
                "last-modified",
                "retry-after",
                "x-ratelimit-remaining",
                "x-ratelimit-reset",
                "x-github-request-id",
                "content-type",
            ] {
                if let Some(value) = response.headers().get(name).and_then(|v| v.to_str().ok()) {
                    headers.insert(name.to_owned(), value.to_owned());
                }
            }
            let mut body = vec![];
            let read_error = response
                .by_ref()
                .take(MAX_BODY + 1)
                .read_to_end(&mut body)
                .err()
                .map(|e| e.to_string());
            let error = if body.len() as u64 > MAX_BODY {
                body.truncate(MAX_BODY as usize);
                Some("response exceeded 16 MiB; retained body is truncated".to_owned())
            } else {
                read_error
            };
            self.output.observations.push(RawResponse {
                source: source.into(),
                url: url.to_string(),
                collected_at: collected_at.clone(),
                status: Some(status),
                headers: headers.clone(),
                sha256: digest(&body),
                body: body.clone(),
                error: error.clone(),
            });
            if let Some(error) = error {
                bail!(error);
            }
            if status == 304 {
                let Some(entry) = &cached else {
                    bail!("HTTP 304 without a validated cached body");
                };
                if !conditional {
                    bail!("HTTP 304 received without a conditional request");
                }
                let mut provenance = entry.headers.clone();
                provenance.insert(
                    "x-observatory-cache-source-collected-at".into(),
                    entry.collected_at.clone(),
                );
                provenance.insert(
                    "x-observatory-cache-body-sha256".into(),
                    entry.sha256.clone(),
                );
                provenance.insert("x-observatory-revalidated-at".into(), collected_at.clone());
                provenance.insert("x-observatory-revalidated-http-status".into(), "304".into());
                // This is cached source evidence, not a new 200 response or newly occurring source event.
                self.output.observations.push(RawResponse {
                    source: format!("{source}.cached_body"),
                    url: url.to_string(),
                    collected_at: entry.collected_at.clone(),
                    status: None,
                    headers: provenance,
                    body: entry.body.clone(),
                    sha256: entry.sha256.clone(),
                    error: None,
                });
                return Ok((
                    serde_json::from_slice(&entry.body)?,
                    headers
                        .get("link")
                        .or_else(|| entry.headers.get("link"))
                        .cloned(),
                ));
            }
            let exhausted = status == 403
                && headers
                    .get("x-ratelimit-remaining")
                    .map(|s| s == "0")
                    .unwrap_or(false);
            if status == 429 || exhausted || matches!(status, 502..=504) {
                // Respect a bounded server delay. Long waits are surfaced as incomplete collection.
                let retry_after = headers.get("retry-after").map(|s| {
                    s.parse::<u64>()
                        .ok()
                        .or_else(|| {
                            chrono::DateTime::parse_from_rfc2822(s)
                                .ok()
                                .map(|t| (t.timestamp() - Utc::now().timestamp()).max(0) as u64)
                        })
                        .unwrap_or(u64::MAX)
                });
                if attempt < 2 && !exhausted && retry_after.unwrap_or(1) <= 5 {
                    std::thread::sleep(Duration::from_secs(retry_after.unwrap_or(1)));
                    continue;
                }
                if exhausted || status == 429 {
                    self.stopped = Some(format!("HTTP {status}; rate limit reached"));
                }
                bail!("HTTP {status}; rate limit/server failure; retry collection later");
            }
            if !(200..300).contains(&status) {
                bail!("HTTP {status}; source unavailable (not a measured zero)");
            }
            let data = serde_json::from_slice(&body).context("response is not valid JSON")?;
            if let Some(cache) = &self.cache {
                let entry = CacheEntry {
                    version: 1,
                    url: url.to_string(),
                    collected_at,
                    headers: headers.clone(),
                    sha256: digest(&body),
                    body,
                };
                if let Err(error) = cache.save(url, &entry) {
                    self.output.coverage.push(Coverage {source:"github.cache".into(),scope:url.to_string(),state:"request_failed".into(),pages:0,records:0,detail:Some(format!("response collected but conditional cache could not be persisted: {error}"))});
                }
            }
            return Ok((data, headers.get("link").cloned()));
        }
        unreachable!()
    }

    fn singleton(&mut self, source: &str, suffix: &str) -> Option<Value> {
        let url = match self.url(suffix) {
            Ok(u) => u,
            Err(_) => return None,
        };
        let result = self.request(source, &url);
        match result {
            Ok((value, _)) => {
                self.output.coverage.push(Coverage {
                    source: source.into(),
                    scope: url.to_string(),
                    state: "complete".into(),
                    pages: 1,
                    records: 1,
                    detail: None,
                });
                Some(value)
            }
            Err(e) => {
                self.output.coverage.push(Coverage {
                    source: source.into(),
                    scope: url.to_string(),
                    state: "request_failed".into(),
                    pages: 0,
                    records: 0,
                    detail: Some(e.to_string()),
                });
                None
            }
        }
    }

    fn list(&mut self, source: &str, suffix: &str, array: Option<&str>) -> Vec<Value> {
        let mut url = match self.url(suffix) {
            Ok(u) => u,
            Err(_) => return vec![],
        };
        url.query_pairs_mut().append_pair("per_page", "100");
        let scope = url.to_string();
        let mut seen = BTreeSet::new();
        let mut records = vec![];
        let mut pages = 0;
        let mut detail = None;
        let mut complete = false;
        let mut reported_total = None;
        loop {
            if !seen.insert(url.to_string()) {
                detail = Some("pagination cycle detected".into());
                break;
            }
            match self.request(source, &url) {
                Ok((value, link)) => {
                    let items = array
                        .and_then(|k| value.get(k))
                        .unwrap_or(&value)
                        .as_array();
                    let Some(items) = items else {
                        detail = Some("expected an array in response".into());
                        break;
                    };
                    pages += 1;
                    reported_total =
                        reported_total.or_else(|| value.get("total_count").and_then(Value::as_u64));
                    records.extend(items.iter().cloned());
                    let next = match link.as_deref().map(next_link).transpose() {
                        Ok(next) => next.flatten(),
                        Err(e) => {
                            detail = Some(e.to_string());
                            break;
                        }
                    };
                    match next {
                        None => {
                            complete = true;
                            break;
                        }
                        Some(target) => {
                            let next_url = match url.join(&target) {
                                Ok(u) => u,
                                Err(e) => {
                                    detail = Some(format!("bad next URL: {e}"));
                                    break;
                                }
                            };
                            if !same_origin(&self.base, &next_url) {
                                detail = Some("refused pagination to an untrusted origin".into());
                                break;
                            }
                            if pages >= MAX_PAGES {
                                detail = Some(format!("pagination cap reached ({MAX_PAGES})"));
                                break;
                            }
                            url = next_url;
                        }
                    }
                }
                Err(e) => {
                    detail = Some(e.to_string());
                    break;
                }
            }
        }
        if complete
            && reported_total
                .map(|n| n > records.len() as u64)
                .unwrap_or(false)
        {
            complete = false;
            detail = Some(format!(
                "reported total {} exceeds {} returned records",
                reported_total.unwrap(),
                records.len()
            ));
        }
        self.output.coverage.push(Coverage {
            source: source.into(),
            scope,
            state: if complete {
                "complete"
            } else if pages == 0 {
                "request_failed"
            } else {
                "incomplete"
            }
            .into(),
            pages,
            records: records.len(),
            detail,
        });
        records
    }

    fn retain(&mut self, family: &str, prefix: &str, items: &[Value]) {
        for item in items {
            let key = item
                .get("id")
                .or_else(|| item.get("filename"))
                .or_else(|| item.get("sha"))
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    v => v.to_string(),
                })
                .unwrap_or_else(|| digest(&serde_json::to_vec(item).unwrap()));
            push_record(
                &mut self.output,
                &self.namespace,
                family,
                &format!("{prefix}{key}"),
                item.clone(),
            );
        }
    }
}

/// Backfill all exposed inventories. API errors remain evidence and coverage entries.
/// Re-run safely: record identities are content-addressed; this collector never mutates GitHub.
pub fn collect_github(repo: &str, token: Option<&str>, api_base: &str) -> Result<Collection> {
    collect_github_impl(repo, token, api_base, None)
}

/// Revalidate every enumerated page using a persistent, integrity-checked conditional cache.
/// Cache keys include the token digest to prevent reuse across accounts or token rotations.
/// This saves transfer; it does not skip pages using lossy updated-at filters.
pub fn collect_github_cached(
    repo: &str,
    token: Option<&str>,
    api_base: &str,
    cache_dir: &Path,
) -> Result<Collection> {
    collect_github_impl(repo, token, api_base, Some(cache_dir))
}

fn collect_github_impl(
    repo: &str,
    token: Option<&str>,
    api_base: &str,
    cache_dir: Option<&Path>,
) -> Result<Collection> {
    let parts: Vec<_> = repo.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|s| {
            s.is_empty()
                || *s == "."
                || *s == ".."
                || !s
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        })
    {
        bail!("repository must be owner/name using GitHub name characters");
    }
    let mut base = Url::parse(api_base).context("invalid GitHub API base URL")?;
    if !matches!(base.scheme(), "http" | "https")
        || !base.username().is_empty()
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        bail!("API base must be an http(s) origin/path without credentials, query or fragment");
    }
    if token.is_some()
        && base.scheme() == "http"
        && !matches!(
            base.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
        )
    {
        bail!("authenticated HTTP is allowed only on loopback; use HTTPS for remote APIs");
    }
    if !base.path().ends_with('/') {
        base.set_path(&format!("{}/", base.path()));
    }
    let client = Client::builder()
        .user_agent("engineering-work-observatory/0.1")
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .redirect(Policy::none())
        .build()?;
    let mut c = GithubCollector {
        client,
        base,
        token: token.map(str::to_owned),
        repository: repo.into(),
        namespace: format!("github:{repo}"),
        requests: 0,
        stopped: None,
        cache: cache_dir.map(|root| HttpCache {
            root: root.to_owned(),
            scope: digest(
                &serde_json::to_vec(&(api_base, repo, token.unwrap_or("")))
                    .expect("strings serialize"),
            ),
        }),
        output: Collection::default(),
    };
    if let Some(metadata) = c.singleton("github.repository", "") {
        if let Some(id) = metadata.get("id").and_then(Value::as_u64) {
            c.namespace = format!("github:repository:{id}");
        }
        push_record(
            &mut c.output,
            &c.namespace,
            "github.repository",
            "repository",
            metadata,
        );
    }
    let pulls = c.list(
        "github.pull_request",
        "pulls?state=all&sort=created&direction=asc",
        None,
    );
    c.retain("github.pull_request", "", &pulls);
    let mut shas = BTreeSet::new();
    let commits = c.list("github.commit", "commits", None);
    for commit in &commits {
        if let Some(sha) = commit.get("sha").and_then(Value::as_str) {
            shas.insert(sha.to_owned());
        }
    }
    c.retain("github.commit", "", &commits);
    for pr in pulls {
        if c.stopped.is_some() {
            break;
        }
        let Some(number) = pr.get("number").and_then(Value::as_u64) else {
            continue;
        };
        let mut reported_commits = None;
        let mut reported_files = None;
        // The detail endpoint contains merge and precise head/base metadata absent in some lists.
        if let Some(detail) = c.singleton("github.pull_request_detail", &format!("pulls/{number}"))
        {
            reported_commits = detail.get("commits").and_then(Value::as_u64);
            reported_files = detail.get("changed_files").and_then(Value::as_u64);
            for pointer in ["/head/sha", "/base/sha", "/merge_commit_sha"] {
                if let Some(sha) = detail.pointer(pointer).and_then(Value::as_str) {
                    shas.insert(sha.to_owned());
                }
            }
            push_record(
                &mut c.output,
                &c.namespace,
                "github.pull_request_detail",
                &number.to_string(),
                detail,
            );
        }
        for (family, path) in [
            ("github.pull_request_commit", "commits"),
            ("github.pull_request_file", "files"),
            ("github.review", "reviews"),
            ("github.review_comment", "comments"),
        ] {
            let items = c.list(family, &format!("pulls/{number}/{path}"), None);
            let expected = match path {
                "commits" => reported_commits,
                "files" => reported_files,
                _ => None,
            };
            if expected.map(|n| n > items.len() as u64).unwrap_or(false)
                && let Some(coverage) = c.output.coverage.last_mut()
            {
                coverage.state = "incomplete".into();
                coverage.detail = Some(format!(
                    "PR detail reports {} {path}, but {} were returned; API truncation or concurrent edits remain possible",
                    expected.unwrap(),
                    items.len()
                ));
            }
            if path == "commits" {
                for item in &items {
                    if let Some(sha) = item.get("sha").and_then(Value::as_str) {
                        shas.insert(sha.to_owned());
                    }
                }
            }
            c.retain(family, &format!("pr:{number}/"), &items);
        }
    }
    // Repository-wide endpoints cover ordinary issues and PR conversation comments, independently paginated.
    let issues = c.list(
        "github.issue",
        "issues?state=all&sort=created&direction=asc",
        None,
    );
    c.retain("github.issue", "", &issues);
    let comments = c.list(
        "github.issue_comment",
        "issues/comments?sort=created&direction=asc",
        None,
    );
    c.retain("github.issue_comment", "", &comments);
    let runs = c.list("github.workflow_run", "actions/runs", Some("workflow_runs"));
    c.retain("github.workflow_run", "", &runs);
    for run in runs {
        if c.stopped.is_some() {
            break;
        }
        let Some(id) = run.get("id").and_then(Value::as_u64) else {
            continue;
        };
        if let Some(sha) = run.get("head_sha").and_then(Value::as_str) {
            shas.insert(sha.to_owned());
        }
        let attempts = run.get("run_attempt").and_then(Value::as_u64).unwrap_or(1);
        for attempt in 1..=attempts.min(MAX_ATTEMPTS) {
            if c.stopped.is_some() {
                break;
            }
            let key = format!("run:{id}/attempt:{attempt}/");
            if let Some(detail) = c.singleton(
                "github.workflow_attempt",
                &format!("actions/runs/{id}/attempts/{attempt}"),
            ) {
                push_record(
                    &mut c.output,
                    &c.namespace,
                    "github.workflow_attempt",
                    &key,
                    detail,
                );
            }
            let jobs = c.list(
                "github.workflow_job",
                &format!("actions/runs/{id}/attempts/{attempt}/jobs"),
                Some("jobs"),
            );
            c.retain("github.workflow_job", &key, &jobs);
        }
        if attempts > MAX_ATTEMPTS {
            c.output.coverage.push(Coverage {
                source: "github.workflow_attempt".into(),
                scope: format!("run:{id}"),
                state: "incomplete".into(),
                pages: 0,
                records: MAX_ATTEMPTS as usize,
                detail: Some(format!(
                    "run has {attempts} attempts; collection cap is {MAX_ATTEMPTS}"
                )),
            });
        }
    }
    for sha in shas {
        if c.stopped.is_some() {
            break;
        }
        for (family, path, array) in [
            (
                "github.check_run",
                "check-runs?filter=all",
                Some("check_runs"),
            ),
            ("github.commit_status", "statuses", None),
            ("github.commit_pull_request", "pulls", None),
        ] {
            let items = c.list(family, &format!("commits/{sha}/{path}"), array);
            c.retain(family, &format!("commit:{sha}/"), &items);
        }
    }
    if let Some(reason) = c.stopped {
        c.output.coverage.push(Coverage {source:"github.request_budget".into(),scope:repo.into(),state:"incomplete".into(),pages:0,records:c.requests,detail:Some(format!("{reason}; remaining nested PR, workflow-attempt and commit-check scopes were not collected"))});
    }
    c.output.coverage.push(Coverage { source: "github.workflow_logs".into(), scope: repo.into(), state: "not_collected".into(), pages: 0, records: 0, detail: Some("Logs and binary workflow artifacts are not collected; their retention is independent of run metadata".into()) });
    c.output.coverage.push(Coverage { source: "github.history".into(), scope: repo.into(), state: "not_collected".into(), pages: 0, records: 0, detail: Some("Current API inventories are not an historical event archive. Deleted objects, expired logs and past edits are not reconstructed".into()) });
    Ok(c.output)
}
