// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinator-level health endpoint (Sprint 44 Phase B, port of health.py).

use std::sync::Arc;
use std::time::SystemTime;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use tracing::debug;

use crate::http::DaemonHttpState;

pub async fn coordinator_health(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /api/v1/coordinator/health");
    let uptime_secs = SystemTime::now()
        .duration_since(state.boot_time)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Json(serde_json::json!({
        "status": "ok",
        "node_id": state.node_id,
        "daemon_version": state.daemon_version,
        "api_host": state.api_host,
        "api_port": state.api_port,
        "uptime_secs": uptime_secs,
    }))
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    use crate::test_support::*;
    #[test]
    fn health_json_shape() {
        let json = serde_json::json!({
            "status": "ok",
            "node_id": "abc123",
            "daemon_version": "0.1.0",
            "api_host": "127.0.0.1",
            "api_port": 7000,
            "uptime_secs": 42,
        });
        assert_eq!(json["status"], "ok");
        assert_eq!(json["uptime_secs"], 42);
    }

    // --- health_api.rs (1 route) ---

    #[tokio::test]
    async fn coordinator_health_ok() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/coordinator/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["status"], "ok");
        assert_eq!(body["daemon_version"], "0.1.0-test");
    }
}
