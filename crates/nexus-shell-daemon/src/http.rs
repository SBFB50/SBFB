// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP surface for `nexus-shell-daemon`.
//!
//! The daemon's HTTP listener is loopback-only and reached by
//! the React shell exclusively through the coordinator
//! `/daemon/*` proxy (Sprint 7 D1). Phase A exposed two routes
//! (`/health`, `/info`); Phase C extends the surface with three
//! more that operate on the shared [`CuratorRuntime`]:
//!
//! - `GET    /health`              — liveness probe (Phase A)
//! - `GET    /info`                — daemon state snapshot (Phase A)
//! - `GET    /curators`            — list every cached curator list
//! - `POST   /curators/subscribe`  — add a curator to the attention set
//! - `DELETE /curators/{pubkey}`   — remove a curator from the attention set
//!
//! Phase D will grow `/browse`. That route is deliberately
//! **absent** here so the Phase D audit can isolate pkarr
//! resolution correctness from subscribe correctness.
//!
//! ## CORS
//!
//! The daemon trusts two and only two origins:
//!
//! - `http://127.0.0.1[:port]`
//! - `http://localhost[:port]`
//!
//! Even though the shell is expected to talk through the
//! coordinator proxy, we keep a strict loopback CORS layer on
//! the daemon itself so a future direct-call path cannot
//! silently widen the trust model.

use std::sync::Arc;
use std::time::SystemTime;

use axum::{
    body::Bytes,
    extract::{Path, Request, State},
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
    Router,
};
use nexus_core_rs::{
    BlobsClient, KeyPair, Node, PowEnvelope, PowSolveCache, RelayPowPolicy, TopicSender,
};
use nexus_shell_daemon_core::auth::{auth_required, AuthState};
use nexus_shell_daemon_core::blob_serve::{self, BlobServeCache};
use nexus_shell_daemon_core::browse::{
    BrowseAggregatorHandle, BrowseEntry, BrowseSource, BrowseStatus,
};
use nexus_shell_daemon_core::iroh_runtime::{CuratorRuntimeError, CuratorRuntimeHandle};
use nexus_shell_daemon_core::publish::ProjectAnnouncement;
use nexus_shell_daemon_core::state::{DaemonStateSnapshot, StateInputs};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{debug, warn};

/// Shared handle to the gossip topic sender. `None` until the
/// gossip task has joined the curator topic. Sprint 11 Phase A.
pub type GossipSenderHandle = Arc<RwLock<Option<TopicSender>>>;

/// Shared state handed to every axum route.
///
/// Holds the static fields needed by `/info` plus the
/// [`CuratorRuntimeHandle`] that curator routes and `/info`'s
/// subscribed-curators / known-lists / known-browse-entries
/// counters read from, the Phase D [`BrowseAggregatorHandle`],
/// and an `Arc<Node>` so the browse handler can drive the
/// [`nexus_core_rs::DiscoveryClient::probe_reachable`] path.
#[derive(Debug, Clone)]
pub struct DaemonHttpState {
    pub node_id: String,
    pub daemon_version: String,
    pub boot_time: SystemTime,
    pub api_host: String,
    pub api_port: u16,
    pub curator_runtime: CuratorRuntimeHandle,
    pub browse_aggregator: BrowseAggregatorHandle,
    /// Shared iroh node handle. The browse route reaches
    /// through the Arc to call `DiscoveryClient::probe_reachable`
    /// on the endpoint.
    pub node: Arc<Node>,
    /// Gossip topic sender handle. `None` until the gossip task
    /// has joined the curator topic. Used by `POST /publish` to
    /// broadcast project announcements. Sprint 11 Phase A.
    pub gossip_sender: GossipSenderHandle,
    /// Default curator pubkeys from `[curator]` config section.
    /// Sprint 11 Phase B. Exposed via `GET /default-curators`.
    pub default_curators: Vec<String>,
    /// Sprint 12 Phase A: LRU cache of decompressed zip archives
    /// for the blob-serve endpoint.
    pub blob_serve_cache: Arc<BlobServeCache>,
    /// Sprint 20 Phase B : duress-mode flag. Set to
    /// [`IdentityMode::Duress`] when the daemon was booted via an
    /// identity unlocked under the duress PIN. Every outbound
    /// handler routes through `crate::noop_identity` to noop
    /// publishes / subscribes under the fake keypair. Normal
    /// boots leave this at `IdentityMode::Normal` (the default).
    pub identity_mode: nexus_core_rs::IdentityMode,
    /// Sprint 20 Phase B : panic wipe service, provisioned at
    /// boot with the real keystore + state-db + blob-cache paths
    /// and the production [`crate::panic::RealExit`] strategy.
    /// Consumed by the `POST /panic/wipe` handler.
    pub panic_wipe: Arc<crate::panic::PanicWipeService>,
    /// Sprint 20 Phase C : PoW solve cache (publisher side). Each
    /// outbound broadcast wraps its payload with a Hashcash proof
    /// minted via [`PowSolveCache::ensure_proof`]. The cache keeps
    /// one live proof per topic for 15 min, so a chatty publisher
    /// pays the ~100 ms solve twice an hour, not per-message.
    pub pow_solve_cache: Arc<PowSolveCache>,
    /// Sprint 20 Phase C : shared PoW policy handle. Read on every
    /// solve so a hot-reloaded `relay_pow_policy.toml` (cf.
    /// [`nexus_shell_daemon_core::pow_policy_loader::PowPolicyWatcher`])
    /// takes effect for the next outbound broadcast without a
    /// restart.
    pub pow_policy: Arc<std::sync::RwLock<RelayPowPolicy>>,
    /// Sprint 20 Phase C : the daemon's long-lived Ed25519 keypair,
    /// used as the `publisher_pubkey` anchor in Hashcash challenges.
    /// When the launcher's `sbfb unlock` hands over a persistent
    /// identity, the keypair matches the node's iroh secret ; on
    /// legacy `cargo run` paths a fresh ephemeral keypair is minted
    /// alongside the ephemeral iroh identity.
    pub pow_keypair: Arc<KeyPair>,
    /// Sprint 20 Phase C : pre-computed 32-byte curator gossip
    /// topic id. The publish handler uses this to key the
    /// [`PowSolveCache`] instead of recomputing the BLAKE3 hash on
    /// every broadcast.
    pub curator_gossip_topic: [u8; 32],
}

impl DaemonHttpState {
    fn snapshot(&self) -> DaemonStateSnapshot {
        // Read the live curator runtime counts rather than
        // leaving the Phase A zero-fallback in place. This is
        // additive — the snapshot schema stays at v1, only the
        // populated values change.
        let subscribed_curators = self.curator_runtime.subscribed_pubkeys_hex();
        let known_lists = self.curator_runtime.known_list_count() as u32;
        let known_browse_entries = self.curator_runtime.known_entry_count() as u32;

        DaemonStateSnapshot::from_inputs(StateInputs {
            node_id: self.node_id.clone(),
            daemon_version: self.daemon_version.clone(),
            boot_time: self.boot_time,
            api_host: self.api_host.clone(),
            api_port: self.api_port,
            subscribed_curators,
            known_lists,
            known_browse_entries,
        })
    }
}

/// Build the axum [`Router`] carrying every Phase A + Phase C
/// route. The caller hands us an [`Arc<DaemonHttpState>`] plus
/// the loopback bearer token; the router clones the state into
/// each handler via the axum `State` extractor and applies the
/// [`auth_required`] middleware on every non-public route.
///
/// ## Auth exemption (Sprint 16 Phase A)
///
/// Two route surfaces bypass the bearer/Host/Origin middleware:
///
/// - `GET /health` — liveness probe, reached by the launcher
///   before it has the token.
/// - `/blob-serve/{hash}/{*path}` — served into a sandboxed iframe
///   with CSP `connect-src 'none'` (Sprint 12 Phase A). The
///   iframe cannot inject a custom header, and its Origin is
///   `null` because the sandbox strips same-origin. The blob
///   content is already public by construction (anyone on the
///   P2P network can fetch the zip by hash), so exempting the
///   route does not leak anything new.
pub fn build_router(state: Arc<DaemonHttpState>, auth: AuthState) -> Router {
    // Sprint 13 Phase A (T37): blob-serve routes get a CSP
    // middleware that injects security headers on ALL responses
    // (200, 400, 404, 500) — not just the success path.
    let blob_serve_routes = Router::new()
        .route("/{hash}/{*path}", get(blob_serve))
        .layer(middleware::from_fn(blob_serve_csp_middleware));

    // Public routes: no bearer, no Host check, no Origin check.
    let public_routes = Router::new()
        .route("/health", get(health))
        .nest("/blob-serve", blob_serve_routes);

    // Sprint 18 audit fix D-1 : the caller picks the variant
    // (`AuthState::Static` for the legacy single-token boot path,
    // `AuthState::Rotated` once the launcher writes a `tokens.json`).
    // The middleware reads the inner state on every request so a
    // rotation reaches `auth_required` without rebuilding the router.

    // Authenticated surface: every other route requires
    // X-SBFB-Token + loopback Host + (absent or loopback) Origin.
    let authed_routes = Router::new()
        .route("/info", get(info))
        .route("/curators", get(list_curators))
        .route("/curators/subscribe", post(subscribe_curator))
        .route("/curators/{pubkey}", delete(unsubscribe_curator))
        .route("/browse", get(list_browse))
        .route("/publish", post(publish_project))
        .route("/publish-blob", post(publish_blob))
        .route("/default-curators", get(default_curators))
        // Sprint 20 Phase B : panic wipe endpoint. Behind the
        // same loopback bearer + Host + Origin gate as every
        // other authenticated route so only a co-located shell
        // with the rotated token can trigger it.
        .route("/panic/wipe", post(panic_wipe))
        .layer(middleware::from_fn_with_state(auth, auth_required));

    Router::new()
        .merge(public_routes)
        .merge(authed_routes)
        .with_state(state)
        .layer(loopback_cors_layer())
}

/// Middleware that injects CSP + X-Content-Type-Options headers
/// on every blob-serve response, including error responses.
/// Sprint 13 Phase A (T37).
async fn blob_serve_csp_middleware(request: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    // Safe to unwrap: both values are compile-time constants.
    headers.insert(
        "content-security-policy",
        blob_serve::BLOB_SERVE_CSP.parse().unwrap(),
    );
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    response
}

/// The loopback-only CORS layer. Accepts exactly the origins
/// `http://127.0.0.1[:PORT]` and `http://localhost[:PORT]`;
/// refuses everything else, including HTTPS variants.
fn loopback_cors_layer() -> CorsLayer {
    CorsLayer::new().allow_origin(AllowOrigin::predicate(
        |origin: &HeaderValue, _request_parts: &_| is_loopback_origin(origin),
    ))
}

/// Return `true` iff `origin` is an HTTP loopback URL with an
/// optional port and no path.
pub fn is_loopback_origin(origin: &HeaderValue) -> bool {
    let Ok(s) = origin.to_str() else {
        return false;
    };
    let rest = match s.strip_prefix("http://") {
        Some(r) => r,
        None => return false,
    };

    let host_port = rest.split('/').next().unwrap_or(rest);
    let (host, port_opt) = match host_port.rsplit_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (host_port, None),
    };

    if host != "127.0.0.1" && host != "localhost" {
        return false;
    }
    if let Some(p) = port_opt {
        if p.parse::<u16>().is_err() {
            return false;
        }
    }
    true
}

// =================================================================
// Request / response DTOs
// =================================================================

/// Body of `POST /curators/subscribe`.
///
/// `deny_unknown_fields` (Sprint 8 audit G-3): the previous
/// definition silently ignored extra JSON fields because that is
/// serde's default. Defense-in-depth demands that a future
/// extension requiring a new field must be wired through on
/// both ends at the same commit — an extra field in the body
/// now fails loud with a 422 instead of being dropped on the
/// floor and forgotten.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscribeCuratorRequest {
    /// Lowercase hex of the curator's Ed25519 public key (64 chars).
    pub curator_pubkey_hex: String,
}

/// Body of both `POST /curators/subscribe` (success) and
/// `DELETE /curators/{pubkey}` (success). Returns the current
/// sorted list of subscribed curator pubkeys so the shell can
/// refresh its UI in a single roundtrip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionsResponse {
    pub subscribed_curators: Vec<String>,
}

/// Body of `GET /curators`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CuratorsListResponse {
    /// Every signed curator list the daemon has cached, keyed
    /// by curator pubkey (sorted ascending). Each entry is a
    /// verbatim `CuratorListEntry` — the shell re-renders it
    /// with the trust-by-construction invariant that any entry
    /// in this array has already been verified.
    pub entries: Vec<nexus_core_rs::CuratorListEntry>,
    /// The current attention set (sorted hex pubkeys). The
    /// shell compares this against `entries` to render "waiting
    /// on first announcement" placeholders for subscribed
    /// curators that have not yet broadcast.
    pub subscribed_curators: Vec<String>,
}

/// Body of `GET /browse`.
///
/// Sorted flat list of every project entry across every cached
/// curator list, each row carrying a reachability bucket the
/// React shell renders as a coloured dot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowseListResponse {
    pub entries: Vec<BrowseEntry>,
}

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

/// Body of `GET /default-curators`. Sprint 11 Phase B.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultCuratorsResponse {
    /// Configured default curator Ed25519 public keys (hex).
    pub default_curators: Vec<String>,
}

/// Body of `POST /publish-blob` (success). Sprint 12 Phase A.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishBlobResponse {
    /// Hex-encoded BLAKE3 hash of the stored blob.
    pub hash: String,
}

/// Body returned when a curator runtime error must be surfaced
/// as a 4xx/5xx response.
#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    error: String,
}

fn runtime_error_to_response(err: CuratorRuntimeError) -> (StatusCode, Json<ErrorResponse>) {
    // Sprint 8 audit C-2 split: `NotSubscribed` and
    // `EnvelopeMismatch` inherit the legacy 422 mapping, so
    // subscribe/unsubscribe callers (which never surface them in
    // practice) continue to behave identically. The split
    // matters to the gossip handler, not to the HTTP surface.
    let status = match &err {
        CuratorRuntimeError::BadPubkeyHex(_) => StatusCode::BAD_REQUEST,
        CuratorRuntimeError::AnnouncementParse(_)
        | CuratorRuntimeError::AnnouncementVersion { .. }
        | CuratorRuntimeError::NotSubscribed { .. }
        | CuratorRuntimeError::EnvelopeMismatch { .. }
        | CuratorRuntimeError::EntryParse(_)
        | CuratorRuntimeError::EntryVerify(_)
        | CuratorRuntimeError::RevisionRollback { .. }
        | CuratorRuntimeError::BlobFetch(_) => StatusCode::UNPROCESSABLE_ENTITY,
        CuratorRuntimeError::Persistence { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        Json(ErrorResponse {
            error: err.to_string(),
        }),
    )
}

// =================================================================
// Handlers
// =================================================================

/// `GET /health` — liveness probe.
async fn health(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /health");
    Json(serde_json::json!({
        "status": "ok",
        "schema_version": nexus_shell_daemon_core::state::SCHEMA_VERSION,
        "daemon_version": state.daemon_version,
    }))
}

/// `GET /info` — full [`DaemonStateSnapshot`] for the shell's
/// Browse / Curators page header.
async fn info(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /info");
    (StatusCode::OK, Json(state.snapshot()))
}

/// `GET /curators` — list every cached curator list + the
/// current attention set.
async fn list_curators(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /curators");
    let body = CuratorsListResponse {
        entries: state.curator_runtime.list_snapshot(),
        subscribed_curators: state.curator_runtime.subscribed_pubkeys_hex(),
    };
    (StatusCode::OK, Json(body))
}

/// `POST /curators/subscribe` — add a curator pubkey to the
/// attention set. Idempotent: subscribing twice is a no-op that
/// still returns 200.
async fn subscribe_curator(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SubscribeCuratorRequest>,
) -> impl IntoResponse {
    debug!(curator = %req.curator_pubkey_hex, "POST /curators/subscribe");
    // Sprint 20 Phase B : in duress mode, accept the request but
    // do NOT mutate the attention set. The shell still sees a
    // 200 so the UI is quiet; the fake identity never subscribes
    // a real curator.
    if crate::noop_identity::curator_subscribe_in_duress(state.identity_mode)
        == crate::noop_identity::SubscribeOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(SubscriptionsResponse {
                subscribed_curators: Vec::new(),
            }),
        )
            .into_response();
    }
    match state.curator_runtime.subscribe(&req.curator_pubkey_hex) {
        Ok(_) => (
            StatusCode::OK,
            Json(SubscriptionsResponse {
                subscribed_curators: state.curator_runtime.subscribed_pubkeys_hex(),
            }),
        )
            .into_response(),
        Err(e) => runtime_error_to_response(e).into_response(),
    }
}

/// `DELETE /curators/{pubkey}` — remove a curator from the
/// attention set and evict any cached list they had published.
async fn unsubscribe_curator(
    State(state): State<Arc<DaemonHttpState>>,
    Path(pubkey): Path<String>,
) -> impl IntoResponse {
    debug!(curator = %pubkey, "DELETE /curators/{{pubkey}}");
    match state.curator_runtime.unsubscribe(&pubkey) {
        Ok(_) => (
            StatusCode::OK,
            Json(SubscriptionsResponse {
                subscribed_curators: state.curator_runtime.subscribed_pubkeys_hex(),
            }),
        )
            .into_response(),
        Err(e) => runtime_error_to_response(e).into_response(),
    }
}

/// `GET /browse` — Phase D reachability-annotated view of every
/// project across every cached curator list.
///
/// The aggregator flattens the Phase C curator runtime's list
/// snapshot, probes each referenced project endpoint (honouring
/// the TTL cache), and returns a sorted vector the React shell
/// renders as a Browse page. If the curator runtime is empty
/// (no subscribed curators, or no announcements received yet)
/// this returns `{"entries": []}` at 200 rather than an error —
/// the shell renders an empty-state card in that case.
async fn list_browse(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /browse");
    let entries = state
        .browse_aggregator
        .aggregate(&state.curator_runtime, &state.node)
        .await;
    (StatusCode::OK, Json(BrowseListResponse { entries }))
}

/// Sprint 20 Phase C : wrap an outbound gossip payload in a PoW
/// envelope. Solves (or reuses) a [`PowSolveCache`] entry for the
/// curator topic under the current live
/// [`RelayPowPolicy`], then concatenates `[u32 BE proof_len][proof
/// bytes][payload]` via [`PowEnvelope::encode`].
///
/// Returns an error if the policy clamps the topic's difficulty to
/// zero (misconfigured policy — loud failure rather than silent
/// bypass) or if the Hashcash solve times out (default 30 s, cf.
/// `SOLVE_TIMEOUT` in `pow_gossip`).
fn wrap_payload_with_pow(
    state: &DaemonHttpState,
    payload: &[u8],
) -> Result<Vec<u8>, nexus_core_rs::PowGossipError> {
    // Graceful degradation on a poisoned lock : recover the inner
    // policy rather than propagating a panic through the publish
    // handler. Mirrors the gossip receive loop
    // (`runtime.rs::spawn_gossip_subscribe_task`) so every reader
    // of `DaemonHttpState.pow_policy` survives a poisoning event.
    let policy = match state.pow_policy.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let proof = state.pow_solve_cache.ensure_proof(
        state.curator_gossip_topic,
        state.pow_keypair.as_ref(),
        &policy,
    )?;
    PowEnvelope::encode(&proof, payload)
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
async fn publish_project(
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

    let mut announcement = ProjectAnnouncement::new(
        state.node_id.clone(),
        req.project_name.clone(),
        req.category.clone(),
        req.description.clone(),
        req.apps.clone(),
    );

    // Sprint 13 Phase B: propagate repo_url.
    if let Some(ref url) = req.repo_url {
        announcement = announcement.with_repo_url(url.clone());
    }

    // Sprint 14 Phase B: propagate provenance_hash.
    if let Some(ref hash) = req.provenance_hash {
        announcement = announcement.with_provenance_hash(hash.clone());
    }

    // Sprint 16 Phase D: propagate is_open_source.
    if req.is_open_source {
        announcement = announcement.with_open_source(true);
    }

    // Sprint 12: if archive_hash is provided, mint a BlobTicket.
    if let Some(ref hash_hex) = req.archive_hash {
        match mint_blob_ticket(&state, hash_hex).await {
            Ok(ticket_str) => {
                announcement = announcement.with_archive_ticket(ticket_str);
            }
            Err(e) => {
                debug!(error = %e, "failed to mint BlobTicket for archive_hash");
                // Non-fatal: publish without archive_ticket.
            }
        }
    }

    // Broadcast via gossip if the sender is available.
    //
    // Sprint 20 Phase C wire : every outbound payload is wrapped in
    // a PoW envelope ([`PowEnvelope::encode`]) so receiver daemons
    // can drop unsolicited noise before processing. The proof is
    // minted (or reused, 15-min session window) by the shared
    // [`PowSolveCache`] driven by the live
    // [`RelayPowPolicy`] — a policy reload picks up on the very
    // next broadcast that misses the cache.
    let sender_guard = state.gossip_sender.read().await;
    if let Some(sender) = sender_guard.as_ref() {
        match announcement.to_gossip_bytes() {
            Ok(payload) => match wrap_payload_with_pow(&state, &payload) {
                Ok(envelope) => {
                    if let Err(e) = sender.broadcast(envelope).await {
                        debug!(error = %e, "gossip broadcast failed for project announcement");
                        // Non-fatal: the project is still added locally.
                    }
                }
                Err(e) => {
                    debug!(error = %e, "PoW envelope encode failed — skipping broadcast");
                }
            },
            Err(e) => {
                debug!(error = %e, "failed to serialize project announcement");
            }
        }
    } else {
        debug!("gossip sender not ready, skipping broadcast");
    }
    drop(sender_guard);

    // Add to the local browse aggregator so `/browse` includes
    // this project immediately without waiting for a gossip
    // round-trip.
    let browse_entry = BrowseEntry {
        project_id: state.node_id.clone(),
        project_name: req.project_name,
        category: req.category,
        description: req.description,
        curator_pubkey: String::new(),
        curator_name: "Self-published".into(),
        source: BrowseSource::Direct,
        status: BrowseStatus::Reachable,
        last_probed_at: None,
        archive_ticket: announcement.archive_ticket.clone(),
        archive_hash: req.archive_hash.clone(),
        repo_url: req.repo_url.clone(),
        provenance_hash: req.provenance_hash.clone(),
        is_open_source: req.is_open_source,
    };
    state.browse_aggregator.add_direct_entry(browse_entry);

    (StatusCode::OK, Json(PublishResponse { published: true })).into_response()
}

/// `GET /default-curators` — return the daemon's configured
/// default curator pubkeys from `[curator]` config section.
/// Sprint 11 Phase B.
async fn default_curators(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /default-curators");
    (
        StatusCode::OK,
        Json(DefaultCuratorsResponse {
            default_curators: state.default_curators.clone(),
        }),
    )
}

/// `POST /publish-blob` — store raw bytes as an iroh blob and
/// return the hex hash. Sprint 12 Phase A.
///
/// Called by the coordinator to upload a zip archive before
/// publishing. The coordinator then passes the hash to
/// `POST /publish` as `archive_hash`.
async fn publish_blob(State(state): State<Arc<DaemonHttpState>>, body: Bytes) -> impl IntoResponse {
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

/// `POST /panic/wipe` — Sprint 20 Phase B. Irreversibly destroy
/// the daemon's on-disk state (identity blobs + OS keyring
/// entries + subscriptions.json + blob cache) then exit the
/// process. Triggered by the shell's 5-tap `Ctrl+Shift+Alt+W`
/// gesture. The handler replies 200 BEFORE scheduling the exit
/// so the shell receives confirmation; the actual
/// `process::exit` runs from a spawned tokio task that sleeps
/// 100 ms to let axum flush the response.
async fn panic_wipe(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    warn!("POST /panic/wipe — executing irreversible wipe");
    let service = Arc::clone(&state.panic_wipe);
    match service.execute() {
        Ok(_) => {
            // Schedule the process exit on a background task so
            // the HTTP response can actually be written back.
            // `exit_only` skips re-running `execute` — the wipe
            // already happened synchronously above.
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                service.exit_only(0);
            });
            (StatusCode::OK, Json(serde_json::json!({ "wiped": true }))).into_response()
        }
        Err(e) => {
            warn!(error = %e, "panic wipe execute failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("panic wipe failed: {e}"),
                }),
            )
                .into_response()
        }
    }
}

/// `GET /blob-serve/{hash}/{*path}` — serve a file from a cached
/// zip archive with CSP headers. Sprint 12 Phase A.
///
/// If the archive is not in cache, attempts to load it from the
/// local blob store. If not in the local store either, returns 404.
async fn blob_serve(
    State(state): State<Arc<DaemonHttpState>>,
    Path((hash, path)): Path<(String, String)>,
) -> impl IntoResponse {
    // Strip leading slash from wildcard capture.
    let path = path.strip_prefix('/').unwrap_or(&path);
    // Default to index.html if path is empty.
    let path = if path.is_empty() { "index.html" } else { path };

    debug!(hash = %hash, path = %path, "GET /blob-serve");

    if !blob_serve::validate_zip_path(path) {
        return (StatusCode::BAD_REQUEST, "invalid path").into_response();
    }

    // Load into cache if not already present.
    if !state.blob_serve_cache.has(&hash) {
        let blobs = BlobsClient::new(state.node.blobs_store());
        let hash_bytes: [u8; 32] = match hex::decode(&hash).ok().and_then(|b| b.try_into().ok()) {
            Some(h) => h,
            None => return (StatusCode::BAD_REQUEST, "invalid hash hex").into_response(),
        };
        match blobs.get_bytes(hash_bytes).await {
            Ok(zip_bytes) => {
                if let Err(e) = state.blob_serve_cache.load(
                    &hash,
                    &zip_bytes,
                    blob_serve::DEFAULT_MAX_DECOMPRESSED_BYTES,
                ) {
                    warn!(error = %e, "failed to decompress zip blob");
                    return (StatusCode::BAD_REQUEST, format!("invalid archive: {e}"))
                        .into_response();
                }
            }
            Err(_) => {
                return (StatusCode::NOT_FOUND, "blob not found").into_response();
            }
        }
    }

    // Serve the file from cache.
    let file_bytes = match state.blob_serve_cache.get_file(&hash, path) {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, "file not found in archive").into_response(),
    };

    let content_type = blob_serve::detect_content_type(path, &file_bytes);

    // CSP + X-Content-Type-Options are injected by
    // blob_serve_csp_middleware on ALL responses (T37).
    (
        StatusCode::OK,
        [
            ("Content-Type", content_type),
            ("Cache-Control", "public, max-age=3600, immutable"),
        ],
        file_bytes,
    )
        .into_response()
}

/// Mint a BlobTicket from a hex hash in the local blob store.
async fn mint_blob_ticket(
    state: &DaemonHttpState,
    hash_hex: &str,
) -> Result<String, anyhow::Error> {
    use iroh_blobs::ticket::BlobTicket;
    use iroh_blobs::{BlobFormat, Hash};
    use nexus_core_rs::DiscoveryClient;

    let hash_bytes: [u8; 32] = hex::decode(hash_hex)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("hash must be 32 bytes"))?;

    // Verify the blob exists locally.
    let blobs = BlobsClient::new(state.node.blobs_store());
    if !blobs.has(hash_bytes).await? {
        anyhow::bail!("blob {hash_hex} not in local store");
    }

    let addr = DiscoveryClient::new(state.node.endpoint())
        .my_endpoint_addr()
        .await?;
    let ticket = BlobTicket::new(addr, Hash::from_bytes(hash_bytes), BlobFormat::Raw);
    Ok(ticket.to_string())
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::{create_node, KeyPair};
    use nexus_shell_daemon_core::blob_serve::BlobServeCache;
    use nexus_shell_daemon_core::browse::BrowseAggregator;
    use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
    use tower::ServiceExt;

    /// Sprint 16 Phase A: known-valid bearer token used by every
    /// test via [`build_test_router`]. 64-char lowercase hex,
    /// the shape
    /// [`nexus_shell_daemon_core::auth::load_or_generate_token`]
    /// would produce but fixed so assertions stay deterministic.
    const TEST_TOKEN: &str = "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef";

    /// Canonicalized `X-SBFB-Token` header name for the test-only
    /// layer below. Kept as a separate const because `HeaderMap`
    /// wants a `HeaderName`, not a `&str`.
    const AUTH_HEADER_NAME: axum::http::HeaderName =
        axum::http::HeaderName::from_static("x-sbfb-token");

    /// Build the production router plus an outer test-only layer
    /// that injects `X-SBFB-Token` and a loopback `Host` on every
    /// inbound request so the tests below can keep their one-liner
    /// `Request::builder().uri(..)` shape without re-attaching
    /// headers by hand 40+ times. Only the outermost layer is
    /// synthetic — every route still runs the real
    /// [`auth_required`] middleware, and the 401 / 403 paths are
    /// covered by `auth::tests` in the core crate.
    fn build_test_router(state: Arc<DaemonHttpState>) -> Router {
        use axum::http::header::{HOST, ORIGIN};
        use axum::http::HeaderValue;
        build_router(state, AuthState::new(TEST_TOKEN.to_string())).layer(middleware::from_fn(
            |mut req: axum::extract::Request, next: middleware::Next| async move {
                let h = req.headers_mut();
                if !h.contains_key(AUTH_HEADER_NAME) {
                    h.insert(AUTH_HEADER_NAME, HeaderValue::from_static(TEST_TOKEN));
                }
                if !h.contains_key(HOST) {
                    h.insert(HOST, HeaderValue::from_static("127.0.0.1:0"));
                }
                h.remove(ORIGIN);
                next.run(req).await
            },
        ))
    }

    /// Build a [`DaemonHttpState`] backed by a live iroh node.
    /// Every HTTP test spins up a fresh node because the
    /// browse route reaches through the Arc<Node> to probe
    /// endpoints. The `_node_guard` return keeps the node
    /// alive for the scope of the test; letting it drop
    /// calls the synchronous Drop path which is fine for
    /// unit tests.
    async fn mk_state() -> Arc<DaemonHttpState> {
        mk_state_with_mode(nexus_core_rs::IdentityMode::Normal).await
    }

    async fn mk_state_with_mode(mode: nexus_core_rs::IdentityMode) -> Arc<DaemonHttpState> {
        let node = create_node().await.expect("boot test node");
        let tmp = tempfile::tempdir().expect("tempdir");
        let keystore = Arc::new(nexus_core_rs::LocalFileKeyStore::new(tmp.path()));
        let panic_wipe = Arc::new(crate::panic::PanicWipeService::new(
            keystore,
            tmp.path().join("state.sqlite"),
            tmp.path().join("blob-cache"),
            Arc::new(crate::panic::RealExit) as Arc<dyn crate::panic::ExitStrategy>,
        ));
        // The tempdir is intentionally leaked so the panic service's
        // keystore can still reference its directory across the
        // state's lifetime. Unit tests never actually invoke the
        // panic service, so the directory stays untouched.
        std::mem::forget(tmp);
        Arc::new(DaemonHttpState {
            node_id: node.node_id(),
            daemon_version: "0.1.0-test".to_string(),
            boot_time: SystemTime::now(),
            api_host: "127.0.0.1".to_string(),
            api_port: 12345,
            curator_runtime: Arc::new(CuratorRuntime::new(None)),
            browse_aggregator: Arc::new(BrowseAggregator::new()),
            node: Arc::new(node),
            gossip_sender: Arc::new(RwLock::new(None)),
            default_curators: vec![],
            blob_serve_cache: Arc::new(BlobServeCache::new(8)),
            identity_mode: mode,
            panic_wipe,
            pow_solve_cache: Arc::new(PowSolveCache::new()),
            pow_policy: nexus_shell_daemon_core::pow_policy_loader::shared_default_policy(),
            pow_keypair: Arc::new(KeyPair::generate()),
            curator_gossip_topic: nexus_shell_daemon_core::iroh_runtime::curator_topic_id(),
        })
    }

    #[tokio::test]
    async fn health_returns_200_with_fixed_shape() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["daemon_version"], "0.1.0-test");
    }

    #[tokio::test]
    async fn info_returns_full_snapshot() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let snap: DaemonStateSnapshot = serde_json::from_slice(&body).unwrap();
        assert_eq!(snap.schema_version, 1);
        assert_eq!(snap.node_id.len(), 64);
        assert_eq!(snap.api_host, "127.0.0.1");
        assert_eq!(snap.api_port, 12345);
        assert!(snap.subscribed_curators.is_empty());
        assert_eq!(snap.known_lists, 0);
        assert_eq!(snap.known_browse_entries, 0);
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/nope")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_curators_returns_empty_when_nothing_cached() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let list: CuratorsListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.entries.is_empty());
        assert!(list.subscribed_curators.is_empty());
    }

    #[tokio::test]
    async fn subscribe_then_list_then_delete_happy_path() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let kp = KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        // POST /curators/subscribe
        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: hex_key.clone(),
        })
        .unwrap();
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let sub: SubscriptionsResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(sub.subscribed_curators, vec![hex_key.clone()]);

        // GET /curators must now show the pubkey in the set.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let list: CuratorsListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(list.subscribed_curators, vec![hex_key.clone()]);
        // No entries yet — no real gossip announcement received.
        assert!(list.entries.is_empty());

        // DELETE /curators/{pubkey}
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/curators/{hex_key}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let sub: SubscriptionsResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(sub.subscribed_curators.is_empty());
    }

    #[tokio::test]
    async fn subscribe_rejects_extra_fields() {
        // Sprint 8 audit G-3: a body that carries an extra
        // field on top of `curator_pubkey_hex` must be rejected
        // by axum's JSON extractor because the DTO now carries
        // `#[serde(deny_unknown_fields)]`. Axum surfaces the
        // serde rejection as HTTP 422 Unprocessable Entity.
        //
        // This is defense-in-depth: it catches the case where a
        // future shell extension starts sending a new field and
        // forgets to update the daemon side. Silent drop used
        // to mask those bugs for a full release cycle — the
        // tightened contract turns them into a first-commit
        // failure instead.
        let app = build_test_router(mk_state().await);
        let body: Vec<u8> = br#"{"curator_pubkey_hex":"aa","evil_field":"surprise"}"#.to_vec();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        // Axum maps `deny_unknown_fields` rejection to 422.
        // Accept either 400 or 422 in case a future axum version
        // changes the mapping — the important property is that
        // the request is refused outright, not silently ignored.
        assert!(
            resp.status() == StatusCode::UNPROCESSABLE_ENTITY
                || resp.status() == StatusCode::BAD_REQUEST,
            "expected 4xx from deny_unknown_fields, got {}",
            resp.status()
        );
    }

    #[tokio::test]
    async fn subscribe_rejects_bad_pubkey_hex_as_400() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: "not-hex".to_string(),
        })
        .unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn info_reflects_live_curator_runtime_counts() {
        let state = mk_state().await;
        let kp = KeyPair::generate();
        state
            .curator_runtime
            .subscribe(&hex::encode(kp.public_bytes()))
            .unwrap();

        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let snap: DaemonStateSnapshot =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(snap.subscribed_curators.len(), 1);
    }

    #[tokio::test]
    async fn browse_returns_empty_list_when_no_curators_cached() {
        // Phase D smoke test: with an empty curator runtime the
        // aggregator has nothing to flatten, so /browse returns
        // `{"entries": []}` at 200. The full Reachable/Unreachable
        // behaviour is covered by the 2-node integration tests
        // in `browse::tests::aggregate_probes_seeded_peer_*`.
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let list: BrowseListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.entries.is_empty());
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
                    .uri("/publish")
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
                    .uri("/browse")
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
                    .uri("/publish")
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
                    .uri("/publish")
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
                    .uri("/publish")
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
                    .uri("/publish")
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
                    .uri("/browse")
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

    // ---------------------------------------------------------
    // CORS helper unit tests (Phase A regression)
    // ---------------------------------------------------------

    #[test]
    fn loopback_origin_accepts_127_0_0_1_with_port() {
        let h = HeaderValue::from_static("http://127.0.0.1:3000");
        assert!(is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_accepts_localhost_without_port() {
        let h = HeaderValue::from_static("http://localhost");
        assert!(is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_remote_host() {
        let h = HeaderValue::from_static("http://example.com");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_https_scheme() {
        let h = HeaderValue::from_static("https://127.0.0.1");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_malformed_port() {
        let h = HeaderValue::from_static("http://127.0.0.1:not-a-port");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_suffix_trick() {
        let h = HeaderValue::from_static("http://127.0.0.1.evil.com");
        assert!(!is_loopback_origin(&h));
    }

    // ---------------------------------------------------------
    // Sprint 11 Phase B: default-curators endpoint
    // ---------------------------------------------------------

    #[tokio::test]
    async fn default_curators_returns_empty_when_unconfigured() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/default-curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let res: DefaultCuratorsResponse = serde_json::from_slice(&body).unwrap();
        assert!(res.default_curators.is_empty());
    }

    #[tokio::test]
    async fn default_curators_returns_configured_list() {
        let node = create_node().await.expect("boot test node");
        let curator_hex = "ab".repeat(32);
        let tmp = tempfile::tempdir().expect("tempdir");
        let keystore = Arc::new(nexus_core_rs::LocalFileKeyStore::new(tmp.path()));
        let panic_wipe = Arc::new(crate::panic::PanicWipeService::new(
            keystore,
            tmp.path().join("state.sqlite"),
            tmp.path().join("blob-cache"),
            Arc::new(crate::panic::RealExit) as Arc<dyn crate::panic::ExitStrategy>,
        ));
        std::mem::forget(tmp);
        let state = Arc::new(DaemonHttpState {
            node_id: node.node_id(),
            daemon_version: "0.1.0-test".to_string(),
            boot_time: SystemTime::now(),
            api_host: "127.0.0.1".to_string(),
            api_port: 12345,
            curator_runtime: Arc::new(CuratorRuntime::new(None)),
            browse_aggregator: Arc::new(BrowseAggregator::new()),
            node: Arc::new(node),
            gossip_sender: Arc::new(RwLock::new(None)),
            default_curators: vec![curator_hex.clone()],
            blob_serve_cache: Arc::new(BlobServeCache::new(8)),
            identity_mode: nexus_core_rs::IdentityMode::Normal,
            panic_wipe,
            pow_solve_cache: Arc::new(PowSolveCache::new()),
            pow_policy: nexus_shell_daemon_core::pow_policy_loader::shared_default_policy(),
            pow_keypair: Arc::new(KeyPair::generate()),
            curator_gossip_topic: nexus_shell_daemon_core::iroh_runtime::curator_topic_id(),
        });
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/default-curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let res: DefaultCuratorsResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(res.default_curators, vec![curator_hex]);
    }

    // ---------------------------------------------------------
    // Sprint 12 Phase A: blob-serve + publish-blob
    // ---------------------------------------------------------

    /// Helper: create a minimal zip archive in memory.
    fn make_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, data) in files {
            writer.start_file(*name, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
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
                    .uri("/publish-blob")
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
    async fn blob_serve_returns_file_from_cached_zip() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        // First, store a zip blob.
        let zip_bytes = make_zip(&[
            ("index.html", b"<h1>Hello SBFB</h1>"),
            ("assets/main.js", b"console.log('ok')"),
        ]);
        let blobs = BlobsClient::new(state.node.blobs_store());
        let hash = blobs.add_bytes(zip_bytes).await.unwrap();
        let hash_hex = hex::encode(hash);

        // GET /blob-serve/{hash}/index.html
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{hash_hex}/index.html"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Check CSP header.
        let csp = resp
            .headers()
            .get("Content-Security-Policy")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(csp.contains("connect-src 'none'"));

        // Check X-Content-Type-Options.
        assert_eq!(
            resp.headers()
                .get("X-Content-Type-Options")
                .unwrap()
                .to_str()
                .unwrap(),
            "nosniff"
        );

        // Check content.
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        assert_eq!(&body[..], b"<h1>Hello SBFB</h1>");

        // GET sub-resource.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{hash_hex}/assets/main.js"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("Content-Type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(ct.contains("javascript"));
    }

    #[tokio::test]
    async fn blob_serve_returns_404_for_unknown_hash() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{}/index.html", "ab".repeat(32)))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
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
                    .uri("/publish")
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
                    .uri("/browse")
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

    #[tokio::test]
    async fn blob_serve_rejects_path_traversal() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!(
                        "/blob-serve/{}/..%2F..%2Fetc%2Fpasswd",
                        "ab".repeat(32)
                    ))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Either 400 (path validation) or 404 (hash not found first).
        assert!(
            resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::NOT_FOUND,
            "expected 400 or 404, got {}",
            resp.status()
        );
    }

    /// Sprint 13 Phase A (T37): error responses from blob-serve
    /// must also carry CSP + X-Content-Type-Options headers, not
    /// just the 200 success path.
    #[tokio::test]
    async fn blob_serve_error_responses_have_csp() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{}/index.html", "ab".repeat(32)))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        // T37: CSP must be present on error responses too.
        assert!(
            resp.headers().get("content-security-policy").is_some(),
            "CSP header missing on 404 blob-serve response",
        );
        assert!(
            resp.headers().get("x-content-type-options").is_some(),
            "X-Content-Type-Options header missing on 404 blob-serve response",
        );
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
                    .uri("/publish")
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
                    .uri("/browse")
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

    /// #B-rt-2 The `/curators/subscribe` handler in Duress mode
    /// returns 200 but does NOT add the pubkey to the attention
    /// set. The peer observer sees an ACK; the local state is
    /// silently untouched.
    #[tokio::test]
    async fn daemon_boot_in_duress_mode_rejects_curator_subscribe_real() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let app = build_test_router(Arc::clone(&state));
        let curator_hex = "cd".repeat(32);

        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: curator_hex.clone(),
        })
        .unwrap();

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let sub_resp: SubscriptionsResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(
            sub_resp.subscribed_curators.is_empty(),
            "Duress mode must not grow the attention set"
        );

        // Verify the curator runtime itself stays empty.
        assert!(
            state.curator_runtime.subscribed_pubkeys_hex().is_empty(),
            "curator_runtime attention set must not mutate under Duress"
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
                    .uri("/publish-blob")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(b"fake blob bytes".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
