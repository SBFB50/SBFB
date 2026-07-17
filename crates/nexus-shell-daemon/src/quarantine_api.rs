// SPDX-License-Identifier: AGPL-3.0-or-later
//! Quarantine queue endpoints (Sprint 45 Phase A, port of quarantine.py).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tracing::debug;

use crate::http::DaemonHttpState;

const QUARANTINE_TTL_SECS: i64 = 900;

#[derive(Debug, Deserialize)]
pub struct QuarantineListQuery {
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "pending".into()
}

pub async fn list_quarantine(
    State(state): State<Arc<DaemonHttpState>>,
    Query(query): Query<QuarantineListQuery>,
) -> impl IntoResponse {
    debug!(status = %query.status, "GET /api/v1/quarantine");
    let db = match state.coordinator_db.lock() {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "db lock poisoned"})),
            )
                .into_response();
        }
    };

    let queue =
        nexus_coordinator_rs::quarantine_queue::QuarantineQueue::new(&db, QUARANTINE_TTL_SECS);
    let result = if query.status == "all" {
        let mut all = Vec::new();
        for status in &["pending", "flushed", "dropped"] {
            match queue.list_by_status(status) {
                Ok(entries) => all.extend(entries),
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": format!("{e}")})),
                    )
                        .into_response();
                }
            }
        }
        Ok(all)
    } else {
        queue.list_by_status(&query.status)
    };

    match result {
        Ok(entries) => {
            let count = entries.len();
            let entries: Vec<serde_json::Value> = entries
                .into_iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "topic": e.topic,
                        "sender_pubkey_hex": e.sender_pubkey_hex,
                        "payload_json": e.payload_json,
                        "received_at": e.received_at,
                        "rate_strikes": e.rate_strikes,
                        "pow_status": e.pow_status,
                        "flush_status": e.flush_status,
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({"entries": entries, "count": count})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

pub async fn flush_quarantine(
    State(state): State<Arc<DaemonHttpState>>,
    Path(row_id): Path<i64>,
) -> impl IntoResponse {
    debug!(row_id, "POST /api/v1/quarantine/:id/flush");
    let db = match state.coordinator_db.lock() {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "db lock poisoned"})),
            )
                .into_response();
        }
    };

    let queue =
        nexus_coordinator_rs::quarantine_queue::QuarantineQueue::new(&db, QUARANTINE_TTL_SECS);
    match queue.flush(row_id) {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"updated": true, "row_id": row_id, "new_status": "flushed"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("row {row_id} not found or already non-pending")})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

pub async fn drop_quarantine(
    State(state): State<Arc<DaemonHttpState>>,
    Path(row_id): Path<i64>,
) -> impl IntoResponse {
    debug!(row_id, "POST /api/v1/quarantine/:id/drop");
    let db = match state.coordinator_db.lock() {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "db lock poisoned"})),
            )
                .into_response();
        }
    };

    let queue =
        nexus_coordinator_rs::quarantine_queue::QuarantineQueue::new(&db, QUARANTINE_TTL_SECS);
    match queue.drop_entry(row_id) {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"updated": true, "row_id": row_id, "new_status": "dropped"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("row {row_id} not found or already non-pending")})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use crate::test_support::*;

    #[test]
    fn quarantine_query_defaults() {
        let q: QuarantineListQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(q.status, "pending");
    }

    #[test]
    fn quarantine_query_with_status() {
        let q: QuarantineListQuery = serde_json::from_str(r#"{"status":"all"}"#).unwrap();
        assert_eq!(q.status, "all");
    }

    #[test]
    fn quarantine_ttl_is_15_min() {
        assert_eq!(QUARANTINE_TTL_SECS, 900);
    }

    // --- quarantine_api.rs (3 routes) ---

    #[tokio::test]
    async fn quarantine_list_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/quarantine")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
    }

    #[tokio::test]
    async fn quarantine_flush_not_found() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/quarantine/99999/flush")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn quarantine_drop_not_found() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/quarantine/99999/drop")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
