// SPDX-License-Identifier: AGPL-3.0-or-later
//! Contributor attestation endpoints — port of api/contributor.py
//! (Sprint 43 Phase C). Replaces the proxy_contributor_verify proxy
//! with a direct Rust handler and adds list + envelope routes.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::http::DaemonHttpState;

fn validate_hex(value: &str, expected_len: usize, label: &str) -> Result<(), (StatusCode, String)> {
    if value.len() != expected_len {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "{label}: expected {expected_len} chars, got {}",
                value.len()
            ),
        ));
    }
    if !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err((StatusCode::BAD_REQUEST, format!("{label}: must be hex")));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub project_id: String,
    pub contributor_node_id: String,
    pub verified: bool,
}

pub async fn verify_contributor(
    State(state): State<Arc<DaemonHttpState>>,
    Path((project_id, node_id_hex)): Path<(String, String)>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    validate_hex(&project_id, 64, "project_id")?;
    validate_hex(&node_id_hex, 64, "node_id_hex")?;
    let project_id = project_id.to_ascii_lowercase();
    let node_id_hex = node_id_hex.to_ascii_lowercase();

    let db = state
        .coordinator_db
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let registry = nexus_coordinator_rs::contributor_registry::ContributorRegistry::new(&db);
    let verified = registry
        .is_verified_contributor(&project_id, &node_id_hex)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(VerifyResponse {
        project_id,
        contributor_node_id: node_id_hex,
        verified,
    }))
}

#[derive(Debug, Serialize)]
pub struct ContributorEntry {
    contributor_node_id: String,
    first_deploy_ts: i64,
    commit_sha: String,
    repo_url: String,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub project_id: String,
    pub count: usize,
    pub contributors: Vec<ContributorEntry>,
}

pub async fn list_contributors(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
) -> Result<Json<ListResponse>, (StatusCode, String)> {
    validate_hex(&project_id, 64, "project_id")?;
    let project_id = project_id.to_ascii_lowercase();

    let db = state
        .coordinator_db
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let registry = nexus_coordinator_rs::contributor_registry::ContributorRegistry::new(&db);
    let rows = registry
        .list_for_project(&project_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let contributors: Vec<ContributorEntry> = rows
        .into_iter()
        .map(|r| ContributorEntry {
            contributor_node_id: r.contributor_node_id,
            first_deploy_ts: r.first_deploy_ts,
            commit_sha: r.commit_sha,
            repo_url: r.repo_url,
        })
        .collect();
    let count = contributors.len();

    Ok(Json(ListResponse {
        project_id,
        count,
        contributors,
    }))
}

pub async fn envelope(
    State(state): State<Arc<DaemonHttpState>>,
    Path((project_id, node_id_hex)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    validate_hex(&project_id, 64, "project_id")?;
    validate_hex(&node_id_hex, 64, "node_id_hex")?;
    let project_id = project_id.to_ascii_lowercase();
    let node_id_hex = node_id_hex.to_ascii_lowercase();

    let db = state
        .coordinator_db
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let registry = nexus_coordinator_rs::contributor_registry::ContributorRegistry::new(&db);
    let record = registry
        .get(&project_id, &node_id_hex)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "attestation not found".into()))?;

    let envelope_obj: serde_json::Value =
        serde_json::from_str(&record.attestation_json).map_err(|e| {
            tracing::error!(
                project_id = %project_id,
                contributor_node_id = %node_id_hex,
                error = %e,
                "contributor envelope parse failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "stored envelope is corrupt".into(),
            )
        })?;

    Ok(Json(envelope_obj))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_hex_valid() {
        assert!(validate_hex(&"a".repeat(64), 64, "test").is_ok());
    }

    #[test]
    fn validate_hex_wrong_len() {
        assert!(validate_hex("abc", 64, "test").is_err());
    }

    #[test]
    fn validate_hex_non_hex() {
        assert!(validate_hex(&"g".repeat(64), 64, "test").is_err());
    }

    #[test]
    fn verify_response_serializes() {
        let resp = VerifyResponse {
            project_id: "a".repeat(64),
            contributor_node_id: "b".repeat(64),
            verified: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"verified\":true"));
    }
}
