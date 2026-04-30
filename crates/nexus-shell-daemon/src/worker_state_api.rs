// SPDX-License-Identifier: AGPL-3.0-or-later
//! Worker state proxy endpoint (Sprint 44 Phase C, port of worker_state.py).
//!
//! Reads `<nexus-grid-root>/worker/state.json` and returns it with
//! a staleness indicator (>15 s since `last_updated_at`).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use tracing::debug;

use crate::http::DaemonHttpState;

const STALE_THRESHOLD_SECS: u64 = 15;
const WORKER_STATE_SCHEMA_VERSION: u64 = 1;

fn worker_state_path() -> Option<std::path::PathBuf> {
    nexus_shell_daemon_core::paths::nexus_grid_root().map(|r| r.join("worker").join("state.json"))
}

pub async fn get_worker_state(State(_state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /api/v1/worker/state");

    let path = match worker_state_path() {
        Some(p) => p,
        None => {
            return (StatusCode::OK, Json(serde_json::json!({"running": false}))).into_response()
        }
    };

    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => {
            return (StatusCode::OK, Json(serde_json::json!({"running": false}))).into_response()
        }
    };

    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "running": false,
                    "error": "invalid JSON",
                })),
            )
                .into_response()
        }
    };

    let schema = parsed
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if schema != WORKER_STATE_SCHEMA_VERSION {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "running": false,
                "error": "schema mismatch",
            })),
        )
            .into_response();
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let last_updated = parsed
        .get("last_updated_at")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let stale = now.saturating_sub(last_updated) > STALE_THRESHOLD_SECS;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "running": true,
            "stale": stale,
            "state": parsed,
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_threshold_is_15() {
        assert_eq!(STALE_THRESHOLD_SECS, 15);
    }

    #[test]
    fn worker_state_path_contains_worker() {
        if let Some(p) = worker_state_path() {
            let s = p.to_string_lossy();
            assert!(s.contains("worker"), "path should contain 'worker': {s}");
            assert!(
                s.contains("state.json"),
                "path should contain 'state.json': {s}"
            );
        }
    }

    #[test]
    fn schema_version_is_1() {
        assert_eq!(WORKER_STATE_SCHEMA_VERSION, 1);
    }
}
