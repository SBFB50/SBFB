// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 74 Phase C — the atelier-fork CLI: fork a network project into a
//! local workspace, then redeploy the (edited) workspace under THIS node's
//! identity.
//!
//! - [`fork`] wires the Phase B [`crate::fork`] primitives to a command: clone
//!   an `https://` forge repo OR reconstruct from a published archive file.
//! - [`redeploy`] zips a local workspace and uploads it to the daemon's
//!   `POST /api/v1/deploy-workspace` route, which re-signs a FRESH provenance
//!   with the local keypair (R5: a fork is a new local author act; the original
//!   author's provenance is never inherited).

use std::io::{Read as _, Write as _};
use std::path::Path;

use crate::daemon_client::DaemonConnection;
use crate::fork::{self, ForkSource, ForkTriplet};

/// `sbfb-factory fork` — materialise a network project into `dest`.
///
/// Prefers the verifiable forge clone (`--repo-url`, optionally pinned to
/// `--commit-sha`); falls back to reconstructing from a published archive file
/// (`--archive <file.zip>`). The fork is NOT redeployed here — edit it, then run
/// `sbfb-factory redeploy <dest>`.
pub fn fork(
    dest: &str,
    repo_url: Option<&str>,
    commit_sha: Option<&str>,
    archive: Option<&str>,
    archive_hash: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let blob_bytes = match archive {
        Some(path) => Some(std::fs::read(path).map_err(|e| format!("read archive {path}: {e}"))?),
        None => None,
    };
    let triplet = ForkTriplet {
        repo_url: repo_url.map(String::from),
        commit_sha: commit_sha.map(String::from),
        // When forking from a local --archive file (which bypasses the daemon's
        // content-addressed fetch), pass the published hash so fork_from_search_hit
        // verifies blake3(bytes) == archive_hash before writing the workspace.
        archive_hash: archive_hash.map(String::from),
    };
    let dest_path = Path::new(dest);

    // The fork primitive is async (it shells out to git); the CLI is sync, so we
    // drive it on a one-shot current-thread runtime (mirrors the operator path).
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let source = rt.block_on(fork::fork_from_search_hit(
        &triplet,
        blob_bytes.as_deref(),
        dest_path,
    ))?;

    let how = match source {
        ForkSource::Forge => "forge clone",
        ForkSource::Blob => "archive reconstruction",
    };
    eprintln!("Forked into {dest} via {how}.");
    eprintln!(
        "Next: edit the workspace, then `sbfb-factory redeploy {dest}` to put your version online."
    );
    Ok(())
}

/// `sbfb-factory redeploy` — deploy a local (forked/edited) workspace under THIS
/// node's identity via the daemon.
pub fn redeploy(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = dunce::canonicalize(path)?;

    let manifest_path = workspace.join("SBFB.json");
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|_| "SBFB.json not found — run `sbfb-factory validate` first")?;
    let manifest: sbfb_manifest::SbfbManifest = serde_json::from_str(&content)?;
    let name = manifest.name.as_deref().unwrap_or("").to_string();
    if name.is_empty() {
        return Err("SBFB.json: name must not be empty".into());
    }
    if !workspace.join("index.html").is_file() {
        return Err("workspace must contain index.html at its root".into());
    }

    let zip_bytes = zip_workspace(&workspace)?;

    let conn = DaemonConnection::discover()?;
    let category = manifest
        .category
        .as_deref()
        .unwrap_or("general")
        .to_string();
    let description = manifest
        .description
        .as_deref()
        .unwrap_or_default()
        .to_string();
    let repo_url = manifest.repo_url.as_deref().unwrap_or_default().to_string();

    // Lineage-only query params; reqwest URL-encodes them. The zip is the body.
    // We forward repo_url (where the fork came from) but NOT a commit_sha: an
    // edited fork no longer corresponds to any specific upstream commit, so a
    // claimed lineage commit would be misleading. The redeploy is an honest
    // local self-attestation (is_open_source=false), not a reproducible build.
    let mut query: Vec<(&str, String)> = vec![
        ("project_name", name),
        ("category", category),
        ("description", description),
    ];
    if !repo_url.is_empty() {
        query.push(("repo_url", repo_url));
    }

    let url = format!("{}/api/v1/deploy-workspace", conn.base_url);
    let resp = conn
        .client()
        .post(&url)
        .query(&query)
        .header("X-SBFB-Token", &conn.token)
        .header("Host", "127.0.0.1")
        .header("content-type", "application/zip")
        .body(zip_bytes)
        .send()
        .map_err(|e| format!("redeploy request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("redeploy failed ({status}): {body}").into());
    }

    let json: serde_json::Value = resp.json()?;
    eprintln!(
        "redeployed under local identity: hash={}, provenance={}",
        json["hash"].as_str().unwrap_or("?"),
        json["provenance_hash"].as_str().unwrap_or("none"),
    );
    eprintln!(
        "note: a local fork redeploy is a self-attestation (not open-source). To \
         publish a verifiable open-source fork, push it to your own forge and use \
         `sbfb-factory publish`."
    );
    Ok(())
}

/// Zip a workspace directory into an in-memory archive, skipping `.git` and
/// symlinks (mirrors the daemon `deploy::zip_directory` rules so the bytes the
/// daemon re-signs match what a verified deploy would produce).
fn zip_workspace(src: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zw = zip::ZipWriter::new(&mut buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for entry in walkdir::WalkDir::new(src).follow_links(false) {
            let entry = entry?;
            let path = entry.path();
            if path == src {
                continue;
            }
            let name = path
                .strip_prefix(src)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            if name == ".git" || name.starts_with(".git/") {
                continue;
            }
            if name.contains("..") || name.starts_with('/') {
                continue;
            }
            if entry.path_is_symlink() {
                continue;
            }
            if path.is_dir() {
                continue; // files create their parents implicitly
            }
            zw.start_file(&name, options)?;
            let mut f = std::fs::File::open(path)?;
            let mut content = Vec::new();
            f.read_to_end(&mut content)?;
            zw.write_all(&content)?;
        }
        zw.finish()?;
    }
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;

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

    #[test]
    fn fork_reconstructs_from_archive_file() {
        // `sbfb-factory fork --archive <zip>` (no forge) reconstructs the
        // workspace from the published archive bytes.
        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("app.zip");
        std::fs::write(
            &zip_path,
            make_zip(&[("index.html", "<h1>x</h1>"), ("app.js", "1")]),
        )
        .unwrap();
        let dest = tmp.path().join("ws");

        fork(
            dest.to_str().unwrap(),
            None,
            None,
            Some(zip_path.to_str().unwrap()),
            None,
        )
        .unwrap();

        assert!(dest.join("index.html").is_file());
        assert!(dest.join("app.js").is_file());
    }

    #[test]
    fn fork_without_any_source_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("ws");
        let err = fork(dest.to_str().unwrap(), None, None, None, None).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("source"));
    }

    #[test]
    fn zip_workspace_skips_git_and_includes_files() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path().join("ws");
        std::fs::create_dir_all(ws.join(".git")).unwrap();
        std::fs::write(ws.join(".git").join("HEAD"), "ref").unwrap();
        std::fs::write(ws.join("index.html"), "<h1>ok</h1>").unwrap();
        std::fs::create_dir_all(ws.join("assets")).unwrap();
        std::fs::write(ws.join("assets").join("a.js"), "1").unwrap();

        let zip = zip_workspace(&ws).unwrap();
        let archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
        let names: Vec<String> = archive.file_names().map(String::from).collect();
        assert!(names.contains(&"index.html".to_string()));
        assert!(names.contains(&"assets/a.js".to_string()));
        assert!(
            !names.iter().any(|n| n.starts_with(".git")),
            ".git must be excluded"
        );
    }

    #[test]
    #[serial(sbfb_env)]
    fn redeploy_requires_running_daemon() {
        // `#[serial(sbfb_env)]`: mutates NEXUS_GRID_ROOT, shared with the other
        // daemon-discovery tests — serialize under plain `cargo test`.
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path().join("app");
        crate::template_engine::create("static", "redeploy-app", ws.to_str().unwrap()).unwrap();

        unsafe { std::env::set_var("NEXUS_GRID_ROOT", tmp.path().join("fake-grid")) };
        let err = redeploy(ws.to_str().unwrap()).unwrap_err();
        unsafe { std::env::remove_var("NEXUS_GRID_ROOT") };
        assert!(
            err.to_string().contains("daemon not running")
                || err.to_string().contains("running.json"),
            "expected daemon error, got: {err}"
        );
    }
}
