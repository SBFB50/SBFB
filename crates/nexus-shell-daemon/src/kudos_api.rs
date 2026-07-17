// SPDX-License-Identifier: AGPL-3.0-or-later
//! Kudos list + leaderboard endpoints (Sprint 44 Phase B).
//!
//! Completes the kudos API surface started in S36 (get + verify).
//! - `GET /api/v1/kudos/entries` — list all entries, optional worker filter
//! - `GET /api/v1/kudos/{project_id}/leaderboard` — top contributors
//! - `GET /api/v1/contributor/{node_id}` — one node's contribution standing
//!   across all projects (Sprint 76 Phase E, D4)

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::http::DaemonHttpState;

#[derive(Debug, Deserialize)]
pub struct KudosListQuery {
    #[serde(default)]
    pub worker_node_id: Option<String>,
    #[serde(default = "default_entries_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_entries_limit() -> usize {
    100
}

#[derive(Debug, Serialize)]
pub struct KudosEntryResponse {
    pub entry_id: String,
    pub worker_node_id: String,
    pub task_id: String,
    pub project_id: String,
    pub amount: u64,
    pub created_at: u64,
    pub entry_hash: String,
}

pub async fn list_entries(
    State(state): State<Arc<DaemonHttpState>>,
    Query(query): Query<KudosListQuery>,
) -> impl IntoResponse {
    debug!(?query.worker_node_id, "GET /api/v1/kudos/entries");
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
    match db.list_kudos_entries(query.worker_node_id.as_deref()) {
        Ok(all_entries) => {
            let total_count = all_entries.len();
            let capped_limit = query.limit.min(500);
            let entries: Vec<KudosEntryResponse> = all_entries
                .into_iter()
                .skip(query.offset)
                .take(capped_limit)
                .map(|e| KudosEntryResponse {
                    entry_id: e.entry_id,
                    worker_node_id: e.worker_node_id,
                    task_id: e.task_id,
                    project_id: e.project_id,
                    amount: e.amount,
                    created_at: e.created_at,
                    entry_hash: e.entry_hash,
                })
                .collect();
            let count = entries.len();
            (
                StatusCode::OK,
                Json(serde_json::json!({"entries": entries, "count": count, "total_count": total_count})),
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

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub worker_node_id: String,
    pub total_kudos: u64,
}

pub async fn leaderboard(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    debug!(id = %project_id, "GET /api/v1/kudos/:id/leaderboard");
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
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    match nexus_coordinator_rs::kudos_ledger::get_project_kudos(&db, &project_id, now_secs) {
        Ok(kudos) => {
            let entries: Vec<LeaderboardEntry> = kudos
                .contributors
                .into_iter()
                .map(|c| LeaderboardEntry {
                    worker_node_id: c.worker_node_id,
                    total_kudos: c.total,
                })
                .collect();
            let count = entries.len();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "project_id": project_id,
                    "leaderboard": entries,
                    "count": count,
                })),
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

/// Sprint 76 Phase E (D4) — one node's contribution dashboard.
///
/// Second aggregation view over the existing kudos ledger, keyed on
/// `worker_node_id` (the node's 64-hex Ed25519 pubkey — the same id the
/// ledger credits). Mirror of [`leaderboard`] but per-node instead of
/// per-project: returns the node's EMA-decayed kudos, tasks served (=
/// quorum-validated ledger lines), and a per-project breakdown. It is a
/// self-view, NOT a network-wide ranking. Lives under `authed_routes`
/// (loopback bearer + Host + Origin gate). GPU-hours are intentionally
/// absent here: they are a local, non-attested figure read from the
/// worker's `usage.json` and never aggregated server-side.
pub async fn contributor_dashboard(
    State(state): State<Arc<DaemonHttpState>>,
    Path(node_id): Path<String>,
) -> impl IntoResponse {
    debug!(node = %node_id, "GET /api/v1/contributor/:node_id");
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
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    match nexus_coordinator_rs::kudos_ledger::get_contributor_summary(&db, &node_id, now_secs) {
        Ok(summary) => {
            let per_project: Vec<serde_json::Value> = summary
                .per_project
                .into_iter()
                .map(|p| {
                    serde_json::json!({
                        "project_id": p.project_id,
                        "effective_kudos": p.effective_total,
                        "tasks_served": p.tasks_served,
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "worker_node_id": summary.worker_node_id,
                    "effective_kudos": summary.effective_total,
                    "tasks_served": summary.tasks_served,
                    "per_project": per_project,
                })),
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use crate::test_support::*;

    #[test]
    fn kudos_entry_response_serializes() {
        let e = KudosEntryResponse {
            entry_id: "e1".into(),
            worker_node_id: "w1".into(),
            task_id: "t1".into(),
            project_id: "p1".into(),
            amount: 100,
            created_at: 1700000000,
            entry_hash: "abc".into(),
        };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json["amount"], 100);
        assert_eq!(json["worker_node_id"], "w1");
    }

    #[test]
    fn leaderboard_entry_serializes() {
        let e = LeaderboardEntry {
            worker_node_id: "w1".into(),
            total_kudos: 500,
        };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json["total_kudos"], 500);
    }

    #[test]
    fn kudos_list_query_defaults() {
        let q: KudosListQuery = serde_json::from_str("{}").unwrap();
        assert!(q.worker_node_id.is_none());
        assert_eq!(q.limit, 100);
        assert_eq!(q.offset, 0);
    }

    // --- kudos_api.rs (2 routes) ---

    #[tokio::test]
    async fn kudos_entries_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["entries"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn kudos_leaderboard_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-test/leaderboard")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["leaderboard"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn contributor_dashboard_aggregates_node_credits() {
        // Sprint 76 Phase E (D4): the /contributor/{node_id} route returns
        // the node's cross-project standing — mirror of leaderboard but
        // per-node. Credit one node across two projects, then read it back.
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "node-x", "t1", 100, 1_000)
                .unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p2", "node-x", "t2", 50, 1_000)
                .unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "node-y", "t3", 10, 1_000)
                .unwrap();
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/contributor/node-x")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["worker_node_id"], "node-x");
        assert_eq!(body["tasks_served"], 2, "node-x served 2 tasks");
        assert!(body["effective_kudos"].as_u64().unwrap() > 0);
        assert_eq!(
            body["per_project"].as_array().unwrap().len(),
            2,
            "node-x served 2 distinct projects"
        );
    }

    #[tokio::test]
    async fn contributor_dashboard_empty_for_unknown_node() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/contributor/nobody")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["tasks_served"], 0);
        assert!(body["per_project"].as_array().unwrap().is_empty());
    }

    // --- Pagination tests (debt items 2 + 4) ---

    #[tokio::test]
    async fn kudos_entries_with_limit_offset() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w1", "t1", 10, 1_000).unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w2", "t2", 20, 1_000).unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w3", "t3", 30, 1_000).unwrap();
        }
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries?limit=2&offset=0")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 2);
        assert_eq!(body["total_count"], 3);

        let app2 = build_test_router(Arc::clone(&state));
        let resp2 = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries?limit=2&offset=2")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let body2: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp2.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body2["count"], 1);
        assert_eq!(body2["total_count"], 3);
    }
}
