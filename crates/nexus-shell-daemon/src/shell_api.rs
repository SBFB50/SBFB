// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shell discover endpoint (Sprint 44 Phase B, port of shell.py).
//!
//! Returns the list of running coordinators visible from this daemon.
//! Post-S45 (Python coordinator removed), the daemon IS the coordinator,
//! so this endpoint returns only self.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use tracing::debug;

use crate::http::DaemonHttpState;

const SHELL_SCHEMA_VERSION: u32 = 1;

pub async fn discover(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /api/v1/shell/discover");
    Json(serde_json::json!({
        "schema_version": SHELL_SCHEMA_VERSION,
        "coordinators": [{
            "node_id": state.node_id,
            "api_host": state.api_host,
            "api_port": state.api_port,
            "daemon_version": state.daemon_version,
        }],
        "count": 1,
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::to_bytes;
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    use crate::test_support::*;
    #[test]
    fn schema_version_is_1() {
        assert_eq!(super::SHELL_SCHEMA_VERSION, 1);
    }

    // --- shell_api.rs (1 route) ---

    #[tokio::test]
    async fn shell_discover_returns_self() {
        let state = mk_state().await;
        let expected_node_id = state.node_id.clone();
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/shell/discover")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 1);
        let coordinators = body["coordinators"].as_array().unwrap();
        assert_eq!(coordinators.len(), 1);
        assert_eq!(coordinators[0]["node_id"], expected_node_id);
    }
}
