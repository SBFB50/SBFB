//! Phase A HTTP surface for `nexus-shell-daemon`.
//!
//! The daemon's HTTP listener is loopback-only and reached by
//! the React shell exclusively through the coordinator
//! `/daemon/*` proxy (Sprint 7 D1). Phase A exposes two routes:
//!
//! - `GET /health` — liveness probe, fixed body, no locks held
//! - `GET /info`   — [`nexus_shell_daemon_core::state::DaemonStateSnapshot`]
//!
//! Phase C will grow `/curators` (GET/POST/DELETE), Phase D will
//! grow `/browse`. Those routes are deliberately **absent** here
//! — the Phase A skeleton must stay minimal so the audit gate
//! can isolate boot correctness from subscribe correctness.
//!
//! ## CORS
//!
//! The daemon trusts two and only two origins:
//!
//! - `http://127.0.0.1[:port]`
//! - `http://localhost[:port]`
//!
//! Every other origin is rejected before the handler runs.
//! Even though the shell is expected to talk through the
//! coordinator proxy, we keep a strict loopback CORS layer on
//! the daemon itself so a future direct-call path cannot
//! silently widen the trust model.

use std::sync::Arc;
use std::time::SystemTime;

use axum::{
    extract::State,
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use nexus_shell_daemon_core::state::{DaemonStateSnapshot, StateInputs};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::debug;

/// Shared state handed to every axum route.
///
/// Kept intentionally small in Phase A — a boot timestamp, the
/// iroh node id, the bound port, and the host. Phase C will add
/// an `Arc<DashMap<...>>` for received curator lists; Phase D
/// will add a browse aggregator. The fields added later must
/// remain `Clone` and cheap to read under contention because
/// `/info` is polled on every shell refresh.
#[derive(Debug, Clone)]
pub struct DaemonHttpState {
    pub node_id: String,
    pub daemon_version: String,
    pub boot_time: SystemTime,
    pub api_host: String,
    pub api_port: u16,
}

impl DaemonHttpState {
    fn snapshot(&self) -> DaemonStateSnapshot {
        DaemonStateSnapshot::from_inputs(StateInputs {
            node_id: self.node_id.clone(),
            daemon_version: self.daemon_version.clone(),
            boot_time: self.boot_time,
            api_host: self.api_host.clone(),
            api_port: self.api_port,
            subscribed_curators: Vec::new(), // Phase C
            known_lists: 0,                  // Phase C
            known_browse_entries: 0,         // Phase D
        })
    }
}

/// Build the Phase A axum [`Router`].
///
/// Every Phase A integration test calls into this function,
/// never into the concrete listener, so the router logic can be
/// tested end-to-end via `tower::ServiceExt::oneshot` without
/// ever binding a real TCP port.
pub fn build_router(state: Arc<DaemonHttpState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
        .with_state(state)
        .layer(loopback_cors_layer())
}

/// The loopback-only CORS layer. Accepts exactly the origins
/// `http://127.0.0.1[:PORT]` and `http://localhost[:PORT]`;
/// refuses everything else, including HTTPS variants (the
/// coordinator is HTTP loopback too, so HTTPS origins don't
/// make sense here).
fn loopback_cors_layer() -> CorsLayer {
    CorsLayer::new().allow_origin(AllowOrigin::predicate(
        |origin: &HeaderValue, _request_parts: &_| is_loopback_origin(origin),
    ))
}

/// Return `true` iff `origin` is an HTTP loopback URL with an
/// optional port and no path.
///
/// Kept as a pure function so the unit tests can exercise the
/// predicate without spinning up a full axum service.
pub fn is_loopback_origin(origin: &HeaderValue) -> bool {
    let Ok(s) = origin.to_str() else {
        return false;
    };
    let rest = match s.strip_prefix("http://") {
        Some(r) => r,
        None => return false,
    };

    // Split off an optional `:PORT` suffix. Paths are not
    // allowed in an `Origin` header per RFC 6454, but strip
    // defensively just in case.
    let host_port = rest.split('/').next().unwrap_or(rest);
    let (host, port_opt) = match host_port.rsplit_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (host_port, None),
    };

    if host != "127.0.0.1" && host != "localhost" {
        return false;
    }
    if let Some(p) = port_opt {
        if p.parse::<u16>().is_err() {
            return false;
        }
    }
    true
}

// =================================================================
// Handlers
// =================================================================

/// `GET /health` — liveness probe.
///
/// Returns 200 with a fixed JSON body containing the schema
/// version and the daemon's crate version. The shell uses this
/// to distinguish a running daemon from a stale proxy path.
async fn health(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /health");
    Json(serde_json::json!({
        "status": "ok",
        "schema_version": nexus_shell_daemon_core::state::SCHEMA_VERSION,
        "daemon_version": state.daemon_version,
    }))
}

/// `GET /info` — full [`DaemonStateSnapshot`] for the shell's
/// Browse / Curators page header.
async fn info(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /info");
    (StatusCode::OK, Json(state.snapshot()))
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    fn mk_state() -> Arc<DaemonHttpState> {
        Arc::new(DaemonHttpState {
            node_id: "deadbeef".repeat(8),
            daemon_version: "0.1.0-test".to_string(),
            boot_time: SystemTime::now(),
            api_host: "127.0.0.1".to_string(),
            api_port: 12345,
        })
    }

    #[tokio::test]
    async fn health_returns_200_with_fixed_shape() {
        let app = build_router(mk_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["daemon_version"], "0.1.0-test");
    }

    #[tokio::test]
    async fn info_returns_full_snapshot() {
        let app = build_router(mk_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let snap: DaemonStateSnapshot = serde_json::from_slice(&body).unwrap();
        assert_eq!(snap.schema_version, 1);
        assert_eq!(snap.node_id.len(), 64);
        assert_eq!(snap.api_host, "127.0.0.1");
        assert_eq!(snap.api_port, 12345);
        // Phase A always-zero curator/browse fields:
        assert!(snap.subscribed_curators.is_empty());
        assert_eq!(snap.known_lists, 0);
        assert_eq!(snap.known_browse_entries, 0);
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let app = build_router(mk_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/nope")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn loopback_origin_accepts_127_0_0_1_with_port() {
        let h = HeaderValue::from_static("http://127.0.0.1:3000");
        assert!(is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_accepts_localhost_without_port() {
        let h = HeaderValue::from_static("http://localhost");
        assert!(is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_remote_host() {
        let h = HeaderValue::from_static("http://example.com");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_https_scheme() {
        // Only http is allowed — an https origin on loopback
        // would imply a TLS stack we don't ship.
        let h = HeaderValue::from_static("https://127.0.0.1");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_malformed_port() {
        let h = HeaderValue::from_static("http://127.0.0.1:not-a-port");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_suffix_trick() {
        // `http://127.0.0.1.evil.com` must not pass as loopback
        // just because it starts with the right bytes.
        let h = HeaderValue::from_static("http://127.0.0.1.evil.com");
        assert!(!is_loopback_origin(&h));
    }
}
