// SPDX-License-Identifier: AGPL-3.0-or-later
//! Warrant-canary HTTP endpoints — the full canary surface.
//!
//! Registry routes (Sprint 39 Phase C, moved here from `http.rs` in
//! Sprint 82 Phase S4): observed, network-health, freshness. Input
//! routes (Sprint 43 Phase C, port of remaining api/canary.py):
//! inject-rate and observed-divergence. Routes stay registered in
//! `crate::http::build_router` inside `authed_routes`; paths, JSON
//! shapes and status codes are unchanged.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::http::DaemonHttpState;

#[derive(Debug, Deserialize)]
pub struct InjectRateBody {
    pub inject_rate: i64,
}

#[derive(Debug, Serialize)]
pub struct InjectRateResponse {
    pub status: String,
    pub inject_rate: usize,
}

pub async fn set_inject_rate(
    State(state): State<Arc<DaemonHttpState>>,
    Json(body): Json<InjectRateBody>,
) -> Result<Json<InjectRateResponse>, (StatusCode, String)> {
    let manager = state.canary_input.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "canary_input manager not initialised".into(),
    ))?;
    let new_rate = body.inject_rate.max(1) as usize;
    manager.set_inject_rate(new_rate);
    let current = manager.policy().inject_rate;
    Ok(Json(InjectRateResponse {
        status: "updated".into(),
        inject_rate: current,
    }))
}

#[derive(Debug, Deserialize)]
pub struct DivergenceQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Serialize)]
pub struct DivergenceResponse {
    pub divergences: Vec<serde_json::Value>,
    pub count: usize,
}

pub async fn observed_divergence(
    State(state): State<Arc<DaemonHttpState>>,
    Query(query): Query<DivergenceQuery>,
) -> Result<Json<DivergenceResponse>, (StatusCode, String)> {
    let manager = state.canary_input.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "canary_input manager not initialised".into(),
    ))?;
    let capped = query.limit.min(100);
    let records = manager.recent_divergences(capped);
    let divergences: Vec<serde_json::Value> = records
        .iter()
        .filter_map(|r| serde_json::to_value(r).ok())
        .collect();
    let count = divergences.len();
    Ok(Json(DivergenceResponse { divergences, count }))
}

// =================================================================
// Sprint 39 Phase C — Canary registry HTTP endpoints
// =================================================================

pub(crate) async fn canary_observed(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let obs = match nexus_coordinator_rs::canary_registry::coerce_canary_payload(&payload) {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            )
                .into_response();
        }
    };
    match state.canary_registry.lock() {
        Ok(mut reg) => {
            reg.observe_canary(obs);
            (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn canary_network_health(
    State(state): State<Arc<DaemonHttpState>>,
) -> impl IntoResponse {
    match state.canary_registry.lock() {
        Ok(reg) => {
            let health = reg.network_health();
            (
                StatusCode::OK,
                Json(serde_json::to_value(&health).unwrap_or_default()),
            )
                .into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn canary_freshness(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(pubkey): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.canary_registry.lock() {
        Ok(reg) => {
            let freshness = reg.freshness(&pubkey);
            (
                StatusCode::OK,
                Json(serde_json::to_value(&freshness).unwrap_or_default()),
            )
                .into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
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
    fn inject_rate_body_deserializes() {
        let json = r#"{"inject_rate": 50}"#;
        let body: InjectRateBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.inject_rate, 50);
    }

    #[test]
    fn divergence_query_defaults() {
        let q: DivergenceQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(q.limit, 50);
    }

    #[test]
    fn inject_rate_response_serializes() {
        let resp = InjectRateResponse {
            status: "updated".into(),
            inject_rate: 42,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("42"));
    }

    #[tokio::test]
    async fn canary_observed_post_ok() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let payload = serde_json::json!({
            "version": 1,
            "pubkey_hex": "aa".repeat(32),
            "date": "2026-04-29",
            "headline": "All clear",
            "next_update": "2026-05-29",
            "signature_hex": "bb".repeat(64)
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/canary/observed")
                    .header("content-type", "application/json")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&payload).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn canary_network_health_get_ok() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/canary/network-health")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // --- canary_api.rs (3 routes) ---

    #[tokio::test]
    async fn canary_freshness_returns_200() {
        let app = build_test_router(mk_state().await);
        let pubkey = "aa".repeat(32);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/canary/freshness/{pubkey}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn canary_freshness_unknown_pubkey_returns_200() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/canary/freshness/unknown-key")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn canary_inject_rate_updates() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let body = serde_json::json!({"inject_rate": 50});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/inject-rate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(resp_body["status"], "updated");
        assert!(resp_body["inject_rate"].as_u64().is_some());
    }

    #[tokio::test]
    async fn canary_observed_divergence_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/canary/observed-divergence")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["divergences"].as_array().unwrap().is_empty());
    }
}
