// SPDX-License-Identifier: AGPL-3.0-or-later
//! CAS file storage endpoints — port of api/files.py (Sprint 43 Phase B).
//!
//! Simplified from the Python per-app AppFileStore to a daemon-level
//! content-addressed store at `~/.sbfb/files/`. Files are keyed by
//! SHA-256 hash. Manifests are stored alongside as JSON.

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::http::DaemonHttpState;

const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

fn files_dir(override_home: Option<&std::path::Path>) -> Option<PathBuf> {
    let home = override_home
        .map(|p| p.to_path_buf())
        .or_else(nexus_shell_daemon_core::auth::sbfb_home);
    home.map(|d| d.join("files"))
}

fn blob_path(sha: &str, override_home: Option<&std::path::Path>) -> Option<PathBuf> {
    files_dir(override_home).map(|d| d.join(format!("{sha}.blob")))
}

fn manifest_path(sha: &str, override_home: Option<&std::path::Path>) -> Option<PathBuf> {
    files_dir(override_home).map(|d| d.join(format!("{sha}.manifest.json")))
}

fn validate_sha256(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    pub sha256: String,
    pub size: u64,
    pub content_type: String,
    pub original_name: String,
}

pub async fn upload_file(
    State(state): State<Arc<DaemonHttpState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<(StatusCode, Json<FileManifest>), (StatusCode, String)> {
    if body.len() > MAX_UPLOAD_BYTES {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "upload exceeds maximum allowed size of {} bytes ({} MB)",
                MAX_UPLOAD_BYTES,
                MAX_UPLOAD_BYTES / (1024 * 1024)
            ),
        ));
    }

    let content_type: String = headers
        .get("x-content-type")
        .or_else(|| headers.get("content-type"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .chars()
        .filter(|&c| c != '\r' && c != '\n')
        .collect();

    let original_name: String = headers
        .get("x-original-name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("upload")
        .chars()
        .filter(|&c| c != '"' && c != '\r' && c != '\n')
        .collect();

    use sha2::{Digest, Sha256};
    let sha_hex = {
        let mut hasher = Sha256::new();
        hasher.update(&body);
        hex::encode(hasher.finalize())
    };

    let dir = files_dir(state.sbfb_home.as_deref()).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "cannot resolve SBFB_HOME".into(),
    ))?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let bp = dir.join(format!("{sha_hex}.blob"));
    let mp = dir.join(format!("{sha_hex}.manifest.json"));

    let manifest = FileManifest {
        sha256: sha_hex.clone(),
        size: body.len() as u64,
        content_type,
        original_name,
    };

    std::fs::write(&bp, &body).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    std::fs::write(&mp, &manifest_json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!(
        sha256 = %sha_hex,
        size = body.len(),
        "file upload complete"
    );

    Ok((StatusCode::CREATED, Json(manifest)))
}

pub async fn get_manifest(
    State(state): State<Arc<DaemonHttpState>>,
    Path(sha256): Path<String>,
) -> Result<Json<FileManifest>, (StatusCode, String)> {
    if !validate_sha256(&sha256) {
        return Err((StatusCode::BAD_REQUEST, "invalid sha256 format".into()));
    }
    let mp = manifest_path(&sha256, state.sbfb_home.as_deref()).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "cannot resolve SBFB_HOME".into(),
    ))?;
    let body = std::fs::read_to_string(&mp).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            format!("no manifest for sha256={sha256}"),
        )
    })?;
    let manifest: FileManifest = serde_json::from_str(&body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(manifest))
}

pub async fn stream_file(
    State(state): State<Arc<DaemonHttpState>>,
    Path(sha256): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if !validate_sha256(&sha256) {
        return Err((StatusCode::BAD_REQUEST, "invalid sha256 format".into()));
    }
    let mp = manifest_path(&sha256, state.sbfb_home.as_deref()).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "cannot resolve SBFB_HOME".into(),
    ))?;
    let manifest_body = std::fs::read_to_string(&mp).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            format!("no manifest for sha256={sha256}"),
        )
    })?;
    let manifest: FileManifest = serde_json::from_str(&manifest_body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let bp = blob_path(&sha256, state.sbfb_home.as_deref()).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "cannot resolve SBFB_HOME".into(),
    ))?;
    let data = std::fs::read(&bp).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            format!("CAS blob missing for sha256={sha256}"),
        )
    })?;

    let response = axum::http::Response::builder()
        .header("content-type", &manifest.content_type)
        .header(
            "content-disposition",
            format!("inline; filename=\"{}\"", manifest.original_name),
        )
        .header("x-nexus-sha256", &sha256)
        .body(Body::from(data))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_sha256_valid() {
        assert!(validate_sha256(&"a".repeat(64)));
    }

    #[test]
    fn validate_sha256_short() {
        assert!(!validate_sha256("abc"));
    }

    #[test]
    fn validate_sha256_non_hex() {
        assert!(!validate_sha256(&"g".repeat(64)));
    }

    #[test]
    fn manifest_roundtrip() {
        let m = FileManifest {
            sha256: "a".repeat(64),
            size: 1234,
            content_type: "image/png".into(),
            original_name: "test.png".into(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let parsed: FileManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.sha256, m.sha256);
        assert_eq!(parsed.size, 1234);
    }

    #[test]
    fn max_upload_50mb() {
        assert_eq!(MAX_UPLOAD_BYTES, 50 * 1024 * 1024);
    }

    #[test]
    fn files_dir_uses_sbfb_home() {
        let dir = files_dir(None);
        if let Some(d) = dir {
            assert!(d.ends_with("files"));
        }
    }

    #[test]
    fn files_dir_override_home() {
        let tmp = std::path::Path::new("/tmp/test-sbfb");
        let dir = files_dir(Some(tmp)).unwrap();
        assert_eq!(dir, tmp.join("files"));
    }
}
