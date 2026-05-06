// SPDX-License-Identifier: AGPL-3.0-or-later
//! Kudos list + leaderboard endpoints (Sprint 44 Phase B).
//!
//! Completes the kudos API surface started in S36 (get + verify).
//! - `GET /api/v1/kudos/entries` — list all entries, optional worker filter
//! - `GET /api/v1/kudos/{project_id}/leaderboard` — top contributors

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
    match db.get_project_contributors(&project_id) {
        Ok(contributors) => {
            let entries: Vec<LeaderboardEntry> = contributors
                .into_iter()
                .map(|(worker, total)| LeaderboardEntry {
                    worker_node_id: worker,
                    total_kudos: total,
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
