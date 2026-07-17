// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ephemeral preview + ProofCard loopback HTTP domain — extracted
//! verbatim from `http.rs` (Sprint 82 Phase S3, PO-10 extended
//! discipline: the domain's 6 tests co-migrated below via
//! `crate::test_support`). Two residual Sprint 68 singletons sharing a
//! file: they have no DTO/helper/state coupling with each other.
//!
//! `POST /api/v1/preview/load` stores a caller-supplied zip in the
//! in-memory `PreviewStore` (TTL-evicted; `MAX_PREVIEW_BYTES` ceiling →
//! 413 via `PreviewError::TooLarge`, Sprint 68 Phase B) and returns its
//! BLAKE3 hash for `/blob-serve/{hash}/..` rendering; `GET
//! /api/daemon/proof-card/{project_id}` aggregates browse entry +
//! curator vouches + provenance DB into the computed ProofCard evidence
//! score (Sprint 68 Phase A). `get_proof_card` is a cross-domain
//! AGGREGATOR by design, which is why it lives here and not in
//! `browse_api` (whose charter is bounded to browse+nodes). T0 tier:
//! the routes stay registered in `crate::http::build_router` inside
//! `authed_routes` (loopback bearer + Host + Origin) and re-point here
//! by full path; route paths, JSON shapes and status codes are
//! unchanged.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::http::DaemonHttpState;

// =================================================================
// Sprint 68 Phase B — Ephemeral preview load endpoint
// =================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewLoadResponse {
    pub hash: String,
}

pub(crate) async fn preview_load(
    State(state): State<Arc<DaemonHttpState>>,
    body: Bytes,
) -> impl IntoResponse {
    debug!(size = body.len(), "POST /api/v1/preview/load");
    match state.preview_store.load(body.to_vec()) {
        Ok(hash) => (StatusCode::OK, Json(PreviewLoadResponse { hash })).into_response(),
        Err(nexus_shell_daemon_core::preview::PreviewError::TooLarge { actual, limit }) => (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "error": format!("preview size {actual} exceeds limit {limit}")
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// =================================================================
// Sprint 68 Phase A — ProofCard evidence score endpoint
// =================================================================

pub(crate) async fn get_proof_card(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    // 1. Look up browse entry (direct entries from project announcements).
    let browse_entry = state.browse_aggregator.get_direct_entry(&project_id);

    // 2. Count distinct curators vouching for this project.
    let curator_snapshot = state.curator_runtime.list_snapshot();
    let mut curator_names: Vec<String> = Vec::new();
    let mut seen_pubkeys = std::collections::HashSet::new();
    for list_entry in &curator_snapshot {
        let curator_hex = hex::encode(list_entry.curator_pubkey);
        for project in &list_entry.list.entries {
            if project.project_id == project_id && seen_pubkeys.insert(curator_hex.clone()) {
                curator_names.push(list_entry.list.curator_name.clone());
            }
        }
    }

    // 3. Extract metadata from browse entry or curator lists.
    let (project_name, is_open_source, archive_hash, provenance_hash, entry_repo_url) =
        match &browse_entry {
            Some(e) => (
                e.project_name.clone(),
                e.is_open_source,
                e.archive_hash.clone(),
                e.provenance_hash.clone(),
                e.repo_url.clone(),
            ),
            None => {
                let name = curator_snapshot
                    .iter()
                    .flat_map(|le| le.list.entries.iter())
                    .find(|p| p.project_id == project_id)
                    .map(|p| p.project_name.clone());
                match name {
                    Some(n) => (n, false, None, None, None),
                    None => {
                        return (
                            StatusCode::NOT_FOUND,
                            Json(serde_json::json!({"error": "project not found"})),
                        )
                            .into_response();
                    }
                }
            }
        };

    // 4. Query provenance from the coordinator DB.
    let provenance_opt = {
        let db = match state.coordinator_db.lock() {
            Ok(guard) => guard,
            Err(_poisoned) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response();
            }
        };
        match db.get_provenance_by_project(&project_id) {
            Ok(record) => record,
            Err(e) => {
                tracing::error!("proof card DB query failed: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response();
            }
        }
    };

    // 5. Verify provenance signature if a record exists.
    let (provenance_verified, repo_url, commit_sha, deploy_timestamp) = match &provenance_opt {
        Some(record) => {
            let record_json = nexus_coordinator_rs::provenance::provenance_to_json(record);
            let verified = match hex::decode(&record.node_id) {
                Ok(bytes) if bytes.len() == 32 => {
                    let pub_bytes: [u8; 32] = bytes.try_into().unwrap();
                    nexus_coordinator_rs::provenance::verify_provenance(&record_json, &pub_bytes)
                }
                _ => false,
            };
            (
                verified,
                Some(record.repo_url.clone()),
                Some(record.commit_sha.clone()),
                Some(record.timestamp.clone()),
            )
        }
        None => (false, None, None, None),
    };

    let effective_repo_url = repo_url.or(entry_repo_url);

    // 6. Compute the ProofCard.
    let input = nexus_coordinator_rs::proof_card::ProofCardInput {
        project_id: project_id.clone(),
        project_name,
        provenance_verified,
        repo_url: effective_repo_url,
        commit_sha,
        is_open_source,
        archive_hash,
        provenance_hash,
        license_spdx: None,
        curator_count: seen_pubkeys.len(),
        curator_names,
        deploy_timestamp_rfc3339: deploy_timestamp,
    };

    let card = nexus_coordinator_rs::proof_card::compute_proof_card(input);
    (StatusCode::OK, Json(card)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_shell_daemon_core::browse::BrowseEntry;
    use tower::ServiceExt;

    use crate::test_support::*;

    // ---------------------------------------------------------
    // Sprint 68 Phase A — ProofCard endpoint
    // ---------------------------------------------------------

    #[tokio::test]
    async fn test_proof_card_endpoint_http() {
        use nexus_shell_daemon_core::browse::{BrowseSource, BrowseStatus};
        let state = mk_state().await;
        let project_id = "f".repeat(64);

        // Seed a direct browse entry so the handler finds metadata.
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: project_id.clone(),
            node_id: None,
            project_name: "test-app".into(),
            category: "tools".into(),
            description: "a test app".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Reachable,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: Some("deadbeef".into()),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });

        let app = build_test_router(state);
        let uri = format!("/api/daemon/proof-card/{project_id}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["project_id"], project_id);
        assert_eq!(json["project_name"], "test-app");
        assert_eq!(json["formula_version"], 1);
        assert_eq!(json["confidence"], 35);
    }

    #[tokio::test]
    async fn test_proof_card_endpoint_not_found() {
        let state = mk_state().await;
        let app = build_test_router(state);
        let unknown_id = "0".repeat(64);
        let uri = format!("/api/daemon/proof-card/{unknown_id}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // =============================================================
    // Sprint 68 Phase B — preview load tests
    // =============================================================

    #[tokio::test]
    async fn test_preview_load_returns_hash() {
        let state = mk_state().await;
        let app = build_test_router(state);
        let zip_bytes = make_test_zip();

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/preview/load")
                    .header("Content-Type", "application/octet-stream")
                    .body(axum::body::Body::from(zip_bytes))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let hash = json["hash"].as_str().unwrap();
        assert_eq!(hash.len(), 64, "BLAKE3 hash should be 64 hex chars");
    }

    #[tokio::test]
    async fn test_preview_blob_serve_accessible() {
        let state = mk_state().await;
        let zip_bytes = make_test_zip();
        let hash = state.preview_store.load(zip_bytes).unwrap();

        let app = build_test_router(state);
        let uri = format!("/blob-serve/{hash}/index.html");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        assert!(
            std::str::from_utf8(&body)
                .unwrap()
                .contains("<body>test</body>")
        );
    }

    #[tokio::test]
    async fn test_preview_eviction_after_ttl() {
        use nexus_shell_daemon_core::preview::PreviewStore;
        let store = PreviewStore::new(std::time::Duration::from_millis(1));
        let data = b"ephemeral zip".to_vec();
        let hash = store.load(data).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        store.evict_expired();
        assert!(!store.has(&hash));
    }

    #[tokio::test]
    async fn test_preview_max_size_rejected() {
        let state = mk_state().await;
        let app = build_test_router(state);
        let oversized = vec![0u8; nexus_shell_daemon_core::preview::MAX_PREVIEW_BYTES + 1];

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/preview/load")
                    .header("Content-Type", "application/octet-stream")
                    .body(axum::body::Body::from(oversized))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}
