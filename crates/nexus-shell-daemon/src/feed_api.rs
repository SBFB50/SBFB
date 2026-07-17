// SPDX-License-Identifier: AGPL-3.0-or-later
//! Feed + provenance loopback HTTP domain — extracted verbatim from
//! `http.rs` (Sprint 82 Phase S3, PO-10 extended discipline: the
//! domain's 9 router-driven tests co-migrated below via
//! `crate::test_support`).
//!
//! `GET /api/v1/project/{project_id}/provenance` serves the SLSA L1
//! self-attested provenance record with its Ed25519 verification status
//! (Sprint 63 Phase B); `GET /api/daemon/feed/cursor` exposes the
//! feed-sync resume position (Sprint 63 Phase C); `GET
//! /api/daemon/feed/entries` pages the public feed with
//! project_id/op_type filters (Sprint 67 Phase A tests). The
//! `#[serde(default)]` fields on `FeedEntriesQuery` are runtime
//! tolerance for minimal-JSON clients, not historical compat
//! (pre-launch policy). T0 tier: the routes stay registered in
//! `crate::http::build_router` inside `authed_routes` (loopback
//! bearer + Host + Origin) and re-point here by full path; route
//! paths, JSON shapes and status codes are unchanged. The write-side
//! `feed_insert`/`feed_status` handlers (internal-header gated) live in
//! `crate::feed_sync` and are NOT part of this read-only domain.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};

use crate::http::DaemonHttpState;

// =================================================================
// Sprint 63 Phase B — Provenance endpoint
// =================================================================

pub(crate) async fn get_provenance(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
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
        Ok(Some(record)) => {
            let record_json = nexus_coordinator_rs::provenance::provenance_to_json(&record);
            let provenance_hash = nexus_coordinator_rs::provenance::provenance_blake3_hex(&record);
            let (status, verified) = match hex::decode(&record.node_id) {
                Ok(bytes) if bytes.len() == 32 => {
                    let pub_bytes: [u8; 32] = bytes.try_into().unwrap();
                    let v = nexus_coordinator_rs::provenance::verify_provenance(
                        &record_json,
                        &pub_bytes,
                    );
                    if v {
                        ("verified", true)
                    } else {
                        ("failed", false)
                    }
                }
                _ => ("failed", false),
            };
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "record": record,
                    "verified": verified,
                    "status": status,
                    "provenance_hash": provenance_hash,
                })),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "absent",
                "verified": false,
                "record": null,
                "provenance_hash": null,
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("provenance DB query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 63 Phase C — Feed cursor endpoint
// =================================================================

pub(crate) async fn get_feed_cursor(
    State(state): State<Arc<DaemonHttpState>>,
) -> impl IntoResponse {
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
    match db.load_feed_cursor() {
        Ok(Some((last_seq, last_entry_hash))) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "last_seq": last_seq,
                "last_entry_hash": last_entry_hash,
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "last_seq": 0,
                "last_entry_hash": null,
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("feed cursor query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct FeedEntriesQuery {
    #[serde(default)]
    after_seq: Option<u64>,
    #[serde(default = "default_feed_limit")]
    limit: u64,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    op_type: Option<String>,
}

fn default_feed_limit() -> u64 {
    50
}

pub(crate) async fn get_feed_entries(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Query(params): axum::extract::Query<FeedEntriesQuery>,
) -> impl IntoResponse {
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

    let limit = params.limit.min(100);
    let after_seq = params.after_seq.unwrap_or(0);

    let rows = match db.get_feed_entries_after_seq(after_seq) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("feed entries query failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };

    let filtered: Vec<_> = rows
        .into_iter()
        .filter(|row| {
            if let Some(ref pid) = params.project_id {
                let payload: serde_json::Value =
                    serde_json::from_str(&row.payload).unwrap_or_default();
                if payload.get("project_id").and_then(|v| v.as_str()) != Some(pid.as_str()) {
                    return false;
                }
            }
            if let Some(ref ot) = params.op_type
                && row.op_type != *ot {
                    return false;
                }
            true
        })
        .take(limit as usize)
        .map(|row| {
            serde_json::json!({
                "seq": row.seq,
                "op_type": row.op_type,
                "payload": serde_json::from_str::<serde_json::Value>(&row.payload).unwrap_or_default(),
                "author": row.author,
                "entry_hash": row.entry_hash,
                "prev_hash": row.prev_hash,
                "created_at": row.created_at,
            })
        })
        .collect();

    let count = filtered.len();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "entries": filtered,
            "count": count,
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::KeyPair;
    use tower::ServiceExt;

    use crate::test_support::*;

    #[tokio::test]
    async fn provenance_endpoint_absent_status() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/project/nonexistent/provenance")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "absent");
        assert_eq!(json["verified"], false);
        assert!(json["record"].is_null());
        assert!(json["provenance_hash"].is_null());
    }

    #[tokio::test]
    async fn provenance_endpoint_found_and_verified() {
        let state = mk_state().await;
        let project_id = state.node_id.clone();
        let kp = &state.pow_keypair;
        let record = nexus_coordinator_rs::provenance::generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            kp,
        );
        {
            let db = state.coordinator_db.lock().unwrap();
            db.insert_provenance_record(&project_id, &record)
                .expect("insert");
        }

        let app = build_test_router(state);
        let uri = format!("/api/v1/project/{project_id}/provenance");
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
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["verified"], true);
        assert_eq!(json["status"], "verified");
        assert_eq!(json["record"]["repo_url"], "https://github.com/user/repo");
        assert_eq!(json["record"]["artifact_hash"], "deadbeef");
        assert_eq!(json["record"]["schema_version"], 1);
        assert!(
            json["provenance_hash"].as_str().is_some(),
            "response must include provenance_hash"
        );
        assert_eq!(json["provenance_hash"].as_str().unwrap().len(), 64);
    }

    #[tokio::test]
    async fn provenance_cross_node_verified() {
        let state = mk_state().await;
        let project_id = state.node_id.clone();
        let other_kp = KeyPair::generate();
        let record = nexus_coordinator_rs::provenance::generate_provenance(
            "https://github.com/other/repo",
            "abc123def456abc123def456abc123def456abc1",
            "cafebabe",
            &hex::encode(other_kp.public_bytes()),
            &other_kp,
        );
        {
            let db = state.coordinator_db.lock().unwrap();
            db.insert_provenance_record(&project_id, &record)
                .expect("insert");
        }

        let app = build_test_router(state);
        let uri = format!("/api/v1/project/{project_id}/provenance");
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
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["verified"], true);
        assert_eq!(json["status"], "verified");
        assert!(json["record"]["repo_url"].as_str().is_some());
    }

    #[tokio::test]
    async fn provenance_cross_node_tampered() {
        let state = mk_state().await;
        let project_id = state.node_id.clone();
        let signer_kp = KeyPair::generate();
        let mut record = nexus_coordinator_rs::provenance::generate_provenance(
            "https://github.com/tampered/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(signer_kp.public_bytes()),
            &signer_kp,
        );
        let impostor_kp = KeyPair::generate();
        record.node_id = hex::encode(impostor_kp.public_bytes());
        {
            let db = state.coordinator_db.lock().unwrap();
            db.insert_provenance_record(&project_id, &record)
                .expect("insert");
        }

        let app = build_test_router(state);
        let uri = format!("/api/v1/project/{project_id}/provenance");
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
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["verified"], false);
        assert_eq!(json["status"], "failed");
    }

    #[tokio::test]
    async fn provenance_endpoint_returns_app_version() {
        let state = mk_state().await;
        let project_id = state.node_id.clone();
        let kp = &state.pow_keypair;
        let mut record = nexus_coordinator_rs::provenance::generate_provenance(
            "https://github.com/user/versioned",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            kp,
        );
        record.app_version = Some("3.2.1".to_string());
        {
            let db = state.coordinator_db.lock().unwrap();
            db.insert_provenance_record(&project_id, &record)
                .expect("insert");
        }

        let app = build_test_router(state);
        let uri = format!("/api/v1/project/{project_id}/provenance");
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
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["record"]["app_version"], "3.2.1");
    }

    // -- Sprint 63 Phase C: feed cursor endpoint tests --

    #[tokio::test]
    async fn feed_cursor_empty_returns_zero() {
        let state = mk_state().await;
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/feed/cursor")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["last_seq"], 0);
        assert!(json["last_entry_hash"].is_null());
    }

    #[tokio::test]
    async fn feed_cursor_returns_saved_position() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            db.save_feed_cursor(42, "abcdef1234567890").expect("save");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/feed/cursor")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["last_seq"], 42);
        assert_eq!(json["last_entry_hash"], "abcdef1234567890");
    }

    // -- Sprint 67 Phase A: feed entries endpoint tests --

    fn insert_test_feed_entry(
        db: &nexus_coordinator_rs::db::CoordinatorDb,
        project_id: &str,
        op_type_str: &str,
    ) {
        let kp = nexus_core_rs::KeyPair::from_secret_bytes(&[42u8; 32]);
        let pk = hex::encode(kp.public_bytes());
        let op = serde_json::json!({
            "op_type": op_type_str,
            "project_id": project_id,
            "repo_url": "https://github.com/org/app",
            "commit_sha": "a".repeat(40),
            "artifact_hash": "b".repeat(64),
            "provenance_hash": "c".repeat(64),
            "is_open_source": true
        });
        nexus_coordinator_rs::public_feed::insert_feed_operation(db, op, &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();
    }

    #[tokio::test]
    async fn test_feed_entries_endpoint_paginated() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            let pid = "a1".repeat(32);
            insert_test_feed_entry(&db, &pid, "ReleasePublished");
            insert_test_feed_entry(&db, &pid, "ReleasePublished");
            insert_test_feed_entry(&db, &pid, "ReleasePublished");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/feed/entries?after_seq=1&limit=2")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["count"], 2);
        let entries = json["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0]["seq"].as_u64().unwrap() > 1);
    }

    #[tokio::test]
    async fn test_feed_entries_endpoint_filters_by_project_id() {
        let state = mk_state().await;
        let pid_a = "a1".repeat(32);
        let pid_b = "b2".repeat(32);
        {
            let db = state.coordinator_db.lock().unwrap();
            insert_test_feed_entry(&db, &pid_a, "ReleasePublished");
            insert_test_feed_entry(&db, &pid_b, "ReleasePublished");
            insert_test_feed_entry(&db, &pid_a, "ReleasePublished");
        }
        let app = build_test_router(state);
        let uri = format!("/api/daemon/feed/entries?project_id={pid_a}");
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
        assert_eq!(json["count"], 2);
        let entries = json["entries"].as_array().unwrap();
        for e in entries {
            assert_eq!(e["payload"]["project_id"].as_str().unwrap(), pid_a);
        }
    }
}
