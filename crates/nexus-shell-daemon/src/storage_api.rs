// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-app key-value storage endpoints (Sprint 56 Phase C, Sprint 58
//! Phase C iroh-docs routing).
//!
//! Apps detected as replicated (hardcoded `sbfb-ideas` in S58 MVP)
//! route through iroh-docs for P2P replication. All other apps use
//! the original in-memory HashMap + SQLite write-through backend.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use nexus_core_rs::BlobsClient;
use nexus_core_rs::docs::{DocHandle, DocsAuthorId, DocsEntry, DocsTicket};

use crate::http::DaemonHttpState;

pub type AppStorage = Arc<RwLock<HashMap<String, HashMap<String, serde_json::Value>>>>;

const REPLICATED_APPS: &[&str] = &["sbfb-ideas"];

fn is_replicated(app_name: &str) -> bool {
    REPLICATED_APPS.contains(&app_name)
}

/// State for a single replicated app's iroh-docs namespace.
pub struct StorageNamespaceState {
    pub doc: Arc<DocHandle>,
    pub author: DocsAuthorId,
    pub ticket: String,
    pub version: AtomicU64,
}

impl std::fmt::Debug for StorageNamespaceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageNamespaceState")
            .field("doc_id", &self.doc.id())
            .field("author", &self.author)
            .field("version", &self.version.load(Ordering::Relaxed))
            .finish()
    }
}

pub type StorageNamespaces = Arc<RwLock<HashMap<String, Arc<StorageNamespaceState>>>>;

pub fn new_storage_namespaces() -> StorageNamespaces {
    Arc::new(RwLock::new(HashMap::new()))
}

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

async fn read_entry_content(
    blobs: &BlobsClient<'_>,
    entry: &DocsEntry,
) -> Option<serde_json::Value> {
    let hash_bytes = *entry.content_hash().as_bytes();
    match blobs.get_bytes(hash_bytes).await {
        Ok(bytes) => match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(val) => Some(val),
            Err(e) => {
                warn!(key = ?String::from_utf8_lossy(entry.key()), error = %e, "invalid JSON in iroh-docs entry");
                None
            }
        },
        Err(e) => {
            warn!(key = ?String::from_utf8_lossy(entry.key()), error = %e, "failed to read blob content");
            None
        }
    }
}

fn is_tombstone(value: &serde_json::Value) -> bool {
    value.get("deleted").and_then(|v| v.as_bool()) == Some(true)
        || value.get("retracted").and_then(|v| v.as_bool()) == Some(true)
}

// ---------------------------------------------------------------------------
// storage_list
// ---------------------------------------------------------------------------

pub async fn storage_list(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app_name): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    debug!(app = %app_name, prefix = ?query.prefix, "GET /app/:name/state");

    if is_replicated(&app_name) {
        return storage_list_replicated(&state, &app_name, query.prefix.as_deref()).await;
    }

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
        .into_response()
}

async fn storage_list_replicated(
    state: &DaemonHttpState,
    app_name: &str,
    prefix: Option<&str>,
) -> axum::response::Response {
    let ns = state.storage_namespaces.read().await;
    let Some(ns_state) = ns.get(app_name) else {
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "entries": [], "count": 0 })),
        )
            .into_response();
    };
    let ns_state = Arc::clone(ns_state);
    drop(ns);

    let blobs = BlobsClient::new(state.node.blobs_store());
    let prefix_bytes = prefix.unwrap_or("").as_bytes();

    // votes/ keys: return all entries (each author = 1 distinct vote).
    // Other keys: deduplicate via single_latest_per_key.
    let use_all_authors = prefix.is_some_and(|p| p.starts_with("votes/"));

    let doc_entries = if use_all_authors {
        ns_state.doc.get_many_by_prefix(prefix_bytes).await
    } else {
        ns_state
            .doc
            .get_many_latest_per_key_prefix(prefix_bytes)
            .await
    };

    let doc_entries = match doc_entries {
        Ok(entries) => entries,
        Err(e) => {
            warn!(app = %app_name, error = %e, "iroh-docs list failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "storage read failed" })),
            )
                .into_response();
        }
    };

    let mut entries = Vec::with_capacity(doc_entries.len());
    for entry in &doc_entries {
        if let Some(value) = read_entry_content(&blobs, entry).await {
            if !is_tombstone(&value) {
                let key = String::from_utf8_lossy(entry.key()).to_string();
                entries.push(serde_json::json!({ "key": key, "value": value }));
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "entries": entries, "count": entries.len() })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// storage_get
// ---------------------------------------------------------------------------

pub async fn storage_get(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app_name, key)): Path<(String, String)>,
) -> impl IntoResponse {
    debug!(app = %app_name, key = %key, "GET /app/:name/state/:key");

    if is_replicated(&app_name) {
        return storage_get_replicated(&state, &app_name, &key).await;
    }

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

async fn storage_get_replicated(
    state: &DaemonHttpState,
    app_name: &str,
    key: &str,
) -> axum::response::Response {
    let ns = state.storage_namespaces.read().await;
    let Some(ns_state) = ns.get(app_name) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("key '{}' not found", key) })),
        )
            .into_response();
    };
    let ns_state = Arc::clone(ns_state);
    drop(ns);

    let entry = match ns_state.doc.get_latest_by_key(key.as_bytes()).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("key '{}' not found", key) })),
            )
                .into_response();
        }
        Err(e) => {
            warn!(app = %app_name, key = %key, error = %e, "iroh-docs get failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "storage read failed" })),
            )
                .into_response();
        }
    };

    let blobs = BlobsClient::new(state.node.blobs_store());
    match read_entry_content(&blobs, &entry).await {
        Some(value) if !is_tombstone(&value) => (
            StatusCode::OK,
            Json(serde_json::json!({ "key": key, "value": value })),
        )
            .into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("key '{}' not found", key) })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// storage_set
// ---------------------------------------------------------------------------

pub async fn storage_set(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app_name, key)): Path<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    debug!(app = %app_name, key = %key, "POST /app/:name/state/:key");

    if is_replicated(&app_name) {
        return storage_set_replicated(&state, &app_name, &key, &body).await;
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
        if let Err(e) = db.upsert_storage(&app_name, &key, &body) {
            tracing::error!(error = %e, "storage persistence failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "storage persistence failed" })),
            )
                .into_response();
        }
    }
    let mut store = state.app_storage.write().await;
    store.entry(app_name).or_default().insert(key, body);
    (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
}

async fn storage_set_replicated(
    state: &DaemonHttpState,
    app_name: &str,
    key: &str,
    value: &serde_json::Value,
) -> axum::response::Response {
    let ns = state.storage_namespaces.read().await;
    let Some(ns_state) = ns.get(app_name) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "storage namespace not initialized" })),
        )
            .into_response();
    };
    let ns_state = Arc::clone(ns_state);
    drop(ns);

    let json_bytes = serde_json::to_vec(value).unwrap_or_default();
    match ns_state
        .doc
        .set(ns_state.author, key.as_bytes().to_vec(), json_bytes)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(e) => {
            warn!(app = %app_name, key = %key, error = %e, "iroh-docs set failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "storage write failed" })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// storage_delete
// ---------------------------------------------------------------------------

pub async fn storage_delete(
    State(state): State<Arc<DaemonHttpState>>,
    Path((app_name, key)): Path<(String, String)>,
) -> impl IntoResponse {
    debug!(app = %app_name, key = %key, "DELETE /app/:name/state/:key");

    if is_replicated(&app_name) {
        return storage_delete_replicated(&state, &app_name, &key).await;
    }

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

async fn storage_delete_replicated(
    state: &DaemonHttpState,
    app_name: &str,
    key: &str,
) -> axum::response::Response {
    let ns = state.storage_namespaces.read().await;
    let Some(ns_state) = ns.get(app_name) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("key '{}' not found", key) })),
        )
            .into_response();
    };
    let ns_state = Arc::clone(ns_state);
    drop(ns);

    let tombstone = serde_json::to_vec(&serde_json::json!({ "deleted": true })).unwrap();
    match ns_state
        .doc
        .set(ns_state.author, key.as_bytes().to_vec(), tombstone)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(e) => {
            warn!(app = %app_name, key = %key, error = %e, "iroh-docs tombstone write failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "storage delete failed" })),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Ticket + Join endpoints (Sprint 58 Phase C)
// ---------------------------------------------------------------------------

pub async fn storage_ticket(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app_name): Path<String>,
) -> impl IntoResponse {
    let ns = state.storage_namespaces.read().await;
    match ns.get(&app_name) {
        Some(ns_state) => (
            StatusCode::OK,
            Json(serde_json::json!({ "app": app_name, "ticket": ns_state.ticket })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::json!({ "error": format!("no storage namespace for '{}'", app_name) }),
            ),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct JoinRequest {
    pub app: String,
    pub ticket: String,
}

pub async fn storage_join(
    State(state): State<Arc<DaemonHttpState>>,
    Json(body): Json<JoinRequest>,
) -> impl IntoResponse {
    let ticket: DocsTicket = match body.ticket.parse() {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("invalid ticket: {}", e) })),
            )
                .into_response();
        }
    };

    let docs_client = nexus_core_rs::docs::DocsClient::new(state.node.docs());
    let doc_handle = match docs_client.import_ticket(ticket).await {
        Ok(d) => d,
        Err(e) => {
            warn!(app = %body.app, error = %e, "import ticket failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "import failed" })),
            )
                .into_response();
        }
    };

    let author = match docs_client.author_default().await {
        Ok(a) => a,
        Err(e) => {
            warn!(error = %e, "author_default failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "author_default failed" })),
            )
                .into_response();
        }
    };

    let ns_id_bytes = doc_handle.id().as_bytes().to_vec();
    let ns_state = Arc::new(StorageNamespaceState {
        doc: Arc::new(doc_handle),
        author,
        ticket: body.ticket.clone(),
        version: AtomicU64::new(0),
    });

    {
        let db = match state.coordinator_db.lock() {
            Ok(db) => db,
            Err(e) => {
                warn!(error = %e, "coordinator DB mutex poisoned during join");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "DB unavailable" })),
                )
                    .into_response();
            }
        };
        if let Err(e) = db.set_storage_namespace(&body.app, &ns_id_bytes, Some(&body.ticket)) {
            warn!(error = %e, "failed to persist joined namespace");
        }
    }

    let mut ns = state.storage_namespaces.write().await;
    ns.insert(body.app.clone(), ns_state);

    (
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true, "app": body.app })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Version endpoint (Phase D will wire InsertRemote → increment)
// ---------------------------------------------------------------------------

pub async fn storage_version(
    State(state): State<Arc<DaemonHttpState>>,
    Path(app_name): Path<String>,
) -> impl IntoResponse {
    let ns = state.storage_namespaces.read().await;
    match ns.get(&app_name) {
        Some(ns_state) => {
            let v = ns_state.version.load(Ordering::Relaxed);
            (
                StatusCode::OK,
                Json(serde_json::json!({ "app": app_name, "version": v })),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::json!({ "error": format!("no storage namespace for '{}'", app_name) }),
            ),
        )
            .into_response(),
    }
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

        let filtered: Vec<_> = app_map
            .iter()
            .filter(|(k, _)| k.starts_with("user:"))
            .collect();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|(k, _)| k.starts_with("user:")));

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

        {
            let mut s = storage.write().await;
            let removed = s.get_mut("myapp").and_then(|m| m.remove("temp")).is_some();
            assert!(removed);
        }

        {
            let s = storage.read().await;
            assert!(s.get("myapp").and_then(|m| m.get("temp")).is_none());
        }

        {
            let mut s = storage.write().await;
            let removed = s.get_mut("myapp").and_then(|m| m.remove("temp")).is_some();
            assert!(!removed);
        }
    }

    #[test]
    fn test_is_replicated() {
        assert!(is_replicated("sbfb-ideas"));
        assert!(!is_replicated("sbfb-explorer"));
        assert!(!is_replicated("unknown-app"));
    }

    #[test]
    fn test_is_tombstone() {
        assert!(is_tombstone(&serde_json::json!({ "deleted": true })));
        assert!(is_tombstone(&serde_json::json!({ "retracted": true })));
        assert!(!is_tombstone(&serde_json::json!({ "deleted": false })));
        assert!(!is_tombstone(&serde_json::json!({ "title": "hello" })));
        assert!(!is_tombstone(&serde_json::json!(42)));
    }
}
