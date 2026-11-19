//! Identify compiled source content even before a working tree is committed.
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[path = "build_support/git.rs"]
mod git;

fn input_error(path: &Path, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("could not read build input {}: {error}", path.display()),
    )
}

fn inputs(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if fs::metadata(path)
        .map_err(|error| input_error(path, error))?
        .is_dir()
    {
        let entries = fs::read_dir(path).map_err(|error| input_error(path, error))?;
        for entry in entries {
            let entry = entry.map_err(|error| input_error(path, error))?;
            inputs(&entry.path(), files)?;
        }
    } else {
        files.push(path.to_owned());
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut files = Vec::new();
    for path in [
        "src",
        "build_support",
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        "rust-toolchain.toml",
    ] {
        println!("cargo:rerun-if-changed={path}");
        inputs(Path::new(path), &mut files)?;
    }
    files.sort();
    let mut source = Sha256::new();
    for path in files {
        let name = path.to_string_lossy().replace('\\', "/");
        let bytes = fs::read(&path).map_err(|error| input_error(&path, error))?;
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
    let revision = match git::git_provenance(Path::new(".")) {
        Some((revision, watched)) => {
            for path in watched {
                println!("cargo:rerun-if-changed={}", path.display());
            }
            revision
        }
        None => "source-archive-unidentified".to_owned(),
    };
    println!("cargo:rustc-env=OBSERVATORY_BUILD_REVISION={revision}");
    Ok(())
}
