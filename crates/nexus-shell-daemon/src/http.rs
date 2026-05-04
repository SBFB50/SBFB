// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP surface for `nexus-shell-daemon`.
//!
//! The daemon's HTTP listener is loopback-only. Daemon-specific
//! JSON routes live under `/api/daemon/*` so they never collide
//! with SPA document routes (`/browse`, `/curators`) when the
//! daemon serves the React shell via `--web-root`.
//!
//! - `GET    /health`                         — liveness probe (public)
//! - `GET    /api/daemon/info`                — daemon state snapshot
//! - `GET    /api/daemon/curators`            — list every cached curator list
//! - `POST   /api/daemon/curators/subscribe`  — add a curator to the attention set
//! - `DELETE /api/daemon/curators/{pubkey}`   — remove a curator
//! - `GET    /api/daemon/browse`              — aggregated browse entries
//! - `POST   /api/daemon/publish`             — publish a project announcement
//! - `POST   /api/daemon/publish-blob`        — upload a zip archive blob
//! - `GET    /api/daemon/default-curators`    — config-provided curator list
//! - `POST   /api/daemon/panic/wipe`          — irreversible identity wipe
//! - `GET    /api/daemon/diagnostic/neighborhood` — peer snapshot
//!
//! ## CORS
//!
//! By default the daemon trusts only loopback origins:
//!
//! - `http://127.0.0.1[:port]`
//! - `http://localhost[:port]`
//!
//! The `--cors-origin` CLI flag (repeatable) extends the
//! allowlist with extra origins for multi-node access.
//! The env fallback `NEXUS_DAEMON_CORS_ORIGINS` (comma-
//! separated) is merged when the flag is absent.

use std::path::Path as FsPath;
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
use tower_http::services::{ServeDir, ServeFile};
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
    /// Sprint 36 Phase A : shared coordinator DB for the Rust-native
    /// task dispatcher and result validator. Opened once at boot from
    /// `~/.sbfb/coordinator.db` (WAL mode). Handlers lock briefly
    /// for each SQL operation (~1 ms).
    pub coordinator_db: std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    pub result_event_tx: crate::validator_loop::ResultEventSender,
    /// Sprint 39 Phase B : warrant canary observation registry (Rust
    /// port of canary_registry.py). Tracks observed canary signings
    /// and duress acks, computes per-pubkey freshness.
    pub canary_registry:
        std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::canary_registry::CanaryRegistry>>,
    pub canary_input:
        Option<std::sync::Arc<nexus_coordinator_rs::canary_input::CanaryInputManager>>,
    pub sbfb_home: Option<std::path::PathBuf>,
    /// Sprint 49 Phase A: handle to the project iroh-docs document.
    /// `Some` in coordinator mode (daemon start), `None` in tests
    /// that don't need doc wiring. Read by future endpoints and the
    /// doc subscription task.
    #[allow(dead_code)]
    pub project_doc: Option<std::sync::Arc<nexus_core_rs::docs::DocHandle>>,
    /// Sprint 49 Phase A: MPSC sender for the dispatch loop. HTTP
    /// task submit handler sends signed TaskEntry values here; the
    /// dispatch loop writes them to the project doc sequentially.
    pub task_dispatch_tx: Option<crate::dispatch_loop::TaskEntrySender>,
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
pub fn build_router(
    state: Arc<DaemonHttpState>,
    auth: AuthState,
    cors_origins: &[String],
    web_root: Option<&FsPath>,
) -> Router {
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

    // Public token bootstrap: the React shell served by this
    // daemon needs the bearer token to call authenticated routes.
    // Host + Origin loopback checks (inside handler) prevent
    // DNS rebinding and cross-origin leaks — same checks as
    // auth_required minus the bearer token itself.
    let auth_for_token = auth.clone();
    let token_route = Router::new()
        .route("/auth/token", get(auth_token_public))
        .with_state(auth_for_token);

    // Sprint 18 audit fix D-1 : the caller picks the variant
    // (`AuthState::Static` for the legacy single-token boot path,
    // `AuthState::Rotated` once the launcher writes a `tokens.json`).
    // The middleware reads the inner state on every request so a
    // rotation reaches `auth_required` without rebuilding the router.

    // Authenticated surface: every other route requires
    // X-SBFB-Token + loopback Host + (absent or loopback) Origin.
    let authed_routes = Router::new()
        .route("/api/daemon/info", get(info))
        .route("/api/daemon/curators", get(list_curators))
        .route("/api/daemon/curators/subscribe", post(subscribe_curator))
        .route("/api/daemon/curators/{pubkey}", delete(unsubscribe_curator))
        .route("/api/daemon/browse", get(list_browse))
        .route("/api/daemon/publish", post(publish_project))
        .route("/api/daemon/publish-blob", post(publish_blob))
        .route("/api/daemon/default-curators", get(default_curators))
        .route("/api/daemon/panic/wipe", post(panic_wipe))
        .route(
            "/api/v1/contributor/verify/{project_id}/{node_id_hex}",
            get(crate::contributor_api::verify_contributor),
        )
        .route(
            "/api/v1/contributor/project/{project_id}",
            get(crate::contributor_api::list_contributors),
        )
        .route(
            "/api/v1/contributor/envelope/{project_id}/{node_id_hex}",
            get(crate::contributor_api::envelope),
        )
        // Sprint 23 Phase E : diagnostic neighborhood snapshot.
        // Returns the node's own ID and known peer IDs from the
        // iroh endpoint's remote info table. Diagnostic-only, no
        // wire format impact.
        .route(
            "/api/daemon/diagnostic/neighborhood",
            get(diagnostic_neighborhood),
        )
        // Sprint 30 Phase C : FROST DKG + ceremony admin endpoints.
        // Trust tier T0 — behind the same loopback bearer + Host +
        // Origin gate as every other authenticated route.
        .route(
            "/api/canary/frost/trusted-dealer",
            post(frost_trusted_dealer),
        )
        .route("/api/canary/frost/round1", post(frost_round1))
        .route("/api/canary/frost/round2", post(frost_round2))
        .route("/api/canary/frost/aggregate", post(frost_aggregate))
        .route("/api/v1/tasks/submit", post(coordinator_submit_task))
        .route("/api/v1/results/submit", post(coordinator_submit_result))
        .route("/api/v1/kudos/{project_id}", get(coordinator_get_kudos))
        .route(
            "/api/v1/kudos/{project_id}/verify",
            get(coordinator_verify_chain),
        )
        .route("/api/canary/observed", post(canary_observed))
        .route("/api/canary/network-health", get(canary_network_health))
        .route("/api/canary/freshness/{pubkey}", get(canary_freshness))
        .route(
            "/api/canary/inject-rate",
            post(crate::canary_api::set_inject_rate),
        )
        .route(
            "/api/canary/observed-divergence",
            get(crate::canary_api::observed_divergence),
        )
        .route("/api/v1/apps", get(crate::apps::list_apps))
        .route("/api/v1/apps/{project_id}", get(crate::apps::get_app))
        .route("/api/v1/deploy", post(crate::deploy::deploy_private))
        .route(
            "/api/v1/deploy-from-repo",
            post(crate::deploy::deploy_from_repo),
        )
        .route("/api/v1/consent", get(crate::consent::get_consent))
        .route("/api/v1/consent/set", post(crate::consent::set_consent))
        .route(
            "/api/v1/consent/whitelist/add",
            post(crate::consent::whitelist_add),
        )
        .route(
            "/api/v1/consent/whitelist/remove",
            post(crate::consent::whitelist_remove),
        )
        .route("/api/v1/files/upload", post(crate::files::upload_file))
        .route(
            "/api/v1/files/{sha256}/manifest",
            get(crate::files::get_manifest),
        )
        .route("/api/v1/files/{sha256}", get(crate::files::stream_file))
        // Sprint 44 Phase B : health + shell + kudos + diagnostic
        .route(
            "/api/v1/coordinator/health",
            get(crate::health_api::coordinator_health),
        )
        .route("/api/v1/shell/discover", get(crate::shell_api::discover))
        .route("/api/v1/kudos/entries", get(crate::kudos_api::list_entries))
        .route(
            "/api/v1/kudos/{project_id}/leaderboard",
            get(crate::kudos_api::leaderboard),
        )
        .route(
            "/api/v1/diagnostic/fairness",
            get(crate::diagnostic_api::fairness_metrics),
        )
        // Sprint 44 Phase C : tasks + worker_state
        .route("/api/v1/tasks", get(crate::tasks_api::list_tasks))
        .route("/api/v1/tasks/{task_id}", get(crate::tasks_api::get_task))
        .route(
            "/api/v1/worker/state",
            get(crate::worker_state_api::get_worker_state),
        )
        // Sprint 45 Phase A : invite + quarantine
        .route(
            "/api/v1/invite/create",
            post(crate::invite_api::create_invite),
        )
        .route("/api/v1/invite", get(crate::invite_api::list_invites))
        .route(
            "/api/v1/invite/{invite_id}",
            delete(crate::invite_api::revoke_invite),
        )
        .route(
            "/api/v1/quarantine",
            get(crate::quarantine_api::list_quarantine),
        )
        .route(
            "/api/v1/quarantine/{row_id}/flush",
            post(crate::quarantine_api::flush_quarantine),
        )
        .route(
            "/api/v1/quarantine/{row_id}/drop",
            post(crate::quarantine_api::drop_quarantine),
        )
        .layer(middleware::from_fn_with_state(auth, auth_required));

    let app = Router::new()
        .merge(public_routes)
        .merge(token_route)
        .merge(authed_routes)
        .with_state(state)
        .layer(cors_layer(cors_origins));

    if let Some(root) = web_root {
        let serve = ServeDir::new(root).fallback(ServeFile::new(root.join("index.html")));
        app.fallback_service(serve)
    } else {
        app
    }
}

/// Public endpoint returning the bearer token so the React shell
/// can bootstrap auth from the same origin. Protected by
/// Host + Origin loopback checks (no bearer required — that's
/// what we're handing out).
async fn auth_token_public(State(auth): State<AuthState>, req: Request) -> impl IntoResponse {
    let host_ok = req
        .headers()
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(nexus_shell_daemon_core::auth::is_loopback_host)
        .unwrap_or(false);
    if !host_ok {
        return (StatusCode::FORBIDDEN, "host not allowed").into_response();
    }
    if let Some(origin) = req.headers().get(axum::http::header::ORIGIN) {
        let ok = origin
            .to_str()
            .ok()
            .map(nexus_shell_daemon_core::auth::is_loopback_origin)
            .unwrap_or(false);
        if !ok {
            return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
        }
    }
    match auth.current_token() {
        Some(token) => Json(serde_json::json!({ "token": token })).into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, "token unavailable").into_response(),
    }
}

/// Middleware that injects security headers on every blob-serve
/// response, including error responses.
async fn blob_serve_csp_middleware(request: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "content-security-policy",
        blob_serve::BLOB_SERVE_CSP.parse().unwrap(),
    );
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert(
        "cross-origin-opener-policy",
        blob_serve::BLOB_SERVE_COOP.parse().unwrap(),
    );
    headers.insert(
        "cross-origin-embedder-policy",
        blob_serve::BLOB_SERVE_COEP.parse().unwrap(),
    );
    response
}

/// Build a CORS layer that always accepts loopback origins and
/// optionally accepts extra origins passed via `--cors-origin`.
fn cors_layer(extra_origins: &[String]) -> CorsLayer {
    if extra_origins.is_empty() {
        return CorsLayer::new().allow_origin(AllowOrigin::predicate(
            |origin: &HeaderValue, _request_parts: &_| is_loopback_origin(origin),
        ));
    }
    let allowed: Vec<HeaderValue> = extra_origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o.as_str()).ok())
        .collect();
    CorsLayer::new().allow_origin(AllowOrigin::predicate(
        move |origin: &HeaderValue, _request_parts: &_| {
            is_loopback_origin(origin) || allowed.iter().any(|a| a == origin)
        },
    ))
}

/// Return `true` iff `origin` looks like a valid HTTP(S) origin
/// (scheme + host + optional port, no path). Used to reject
/// obviously malformed `--cors-origin` values at boot.
pub fn is_valid_origin(s: &str) -> bool {
    let rest = match s.strip_prefix("http://") {
        Some(r) => r,
        None => match s.strip_prefix("https://") {
            Some(r) => r,
            None => return false,
        },
    };
    if rest.is_empty() || rest.contains('/') {
        return false;
    }
    let (host, port_opt) = match rest.rsplit_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (rest, None),
    };
    if host.is_empty() {
        return false;
    }
    if let Some(p) = port_opt {
        if p.parse::<u16>().is_err() {
            return false;
        }
    }
    true
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

/// Body of `GET /diagnostic/neighborhood`. Sprint 23 Phase E.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeighborhoodResponse {
    pub node_id: String,
    pub peers: Vec<String>,
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
pub(crate) fn wrap_payload_with_pow(
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
pub(crate) async fn mint_blob_ticket(
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

/// `GET /diagnostic/neighborhood` — Sprint 23 Phase E. Returns the
/// node's own ID and the peer pubkeys currently in the daemon's
/// observable neighborhood. iroh 0.98 does not expose a DHT routing
/// table enumeration (`remote_info_iter` landed post-0.98), so the
/// observable neighborhood is the set of subscribed curator pubkeys
/// — the peers this daemon actively tracks via gossip. Post-0.98
/// upgrade or pkarr canary integration (S24) will enrich this with
/// transport-layer peer discovery.
async fn diagnostic_neighborhood(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /diagnostic/neighborhood");
    let peers = state.curator_runtime.subscribed_pubkeys_hex();
    (
        StatusCode::OK,
        Json(NeighborhoodResponse {
            node_id: state.node_id.clone(),
            peers,
        }),
    )
}

// =================================================================
// FROST DKG + ceremony endpoints (Sprint 30 Phase C)
// =================================================================

#[derive(Debug, Deserialize)]
struct FrostTrustedDealerRequest {
    k: u16,
    n: u16,
}

#[derive(Debug, Serialize)]
struct FrostTrustedDealerResponse {
    shares: Vec<nexus_shell_daemon_core::canary::DkgShareFile>,
    pubkey_package: nexus_shell_daemon_core::canary::DkgPubkeyFile,
}

async fn frost_trusted_dealer(Json(body): Json<FrostTrustedDealerRequest>) -> impl IntoResponse {
    match nexus_shell_daemon_core::canary::generate_dkg(body.k, body.n) {
        Ok((shares, pubkey_package)) => (
            StatusCode::OK,
            Json(serde_json::json!(FrostTrustedDealerResponse {
                shares,
                pubkey_package
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct FrostRound1Request {
    participant: u16,
    key_package_hex: String,
}

#[derive(Debug, Serialize)]
struct FrostRound1Response {
    commitment: nexus_shell_daemon_core::canary::CeremonyCommitment,
    nonces: nexus_shell_daemon_core::canary::CeremonyNonces,
}

async fn frost_round1(Json(body): Json<FrostRound1Request>) -> impl IntoResponse {
    let share_file = nexus_shell_daemon_core::canary::DkgShareFile {
        participant: body.participant,
        key_package_hex: body.key_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let frost_share = match nexus_shell_daemon_core::canary::load_share(&share_file) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_round1(
        body.participant,
        &frost_share.key_package,
    ) {
        Ok((commitment, nonces)) => (
            StatusCode::OK,
            Json(serde_json::json!(FrostRound1Response {
                commitment,
                nonces
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct FrostRound2Request {
    nonces: nexus_shell_daemon_core::canary::CeremonyNonces,
    signing_package: nexus_shell_daemon_core::canary::CeremonySigningPackage,
    key_package_hex: String,
    participant: u16,
}

async fn frost_round2(Json(body): Json<FrostRound2Request>) -> impl IntoResponse {
    let share_file = nexus_shell_daemon_core::canary::DkgShareFile {
        participant: body.participant,
        key_package_hex: body.key_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let frost_share = match nexus_shell_daemon_core::canary::load_share(&share_file) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_round2(
        &body.nonces,
        &body.signing_package,
        &frost_share.key_package,
    ) {
        Ok(sig_share) => (StatusCode::OK, Json(serde_json::json!(sig_share))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct FrostAggregateRequest {
    signing_package: nexus_shell_daemon_core::canary::CeremonySigningPackage,
    shares: Vec<nexus_shell_daemon_core::canary::CeremonySignatureShare>,
    pubkey_package_hex: String,
}

async fn frost_aggregate(Json(body): Json<FrostAggregateRequest>) -> impl IntoResponse {
    let pubkey_file = nexus_shell_daemon_core::canary::DkgPubkeyFile {
        verifying_key_hex: String::new(),
        pubkey_package_hex: body.pubkey_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let pubkey = match nexus_shell_daemon_core::canary::load_pubkey(&pubkey_file) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_aggregate(
        &body.signing_package,
        &body.shares,
        pubkey.package(),
    ) {
        Ok(sig) => (
            StatusCode::OK,
            Json(serde_json::json!({ "signature_hex": hex::encode(sig) })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// =================================================================
// Sprint 35 Phase B — Coordinator Rust-native task submission
// =================================================================

async fn coordinator_submit_task(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(submission): axum::Json<nexus_coordinator_rs::types::TaskSubmission>,
) -> impl IntoResponse {
    let input_ctx = nexus_coordinator_rs::guardrails::GuardrailContext {
        system_prompt: &submission.system_prompt,
        user_prompt: &submission.prompt,
        model_output: "",
    };
    let input_check = nexus_coordinator_rs::guardrails::default_input_chain().run(&input_ctx);
    if !input_check.passed {
        let reason = input_check
            .tripwire
            .unwrap_or_else(|| "input_guardrail_rejected".into());
        tracing::warn!(
            project_id = %submission.project_id,
            %reason,
            "task rejected by input guardrail"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "input_rejected", "reason": reason})),
        )
            .into_response();
    }

    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    let keypair = (*state.pow_keypair).clone();
    match nexus_coordinator_rs::dispatcher::submit_task(&db, &keypair, submission) {
        Ok(entry) => {
            if let Some(ref tx) = state.task_dispatch_tx {
                if let Err(e) = tx.try_send(entry.clone()) {
                    tracing::warn!("dispatch channel full or closed: {e}");
                }
            }
            match serde_json::to_value(&entry) {
                Ok(body) => (StatusCode::OK, Json(body)).into_response(),
                Err(e) => {
                    tracing::error!("task entry serialization failed: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "internal"})),
                    )
                        .into_response()
                }
            }
        }
        Err(nexus_coordinator_rs::error::CoordinatorError::Validation(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("task submit failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 36 Phase B — Coordinator Rust-native result submission
// =================================================================

async fn coordinator_submit_result(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(entry): axum::Json<nexus_core_rs::task::ResultEntry>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match nexus_coordinator_rs::validator::validate_result(&db, &entry) {
        Ok((nexus_coordinator_rs::validator::ValidationOutcome::Accepted, Some(task_record))) => {
            let guardrail_ctx = nexus_coordinator_rs::guardrails::GuardrailContext {
                system_prompt: "",
                user_prompt: "",
                model_output: &entry.payload.result_text,
            };
            let gr = nexus_coordinator_rs::guardrails::default_output_chain().run(&guardrail_ctx);
            if !gr.passed {
                let reason = gr.tripwire.unwrap_or_else(|| "guardrail_rejected".into());
                tracing::warn!(
                    task_id = %entry.payload.task_id,
                    %reason,
                    "result rejected by output guardrail — no kudos credited"
                );
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        serde_json::json!({"outcome": "rejected", "reason": "guardrail_rejected"}),
                    ),
                )
                    .into_response();
            }
            let worker_id = hex::encode(entry.worker_pubkey);
            if let Err(e) = nexus_coordinator_rs::kudos_ledger::credit(
                &db,
                &task_record.project_id,
                &worker_id,
                &entry.payload.task_id,
                entry.payload.tokens_generated,
            ) {
                tracing::warn!("kudos credit failed (non-fatal): {e}");
            }
            let _ = state
                .result_event_tx
                .send(crate::validator_loop::ResultEvent::NewResult(entry));
            (
                StatusCode::OK,
                Json(serde_json::json!({"outcome": "accepted"})),
            )
                .into_response()
        }
        Ok((outcome, _)) => {
            let reason = match outcome {
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedBadSignature => {
                    "bad_signature"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedTaskNotFound => {
                    "task_not_found"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedTaskNotPending => {
                    "task_not_pending"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::Accepted => unreachable!(),
            };
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"outcome": "rejected", "reason": reason})),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("result validation failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 36 Phase C — Kudos read endpoint
// =================================================================

async fn coordinator_get_kudos(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match nexus_coordinator_rs::kudos_ledger::get_project_kudos(&db, &project_id) {
        Ok(kudos) => match serde_json::to_value(&kudos) {
            Ok(body) => (StatusCode::OK, Json(body)).into_response(),
            Err(e) => {
                tracing::error!("kudos serialization failed: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response()
            }
        },
        Err(e) => {
            tracing::error!("kudos query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 38 Phase A — verify_chain endpoint
// =================================================================

async fn coordinator_verify_chain(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match nexus_coordinator_rs::kudos_ledger::verify_chain(&db, &project_id) {
        Ok(valid) => (StatusCode::OK, Json(serde_json::json!({"valid": valid}))).into_response(),
        Err(e) => {
            tracing::error!("verify_chain failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 39 Phase C — Canary registry HTTP endpoints
// =================================================================

async fn canary_observed(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let obs = match nexus_coordinator_rs::canary_registry::coerce_canary_payload(&payload) {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            )
                .into_response();
        }
    };
    match state.canary_registry.lock() {
        Ok(mut reg) => {
            reg.observe_canary(obs);
            (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

async fn canary_network_health(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    match state.canary_registry.lock() {
        Ok(reg) => {
            let health = reg.network_health();
            (
                StatusCode::OK,
                Json(serde_json::to_value(&health).unwrap_or_default()),
            )
                .into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

async fn canary_freshness(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(pubkey): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.canary_registry.lock() {
        Ok(reg) => {
            let freshness = reg.freshness(&pubkey);
            (
                StatusCode::OK,
                Json(serde_json::to_value(&freshness).unwrap_or_default()),
            )
                .into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
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
        build_test_router_with_cors(state, &[])
    }

    fn build_test_router_with_cors(state: Arc<DaemonHttpState>, cors: &[String]) -> Router {
        use axum::http::header::{HOST, ORIGIN};
        use axum::http::HeaderValue;
        build_router(state, AuthState::new(TEST_TOKEN.to_string()), cors, None).layer(
            middleware::from_fn(
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
            ),
        )
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

    async fn mk_state_with_sbfb_home(home: std::path::PathBuf) -> Arc<DaemonHttpState> {
        let mut state = (*mk_state().await).clone();
        state.sbfb_home = Some(home);
        Arc::new(state)
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
            coordinator_db: std::sync::Arc::new(std::sync::Mutex::new(
                nexus_coordinator_rs::db::CoordinatorDb::open_in_memory()
                    .expect("test coordinator DB"),
            )),
            result_event_tx: tokio::sync::broadcast::channel(8).0,
            canary_registry: {
                let tmp = tempfile::tempdir().expect("canary tmp");
                std::sync::Arc::new(std::sync::Mutex::new(
                    nexus_coordinator_rs::canary_registry::CanaryRegistry::new(
                        tmp.keep().join("canary_registry.json"),
                    ),
                ))
            },
            canary_input: Some(std::sync::Arc::new(
                nexus_coordinator_rs::canary_input::CanaryInputManager::new(None, None, None),
            )),
            sbfb_home: None,
            project_doc: None,
            task_dispatch_tx: None,
        })
    }

    #[tokio::test]
    async fn contributor_verify_rejects_non_hex_path_params() {
        let app = build_test_router(mk_state().await);
        let bad_project = "NOT-HEX";
        let node_hex = "a".repeat(64);
        let uri = format!("/api/v1/contributor/verify/{bad_project}/{node_hex}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
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
                    .uri("/api/daemon/info")
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
                    .uri("/api/daemon/curators")
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
                    .uri("/api/daemon/curators/subscribe")
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
                    .uri("/api/daemon/curators")
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
                    .uri(format!("/api/daemon/curators/{hex_key}"))
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
                    .uri("/api/daemon/curators/subscribe")
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
                    .uri("/api/daemon/curators/subscribe")
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
                    .uri("/api/daemon/info")
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
                    .uri("/api/daemon/browse")
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
    // Sprint 33 Phase A: CORS layer with --cors-origin
    // ---------------------------------------------------------

    #[test]
    fn valid_origin_accepts_http_with_port() {
        assert!(is_valid_origin("http://192.168.1.10:8080"));
    }

    #[test]
    fn valid_origin_accepts_https_without_port() {
        assert!(is_valid_origin("https://example.com"));
    }

    #[test]
    fn valid_origin_rejects_no_scheme() {
        assert!(!is_valid_origin("192.168.1.10:8080"));
    }

    #[test]
    fn valid_origin_rejects_with_path() {
        assert!(!is_valid_origin("http://example.com/path"));
    }

    #[test]
    fn valid_origin_rejects_javascript_scheme() {
        assert!(!is_valid_origin("javascript:alert('xss')"));
    }

    #[test]
    fn valid_origin_rejects_data_scheme() {
        assert!(!is_valid_origin("data:text/html,<script>alert(1)</script>"));
    }

    #[test]
    fn valid_origin_rejects_file_scheme() {
        assert!(!is_valid_origin("file:///etc/passwd"));
    }

    fn build_cors_test_router(state: Arc<DaemonHttpState>, cors: &[String]) -> Router {
        build_router(state, AuthState::new(TEST_TOKEN.to_string()), cors, None)
    }

    #[tokio::test]
    async fn cors_loopback_default_allows_localhost() {
        let app = build_cors_test_router(mk_state().await, &[]);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "http://localhost:3000")
                    .header("access-control-request-method", "GET")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(acao.is_some(), "loopback origin must be allowed by default");
    }

    #[tokio::test]
    async fn cors_loopback_default_rejects_external() {
        let app = build_cors_test_router(mk_state().await, &[]);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "http://192.168.1.10:8080")
                    .header("access-control-request-method", "GET")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(
            acao.is_none(),
            "external origin must be rejected by default"
        );
    }

    #[tokio::test]
    async fn cors_custom_origin_allows_configured() {
        let origins = vec!["http://192.168.1.10:8080".to_string()];
        let app = build_cors_test_router(mk_state().await, &origins);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "http://192.168.1.10:8080")
                    .header("access-control-request-method", "GET")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(acao.is_some(), "configured origin must be allowed");
    }

    #[tokio::test]
    async fn cors_custom_origin_preserves_loopback() {
        let origins = vec!["http://192.168.1.10:8080".to_string()];
        let app = build_cors_test_router(mk_state().await, &origins);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "http://localhost:3000")
                    .header("access-control-request-method", "GET")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(
            acao.is_some(),
            "loopback must still be allowed with custom origins"
        );
    }

    #[tokio::test]
    async fn cors_rejects_unconfigured_external() {
        let origins = vec!["http://192.168.1.10:8080".to_string()];
        let app = build_cors_test_router(mk_state().await, &origins);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "http://evil.com:9999")
                    .header("access-control-request-method", "GET")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(
            acao.is_none(),
            "unconfigured external origin must be rejected"
        );
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
                    .uri("/api/daemon/default-curators")
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
            coordinator_db: std::sync::Arc::new(std::sync::Mutex::new(
                nexus_coordinator_rs::db::CoordinatorDb::open_in_memory()
                    .expect("test coordinator DB"),
            )),
            result_event_tx: tokio::sync::broadcast::channel(8).0,
            canary_registry: {
                let tmp = tempfile::tempdir().expect("canary tmp");
                std::sync::Arc::new(std::sync::Mutex::new(
                    nexus_coordinator_rs::canary_registry::CanaryRegistry::new(
                        tmp.keep().join("canary_registry.json"),
                    ),
                ))
            },
            canary_input: Some(std::sync::Arc::new(
                nexus_coordinator_rs::canary_input::CanaryInputManager::new(None, None, None),
            )),
            sbfb_home: None,
            project_doc: None,
            task_dispatch_tx: None,
        });
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/default-curators")
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

        // Check COOP/COEP isolation headers.
        assert_eq!(
            resp.headers()
                .get("Cross-Origin-Opener-Policy")
                .unwrap()
                .to_str()
                .unwrap(),
            "same-origin"
        );
        assert_eq!(
            resp.headers()
                .get("Cross-Origin-Embedder-Policy")
                .unwrap()
                .to_str()
                .unwrap(),
            "require-corp"
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
        assert!(
            resp.headers().get("cross-origin-opener-policy").is_some(),
            "COOP header missing on 404 blob-serve response",
        );
        assert!(
            resp.headers().get("cross-origin-embedder-policy").is_some(),
            "COEP header missing on 404 blob-serve response",
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
                    .uri("/api/daemon/curators/subscribe")
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
                    .uri("/api/daemon/publish-blob")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(b"fake blob bytes".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    // ---------------------------------------------------------
    // Sprint 23 Phase E: diagnostic neighborhood endpoint
    // ---------------------------------------------------------

    #[tokio::test]
    async fn diagnostic_neighborhood_returns_own_node_id_and_empty_peers() {
        let state = mk_state().await;
        let expected_node_id = state.node_id.clone();
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/diagnostic/neighborhood")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let neighborhood: NeighborhoodResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(neighborhood.node_id, expected_node_id);
        assert!(
            neighborhood.peers.is_empty(),
            "fresh node should have no known peers"
        );
    }

    // ---------------------------------------------------------
    // Sprint 53 Phase A: SPA route collision regression tests
    // ---------------------------------------------------------

    fn build_test_router_with_web_root(state: Arc<DaemonHttpState>) -> (Router, tempfile::TempDir) {
        use axum::http::header::{HOST, ORIGIN};
        use std::fs;
        let tmp = tempfile::tempdir().expect("tempdir for web_root");
        fs::write(
            tmp.path().join("index.html"),
            "<!doctype html><div id=root></div>",
        )
        .unwrap();
        let router = build_router(
            state,
            AuthState::new(TEST_TOKEN.to_string()),
            &[],
            Some(tmp.path()),
        )
        .layer(middleware::from_fn(
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
        ));
        (router, tmp)
    }

    #[tokio::test]
    async fn spa_fallback_serves_browse_as_html_document() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
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
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("text/html"),
            "GET /browse with web_root must serve SPA HTML, got content-type: {ct}"
        );
    }

    #[tokio::test]
    async fn spa_fallback_serves_curators_as_html_document() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
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
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("text/html"),
            "GET /curators with web_root must serve SPA HTML, got content-type: {ct}"
        );
    }

    #[tokio::test]
    async fn api_daemon_browse_still_returns_json_with_web_root() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
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
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let list: BrowseListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.entries.is_empty());
    }

    #[tokio::test]
    async fn api_daemon_info_still_returns_json_with_web_root() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let snap: DaemonStateSnapshot = serde_json::from_slice(&body).unwrap();
        assert_eq!(snap.schema_version, 1);
    }

    // =============================================================
    // Sprint 31 Phase D — HTTP integration tests FROST endpoints
    // =============================================================

    #[tokio::test]
    async fn frost_http_trusted_dealer_returns_shares_and_pubkey() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let shares = json["shares"].as_array().expect("shares array");
        assert_eq!(shares.len(), 3, "K=2/N=3 must produce 3 shares");
        for (i, share) in shares.iter().enumerate() {
            assert_eq!(
                share["participant"].as_u64().unwrap(),
                (i + 1) as u64,
                "share participant must be 1-indexed"
            );
            assert!(
                !share["key_package_hex"].as_str().unwrap().is_empty(),
                "key_package_hex must be non-empty"
            );
        }
        let pubkey = &json["pubkey_package"];
        assert!(
            !pubkey["verifying_key_hex"].as_str().unwrap().is_empty(),
            "verifying_key_hex must be non-empty"
        );
        assert_eq!(
            pubkey["verifying_key_hex"].as_str().unwrap().len(),
            64,
            "verifying key must be 32 bytes (64 hex chars)"
        );
    }

    #[tokio::test]
    async fn frost_http_round1_returns_commitment_and_nonces() {
        let app_dealer = build_test_router(mk_state().await);
        let dealer_resp = app_dealer
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dealer_body = to_bytes(dealer_resp.into_body(), 16384).await.unwrap();
        let dealer_json: serde_json::Value = serde_json::from_slice(&dealer_body).unwrap();
        let share = &dealer_json["shares"][0];
        let key_package_hex = share["key_package_hex"].as_str().unwrap();
        let participant = share["participant"].as_u64().unwrap() as u16;

        let round1_body = serde_json::json!({
            "participant": participant,
            "key_package_hex": key_package_hex
        });

        let app_round1 = build_test_router(mk_state().await);
        let resp = app_round1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round1")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(round1_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            !json["commitment"]["commitment_hex"]
                .as_str()
                .unwrap()
                .is_empty(),
            "commitment_hex must be non-empty"
        );
        assert!(
            !json["nonces"]["nonces_hex"].as_str().unwrap().is_empty(),
            "nonces_hex must be non-empty"
        );
        assert_eq!(
            json["commitment"]["participant"].as_u64().unwrap(),
            participant as u64
        );
    }

    #[tokio::test]
    async fn frost_http_round2_returns_signature_share() {
        let state = mk_state().await;

        let app1 = build_test_router(Arc::clone(&state));
        let dealer_resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let mut commitments = Vec::new();
        let mut nonces_list = Vec::new();
        for i in 0..2 {
            let share = &dj["shares"][i];
            let r1_body = serde_json::json!({
                "participant": share["participant"],
                "key_package_hex": share["key_package_hex"]
            });
            let app = build_test_router(Arc::clone(&state));
            let r1_resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round1")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r1_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(r1_resp.status(), StatusCode::OK);
            let r1j: serde_json::Value =
                serde_json::from_slice(&to_bytes(r1_resp.into_body(), 16384).await.unwrap())
                    .unwrap();
            commitments.push(r1j["commitment"].clone());
            nonces_list.push(r1j["nonces"].clone());
        }

        let sp = nexus_shell_daemon_core::canary::build_signing_package(
            &commitments
                .iter()
                .map(|c| serde_json::from_value(c.clone()).unwrap())
                .collect::<Vec<nexus_shell_daemon_core::canary::CeremonyCommitment>>(),
            b"round2 HTTP test message",
        )
        .expect("build signing package");

        let r2_body = serde_json::json!({
            "nonces": nonces_list[0],
            "signing_package": sp,
            "key_package_hex": dj["shares"][0]["key_package_hex"],
            "participant": dj["shares"][0]["participant"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round2")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r2_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(
            !json["signature_share_hex"].as_str().unwrap().is_empty(),
            "signature_share_hex must be non-empty"
        );
    }

    #[tokio::test]
    async fn frost_http_aggregate_returns_valid_signature() {
        let state = mk_state().await;

        let app1 = build_test_router(Arc::clone(&state));
        let dealer_resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let mut commitments = Vec::new();
        let mut nonces_list = Vec::new();
        for i in 0..2 {
            let share = &dj["shares"][i];
            let r1_body = serde_json::json!({
                "participant": share["participant"],
                "key_package_hex": share["key_package_hex"]
            });
            let app = build_test_router(Arc::clone(&state));
            let r1_resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round1")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r1_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            let r1j: serde_json::Value =
                serde_json::from_slice(&to_bytes(r1_resp.into_body(), 16384).await.unwrap())
                    .unwrap();
            commitments.push(r1j["commitment"].clone());
            nonces_list.push(r1j["nonces"].clone());
        }

        let message = b"aggregate HTTP test message";
        let ceremony_commitments: Vec<nexus_shell_daemon_core::canary::CeremonyCommitment> =
            commitments
                .iter()
                .map(|c| serde_json::from_value(c.clone()).unwrap())
                .collect();
        let sp =
            nexus_shell_daemon_core::canary::build_signing_package(&ceremony_commitments, message)
                .expect("build signing package");

        let mut sig_shares = Vec::new();
        for i in 0..2 {
            let r2_body = serde_json::json!({
                "nonces": nonces_list[i],
                "signing_package": sp,
                "key_package_hex": dj["shares"][i]["key_package_hex"],
                "participant": dj["shares"][i]["participant"]
            });
            let app = build_test_router(Arc::clone(&state));
            let resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round2")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r2_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let rj: serde_json::Value =
                serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
            sig_shares.push(rj);
        }

        let agg_body = serde_json::json!({
            "signing_package": sp,
            "shares": sig_shares,
            "pubkey_package_hex": dj["pubkey_package"]["pubkey_package_hex"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/aggregate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(agg_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        let sig_hex = json["signature_hex"]
            .as_str()
            .expect("signature_hex present");
        assert_eq!(sig_hex.len(), 128, "64-byte Ed25519 sig = 128 hex chars");

        let sig_bytes = hex::decode(sig_hex).expect("valid hex");
        let vk_hex = dj["pubkey_package"]["verifying_key_hex"].as_str().unwrap();
        let vk_bytes = hex::decode(vk_hex).expect("valid vk hex");
        let vk: [u8; 32] = vk_bytes.try_into().expect("32 bytes");
        let sig: [u8; 64] = sig_bytes.try_into().expect("64 bytes");
        nexus_core_rs::crypto::verify(&vk, message, &sig)
            .expect("aggregated FROST sig must verify as Ed25519");
    }

    #[tokio::test]
    async fn frost_http_invalid_threshold_k_gt_n() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":5,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn frost_http_malformed_json_body() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k": not valid json"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            resp.status().is_client_error(),
            "malformed JSON should return 4xx"
        );
    }

    #[tokio::test]
    async fn frost_http_round1_invalid_key_package() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round1")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"participant":1,"key_package_hex":"deadbeef"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn frost_http_aggregate_invalid_pubkey() {
        let state = mk_state().await;

        let app = build_test_router(Arc::clone(&state));
        let dealer_resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let agg_body = serde_json::json!({
            "signing_package": { "signing_package_hex": "deadbeef" },
            "shares": [],
            "pubkey_package_hex": dj["pubkey_package"]["pubkey_package_hex"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/aggregate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(agg_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }

    // ===============================================================
    // Sprint 36 Phase B — result submission integration tests
    // ===============================================================

    fn make_test_submission() -> nexus_coordinator_rs::types::TaskSubmission {
        nexus_coordinator_rs::types::TaskSubmission {
            project_id: "test-project".into(),
            task_type: "analysis".into(),
            prompt: "Analyze this".into(),
            system_prompt: String::new(),
            model: "llama3".into(),
            priority: 5,
            parent_task_id: String::new(),
            metadata: std::collections::BTreeMap::new(),
            is_open_source: false,
            estimated_watts: 0,
            estimated_vram_mb: 0,
            estimated_hours: 0.0,
            redundancy_factor: 1,
        }
    }

    fn make_result_entry(task_id: &str, worker_kp: &KeyPair) -> nexus_core_rs::task::ResultEntry {
        let payload = nexus_core_rs::task::ResultPayload {
            version: nexus_core_rs::task::TASK_FORMAT_VERSION,
            task_id: task_id.to_string(),
            result_text: "result text".into(),
            tokens_generated: 42,
            generation_time_ms: 1000,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 100,
            finished_at: 200,
            output_token_ids: vec![],
        };
        nexus_core_rs::task::ResultEntry::sign(payload, worker_kp).expect("sign result")
    }

    #[tokio::test]
    async fn result_submit_accepts_valid() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["outcome"], "accepted");

        let db = state.coordinator_db.lock().unwrap();
        let task = db
            .get_task(&task_entry.task.task_id)
            .expect("get")
            .expect("found");
        assert_eq!(
            task.status,
            nexus_coordinator_rs::types::TaskStatus::Completed
        );
    }

    #[tokio::test]
    async fn result_submit_rejects_bad_signature() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let mut result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);
        result_entry.signature[0] ^= 0xff;

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "bad_signature");
    }

    #[tokio::test]
    async fn result_submit_rejects_unknown_task() {
        let state = mk_state().await;
        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry("nonexistent-task-id", &worker_kp);

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "task_not_found");
    }

    #[tokio::test]
    async fn result_submit_rejects_completed_task() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        {
            let db = state.coordinator_db.lock().unwrap();
            db.set_task_result(&task_entry.task.task_id, "w1", "r1", 100)
                .expect("complete");
        }

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "task_not_pending");
    }

    // ===============================================================
    // Sprint 36 Phase C — kudos integration tests
    // ===============================================================

    #[tokio::test]
    async fn e2e_task_result_kudos_credited() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let db = state.coordinator_db.lock().unwrap();
        let total = db
            .get_project_kudos_total("test-project")
            .expect("kudos total");
        assert!(total > 0, "kudos must be credited after accepted result");
    }

    #[tokio::test]
    async fn kudos_endpoint_returns_json() {
        let state = mk_state().await;

        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(
                &db,
                "proj-abc",
                "worker-xyz",
                "task-1",
                100,
            )
            .expect("credit");
        }

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-abc")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["project_id"], "proj-abc");
        assert_eq!(body["total"], 100);
        assert_eq!(body["contributors"][0]["worker_node_id"], "worker-xyz");
    }

    // =========================================================
    // Mutex poisoned tests (P2-REVIEW-A-1/B-1)
    // =========================================================

    #[tokio::test]
    async fn submit_task_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        // Poison the mutex by panicking while holding the guard.
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();
        assert!(
            state.coordinator_db.lock().is_err(),
            "mutex must be poisoned"
        );

        let app = build_test_router(state);
        let body = serde_json::json!({
            "project_id": "p1",
            "task_type": "inference",
            "prompt": "test",
            "system_prompt": "",
            "model": "llama3"
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/tasks/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn submit_result_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();

        let app = build_test_router(state);
        let kp = KeyPair::generate();
        let payload = nexus_core_rs::task::ResultPayload {
            version: nexus_core_rs::task::TASK_FORMAT_VERSION,
            task_id: "t-1".to_string(),
            result_text: "out".to_string(),
            tokens_generated: 1,
            generation_time_ms: 1,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 0,
            finished_at: 1,
            output_token_ids: vec![],
        };
        let entry = nexus_core_rs::task::ResultEntry::sign(payload, &kp).unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&entry).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn get_kudos_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn verify_chain_endpoint_returns_valid() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "proj-vc", "worker-a", "task-1", 10)
                .expect("credit");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-vc/verify")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["valid"], true);
    }

    #[tokio::test]
    async fn submit_task_pii_rejected() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let mut sub = make_test_submission();
        sub.prompt = "Contact me at test@example.com for details".into();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/submit")
                    .header("content-type", "application/json")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::from(serde_json::to_vec(&sub).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["error"], "input_rejected");
    }

    #[tokio::test]
    async fn canary_observed_post_ok() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let payload = serde_json::json!({
            "version": 1,
            "pubkey_hex": "aa".repeat(32),
            "date": "2026-04-29",
            "headline": "All clear",
            "next_update": "2026-05-29",
            "signature_hex": "bb".repeat(64)
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/canary/observed")
                    .header("content-type", "application/json")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&payload).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn canary_network_health_get_ok() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/canary/network-health")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // =============================================================
    // Sprint 46 Phase A — integration tests 12 MANDATORY routes
    // =============================================================

    // --- consent.rs (4 routes) ---

    #[tokio::test]
    async fn consent_get_returns_default_config() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/consent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["level"], 1);
    }

    #[tokio::test]
    async fn consent_set_invalid_level_400() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({"level": 0});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/set")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn consent_set_level_5_400() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({"level": 5});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/set")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn consent_whitelist_add_invalid_node_id_400() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({"project_id": "not-valid-hex"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/add")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn consent_whitelist_add_missing_project_id_422() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/add")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn consent_whitelist_remove_missing_project_id_422() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/remove")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // --- files.rs (3 routes) ---

    #[tokio::test]
    async fn files_manifest_invalid_sha_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/files/not-a-valid-sha/manifest")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn files_manifest_not_found_404() {
        let app = build_test_router(mk_state().await);
        let sha = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}/manifest"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn files_stream_invalid_sha_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/files/bad-sha")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn files_stream_not_found_404() {
        let app = build_test_router(mk_state().await);
        let sha = "b".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn files_upload_too_large_413() {
        let app = build_test_router(mk_state().await);
        let big_body = vec![0u8; 50 * 1024 * 1024 + 1];
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(big_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    // --- canary_api.rs (3 routes) ---

    #[tokio::test]
    async fn canary_freshness_returns_200() {
        let app = build_test_router(mk_state().await);
        let pubkey = "aa".repeat(32);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/canary/freshness/{pubkey}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn canary_freshness_unknown_pubkey_returns_200() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/canary/freshness/unknown-key")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn canary_inject_rate_updates() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let body = serde_json::json!({"inject_rate": 50});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/inject-rate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(resp_body["status"], "updated");
        assert!(resp_body["inject_rate"].as_u64().is_some());
    }

    #[tokio::test]
    async fn canary_observed_divergence_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/canary/observed-divergence")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["divergences"].as_array().unwrap().is_empty());
    }

    // --- contributor_api.rs (2 routes) ---

    #[tokio::test]
    async fn contributor_project_empty_list() {
        let app = build_test_router(mk_state().await);
        let project_id = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/contributor/project/{project_id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["contributors"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn contributor_project_invalid_hex_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/contributor/project/not-a-hex")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn contributor_envelope_not_found_404() {
        let app = build_test_router(mk_state().await);
        let project_id = "a".repeat(64);
        let node_id = "b".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!(
                        "/api/v1/contributor/envelope/{project_id}/{node_id}"
                    ))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn contributor_envelope_invalid_hex_400() {
        let app = build_test_router(mk_state().await);
        let valid = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/contributor/envelope/bad-hex/{valid}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // =============================================================
    // Sprint 46 Phase B — integration tests 14 recent routes + debt
    // =============================================================

    // --- invite_api.rs (3 routes) ---

    #[tokio::test]
    async fn invite_create_success() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/invite/create")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        let id = body["id"].as_str().expect("id must be a string");
        assert!(id.starts_with("inv-"), "invite ID must start with inv-");
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.len(), 4, "format inv-{{node8}}-{{ts}}-{{seq}}");
        assert_eq!(parts[1].len(), 8, "node_id prefix must be 8 hex chars");
        assert_eq!(body["scope"], "worker");
    }

    #[tokio::test]
    async fn invite_list_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/invite")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["invites"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn invite_revoke_not_found_404() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/v1/invite/nonexistent-id")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // --- quarantine_api.rs (3 routes) ---

    #[tokio::test]
    async fn quarantine_list_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/quarantine")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
    }

    #[tokio::test]
    async fn quarantine_flush_not_found() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/quarantine/99999/flush")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn quarantine_drop_not_found() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/quarantine/99999/drop")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
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

    // --- kudos_api.rs (2 routes) ---

    #[tokio::test]
    async fn kudos_entries_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["entries"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn kudos_leaderboard_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-test/leaderboard")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["leaderboard"].as_array().unwrap().is_empty());
    }

    // --- health_api.rs (1 route) ---

    #[tokio::test]
    async fn coordinator_health_ok() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/coordinator/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["status"], "ok");
        assert_eq!(body["daemon_version"], "0.1.0-test");
    }

    // --- shell_api.rs (1 route) ---

    #[tokio::test]
    async fn shell_discover_returns_self() {
        let state = mk_state().await;
        let expected_node_id = state.node_id.clone();
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/shell/discover")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 1);
        let coordinators = body["coordinators"].as_array().unwrap();
        assert_eq!(coordinators.len(), 1);
        assert_eq!(coordinators[0]["node_id"], expected_node_id);
    }

    // --- diagnostic_api.rs (1 route) ---

    #[tokio::test]
    async fn diagnostic_fairness_ok() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["worker_count"], 0);
    }

    // --- worker_state_api.rs (1 route) ---

    #[tokio::test]
    async fn worker_state_returns_200() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/worker/state")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(body.get("running").is_some());
    }

    // --- Pagination tests (debt items 2 + 4) ---

    #[tokio::test]
    async fn kudos_entries_with_limit_offset() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w1", "t1", 10).unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w2", "t2", 20).unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w3", "t3", 30).unwrap();
        }
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries?limit=2&offset=0")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 2);
        assert_eq!(body["total_count"], 3);

        let app2 = build_test_router(Arc::clone(&state));
        let resp2 = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries?limit=2&offset=2")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let body2: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp2.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body2["count"], 1);
        assert_eq!(body2["total_count"], 3);
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

    // --- Debt item 5: diagnostic error propagation ---

    #[tokio::test]
    async fn diagnostic_fairness_returns_500_on_corrupted_db() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            db.execute_batch_raw("DROP TABLE IF EXISTS kudos")
                .expect("drop kudos table");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(body["error"]
            .as_str()
            .unwrap()
            .contains("worker_contributions"));
    }

    #[tokio::test]
    async fn diagnostic_fairness_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();
        assert!(state.coordinator_db.lock().is_err());

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // --- deploy.rs integration tests (2 routes) ---

    fn make_test_zip() -> Vec<u8> {
        use std::io::Write;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            zw.start_file("index.html", opts).unwrap();
            zw.write_all(b"<html><body>test</body></html>").unwrap();
            zw.finish().unwrap();
        }
        buf.into_inner()
    }

    #[tokio::test]
    async fn deploy_private_valid_zip_returns_200() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let zip = make_test_zip();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/deploy")
                    .body(axum::body::Body::from(zip))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["deployed"], true);
        assert!(body["hash"].as_str().unwrap().len() >= 32);
    }

    #[tokio::test]
    async fn deploy_private_invalid_zip_returns_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/deploy")
                    .body(axum::body::Body::from(b"not a zip".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn deploy_from_repo_non_http_url_returns_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/deploy-from-repo")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({
                            "repo_url": "ssh://git@github.com/test/repo.git",
                            "project_name": "test"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn deploy_from_repo_invalid_sha_returns_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/deploy-from-repo")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({
                            "repo_url": "https://github.com/test/repo.git",
                            "project_name": "test",
                            "commit_sha": "not-a-sha"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // --- apps.rs integration tests (2 routes) ---

    #[tokio::test]
    async fn apps_list_empty_returns_200() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/apps")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
    }

    #[tokio::test]
    async fn apps_list_with_entries_returns_populated() {
        use nexus_shell_daemon_core::browse::{BrowseEntry, BrowseSource, BrowseStatus};
        let state = mk_state().await;
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: "a".repeat(64),
            project_name: "Test App".into(),
            category: "test".into(),
            description: "A test app".into(),
            curator_pubkey: "b".repeat(64),
            curator_name: "Test Curator".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: Some("c".repeat(64)),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/apps")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 1);
        assert_eq!(body["apps"][0]["project_name"], "Test App");
    }

    #[tokio::test]
    async fn apps_get_by_id_returns_detail() {
        use nexus_shell_daemon_core::browse::{BrowseEntry, BrowseSource, BrowseStatus};
        let state = mk_state().await;
        let pid = "d".repeat(64);
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: pid.clone(),
            project_name: "Detail App".into(),
            category: "test".into(),
            description: "Detailed".into(),
            curator_pubkey: "e".repeat(64),
            curator_name: "Curator".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: Some("https://example.com/repo".into()),
            provenance_hash: None,
            is_open_source: true,
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/apps/{pid}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["project_name"], "Detail App");
        assert_eq!(body["is_open_source"], true);
    }

    #[tokio::test]
    async fn apps_get_unknown_id_returns_404() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/apps/nonexistent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // --- auth/token integration test (1 route) ---

    #[tokio::test]
    async fn auth_token_returns_200_from_loopback() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/auth/token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["token"], TEST_TOKEN);
    }

    // --- consent.rs happy path tests (4 routes) ---

    #[tokio::test]
    async fn consent_set_level_2_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/set")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"level": 2}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["level"], 2);
    }

    #[tokio::test]
    async fn consent_get_returns_persisted_level() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        let app1 = build_test_router(Arc::clone(&state));
        let resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/set")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"level": 3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/consent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["level"], 3);
    }

    #[tokio::test]
    async fn consent_whitelist_add_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
        let pid = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/add")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(body["allowed_project_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some(&pid)));
    }

    #[tokio::test]
    async fn consent_whitelist_remove_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let pid = "b".repeat(64);

        let app1 = build_test_router(Arc::clone(&state));
        app1.oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/consent/whitelist/add")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({"project_id": pid}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/remove")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(body["allowed_project_ids"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v.as_str() != Some(&pid)));
    }

    // --- files.rs happy path tests (3 routes) ---

    #[tokio::test]
    async fn files_upload_small_returns_201_with_sha() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "text/plain")
                    .header("x-original-name", "test.txt")
                    .body(axum::body::Body::from(b"hello world".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["sha256"].as_str().unwrap().len(), 64);
        assert_eq!(body["size"], 11);
        assert_eq!(body["original_name"], "test.txt");
    }

    #[tokio::test]
    async fn files_manifest_after_upload_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        let app1 = build_test_router(Arc::clone(&state));
        let resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "text/plain")
                    .body(axum::body::Body::from(b"manifest test".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let upload_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        let sha = upload_body["sha256"].as_str().unwrap();

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}/manifest"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["sha256"].as_str().unwrap(), sha);
    }

    #[tokio::test]
    async fn files_stream_after_upload_returns_content() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let content = b"stream test content";

        let app1 = build_test_router(Arc::clone(&state));
        let resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .body(axum::body::Body::from(content.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let upload_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        let sha = upload_body["sha256"].as_str().unwrap().to_owned();

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
        assert_eq!(&body_bytes[..], content);
    }
}
