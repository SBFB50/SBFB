// SPDX-License-Identifier: AGPL-3.0-or-later
//! Task list + detail endpoints (Sprint 44 Phase C, port of tasks.py).
//!
//! Completes the task API surface started in S35 (submit).
//! - `GET /api/v1/tasks` — list tasks, optional status filter + limit
//! - `GET /api/v1/tasks/{task_id}` — single task detail
//! - `GET /api/v1/tasks/{task_id}/result` — completed result text
//!   (Sprint 72 Phase D: the primitive the Operator network execution
//!   arm reads to render a finished network task's reply in the chat).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::http::DaemonHttpState;

#[derive(Debug, Deserialize)]
pub struct TaskListQuery {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    100
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub status: String,
    pub project_id: String,
    pub model: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub task_hash: String,
    pub worker_node_id: Option<String>,
    pub result_hash: Option<String>,
}

pub async fn list_tasks(
    State(state): State<Arc<DaemonHttpState>>,
    Query(query): Query<TaskListQuery>,
) -> impl IntoResponse {
    debug!(?query.state, query.limit, "GET /api/v1/tasks");
    const VALID_STATES: &[&str] = &[
        "pending",
        "dispatched",
        "completed",
        "rejected",
        "timed_out",
    ];
    if let Some(ref s) = query.state
        && !VALID_STATES.contains(&s.as_str())
    {
        return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("invalid state: {s}. valid: {VALID_STATES:?}")})),
            )
                .into_response();
    }
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
    let limit = query.limit.min(500);
    match db.list_tasks(query.state.as_deref(), limit) {
        Ok(tasks) => {
            let count = tasks.len();
            let tasks: Vec<TaskResponse> = tasks
                .into_iter()
                .map(|t| TaskResponse {
                    task_id: t.task_id,
                    status: t.status.as_str().to_owned(),
                    project_id: t.project_id,
                    model: t.model,
                    created_at: t.created_at,
                    updated_at: t.updated_at,
                    task_hash: t.task_hash,
                    worker_node_id: t.worker_node_id,
                    result_hash: t.result_hash,
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({"tasks": tasks, "count": count})),
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

pub async fn get_task(
    State(state): State<Arc<DaemonHttpState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    debug!(id = %task_id, "GET /api/v1/tasks/:id");
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
    match db.get_task(&task_id) {
        Ok(Some(t)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "task_id": t.task_id,
                "status": t.status.as_str(),
                "project_id": t.project_id,
                "model": t.model,
                "created_at": t.created_at,
                "updated_at": t.updated_at,
                "task_hash": t.task_hash,
                "worker_node_id": t.worker_node_id,
                "result_hash": t.result_hash,
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("task {task_id} not found")})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

/// `GET /api/v1/tasks/{task_id}/result` — return the completed task's
/// human-readable output (Sprint 72 Phase D). A task that exists but has
/// no result text yet (pending/dispatched/rejected) is a 404 so the
/// Operator's network arm polls `/{task_id}` for status and only fetches
/// the text once `completed`. Same T0 loopback `auth_required` tier as
/// the rest of the task API — a read-only endpoint, no new trust surface.
pub async fn get_task_result(
    State(state): State<Arc<DaemonHttpState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    debug!(id = %task_id, "GET /api/v1/tasks/:id/result");
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
    match db.get_task_result(&task_id) {
        Ok(Some(detail)) => match detail.result_text {
            Some(text) => (
                StatusCode::OK,
                Json(serde_json::json!({
                    "task_id": task_id,
                    "status": detail.status,
                    "result_text": text,
                    "result_hash": detail.result_hash,
                })),
            )
                .into_response(),
            None => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("task {task_id} has no result yet (status: {})", detail.status),
                })),
            )
                .into_response(),
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("task {task_id} not found")})),
        )
            .into_response(),
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
    fn task_list_query_defaults() {
        let q: TaskListQuery = serde_json::from_str("{}").unwrap();
        assert!(q.state.is_none());
        assert_eq!(q.limit, 100);
    }

    #[test]
    fn task_list_query_with_state() {
        let q: TaskListQuery = serde_json::from_str(r#"{"state":"pending","limit":50}"#).unwrap();
        assert_eq!(q.state.as_deref(), Some("pending"));
        assert_eq!(q.limit, 50);
    }

    #[test]
    fn task_response_serializes() {
        let t = TaskResponse {
            task_id: "t1".into(),
            status: "pending".into(),
            project_id: "p1".into(),
            model: "llama3".into(),
            created_at: 1700000000,
            updated_at: 1700000001,
            task_hash: "abc".into(),
            worker_node_id: None,
            result_hash: None,
        };
        let json = serde_json::to_value(&t).unwrap();
        assert_eq!(json["status"], "pending");
        assert_eq!(json["worker_node_id"], serde_json::Value::Null);
    }

    // --- tasks_api.rs (2 routes) ---

    #[tokio::test]
    async fn tasks_list_default() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/tasks")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["tasks"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn tasks_get_not_found_404() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/tasks/nonexistent-task-id")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // Sprint 72 Phase D: `/{task_id}/result` is 404 while the task is
    // pending and returns the human-readable text once completed — the
    // primitive the Operator network arm polls then fetches.
    #[tokio::test]
    async fn task_result_route_404_then_text_on_completed() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();
        let db_handle = state.coordinator_db.clone();

        let task_id = {
            let db = db_handle.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit")
                .task
                .task_id
        };

        let app = build_test_router(state);

        // Pending → 404 (status carried in the error message).
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/tasks/{task_id}/result"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // Complete it with a retrievable text.
        {
            let db = db_handle.lock().unwrap();
            db.set_task_result(&task_id, "w1", "sig-hex", "the network reply", 100)
                .expect("complete");
        }

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/tasks/{task_id}/result"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["status"], "completed");
        assert_eq!(body["result_text"], "the network reply");
    }

    #[tokio::test]
    async fn tasks_list_with_limit() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .unwrap();
        }
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/tasks?limit=1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 1);
    }
}
