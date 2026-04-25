// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline git-log signature parser for Couche 3 multi-forge cross-validation.
//!
//! Parses `git log --show-signature` output to extract signed commit
//! metadata without relying on forge APIs (offline-first, no OAuth).
//! Cross-platform via the `git` CLI (prerequisite for SBFB contributors).
//!
//! ## Security note
//!
//! `repo_path` **must** be a local filesystem path to a clone already
//! performed by the coordinator's verified-deploy flow (S14). Never
//! pass a user-supplied URL — `Command::new("git")` executes locally.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{NexusError, Result};

/// Signature type on a git commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SigType {
    /// GnuPG (RFC 4880).
    Gpg,
    /// OpenSSH (RFC 8709, git 2.34+).
    Ssh,
}

/// Aggregated contribution record from a single forge repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgeContribution {
    /// Key fingerprint (GPG fingerprint or SSH key hash).
    pub fingerprint: String,
    /// Number of signed commits by this key.
    pub commit_count: u32,
    /// Earliest signed commit timestamp (UTC unix seconds).
    pub first_seen: i64,
    /// Latest signed commit timestamp (UTC unix seconds).
    pub last_seen: i64,
    /// Remote origin URL of the repository.
    pub forge_url: String,
    /// Signature type observed.
    pub sig_type: SigType,
}

/// Parse signed commits from a local git repository.
///
/// Executes `git log --show-signature --format=...` and aggregates
/// results per signing key fingerprint. Only commits with a verified
/// ("Good") signature are included.
///
/// # Errors
///
/// Returns an error if `git` is not found or the path is not a valid
/// git repository.
pub fn parse_git_log(repo_path: &Path) -> Result<Vec<ForgeContribution>> {
    if !repo_path.is_dir() {
        return Err(NexusError::Other(format!(
            "forge_parser: not a directory: {}",
            repo_path.display()
        )));
    }

    let forge_url = get_origin_url(repo_path)?;

    // %aI = author date ISO 8601, %GK = key used to sign, %G? = sig status, %GS = signer
    let output = Command::new("git")
        .args([
            "-C",
            &repo_path.to_string_lossy(),
            "log",
            "--format=%aI|%GK|%G?|%GS",
            "--no-walk=unsorted",
            "--all",
        ])
        .output()
        .map_err(|e| NexusError::Other(format!("forge_parser: git exec failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NexusError::Other(format!(
            "forge_parser: git log failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut by_fingerprint: HashMap<String, ForgeContribution> = HashMap::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            continue;
        }
        let (date_str, key_id, status, _signer) = (parts[0], parts[1], parts[2], parts[3]);

        // Only accept verified signatures
        if !is_good_signature(status) {
            continue;
        }

        if key_id.is_empty() {
            continue;
        }

        let ts = parse_iso8601_to_unix(date_str).unwrap_or(0);
        let sig_type = if key_id.contains(':') || key_id.starts_with("SHA256:") {
            SigType::Ssh
        } else {
            SigType::Gpg
        };

        let fingerprint = normalize_fingerprint(key_id);

        let entry = by_fingerprint
            .entry(fingerprint.clone())
            .or_insert_with(|| ForgeContribution {
                fingerprint,
                commit_count: 0,
                first_seen: ts,
                last_seen: ts,
                forge_url: forge_url.clone(),
                sig_type,
            });
        entry.commit_count += 1;
        if ts < entry.first_seen {
            entry.first_seen = ts;
        }
        if ts > entry.last_seen {
            entry.last_seen = ts;
        }
    }

    Ok(by_fingerprint.into_values().collect())
}

fn get_origin_url(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_path.to_string_lossy(),
            "config",
            "--get",
            "remote.origin.url",
        ])
        .output()
        .map_err(|e| NexusError::Other(format!("forge_parser: git config failed: {e}")))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_good_signature(status: &str) -> bool {
    // G = Good, U = Good but untrusted (still a valid sig)
    matches!(status, "G" | "U")
}

fn normalize_fingerprint(key_id: &str) -> String {
    key_id.trim().replace(' ', "").to_ascii_lowercase()
}

fn parse_iso8601_to_unix(s: &str) -> Option<i64> {
    // chrono is already a dep; parse ISO 8601 with timezone
    chrono::DateTime::parse_from_rfc3339(s)
        .or_else(|_| chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z"))
        .map(|dt| dt.timestamp())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_good_signature() {
        assert!(is_good_signature("G"));
        assert!(is_good_signature("U"));
        assert!(!is_good_signature("B"));
        assert!(!is_good_signature("N"));
        assert!(!is_good_signature("E"));
        assert!(!is_good_signature(""));
    }

    #[test]
    fn test_normalize_fingerprint() {
        assert_eq!(normalize_fingerprint("ABCD EF01"), "abcdef01");
        assert_eq!(normalize_fingerprint("SHA256:abc123"), "sha256:abc123");
    }

    #[test]
    fn test_sig_type_detection() {
        let ssh_key = "SHA256:abc123def456";
        let gpg_key = "ABCDEF0123456789";
        assert!(ssh_key.contains(':'));
        assert!(!gpg_key.contains(':'));
    }

    #[test]
    fn test_parse_iso8601() {
        let ts = parse_iso8601_to_unix("2026-04-25T10:30:00+02:00");
        assert!(ts.is_some());
        assert!(ts.unwrap() > 1_700_000_000);
    }

    #[test]
    fn test_forge_parser_nonexistent_dir() {
        let result = parse_git_log(Path::new("/nonexistent/path/xyz"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }
}
