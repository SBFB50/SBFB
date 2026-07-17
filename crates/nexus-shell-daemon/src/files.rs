// SPDX-License-Identifier: AGPL-3.0-or-later
//! CAS file storage endpoints — port of api/files.py (Sprint 43 Phase B).
//!
//! Simplified from the Python per-app AppFileStore to a daemon-level
//! content-addressed store at `~/.sbfb/files/`. Files are keyed by
//! SHA-256 hash. Manifests are stored alongside as JSON.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
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
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use crate::test_support::*;

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

    // --- files.rs (3 routes) ---

    #[tokio::test]
    async fn files_manifest_invalid_sha_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/files/not-a-valid-sha/manifest")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn files_manifest_not_found_404() {
        let app = build_test_router(mk_state().await);
        let sha = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}/manifest"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn files_stream_invalid_sha_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/files/bad-sha")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn files_stream_not_found_404() {
        let app = build_test_router(mk_state().await);
        let sha = "b".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn files_upload_too_large_413() {
        let app = build_test_router(mk_state().await);
        let big_body = vec![0u8; 50 * 1024 * 1024 + 1];
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(big_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    // --- files.rs happy path tests (3 routes) ---

    #[tokio::test]
    async fn files_upload_small_returns_201_with_sha() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "text/plain")
                    .header("x-original-name", "test.txt")
                    .body(axum::body::Body::from(b"hello world".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["sha256"].as_str().unwrap().len(), 64);
        assert_eq!(body["size"], 11);
        assert_eq!(body["original_name"], "test.txt");
    }

    #[tokio::test]
    async fn files_manifest_after_upload_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        let app1 = build_test_router(Arc::clone(&state));
        let resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "text/plain")
                    .body(axum::body::Body::from(b"manifest test".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let upload_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        let sha = upload_body["sha256"].as_str().unwrap();

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}/manifest"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["sha256"].as_str().unwrap(), sha);
    }

    #[tokio::test]
    async fn files_stream_after_upload_returns_content() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let content = b"stream test content";

        let app1 = build_test_router(Arc::clone(&state));
        let resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .body(axum::body::Body::from(content.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let upload_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        let sha = upload_body["sha256"].as_str().unwrap().to_owned();

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
        assert_eq!(&body_bytes[..], content);
    }
}
