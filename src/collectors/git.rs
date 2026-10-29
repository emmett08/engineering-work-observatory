//! Read-only local Git graph, patch, dependency and workflow-definition evidence.
use super::{Collection, Coverage, RawResponse, digest, push_record};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

fn git(path: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("--no-pager")
        .arg("-C")
        .arg(path)
        .args([
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "diff.external=",
            "-c",
            "color.ui=false",
            "-c",
            "diff.algorithm=myers",
            "-c",
            "diff.renames=false",
        ])
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .output()
        .context("running read-only Git command")?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.stdout)
}

fn git_failure(out: &mut Collection, repo: &str, source: &str, scope: &str, error: &anyhow::Error) {
    let message = error.to_string();
    out.observations.push(RawResponse {
        source: source.into(),
        url: format!("git:{repo}/{scope}"),
        collected_at: Utc::now().to_rfc3339(),
        status: None,
        headers: BTreeMap::new(),
        sha256: digest(&[]),
        body: vec![],
        error: Some(message.clone()),
    });
    out.coverage.push(Coverage {
        source: source.into(),
        scope: scope.into(),
        state: "request_failed".into(),
        pages: 0,
        records: 0,
        detail: Some(message),
    });
}

/// Read a local repository's reachable graph and current refs. Does not fetch, checkout, or execute hooks.
pub fn collect_git(path: &Path, repo: &str) -> Result<Collection> {
    git(path, &["rev-parse", "--git-dir"])?;
    let mut out = Collection::default();
    let namespace = format!("git:{repo}");
    let collected_at = Utc::now().to_rfc3339();
    let refs = git(
        path,
        &[
            "for-each-ref",
            "--format=%(refname)%00%(objectname)%00%(objecttype)",
        ],
    )?;
    let refs_text = String::from_utf8(refs.clone()).context("Git ref output must be UTF-8")?;
    let mut reference_shas = BTreeSet::new();
    let mut ref_count = 0;
    for line in refs_text.lines() {
        let fields: Vec<_> = line.split('\0').collect();
        if fields.len() != 3 {
            continue;
        }
        ref_count += 1;
        reference_shas.insert(fields[1].to_string());
        push_record(
            &mut out,
            &namespace,
            "git.ref",
            fields[0],
            json!({"ref": fields[0], "object_id": fields[1], "object_type": fields[2]}),
        );
    }
    // HEAD may be detached, so retain it and include its reachable graph explicitly.
    let head = git(path, &["rev-parse", "--verify", "HEAD"]);
    if let Ok(head) = &head {
        let sha = String::from_utf8_lossy(head).trim().to_string();
        reference_shas.insert(sha.clone());
        push_record(
            &mut out,
            &namespace,
            "git.head",
            "HEAD",
            json!({"object_id": sha}),
        );
    }
    let shallow = git(path, &["rev-parse", "--is-shallow-repository"])?;
    let shallow = String::from_utf8_lossy(&shallow).trim() == "true";
    let mut rev_args = vec!["rev-list", "--all", "--parents"];
    if head.is_ok() {
        rev_args.push("HEAD");
    }
    let graph = match git(path, &rev_args) {
        Ok(graph) => graph,
        Err(error) => {
            git_failure(&mut out, repo, "git.graph", repo, &error);
            return Ok(out);
        }
    };
    let mut commit_count = 0;
    let mut failed_commits = 0;
    let mut all_revisions = reference_shas;
    for line in String::from_utf8_lossy(&graph).lines() {
        let Some(sha) = line.split_whitespace().next() else {
            continue;
        };
        all_revisions.insert(sha.to_string());
        let raw = match git(path, &["cat-file", "commit", sha]) {
            Ok(raw) => raw,
            Err(error) => {
                git_failure(&mut out, repo, "git.commit", sha, &error);
                failed_commits += 1;
                continue;
            }
        };
        let text = String::from_utf8_lossy(&raw).into_owned();
        let (headers, message) = text.split_once("\n\n").unwrap_or((&text, ""));
        let mut parents = vec![];
        let mut author = None;
        let mut committer = None;
        let mut tree = None;
        for header in headers.lines() {
            if let Some(s) = header.strip_prefix("parent ") {
                parents.push(s.to_owned());
            }
            if let Some(s) = header.strip_prefix("author ") {
                author = Some(s.to_owned());
            }
            if let Some(s) = header.strip_prefix("committer ") {
                committer = Some(s.to_owned());
            }
            if let Some(s) = header.strip_prefix("tree ") {
                tree = Some(s.to_owned());
            }
        }
        push_record(
            &mut out,
            &namespace,
            "git.commit",
            sha,
            json!({"sha":sha,"parents":parents,"tree":tree,"author":author,"committer":committer,"message":message,"raw_sha256":digest(&raw),"raw_commit":text}),
        );
        out.observations.push(RawResponse {
            source: "git.commit".into(),
            url: format!("git:{repo}/commit/{sha}"),
            collected_at: collected_at.clone(),
            status: None,
            headers: BTreeMap::new(),
            sha256: digest(&raw),
            body: raw,
            error: None,
        });
        // Retain separate comparisons to every parent. Merge diffs must not silently disappear.
        let comparisons: Vec<Option<&str>> = if parents.is_empty() {
            vec![None]
        } else {
            parents.iter().map(|p| Some(p.as_str())).collect()
        };
        for parent in comparisons {
            let key = format!("{sha}/parent:{}", parent.unwrap_or("root"));
            let patch = match parent {
                None => git(
                    path,
                    &[
                        "diff-tree",
                        "--root",
                        "--no-commit-id",
                        "--no-ext-diff",
                        "--no-textconv",
                        "--binary",
                        "--full-index",
                        "--ignore-submodules=none",
                        "-p",
                        sha,
                        "--",
                    ],
                ),
                Some(parent) => git(
                    path,
                    &[
                        "diff",
                        "--no-ext-diff",
                        "--no-textconv",
                        "--binary",
                        "--full-index",
                        "--ignore-submodules=none",
                        parent,
                        sha,
                        "--",
                    ],
                ),
            };
            match patch {
                Ok(body) => {
                    push_record(
                        &mut out,
                        &namespace,
                        "git.commit_diff",
                        &key,
                        json!({"revision":sha,"parent":parent,"patch":String::from_utf8_lossy(&body),"sha256":digest(&body),"interpretation":"version difference; not a measure of human effort or product value"}),
                    );
                    out.observations.push(RawResponse {
                        source: "git.commit_diff".into(),
                        url: format!("git:{repo}/diff/{key}"),
                        collected_at: collected_at.clone(),
                        status: None,
                        headers: BTreeMap::new(),
                        sha256: digest(&body),
                        body,
                        error: None,
                    });
                }
                Err(error) => git_failure(&mut out, repo, "git.commit_diff", &key, &error),
            }
        }
        commit_count += 1;
    }
    // Dependency and workflow definitions at each reachable revision, including tagged trees.
    let mut definition_count = 0;
    let mut definition_errors = 0;
    for sha in all_revisions {
        let files = match git(path, &["ls-tree", "-r", "--name-only", "-z", &sha]) {
            Ok(v) => v,
            Err(error) => {
                git_failure(&mut out, repo, "git.definition_tree", &sha, &error);
                definition_errors += 1;
                continue;
            }
        };
        for name in files.split(|b| *b == 0).filter(|n| !n.is_empty()) {
            let filename = String::from_utf8_lossy(name);
            let source = if filename == "Cargo.toml"
                || filename == "Cargo.lock"
                || filename.ends_with("/Cargo.toml")
                || filename.ends_with("/Cargo.lock")
            {
                "git.dependency_manifest"
            } else if filename.starts_with(".github/workflows/")
                && (filename.ends_with(".yml") || filename.ends_with(".yaml"))
            {
                "git.workflow_definition"
            } else {
                continue;
            };
            let spec = format!("{sha}:{filename}");
            match git(path, &["show", &spec]) {
                Ok(body) => {
                    push_record(
                        &mut out,
                        &namespace,
                        source,
                        &spec,
                        json!({"revision":sha,"path":filename,"body":String::from_utf8_lossy(&body),"sha256":digest(&body)}),
                    );
                    out.observations.push(RawResponse {
                        source: source.into(),
                        url: format!("git:{repo}/{spec}"),
                        collected_at: collected_at.clone(),
                        status: None,
                        headers: BTreeMap::new(),
                        sha256: digest(&body),
                        body,
                        error: None,
                    });
                    definition_count += 1;
                }
                Err(error) => {
                    git_failure(&mut out, repo, source, &spec, &error);
                    definition_errors += 1;
                }
            }
        }
    }
    out.observations.push(RawResponse {
        source: "git.refs".into(),
        url: format!("git:{repo}/refs"),
        collected_at,
        status: None,
        headers: BTreeMap::new(),
        sha256: digest(&refs),
        body: refs,
        error: None,
    });
    out.coverage.push(Coverage {source:"git.graph".into(),scope:repo.into(),state:if shallow || failed_commits > 0 {"incomplete"} else {"complete"}.into(),pages:1,records:commit_count,detail:Some(if shallow {"Shallow repository: missing ancestors are not reconstructed"} else {"Local refs and HEAD only; missing object reads have individual coverage errors. No remote completeness claim, fetch or garbage collection performed"}.into())});
    out.coverage.push(Coverage {source:"git.definitions".into(),scope:repo.into(),state:if definition_errors == 0 {"complete"} else {"incomplete"}.into(),pages:1,records:definition_count,detail:Some("Cargo manifests/locks and .github/workflows YAML at reachable revisions; empty inventory does not imply CI passed".into())});
    out.coverage.push(Coverage {source:"git.refs".into(),scope:repo.into(),state:"complete".into(),pages:1,records:ref_count,detail:Some("Current local refs; prior force-pushed or deleted refs require an earlier retained snapshot".into())});
    Ok(out)
}
