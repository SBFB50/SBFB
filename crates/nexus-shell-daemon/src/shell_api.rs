// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shell discover endpoint (Sprint 44 Phase B, port of shell.py).
//!
//! Returns the list of running coordinators visible from this daemon.
//! Post-S45 (Python coordinator removed), the daemon IS the coordinator,
//! so this endpoint returns only self.

use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
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
    #[test]
    fn schema_version_is_1() {
        assert_eq!(super::SHELL_SCHEMA_VERSION, 1);
    }
}
