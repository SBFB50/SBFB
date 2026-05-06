// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deploy handlers for verified deploy from source (Sprint 42 Phase B,
//! port of deploy.py S14).
//!
//! Two endpoints:
//! - `POST /api/v1/deploy` — private deploy (raw zip upload)
//! - `POST /api/v1/deploy-from-repo` — public verified deploy (clone+verify+provenance)

use std::io::{self, Read as _, Write as _};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use nexus_coordinator_rs::forge::normalize_clone_url;
use nexus_coordinator_rs::provenance;
use nexus_core_rs::BlobsClient;
use nexus_core_rs::crypto::blake3_hash;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::http::DaemonHttpState;

const MAX_DEPLOY_BYTES: usize = 100 * 1024 * 1024;
const MAX_CLONE_BYTES: u64 = 500 * 1024 * 1024;
const CLONE_TIMEOUT_SECS: u64 = 30;
const CHECKOUT_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Deserialize)]
pub struct DeployFromRepoRequest {
    pub repo_url: String,
    #[serde(default)]
    pub commit_sha: Option<String>,
    pub project_name: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub apps: Vec<String>,
}

fn default_category() -> String {
    "general".to_string()
}

#[derive(Debug, Serialize)]
pub struct DeployResponse {
    pub deployed: bool,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeployError {
    error: String,
}

pub async fn deploy_from_repo(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<DeployFromRepoRequest>,
) -> Response {
    debug!(repo = %req.repo_url, "POST /api/v1/deploy-from-repo");

    let repo_url = normalize_clone_url(&req.repo_url);
    if !repo_url.starts_with("http") {
        return error_response(StatusCode::BAD_REQUEST, "repo_url must be an HTTP(S) URL");
    }

    if let Some(ref sha) = req.commit_sha {
        if !is_valid_sha(sha) {
            return error_response(
                StatusCode::BAD_REQUEST,
                "commit_sha must be a full 40-character hex SHA",
            );
        }
    }

    if !is_repo_public(&repo_url).await {
        return error_response(
            StatusCode::BAD_REQUEST,
            "repository is not publicly accessible",
        );
    }

    let tmpdir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("tmpdir: {e}"));
        }
    };
    let clone_dir = tmpdir.path().join("repo");

    if let Err(e) = clone_repo(&repo_url, &clone_dir, req.commit_sha.as_deref()).await {
        return error_response(StatusCode::BAD_REQUEST, &e);
    }

    let clone_size = dir_size(&clone_dir);
    if clone_size > MAX_CLONE_BYTES {
        return error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!(
                "repository exceeds {} MB limit",
                MAX_CLONE_BYTES / (1024 * 1024)
            ),
        );
    }

    let sbfb = match read_sbfb_json(&clone_dir) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };
    if sbfb.node_id != state.node_id {
        return error_response(
            StatusCode::BAD_REQUEST,
            &format!(
                "SBFB.json node_id ({}...) does not match daemon node_id ({}...)",
                &sbfb.node_id[..16.min(sbfb.node_id.len())],
                &state.node_id[..16.min(state.node_id.len())],
            ),
        );
    }

    if !clone_dir.join("index.html").is_file() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "repository must contain index.html at root",
        );
    }

    let commit_sha = match req.commit_sha {
        Some(sha) => sha.to_lowercase(),
        None => match git_rev_parse(&clone_dir).await {
            Ok(sha) => sha,
            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
        },
    };

    let zip_bytes = match zip_directory(&clone_dir) {
        Ok(b) => b,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("zip creation: {e}"),
            );
        }
    };
    debug!(size = zip_bytes.len(), "deploy-from-repo: zipped");

    let artifact_hash_bytes = blake3_hash(&zip_bytes);
    let artifact_hash_hex = hex::encode(artifact_hash_bytes);

    let prov = provenance::generate_provenance(
        &repo_url,
        &commit_sha,
        &artifact_hash_hex,
        &state.node_id,
        &state.pow_keypair,
    );

    // Best-effort contributor attestation (Couche 2 Sybil gate).
    {
        let db_guard = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let contributor_reg =
            nexus_coordinator_rs::contributor_registry::ContributorRegistry::new(&db_guard);
        let sig = state.pow_keypair.sign(artifact_hash_hex.as_bytes());
        let sig_hex = hex::encode(sig);
        let attestation_json = serde_json::json!({
            "artifact_hash": artifact_hash_hex,
            "commit_sha": commit_sha,
            "repo_url": repo_url,
        })
        .to_string();
        if let Err(e) = contributor_reg.record(
            &state.node_id,
            &state.node_id,
            &commit_sha,
            &repo_url,
            &sig_hex,
            &attestation_json,
        ) {
            debug!(error = %e, "contributor attestation record failed (non-fatal)");
        }
    }

    let zip_bytes = match add_to_zip(
        &zip_bytes,
        "provenance.json",
        &provenance::provenance_to_json(&prov),
    ) {
        Ok(b) => b,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("provenance inject: {e}"),
            );
        }
    };
    debug!(size = zip_bytes.len(), "deploy-from-repo: provenance added");

    let blobs = BlobsClient::new(state.node.blobs_store());
    let hash_hex = match blobs.add_bytes(zip_bytes).await {
        Ok(hash) => hex::encode(hash),
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("blob store: {e}"),
            );
        }
    };
    debug!(hash = %hash_hex, "deploy-from-repo: blob stored");

    let prov_hash = provenance::provenance_blake3_hex(&prov);

    publish_announcement(
        &state,
        AnnouncementParams {
            project_name: &req.project_name,
            category: &req.category,
            description: &req.description,
            apps: &req.apps,
            archive_hash: &hash_hex,
            repo_url: Some(&repo_url),
            provenance_hash: Some(&prov_hash),
            is_open_source: true,
        },
    )
    .await;

    (
        StatusCode::OK,
        Json(DeployResponse {
            deployed: true,
            hash: hash_hex,
            provenance_hash: Some(prov_hash),
            commit_sha: Some(commit_sha),
        }),
    )
        .into_response()
}

pub async fn deploy_private(State(state): State<Arc<DaemonHttpState>>, body: Bytes) -> Response {
    debug!(size = body.len(), "POST /api/v1/deploy");

    if body.len() > MAX_DEPLOY_BYTES {
        return error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!(
                "upload exceeds {} MB limit",
                MAX_DEPLOY_BYTES / (1024 * 1024)
            ),
        );
    }

    if let Err(e) = validate_zip(&body) {
        return error_response(StatusCode::BAD_REQUEST, &e);
    }

    let blobs = BlobsClient::new(state.node.blobs_store());
    let hash_hex = match blobs.add_bytes(body.to_vec()).await {
        Ok(hash) => hex::encode(hash),
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("blob store: {e}"),
            );
        }
    };
    debug!(hash = %hash_hex, "deploy: blob stored");

    (
        StatusCode::OK,
        Json(DeployResponse {
            deployed: true,
            hash: hash_hex,
            provenance_hash: None,
            commit_sha: None,
        }),
    )
        .into_response()
}

struct AnnouncementParams<'a> {
    project_name: &'a str,
    category: &'a str,
    description: &'a str,
    apps: &'a [String],
    archive_hash: &'a str,
    repo_url: Option<&'a str>,
    provenance_hash: Option<&'a str>,
    is_open_source: bool,
}

async fn publish_announcement(state: &DaemonHttpState, params: AnnouncementParams<'_>) {
    let AnnouncementParams {
        project_name,
        category,
        description,
        apps,
        archive_hash,
        repo_url,
        provenance_hash,
        is_open_source,
    } = params;
    use nexus_shell_daemon_core::browse::{BrowseEntry, BrowseSource, BrowseStatus};
    use nexus_shell_daemon_core::publish::ProjectAnnouncement;

    let mut announcement = ProjectAnnouncement::new(
        state.node_id.clone(),
        project_name.to_string(),
        category.to_string(),
        description.to_string(),
        apps.to_vec(),
    );

    if let Some(url) = repo_url {
        announcement = announcement.with_repo_url(url.to_string());
    }
    if let Some(hash) = provenance_hash {
        announcement = announcement.with_provenance_hash(hash.to_string());
    }
    if is_open_source {
        announcement = announcement.with_open_source(true);
    }

    if let Ok(ticket_str) = crate::http::mint_blob_ticket(state, archive_hash).await {
        announcement = announcement.with_archive_ticket(ticket_str);
    }

    let sender_guard = state.gossip_sender.read().await;
    if let Some(sender) = sender_guard.as_ref() {
        if let Ok(payload) = announcement.to_gossip_bytes() {
            if let Ok(envelope) = crate::http::wrap_payload_with_pow(state, &payload) {
                let _ = sender.broadcast(envelope).await;
            }
        }
    }
    drop(sender_guard);

    let browse_entry = BrowseEntry {
        project_id: state.node_id.clone(),
        project_name: project_name.to_string(),
        category: category.to_string(),
        description: description.to_string(),
        curator_pubkey: String::new(),
        curator_name: "Self-published".into(),
        source: BrowseSource::Direct,
        status: BrowseStatus::Reachable,
        last_probed_at: None,
        archive_ticket: announcement.archive_ticket.clone(),
        archive_hash: Some(archive_hash.to_string()),
        repo_url: repo_url.map(String::from),
        provenance_hash: provenance_hash.map(String::from),
        is_open_source,
    };
    state.browse_aggregator.add_direct_entry(browse_entry);
}

fn error_response(status: StatusCode, msg: &str) -> Response {
    (
        status,
        Json(DeployError {
            error: msg.to_string(),
        }),
    )
        .into_response()
}

fn is_valid_sha(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

async fn is_repo_public(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap_or_default();
    match client.head(url).send().await {
        Ok(resp) => resp.status() == reqwest::StatusCode::OK,
        Err(_) => false,
    }
}

async fn clone_repo(repo_url: &str, dest: &Path, sha: Option<&str>) -> Result<(), String> {
    run_git(
        &[
            "git",
            "clone",
            "--depth",
            "1",
            "--single-branch",
            repo_url,
            &dest.to_string_lossy(),
        ],
        Duration::from_secs(CLONE_TIMEOUT_SECS),
        "clone",
    )
    .await?;

    if let Some(sha) = sha {
        run_git(
            &[
                "git",
                "-C",
                &dest.to_string_lossy(),
                "fetch",
                "--depth",
                "1",
                "origin",
                sha,
            ],
            Duration::from_secs(CLONE_TIMEOUT_SECS),
            "fetch",
        )
        .await?;

        run_git(
            &[
                "git",
                "-C",
                &dest.to_string_lossy(),
                "checkout",
                "FETCH_HEAD",
            ],
            Duration::from_secs(CHECKOUT_TIMEOUT_SECS),
            "checkout",
        )
        .await?;
    }

    Ok(())
}

async fn run_git(cmd: &[&str], timeout: Duration, action: &str) -> Result<(), String> {
    let child = tokio::process::Command::new(cmd[0])
        .args(&cmd[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("git {action} spawn: {e}"))?;

    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let detail: String = stderr.chars().take(500).collect();
                return Err(format!("git {action} failed: {detail}"));
            }
            Ok(())
        }
        Ok(Err(e)) => Err(format!("git {action}: {e}")),
        Err(_) => Err(format!(
            "git {action} timed out after {}s",
            timeout.as_secs()
        )),
    }
}

async fn git_rev_parse(repo_dir: &Path) -> Result<String, String> {
    let output = tokio::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .await
        .map_err(|e| format!("git rev-parse: {e}"))?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[derive(Debug, Deserialize)]
struct SbfbJson {
    node_id: String,
}

fn read_sbfb_json(repo_dir: &Path) -> Result<SbfbJson, String> {
    let path = repo_dir.join("SBFB.json");
    if !path.is_file() {
        return Err("repository must contain SBFB.json at root".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("read SBFB.json: {e}"))?;
    serde_json::from_str::<SbfbJson>(&text).map_err(|e| format!("invalid SBFB.json: {e}"))
}

fn zip_directory(src: &Path) -> Result<Vec<u8>, io::Error> {
    let mut buf = io::Cursor::new(Vec::new());
    {
        let mut zw = zip::ZipWriter::new(&mut buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        add_dir_to_zip(&mut zw, src, src, &options)?;
        zw.finish()?;
    }
    Ok(buf.into_inner())
}

fn add_dir_to_zip(
    zw: &mut zip::ZipWriter<&mut io::Cursor<Vec<u8>>>,
    root: &Path,
    dir: &Path,
    options: &zip::write::SimpleFileOptions,
) -> Result<(), io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        if name == ".git" || name.starts_with(".git/") {
            continue;
        }
        if name.contains("..") || name.starts_with('/') {
            warn!(path = %name, "skipping suspicious path");
            continue;
        }
        if path.is_symlink() {
            warn!(path = %name, "skipping symlink");
            continue;
        }

        if path.is_dir() {
            add_dir_to_zip(zw, root, &path, options)?;
        } else {
            zw.start_file(&name, *options).map_err(io::Error::other)?;
            let mut f = std::fs::File::open(&path)?;
            let mut content = Vec::new();
            f.read_to_end(&mut content)?;
            zw.write_all(&content)?;
        }
    }
    Ok(())
}

fn add_to_zip(zip_bytes: &[u8], name: &str, content: &str) -> Result<Vec<u8>, io::Error> {
    let mut buf = io::Cursor::new(zip_bytes.to_vec());
    {
        let mut zw = zip::ZipWriter::new_append(&mut buf).map_err(io::Error::other)?;
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zw.start_file(name, options).map_err(io::Error::other)?;
        zw.write_all(content.as_bytes())?;
        zw.finish().map_err(io::Error::other)?;
    }
    Ok(buf.into_inner())
}

fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = walkdir(path) {
        for size in entries {
            total = total.saturating_add(size);
        }
    }
    total
}

fn walkdir(path: &Path) -> Result<Vec<u64>, io::Error> {
    let mut sizes = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            sizes.extend(walkdir(&entry.path())?);
        } else {
            sizes.push(entry.metadata()?.len());
        }
    }
    Ok(sizes)
}

fn validate_zip(data: &[u8]) -> Result<(), String> {
    let reader = io::Cursor::new(data);
    let archive = zip::ZipArchive::new(reader).map_err(|e| format!("invalid zip archive: {e}"))?;
    let names: Vec<&str> = archive.file_names().collect();
    if !names.contains(&"index.html") {
        return Err("zip archive must contain an index.html at the root".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_sha_accepts_40_hex() {
        assert!(is_valid_sha("abc123def456abc123def456abc123def456abc1"));
    }

    #[test]
    fn valid_sha_rejects_short() {
        assert!(!is_valid_sha("abc123"));
    }

    #[test]
    fn valid_sha_rejects_non_hex() {
        assert!(!is_valid_sha("xyz123def456abc123def456abc123def456abc1"));
    }

    #[test]
    fn validate_zip_requires_index_html() {
        let mut buf = io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            zw.start_file("other.html", opts).unwrap();
            zw.write_all(b"hello").unwrap();
            zw.finish().unwrap();
        }
        assert!(validate_zip(&buf.into_inner()).is_err());
    }

    #[test]
    fn validate_zip_accepts_with_index() {
        let mut buf = io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            zw.start_file("index.html", opts).unwrap();
            zw.write_all(b"<html></html>").unwrap();
            zw.finish().unwrap();
        }
        assert!(validate_zip(&buf.into_inner()).is_ok());
    }

    #[test]
    fn zip_and_add_to_zip() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("index.html"), "<html></html>").unwrap();
        let zip_bytes = zip_directory(tmp.path()).unwrap();
        assert!(validate_zip(&zip_bytes).is_ok());

        let zip_bytes = add_to_zip(&zip_bytes, "provenance.json", r#"{"test": true}"#).unwrap();
        let reader = io::Cursor::new(&zip_bytes);
        let archive = zip::ZipArchive::new(reader).unwrap();
        let names: Vec<&str> = archive.file_names().collect();
        assert!(names.contains(&"index.html"));
        assert!(names.contains(&"provenance.json"));
    }

    #[test]
    fn dir_size_counts_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.txt"), "hello").unwrap();
        std::fs::write(tmp.path().join("b.txt"), "world!").unwrap();
        let size = dir_size(tmp.path());
        assert_eq!(size, 11);
    }

    #[test]
    fn sbfb_json_parse() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("SBFB.json"), r#"{"node_id": "abc123"}"#).unwrap();
        let sbfb = read_sbfb_json(tmp.path()).unwrap();
        assert_eq!(sbfb.node_id, "abc123");
    }

    #[test]
    fn sbfb_json_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_sbfb_json(tmp.path()).is_err());
    }
}
