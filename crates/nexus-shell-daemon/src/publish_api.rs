// SPDX-License-Identifier: AGPL-3.0-or-later
//! Publish loopback HTTP domain — extracted verbatim from `http.rs`
//! (Sprint 82 Phase S, PO-10 extended discipline: the domain's 18
//! router-driven and direct-call tests co-migrated below via
//! `crate::test_support`).
//!
//! `POST /publish` broadcasts a project announcement through the single
//! canonical announce path `crate::deploy::publish_announcement`
//! (Remediation #8) behind the Sprint 20 Phase B duress gate and the
//! Sprint 16 audit D-1 provenance-chain gate; `POST /publish-blob`
//! stores a zip archive as an iroh blob (Sprint 12 Phase A); `POST
//! /directory/publish` builds, signs and gossip-announces THIS node's
//! own catalog (Sprint 75 Phase B, verrou 1/verrou 4/lock-3 guards in
//! [`build_sign_announce_directory`]), with the monotone anti-rollback
//! revision counter and the Sprint 75 Phase E state-driven boot
//! re-announce. T0 tier: the routes stay registered in
//! `crate::http::build_router` inside `authed_routes` (loopback bearer +
//! Host + Origin) and re-point here by full path; route paths, JSON
//! shapes and status codes are unchanged. The SHARED `ErrorResponse`
//! DTO, `truncate_on_char_boundary`, `mint_blob_ticket` and
//! `wrap_payload_with_pow` stay in `http.rs` (multi-domain consumers).

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use nexus_core_rs::BlobsClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::http::{
    DaemonHttpState, ErrorResponse, mint_blob_ticket, truncate_on_char_boundary,
    wrap_payload_with_pow,
};

/// Body of `POST /publish`. Sprint 11 Phase A, extended Sprint 12.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    /// Project display name.
    pub project_name: String,
    /// Category tag (e.g. `"gov"`, `"investigation"`).
    pub category: String,
    /// Short description.
    pub description: String,
    /// List of app names available on this project.
    #[serde(default)]
    pub apps: Vec<String>,
    /// Hex hash of a zip blob already stored via `POST /publish-blob`.
    /// If present, the daemon mints a BlobTicket and includes it in
    /// the gossip announcement (Sprint 12 Phase A).
    #[serde(default)]
    pub archive_hash: Option<String>,
    /// URL of the public source code repository (Sprint 13 Phase B).
    #[serde(default)]
    pub repo_url: Option<String>,
    /// BLAKE3 hex hash of provenance.json (Sprint 14 Phase B).
    #[serde(default)]
    pub provenance_hash: Option<String>,
    /// Whether this project was deployed from a public repo with
    /// signed provenance (Sprint 16 Phase D). The coordinator sets
    /// this on every `deploy-from-repo` publish; private zip uploads
    /// and the legacy auto-publish path leave it at `false`. Workers
    /// running at consent level `OpenSource` only accept tasks from
    /// projects where this flag is true.
    #[serde(default)]
    pub is_open_source: bool,
}

/// Body of `POST /publish` (success). Sprint 11 Phase A.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub published: bool,
}

/// Body of `POST /publish-blob` (success). Sprint 12 Phase A.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishBlobResponse {
    /// Hex-encoded BLAKE3 hash of the stored blob.
    pub hash: String,
}

/// `POST /publish` — broadcast a project announcement via gossip
/// and add it to the local browse aggregator. Sprint 11 Phase A.
///
/// Called by the coordinator's `POST /project/publish` endpoint
/// (proxied through `/daemon/publish`) when the project has
/// `visibility=public`. The daemon constructs a
/// [`ProjectAnnouncement`] from the request body + its own
/// `node_id`, broadcasts it on the curator gossip topic, and
/// adds the resulting [`BrowseEntry`] to the aggregator so it
/// appears in the local `/browse` immediately.
///
/// [`BrowseEntry`]: nexus_shell_daemon_core::browse::BrowseEntry
pub(crate) async fn publish_project(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<PublishRequest>,
) -> Response {
    debug!(project = %req.project_name, "POST /publish");

    // Sprint 20 Phase B : in duress mode, short-circuit BEFORE
    // touching the gossip sender so the fake keypair never
    // signs a ProjectAnnouncement. The response says
    // `published: false` — the handler is authoritative, not
    // the peer observer, so a local UI getting a false-flag
    // response is fine; the wire saw nothing.
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (StatusCode::OK, Json(PublishResponse { published: false })).into_response();
    }

    // Sprint 16 audit finding D-1: the kickoff §D4 declares
    // `is_open_source` as "derived by coordinator, never
    // user-settable". The daemon is the gossip writer, so it
    // must refuse to flag a project open-source unless the
    // provenance chain (Sprint 14 deploy-from-repo) is present.
    // Without this check, any local process holding the bearer
    // token could submit `{"is_open_source": true, ...}` and
    // see workers at consent level L2 accept its tasks.
    if req.is_open_source && (req.provenance_hash.is_none() || req.repo_url.is_none()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "is_open_source=true requires both provenance_hash and repo_url \
                        (deploy-from-repo chain). The coordinator's \
                        `POST /project/deploy-from-repo` is the only supported path."
                    .to_string(),
            }),
        )
            .into_response();
    }

    // Per-app identity: blake3(project_name) hex — the same id the feed and
    // deploy already use. NOT node_id: one node hosts many apps, and keying the
    // browse card on node_id collapses them all to a single card (and gives the
    // same app a different id depending on the viewing node).
    let project_id = hex::encode(nexus_core_rs::crypto::blake3_hash(
        req.project_name.as_bytes(),
    ));

    // Remediation #8: route /publish through the single canonical
    // announce → broadcast → persist-to-outbox → index → cache helper in
    // `deploy.rs`, so the publish and deploy-from-repo paths can never diverge
    // (the deploy path used to skip the outbox persist).
    crate::deploy::publish_announcement(
        &state,
        crate::deploy::AnnouncementParams {
            project_id: &project_id,
            project_name: &req.project_name,
            category: &req.category,
            description: &req.description,
            apps: &req.apps,
            archive_hash: req.archive_hash.as_deref(),
            repo_url: req.repo_url.as_deref(),
            provenance_hash: req.provenance_hash.as_deref(),
            is_open_source: req.is_open_source,
        },
    )
    .await;

    (StatusCode::OK, Json(PublishResponse { published: true })).into_response()
}

/// Response for `POST /api/daemon/directory/publish`.
#[derive(Debug, Serialize)]
struct PublishDirectoryResponse {
    /// Hex of the publishing node's Ed25519 pubkey (== the signer).
    node_id: String,
    /// The monotone revision stamped on this directory.
    revision: u64,
    /// Number of apps advertised in the catalog.
    catalog_len: usize,
    /// Hex BLAKE3 hash of the stored signed directory blob.
    archive_hash: String,
}

/// `POST /api/daemon/directory/publish` — Sprint 75 Phase B. Build,
/// sign, blob-store, and gossip-announce THIS node's signed
/// [`nexus_core_rs::NodeDirectoryEntry`]: the catalog of apps it hosts,
/// advertised so fresh peers can PULL them (the discovery pivot — list
/// of nodes → a node's catalogue → download). Loopback-authenticated
/// like every `/api/daemon` route.
///
/// Anti-recentralization guards (kickoff §4): the node advertises only
/// its OWN apps (the browse aggregator's direct entries tagged with our
/// node id — never a peer's), signs with the LOCAL node keypair so
/// provenance stays the author's (verrou 4), and embeds no peer node id
/// anywhere (lock-3). The directory is a read-side projection of what we
/// host, never a write-side "publish to X" selector (verrou 1).
pub(crate) async fn publish_directory(State(state): State<Arc<DaemonHttpState>>) -> Response {
    debug!("POST /api/daemon/directory/publish");
    match build_sign_announce_directory(&state).await {
        Ok(DirectoryPublishOutcome::DuressNoop) => (
            StatusCode::OK,
            Json(serde_json::json!({ "published": false })),
        )
            .into_response(),
        Ok(DirectoryPublishOutcome::Published {
            node_id_hex,
            revision,
            catalog_len,
            archive_hash,
        }) => (
            StatusCode::OK,
            Json(PublishDirectoryResponse {
                node_id: node_id_hex,
                revision,
                catalog_len,
                archive_hash,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
            .into_response(),
    }
}

/// What [`build_sign_announce_directory`] produced.
pub(crate) enum DirectoryPublishOutcome {
    /// Duress mode: nothing was signed (never sign under the fake keypair).
    DuressNoop,
    /// A signed directory was built, blob-stored and (best-effort)
    /// gossip-announced.
    Published {
        node_id_hex: String,
        revision: u64,
        catalog_len: usize,
        archive_hash: String,
    },
}

/// Core of the directory authoring path, shared by the HTTP route and the
/// headless boot re-announce (Sprint 75 Phase E): build THIS node's signed
/// [`nexus_core_rs::NodeDirectoryEntry`] from the apps it actually holds,
/// blob-store it, and gossip-announce it with a fresh ticket + PoW.
///
/// Every anti-recentralization guard lives HERE so each caller (browser
/// route, scripted loopback call, headless boot driver) inherits them
/// identically: duress no-op BEFORE any signing, own-apps-only +
/// local-blob-held ownership gate (verrou 4), LOCAL node keypair
/// provenance, no peer node id anywhere (lock-3).
pub(crate) async fn build_sign_announce_directory(
    state: &Arc<DaemonHttpState>,
) -> Result<DirectoryPublishOutcome, String> {
    // Duress short-circuit BEFORE signing — never sign a directory under
    // the fake keypair (mirrors publish_project).
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return Ok(DirectoryPublishOutcome::DuressNoop);
    }

    // Source the OWN catalog: the apps this node hosts (direct entries tagged
    // with our node id). A node never advertises a peer's apps. `node.node_id()`
    // is the lowercase-hex encoding of the SAME Ed25519 key as
    // `pow_keypair.public_bytes()` below: on a real install both derive from one
    // secret (the daemon keypair IS the iroh secret), so the catalog membership
    // and the signed directory identity are the same key (verrou 4).
    let my_node_id = state.node.node_id();
    let own = state.browse_aggregator.own_entries(&my_node_id);

    // Build the directory signed with the node keypair. directory.node_id == the
    // signing pubkey == the dialable identity a puller dials.
    let node_pubkey = state.pow_keypair.public_bytes();
    let revision = next_directory_revision(state);
    let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
    let mut directory = nexus_core_rs::NodeDirectory::new(node_pubkey, revision);
    for e in &own {
        // Cap the catalog at NODE_DIRECTORY_MAX_ENTRIES so a pathological own-app
        // count cannot drive sign() into its over-cap error and 500 the route
        // (defense-in-depth; the gossip self-node_id guard already keeps a peer
        // from inflating own_entries).
        if directory.catalog.len() >= nexus_core_rs::NODE_DIRECTORY_MAX_ENTRIES {
            break;
        }
        // Only advertise PULLABLE apps with a well-formed content address: skip
        // an entry whose archive_hash is empty or not a valid BLAKE3 hash (exactly
        // 64 lowercase hex). The hash is NOT truncated — truncating a content
        // address yields a different, unfetchable hash; we skip the whole entry.
        let Some(archive_hash) = e
            .archive_hash
            .clone()
            .filter(|h| !h.is_empty() && nexus_core_rs::is_valid_archive_hash(h))
        else {
            continue;
        };
        // Content-addressing ownership guard: only advertise an app whose archive
        // blob this node ACTUALLY HOLDS locally. A gossiped ProjectAnnouncement
        // can forge `BrowseEntry.node_id == our node_id` (the gossip ingest does
        // not cross-check `ann.node_id` against the PoW publisher), so the
        // node_id filter alone is spoofable — a peer could otherwise trick us
        // into signing its app into OUR directory (verrou 4 violation). Requiring
        // local blob presence means a spoofed entry (whose blob we do not hold)
        // can never be signed in: content-addressing is the ownership truth, and
        // we only ever claim to host what we can actually serve.
        let Some(hash_arr) = hex::decode(&archive_hash)
            .ok()
            .and_then(|b| <[u8; 32]>::try_from(b.as_slice()).ok())
        else {
            continue;
        };
        if !matches!(blobs.has(hash_arr).await, Ok(true)) {
            continue;
        }
        // The DISPLAY fields are truncated to their NODE_DIRECTORY_*_MAX on a
        // UTF-8 boundary: the deploy/publish path imposes no length cap, so a
        // single over-long local description must NOT make sign() reject the
        // WHOLE catalog (a self-inflicted availability hole) — the app still
        // appears, just clamped.
        directory.catalog.push(nexus_core_rs::CatalogApp {
            project_id: truncate_on_char_boundary(
                &e.project_id,
                nexus_core_rs::NODE_DIRECTORY_PROJECT_ID_MAX,
            ),
            archive_hash,
            project_name: truncate_on_char_boundary(
                &e.project_name,
                nexus_core_rs::NODE_DIRECTORY_PROJECT_NAME_MAX,
            ),
            category: truncate_on_char_boundary(
                &e.category,
                nexus_core_rs::NODE_DIRECTORY_CATEGORY_MAX,
            ),
            description: truncate_on_char_boundary(
                &e.description,
                nexus_core_rs::NODE_DIRECTORY_DESCRIPTION_MAX,
            ),
        });
    }
    let catalog_len = directory.catalog.len();

    let entry = match nexus_core_rs::NodeDirectoryEntry::sign(directory, state.pow_keypair.as_ref())
    {
        Ok(entry) => entry,
        Err(e) => {
            return Err(format!("failed to sign node directory: {e}"));
        }
    };

    // Blob-store the signed entry JSON so peers can fetch it by ticket.
    let entry_bytes = match serde_json::to_vec(&entry) {
        Ok(b) => b,
        Err(e) => {
            return Err(format!("failed to serialize node directory: {e}"));
        }
    };
    let hash_hex = match blobs.add_bytes(entry_bytes).await {
        Ok(hash) => hex::encode(hash),
        Err(e) => {
            return Err(format!("failed to store node directory blob: {e}"));
        }
    };

    // Gossip-announce: PoW-wrap a NodeDirectoryAnnouncement and broadcast it.
    // Best-effort and LIVE-ONLY (a no-op while isolated): unlike the project
    // announce path this does NOT persist to the outbox — it does not need to.
    // The receive-side ingest arm that consumes a directory announcement is
    // `handle_directory_announcement` → `process_directory_announcement_bytes`
    // (Sprint 75 Phase C), and remote-catalog DURABILITY is handled
    // CONSUMER-side: a subscriber persists a re-fetch locator (`anchors.json`)
    // and re-pulls + re-validates at boot (`CuratorRuntime::repull_directories`).
    // The PRODUCER side re-emits at boot via `reannounce_directory_at_boot`
    // (Sprint 75 Phase E, the headless boot driver): state-driven on the
    // persisted revision counter, it re-builds + re-signs + re-announces this
    // same announcement so a subscribed peer ONLINE AT THIS ANCHOR'S BOOT
    // does not wait for the next manual publish. A subscriber that joins
    // LATER still needs a live overlap (boot-only re-emit, no outbox replay
    // for directory announcements — accepted residual of the Phase C
    // deferral closure).
    if let Ok(ticket) = mint_blob_ticket(state, &hash_hex).await {
        let announcement = nexus_shell_daemon_core::iroh_runtime::NodeDirectoryAnnouncement::new(
            node_pubkey,
            ticket,
        );
        if let Ok(payload) = announcement.to_bytes()
            && let Ok(envelope) = wrap_payload_with_pow(state, &payload)
        {
            let sender_guard = state.gossip_sender.read().await;
            if let Some(sender) = sender_guard.as_ref()
                && let Err(e) = sender.broadcast(envelope).await
            {
                debug!(error = %e, "node directory announce broadcast failed (non-fatal)");
            }
        }
    }

    debug!(
        revision,
        catalog = catalog_len,
        "published signed node directory"
    );
    Ok(DirectoryPublishOutcome::Published {
        node_id_hex: hex::encode(node_pubkey),
        revision,
        catalog_len,
        archive_hash: hash_hex,
    })
}

/// Sprint 75 Phase E — the PRODUCER side of directory durability (the
/// Phase C deferral): `publish_directory`'s gossip announce is LIVE-only
/// and never persisted to the outbox, so after a reboot a catalogue
/// publisher goes silent — without this, a subscribed peer online at the
/// anchor's boot would wait for the next manual publish to (re)discover
/// the catalogue. The re-emit is boot-only: a subscriber that joins
/// later still needs a live overlap (accepted residual). The
/// consumer-side re-pull (`repull_directories`) covers SUBSCRIBERS, not
/// the producer's own re-emission.
///
/// State-driven gate: only a node that ALREADY published a directory
/// (persisted revision > 0) re-builds, re-signs (revision bump, monotone)
/// and re-announces at boot. A node that never published stays silent —
/// this is not a default-on behaviour, and the re-announce is a gossip
/// EMIT of our own signed catalogue, never a fetch (verrou 5). The
/// rebuilt catalogue reflects the apps actually held at boot, through the
/// same ownership gate as the route.
pub(crate) async fn reannounce_directory_at_boot(state: &Arc<DaemonHttpState>) -> bool {
    if read_directory_revision(state) == 0 {
        return false;
    }
    match build_sign_announce_directory(state).await {
        Ok(DirectoryPublishOutcome::Published {
            revision,
            catalog_len,
            ..
        }) => {
            info!(
                revision,
                catalog = catalog_len,
                "producer directory re-announced at boot"
            );
            true
        }
        Ok(DirectoryPublishOutcome::DuressNoop) => false,
        Err(e) => {
            warn!(error = %e, "producer directory boot re-announce failed");
            false
        }
    }
}

/// On-disk shape of `<sbfb-home>/directory_revision.json`: the monotone
/// counter stamped on this node's published directory. Persisted so a
/// re-publish after a restart bumps past the last value rather than
/// resetting to 1 (which a subscribed peer would reject as a rollback).
#[derive(Debug, Serialize, Deserialize)]
struct DirectoryRevisionFile {
    schema_version: u32,
    revision: u64,
}

/// Read the persisted directory revision WITHOUT incrementing it. `0`
/// means this node never published a directory (no persisted counter, or
/// no resolvable home) — the state-driven gate
/// [`reannounce_directory_at_boot`] keys on: a non-producer must stay
/// silent at boot.
pub(crate) fn read_directory_revision(state: &DaemonHttpState) -> u64 {
    let Some(home) = state
        .sbfb_home
        .clone()
        .or_else(nexus_shell_daemon_core::auth::sbfb_home)
    else {
        return 0;
    };
    let path = home.join("directory_revision.json");
    std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice::<DirectoryRevisionFile>(&b).ok())
        .map(|f| f.revision)
        .unwrap_or(0)
}

/// Read the persisted directory revision, return `previous + 1`, and persist
/// the new value atomically. The home directory is `state.sbfb_home`,
/// resolved ONCE at daemon boot (explicit test override or
/// [`auth::sbfb_home`] `$SBFB_HOME` / `~/.sbfb`). WITHOUT a resolvable
/// home the counter would reset to 1 on every boot and a subscribed peer
/// would reject each re-publish as a revision rollback — the anti-rollback
/// control the `revision` field exists for would be inert (the shipped
/// systemd unit pins `SBFB_HOME` for exactly this reason). Best-effort on
/// the write side: an IO error skips the persist and still returns the
/// computed revision.
///
/// The read-modify-write is serialized by a process-wide lock so two
/// concurrent calls (the daemon runs on a multi-thread runtime) get
/// strictly-distinct, strictly-increasing revisions rather than both
/// reading the same value and signing two directories at the same revision
/// (which a peer would then reject the second of as a rollback). The
/// publish route and the boot re-announce are the only writers, both
/// in-process through [`build_sign_announce_directory`], so one
/// process-wide lock suffices.
fn next_directory_revision(state: &DaemonHttpState) -> u64 {
    static REVISION_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = REVISION_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let Some(home) = state
        .sbfb_home
        .clone()
        .or_else(nexus_shell_daemon_core::auth::sbfb_home)
    else {
        return 1;
    };
    let path = home.join("directory_revision.json");
    let next = read_directory_revision(state).saturating_add(1);
    if let Ok(body) = serde_json::to_vec_pretty(&DirectoryRevisionFile {
        schema_version: 1,
        revision: next,
    }) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, &body).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
    next
}

/// `POST /publish-blob` — store raw bytes as an iroh blob and
/// return the hex hash. Sprint 12 Phase A.
///
/// Called by the coordinator to upload a zip archive before
/// publishing. The coordinator then passes the hash to
/// `POST /publish` as `archive_hash`.
pub(crate) async fn publish_blob(
    State(state): State<Arc<DaemonHttpState>>,
    body: Bytes,
) -> impl IntoResponse {
    debug!(size = body.len(), "POST /publish-blob");
    // Sprint 20 Phase B : in duress mode, reject task / blob
    // dispatch with a generic 503. Matches the observable
    // surface of any daemon in a maintenance window — no
    // duress-specific signal.
    if crate::noop_identity::task_dispatch_in_duress(state.identity_mode)
        == crate::noop_identity::DispatchOutcome::Reject503
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "service in maintenance mode".to_string(),
            }),
        )
            .into_response();
    }
    let blobs = BlobsClient::new(state.node.blobs_store());
    match blobs.add_bytes(body.to_vec()).await {
        Ok(hash) => {
            let hash_hex = hex::encode(hash);
            (StatusCode::OK, Json(PublishBlobResponse { hash: hash_hex })).into_response()
        }
        Err(e) => {
            warn!(error = %e, "failed to store blob");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("blob store failed: {e}"),
                }),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use crate::http::BrowseListResponse;
    use crate::test_support::*;

    /// Sprint 75 Phase B: the authoring route builds a signed directory
    /// from the node's OWN apps, stores it as a verifiable blob, and the
    /// signature provenance is the node keypair (verrou 4). A remote
    /// node's app (different node_id) is excluded from our catalog.
    #[tokio::test]
    async fn publish_directory_route_signs_and_announces() {
        // sbfb_home is an isolated tempdir so the persisted revision counter does
        // not touch (or read a stale value from) the real ~/.sbfb via the
        // auth::sbfb_home fallback.
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let my_id = state.node.node_id();
        let a = "a".repeat(64);
        let b = "b".repeat(64);
        let c = "c".repeat(64);

        // The two OWN apps reference blobs the node actually HOLDS (the ownership
        // truth that blocks gossip-spoofed entries from being signed in).
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let ha = hex::encode(blobs.add_bytes(b"zip-babel".to_vec()).await.unwrap());
        let hb = hex::encode(blobs.add_bytes(b"zip-atlas".to_vec()).await.unwrap());
        let mut ea = own_browse_entry(&a, "Babel", Some(my_id.clone()));
        ea.archive_hash = Some(ha);
        let mut eb = own_browse_entry(&b, "Atlas", Some(my_id.clone()));
        eb.archive_hash = Some(hb);
        state.browse_aggregator.add_direct_entry(ea);
        state.browse_aggregator.add_direct_entry(eb);
        // A remote app discovered via gossip — different hosting node id (excluded
        // by the node_id filter before the blob check).
        state.browse_aggregator.add_direct_entry(own_browse_entry(
            &c,
            "RemoteApp",
            Some("dead".repeat(16)),
        ));

        let resp = publish_directory(axum::extract::State(state.clone())).await;
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["revision"], 1);
        assert_eq!(v["catalog_len"], 2);
        assert_eq!(
            v["node_id"].as_str().unwrap(),
            hex::encode(state.pow_keypair.public_bytes())
        );

        // Fetch the stored blob back and prove it is a verifiable signed
        // directory carrying only our OWN apps, sorted by project_id.
        let archive_hash = v["archive_hash"].as_str().unwrap();
        let hash: [u8; 32] = hex::decode(archive_hash).unwrap().try_into().unwrap();
        let bytes = blobs.get_bytes(hash).await.unwrap();
        let entry: nexus_core_rs::NodeDirectoryEntry = serde_json::from_slice(&bytes).unwrap();
        entry
            .verify_signature()
            .expect("published directory must verify");
        assert_eq!(entry.node_id, state.pow_keypair.public_bytes());
        assert_eq!(entry.directory.revision, 1);
        let ids: Vec<&str> = entry
            .directory
            .catalog
            .iter()
            .map(|app| app.project_id.as_str())
            .collect();
        assert_eq!(ids, vec![a.as_str(), b.as_str()]);
        assert!(
            entry
                .directory
                .catalog
                .iter()
                .all(|app| app.project_name != "RemoteApp"),
            "a remote node's app must never appear in our own directory"
        );
    }

    /// Sprint 75 Phase B: in duress mode the route never signs a
    /// directory under the fake keypair — it returns `published: false`
    /// before touching the keypair (mirrors `publish_project`).
    #[tokio::test]
    async fn publish_directory_noop_in_duress() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let resp = publish_directory(axum::extract::State(state.clone())).await;
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["published"], false);
    }

    /// Sprint 75 Phase B: the directory revision is a monotone counter
    /// persisted under sbfb_home, so a re-publish after a restart bumps
    /// past the last value rather than resetting to 1 (which a subscribed
    /// peer would reject as a rollback).
    #[tokio::test]
    async fn publish_directory_revision_is_monotone_across_publishes() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        let r1 = publish_directory(axum::extract::State(state.clone())).await;
        let b1 = to_bytes(r1.into_body(), usize::MAX).await.unwrap();
        let v1: serde_json::Value = serde_json::from_slice(&b1).unwrap();
        assert_eq!(v1["revision"], 1);

        let r2 = publish_directory(axum::extract::State(state.clone())).await;
        let b2 = to_bytes(r2.into_body(), usize::MAX).await.unwrap();
        let v2: serde_json::Value = serde_json::from_slice(&b2).unwrap();
        assert_eq!(v2["revision"], 2);
    }

    /// Sprint 75 Phase B: the revision counter persists on disk, so a logical
    /// restart (a fresh `DaemonHttpState` over the same home) continues the
    /// sequence rather than resetting to 1 — the scenario the doc comment
    /// motivates. Distinct from the same-state test above (which proves the
    /// write→read→write round-trip within one process lifetime).
    #[tokio::test]
    async fn publish_directory_revision_survives_logical_restart() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let s1 = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let r1 = publish_directory(axum::extract::State(s1)).await;
        let v1: serde_json::Value =
            serde_json::from_slice(&to_bytes(r1.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(v1["revision"], 1);

        // Fresh state, SAME on-disk home — simulates a daemon restart.
        let s2 = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let r2 = publish_directory(axum::extract::State(s2)).await;
        let v2: serde_json::Value =
            serde_json::from_slice(&to_bytes(r2.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(
            v2["revision"], 2,
            "the counter must survive a logical restart"
        );
    }

    /// Sprint 75 Phase B (review P1): production `DaemonHttpState` carries
    /// `sbfb_home: None`, so `next_directory_revision` MUST fall back to
    /// `auth::sbfb_home()` (`$SBFB_HOME` / `~/.sbfb`) — without it the counter
    /// resets to 1 on every boot and peers reject re-publishes as rollbacks.
    /// This drives the route with `sbfb_home: None` and only `$SBFB_HOME` set,
    /// the way production resolves it. (nextest runs each test in its own
    /// process, so the env mutation is isolated; no other test reads
    /// `$SBFB_HOME` via the fallback.)
    #[tokio::test]
    async fn publish_directory_revision_falls_back_to_sbfb_home_env() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // SAFETY (edition 2024): same env-mutation pattern the runtime tests use.
        unsafe {
            std::env::set_var("SBFB_HOME", tmp.path());
        }
        let state = mk_state().await; // sbfb_home: None — the production shape.
        let r1 = publish_directory(axum::extract::State(state.clone())).await;
        let v1: serde_json::Value =
            serde_json::from_slice(&to_bytes(r1.into_body(), usize::MAX).await.unwrap()).unwrap();
        let r2 = publish_directory(axum::extract::State(state.clone())).await;
        let v2: serde_json::Value =
            serde_json::from_slice(&to_bytes(r2.into_body(), usize::MAX).await.unwrap()).unwrap();
        unsafe {
            std::env::remove_var("SBFB_HOME");
        }
        assert_eq!(v1["revision"], 1, "first publish via env-resolved home");
        assert_eq!(
            v2["revision"], 2,
            "fallback home persists the counter (regression guard for the or_else fix)"
        );
    }

    /// Sprint 75 Phase B (review P1): the deploy/publish path imposes no length
    /// cap, but the directory signer enforces NODE_DIRECTORY_*_MAX. A single
    /// over-cap local app must NOT 500 the whole route — the field is truncated
    /// and the app still appears.
    #[tokio::test]
    async fn publish_directory_truncates_oversized_fields() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let my_id = state.node.node_id();
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let held = hex::encode(blobs.add_bytes(b"zip-babel".to_vec()).await.unwrap());
        let mut entry = own_browse_entry(&"a".repeat(64), "Babel", Some(my_id));
        entry.archive_hash = Some(held);
        entry.description = "x".repeat(nexus_core_rs::NODE_DIRECTORY_DESCRIPTION_MAX + 50);
        state.browse_aggregator.add_direct_entry(entry);

        let resp = publish_directory(axum::extract::State(state.clone())).await;
        assert_eq!(
            resp.status(),
            axum::http::StatusCode::OK,
            "an over-cap field must be clamped, not 500 the route"
        );
        let v: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(v["catalog_len"], 1);
        let archive_hash = v["archive_hash"].as_str().unwrap();
        let hash: [u8; 32] = hex::decode(archive_hash).unwrap().try_into().unwrap();
        let bytes = blobs.get_bytes(hash).await.unwrap();
        let signed: nexus_core_rs::NodeDirectoryEntry = serde_json::from_slice(&bytes).unwrap();
        signed
            .verify_signature()
            .expect("truncated directory must still verify");
        assert!(
            signed.directory.catalog[0].description.len()
                <= nexus_core_rs::NODE_DIRECTORY_DESCRIPTION_MAX,
            "description must be clamped to the cap"
        );
    }

    /// Sprint 75 Phase B (Codex round 2 GAP): a gossiped ProjectAnnouncement can
    /// forge `BrowseEntry.node_id == our node_id`. Such a spoofed entry — whose
    /// archive blob we do NOT hold — must never be signed into our directory
    /// (verrou 4: we only ever claim to host what we can actually serve).
    /// Content-addressing (local blob presence) is the ownership truth.
    #[tokio::test]
    async fn publish_directory_excludes_spoofed_unheld_blob() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let my_id = state.node.node_id();
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());

        // A real, locally-held app → legitimately advertised.
        let held = hex::encode(blobs.add_bytes(b"real-zip".to_vec()).await.unwrap());
        let mut real = own_browse_entry(&"a".repeat(64), "Real", Some(my_id.clone()));
        real.archive_hash = Some(held);
        state.browse_aggregator.add_direct_entry(real);

        // A spoofed entry: our node_id (as a remote gossip could forge), valid
        // hash FORMAT, but a blob we do NOT hold.
        let mut spoof = own_browse_entry(&"b".repeat(64), "Spoofed", Some(my_id));
        spoof.archive_hash = Some("c".repeat(64));
        state.browse_aggregator.add_direct_entry(spoof);

        let resp = publish_directory(axum::extract::State(state.clone())).await;
        let v: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(
            v["catalog_len"], 1,
            "only the locally-held app is advertised"
        );
        let hash: [u8; 32] = hex::decode(v["archive_hash"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let bytes = blobs.get_bytes(hash).await.unwrap();
        let entry: nexus_core_rs::NodeDirectoryEntry = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(entry.directory.catalog.len(), 1);
        assert_eq!(entry.directory.catalog[0].project_name, "Real");
        assert!(
            entry
                .directory
                .catalog
                .iter()
                .all(|app| app.project_name != "Spoofed"),
            "a spoofed entry whose blob we do not hold must never be signed in"
        );
    }

    /// Sprint 75 Phase B (Codex GAP): two CONCURRENT publishes (the daemon runs
    /// on a multi-thread runtime) must get strictly-distinct, monotone revisions
    /// — not both read the same value and sign two directories at the same
    /// revision (the second of which a peer would reject as a rollback). Guards
    /// the process-wide revision lock.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn publish_directory_concurrent_revisions_are_distinct() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let (ra, rb) = tokio::join!(
            publish_directory(axum::extract::State(state.clone())),
            publish_directory(axum::extract::State(state.clone())),
        );
        let va: serde_json::Value =
            serde_json::from_slice(&to_bytes(ra.into_body(), usize::MAX).await.unwrap()).unwrap();
        let vb: serde_json::Value =
            serde_json::from_slice(&to_bytes(rb.into_body(), usize::MAX).await.unwrap()).unwrap();
        let mut revs = [
            va["revision"].as_u64().unwrap(),
            vb["revision"].as_u64().unwrap(),
        ];
        revs.sort_unstable();
        assert_eq!(
            revs,
            [1, 2],
            "concurrent publishes must produce distinct monotone revisions"
        );
    }

    #[tokio::test]
    async fn publish_announcement_persists_to_outbox_for_replay() {
        // Remediation #8 real-frontier test (§P57): the canonical announce path
        // must persist its envelope to the outbox even when ISOLATED
        // (gossip_sender == None). That persist-while-isolated is what lets a
        // deploy-from-repo / publish app be replayed to peers on NeighborUp AND
        // restored into Browse at boot (#7). No mock: real PoW envelope, real
        // mpsc channel, real aggregator.
        use nexus_core_rs::crypto::blake3_hash;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<crate::runtime::GossipCmd>(8);
        let state = mk_state_with_mode_tx(nexus_core_rs::IdentityMode::Normal, tx).await;
        let pid = hex::encode(blake3_hash(b"Outbox Test App"));

        crate::deploy::publish_announcement(
            &state,
            crate::deploy::AnnouncementParams {
                project_id: &pid,
                project_name: "Outbox Test App",
                category: "tools",
                description: "persisted for replay",
                apps: &[],
                archive_hash: None,
                repo_url: None,
                provenance_hash: None,
                is_open_source: false,
            },
        )
        .await;

        // (a) the announce path pushed an Outbox command despite no live sender.
        let cmd = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("outbox send must arrive within 2s")
            .expect("channel open");
        let crate::runtime::GossipCmd::Outbox(payload) = cmd else {
            panic!("expected GossipCmd::Outbox, got a different command");
        };
        // (b) Sprint 75 Phase A: the outbox carries the UNWRAPPED announcement
        // payload (so every replay re-mints the address + re-stamps a fresh PoW),
        // NOT a frozen PoW envelope. It parses directly as a ProjectAnnouncement.
        assert!(
            nexus_shell_daemon_core::publish::is_project_announcement(&payload),
            "outbox entry must be the unwrapped announcement payload, not a PoW envelope"
        );
        let ann =
            nexus_shell_daemon_core::publish::ProjectAnnouncement::from_gossip_bytes(&payload)
                .expect("payload is a project announcement");
        assert_eq!(ann.project_id, pid);
        assert_eq!(ann.project_name, "Outbox Test App");
        // (c) the card is in the aggregator immediately as well.
        assert_eq!(state.browse_aggregator.direct_entry_count(), 1);
        assert!(state.browse_aggregator.get_direct_entry(&pid).is_some());
    }

    #[tokio::test]
    async fn vps_authoring_signs_own_directory() {
        // Plan §E.3 #4 + the Phase C producer-reannounce carry: the
        // headless authoring path — no HTTP route, no browser — signs THIS
        // node's directory with the node keypair, and the boot re-announce
        // is state-driven: a node that never published stays silent; one
        // that did re-signs at a bumped (monotone) revision.
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        // Never published → the boot re-announce must be a strict no-op.
        assert!(
            !reannounce_directory_at_boot(&state).await,
            "a node that never published a directory must stay silent at boot"
        );
        assert_eq!(read_directory_revision(&state), 0);

        // Publish headlessly via the boot-builder core (the same core the
        // HTTP route wraps): one OWN app whose blob this node holds.
        let my_id = state.node.node_id();
        let pid = "7".repeat(64);
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let held = hex::encode(blobs.add_bytes(b"vps-own-app-zip".to_vec()).await.unwrap());
        let mut e = own_browse_entry(&pid, "VpsApp", Some(my_id));
        e.archive_hash = Some(held);
        state.browse_aggregator.add_direct_entry(e);

        let out = build_sign_announce_directory(&state)
            .await
            .expect("headless publish must succeed");
        let DirectoryPublishOutcome::Published {
            node_id_hex,
            revision,
            catalog_len,
            archive_hash,
        } = out
        else {
            panic!("expected a Published outcome");
        };
        assert_eq!(revision, 1);
        assert_eq!(catalog_len, 1);
        assert_eq!(node_id_hex, hex::encode(state.pow_keypair.public_bytes()));
        // The stored blob is a verifiable signed directory — provenance is
        // the node keypair, no browser anywhere in this path.
        let hash: [u8; 32] = hex::decode(&archive_hash).unwrap().try_into().unwrap();
        let bytes = blobs.get_bytes(hash).await.unwrap();
        let entry: nexus_core_rs::NodeDirectoryEntry = serde_json::from_slice(&bytes).unwrap();
        entry
            .verify_signature()
            .expect("headless-published directory must verify");
        assert_eq!(entry.node_id, state.pow_keypair.public_bytes());

        // Reboot shape: the producer re-announce now fires and bumps the
        // monotone revision (a subscriber's persisted floor accepts it).
        assert!(
            reannounce_directory_at_boot(&state).await,
            "a publisher must re-announce its directory at boot"
        );
        assert_eq!(
            read_directory_revision(&state),
            2,
            "the boot re-announce re-signs at a bumped revision"
        );
    }

    #[tokio::test]
    async fn publish_returns_200_and_adds_direct_entry() {
        // Sprint 11 Phase A: POST /publish adds a direct entry
        // to the browse aggregator and returns published=true.
        // Gossip broadcast is skipped (sender is None in tests).
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        let body = serde_json::to_vec(&PublishRequest {
            project_name: "gov-officiel".into(),
            category: "gov".into(),
            description: "Le projet gouvernance".into(),
            apps: vec!["gov".into()],
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let pub_resp: PublishResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(pub_resp.published);

        // The direct entry must now appear in /browse.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let browse: BrowseListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(browse.entries.len(), 1);
        assert_eq!(browse.entries[0].project_name, "gov-officiel");
        assert_eq!(
            serde_json::to_string(&browse.entries[0].source).unwrap(),
            "\"direct\""
        );
    }

    // ---------------------------------------------------------
    // Sprint 16 audit finding D-1 regression
    // ---------------------------------------------------------

    #[tokio::test]
    async fn publish_rejects_is_open_source_without_provenance_chain() {
        // Sprint 16 audit finding D-1: a malicious local process
        // holding the bearer token must not be able to flag a
        // zip deploy as open source without going through the
        // coord's deploy-from-repo clone+verify+sign path. The
        // daemon rejects `is_open_source=true` unless both
        // `provenance_hash` and `repo_url` are present.
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        // Case 1: flag=true, no provenance_hash, no repo_url → 400
        let body = serde_json::to_vec(&PublishRequest {
            project_name: "fake-open-source".into(),
            category: "misc".into(),
            description: "pretend I'm OSS".into(),
            apps: vec![],
            archive_hash: Some("ab".repeat(32)),
            repo_url: None,
            provenance_hash: None,
            is_open_source: true,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // Case 2: flag=true, provenance_hash present, repo_url absent → 400
        let body = serde_json::to_vec(&PublishRequest {
            project_name: "fake-2".into(),
            category: "misc".into(),
            description: "still pretending".into(),
            apps: vec![],
            archive_hash: Some("ab".repeat(32)),
            repo_url: None,
            provenance_hash: Some("cd".repeat(32)),
            is_open_source: true,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // Case 3: flag=true, repo_url present, provenance_hash absent → 400
        let body = serde_json::to_vec(&PublishRequest {
            project_name: "fake-3".into(),
            category: "misc".into(),
            description: "one more try".into(),
            apps: vec![],
            archive_hash: Some("ab".repeat(32)),
            repo_url: Some("https://example.com/repo".into()),
            provenance_hash: None,
            is_open_source: true,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn publish_accepts_is_open_source_with_full_provenance_chain() {
        // Mirror of the D-1 reject test: the happy path — both
        // provenance_hash and repo_url present — passes. This is
        // what the coord's `POST /project/deploy-from-repo` emits
        // after cloning and signing.
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        let body = serde_json::to_vec(&PublishRequest {
            project_name: "legit-oss".into(),
            category: "gov".into(),
            description: "verified from repo".into(),
            apps: vec!["gov".into()],
            archive_hash: Some("ab".repeat(32)),
            repo_url: Some("https://github.com/example/sbfb-app".into()),
            provenance_hash: Some("cd".repeat(32)),
            is_open_source: true,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Browse entry must carry is_open_source=true.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let browse: BrowseListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(browse.entries.len(), 1);
        assert!(browse.entries[0].is_open_source);
    }

    #[tokio::test]
    async fn publish_blob_stores_and_returns_hash() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let zip_bytes = make_zip(&[("index.html", b"<h1>Hello</h1>")]);

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish-blob")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(zip_bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let res: PublishBlobResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(res.hash.len(), 64, "hash should be 32 bytes hex-encoded");
    }

    #[tokio::test]
    async fn publish_with_archive_hash_populates_browse_entry() {
        // Sprint 12 Phase D: POST /publish with archive_hash
        // sets archive_hash on the browse entry visible in /browse.
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        // Store a blob first.
        let zip_bytes = make_zip(&[("index.html", b"<h1>Hi</h1>")]);
        let blobs = BlobsClient::new(state.node.blobs_store());
        let hash = blobs.add_bytes(zip_bytes).await.unwrap();
        let hash_hex = hex::encode(hash);

        // Publish with archive_hash.
        let body = serde_json::to_vec(&PublishRequest {
            project_name: "web-app".into(),
            category: "misc".into(),
            description: "test archive".into(),
            apps: vec![],
            archive_hash: Some(hash_hex.clone()),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify /browse returns the entry with archive_hash.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let browse: BrowseListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 8192).await.unwrap()).unwrap();
        assert_eq!(browse.entries.len(), 1);
        assert_eq!(
            browse.entries[0].archive_hash.as_deref(),
            Some(hash_hex.as_str())
        );
        assert!(browse.entries[0].archive_ticket.is_some());
    }

    // =============================================================
    // Sprint 20 Phase B — duress runtime HTTP surface
    // =============================================================

    /// #B-rt-1 The `/publish` handler in Duress mode returns 200
    /// with `{published: false}` and does NOT add a direct entry
    /// to the browse aggregator. A peer observer sees no gossip
    /// broadcast (the gossip_sender is None here so we rely on
    /// the handler short-circuit before even reading the sender
    /// guard — the empty browse aggregator is the local witness).
    #[tokio::test]
    async fn daemon_boot_in_duress_mode_publishes_fake_curator_empty() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let app = build_test_router(Arc::clone(&state));

        let body = serde_json::to_vec(&PublishRequest {
            project_name: "real-project".into(),
            category: "gov".into(),
            description: "should-not-reach-wire".into(),
            apps: vec!["gov".into()],
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let pub_resp: PublishResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(
            !pub_resp.published,
            "Duress mode must report published=false (no wire broadcast)"
        );

        // The browse aggregator must be empty — no direct entry
        // was added under the fake identity.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let browse: BrowseListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(
            browse.entries.is_empty(),
            "browse aggregator must stay empty in Duress mode"
        );
    }

    /// #B-rt-3 The `/publish-blob` handler in Duress mode returns
    /// 503 with a generic "maintenance" payload — no signal that
    /// duress is active, just a plausible service-unavailable.
    #[tokio::test]
    async fn daemon_boot_in_duress_mode_rejects_task_dispatch() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let app = build_test_router(Arc::clone(&state));

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish-blob")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(b"fake blob bytes".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn publish_and_gossip_use_per_app_project_id() {
        // OFF-SPRINT-2b regression: the gossip ProjectAnnouncement carries the
        // per-app project_id (blake3(name)), distinct from the hosting node_id.
        // The node_id stays on the wire as the dialable identity, but the app
        // identity is per-app. Captures the real outbox envelope (no mock).
        use nexus_core_rs::crypto::blake3_hash;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<crate::runtime::GossipCmd>(8);
        let state = mk_state_with_mode_tx(nexus_core_rs::IdentityMode::Normal, tx).await;
        let pid = hex::encode(blake3_hash(b"Per App Gossip"));
        crate::deploy::publish_announcement(
            &state,
            crate::deploy::AnnouncementParams {
                project_id: &pid,
                project_name: "Per App Gossip",
                category: "tools",
                description: "x",
                apps: &[],
                archive_hash: None,
                repo_url: None,
                provenance_hash: None,
                is_open_source: false,
            },
        )
        .await;
        let cmd = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("outbox arrives")
            .expect("channel open");
        let crate::runtime::GossipCmd::Outbox(payload) = cmd else {
            panic!("expected GossipCmd::Outbox");
        };
        // Sprint 75 Phase A: the outbox carries the UNWRAPPED announcement payload
        // (each replay re-mints + re-stamps), so it parses directly — no PoW unwrap.
        let ann =
            nexus_shell_daemon_core::publish::ProjectAnnouncement::from_gossip_bytes(&payload)
                .unwrap();
        assert_eq!(ann.project_id, pid, "per-app id on the wire");
        assert_ne!(ann.project_id, ann.node_id, "per-app id is not the node_id");
    }
}
