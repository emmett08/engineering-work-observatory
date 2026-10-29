//! Identify compiled source content even before a working tree is committed.
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
fn inputs(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).expect("source directory") {
            inputs(&entry.expect("source entry").path(), files);
        }
    } else {
        files.push(path.to_owned());
    }
}
fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/main");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=rust-toolchain.toml");
    let mut files = Vec::new();
    for path in [
        "src",
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        "rust-toolchain.toml",
    ] {
        inputs(Path::new(path), &mut files);
    }
    files.sort();
    let mut source = Sha256::new();
    for path in files {
        let name = path.to_string_lossy().replace('\\', "/");
        let bytes = std::fs::read(&path).expect("source bytes");
        source.update((name.len() as u64).to_be_bytes());
        source.update(name.as_bytes());
        source.update((bytes.len() as u64).to_be_bytes());
        source.update(&bytes);
    }
    let source_hash = source
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    println!("cargo:rustc-env=OBSERVATORY_SOURCE_SHA256={source_hash}");
    let revision = std::process::Command::new("git")
        .args(["describe", "--always", "--dirty", "--abbrev=40"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "source-archive-unidentified".to_owned());
    println!("cargo:rustc-env=OBSERVATORY_BUILD_REVISION={revision}");
}
