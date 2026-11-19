//! Git metadata used to identify the compiled checkout.
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

/// The dirty suffix includes all tracked files, including files outside src.
pub(crate) fn git_provenance(root: &Path) -> Option<(String, BTreeSet<PathBuf>)> {
    let git_marker = root.join(".git");
    if !git_marker.exists() {
        // An unpacked crate may itself be inside someone else's Git checkout.
        return None;
    }
    let revision = git(root, &["describe", "--always", "--dirty", "--abbrev=40"])?;
    let metadata = git(
        root,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "HEAD",
            "--git-path",
            "index",
            "--git-path",
            "refs",
            "--git-path",
            "packed-refs",
            "--git-path",
            "reftable",
        ],
    )?;
    let mut watched: BTreeSet<_> = metadata
        .lines()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();
    if git_marker.is_file() {
        watched.insert(git_marker);
    }
    let tracked = git(root, &["ls-files", "-t", "-z"])?;
    for entry in tracked.split_terminator('\0') {
        let (status, name) = entry.split_once(' ')?;
        // Cargo build directives occupy a single UTF-8 line.
        if name.contains(['\n', '\r']) {
            return None;
        }
        // Sparse checkout intentionally omits these files from the worktree.
        if status != "S" {
            watched.insert(root.join(name));
        }
    }
    Some((revision.trim_end().to_owned(), watched))
}
