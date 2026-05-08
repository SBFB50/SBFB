// SPDX-License-Identifier: AGPL-3.0-or-later
//! Build executor MVP — clone, compile, hash.
//!
//! Tier 2 of the LT-7 self-hosted build pipeline. Clones a git
//! repository, runs `cargo build --release --locked`, and computes
//! the SHA256 of the resulting binary. Sandbox isolation (podman
//! rootless) is deferred to S56+.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("missing build parameter: {0}")]
    MissingParam(String),
    #[error("git clone failed: {0}")]
    CloneFailed(String),
    #[error("git checkout failed: {0}")]
    CheckoutFailed(String),
    #[error("cargo build failed: {0}")]
    BuildFailed(String),
    #[error("binary not found at {0}")]
    BinaryNotFound(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct BuildParams {
    pub repo: String,
    pub commit: String,
    pub binary: String,
}

#[derive(Debug)]
pub struct BuildResult {
    pub sha256: String,
    pub binary_path: PathBuf,
}

impl BuildParams {
    pub fn from_metadata(metadata: &BTreeMap<String, String>) -> Result<Self, BuildError> {
        let repo = metadata
            .get("build.repo")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| BuildError::MissingParam("build.repo".into()))?
            .clone();
        let commit = metadata
            .get("build.commit")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| BuildError::MissingParam("build.commit".into()))?
            .clone();
        let binary = metadata
            .get("build.binary")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| BuildError::MissingParam("build.binary".into()))?
            .clone();
        Ok(Self {
            repo,
            commit,
            binary,
        })
    }
}

/// Compute the SHA256 digest of a file, returned as lowercase hex.
pub fn sha256_file(path: &Path) -> Result<String, BuildError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Execute a build task in `work_dir`.
///
/// Clone → checkout → `cargo build --release --locked` → SHA256.
/// The caller manages the lifetime of `work_dir`.
pub fn execute_build(params: &BuildParams, work_dir: &Path) -> Result<BuildResult, BuildError> {
    let source_dir = work_dir.join("source");

    let clone_out = Command::new("git")
        .args(["clone", &params.repo, "source"])
        .current_dir(work_dir)
        .output()
        .map_err(|e| BuildError::CloneFailed(e.to_string()))?;
    if !clone_out.status.success() {
        return Err(BuildError::CloneFailed(
            String::from_utf8_lossy(&clone_out.stderr).into_owned(),
        ));
    }

    let checkout_out = Command::new("git")
        .args(["checkout", &params.commit])
        .current_dir(&source_dir)
        .output()
        .map_err(|e| BuildError::CheckoutFailed(e.to_string()))?;
    if !checkout_out.status.success() {
        return Err(BuildError::CheckoutFailed(
            String::from_utf8_lossy(&checkout_out.stderr).into_owned(),
        ));
    }

    let ts_out = Command::new("git")
        .args(["log", "-1", "--format=%ct"])
        .current_dir(&source_dir)
        .output()
        .map_err(|e| BuildError::BuildFailed(format!("git log timestamp: {e}")))?;
    let source_date_epoch = String::from_utf8_lossy(&ts_out.stdout).trim().to_string();

    let build_out = Command::new("cargo")
        .args(["build", "--release", "--locked", "-p", &params.binary])
        .env("SOURCE_DATE_EPOCH", &source_date_epoch)
        .env("CARGO_INCREMENTAL", "0")
        .current_dir(&source_dir)
        .output()
        .map_err(|e| BuildError::BuildFailed(e.to_string()))?;
    if !build_out.status.success() {
        return Err(BuildError::BuildFailed(
            String::from_utf8_lossy(&build_out.stderr).into_owned(),
        ));
    }

    let binary_path = source_dir
        .join("target")
        .join("release")
        .join(&params.binary);
    if !binary_path.exists() {
        return Err(BuildError::BinaryNotFound(binary_path));
    }

    let sha256 = sha256_file(&binary_path)?;
    Ok(BuildResult {
        sha256,
        binary_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sha256_matches_known_binary() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let file_path = dir.path().join("test-binary");
        let content = b"hello SBFB build executor";
        std::fs::File::create(&file_path)
            .expect("create")
            .write_all(content)
            .expect("write");

        let hash = sha256_file(&file_path).expect("hash");

        let mut expected_hasher = Sha256::new();
        expected_hasher.update(content);
        let expected = hex::encode(expected_hasher.finalize());
        assert_eq!(hash, expected);
    }

    #[test]
    fn from_metadata_rejects_missing_repo() {
        let metadata = BTreeMap::new();
        let err = BuildParams::from_metadata(&metadata).expect_err("should reject");
        assert!(matches!(err, BuildError::MissingParam(_)));
        assert!(err.to_string().contains("build.repo"));
    }
}
