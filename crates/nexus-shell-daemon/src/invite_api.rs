// SPDX-License-Identifier: AGPL-3.0-or-later
//! Invite endpoints (Sprint 45 Phase A, port of invites.py).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tracing::debug;

use crate::http::DaemonHttpState;

static INVITE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize)]
pub struct CreateInviteBody {
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default = "default_expiry_secs")]
    pub expiry_secs: i64,
    #[serde(default)]
    pub max_uses: Option<i64>,
    #[serde(default)]
    pub note: Option<String>,
}

fn default_scope() -> String {
    "worker".into()
}

fn default_expiry_secs() -> i64 {
    7 * 24 * 3600
}

const VALID_SCOPES: &[&str] = &["worker", "observer"];
const MIN_EXPIRY_SECS: i64 = 60;
const DEFAULT_PROJECT_NAME: &str = "sbfb";

pub async fn create_invite(
    State(state): State<Arc<DaemonHttpState>>,
    Json(body): Json<CreateInviteBody>,
) -> impl IntoResponse {
    debug!(scope = %body.scope, expiry = body.expiry_secs, "POST /api/v1/invite/create");

    if !VALID_SCOPES.contains(&body.scope.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "scope must be 'worker' or 'observer'"})),
        )
            .into_response();
    }
    if body.expiry_secs < MIN_EXPIRY_SECS {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "expiry_secs must be >= 60"})),
        )
            .into_response();
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let expires_at = now + body.expiry_secs;
    let seq = INVITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let node_prefix = &state.node_id[..8.min(state.node_id.len())];
    let id = format!("inv-{node_prefix}-{now}-{seq}");

    let scope = match body.scope.as_str() {
        "observer" => nexus_worker_core::invite::InviteScope::Observer,
        _ => nexus_worker_core::invite::InviteScope::Worker,
    };

    let tasks_doc_ticket = if scope.can_serve_tasks() {
        match state.project_doc.as_ref() {
            Some(doc) => match doc.share_write().await {
                Ok(ticket) => Some(ticket.to_string()),
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": format!("failed to generate tasks doc ticket: {e}")})),
                    )
                        .into_response();
                }
            },
            None => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({"error": "project doc not initialized — cannot mint worker invite"})),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let project_id = state
        .project_doc
        .as_ref()
        .map(|d| d.id().to_string())
        .unwrap_or_default();

    let invite = match nexus_worker_core::invite::Invite::mint(
        &state.pow_keypair,
        &project_id,
        DEFAULT_PROJECT_NAME,
        None,
        tasks_doc_ticket.clone(),
        scope,
        expires_at as u64,
    ) {
        Ok(inv) => inv,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("invite mint failed: {e}")})),
            )
                .into_response();
        }
    };
    let wire = invite.encode();

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
    let ledger = nexus_coordinator_rs::invite::InviteLedger::new(&db);
    let mut req = nexus_coordinator_rs::invite::MintRequest::new(
        &id,
        &wire,
        &body.scope,
        &project_id,
        DEFAULT_PROJECT_NAME,
        expires_at,
    );
    req.max_uses = body.max_uses;
    req.note = body.note.as_deref();
    req.tasks_doc_ticket = tasks_doc_ticket.as_deref();

    match ledger.mint(&req) {
        Ok(rec) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": rec.id,
                "wire": rec.wire,
                "scope": rec.scope,
                "project_id": rec.project_id,
                "expires_at": rec.expires_at,
                "max_uses": rec.max_uses,
                "note": rec.note,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("{e}")})),
        )
            .into_response(),
    }
}

pub async fn list_invites(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /api/v1/invite");
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

    let ledger = nexus_coordinator_rs::invite::InviteLedger::new(&db);
    match ledger.list(100) {
        Ok(records) => {
            let count = records.len();
            let invites: Vec<serde_json::Value> = records
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "scope": r.scope,
                        "project_id": r.project_id,
                        "expires_at": r.expires_at,
                        "max_uses": r.max_uses,
                        "uses_count": r.uses_count,
                        "revoked_at": r.revoked_at,
                        "note": r.note,
                        "created_at": r.created_at,
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({"invites": invites, "count": count})),
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

pub async fn revoke_invite(
    State(state): State<Arc<DaemonHttpState>>,
    Path(invite_id): Path<String>,
) -> impl IntoResponse {
    debug!(id = %invite_id, "DELETE /api/v1/invite/:id");
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

    let ledger = nexus_coordinator_rs::invite::InviteLedger::new(&db);
    match ledger.revoke(&invite_id) {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"id": invite_id, "revoked": true})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("invite {invite_id} not found or already revoked")})),
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

    #[test]
    fn create_invite_body_deserializes_defaults() {
        let body: CreateInviteBody = serde_json::from_str("{}").unwrap();
        assert_eq!(body.scope, "worker");
        assert_eq!(body.expiry_secs, 604800);
        assert!(body.max_uses.is_none());
        assert!(body.note.is_none());
    }

    #[test]
    fn create_invite_body_with_values() {
        let body: CreateInviteBody = serde_json::from_str(
            r#"{"scope":"observer","expiry_secs":3600,"max_uses":5,"note":"test"}"#,
        )
        .unwrap();
        assert_eq!(body.scope, "observer");
        assert_eq!(body.expiry_secs, 3600);
        assert_eq!(body.max_uses, Some(5));
        assert_eq!(body.note.as_deref(), Some("test"));
    }

    #[test]
    fn valid_scopes() {
        assert!(VALID_SCOPES.contains(&"worker"));
        assert!(VALID_SCOPES.contains(&"observer"));
        assert!(!VALID_SCOPES.contains(&"admin"));
    }
}
