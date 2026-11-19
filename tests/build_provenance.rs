//! Build provenance for the linked checkouts used to prepare research releases.
#[path = "../build_support/git.rs"]
mod build_git;

use std::{fs, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) -> String {
    let result = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().to_owned()
}

#[test]
fn linked_checkout_tracks_head_refs_index_and_dirty_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let base = directory.path().join("base");
    let worktree = directory.path().join("worktree");
    fs::create_dir(&base).unwrap();
    git(&base, &["init", "-b", "main"]);
    fs::write(base.join("notes"), "initial").unwrap();
    git(&base, &["add", "notes"]);
    git(&base, &["commit", "-m", "initial"]);
    git(
        &base,
        &[
            "worktree",
            "add",
            "-b",
            "topic/release",
            worktree.to_str().unwrap(),
        ],
    );
    git(&base, &["pack-refs", "--all"]);
    let (before, watched) = build_git::git_provenance(&worktree).unwrap();
    assert_eq!(before, git(&worktree, &["rev-parse", "HEAD"]));

    for key in ["HEAD", "index", "refs", "packed-refs"] {
        let path = git(
            &worktree,
            &["rev-parse", "--path-format=absolute", "--git-path", key],
        );
        assert!(watched.contains(Path::new(&path)), "{key}");
    }
    assert!(watched.contains(&worktree.join(".git")));
    assert!(watched.contains(&worktree.join("notes")));

    git(
        &worktree,
        &["commit", "--allow-empty", "-m", "new revision"],
    );
    let (after, _) = build_git::git_provenance(&worktree).unwrap();
    assert_ne!(before, after);
    assert_eq!(after, git(&worktree, &["rev-parse", "HEAD"]));
    fs::write(worktree.join("notes"), "changed").unwrap();
    assert_eq!(
        build_git::git_provenance(&worktree).unwrap().0,
        format!("{after}-dirty")
    );

    let package = worktree.join("unpacked-crate");
    fs::create_dir(&package).unwrap();
    assert!(build_git::git_provenance(&package).is_none());
}
