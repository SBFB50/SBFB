// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 74 Phase B — fork a network project into a local target workspace.
//!
//! Given the provenance triplet of a search hit (`repo_url` / `commit_sha` /
//! `archive_hash`), materialise a NEW workspace **distinct from the nexus
//! repo**, by one of two source paths:
//!
//!   1. **Forge clone** (primary) — `git clone --depth 1 --single-branch` of an
//!      `https://` `repo_url`, then `fetch`/`checkout FETCH_HEAD` to pin
//!      `commit_sha`. Mirrors the daemon's verified deploy-from-repo clone
//!      (`nexus-shell-daemon::deploy::clone_repo`): https-only, depth-1, bounded
//!      timeouts, a 40-hex `commit_sha` validation, and a post-clone size cap.
//!      Reuses the git CLI already shelled by `process.rs` — zero new dependency.
//!   2. **Blob reconstruction** (fallback) — unzip the published archive bytes
//!      (`archive_hash`) into the workspace, guarded against zip-slip, symlink
//!      escapes, a compressed-input cap AND a decompressed-output cap
//!      (zip-bomb defence).
//!
//! Phase B materialises a workspace ONLY. Phase C re-deploys it under the LOCAL
//! node identity (a fresh author act — the seeder/forker never inherits the
//! original author's provenance; that invariant lives in Phase C). No
//! templates, no redeploy, no provenance re-signing here.
//!
//! ## Untrusted input
//! Both `repo_url` and `commit_sha` originate from a peer-published feed op
//! (untrusted gossip). The forge path therefore mirrors EVERY deploy.rs guard:
//! the `https://` scheme guard stops `repo_url` being parsed as a git option,
//! and the 40-hex `commit_sha` validation (+ a `--end-of-options` separator)
//! stops a leading-dash value (`--upload-pack=…`) becoming a git argument
//! injection / arbitrary-command-execution vector (CVE-2017-1000117 class).
//!
//! ## Blob integrity
//! `fork_from_blob` does NOT re-verify `blob_bytes` against `archive_hash`: the
//! Factory fetches those bytes over the daemon HTTP boundary, and the daemon's
//! iroh-blobs fetch is content-addressed (blake3-verified on fetch). The daemon
//! fetch boundary is the integrity authority; the Factory only defends disk
//! safety (zip-slip / symlink / size).
//!
//! ## Workspace location (G17)
//! The caller chooses `dest`; `fork.rs` never derives it from
//! [`crate::process::repo_root_pub`]. Writing untrusted forge content under the
//! maintainer's nexus checkout would be a supply-chain footgun — the target
//! workspace MUST be a fresh directory outside the nexus repo.

use std::path::Path;
use std::time::Duration;

/// Hard ceiling on the COMPRESSED archive input (mirrors the daemon deploy cap).
pub const MAX_ARCHIVE_BYTES: u64 = 500 * 1024 * 1024;

/// Hard ceiling on the DECOMPRESSED output across all archive entries — the
/// zip-bomb guard (a tiny deflate-of-zeros can inflate 1000x). Mirrors the
/// daemon `blob_serve::load` decompressed accounting.
pub const MAX_DECOMPRESSED_BYTES: u64 = 500 * 1024 * 1024;

/// Hard ceiling on the cloned forge workspace size (mirrors deploy.rs).
pub const MAX_CLONE_BYTES: u64 = 500 * 1024 * 1024;

const CLONE_TIMEOUT_SECS: u64 = 30;
const CHECKOUT_TIMEOUT_SECS: u64 = 10;

/// Errors materialising a fork workspace.
#[derive(Debug, thiserror::Error)]
pub enum ForkError {
    #[error("repo_url must be an https:// forge URL, got: {0}")]
    NonHttpsRepo(String),
    #[error("commit_sha must be 40 hex chars (git sha-1), got: {0}")]
    InvalidCommitSha(String),
    #[error(
        "no usable fork source: provide an https repo_url (with optional commit_sha) \
         or the published archive bytes"
    )]
    NoSource,
    #[error("git {action} failed: {detail}")]
    Git { action: String, detail: String },
    #[error("git {action} timed out after {secs}s")]
    GitTimeout { action: String, secs: u64 },
    #[error("archive too large: {0} bytes (max {max})", max = MAX_ARCHIVE_BYTES)]
    ArchiveTooLarge(u64),
    #[error("cloned workspace too large: {0} bytes (max {max})", max = MAX_CLONE_BYTES)]
    CloneTooLarge(u64),
    #[error("unsafe archive entry (zip-slip / escape): {0}")]
    UnsafePath(String),
    #[error("invalid zip archive: {0}")]
    Zip(String),
    #[error("io error: {0}")]
    Io(String),
}

/// Which source path materialised the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkSource {
    Forge,
    Blob,
}

/// The provenance triplet of a network search hit, as the Factory receives it
/// from the daemon's search JSON (plain strings — the Factory does not depend on
/// the coordinator's `SearchResult` type, keeping it a decoupled HTTP client).
#[derive(Debug, Clone, Default)]
pub struct ForkTriplet {
    pub repo_url: Option<String>,
    pub commit_sha: Option<String>,
    pub archive_hash: Option<String>,
}

/// Fork a network project into `dest`, preferring the verifiable forge source
/// and falling back to the published archive bytes.
///
/// `blob_bytes` is the raw published zip (fetched by the caller from the daemon
/// when no forge source is available). The unit tests pass it in-memory; the
/// production caller fetches it over the daemon HTTP boundary (the Factory has
/// no live iroh endpoint).
pub async fn fork_from_search_hit(
    triplet: &ForkTriplet,
    blob_bytes: Option<&[u8]>,
    dest: &Path,
) -> Result<ForkSource, ForkError> {
    // Prefer the forge clone: it yields the real, human-readable source the
    // fork author will edit and re-deploy.
    if let Some(repo_url) = triplet.repo_url.as_deref() {
        if is_https_url(repo_url) {
            fork_from_forge(repo_url, triplet.commit_sha.as_deref(), dest).await?;
            return Ok(ForkSource::Forge);
        }
    }
    // Fallback: reconstruct from the published archive bytes.
    if let Some(bytes) = blob_bytes {
        fork_from_blob(bytes, dest)?;
        return Ok(ForkSource::Blob);
    }
    Err(ForkError::NoSource)
}

/// Clone an `https://` forge repo into `dest`, pinning `commit_sha` when given.
pub async fn fork_from_forge(
    repo_url: &str,
    commit_sha: Option<&str>,
    dest: &Path,
) -> Result<(), ForkError> {
    if !is_https_url(repo_url) {
        return Err(ForkError::NonHttpsRepo(repo_url.to_string()));
    }
    if let Some(sha) = commit_sha {
        if !is_valid_sha(sha) {
            return Err(ForkError::InvalidCommitSha(sha.to_string()));
        }
    }
    run_git_clone(repo_url, commit_sha, dest).await?;
    // Post-clone size cap (mirrors deploy.rs): a squatted/malicious repo must
    // not fill the fork target's disk.
    let size = dir_size(dest);
    if size > MAX_CLONE_BYTES {
        let _ = std::fs::remove_dir_all(dest);
        return Err(ForkError::CloneTooLarge(size));
    }
    Ok(())
}

/// Reconstruct a workspace from the published zip archive bytes.
///
/// Defends every disk write against:
/// - **zip-slip** — entries with `..`, an absolute prefix, or an embedded
///   backslash are rejected (same rules as
///   `nexus_shell_daemon_core::blob_serve::validate_zip_path`; duplicated here
///   as a ~6-line pure check so the client Factory stays decoupled from
///   `nexus-shell-daemon-core`, per v4 D2 "Factory hors daemon");
/// - **symlink escape** — symlink entries are skipped (mirrors `deploy.rs`);
/// - **path escape** — the canonical output path is re-checked to be inside
///   `dest` after `join` (defense in depth against platform-specific quirks);
/// - **size** — both the compressed input ([`MAX_ARCHIVE_BYTES`]) and the
///   decompressed output ([`MAX_DECOMPRESSED_BYTES`], the zip-bomb guard).
pub fn fork_from_blob(zip_bytes: &[u8], dest: &Path) -> Result<(), ForkError> {
    extract_zip(zip_bytes, dest, MAX_DECOMPRESSED_BYTES)
}

/// Inner extraction with an injectable decompressed-size ceiling (the public
/// [`fork_from_blob`] pins it to [`MAX_DECOMPRESSED_BYTES`]; tests inject a
/// small ceiling to exercise the zip-bomb guard without a 500 MB fixture).
fn extract_zip(zip_bytes: &[u8], dest: &Path, max_decompressed: u64) -> Result<(), ForkError> {
    if zip_bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(ForkError::ArchiveTooLarge(zip_bytes.len() as u64));
    }
    let reader = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| ForkError::Zip(e.to_string()))?;

    std::fs::create_dir_all(dest).map_err(|e| ForkError::Io(e.to_string()))?;
    let dest_canon = canonicalize_lossy(dest);

    let mut total_decompressed: u64 = 0;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ForkError::Zip(e.to_string()))?;
        let name = file.name().to_string();

        // Skip directory entries (created lazily for files below).
        if name.ends_with('/') {
            continue;
        }
        // Skip symlink entries — never materialise them on disk (a byzantine
        // publisher could embed a symlink to escape the workspace).
        if file.is_symlink() {
            continue;
        }
        // zip-slip guard (write-time, BEFORE touching the disk).
        if !is_safe_archive_path(&name) {
            return Err(ForkError::UnsafePath(name));
        }

        let out_path = dest.join(&name);
        // Defense in depth: the canonicalised parent must stay under dest.
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ForkError::Io(e.to_string()))?;
            if let Some(dc) = &dest_canon {
                if let Some(pc) = canonicalize_lossy(parent) {
                    if !pc.starts_with(dc) {
                        return Err(ForkError::UnsafePath(name));
                    }
                }
            }
        }

        // Bounded copy: a single lying entry cannot inflate past the remaining
        // decompressed budget. `take(remaining + 1)` lets us detect overflow.
        let remaining = max_decompressed.saturating_sub(total_decompressed);
        let mut out = std::fs::File::create(&out_path).map_err(|e| ForkError::Io(e.to_string()))?;
        let mut bounded = std::io::Read::take(&mut file, remaining.saturating_add(1));
        let written =
            std::io::copy(&mut bounded, &mut out).map_err(|e| ForkError::Io(e.to_string()))?;
        total_decompressed = total_decompressed.saturating_add(written);
        if total_decompressed > max_decompressed {
            return Err(ForkError::ArchiveTooLarge(total_decompressed));
        }
    }
    Ok(())
}

/// `https://`-only scheme guard (mirrors the daemon deploy guard). Rejects
/// `http`, `file`, `git`, `ssh`, `javascript`, etc. — the forge clone only
/// trusts TLS forge origins, and a `-`-leading value can never pass.
fn is_https_url(url: &str) -> bool {
    url.starts_with("https://")
}

/// Validate a git sha (mirrors `deploy.rs::is_valid_sha`): exactly 40 ascii
/// hex chars. An all-hex value can never start with `-`, so it can never be
/// parsed by git as an option (argument-injection defence).
fn is_valid_sha(sha: &str) -> bool {
    sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Zip-slip path guard — the rules of
/// `nexus_shell_daemon_core::blob_serve::validate_zip_path` PLUS a `:` rejection
/// (Windows drive-absolute / ADS escape) that the canonical guard omits because
/// it never joins to disk. See the parity test
/// `is_safe_archive_path_matches_canonical_rules` below; if the canonical guard
/// is ever hardened, mirror it here.
fn is_safe_archive_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains("..")
        && !path.contains('\\')
        // Reject any `:` — STRICTER than the canonical `validate_zip_path`. On
        // Windows a drive-absolute entry (`C:/x`) makes `dest.join` ESCAPE dest
        // (join with an absolute path replaces the base), and `name:stream` is
        // an alternate-data-stream vector. `validate_zip_path` only serves bytes
        // in memory (never joins to disk), so it does not need this; fork.rs
        // writes to disk, so it does. No legitimate web archive path needs `:`.
        && !path.contains(':')
}

fn canonicalize_lossy(p: &Path) -> Option<std::path::PathBuf> {
    std::fs::canonicalize(p).ok()
}

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .fold(0u64, |a, b| a.saturating_add(b))
}

/// Low-level git clone mechanics (no scheme/sha check — callers gate them).
/// Split out from [`fork_from_forge`] so unit tests can clone a local fixture
/// repo (which is not `https://`) without disabling the production guards.
async fn run_git_clone(
    repo_url: &str,
    commit_sha: Option<&str>,
    dest: &Path,
) -> Result<(), ForkError> {
    let dest_str = dest.to_string_lossy().to_string();
    run_git(
        &[
            "clone",
            "--depth",
            "1",
            "--single-branch",
            repo_url,
            &dest_str,
        ],
        Duration::from_secs(CLONE_TIMEOUT_SECS),
        "clone",
    )
    .await?;

    if let Some(sha) = commit_sha {
        // `--end-of-options` ensures a leading-dash value can never be parsed as
        // a git option (defence in depth; `fork_from_forge` already validates
        // 40-hex, so a dash value never reaches here in production).
        run_git(
            &[
                "-C",
                &dest_str,
                "fetch",
                "--depth",
                "1",
                "--end-of-options",
                "origin",
                sha,
            ],
            Duration::from_secs(CLONE_TIMEOUT_SECS),
            "fetch",
        )
        .await?;
        run_git(
            &["-C", &dest_str, "checkout", "FETCH_HEAD"],
            Duration::from_secs(CHECKOUT_TIMEOUT_SECS),
            "checkout",
        )
        .await?;
    }
    Ok(())
}

async fn run_git(args: &[&str], timeout: Duration, action: &str) -> Result<(), ForkError> {
    let child = tokio::process::Command::new("git")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        // Kill a hung/slowloris git on timeout instead of orphaning it.
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| ForkError::Git {
            action: action.to_string(),
            detail: format!("spawn: {e}"),
        })?;

    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let detail: String = stderr.chars().take(500).collect();
                Err(ForkError::Git {
                    action: action.to_string(),
                    detail,
                })
            }
        }
        Ok(Err(e)) => Err(ForkError::Git {
            action: action.to_string(),
            detail: e.to_string(),
        }),
        Err(_) => Err(ForkError::GitTimeout {
            action: action.to_string(),
            secs: timeout.as_secs(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn git(args: &[&str], cwd: &Path) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("git available");
        assert!(status.success(), "git {args:?} failed");
    }

    fn head_sha(repo: &Path) -> String {
        String::from_utf8(
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(repo)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string()
    }

    /// Build a tiny in-memory zip with the given (name, contents) entries.
    fn make_zip(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for (name, contents) in entries {
                zw.start_file(*name, opts).unwrap();
                zw.write_all(contents.as_bytes()).unwrap();
            }
            zw.finish().unwrap();
        }
        buf.into_inner()
    }

    #[tokio::test]
    async fn fork_from_forge_clones_repo_at_commit() {
        let tmp = tempfile::tempdir().unwrap();
        // Fixture repo with TWO commits so checkout of the EARLIER sha is
        // distinguishable from the default (depth-1 clone lands on HEAD).
        let origin = tmp.path().join("origin");
        std::fs::create_dir_all(&origin).unwrap();
        git(&["init", "-q"], &origin);
        git(&["config", "user.email", "t@t.t"], &origin);
        git(&["config", "user.name", "t"], &origin);
        std::fs::write(origin.join("index.html"), "<h1>old</h1>").unwrap();
        git(&["add", "."], &origin);
        git(&["commit", "-q", "-m", "old"], &origin);
        let sha_old = head_sha(&origin);
        std::fs::write(origin.join("index.html"), "<h1>new</h1>").unwrap();
        git(&["add", "."], &origin);
        git(&["commit", "-q", "-m", "new"], &origin);
        // Local fetch of an arbitrary (non-tip) sha must be allowed.
        git(
            &["config", "uploadpack.allowAnySHA1InWant", "true"],
            &origin,
        );

        let dest = tmp.path().join("workspace");
        run_git_clone(&origin.to_string_lossy(), Some(&sha_old), &dest)
            .await
            .expect("clone + checkout pinned sha");

        // Pinned the EARLIER commit, not the default HEAD → proves checkout.
        assert_eq!(
            std::fs::read_to_string(dest.join("index.html")).unwrap(),
            "<h1>old</h1>",
            "checkout must pin the requested sha, not land on default HEAD"
        );
    }

    #[tokio::test]
    async fn fork_from_forge_rejects_non_https() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("ws");
        for url in ["http://evil.example/repo.git", "file:///etc", "git://x/y"] {
            let err = fork_from_forge(url, None, &dest).await.unwrap_err();
            assert!(matches!(err, ForkError::NonHttpsRepo(_)), "url={url}");
        }
    }

    #[tokio::test]
    async fn fork_from_forge_rejects_argument_injection_sha() {
        // P0 regression: a leading-dash commit_sha (git argument injection,
        // CVE-2017-1000117 class) must be rejected BEFORE any git runs, and the
        // sentinel command must NEVER execute.
        let tmp = tempfile::tempdir().unwrap();
        let sentinel = tmp.path().join("PWNED");
        let payload = format!(
            "--upload-pack=touch {}",
            sentinel.to_string_lossy().replace('\\', "/")
        );
        let err = fork_from_forge(
            "https://example.com/r.git",
            Some(&payload),
            &tmp.path().join("ws"),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ForkError::InvalidCommitSha(_)));
        assert!(
            !sentinel.exists(),
            "injected command must NOT have executed"
        );

        // Non-hex / wrong-length shas also rejected.
        for bad in ["", "abc", &"g".repeat(40), &"a".repeat(41)] {
            let e = fork_from_forge(
                "https://example.com/r.git",
                Some(bad),
                &tmp.path().join("w2"),
            )
            .await
            .unwrap_err();
            assert!(
                matches!(e, ForkError::InvalidCommitSha(_)),
                "bad sha={bad:?}"
            );
        }
        // A well-formed 40-hex sha passes validation (clone then fails on the
        // unreachable host — proving validation let it through to the clone).
        assert!(is_valid_sha(&"a".repeat(40)));
    }

    #[test]
    fn fork_from_blob_reconstructs_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("ws");
        let zip = make_zip(&[
            ("index.html", "<h1>blob</h1>"),
            ("assets/app.js", "console.log(1)"),
        ]);
        fork_from_blob(&zip, &dest).expect("reconstruct");
        assert_eq!(
            std::fs::read_to_string(dest.join("index.html")).unwrap(),
            "<h1>blob</h1>"
        );
        assert!(dest.join("assets/app.js").is_file());
    }

    #[test]
    fn fork_from_blob_rejects_zip_slip_all_vectors() {
        let tmp = tempfile::tempdir().unwrap();
        for bad in [
            "../escape.txt",
            "/abs.txt",
            "\\win.txt",
            "a\\b.txt",
            "nested/../../escape.txt",
            "C:/escape.txt",
        ] {
            let dest = tmp.path().join("ws");
            let zip = make_zip(&[(bad, "pwned")]);
            let err = fork_from_blob(&zip, &dest).unwrap_err();
            assert!(matches!(err, ForkError::UnsafePath(_)), "vector={bad:?}");
        }
        // The escaping file must NOT exist anywhere outside dest.
        assert!(!tmp.path().join("escape.txt").exists());
        assert!(!tmp.path().join("abs.txt").exists());
    }

    #[test]
    fn fork_from_blob_skips_symlink_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("ws");
        // Build a zip whose "link" entry is flagged as a symlink (unix mode).
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let regular = zip::write::SimpleFileOptions::default();
            zw.start_file("index.html", regular).unwrap();
            zw.write_all(b"<h1>ok</h1>").unwrap();
            // A real symlink entry (S_IFLNK in the external attributes) — the
            // exact byzantine vector fork_from_blob must skip.
            zw.add_symlink("link", "/etc/passwd", regular).unwrap();
            zw.finish().unwrap();
        }
        fork_from_blob(&buf.into_inner(), &dest).expect("reconstruct");
        assert!(dest.join("index.html").is_file());
        assert!(!dest.join("link").exists(), "symlink entry must be skipped");
    }

    #[test]
    fn fork_from_blob_rejects_zip_bomb() {
        // A small entry that inflates past the (tiny, test-injected) cap is
        // rejected — the decompressed-output guard, not just the compressed cap.
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("ws");
        let zip = make_zip(&[("big.txt", &"A".repeat(10_000))]);
        let err = extract_zip(&zip, &dest, 1024).unwrap_err();
        assert!(matches!(err, ForkError::ArchiveTooLarge(_)));
    }

    #[test]
    fn is_safe_archive_path_matches_canonical_rules() {
        // Parity guard against `blob_serve::validate_zip_path` (duplicated for
        // crate decoupling). If the canonical guard is hardened, this fixture
        // table must be extended to match.
        for ok in ["index.html", "assets/app.js", "a/b/c.txt"] {
            assert!(is_safe_archive_path(ok), "should accept {ok:?}");
        }
        for bad in [
            "",
            "/abs",
            "\\win",
            "..",
            "a/../b",
            "a\\b",
            "C:/evil",
            "C:evil",
            "file.txt:stream",
        ] {
            assert!(!is_safe_archive_path(bad), "should reject {bad:?}");
        }
    }

    #[test]
    fn fork_target_workspace_distinct_from_nexus_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("ws");
        let zip = make_zip(&[("index.html", "x")]);
        fork_from_blob(&zip, &dest).unwrap();

        let repo_root = crate::process::repo_root_pub();
        let dest_canon = std::fs::canonicalize(&dest).unwrap();
        let root_canon = std::fs::canonicalize(&repo_root).unwrap_or_else(|_| repo_root.clone());
        assert!(
            !dest_canon.starts_with(&root_canon),
            "fork workspace {dest_canon:?} must be outside the nexus repo {root_canon:?}"
        );
    }

    #[tokio::test]
    async fn fork_from_search_hit_prefers_forge_then_blob() {
        let tmp = tempfile::tempdir().unwrap();

        // BOTH a forge repo_url AND blob bytes present → the FORGE path is taken.
        // The repo_url is an unreachable loopback port, so the forge clone FAILS
        // fast with a Git error — proving the dispatch chose forge, not blob
        // (which would have returned Ok(Blob)).
        let zip = make_zip(&[("index.html", "<h1>blob</h1>")]);
        let both = ForkTriplet {
            repo_url: Some("https://127.0.0.1:1/unreachable.git".into()),
            commit_sha: None,
            archive_hash: Some("ab".repeat(32)),
        };
        let err = fork_from_search_hit(&both, Some(&zip), &tmp.path().join("both_ws"))
            .await
            .unwrap_err();
        assert!(
            matches!(err, ForkError::Git { .. } | ForkError::GitTimeout { .. }),
            "both-present must attempt the forge path (and fail on the unreachable host), not fall back to blob"
        );

        // No https repo_url, but archive bytes present → Blob path.
        let dest_blob = tmp.path().join("blob_ws");
        let blob_only = ForkTriplet {
            repo_url: None,
            commit_sha: None,
            archive_hash: Some("ab".repeat(32)),
        };
        let src = fork_from_search_hit(&blob_only, Some(&zip), &dest_blob)
            .await
            .unwrap();
        assert_eq!(src, ForkSource::Blob);
        assert!(dest_blob.join("index.html").is_file());

        // Neither source → NoSource error.
        let err = fork_from_search_hit(&ForkTriplet::default(), None, &tmp.path().join("x"))
            .await
            .unwrap_err();
        assert!(matches!(err, ForkError::NoSource));
    }
}
