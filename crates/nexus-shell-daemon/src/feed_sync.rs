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
use nexus_coordinator_rs::public_feed::{self, FeedEntry, op_type, validate_feed_operation};
use nexus_core_rs::BlobsClient;
use nexus_core_rs::docs::{DocHandle, DocsAuthorId, DocsEntry, DocsLiveEvent, DocsTicket};
use nexus_shell_daemon_core::feed_limiter::FeedRateLimiter;

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

pub(crate) fn format_feed_key(author_hex: &str, seq: u64) -> String {
    format!("feed/{author_hex}/{seq:010}")
}

// ---------------------------------------------------------------------------
// Publish local entry → iroh-docs
// ---------------------------------------------------------------------------

pub async fn publish_feed_entry_to_docs(
    feed_state: &FeedSyncState,
    entry: &FeedEntry,
) -> Result<(), String> {
    let mut entry_with_pow = entry.clone();
    if entry_with_pow.pow_nonce.is_none() {
        entry_with_pow.pow_nonce = Some(public_feed::compute_feed_pow(&entry.entry_hash));
    }
    let key = format_feed_key(&entry.author_pubkey, entry.seq);
    let value = serde_json::to_vec(&entry_with_pow)
        .map_err(|e| format!("feed entry serialization: {e}"))?;
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
    op: serde_json::Value,
    author_pubkey: &str,
    sign_fn: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Result<FeedEntry, String> {
    let entry =
        nexus_coordinator_rs::public_feed::insert_feed_operation(db, op, author_pubkey, sign_fn)?;
    if let Err(e) = publish_feed_entry_to_docs(feed_state, &entry).await {
        warn!(error = %e, seq = entry.seq, "iroh-docs publish failed, attempting rollback");
        match db.delete_feed_entry_if_tail(&entry.entry_hash) {
            Ok(true) => info!(seq = entry.seq, "orphan feed entry rolled back"),
            Ok(false) => warn!(
                seq = entry.seq,
                "rollback skipped: another entry already chains on this one"
            ),
            Err(del_err) => warn!(error = %del_err, "rollback query failed"),
        }
        return Err(e);
    }
    Ok(entry)
}

// ---------------------------------------------------------------------------
// Seed announcement emit (Sprint 74 Phase F)
// ---------------------------------------------------------------------------

/// Emit one `SeedAnnounced` feed op for `(project_id, archive_hash)` signed by
/// `keypair` and publish it to iroh-docs.
///
/// The signer's public key is BOTH the FeedEntry `author_pubkey` AND the op's
/// `seeder_node_id` (F-3): the seeder signs only its own seed claim — never the
/// app's provenance — so authorship is never re-attributed (R5). The DB lock is
/// taken only for the synchronous insert and dropped BEFORE the async publish
/// (never held across an await, mirroring `feed_insert`).
pub(crate) async fn emit_seed_announced(
    feed_state: &FeedSyncState,
    coordinator_db: &std::sync::Mutex<CoordinatorDb>,
    keypair: &nexus_core_rs::KeyPair,
    project_id: &str,
    archive_hash: &str,
) -> Result<(), String> {
    let author_pubkey = hex::encode(keypair.public_bytes());
    let op = serde_json::to_value(public_feed::PublicFeedOperation::SeedAnnounced(
        public_feed::SeedAnnouncedPayload {
            project_id: project_id.to_string(),
            seeder_node_id: author_pubkey.clone(),
            archive_hash: archive_hash.to_string(),
        },
    ))
    .map_err(|e| format!("seed-announced op serialize: {e}"))?;

    let entry = {
        let db = coordinator_db
            .lock()
            .map_err(|e| format!("coordinator DB lock: {e}"))?;
        public_feed::insert_feed_operation(&db, op, &author_pubkey, |data| {
            keypair.sign(data).to_vec()
        })?
    };
    publish_feed_entry_to_docs(feed_state, &entry).await?;
    Ok(())
}

/// Re-emit a `SeedAnnounced` for every app this node is actively keeping online
/// (its `keep_online enabled = 1` rows with a known archive hash). Called once
/// at boot AFTER the feed namespace is ready, so a peer that rebooted re-tells
/// the network it still seeds those apps (the freshness basis of the count).
///
/// This is a NEW feed-emit path, NOT the gossip outbox replay: the outbox holds
/// PROJECT announcements (self-published apps), while `SeedAnnounced` is a FEED
/// op propagated via iroh-docs and covers BOTH self-deployed AND voluntarily-
/// seeded distant apps. Best-effort: a single failed emit is logged, the rest
/// proceed. Returns the count actually emitted.
pub(crate) async fn reannounce_seeds_at_boot(
    feed_state: &FeedSyncState,
    coordinator_db: &std::sync::Mutex<CoordinatorDb>,
    keypair: &nexus_core_rs::KeyPair,
) -> u64 {
    let rows = {
        let db = match coordinator_db.lock() {
            Ok(db) => db,
            Err(e) => {
                warn!(error = %e, "seed re-announce: coordinator DB lock failed");
                return 0;
            }
        };
        match db.list_keep_online_enabled() {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "seed re-announce: list_keep_online_enabled failed");
                return 0;
            }
        }
    };
    let mut emitted = 0u64;
    for (project_id, archive_hash) in rows {
        match emit_seed_announced(
            feed_state,
            coordinator_db,
            keypair,
            &project_id,
            &archive_hash,
        )
        .await
        {
            Ok(()) => emitted += 1,
            Err(e) => warn!(project = %project_id, error = %e, "seed re-announce emit failed"),
        }
    }
    if emitted > 0 {
        info!(emitted, "re-announced kept-online apps to the feed at boot");
    }
    emitted
}

// ---------------------------------------------------------------------------
// Ingest a single iroh-docs entry into the local feed DB
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
async fn ingest_doc_entry(
    doc_entry: &DocsEntry,
    node: &nexus_core_rs::Node,
    coordinator_db: &std::sync::Mutex<CoordinatorDb>,
    feed_limiter: &FeedRateLimiter,
    seed_registry: &crate::seed_registry::SeedRegistry,
    my_node_id: &str,
    apply_rate_limit: bool,
) {
    let key_bytes = doc_entry.key();
    let key_str = String::from_utf8_lossy(key_bytes);
    if !key_str.starts_with("feed/") {
        return;
    }

    // Blob content may not be downloaded yet when InsertRemote fires
    // (iroh-docs syncs metadata before content). Retry with backoff.
    let blobs = BlobsClient::new(node.blobs_store());
    let hash_bytes = *doc_entry.content_hash().as_bytes();
    let content = {
        let mut backoff = std::time::Duration::from_millis(50);
        let max_backoff = std::time::Duration::from_secs(2);
        loop {
            match blobs.get_bytes(hash_bytes).await {
                Ok(b) => break b,
                Err(e) => {
                    if backoff > max_backoff {
                        warn!(key = %key_str, error = %e, "blob unavailable after retries");
                        return;
                    }
                    debug!(
                        key = %key_str,
                        wait_ms = backoff.as_millis(),
                        "blob not ready, retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff = backoff.saturating_mul(3);
                }
            }
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

    let now_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if let Err(e) = public_feed::validate_feed_entry_timestamp(&feed_entry, now_epoch) {
        warn!(key = %key_str, error = %e, "feed entry timestamp rejected");
        return;
    }

    if let Err(e) = validate_feed_operation(&feed_entry.op) {
        warn!(key = %key_str, error = %e, "feed operation validation failed");
        return;
    }

    match feed_entry.pow_nonce {
        Some(nonce) => {
            if !public_feed::verify_feed_pow(&feed_entry.entry_hash, nonce) {
                warn!(key = %key_str, "feed entry rejected: invalid PoW nonce");
                return;
            }
        }
        None => {
            warn!(key = %key_str, "feed entry rejected: missing PoW nonce");
            return;
        }
    }

    // Dedup BEFORE rate-limit: existing entries skip without
    // consuming a GCRA token. Prevents backfill of 6+ historical
    // entries from exhausting the author's quota.
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

    // Release DB lock before rate-limit check so the token is
    // consumed only for genuinely new entries.
    drop(db);

    if apply_rate_limit && !feed_limiter.check_author(&feed_entry.author_pubkey) {
        warn!(
            key = %key_str,
            author = &feed_entry.author_pubkey[..8],
            "feed entry rejected: author rate limit exceeded"
        );
        return;
    }

    let db = match coordinator_db.lock() {
        Ok(db) => db,
        Err(e) => {
            warn!(error = %e, "coordinator DB lock failed in feed ingest");
            return;
        }
    };

    let op_type_str = op_type(&feed_entry.op).unwrap_or("Unknown");
    let payload = match serde_json::to_string(&feed_entry.op) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "feed op re-serialization failed");
            return;
        }
    };

    let row = nexus_coordinator_rs::db::FeedEntryRow {
        seq: 0,
        op_type: op_type_str.to_string(),
        payload,
        author: feed_entry.author_pubkey.clone(),
        signature: feed_entry.signature.clone(),
        entry_hash: feed_entry.entry_hash.clone(),
        prev_hash: feed_entry.prev_hash.clone(),
        created_at: feed_entry.timestamp,
    };

    match db.insert_feed_entry(&row) {
        Ok(seq) => {
            // Hot incremental reindex (Sprint 73 Phase C): make the freshly
            // ingested project searchable at once instead of only at the next
            // boot rebuild. Same `db` lock scope as the insert, so the short
            // FTS5 upsert shares the critical section. Best-effort relative to
            // the durable feed insert: on failure the entry is still stored
            // and will be picked up by the next rebuild_from_feed.
            if let Err(e) = nexus_coordinator_rs::search::upsert_feed_entry(
                &db,
                seq,
                &feed_entry.op,
                op_type_str,
            ) {
                warn!(
                    seq,
                    error = %e,
                    "hot search reindex failed (entry stored, searchable after next rebuild)"
                );
            }
            // Sprint 74 Phase F: feed the best-effort multi-seed registry. A
            // SeedAnnounced op from a REMOTE peer (author == seeder, seeder !=
            // me) refreshes the "Toi + N pairs (vus recemment)" count; every
            // other op (and our own echoed announcement) is ignored. The
            // freshness basis is the entry's own timestamp, so a stale
            // re-delivery never resurrects an expired seeder — and the
            // registry clamps it to our local receive clock (SEED-1), so a
            // forged FUTURE timestamp cannot stay "fresh" past the TTL.
            let recv_now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if seed_registry.record_announced(
                &feed_entry.op,
                &feed_entry.author_pubkey,
                my_node_id,
                feed_entry.timestamp,
                recv_now,
            ) {
                debug!(
                    author = &feed_entry.author_pubkey[..8],
                    "seed announcement recorded in multi-seed registry"
                );
            }
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

#[allow(clippy::too_many_arguments)]
pub fn spawn_feed_subscribe(
    feed_state: Arc<FeedSyncState>,
    coordinator_db: Arc<std::sync::Mutex<CoordinatorDb>>,
    node: Arc<nexus_core_rs::Node>,
    feed_limiter: Arc<FeedRateLimiter>,
    seed_registry: Arc<crate::seed_registry::SeedRegistry>,
    my_node_id: String,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut backoff = std::time::Duration::from_millis(500);
        let max_backoff = std::time::Duration::from_secs(30);
        let mut shutdown = shutdown;

        loop {
            let stream = match tokio::time::timeout(
                std::time::Duration::from_secs(30),
                feed_state.doc.subscribe(),
            )
            .await
            {
                Ok(Ok(s)) => s,
                Ok(Err(e)) => {
                    warn!(error = %e, "feed subscribe failed, retrying");
                    tokio::select! {
                        _ = tokio::time::sleep(backoff) => {}
                        _ = shutdown.changed() => { return; }
                    }
                    backoff = (backoff * 2).min(max_backoff);
                    continue;
                }
                Err(_) => {
                    warn!("feed subscribe timed out (30s), retrying");
                    tokio::select! {
                        _ = tokio::time::sleep(backoff) => {}
                        _ = shutdown.changed() => { return; }
                    }
                    backoff = (backoff * 2).min(max_backoff);
                    continue;
                }
            };

            info!("feed subscribe active");
            backoff = std::time::Duration::from_millis(500);
            let mut stream = stream;

            loop {
                tokio::select! {
                    event = stream.next() => {
                        match event {
                            Some(Ok(DocsLiveEvent::InsertRemote {
                                entry: doc_entry, ..
                            })) => {
                                ingest_doc_entry(&doc_entry, &node, &coordinator_db, &feed_limiter, &seed_registry, &my_node_id, true).await;
                            }
                            Some(Ok(_)) => {}
                            Some(Err(e)) => {
                                warn!(error = %e, "feed subscribe stream error, reconnecting");
                                break;
                            }
                            None => {
                                info!("feed subscribe stream ended, reconnecting");
                                break;
                            }
                        }
                    }
                    _ = shutdown.changed() => {
                        info!("feed subscribe shutting down");
                        return;
                    }
                }
            }

            tokio::select! {
                _ = tokio::time::sleep(backoff) => {}
                _ = shutdown.changed() => { return; }
            }
            backoff = (backoff * 2).min(max_backoff);
        }
    })
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
    pub op: serde_json::Value,
}

pub async fn feed_insert(
    State(state): State<Arc<DaemonHttpState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<FeedInsertRequest>,
) -> impl IntoResponse {
    let internal = headers
        .get("x-sbfb-feed-internal")
        .and_then(|v| v.to_str().ok())
        == Some("1");
    if !internal {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "feed insert requires internal auth"
            })),
        )
            .into_response();
    }

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
        warn!(error = %e, seq = entry.seq, "iroh-docs publish failed, attempting rollback");
        if let Ok(db) = state.coordinator_db.lock() {
            match db.delete_feed_entry_if_tail(&entry.entry_hash) {
                Ok(true) => info!(seq = entry.seq, "orphan feed entry rolled back"),
                Ok(false) => warn!(
                    seq = entry.seq,
                    "rollback skipped: another entry already chains on this one"
                ),
                Err(del_err) => warn!(error = %del_err, "rollback query failed"),
            }
        }
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("publish failed: {e}"),
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

    if let Ok(mut handles) = state.feed_join_handles.lock() {
        handles.retain(|h| !h.is_finished());
        const MAX_FEED_JOINS: usize = 10;
        if handles.len() >= MAX_FEED_JOINS {
            warn!(
                active = handles.len(),
                "feed_join cap reached, rejecting new join"
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({ "error": "too many active feed joins" })),
            )
                .into_response();
        }
    }

    let mut shutdown_rx = state.feed_join_shutdown.subscribe();
    let feed_st = Arc::clone(&joined_state);
    let db_sp = Arc::clone(&state.coordinator_db);
    let node_sp = Arc::clone(&state.node);
    let limiter_sp = Arc::clone(&state.feed_rate_limiter);
    let registry_sp = Arc::clone(&state.seed_registry);
    let my_node_id = state.node_id.clone();
    let handle = tokio::spawn(async move {
        match feed_st.doc.get_many_by_prefix(b"feed/").await {
            Ok(entries) => {
                info!(count = entries.len(), "backfilling existing feed entries");
                for doc_entry in &entries {
                    ingest_doc_entry(
                        doc_entry,
                        &node_sp,
                        &db_sp,
                        &limiter_sp,
                        &registry_sp,
                        &my_node_id,
                        false,
                    )
                    .await;
                }
            }
            Err(e) => warn!(error = %e, "feed backfill scan failed"),
        }

        futures_lite::pin!(live_stream);
        info!("feed join subscribe active");
        loop {
            tokio::select! {
                event = live_stream.next() => {
                    match event {
                        Some(Ok(DocsLiveEvent::InsertRemote {
                            entry: doc_entry, ..
                        })) => {
                            ingest_doc_entry(&doc_entry, &node_sp, &db_sp, &limiter_sp, &registry_sp, &my_node_id, true).await;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            warn!(error = %e, "feed join stream error");
                            break;
                        }
                        None => break,
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("feed join subscribe shutting down");
                    break;
                }
            }
        }
        info!("feed join subscribe ended");
    });

    if let Ok(mut handles) = state.feed_join_handles.lock() {
        handles.push(handle);
    }

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
    fn test_subscribe_stream_break_backoff_progression() {
        let initial = std::time::Duration::from_millis(500);
        let max_backoff = std::time::Duration::from_secs(30);
        let mut backoff = initial;

        assert_eq!(backoff, std::time::Duration::from_millis(500));

        backoff = (backoff * 2).min(max_backoff);
        assert_eq!(backoff, std::time::Duration::from_secs(1));

        backoff = (backoff * 2).min(max_backoff);
        assert_eq!(backoff, std::time::Duration::from_secs(2));

        backoff = (backoff * 2).min(max_backoff);
        assert_eq!(backoff, std::time::Duration::from_secs(4));

        backoff = (backoff * 2).min(max_backoff);
        assert_eq!(backoff, std::time::Duration::from_secs(8));

        backoff = (backoff * 2).min(max_backoff);
        assert_eq!(backoff, std::time::Duration::from_secs(16));

        backoff = (backoff * 2).min(max_backoff);
        assert_eq!(backoff, max_backoff, "must cap at max_backoff");

        backoff = (backoff * 2).min(max_backoff);
        assert_eq!(backoff, max_backoff, "must stay at max_backoff");
    }

    #[test]
    fn test_subscribe_backoff_resets_on_success() {
        let initial = std::time::Duration::from_millis(500);
        let max_backoff = std::time::Duration::from_secs(30);
        let mut backoff = initial;

        for _ in 0..5 {
            backoff = (backoff * 2).min(max_backoff);
        }
        assert!(backoff > initial);

        backoff = initial;
        assert_eq!(
            backoff,
            std::time::Duration::from_millis(500),
            "backoff must reset to initial after successful subscribe"
        );
    }

    #[test]
    fn test_feed_entry_roundtrip_json() {
        let op = serde_json::to_value(
            nexus_coordinator_rs::public_feed::PublicFeedOperation::ReleasePublished(
                nexus_coordinator_rs::public_feed::ReleasePublishedPayload {
                    project_id: "a1".repeat(32),
                    repo_url: "https://github.com/org/app".to_string(),
                    commit_sha: "a".repeat(40),
                    artifact_hash: "b".repeat(64),
                    provenance_hash: Some("c".repeat(64)),
                    is_open_source: true,
                    project_name: None,
                    category: None,
                },
            ),
        )
        .unwrap();
        let entry = FeedEntry {
            version: 1,
            seq: 1,
            op,
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            entry_hash: "e".repeat(64),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: "f".repeat(128),
            pow_nonce: Some(42),
        };
        let json = serde_json::to_vec(&entry).unwrap();
        let back: FeedEntry = serde_json::from_slice(&json).unwrap();
        assert_eq!(entry, back);
    }

    /// §P57 real-frontier gate: a peer that pins a distant app re-announces it
    /// to the feed AFTER a reboot, and a SECOND node ingesting that feed sees
    /// its multi-seed count increment. Both sides are real iroh nodes syncing a
    /// real iroh-docs feed namespace — only the work is mocked away (there is
    /// none). multi_thread is mandatory: two iroh nodes each need the docs
    /// actor on a dedicated thread (P2-A-1, PATTERNS §P54).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn remote_seeder_reannounces_after_reboot_e2e() {
        use nexus_core_rs::KeyPair;
        use nexus_core_rs::docs::{DocsClient, DocsTicket};

        let project_id = "a".repeat(64);
        let archive_hash = "cc".repeat(32);

        // ---------- Node A: the remote seeder ----------
        let node_a = Arc::new(nexus_core_rs::create_node().await.expect("boot node A"));
        let docs_a = DocsClient::new(node_a.docs());
        let author_a = docs_a.author_default().await.expect("author A");
        let doc_a = Arc::new(docs_a.create_doc().await.expect("create doc A"));
        let ticket = doc_a.share_write().await.expect("share write ticket");

        let db_a = Arc::new(std::sync::Mutex::new(
            CoordinatorDb::open_in_memory().expect("db A"),
        ));
        // The seeder identity that signs the feed (== seeder_node_id, F-3).
        let a_keypair = KeyPair::generate();
        // A pins the distant app (the keep_online row the boot loop re-emits).
        db_a.lock()
            .unwrap()
            .set_keep_online(&project_id, true, Some(&archive_hash))
            .expect("set keep_online A");
        let fs_a = FeedSyncState {
            doc: Arc::clone(&doc_a),
            author: author_a,
            ticket: ticket.to_string(),
        };

        // ---------- Node B: the ingesting observer ----------
        let node_b = Arc::new(nexus_core_rs::create_node().await.expect("boot node B"));
        let docs_b = DocsClient::new(node_b.docs());
        let ticket_b: DocsTicket = ticket.to_string().parse().expect("parse ticket B");
        let (doc_b, live_stream_b) = docs_b
            .import_and_subscribe(ticket_b)
            .await
            .expect("B import+subscribe");
        let _doc_b = Arc::new(doc_b);
        let db_b = Arc::new(std::sync::Mutex::new(
            CoordinatorDb::open_in_memory().expect("db B"),
        ));
        let limiter_b = Arc::new(FeedRateLimiter::new());
        let registry_b = Arc::new(crate::seed_registry::SeedRegistry::new());
        let node_id_b = node_b.node_id();

        // B pumps the live feed stream through the real ingest path.
        let node_b_sp = Arc::clone(&node_b);
        let db_b_sp = Arc::clone(&db_b);
        let limiter_b_sp = Arc::clone(&limiter_b);
        let registry_b_sp = Arc::clone(&registry_b);
        let node_id_b_sp = node_id_b.clone();
        let pump = tokio::spawn(async move {
            futures_lite::pin!(live_stream_b);
            while let Some(ev) = live_stream_b.next().await {
                if let Ok(DocsLiveEvent::InsertRemote { entry, .. }) = ev {
                    ingest_doc_entry(
                        &entry,
                        &node_b_sp,
                        &db_b_sp,
                        &limiter_b_sp,
                        &registry_b_sp,
                        &node_id_b_sp,
                        true,
                    )
                    .await;
                }
            }
        });

        // ---------- Simulate A's reboot: re-announce its kept-online apps ----------
        let emitted = reannounce_seeds_at_boot(&fs_a, &db_a, &a_keypair).await;
        assert_eq!(emitted, 1, "A must re-emit exactly one SeedAnnounced");

        // ---------- B's registry must reflect the new seeder ----------
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ok = tokio::time::timeout(std::time::Duration::from_secs(20), async {
            loop {
                if registry_b.count_recent(&project_id, Some(&archive_hash), now) == 1 {
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
        .await;
        pump.abort();
        ok.expect("B should record A as a seeder within 20s");

        // The recorded seeder is EXACTLY A's feed identity (== seeder_node_id ==
        // author_pubkey, F-3), not B itself and not a corrupted id — assert WHICH
        // seeder, not just the count, so a mutation of the stored id is caught.
        let a_pub = hex::encode(a_keypair.public_bytes());
        assert_ne!(a_pub, node_id_b, "A's feed identity must differ from B");
        assert_eq!(
            registry_b.count_recent(&project_id, Some(&archive_hash), now),
            1
        );
        assert_eq!(
            registry_b.seeders_recent(&project_id, &archive_hash, now),
            vec![a_pub]
        );
    }
}
