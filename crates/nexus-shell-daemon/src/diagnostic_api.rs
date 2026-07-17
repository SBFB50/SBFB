// SPDX-License-Identifier: AGPL-3.0-or-later
//! Diagnostic HTTP endpoints.
//!
//! Fairness (Sprint 44 Phase B, port of diagnostic.py): Gini
//! coefficient, top-5% share and worker churn rate computed from the
//! kudos ledger. Neighborhood snapshot (Sprint 23 Phase E, moved here
//! from `http.rs` in Sprint 82 Phase S4): the node's own id plus the
//! subscribed curator pubkeys — the peers this daemon actively tracks.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use nexus_coordinator_rs::fairness;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::http::DaemonHttpState;

const DAY_SECS: u64 = 86400;

pub async fn fairness_metrics(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /api/v1/diagnostic/fairness");
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

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let contributions: Vec<f64> = match db.list_kudos_entries(None) {
        Ok(entries) => {
            let mut by_worker: std::collections::HashMap<
                &str,
                Vec<&nexus_coordinator_rs::types::KudosEntry>,
            > = std::collections::HashMap::new();
            for entry in &entries {
                by_worker
                    .entry(&entry.worker_node_id)
                    .or_default()
                    .push(entry);
            }
            by_worker
                .values()
                .map(|worker_entries| {
                    worker_entries
                        .iter()
                        .map(|e| {
                            let age_days = now.saturating_sub(e.created_at) / 86400;
                            e.amount as f64
                                * nexus_coordinator_rs::kudos_ledger::KUDOS_EMA_ALPHA
                                    .powi(age_days as i32)
                        })
                        .sum()
                })
                .collect()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("kudos_entries: {e}")})),
            )
                .into_response();
        }
    };

    let current_workers = match db.active_workers_since(now.saturating_sub(DAY_SECS)) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("active_workers_current: {e}")})),
            )
                .into_response();
        }
    };
    let previous_workers = match db.active_workers_since(now.saturating_sub(2 * DAY_SECS)) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("active_workers_previous: {e}")})),
            )
                .into_response();
        }
    };

    drop(db);

    let worker_count = contributions.len();
    let gini = fairness::compute_gini(&contributions);
    let top_5_pct_share = fairness::compute_top_k_share(&contributions, 5);
    let churn_rate = fairness::compute_churn_rate(&previous_workers, &current_workers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "gini": (gini * 10000.0).round() / 10000.0,
            "top_5_pct_share": (top_5_pct_share * 10000.0).round() / 10000.0,
            "churn_rate": (churn_rate * 10000.0).round() / 10000.0,
            "worker_count": worker_count,
        })),
    )
        .into_response()
}

/// Body of `GET /diagnostic/neighborhood`. Sprint 23 Phase E.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeighborhoodResponse {
    pub node_id: String,
    pub peers: Vec<String>,
}

/// `GET /diagnostic/neighborhood` — Sprint 23 Phase E. Returns the
/// node's own ID and the peer pubkeys currently in the daemon's
/// observable neighborhood. iroh exposes no DHT routing-table
/// enumeration (re-checked against 1.0.1 at the S81 Phase C bump:
/// only per-peer `Endpoint::remote_info(EndpointId)` exists — the
/// `remote_info_iter` once expected "post-0.98" never landed), so
/// the observable neighborhood is the set of subscribed curator
/// pubkeys — the peers this daemon actively tracks via gossip.
pub(crate) async fn diagnostic_neighborhood(
    State(state): State<Arc<DaemonHttpState>>,
) -> impl IntoResponse {
    debug!("GET /diagnostic/neighborhood");
    let peers = state.curator_runtime.subscribed_pubkeys_hex();
    (
        StatusCode::OK,
        Json(NeighborhoodResponse {
            node_id: state.node_id.clone(),
            peers,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use crate::test_support::*;

    #[test]
    fn day_secs_correct() {
        assert_eq!(DAY_SECS, 24 * 60 * 60);
    }

    #[test]
    fn rounding_precision() {
        let val = 0.12345_f64;
        let rounded = (val * 10000.0).round() / 10000.0;
        assert!((rounded - 0.1235).abs() < 1e-10);
    }

    // ---------------------------------------------------------
    // Sprint 23 Phase E: diagnostic neighborhood endpoint
    // ---------------------------------------------------------

    #[tokio::test]
    async fn diagnostic_neighborhood_returns_own_node_id_and_empty_peers() {
        let state = mk_state().await;
        let expected_node_id = state.node_id.clone();
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/diagnostic/neighborhood")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let neighborhood: NeighborhoodResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(neighborhood.node_id, expected_node_id);
        assert!(
            neighborhood.peers.is_empty(),
            "fresh node should have no known peers"
        );
    }

    // --- diagnostic_api.rs (1 route) ---

    #[tokio::test]
    async fn diagnostic_fairness_ok() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["worker_count"], 0);
    }

    #[tokio::test]
    async fn diagnostic_fairness_ema_on_nonempty_ledger() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w1", "t1", 100, 1_000).unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w2", "t2", 100, 1_000).unwrap();
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["worker_count"], 2);
        let gini = body["gini"].as_f64().unwrap();
        assert!(
            gini < 0.01,
            "two equal-contribution workers must have near-zero Gini (got {gini})"
        );
    }

    // --- Debt item 5: diagnostic error propagation ---

    #[tokio::test]
    async fn diagnostic_fairness_returns_500_on_corrupted_db() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            db.execute_batch_raw("DROP TABLE IF EXISTS kudos")
                .expect("drop kudos table");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(body["error"].as_str().unwrap().contains("kudos_entries"));
    }

    #[tokio::test]
    async fn diagnostic_fairness_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();
        assert!(state.coordinator_db.lock().is_err());

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
