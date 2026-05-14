// SPDX-License-Identifier: AGPL-3.0-or-later
//! Feed sync via iroh-docs (Sprint 62 Phase B).
//!
//! Replicates the local public feed to/from remote nodes using an
//! iroh-docs namespace. Pattern mirrors `storage_api.rs` (Sprint 58).

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use futures_lite::StreamExt;
use serde::Deserialize;
use tracing::{debug, info, warn};

use nexus_coordinator_rs::db::CoordinatorDb;
use nexus_coordinator_rs::public_feed::{self, FeedEntry, validate_feed_operation};
use nexus_core_rs::BlobsClient;
use nexus_core_rs::docs::{DocHandle, DocsAuthorId, DocsEntry, DocsLiveEvent, DocsTicket};

use crate::http::DaemonHttpState;

pub const FEED_NAMESPACE_KEY: &str = "sbfb-feed";

pub struct FeedSyncState {
    pub doc: Arc<DocHandle>,
    pub author: DocsAuthorId,
    pub ticket: String,
}

impl std::fmt::Debug for FeedSyncState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeedSyncState")
            .field("doc_id", &self.doc.id())
            .field("author", &self.author)
            .finish()
    }
}

fn format_feed_key(author_hex: &str, seq: u64) -> String {
    format!("feed/{author_hex}/{seq:010}")
}

// ---------------------------------------------------------------------------
// Publish local entry → iroh-docs
// ---------------------------------------------------------------------------

pub async fn publish_feed_entry_to_docs(
    feed_state: &FeedSyncState,
    entry: &FeedEntry,
) -> Result<(), String> {
    let key = format_feed_key(&entry.author_pubkey, entry.seq);
    let value = serde_json::to_vec(entry).map_err(|e| format!("feed entry serialization: {e}"))?;
    feed_state
        .doc
        .set(feed_state.author, key.into_bytes(), value)
        .await
        .map_err(|e| format!("iroh-docs set failed: {e}"))?;
    debug!(
        seq = entry.seq,
        key_author = &entry.author_pubkey[..8],
        "feed entry published to iroh-docs"
    );
    Ok(())
}

/// Insert a feed operation into SQLite AND publish to iroh-docs.
///
/// Combines `insert_feed_operation()` (coordinator DB) with
/// `publish_feed_entry_to_docs()` (iroh-docs namespace) so the
/// daemon never inserts without publishing. Acceptance criterion
/// Phase B §5.4: "Un daemon qui insere publie dans iroh-docs".
///
/// Not used by the HTTP endpoint (which splits DB lock and async
/// publish to avoid holding a mutex across an await point) but
/// available for internal coordinator flows.
#[allow(dead_code)]
pub async fn insert_and_publish_feed_operation(
    feed_state: &FeedSyncState,
    db: &CoordinatorDb,
    op: nexus_coordinator_rs::public_feed::PublicFeedOperation,
    author_pubkey: &str,
    sign_fn: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Result<FeedEntry, String> {
    let entry =
        nexus_coordinator_rs::public_feed::insert_feed_operation(db, op, author_pubkey, sign_fn)?;
    publish_feed_entry_to_docs(feed_state, &entry).await?;
    Ok(entry)
}

// ---------------------------------------------------------------------------
// Ingest a single iroh-docs entry into the local feed DB
// ---------------------------------------------------------------------------

async fn ingest_doc_entry(
    doc_entry: &DocsEntry,
    node: &nexus_core_rs::Node,
    coordinator_db: &std::sync::Mutex<CoordinatorDb>,
) {
    let key_bytes = doc_entry.key();
    let key_str = String::from_utf8_lossy(key_bytes);
    if !key_str.starts_with("feed/") {
        return;
    }

    let blobs = BlobsClient::new(node.blobs_store());
    let hash_bytes = *doc_entry.content_hash().as_bytes();
    let content = match blobs.get_bytes(hash_bytes).await {
        Ok(b) => b,
        Err(e) => {
            warn!(key = %key_str, error = %e, "failed to read feed entry blob");
            return;
        }
    };

    let feed_entry: FeedEntry = match serde_json::from_slice(&content) {
        Ok(e) => e,
        Err(e) => {
            warn!(key = %key_str, error = %e, "invalid feed entry JSON");
            return;
        }
    };

    if let Err(e) = public_feed::verify_entry(&feed_entry) {
        warn!(key = %key_str, error = %e, "feed entry verification failed");
        return;
    }

    if let Err(e) = validate_feed_operation(&feed_entry.op) {
        warn!(key = %key_str, error = %e, "feed operation validation failed");
        return;
    }

    let db = match coordinator_db.lock() {
        Ok(db) => db,
        Err(e) => {
            warn!(error = %e, "coordinator DB lock failed in feed ingest");
            return;
        }
    };

    match db.feed_entry_exists_by_hash(&feed_entry.entry_hash) {
        Ok(true) => {
            debug!(
                hash = &feed_entry.entry_hash[..8],
                "feed entry already exists, skipping"
            );
            return;
        }
        Ok(false) => {}
        Err(e) => {
            warn!(error = %e, "feed dedup check failed");
            return;
        }
    }

    let op_type = match &feed_entry.op {
        nexus_coordinator_rs::public_feed::PublicFeedOperation::ReleasePublished(_) => {
            "ReleasePublished"
        }
        nexus_coordinator_rs::public_feed::PublicFeedOperation::SourceBecameStale(_) => {
            "SourceBecameStale"
        }
    };
    let payload = match serde_json::to_string(&feed_entry.op) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "feed op re-serialization failed");
            return;
        }
    };

    let row = nexus_coordinator_rs::db::FeedEntryRow {
        seq: 0,
        op_type: op_type.to_string(),
        payload,
        author: feed_entry.author_pubkey.clone(),
        signature: feed_entry.signature.clone(),
        entry_hash: feed_entry.entry_hash.clone(),
        prev_hash: feed_entry.prev_hash.clone(),
        created_at: feed_entry.timestamp,
    };

    match db.insert_feed_entry(&row) {
        Ok(seq) => {
            info!(
                seq,
                author = &feed_entry.author_pubkey[..8],
                hash = &feed_entry.entry_hash[..8],
                "remote feed entry inserted"
            );
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE constraint") {
                debug!(
                    hash = &feed_entry.entry_hash[..8],
                    "feed entry duplicate (race)"
                );
            } else {
                warn!(error = %e, "feed entry insert failed");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Subscribe — ingest remote entries (boot path, own namespace)
// ---------------------------------------------------------------------------

pub fn spawn_feed_subscribe(
    feed_state: Arc<FeedSyncState>,
    coordinator_db: Arc<std::sync::Mutex<CoordinatorDb>>,
    node: Arc<nexus_core_rs::Node>,
) {
    tokio::spawn(async move {
        let mut stream = match feed_state.doc.subscribe().await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "feed subscribe failed");
                return;
            }
        };
        info!("feed subscribe active");
        while let Some(event) = stream.next().await {
            match event {
                Ok(DocsLiveEvent::InsertRemote {
                    entry: doc_entry, ..
                }) => {
                    ingest_doc_entry(&doc_entry, &node, &coordinator_db).await;
                }
                Ok(_) => {}
                Err(e) => {
                    warn!(error = %e, "feed subscribe stream error");
                    break;
                }
            }
        }
        info!("feed subscribe ended");
    });
}

// ---------------------------------------------------------------------------
// HTTP endpoints
// ---------------------------------------------------------------------------

pub async fn feed_status(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(db) => db,
        Err(e) => {
            warn!(error = %e, "coordinator DB lock failed in feed_status");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "DB unavailable" })),
            )
                .into_response();
        }
    };

    let count = match db.count_feed_entries() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("count: {e}") })),
            )
                .into_response();
        }
    };

    let last_seq = match db.get_feed_last_seq() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("last_seq: {e}") })),
            )
                .into_response();
        }
    };

    let authors = match db.get_feed_author_stats() {
        Ok(a) => a,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("authors: {e}") })),
            )
                .into_response();
        }
    };

    let author_list: Vec<serde_json::Value> = authors
        .into_iter()
        .map(|(pubkey, count)| serde_json::json!({ "pubkey": pubkey, "count": count }))
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "count": count,
            "last_seq": last_seq,
            "authors": author_list,
        })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct FeedInsertRequest {
    pub op: nexus_coordinator_rs::public_feed::PublicFeedOperation,
}

pub async fn feed_insert(
    State(state): State<Arc<DaemonHttpState>>,
    Json(body): Json<FeedInsertRequest>,
) -> impl IntoResponse {
    let feed_state = match &state.feed_sync_state {
        Some(fs) => Arc::clone(fs),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "feed sync not initialized" })),
            )
                .into_response();
        }
    };

    let keypair = Arc::clone(&state.pow_keypair);
    let author_pubkey = hex::encode(keypair.public_bytes());

    let entry = {
        let db = match state.coordinator_db.lock() {
            Ok(db) => db,
            Err(e) => {
                warn!(error = %e, "coordinator DB lock failed in feed_insert");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "DB unavailable" })),
                )
                    .into_response();
            }
        };
        match public_feed::insert_feed_operation(&db, body.op, &author_pubkey, |data| {
            keypair.sign(data).to_vec()
        }) {
            Ok(e) => e,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response();
            }
        }
    };

    if let Err(e) = publish_feed_entry_to_docs(&feed_state, &entry).await {
        warn!(error = %e, "feed entry in DB but iroh-docs publish failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("publish failed: {e}"),
                "seq": entry.seq,
                "entry_hash": entry.entry_hash,
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "seq": entry.seq,
            "entry_hash": entry.entry_hash,
            "author_pubkey": entry.author_pubkey,
        })),
    )
        .into_response()
}

pub async fn feed_ticket(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    match &state.feed_sync_state {
        Some(fs) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ticket": fs.ticket })),
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "feed sync not initialized" })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct FeedJoinRequest {
    pub ticket: String,
}

pub async fn feed_join(
    State(state): State<Arc<DaemonHttpState>>,
    Json(body): Json<FeedJoinRequest>,
) -> impl IntoResponse {
    let ticket: DocsTicket = match body.ticket.parse() {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("invalid ticket: {e}") })),
            )
                .into_response();
        }
    };

    let docs_client = nexus_core_rs::docs::DocsClient::new(state.node.docs());

    // import_and_subscribe atomically: no window between import and
    // subscribe where initial sync events could be missed.
    let (doc_handle, live_stream) = match docs_client.import_and_subscribe(ticket).await {
        Ok(pair) => pair,
        Err(e) => {
            warn!(error = %e, "feed import_and_subscribe failed");
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
    let joined_key = format!("sbfb-feed-joined-{}", hex::encode(&ns_id_bytes[..4]));
    let joined_state = Arc::new(FeedSyncState {
        doc: Arc::new(doc_handle),
        author,
        ticket: body.ticket.clone(),
    });

    {
        let db = match state.coordinator_db.lock() {
            Ok(db) => db,
            Err(e) => {
                warn!(error = %e, "coordinator DB lock failed during feed join");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "DB unavailable" })),
                )
                    .into_response();
            }
        };
        if let Err(e) = db.set_storage_namespace(&joined_key, &ns_id_bytes, Some(&body.ticket)) {
            warn!(error = %e, "failed to persist joined feed namespace");
        }
    }

    // Spawn a task that first backfills existing entries (from the
    // iroh-docs namespace) then processes the live event stream.
    // Dedup via entry_hash UNIQUE index handles overlap.
    let feed_st = Arc::clone(&joined_state);
    let db_sp = Arc::clone(&state.coordinator_db);
    let node_sp = Arc::clone(&state.node);
    tokio::spawn(async move {
        // Backfill: iterate entries already present in the doc.
        match feed_st.doc.get_many_by_prefix(b"feed/").await {
            Ok(entries) => {
                info!(count = entries.len(), "backfilling existing feed entries");
                for doc_entry in &entries {
                    ingest_doc_entry(doc_entry, &node_sp, &db_sp).await;
                }
            }
            Err(e) => warn!(error = %e, "feed backfill scan failed"),
        }

        // Process live stream (captures events from import time).
        futures_lite::pin!(live_stream);
        info!("feed join subscribe active");
        while let Some(event) = live_stream.next().await {
            match event {
                Ok(DocsLiveEvent::InsertRemote {
                    entry: doc_entry, ..
                }) => {
                    ingest_doc_entry(&doc_entry, &node_sp, &db_sp).await;
                }
                Ok(_) => {}
                Err(e) => {
                    warn!(error = %e, "feed join stream error");
                    break;
                }
            }
        }
        info!("feed join subscribe ended");
    });

    (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_coordinator_rs::public_feed::GENESIS_PREV_HASH;

    #[test]
    fn test_format_feed_key() {
        let key = format_feed_key("abcdef1234567890", 42);
        assert_eq!(key, "feed/abcdef1234567890/0000000042");
    }

    #[test]
    fn test_format_feed_key_zero() {
        let key = format_feed_key("aa".repeat(32).as_str(), 0);
        assert!(key.starts_with("feed/"));
        assert!(key.ends_with("/0000000000"));
    }

    #[test]
    fn test_feed_entry_roundtrip_json() {
        let entry = FeedEntry {
            version: 1,
            seq: 1,
            op: nexus_coordinator_rs::public_feed::PublicFeedOperation::ReleasePublished(
                nexus_coordinator_rs::public_feed::ReleasePublishedPayload {
                    project_id: "a1".repeat(32),
                    repo_url: "https://github.com/org/app".to_string(),
                    commit_sha: "a".repeat(40),
                    artifact_hash: "b".repeat(64),
                    provenance_hash: Some("c".repeat(64)),
                    is_open_source: true,
                },
            ),
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            entry_hash: "e".repeat(64),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: "f".repeat(128),
        };
        let json = serde_json::to_vec(&entry).unwrap();
        let back: FeedEntry = serde_json::from_slice(&json).unwrap();
        assert_eq!(entry, back);
    }
}
