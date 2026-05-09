// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-app key-value storage endpoints (Sprint 56 Phase C).
//!
//! In-memory storage keyed by `(app_name, key)`. Provides the HTTP
//! backend for the bridge `storage_get`, `storage_set`,
//! `storage_list`, and `storage_delete` methods.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::debug;

use crate::http::DaemonHttpState;

pub type AppStorage = Arc<RwLock<HashMap<String, HashMap<String, serde_json::Value>>>>;

#[cfg(test)]
pub fn new_app_storage() -> AppStorage {
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn load_app_storage_from_db(
    db: &std::sync::MutexGuard<'_, nexus_coordinator_rs::db::CoordinatorDb>,
) -> AppStorage {
    match db.load_all_storage() {
        Ok(map) => {
            let count: usize = map.values().map(|m| m.len()).sum();
            if count > 0 {
                tracing::info!(apps = map.len(), keys = count, "loaded app storage from DB");
            }
            Arc::new(RwLock::new(map))
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to load app storage from DB, starting empty");
            Arc::new(RwLock::new(HashMap::new()))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub prefix: Option<String>,
}

pub async fn storage_list(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app_name): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    debug!(app = %app_name, prefix = ?query.prefix, "GET /app/:name/state");
    let store = state.app_storage.read().await;
    let entries: Vec<serde_json::Value> = store
        .get(&app_name)
        .map(|app_map| {
            app_map
                .iter()
                .filter(|(k, _)| match &query.prefix {
                    Some(p) => k.starts_with(p.as_str()),
                    None => true,
                })
                .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
                .collect()
        })
        .unwrap_or_default();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "entries": entries, "count": entries.len() })),
    )
}

pub async fn storage_get(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app_name, key)): Path<(String, String)>,
) -> impl IntoResponse {
    debug!(app = %app_name, key = %key, "GET /app/:name/state/:key");
    let store = state.app_storage.read().await;
    match store.get(&app_name).and_then(|m| m.get(&key)) {
        Some(value) => (
            StatusCode::OK,
            Json(serde_json::json!({ "key": key, "value": value })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("key '{}' not found", key) })),
        )
            .into_response(),
    }
}

pub async fn storage_set(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app_name, key)): Path<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    debug!(app = %app_name, key = %key, "POST /app/:name/state/:key");
    {
        let db = match state.coordinator_db.lock() {
            Ok(db) => db,
            Err(e) => {
                tracing::error!(error = %e, "coordinator DB mutex poisoned");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "storage unavailable" })),
                );
            }
        };
        if let Err(e) = db.upsert_storage(&app_name, &key, &body) {
            tracing::error!(error = %e, "storage persistence failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "storage persistence failed" })),
            );
        }
    }
    let mut store = state.app_storage.write().await;
    store.entry(app_name).or_default().insert(key, body);
    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}

pub async fn storage_delete(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app_name, key)): Path<(String, String)>,
) -> impl IntoResponse {
    debug!(app = %app_name, key = %key, "DELETE /app/:name/state/:key");
    {
        let store = state.app_storage.read().await;
        let exists = store.get(&app_name).and_then(|m| m.get(&key)).is_some();
        if !exists {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("key '{}' not found", key) })),
            )
                .into_response();
        }
    }
    {
        let db = match state.coordinator_db.lock() {
            Ok(db) => db,
            Err(e) => {
                tracing::error!(error = %e, "coordinator DB mutex poisoned");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "storage unavailable" })),
                )
                    .into_response();
            }
        };
        if let Err(e) = db.delete_storage(&app_name, &key) {
            tracing::error!(error = %e, "storage delete persistence failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "storage persistence failed" })),
            )
                .into_response();
        }
    }
    let mut store = state.app_storage.write().await;
    store.get_mut(&app_name).and_then(|m| m.remove(&key));
    (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_list_by_prefix() {
        let storage = new_app_storage();
        {
            let mut s = storage.write().await;
            let app = s.entry("myapp".to_string()).or_default();
            app.insert("user:alice".to_string(), serde_json::json!("data-a"));
            app.insert("user:bob".to_string(), serde_json::json!("data-b"));
            app.insert("config:theme".to_string(), serde_json::json!("dark"));
        }

        let store = storage.read().await;
        let app_map = store.get("myapp").unwrap();

        // Filter by prefix "user:"
        let filtered: Vec<_> = app_map
            .iter()
            .filter(|(k, _)| k.starts_with("user:"))
            .collect();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|(k, _)| k.starts_with("user:")));

        // No prefix — all entries
        let all: Vec<_> = app_map.iter().collect();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_storage_delete_key() {
        let storage = new_app_storage();
        {
            let mut s = storage.write().await;
            s.entry("myapp".to_string())
                .or_default()
                .insert("temp".to_string(), serde_json::json!(42));
        }

        // Delete existing key
        {
            let mut s = storage.write().await;
            let removed = s.get_mut("myapp").and_then(|m| m.remove("temp")).is_some();
            assert!(removed);
        }

        // Verify gone
        {
            let s = storage.read().await;
            assert!(s.get("myapp").and_then(|m| m.get("temp")).is_none());
        }

        // Delete non-existent key
        {
            let mut s = storage.write().await;
            let removed = s.get_mut("myapp").and_then(|m| m.remove("temp")).is_some();
            assert!(!removed);
        }
    }
}
