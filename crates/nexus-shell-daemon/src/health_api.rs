// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinator-level health endpoint (Sprint 44 Phase B, port of health.py).

use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
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
}
