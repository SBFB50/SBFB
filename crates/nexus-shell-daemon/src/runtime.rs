// SPDX-License-Identifier: AGPL-3.0-or-later
//! Boot, HTTP serve, gossip subscribe, graceful shutdown.
//!
//! [`DaemonRuntime::start`] is the single entry point the binary
//! uses to bring up the daemon. It performs the following ordered
//! sequence (Phase A + Phase C):
//!
//! 1. [`nexus_shell_daemon_core::registry::check_stale_or_bail`]
//!    — refuse to boot if a live daemon is already running.
//! 2. `nexus_core_rs::create_node()` — spin up the iroh endpoint +
//!    protocol router + blobs/docs/gossip stack. Wrapped in
//!    [`Arc`] so the gossip subscribe task can hold a second
//!    reference for its whole lifetime.
//! 3. Bind a TCP listener on `(api_host, 0)` so the OS picks an
//!    ephemeral port.
//! 4. Write `running.json` (Phase A singleton marker).
//! 5. Construct [`CuratorRuntime::with_persistence`] over
//!    `<root>/shell-daemon/subscriptions.json` so the attention
//!    set survives daemon restarts (R7 mitigation).
//! 6. Build the shared HTTP state (carries the `Arc<CuratorRuntime>`)
//!    and spawn the axum serve task on a oneshot shutdown channel.
//! 7. Spawn the gossip subscribe task on a second oneshot
//!    shutdown channel. The task joins the curator topic, pulls
//!    events, and hands every message to
//!    [`CuratorRuntime::process_announcement_bytes`].
//!
//! [`DaemonRuntime::shutdown`] drives the reverse order:
//! stop gossip task → stop HTTP serve → shutdown iroh node
//! (via `Arc::try_unwrap` once every task has dropped its clone)
//! → remove `running.json`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{Context, Result, anyhow};
use nexus_core_rs::{
    GossipClient, GossipEvent, KeyPair, Node, NodeConfig, PowSolveCache, PowVerifyCache,
    RelayPowPolicy, create_node_with_protocols, load_quorum_resolvers_from_env,
    relay_pow_policy_file_path,
};
use nexus_shell_daemon_core::auth;
use nexus_shell_daemon_core::browse::{
    BrowseAggregator, BrowseAggregatorHandle, BrowseEntry, BrowseSource, BrowseStatus,
};
use nexus_shell_daemon_core::browse_limiter::BrowseRequestLimiter;
use nexus_shell_daemon_core::config::{CuratorConfig, SeedConfig, ShellDaemonPaths};
use nexus_shell_daemon_core::iroh_runtime::{
    CuratorRuntime, CuratorRuntimeError, CuratorRuntimeHandle, curator_topic_id,
    is_node_directory_announcement,
};
use nexus_shell_daemon_core::pow_policy_loader::PowPolicyWatcher;
use nexus_shell_daemon_core::publish;
use nexus_shell_daemon_core::registry::{
    self, StaleOutcome, new_running_state, remove_running, write_running,
};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use nexus_trace_core::batch_log::BatchLogProcessor;

use crate::http::{DaemonHttpState, GossipSenderHandle, build_router};

// Sprint 20 Phase A : the env var name used by the launcher to
// hand the unlocked 32-byte Ed25519 secret key to the daemon child
// is defined canonically in `nexus_core_rs::keystore::
// SBFB_IDENTITY_SECRET_HEX_ENV` so launcher + daemon share exactly
// one string constant. We import it here rather than redeclaring.
use nexus_core_rs::SBFB_IDENTITY_SECRET_HEX_ENV;

/// Read the identity env var if the launcher set one, parse it as
/// 32 hex bytes, wipe the env table entry, and return the bytes.
///
/// The env var is removed before this function returns so a
/// subsequent child spawn (for example a future worker process the
/// daemon may fork) does not inherit the secret. The remaining
/// copies — the `[u8; 32]` returned to the caller and the original
/// heap string pulled by `env::var` — are either owned by the
/// caller (and handed straight to `NodeConfig::with_secret_key`,
/// where `iroh::SecretKey` takes over lifetime management) or are
/// dropped and zeroed by `zeroize::Zeroize` inside this function.
fn read_optional_identity_env() -> Option<[u8; 32]> {
    let mut raw = match std::env::var(SBFB_IDENTITY_SECRET_HEX_ENV) {
        Ok(v) => v,
        Err(_) => return None,
    };
    // Remove the env var from the process environment regardless of
    // whether the parse succeeds. If the var is malformed we want
    // the daemon to fall back to legacy mode rather than a second
    // parse attempt later picking up stale state.
    // SAFETY: called during early daemon init, before async runtime.
    unsafe { std::env::remove_var(SBFB_IDENTITY_SECRET_HEX_ENV) };

    let decoded = match hex::decode(raw.trim()) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "SBFB_IDENTITY_SECRET_HEX is not valid hex, falling back to ephemeral identity");
            use zeroize::Zeroize;
            raw.zeroize();
            return None;
        }
    };
    use zeroize::Zeroize;
    raw.zeroize();

    if decoded.len() != 32 {
        warn!(
            actual = decoded.len(),
            "SBFB_IDENTITY_SECRET_HEX decoded to wrong length (expected 32), falling back to ephemeral identity"
        );
        let mut decoded = decoded;
        decoded.zeroize();
        return None;
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&decoded);
    let mut decoded = decoded;
    decoded.zeroize();
    Some(out)
}

/// Load the 32-byte Ed25519 secret from `<root>/node_key`, or
/// generate a fresh one and persist it. This gives the daemon a
/// stable identity across restarts without requiring the launcher
/// keystore (`SBFB_IDENTITY_SECRET_HEX`).
fn load_or_generate_node_key(root: &Path) -> Result<[u8; 32]> {
    let path = root.join("node_key");
    if path.exists() {
        let data = std::fs::read(&path)
            .with_context(|| format!("failed to read node_key from {}", path.display()))?;
        if data.len() == 32 {
            let mut out = [0u8; 32];
            out.copy_from_slice(&data);
            return Ok(out);
        }
        warn!(len = data.len(), "node_key has wrong length, regenerating");
    }
    let secret = KeyPair::generate().secret_bytes();
    std::fs::write(&path, secret)
        .with_context(|| format!("failed to write node_key to {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("failed to set node_key permissions on {}", path.display()))?;
    }
    Ok(secret)
}

/// How long the headless boot seed driver (Sprint 75 Phase E) waits for
/// the gossip task's boot replay (outbox browse-restore + subscribed
/// directory re-pull) before proceeding best-effort. The replay itself is
/// bounded (per-anchor re-pull timeout, no network on a fresh install), so
/// this only fires when gossip subscription itself wedges.
const BOOT_DRIVER_REPLAY_WAIT_SECS: u64 = 90;

/// Options the binary hands to [`DaemonRuntime::start`].
///
/// Owned rather than borrowed so the runtime can hold on to the
/// paths for the whole daemon lifetime and the binary's `main`
/// does not have to keep `Arc`-ing them around.
pub struct DaemonStartOptions {
    pub paths: ShellDaemonPaths,
    pub api_host: String,
    /// Caller-supplied port; `0` means ephemeral. The Phase A
    /// config default is `0` so most real runs let the OS pick.
    pub api_port: u16,
    pub daemon_version: String,
    /// Sprint 11 Phase B: `[curator]` section from config.
    pub curator: CuratorConfig,
    /// Sprint 75 Phase E (D3): `[seed]` section from config — the
    /// headless boot seed driver's accept-list. Empty by default
    /// (verrou 3); an empty list drives ZERO boot network calls.
    pub seed: SeedConfig,
    /// Sprint 75 Phase E: explicit `.sbfb` security-root override.
    /// `None` (production) resolves `auth::sbfb_home()` ONCE at boot;
    /// the runtime tests inject a tempdir so the boot driver's
    /// directory-revision reads/writes can never touch the developer's
    /// real `~/.sbfb` (test-isolation finding, Phase E review).
    pub sbfb_home: Option<PathBuf>,
    /// Sprint 20 Phase B : which slot the launcher's `sbfb
    /// unlock` matched. `Normal` in the typical case, `Duress`
    /// when the user typed the duress PIN. Propagated into
    /// `DaemonHttpState` so the noop routing helpers can gate
    /// every publish / subscribe / dispatch handler.
    pub identity_mode: nexus_core_rs::IdentityMode,
    /// Sprint 33 Phase A: extra CORS origins from `--cors-origin`.
    pub cors_origins: Vec<String>,
    /// Path to built React shell directory. When set, the daemon
    /// serves static files on `/` without bearer auth.
    pub web_root: Option<PathBuf>,
}

/// A live `nexus-shell-daemon` process.
///
/// Owns the iroh [`Node`] (via an `Arc` so the gossip task can
/// share it), the running.json path (so it can remove it on
/// shutdown), the HTTP and gossip task handles, their oneshot
/// shutdown senders, and the shared [`CuratorRuntime`] handle
/// that HTTP routes read out of.
///
/// The `curator_runtime` field is cloned into both the HTTP
/// state and the gossip task, so from the `cargo build` main
/// binary's perspective it looks unused (the main code path
/// only hands the clone off once and forgets about it). The
/// test harness and the browse path reach it through
/// [`DaemonRuntime::curator_runtime`], so the field is
/// explicitly allowed as dead_code: the main binary hands the
/// clone off to the HTTP + gossip state, where the `/browse`
/// aggregation consumes it (attention-set + directory snapshot).
pub struct DaemonRuntime {
    node: Option<Arc<Node>>,
    #[allow(dead_code)]
    curator_runtime: CuratorRuntimeHandle,
    running_json: PathBuf,
    http_handle: JoinHandle<()>,
    http_shutdown: Option<oneshot::Sender<()>>,
    gossip_handle: JoinHandle<()>,
    gossip_shutdown: Option<oneshot::Sender<()>>,
    /// Sprint 16 Phase B (D2): UDS (Unix) or Named Pipe (Windows)
    /// accept loop running beside the TCP listener. `None` only
    /// when the bind failed at boot — the daemon then runs with
    /// TCP-only loopback (warn already logged).
    peer_handle: Option<JoinHandle<()>>,
    peer_shutdown: Option<oneshot::Sender<()>>,
    /// Sprint 18 audit fix D-1: file-watcher on `tokens.json`. Keeps
    /// the rotator handed to `AuthState::Rotated` synchronised with
    /// the launcher's 24 h rotation. `None` when no `tokens.json`
    /// exists at boot — the static `auth_token` path is used and
    /// no watcher is spawned. Stored on the runtime so the watcher
    /// thread + inotify handle live for the whole daemon process.
    #[allow(dead_code)]
    tokens_watcher: Option<nexus_shell_daemon_core::auth::TokenRotatorWatcher>,
    /// Sprint 20 Phase C : file-watcher on `relay_pow_policy.toml`.
    /// Kept alive for the daemon's whole lifetime so reloads reach
    /// both the publish handler (`PowSolveCache` next-solve) and the
    /// gossip receive loop (`PowVerifyCache` policy check). `None`
    /// when no `sbfb_home` resolves — the daemon then runs on a
    /// detached default-policy handle (fallback logged at boot).
    #[allow(dead_code)]
    pow_policy_watcher: Option<PowPolicyWatcher>,
    dispatch_handle: Option<JoinHandle<()>>,
    dispatch_shutdown: Option<oneshot::Sender<()>>,
    /// 2026-06-05 platform remediation (hotfix #5): result-sync bridge
    /// task + its shutdown watch. Forwards worker-written `result:`
    /// doc entries into the validator loop. Kept on the runtime so the
    /// task lives for the daemon's whole lifetime and joins cleanly.
    result_sync_handle: Option<JoinHandle<()>>,
    result_sync_shutdown: Option<tokio::sync::watch::Sender<bool>>,
    /// Hotfix #5 (maillon A): the on-demand local worker supervisor.
    /// Killed first at shutdown so the worker process never outlives
    /// the daemon (the Job Object / PDEATHSIG covers an abrupt kill).
    local_worker: Option<Arc<crate::local_worker::LocalWorkerSupervisor>>,
    feed_handle: Option<JoinHandle<()>>,
    feed_shutdown: Option<tokio::sync::watch::Sender<bool>>,
    feed_join_handles: Option<Arc<std::sync::Mutex<Vec<JoinHandle<()>>>>>,
    feed_join_shutdown: Option<Arc<tokio::sync::watch::Sender<bool>>>,
    /// Sprint 75 Phase E (Codex round 1): the headless boot task
    /// (producer re-announce + seed driver). Retained so shutdown can
    /// abort+join it — it holds an `Arc<DaemonHttpState>` (and through it
    /// the `Arc<Node>`), so a detached task still mid-pull (up to 120s
    /// per app) would keep boot network work alive past shutdown and
    /// make the Node Arc reclamation fail.
    boot_driver_handle: Option<JoinHandle<()>>,
    bound_addr: std::net::SocketAddr,
    #[allow(dead_code)]
    revocation_cache: Arc<std::sync::RwLock<nexus_core_rs::RevocationCache>>,
}

impl DaemonRuntime {
    /// Execute the full boot sequence and return a live
    /// [`DaemonRuntime`].
    pub async fn start(opts: DaemonStartOptions) -> Result<Self> {
        opts.paths
            .ensure_dirs()
            .context("failed to create shell-daemon directories")?;

        // 1. Singleton check.
        match registry::check_stale_or_bail(&opts.paths.running_json) {
            StaleOutcome::Live { pid, state } => {
                return Err(anyhow!(
                    "daemon already running (pid {}, node_id {}, port {}); stop it first or delete {} if the process is known-dead",
                    pid,
                    &state.node_id,
                    state.api_port,
                    opts.paths.running_json.display()
                ));
            }
            StaleOutcome::Stale { pid, .. } => {
                warn!(
                    pid = pid,
                    path = %opts.paths.running_json.display(),
                    "found stale running.json — previous daemon exited without cleanup, overwriting"
                );
            }
            StaleOutcome::Corrupt { reason } => {
                warn!(
                    reason = %reason,
                    path = %opts.paths.running_json.display(),
                    "running.json is unreadable, overwriting"
                );
            }
            StaleOutcome::NoFile => {
                info!("no existing running.json — fresh daemon boot");
            }
        }

        // 2. Boot the iroh endpoint + protocol router. Arc so
        //    the gossip task can hold a clone without fighting
        //    with the shutdown path for ownership.
        //
        //    Sprint 20 Phase A: if the launcher ran `sbfb unlock`
        //    before spawning us, it put the 32-byte Ed25519 secret
        //    key as a 64-char hex string into
        //    `SBFB_IDENTITY_SECRET_HEX`. We pick it up here and
        //    hand it to `NodeConfig::with_secret_key` so the iroh
        //    endpoint boots with a persistent identity instead of
        //    minting a fresh ephemeral keypair each run. Absent or
        //    malformed env → legacy `create_node()` path so dev
        //    flows that predate the encrypted keystore still work.
        // Sprint 20 Phase C : mint the PoW keypair from the same
        // secret the iroh endpoint consumes so the Hashcash
        // `publisher_pubkey` field matches the node identity the
        // peers already know via gossip. A fallback ephemeral keypair
        // (generated when the launcher did not hand a secret) pairs
        // naturally with the `create_node()` ephemeral identity path.
        // Sprint 74 Phase E: open the coordinator DB BEFORE the node so the
        // cross-node seed protocol handler (registered on the Router, which
        // accepts no post-spawn protocols) can capture it. The DB only needs
        // `paths.root` — no dependency on the node — so opening it early is
        // free. The single open lives here; the later steps reuse this handle.
        let coordinator_db_path = opts.paths.root.join("coordinator.db");
        let coordinator_db = nexus_coordinator_rs::db::CoordinatorDb::open(&coordinator_db_path)
            .map_err(|e| anyhow::anyhow!("coordinator DB open failed: {e}"))?;
        let coordinator_db = std::sync::Arc::new(std::sync::Mutex::new(coordinator_db));

        // Shared anti-replay nonce cache for the seed handler (in-memory,
        // TTL-purged). One instance, captured by the handler factory.
        let seed_nonce_cache = std::sync::Arc::new(crate::seed_protocol::NonceCache::default());

        let iroh_data_dir = opts.paths.root.join("iroh");
        let (node, pow_keypair) = match read_optional_identity_env() {
            Some(secret_bytes) => {
                info!("shell daemon using persistent identity from launcher keystore");
                let pow_kp = Arc::new(KeyPair::from_secret_bytes(&secret_bytes));
                let cfg = NodeConfig::default()
                    .with_secret_key(secret_bytes)
                    .with_data_dir(iroh_data_dir.clone());
                let factory = crate::seed_protocol::seed_protocol_factory(
                    Arc::clone(&coordinator_db),
                    Arc::clone(&pow_kp),
                    Arc::clone(&seed_nonce_cache),
                );
                let n = create_node_with_protocols(
                    cfg,
                    vec![(nexus_core_rs::SEED_ALPN.to_vec(), factory)],
                )
                .await
                .context("failed to boot iroh node with persistent identity")?;
                (n, pow_kp)
            }
            None => {
                let secret_bytes = load_or_generate_node_key(&opts.paths.root)?;
                info!(
                    path = %opts.paths.root.join("node_key").display(),
                    "shell daemon using file-based persistent identity"
                );
                let pow_kp = Arc::new(KeyPair::from_secret_bytes(&secret_bytes));
                let cfg = NodeConfig::default()
                    .with_secret_key(secret_bytes)
                    .with_data_dir(iroh_data_dir.clone());
                let factory = crate::seed_protocol::seed_protocol_factory(
                    Arc::clone(&coordinator_db),
                    Arc::clone(&pow_kp),
                    Arc::clone(&seed_nonce_cache),
                );
                let n = create_node_with_protocols(
                    cfg,
                    vec![(nexus_core_rs::SEED_ALPN.to_vec(), factory)],
                )
                .await
                .context("failed to boot iroh node with file-based identity")?;
                (n, pow_kp)
            }
        };
        let node_id = node.node_id();
        info!(node_id = %node_id, "shell daemon iroh node ready");
        let node = Arc::new(node);

        // 3. Bind the TCP listener. An empty host in the config
        //    was clamped to 127.0.0.1 at load time (see
        //    `ShellDaemonConfig::clamped`); defend-in-depth here
        //    too so a future bypass of `load` cannot slip
        //    through.
        let host = if opts.api_host.is_empty() {
            "127.0.0.1".to_string()
        } else {
            opts.api_host.clone()
        };
        let bind_target = format!("{}:{}", host, opts.api_port);
        let listener = TcpListener::bind(&bind_target)
            .await
            .with_context(|| format!("failed to bind HTTP listener to {bind_target}"))?;
        let bound_addr = listener
            .local_addr()
            .context("local_addr on freshly bound TcpListener")?;
        info!(addr = %bound_addr, "shell daemon HTTP listener bound");

        // 4. Write running.json with the real port.
        let running_json_path = opts.paths.running_json.clone();
        let running_state = new_running_state(
            node_id.clone(),
            host.clone(),
            bound_addr.port(),
            opts.daemon_version.clone(),
        );
        write_running(&running_state, &running_json_path)
            .with_context(|| format!("failed to write {}", running_json_path.display()))?;

        // 5. Construct the curator runtime and pre-load any
        //    previously persisted attention set.
        let curator_runtime: CuratorRuntimeHandle = Arc::new(CuratorRuntime::with_persistence(
            opts.paths.subscriptions_json.clone(),
        ));

        // 5a. Sprint 11 Phase B: auto-subscribe to default curators.
        // Idempotent — curators already in the persisted attention
        // set are silently skipped.
        let mut auto_subscribed = 0u32;
        for hex_key in &opts.curator.default_curators {
            if curator_runtime.subscribed_pubkeys_hex().contains(hex_key) {
                debug!(curator = %hex_key, "default curator already subscribed, skipping");
                continue;
            }
            match curator_runtime.subscribe(hex_key) {
                Ok(_) => {
                    auto_subscribed += 1;
                    debug!(curator = %hex_key, "auto-subscribed to default curator");
                }
                Err(e) => {
                    warn!(curator = %hex_key, error = %e, "failed to auto-subscribe to default curator");
                }
            }
        }
        if auto_subscribed > 0 {
            info!(
                count = auto_subscribed,
                "auto-subscribed to default curator(s)"
            );
        }

        // 5b. Construct the Phase D browse aggregator. It is
        //     cheap (just a DashMap + two duration knobs) so
        //     it is always instantiated at boot; GET /browse
        //     reaches through this handle + the Arc<Node> to
        //     probe each project's reachability on demand.
        //
        //     Sprint 19 Phase A : if `SBFB_PKARR_RELAYS` is set,
        //     wire the pkarr quorum canary so probe_and_cache
        //     short-circuits to Unreachable on an incoherent
        //     multi-relay lookup (Eclipse-by-DHT defence active
        //     in production). When the env var is absent the
        //     aggregator boots without the canary — behaviour is
        //     byte-for-byte the pre-Sprint-19 path.
        let quorum_resolvers = load_quorum_resolvers_from_env()
            .context("failed to build pkarr quorum resolvers from SBFB_PKARR_RELAYS env")?;
        let browse_aggregator: BrowseAggregatorHandle = {
            let mut agg = BrowseAggregator::new();
            if let Some(resolvers) = quorum_resolvers {
                let count = resolvers.len();
                agg = agg.with_quorum_resolvers(resolvers);
                if count < 2 {
                    warn!(
                        count,
                        "pkarr quorum canary armed with a single relay — inter-relay cross-checking requires 2+ distinct URLs (SBFB_PKARR_RELAYS)"
                    );
                } else {
                    info!(
                        count,
                        "pkarr quorum canary armed — Eclipse-by-DHT defence active"
                    );
                }
            } else {
                info!(
                    "pkarr quorum canary disabled (SBFB_PKARR_RELAYS not set) — browse probes use the default iroh N0 discovery path"
                );
            }
            Arc::new(agg)
        };

        // 5c. Sprint 11 Phase A: shared gossip sender slot. Set to
        //     `Some(sender)` once the gossip task joins the topic.
        let gossip_sender: GossipSenderHandle = Arc::new(tokio::sync::RwLock::new(None));

        // 5d. Sprint 20 Phase B : provision the panic wipe service
        //     + read the identity mode the launcher handed us.
        //     The keystore points at the same `<root>/keyring/`
        //     directory the launcher's `sbfb unlock` wrote to,
        //     so `wipe_all` reaches the exact two blob files.
        //     The state_db path matches `subscriptions_json`
        //     (the Sprint 11 persistence file, named .json but
        //     treated here as the "session state" store — any
        //     future sqlite file would be added alongside).
        let keyring_dir = opts.paths.root.join("keyring");
        let blob_cache_dir = opts.paths.root.join("blob-cache");
        let keystore_for_panic = Arc::new(nexus_core_rs::LocalFileKeyStore::new(&keyring_dir));
        let panic_wipe = Arc::new(crate::panic::PanicWipeService::new(
            keystore_for_panic,
            opts.paths.subscriptions_json.clone(),
            blob_cache_dir,
            Arc::new(crate::panic::RealExit) as Arc<dyn crate::panic::ExitStrategy>,
        ));
        let identity_mode = opts.identity_mode;

        // 5e. Sprint 20 Phase C : spawn the PoW policy file-watcher
        //     and provision the publisher / subscriber caches.
        //
        //     The watcher resolves `~/.sbfb/relay_pow_policy.toml`
        //     (overridable via `SBFB_POW_POLICY_PATH`), loads the
        //     initial policy synchronously, then starts a background
        //     thread that reloads the TOML on every write. A missing
        //     file boots on the S19 default (2^18 leading zero bits
        //     for every topic, no overrides) ; a malformed file at
        //     boot fails loud, a malformed edit at runtime keeps the
        //     last known-good policy in memory.
        //
        //     When no `$SBFB_HOME` / `$HOME` resolves (rare — headless
        //     test harness that strips the environment) the watcher
        //     is skipped and we run on a detached default-policy
        //     handle so the gate still enforces 2^18 everywhere.
        let curator_topic = curator_topic_id();
        let pow_solve_cache = Arc::new(PowSolveCache::new());
        let pow_verify_cache = Arc::new(PowVerifyCache::new());
        let (pow_policy, _pow_policy_watcher) = match relay_pow_policy_file_path() {
            Some(path) => match PowPolicyWatcher::spawn(path.clone()) {
                Ok(w) => {
                    info!(
                        path = %path.display(),
                        "PoW policy watcher armed — hot-reload enabled"
                    );
                    (w.shared(), Some(w))
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        path = %path.display(),
                        "PoW policy watcher spawn failed, falling back to default policy"
                    );
                    (
                        nexus_shell_daemon_core::pow_policy_loader::shared_default_policy(),
                        None,
                    )
                }
            },
            None => {
                warn!(
                    "SBFB_HOME / HOME / USERPROFILE not set — PoW policy locked to default (2^18)"
                );
                (
                    nexus_shell_daemon_core::pow_policy_loader::shared_default_policy(),
                    None,
                )
            }
        };

        // 5f. Sprint 29 Phase D: initialize the trace processor
        //     pipeline with a BatchLogProcessor writing to
        //     `<root>/traces/daemon.jsonl`. The processor is
        //     registered globally via `set_trace_processors` so
        //     any crate in the daemon process can call `emit()`.
        let trace_log_path = opts.paths.root.join("traces").join("daemon.jsonl");
        match BatchLogProcessor::new(&trace_log_path, 10 * 1024 * 1024) {
            Ok(proc) => {
                nexus_trace_core::set_trace_processors(vec![Box::new(proc)]);
                info!(path = %trace_log_path.display(), "trace processor pipeline initialized");
            }
            Err(e) => {
                warn!(error = %e, "failed to initialize trace processor — tracing disabled");
            }
        }

        // 6a. The coordinator SQLite database was opened earlier (before the
        //     node) so the seed protocol handler could capture it; reuse the
        //     `coordinator_db` handle from that step.

        // 6a-2. Sprint 66 Phase D: restore the RevocationCache from
        //       persisted key rotations in SQLite.
        let revocation_cache =
            nexus_shell_daemon_core::key_rotation_handler::shared_revocation_cache();
        {
            let db = coordinator_db
                .lock()
                .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
            match db.load_key_rotations() {
                Ok(rows) if !rows.is_empty() => {
                    let tuples: Vec<(String, String, u64, u16, String)> = rows
                        .iter()
                        .map(|r| {
                            (
                                r.old_pubkey.clone(),
                                r.new_pubkey.clone(),
                                r.timestamp,
                                r.transition_days,
                                r.reason.clone(),
                            )
                        })
                        .collect();
                    let applied = nexus_shell_daemon_core::key_rotation_handler::populate_cache(
                        &revocation_cache,
                        &tuples,
                    );
                    info!(
                        total = rows.len(),
                        applied, "RevocationCache restored from SQLite"
                    );
                }
                Ok(_) => {
                    debug!("no persisted key rotations to restore");
                }
                Err(e) => {
                    warn!(error = %e, "failed to load key rotations from DB");
                }
            }
        }

        // 6b. Sprint 38 Phase A: create the result event broadcast
        //     channel for the validator loop. The sender is stored in
        //     DaemonHttpState so future gossip wiring can forward
        //     result events to the loop.
        let (result_event_tx, result_event_rx) = crate::validator_loop::create_result_channel();

        // 6c. Sprint 49 Phase A: create or reopen the project iroh-docs
        //     document. The daemon acts as coordinator for the local
        //     user's project — single-project mode (D1).
        //     Sprint 81 Phase A4: open/create + sync-set entry moved to
        //     `open_project_doc_for_dispatch` so the boot path is
        //     unit-testable and the coordinator never sits outside its
        //     own doc's sync-set (dead boot->first-submit window
        //     observed LIVE on the anchor, S81 Phase A3 baseline).
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let doc_author = docs_client
            .author_default()
            .await
            .context("failed to get default docs author")?;
        let project_doc = open_project_doc_for_dispatch(&docs_client, identity_mode).await?;
        info!(
            doc_id = %project_doc.id(),
            author = %doc_author,
            "project doc ready for coordinator dispatch (sync-set entered)"
        );
        let project_doc = Arc::new(project_doc);

        // 6c-2. Sprint 49 Phase A: create the dispatch MPSC channel and
        //       spawn the dispatch loop. The loop is the sole writer to
        //       the project doc (G1 D2 ack — sequential writes, no
        //       contention with HTTP handlers).
        let (task_dispatch_tx, task_dispatch_rx) = crate::dispatch_loop::create_dispatch_channel();
        let (dispatch_shutdown_tx, dispatch_shutdown_rx) = oneshot::channel::<()>();
        let dispatch_handle = {
            let doc_clone = Arc::clone(&project_doc);
            tokio::spawn(crate::dispatch_loop::run(
                task_dispatch_rx,
                doc_clone,
                doc_author,
                dispatch_shutdown_rx,
            ))
        };

        // 6c-2b. 2026-06-05 platform remediation (hotfix #5): result-sync
        //        bridge. The dispatch loop writes `task:` entries; a
        //        worker writes `result:` entries back onto the same doc,
        //        which iroh-docs replicates here. This loop is the
        //        missing producer that forwards replicated `result:`
        //        entries into the validator loop (guardrail → persist →
        //        kudos). Without it `GET /api/v1/tasks/{id}/result`
        //        404'd forever and the Network execute arm timed out.
        let (result_sync_shutdown_tx, result_sync_shutdown_rx) = tokio::sync::watch::channel(false);
        let result_sync_handle = crate::result_sync::spawn_result_subscribe(
            Arc::clone(&project_doc),
            Arc::clone(&node),
            result_event_tx.clone(),
            result_sync_shutdown_rx,
        );

        // 6c-3. Sprint 58 Phase C: create or reopen iroh-docs storage
        //       namespaces for replicated apps. Uses the
        //       storage_namespaces M8 table to persist namespace IDs
        //       across restarts.
        let storage_namespaces = crate::storage_api::new_storage_namespaces();
        {
            let replicated_apps: &[&str] = &["sbfb-ideas"];
            for app_name in replicated_apps {
                match boot_storage_namespace(
                    &docs_client,
                    &coordinator_db,
                    app_name,
                    doc_author,
                    identity_mode,
                    Some(iroh_data_dir.as_path()),
                )
                .await
                {
                    Ok(ns_state) => {
                        info!(
                            app = %app_name,
                            doc_id = %ns_state.doc.id(),
                            "storage namespace ready"
                        );
                        let ns_arc = Arc::new(ns_state);
                        crate::storage_api::spawn_storage_subscribe(
                            app_name.to_string(),
                            Arc::clone(&ns_arc),
                        );
                        storage_namespaces
                            .write()
                            .await
                            .insert(app_name.to_string(), ns_arc);
                    }
                    Err(e) => {
                        // S81 Phase A2: a non-NotFound docs error is a corrupted
                        // store — abort the boot (diagnosable crash) instead of
                        // degrading to "storage namespace not initialized" for
                        // the whole session.
                        return Err(e.context(format!(
                            "failed to boot storage namespace for app {app_name}"
                        )));
                    }
                }
            }
        }

        // 6c-4. Sprint 62 Phase B: create or reopen iroh-docs feed
        //       namespace for public feed P2P sync. Reuses the M8
        //       storage_namespaces table with key "sbfb-feed".
        let feed_rate_limiter =
            Arc::new(nexus_shell_daemon_core::feed_limiter::FeedRateLimiter::new());
        // Sprint 74 Phase F: best-effort multi-seed registry, created before the
        // feed subscribe so the ingest path can record remote SeedAnnounced ops.
        let seed_registry = Arc::new(crate::seed_registry::SeedRegistry::new());
        let (feed_shutdown_tx, feed_shutdown_rx) = tokio::sync::watch::channel(false);
        let (feed_sync_state, feed_handle) = match boot_feed_namespace(
            &docs_client,
            &coordinator_db,
            doc_author,
            identity_mode,
            Some(iroh_data_dir.as_path()),
        )
        .await
        {
            Ok(fs) => {
                info!(
                    doc_id = %fs.doc.id(),
                    "feed sync namespace ready"
                );
                let fs_arc = Arc::new(fs);
                let handle = crate::feed_sync::spawn_feed_subscribe(
                    Arc::clone(&fs_arc),
                    Arc::clone(&coordinator_db),
                    Arc::clone(&node),
                    Arc::clone(&feed_rate_limiter),
                    Arc::clone(&seed_registry),
                    node_id.clone(),
                    feed_shutdown_rx,
                );
                (Some(fs_arc), Some(handle))
            }
            Err(e) => {
                // S81 Phase A2: same fail-fast rationale as the storage
                // namespace call-site above.
                return Err(e.context("failed to boot feed sync namespace"));
            }
        };

        // Sprint 75 audit (DURESS-BOOT-LEAK, P1): under duress the boot feed
        // republish (6c-5 + 6c-5b) is a no-op — a decoy must not re-emit the
        // operator's REAL feed history to iroh-docs under the fake keypair
        // (mirrors `run_boot_seed_driver`). Presenting `None` to both blocks
        // skips them without touching the working logic below.
        let feed_sync_for_republish =
            if crate::noop_identity::gossip_publish_in_duress(identity_mode)
                == crate::noop_identity::PublishOutcome::Noop
            {
                None
            } else {
                feed_sync_state.as_ref()
            };
        // 6c-5. Sprint 66 Phase C: republish SQLite feed entries to
        //       iroh-docs at boot (one-shot, synchronous before HTTP).
        if let Some(fs) = feed_sync_for_republish {
            let entries_result = {
                let db = coordinator_db
                    .lock()
                    .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
                nexus_coordinator_rs::public_feed::replay_all(&db)
            };
            match entries_result {
                Ok(entries) => {
                    let mut published = 0u64;
                    for entry in &entries {
                        if let Err(e) =
                            crate::feed_sync::publish_feed_entry_to_docs(fs, entry).await
                        {
                            warn!(seq = entry.seq, error = %e, "feed republish failed");
                        } else {
                            published += 1;
                        }
                    }
                    info!(
                        total = entries.len(),
                        published, "feed entries republished to iroh-docs at boot"
                    );
                }
                Err(e) => warn!(error = %e, "feed replay_all failed, skipping republish"),
            }
        }

        // 6c-5b. Sprint 66 Phase D: orphan recovery — detect entries
        //        in SQLite but missing from iroh-docs and republish.
        //        (Sprint 75 audit DURESS-BOOT-LEAK: also a no-op under duress
        //        via `feed_sync_for_republish`.)
        if let Some(fs) = feed_sync_for_republish {
            match fs.doc.get_many_by_prefix("feed/").await {
                Ok(doc_entries) => {
                    let present_keys: std::collections::HashSet<String> = doc_entries
                        .iter()
                        .filter_map(|e| String::from_utf8(e.key().to_vec()).ok())
                        .collect();

                    let entries_result = {
                        let db = coordinator_db
                            .lock()
                            .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
                        nexus_coordinator_rs::public_feed::replay_all(&db)
                    };
                    if let Ok(entries) = entries_result {
                        let entry_hash_set: std::collections::HashSet<&str> =
                            entries.iter().map(|e| e.entry_hash.as_str()).collect();
                        let mut orphan_count = 0u64;
                        let mut recovered = 0u64;
                        for entry in &entries {
                            let key =
                                crate::feed_sync::format_feed_key(&entry.author_pubkey, entry.seq);
                            if present_keys.contains(&key) {
                                continue;
                            }
                            orphan_count += 1;
                            let is_genesis = entry.prev_hash.chars().all(|c| c == '0');
                            if !is_genesis && !entry_hash_set.contains(entry.prev_hash.as_str()) {
                                warn!(
                                    seq = entry.seq,
                                    prev_hash = %entry.prev_hash,
                                    "orphan recovery: skipping broken chain tail"
                                );
                                continue;
                            }
                            if let Err(e) =
                                crate::feed_sync::publish_feed_entry_to_docs(fs, entry).await
                            {
                                warn!(seq = entry.seq, error = %e, "orphan recovery: republish failed");
                            } else {
                                recovered += 1;
                            }
                        }
                        if orphan_count > 0 {
                            info!(
                                orphans = orphan_count,
                                recovered, "feed orphan recovery completed"
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(error = %e, "orphan recovery: iroh-docs query failed, skipping");
                }
            }
        }

        // 6c-5c. Sprint 74 Phase F: re-announce every app this node keeps online
        //        (its keep_online enabled rows) to the feed, so peers learn this
        //        node still seeds them after a reboot. NEW feed-emit path (not
        //        the gossip outbox replay); covers self-deployed AND voluntarily
        //        seeded distant apps. Best-effort, after the feed namespace is
        //        ready. Self never counts itself ("Toi" is added at query time).
        if let Some(ref fs) = feed_sync_state {
            crate::feed_sync::reannounce_seeds_at_boot(
                fs,
                &coordinator_db,
                &pow_keypair,
                identity_mode,
            )
            .await;
        }

        // 6c-7. Sprint 67 Phase B: rebuild FTS5 search index from feed.
        {
            let db = coordinator_db
                .lock()
                .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
            match nexus_coordinator_rs::search::rebuild_from_feed(&db) {
                Ok(n) => info!(indexed = n, "search index rebuilt from feed at boot"),
                // H.1 (Sprint 74 Phase D, carry audit S73): boot recovery must be
                // LOUD, not a warn swallowed in the log. A failed FTS5 rebuild
                // leaves search stale/empty until the next deploy reindex; surface
                // it as an error so an operator notices (the index is fully
                // reconstructible from public_feed, so this is recoverable).
                Err(e) => tracing::error!(
                    error = %e,
                    "BOOT RECOVERY FAILED: search index rebuild_from_feed errored — \
                     search is stale/empty until the next successful deploy reindex"
                ),
            }
        }

        // 6c-6. Sprint 66 Phase C: feed_join shutdown channel +
        //       shared handle Vec for clean join at shutdown.
        let (feed_join_shutdown_tx, _) = tokio::sync::watch::channel(false);
        let feed_join_shutdown = Arc::new(feed_join_shutdown_tx);
        let feed_join_handles: Arc<std::sync::Mutex<Vec<JoinHandle<()>>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        // 6d. Build gossip command channel + shared HTTP state +
        //     spawn the serve task.
        let (gossip_cmd_tx, gossip_cmd_rx) = tokio::sync::mpsc::channel::<GossipCmd>(64);
        // Hotfix #5 (maillon A): the on-demand local worker supervisor.
        // Held both on the runtime (for shutdown) and in the HTTP state
        // (so the task submit handler can spawn the worker lazily).
        let local_worker = Arc::new(crate::local_worker::LocalWorkerSupervisor::new());
        let http_state = Arc::new(DaemonHttpState {
            node_id,
            daemon_version: opts.daemon_version.clone(),
            boot_time: SystemTime::now(),
            api_host: host,
            api_port: bound_addr.port(),
            curator_runtime: Arc::clone(&curator_runtime),
            browse_aggregator: Arc::clone(&browse_aggregator),
            node: Arc::clone(&node),
            gossip_sender: Arc::clone(&gossip_sender),
            gossip_cmd_tx: gossip_cmd_tx.clone(),
            default_curators: opts.curator.default_curators.clone(),
            blob_serve_cache: Arc::new(nexus_shell_daemon_core::blob_serve::BlobServeCache::new(
                nexus_shell_daemon_core::blob_serve::DEFAULT_MAX_CACHE_ENTRIES,
            )),
            identity_mode,
            panic_wipe,
            pow_solve_cache: Arc::clone(&pow_solve_cache),
            pow_policy: Arc::clone(&pow_policy),
            pow_keypair: Arc::clone(&pow_keypair),
            curator_gossip_topic: curator_topic,
            // Sprint 22 Phase C : outbound HTTP client for the
            // contributor-verify proxy. Built once at boot so
            coordinator_db: Arc::clone(&coordinator_db),
            result_event_tx,
            canary_registry: {
                let dir = nexus_shell_daemon_core::paths::shell_daemon_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from(".sbfb"));
                Arc::new(std::sync::Mutex::new(
                    nexus_coordinator_rs::canary_registry::CanaryRegistry::new(
                        dir.join("canary_registry.json"),
                    ),
                ))
            },
            canary_input: None,
            // Resolved ONCE at boot: the caller's explicit override (tests)
            // or the env-derived security root. Routes and the boot driver
            // all read this field's value through the same
            // `state.sbfb_home.or_else(auth::sbfb_home)` chain as before —
            // pinning it here makes the override reach the boot driver.
            sbfb_home: opts
                .sbfb_home
                .clone()
                .or_else(nexus_shell_daemon_core::auth::sbfb_home),
            project_doc: Some(Arc::clone(&project_doc)),
            task_dispatch_tx: Some(task_dispatch_tx),
            local_worker: Arc::clone(&local_worker),
            app_storage: {
                let guard = coordinator_db.lock().unwrap();
                crate::storage_api::load_app_storage_from_db(&guard)
            },
            storage_namespaces: Arc::clone(&storage_namespaces),
            storage_write_limiter: Arc::new(
                nexus_shell_daemon_core::storage_limiter::StorageWriteLimiter::new(),
            ),
            feed_sync_state,
            feed_rate_limiter,
            feed_join_handles: Arc::clone(&feed_join_handles),
            feed_join_shutdown: Arc::clone(&feed_join_shutdown),
            preview_store: nexus_shell_daemon_core::preview::PreviewStore::new(
                nexus_shell_daemon_core::preview::DEFAULT_TTL,
            ),
            seed_registry: Arc::clone(&seed_registry),
            // Sprint 81 Phase I: live shard-session registry (the store the
            // S77 stub was the seam for). Starts empty; the operator mount
            // route populates it behind the signature + membership gate.
            shard_sessions: Arc::new(crate::shard_session::ShardSessionRegistry::default()),
        });
        // Sprint 16 Phase A (D1): load the loopback bearer token.
        // The launcher generates it at first boot; if we are being
        // started directly (cargo run, tests, packaging without a
        // launcher), generate + persist one ourselves so the
        // shell binary can hit a clean daemon stand-alone.
        //
        // `SBFB_AUTH_TOKEN` env wins over the file so integration
        // tests can inject a known token without touching disk.
        //
        // Sprint 18 audit fix D-1: when the launcher has written a
        // `tokens.json` (rotation pair persisted by
        // `nexus_launcher::token_rotation::spawn_rotation_loop`),
        // boot the daemon in `AuthState::Rotated` mode and spawn a
        // file-watcher so subsequent rotations are picked up
        // without a daemon restart. The env var keeps absolute
        // precedence so test harnesses that inject a known token
        // are unaffected, and a missing file falls through to the
        // legacy single-token path (`AuthState::Static`).
        let env_token = std::env::var(auth::AUTH_TOKEN_ENV)
            .ok()
            .filter(|t| !t.is_empty());
        let (auth_state, tokens_watcher) = if let Some(t) = env_token {
            (auth::AuthState::new(t), None)
        } else if let Some(rotator_with_path) = load_initial_rotator()? {
            let (path, initial) = rotator_with_path;
            let watcher = auth::TokenRotatorWatcher::spawn(path, initial)
                .context("failed to spawn tokens.json watcher")?;
            let auth_state = auth::AuthState::rotated(watcher.shared());
            (auth_state, Some(watcher))
        } else {
            let static_token = resolve_token_from_disk()?;
            (auth::AuthState::new(static_token), None)
        };

        {
            let limiter = Arc::clone(&http_state.storage_write_limiter);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    limiter.retain_recent();
                }
            });
        }

        {
            let limiter = Arc::clone(&http_state.feed_rate_limiter);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    limiter.retain_recent();
                }
            });
        }

        {
            let store = http_state.preview_store.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    store.evict_expired();
                }
            });
        }

        // Sprint 75 Phase E: keep a handle for the headless boot driver
        // spawned below — build_router takes ownership of http_state.
        let boot_driver_state = Arc::clone(&http_state);

        let router = build_router(
            http_state,
            auth_state,
            &opts.cors_origins,
            opts.web_root.as_deref(),
        );

        // 6a. Sprint 16 Phase B (D2): spawn the UDS / Named Pipe
        //     accept loop on a clone of the same router. The
        //     accept loop wraps the cloned router with the
        //     `PeerCredsVerified` extension layer so the auth
        //     middleware bypasses bearer + Host + Origin for
        //     kernel-authenticated peers. The TCP listener spawned
        //     just below keeps the strict triple-check for browser
        //     traffic.
        let (peer_handle, peer_shutdown) = spawn_peer_listener(router.clone());

        let (http_shutdown_tx, http_shutdown_rx) = oneshot::channel::<()>();
        let http_handle = tokio::spawn(async move {
            let serve = axum::serve(listener, router).with_graceful_shutdown(async move {
                let _ = http_shutdown_rx.await;
            });
            if let Err(e) = serve.await {
                warn!(error = %e, "axum serve exited with an error");
            }
        });

        // 6d. Sprint 38 Phase A: spawn the validator loop. It
        //     drains ResultEvents from the broadcast channel and
        //     validates+credits each result. Runs until the sender
        //     half is dropped (shutdown).
        let validator_db = Arc::clone(&coordinator_db);
        tokio::spawn(crate::validator_loop::run(validator_db, result_event_rx));

        // 7. Spawn the gossip subscribe task. It joins the
        //    curator topic, streams events, and forwards each
        //    message body to the curator runtime. The oneshot
        //    lets shutdown signal a clean exit.
        //
        //    Sprint 20 Phase C : the task also receives the
        //    `PowVerifyCache` + shared policy so every inbound
        //    gossip message is unwrapped from its PoW envelope
        //    and dropped if the proof fails to satisfy the
        //    policy's topic difficulty.
        let (gossip_shutdown_tx, gossip_shutdown_rx) = oneshot::channel::<()>();
        let bootstrap_peers = curator_runtime.subscribed_pubkeys_hex();
        let initial_outbox = {
            let guard = coordinator_db
                .lock()
                .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
            guard
                .load_outbox()
                .map_err(|e| anyhow::anyhow!("outbox load failed: {e}"))?
        };
        let (boot_replay_done_tx, boot_replay_done_rx) = oneshot::channel::<()>();
        // Sprint 82 Phase A: serialize the one-shot boot seed driver (spawned
        // below) against the re-drive-on-ingest hook in the gossip task, so the
        // `was_already_announced` read-before-write in `run_boot_seed_driver`
        // cannot double-emit `SeedAnnounced` when both fire close together.
        let seed_driver_lock = Arc::new(tokio::sync::Mutex::new(()));
        let gossip_handle = spawn_gossip_subscribe_task(GossipTaskConfig {
            node: Arc::clone(&node),
            curator_runtime: Arc::clone(&curator_runtime),
            browse_aggregator: Arc::clone(&browse_aggregator),
            gossip_sender_slot: Arc::clone(&gossip_sender),
            pow_verify_cache: Arc::clone(&pow_verify_cache),
            pow_policy: Arc::clone(&pow_policy),
            shutdown_rx: gossip_shutdown_rx,
            bootstrap_peers,
            cmd_rx: gossip_cmd_rx,
            pow_solve_cache: Arc::clone(&pow_solve_cache),
            pow_keypair: Arc::clone(&pow_keypair),
            curator_topic,
            coordinator_db: Arc::clone(&coordinator_db),
            initial_outbox,
            boot_replay_done: Some(boot_replay_done_tx),
            boot_driver_state: Some(Arc::clone(&boot_driver_state)),
            keep_online_projects: opts.seed.keep_online_projects.clone(),
            seed_driver_lock: Arc::clone(&seed_driver_lock),
            redrive_coord: Arc::new(tokio::sync::Mutex::new(RedriveCoord::default())),
        });

        // Sprint 75 Phase E (D3, PO-signed): the headless boot driver.
        // Two state-driven jobs, both no-ops on a fresh default install
        // (verrou 5: empty config + never-published = zero work):
        //   1. acquire + pin every `[seed] keep_online_projects` app — the
        //      operator's EXPLICIT accept-list (an app this node may have
        //      NEVER deployed locally resolves through the subscribed node
        //      directories + the best-effort seeder registry, then
        //      `fetch_and_pin_multi` — the Phase D consumer leg, never a
        //      ticket re-mint);
        //   2. re-announce this PRODUCER's own signed `NodeDirectoryEntry`
        //      if it ever published one (the Phase C deferral: the publish
        //      route's gossip announce is live-only and does not survive a
        //      reboot on the producer side).
        // Waits for the gossip task's boot replay (outbox restore +
        // directory re-pull) so resolution sees the restored state;
        // proceeds best-effort on timeout. The handle is RETAINED on the
        // runtime (Codex round 1): shutdown aborts+joins it so a pull
        // still in flight cannot outlive the daemon or block the Node
        // Arc reclamation.
        let boot_driver_handle = {
            let configured = opts.seed.keep_online_projects.clone();
            let seed_driver_lock = Arc::clone(&seed_driver_lock);
            tokio::spawn(async move {
                if tokio::time::timeout(
                    std::time::Duration::from_secs(BOOT_DRIVER_REPLAY_WAIT_SECS),
                    boot_replay_done_rx,
                )
                .await
                .is_err()
                {
                    warn!(
                        "boot seed driver: gossip boot-replay signal timed out — proceeding best-effort"
                    );
                }
                // Producer re-announce FIRST: it is local + instantaneous
                // (build/sign/store + one gossip emit) and must not wait
                // behind the seed acquisition's network budgets — closing
                // the discovery window is the very point of the re-emit.
                if crate::http::reannounce_directory_at_boot(&boot_driver_state).await {
                    info!("producer node directory re-announced at boot");
                }
                // Sprint 82 Phase A: hold the shared lock across the driver so a
                // concurrent re-drive-on-ingest cannot double-announce.
                let pinned = {
                    let _guard = seed_driver_lock.lock().await;
                    crate::http::run_boot_seed_driver(&boot_driver_state, &configured).await
                };
                if pinned > 0 {
                    info!(
                        pinned,
                        "boot seed driver: configured apps acquired + pinned"
                    );
                }
            })
        };

        Ok(Self {
            node: Some(node),
            curator_runtime,
            running_json: running_json_path,
            http_handle,
            http_shutdown: Some(http_shutdown_tx),
            gossip_handle,
            gossip_shutdown: Some(gossip_shutdown_tx),
            peer_handle,
            peer_shutdown,
            tokens_watcher,
            pow_policy_watcher: _pow_policy_watcher,
            dispatch_handle: Some(dispatch_handle),
            dispatch_shutdown: Some(dispatch_shutdown_tx),
            result_sync_handle: Some(result_sync_handle),
            result_sync_shutdown: Some(result_sync_shutdown_tx),
            local_worker: Some(local_worker),
            feed_handle,
            feed_shutdown: Some(feed_shutdown_tx),
            feed_join_handles: Some(feed_join_handles),
            feed_join_shutdown: Some(feed_join_shutdown),
            boot_driver_handle: Some(boot_driver_handle),
            bound_addr,
            revocation_cache,
        })
    }

    /// Return the real bound socket address. Exposed so the
    /// binary's handler can print the listening port on boot
    /// for operators.
    pub fn bound_addr(&self) -> std::net::SocketAddr {
        self.bound_addr
    }

    /// Return the shared curator runtime handle. Used by tests
    /// that want to assert on the runtime state without an HTTP
    /// roundtrip, and reserved for the Phase D browse path that
    /// filters DHT resolutions by the live attention set.
    #[allow(dead_code)]
    pub fn curator_runtime(&self) -> &CuratorRuntimeHandle {
        &self.curator_runtime
    }

    #[allow(dead_code)]
    pub fn revocation_cache(&self) -> &Arc<std::sync::RwLock<nexus_core_rs::RevocationCache>> {
        &self.revocation_cache
    }

    /// Gracefully tear down the runtime.
    ///
    /// Order:
    /// 1. Signal the gossip task to stop + join it. This drops
    ///    the gossip task's `Arc<Node>` clone.
    /// 2. Signal axum to stop + join the HTTP serve task.
    /// 3. Reclaim ownership of the iroh `Node` via
    ///    `Arc::try_unwrap` (succeeds iff every task has dropped
    ///    its clone) and call the async `shutdown()`. If
    ///    `try_unwrap` fails — meaning a task leaked a clone —
    ///    we log and let the Arc fall off the stack so Drop runs.
    /// 4. Remove `running.json`.
    ///
    /// Any individual step may fail; the function logs the
    /// failure and still runs the remaining steps so the user
    /// never sees a dangling `running.json` + dangling iroh
    /// endpoint combo from a half-executed shutdown.
    pub async fn shutdown(mut self) -> Result<()> {
        // Hotfix #5: stop the on-demand worker first — it is a child
        // process that syncs the project doc, so it must go before the
        // doc subscriptions and the iroh node close.
        if let Some(lw) = self.local_worker.take() {
            lw.shutdown().await;
        }

        // Sprint 75 Phase E (Codex round 1): abort+join the boot task
        // BEFORE reclaiming the node — it holds an Arc<DaemonHttpState>
        // (and through it the Arc<Node>), and a seed pull can run up to
        // 120s per app. Abort is safe here: the driver never holds a sync
        // lock across an await (lexical-block discipline) and every DB
        // write inside it is a single synchronous statement.
        if let Some(handle) = self.boot_driver_handle.take() {
            handle.abort();
            if let Err(e) = handle.await
                && !e.is_cancelled()
            {
                warn!(error = %e, "boot driver task join failed");
            }
        }

        if let Some(tx) = self.gossip_shutdown.take() {
            let _ = tx.send(());
        }
        if let Err(e) = (&mut self.gossip_handle).await {
            warn!(error = %e, "gossip subscribe task join failed");
        }

        // Sprint 16 Phase B: signal + join the UDS / Named Pipe
        // accept loop before the TCP serve so an in-flight
        // peer-creds connection finishes draining first.
        if let Some(tx) = self.peer_shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(mut handle) = self.peer_handle.take()
            && let Err(e) = (&mut handle).await
        {
            warn!(error = %e, "peer (UDS / NP) accept task join failed");
        }

        if let Some(tx) = self.http_shutdown.take() {
            let _ = tx.send(());
        }
        if let Err(e) = (&mut self.http_handle).await {
            warn!(error = %e, "HTTP serve task join failed");
        }

        if let Some(tx) = self.feed_shutdown.take() {
            let _ = tx.send(true);
        }
        if let Some(mut handle) = self.feed_handle.take()
            && let Err(e) = (&mut handle).await
        {
            warn!(error = %e, "feed subscribe task join failed");
        }

        if let Some(sender) = self.feed_join_shutdown.take() {
            let _ = sender.send(true);
        }
        if let Some(handles_arc) = self.feed_join_handles.take() {
            let handles: Vec<_> = {
                let mut guard = handles_arc.lock().unwrap_or_else(|e| e.into_inner());
                std::mem::take(&mut *guard)
            };
            for mut h in handles {
                if let Err(e) = (&mut h).await {
                    warn!(error = %e, "feed join task join failed");
                }
            }
        }

        // Hotfix #5: stop the result-sync bridge before the dispatch
        // loop and the node — it subscribes to the project doc, so it
        // must release that subscription ahead of the iroh node close.
        if let Some(tx) = self.result_sync_shutdown.take() {
            let _ = tx.send(true);
        }
        if let Some(mut handle) = self.result_sync_handle.take()
            && let Err(e) = (&mut handle).await
        {
            warn!(error = %e, "result sync task join failed");
        }

        if let Some(tx) = self.dispatch_shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(mut handle) = self.dispatch_handle.take()
            && let Err(e) = (&mut handle).await
        {
            warn!(error = %e, "dispatch loop task join failed");
        }

        if let Some(node_arc) = self.node.take() {
            match Arc::try_unwrap(node_arc) {
                Ok(node) => {
                    if let Err(e) = node.shutdown().await {
                        warn!(error = %e, "iroh node shutdown returned an error");
                    }
                }
                Err(still_shared) => {
                    // A task leaked a clone — extremely
                    // unexpected but non-fatal. Log the strong
                    // count so a bug report has enough
                    // information, then drop the Arc and let the
                    // non-graceful Drop path run.
                    warn!(
                        strong_count = Arc::strong_count(&still_shared),
                        "iroh Node still shared at shutdown — cannot drive graceful close"
                    );
                }
            }
        }

        remove_running(&self.running_json);
        info!("shell daemon shutdown complete");
        Ok(())
    }
}

/// Resolve the bearer token from `~/.sbfb/auth_token`, generating
/// and persisting one if the file does not exist.
fn resolve_token_from_disk() -> Result<String> {
    let path = auth::auth_token_path()
        .ok_or_else(|| anyhow!("could not resolve ~/.sbfb/auth_token path for this platform"))?;
    auth::load_or_generate_token(&path).with_context(|| {
        format!(
            "failed to load or generate auth token at {}",
            path.display()
        )
    })
}

/// Sprint 18 audit fix D-1.
///
/// Try to load `<sbfb_home>/tokens.json` written by the launcher's
/// rotation loop. Returns:
///
/// - `Ok(Some((path, rotator)))` when the file is present and
///   well-formed — the daemon will boot in `AuthState::Rotated`
///   mode and spawn a watcher on `path`.
/// - `Ok(None)` when the file is absent (no launcher rotation in
///   place yet) or when the home dir does not resolve — the
///   daemon falls back to the legacy single-token `auth_token`
///   path.
/// - `Err(_)` when the file exists but is malformed — surfaced to
///   the caller because a corrupted rotation file is operator-
///   visible state, not a soft "use defaults" condition.
fn load_initial_rotator() -> Result<Option<(PathBuf, auth::TokenRotator)>> {
    let Some(path) = auth::tokens_file_path() else {
        return Ok(None);
    };
    match auth::TokenRotator::load(&path) {
        Ok(Some(rotator)) => Ok(Some((path, rotator))),
        Ok(None) => Ok(None),
        Err(e) => Err(anyhow!(
            "failed to read tokens.json at {}: {e}",
            path.display()
        )),
    }
}

impl Drop for DaemonRuntime {
    /// Best-effort `running.json` cleanup if `shutdown()` was
    /// never called (panic, early return, test drop). The iroh
    /// node and the HTTP task are left to their own Drop impls
    /// — they are not as visible to the user as the singleton
    /// marker file.
    fn drop(&mut self) {
        if Path::new(&self.running_json).exists() {
            remove_running(&self.running_json);
        }
    }
}

// =================================================================
// Peer (UDS / Named Pipe) accept loop
// =================================================================

/// Sprint 16 Phase B (D2): spawn the UDS (Unix) or Named Pipe
/// (Windows) accept loop on `router`. Returns `(None, None)` if
/// the bind fails — the daemon then falls back to TCP-only
/// loopback (with the bearer token still required), and the
/// failure is logged.
///
/// The router is consumed (cloned by the caller before the call
/// when the original is needed for TCP serving).
fn spawn_peer_listener(
    router: axum::Router,
) -> (Option<JoinHandle<()>>, Option<oneshot::Sender<()>>) {
    #[cfg(unix)]
    {
        let path = match crate::uds_server::resolve_socket_path() {
            Ok(p) => p,
            Err(e) => {
                warn!(error = %e, "could not resolve UDS path; UDS listener disabled");
                return (None, None);
            }
        };
        match crate::uds_server::spawn(router, path) {
            Ok((handle, tx)) => (Some(handle), Some(tx)),
            Err(e) => {
                warn!(error = %e, "UDS listener bind failed; daemon runs TCP-only with bearer auth");
                (None, None)
            }
        }
    }
    #[cfg(windows)]
    {
        let pipe_name = crate::named_pipe_server::resolve_pipe_name();
        match crate::named_pipe_server::spawn(router, pipe_name) {
            Ok((handle, tx)) => (Some(handle), Some(tx)),
            Err(e) => {
                warn!(error = %e, "Named Pipe listener bind failed; daemon runs TCP-only with bearer auth");
                (None, None)
            }
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = router;
        warn!("no UDS/Named Pipe support on this platform; daemon runs TCP-only");
        (None, None)
    }
}

// =================================================================
// Gossip subscribe task
// =================================================================

/// Command sent to the gossip task from HTTP handlers or the
/// curator subscribe endpoint.
pub enum GossipCmd {
    /// A new announcement was published locally — add it to the
    /// outbox so it gets replayed on NeighborUp.
    Outbox(Vec<u8>),
    /// Broadcast a browse_request to all peers so they replay
    /// their outbox. Triggered by the "Rafraichir" button.
    RequestBrowse,
    /// Hot-join the given peers (hex node ids) on the live curator
    /// topic (Sprint 81 Phase E3). Pushed by `subscribe_curator`
    /// so a peer subscribed at runtime is dialed immediately —
    /// before E3 the bootstrap set was read once at boot and a hot
    /// subscribe produced no dial until the next restart. The
    /// subscribe HTTP handler is the ONLY producer of this variant
    /// and it early-returns under duress BEFORE reaching its push
    /// (see `curator_subscribe_in_duress`), so this arm — like
    /// `RequestBrowse` — carries no duress gate of its own.
    JoinPeers(Vec<String>),
}

/// Channel sender for [`GossipCmd`]. Stored in [`DaemonHttpState`]
/// so HTTP handlers can push to the gossip outbox.
pub type GossipCmdTx = tokio::sync::mpsc::Sender<GossipCmd>;

struct GossipTaskConfig {
    node: Arc<Node>,
    curator_runtime: CuratorRuntimeHandle,
    browse_aggregator: BrowseAggregatorHandle,
    gossip_sender_slot: GossipSenderHandle,
    pow_verify_cache: Arc<PowVerifyCache>,
    pow_policy: Arc<std::sync::RwLock<RelayPowPolicy>>,
    shutdown_rx: oneshot::Receiver<()>,
    bootstrap_peers: Vec<String>,
    cmd_rx: tokio::sync::mpsc::Receiver<GossipCmd>,
    pow_solve_cache: Arc<PowSolveCache>,
    pow_keypair: Arc<KeyPair>,
    curator_topic: [u8; 32],
    coordinator_db: std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    initial_outbox: Vec<Vec<u8>>,
    /// Sprint 75 Phase E: fired once after the boot replay work inside the
    /// gossip task (outbox browse-restore + subscribed-directory re-pull) so
    /// the headless boot seed driver starts only when the directory snapshot
    /// it resolves configured apps against is populated. Best-effort: the
    /// driver proceeds on timeout if the gossip task never reaches it.
    boot_replay_done: Option<oneshot::Sender<()>>,
    /// Sprint 82 Phase A: re-drive-on-ingest context. The full HTTP state the
    /// boot seed driver resolves + pins against (a clone of the runtime's
    /// `boot_driver_state`), the operator's `[seed] keep_online_projects`
    /// accept-list, and the mutex serializing a re-drive against the in-flight
    /// boot driver. When a subscribed anchor's directory is freshly accepted,
    /// a configured `keep_online` app it now covers is pinned without a restart.
    /// `boot_driver_state` is `Option` so unit tests of the gossip task can run
    /// without constructing a full `DaemonHttpState` (a `None` never re-drives).
    boot_driver_state: Option<Arc<DaemonHttpState>>,
    keep_online_projects: Vec<String>,
    seed_driver_lock: Arc<tokio::sync::Mutex<()>>,
    /// Single-flight + dirty coordinator so a burst of accepted ingests
    /// coalesces into one re-drive chain (+ one trailing pass), never losing a
    /// trigger (Codex P1-1).
    redrive_coord: Arc<tokio::sync::Mutex<RedriveCoord>>,
}

/// Spawn the background task that subscribes to the curator
/// gossip topic (non-blocking), stores the sender immediately,
/// and replays the outbox on every NeighborUp event.
fn spawn_gossip_subscribe_task(cfg: GossipTaskConfig) -> JoinHandle<()> {
    let GossipTaskConfig {
        node,
        curator_runtime,
        browse_aggregator,
        gossip_sender_slot,
        pow_verify_cache,
        pow_policy,
        mut shutdown_rx,
        bootstrap_peers,
        mut cmd_rx,
        pow_solve_cache,
        pow_keypair,
        curator_topic,
        coordinator_db,
        initial_outbox,
        boot_replay_done,
        boot_driver_state,
        keep_online_projects,
        seed_driver_lock,
        redrive_coord,
    } = cfg;
    tokio::spawn(async move {
        let gossip = GossipClient::new(node.gossip());
        let topic_id = curator_topic_id();

        info!(
            count = bootstrap_peers.len(),
            "gossip: subscribing to topic (non-blocking)"
        );

        let topic = match gossip.subscribe_topic(topic_id, bootstrap_peers).await {
            Ok(t) => t,
            Err(e) => {
                warn!(error = %e, "gossip subscribe failed — task exits");
                return;
            }
        };

        let (sender, mut receiver) = topic.split();
        info!("gossip: sender available, topic subscribed (may be isolated until peers connect)");

        {
            let mut lock = gossip_sender_slot.write().await;
            *lock = Some(sender.clone());
        }

        let mut outbox: Vec<Vec<u8>> = initial_outbox;
        if !outbox.is_empty() {
            info!(
                entries = outbox.len(),
                "gossip: loaded persisted outbox from DB"
            );
        }
        // Self-heal (production): drop outbox announcements whose archive blob is
        // no longer held (the app was retired / GC'd). The node must not keep
        // re-advertising a card that fails to fetch on open, and a fresh peer
        // must never hear it. Runs once before restore/replay; rewrites the DB so
        // the prune is durable. Kept-online apps are pinned (skip-GC) so this
        // never removes a live app.
        let pruned = prune_stale_outbox(&node, &coordinator_db, &mut outbox).await;
        if pruned > 0 {
            info!(
                pruned,
                remaining = outbox.len(),
                "gossip: pruned stale outbox announcements (archive blob GC'd)"
            );
        }
        // Remediation #7 (Browse boot-restore): the in-memory Browse
        // aggregator starts empty on every boot, so a node's OWN published
        // apps vanish from its Browse after a restart even though their
        // announcements persist in the gossip outbox (only the outbox + feed
        // survive a restart, never the aggregator). Re-ingest each persisted
        // project announcement through the same handler the live gossip path
        // uses: it repopulates the aggregator AND, via index_browse_entry,
        // re-indexes the search corpus with the real project_name (the feed's
        // ReleasePublished op carries none, which is why search-by-name was
        // empty too). Restored self entries (our own node_id) render Reachable
        // through the aggregate() self-branch. Sprint 75 Phase A: entries are
        // unwrapped payloads; pre-S75 wrapped entries are normalized via
        // normalize_outbox_payload (structural, no PoW re-verification: our OWN
        // trusted bytes, and a difficulty-policy bump must NOT drop them).
        // `restored` counts announcements re-ingested (one per project
        // announcement envelope), not distinct cards — several envelopes for
        // the same project_id collapse to one card via add_direct_entry dedup.
        let restored =
            restore_browse_from_outbox(&browse_aggregator, &coordinator_db, &node, &outbox);
        if restored > 0 {
            info!(
                restored,
                "gossip: restored project announcements from persisted outbox"
            );
        }
        // Sprint 75 Phase C (D4 durability): re-pull each SUBSCRIBED anchor's
        // node directory from its persisted locator (`anchors.json`) so a remote
        // catalog survives this node's reboot — the in-memory directory store
        // starts empty every boot, and OWN-only outbox restore above does not
        // cover catalogs published by OTHER nodes. Gated on the attention set
        // inside `repull_directories` (verrou 5: a fresh install with no
        // subscription does ZERO boot network fetch) and bounded per anchor so a
        // dead anchor cannot stall the gossip loop. Logs internally when > 0.
        curator_runtime.repull_directories(&node).await;
        // Sprint 75 Phase E: boot replay done — outbox restored + subscribed
        // directories re-pulled. Unblock the headless boot seed driver, which
        // resolves its configured apps against this state.
        if let Some(tx) = boot_replay_done {
            let _ = tx.send(());
        }
        let mut neighbor_count: u32 = 0;
        let browse_limiter = BrowseRequestLimiter::new();
        let republish_delay = tokio::time::sleep(jittered_republish_duration());
        tokio::pin!(republish_delay);
        let mut retain_interval = tokio::time::interval(std::time::Duration::from_secs(60));
        retain_interval.tick().await; // consume the immediate first tick

        loop {
            tokio::select! {
                ev = receiver.next_event() => {
                    match ev {
                        Ok(Some(GossipEvent::Message { content, delivered_from })) => {
                            debug!(
                                delivered_from = %delivered_from,
                                bytes = content.len(),
                                "gossip message received"
                            );
                            let policy_snapshot = {
                                match pow_policy.read() {
                                    Ok(guard) => guard.clone(),
                                    Err(poisoned) => {
                                        warn!("PoW policy lock poisoned, using default");
                                        poisoned.into_inner().clone()
                                    }
                                }
                            };
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let payload: Vec<u8> = match pow_verify_cache
                                .verify_envelope(&content, &policy_snapshot, now)
                            {
                                Ok((_proof, payload)) => payload.to_vec(),
                                Err(e) => {
                                    warn!(
                                        error = %e,
                                        delivered_from = %delivered_from,
                                        bytes = content.len(),
                                        "PoW envelope verify failed — dropping gossip message"
                                    );
                                    continue;
                                }
                            };
                            if publish::is_browse_request(&payload) {
                                if !browse_limiter.check_peer(&delivered_from) {
                                    debug!(
                                        delivered_from = %delivered_from,
                                        "browse_request rate-limited — dropping"
                                    );
                                    continue;
                                }
                                debug!(delivered_from = %delivered_from, "browse_request received — replaying outbox");
                                // Phase D OFF gate (same as NeighborUp/republish).
                                let disabled = load_disabled_keep_online(&coordinator_db);
                                // Sprint 76 Phase B (B5): one endpoint-address
                                // fetch for the whole pass, reused per entry.
                                if let Some(addr) = current_replay_addr(&node).await {
                                    for stored in &outbox {
                                        if !keep_online_allows_rebroadcast(stored, &disabled) {
                                            continue;
                                        }
                                        let Some(fresh) = remint_and_wrap_for_replay(
                                            &node,
                                            &pow_solve_cache,
                                            &pow_policy,
                                            &pow_keypair,
                                            &curator_topic,
                                            &addr,
                                            stored,
                                        )
                                        .await
                                        else {
                                            continue;
                                        };
                                        if let Err(e) = sender.broadcast(fresh).await {
                                            debug!(error = %e, "browse_request outbox replay failed");
                                        }
                                    }
                                }
                            } else if publish::is_project_announcement(&payload) {
                                // Sprint 75 Phase B (Codex round 3): drop a LIVE gossip
                                // announcement that forges OUR own node_id. A peer can
                                // never legitimately announce as us — our own apps reach
                                // the aggregator via deploy (direct add) and boot-restore
                                // (our own outbox), never via the live gossip-receive
                                // path. Without this guard a peer could forge
                                // `node_id == ours` (even reusing a hash we hold) and
                                // poison `own_entries`, getting attacker-controlled
                                // metadata signed into our node directory. Boot-restore
                                // (restore_browse_from_outbox) calls the handler directly
                                // and is unaffected.
                                if announcement_claims_own_node_id(&payload, &node) {
                                    debug!(
                                        delivered_from = %delivered_from,
                                        "dropping live project announcement that forges our own node_id"
                                    );
                                } else {
                                    handle_project_announcement(
                                        &browse_aggregator,
                                        &coordinator_db,
                                        &node,
                                        &payload,
                                    );
                                }
                            } else if is_node_directory_announcement(&payload) {
                                // Sprint 75 Phase C: ingest the node directory via the
                                // shared SignedList gate — subscription-gated fetch +
                                // signature/attribution/revision verify + store, plus a
                                // persisted re-fetch locator for boot durability. The
                                // receive-side sibling of the curator arm; replaces the
                                // Phase B drop-at-debug.
                                let accepted = handle_directory_announcement(
                                    &curator_runtime,
                                    &node,
                                    &payload,
                                )
                                .await;
                                // Sprint 82 Phase A: a freshly-accepted directory may
                                // make a configured keep_online app resolvable for the
                                // first time — re-drive the boot seed driver
                                // (cooldown-coalesced, duress-safe) so it is pinned
                                // without a daemon restart. Closes S81-G-ESC-1.
                                if accepted && let Some(ref bds) = boot_driver_state {
                                    maybe_redrive_seed_on_ingest(
                                        bds,
                                        &keep_online_projects,
                                        &seed_driver_lock,
                                        &redrive_coord,
                                        REDRIVE_MIN_INTERVAL,
                                    )
                                    .await;
                                }
                            } else {
                                handle_announcement(&curator_runtime, &node, &payload).await;
                            }
                        }
                        Ok(Some(GossipEvent::NeighborUp { node_id })) => {
                            neighbor_count += 1;
                            info!(
                                neighbor = %node_id,
                                neighbors = neighbor_count,
                                outbox = outbox.len(),
                                "gossip: neighbor up — replaying outbox"
                            );
                            // Sprint 74 Phase D: do not re-broadcast apps the node
                            // has turned OFF (keep_online disabled). Fast path: an
                            // empty disabled set replays all without decoding.
                            let disabled = load_disabled_keep_online(&coordinator_db);
                            // Sprint 76 Phase B (B5): hoist the endpoint-address
                            // fetch out of the per-entry path.
                            if let Some(addr) = current_replay_addr(&node).await {
                                for stored in &outbox {
                                    if !keep_online_allows_rebroadcast(stored, &disabled) {
                                        continue;
                                    }
                                    let Some(fresh) = remint_and_wrap_for_replay(
                                        &node,
                                        &pow_solve_cache,
                                        &pow_policy,
                                        &pow_keypair,
                                        &curator_topic,
                                        &addr,
                                        stored,
                                    )
                                    .await
                                    else {
                                        continue;
                                    };
                                    if let Err(e) = sender.broadcast(fresh).await {
                                        debug!(error = %e, "outbox replay broadcast failed");
                                    }
                                }
                            }
                        }
                        Ok(Some(GossipEvent::NeighborDown { node_id })) => {
                            neighbor_count = neighbor_count.saturating_sub(1);
                            debug!(neighbor = %node_id, neighbors = neighbor_count, "gossip: neighbor down");
                        }
                        Ok(Some(GossipEvent::Lagged)) => {
                            warn!("gossip receiver lagged — some messages dropped");
                        }
                        Ok(None) => {
                            info!("gossip stream ended cleanly");
                            break;
                        }
                        Err(e) => {
                            warn!(error = %e, "gossip next_event error");
                            break;
                        }
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(GossipCmd::Outbox(payload)) => {
                            // Sprint 75 Phase A: the outbox persists the UNWRAPPED
                            // announcement payload (D2), so every later replay
                            // re-mints the address + re-stamps the PoW from it.
                            // Best-effort DB persistence: gossip broadcast is the
                            // primary transport, the DB insert is boot-recovery only;
                            // a failed insert still allows in-memory replay.
                            if let Ok(guard) = coordinator_db.lock()
                                && let Err(e) = guard.insert_outbox(&payload) {
                                    warn!(error = %e, "outbox DB insert failed");
                                }
                            // Broadcast a freshly minted + stamped envelope through
                            // the SAME helper as every replay path (the just-published
                            // payload's ticket is already fresh, so the re-mint is a
                            // no-op here — one helper keeps all broadcast paths
                            // identical). Push the unwrapped payload AFTER the borrow.
                            if neighbor_count > 0
                                && let Some(addr) = current_replay_addr(&node).await
                                    && let Some(fresh) = remint_and_wrap_for_replay(
                                        &node,
                                        &pow_solve_cache,
                                        &pow_policy,
                                        &pow_keypair,
                                        &curator_topic,
                                        &addr,
                                        &payload,
                                    )
                                    .await
                                        && let Err(e) = sender.broadcast(fresh).await {
                                            debug!(error = %e, "outbox broadcast failed");
                                        }
                            outbox.push(payload);
                        }
                        Some(GossipCmd::RequestBrowse) => {
                            let req = publish::browse_request_bytes();
                            if let Ok(envelope) = wrap_payload_with_pow_static(
                                &pow_solve_cache,
                                &pow_policy,
                                &pow_keypair,
                                &curator_topic,
                                &req,
                            ) {
                                if let Err(e) = sender.broadcast(envelope).await {
                                    debug!(error = %e, "browse_request broadcast failed");
                                } else {
                                    info!("browse_request broadcast sent to peers");
                                }
                            }
                        }
                        Some(GossipCmd::JoinPeers(peers)) => {
                            // Sprint 81 Phase E3: dial freshly subscribed peers on
                            // the live topic. Best-effort like every other arm —
                            // the join is a membership hint, the gossip swarm
                            // remains the source of truth for connectivity.
                            if let Err(e) = sender.join_peers(peers).await {
                                debug!(error = %e, "hot join_peers failed");
                            }
                        }
                        None => {
                            debug!("gossip cmd channel closed");
                            break;
                        }
                    }
                }
                _ = &mut republish_delay => {
                    if neighbor_count > 0 && !outbox.is_empty() {
                        // Phase D OFF gate (same as NeighborUp/browse_request) — an
                        // app turned OFF must stop diffusing on EVERY replay path.
                        let disabled = load_disabled_keep_online(&coordinator_db);
                        // Sprint 76 Phase B (B5): one address fetch for the pass.
                        if let Some(addr) = current_replay_addr(&node).await {
                            for stored in &outbox {
                                if !keep_online_allows_rebroadcast(stored, &disabled) {
                                    continue;
                                }
                                let Some(fresh) = remint_and_wrap_for_replay(
                                    &node,
                                    &pow_solve_cache,
                                    &pow_policy,
                                    &pow_keypair,
                                    &curator_topic,
                                    &addr,
                                    stored,
                                )
                                .await
                                else {
                                    continue;
                                };
                                if let Err(e) = sender.broadcast(fresh).await {
                                    debug!(error = %e, "periodic republish broadcast failed");
                                }
                            }
                        }
                        debug!(entries = outbox.len(), neighbors = neighbor_count, "periodic republish completed");
                    }
                    republish_delay.as_mut().reset(tokio::time::Instant::now() + jittered_republish_duration());
                }
                _ = retain_interval.tick() => {
                    browse_limiter.retain_recent();
                }
                _ = &mut shutdown_rx => {
                    info!("gossip task shut down on signal");
                    break;
                }
            }
        }
    })
}

fn jittered_republish_duration() -> std::time::Duration {
    use rand::Rng;
    let secs = rand::thread_rng().gen_range(30..=60);
    std::time::Duration::from_secs(secs)
}

/// Hand a raw gossip message body to the curator runtime and
/// log the outcome.
///
/// Sprint 8 audit C-2 split: the legacy
/// `AnnouncementAttributionMismatch` variant conflated two very
/// different situations — a benign flood of non-subscribed
/// announcements (expected in a healthy gossip network) and a
/// genuine spoofing attempt where the envelope pubkey disagrees
/// with the fetched entry. The gossip handler now logs each
/// case at its own severity:
///
/// - `NotSubscribed` → `debug!`, silent drop.
/// - `EnvelopeMismatch` → `warn!` with both hexes so an
///   operator watching `warn` traffic can investigate the
///   culprit.
/// - `RevisionRollback` → `debug!`, expected in churny networks.
/// - Everything else (blob fetch, parse, signature, persistence)
///   → `warn!`.
async fn handle_announcement(curator_runtime: &CuratorRuntimeHandle, node: &Node, content: &[u8]) {
    // Sprint 9 audit I2-F2 fix: use the throttled variant so the
    // C-4 semaphore actually caps in-flight blob fetches.
    match curator_runtime
        .process_announcement_bytes_throttled(content, node)
        .await
    {
        Ok(entry) => {
            info!(
                curator = %hex::encode(entry.curator_pubkey),
                revision = entry.list.revision,
                "curator list accepted via gossip"
            );
        }
        Err(CuratorRuntimeError::NotSubscribed { curator }) => {
            debug!(curator = %curator, "dropped announcement from non-subscribed curator");
        }
        Err(CuratorRuntimeError::EnvelopeMismatch {
            announcement,
            entry,
        }) => {
            warn!(
                announcement = %announcement,
                entry = %entry,
                "gossip announcement attribution mismatch — a peer is stapling a signed list to a different pubkey"
            );
        }
        Err(CuratorRuntimeError::RevisionRollback { new, stored }) => {
            debug!(new, stored, "ignored revision rollback");
        }
        Err(e) => {
            warn!(error = %e, "failed to process curator announcement");
        }
    }
}

/// Sprint 82 Phase A: pace between two SEQUENTIAL re-drive passes. The re-drive
/// is single-flight + dirty-coalesced (see [`RedriveCoord`]); this only spaces
/// the TRAILING pass a burst of ingests coalesces into, so a rapidly-bumping
/// subscribed anchor cannot spin the network-heavy driver.
pub(crate) const REDRIVE_MIN_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

/// Sprint 82 Phase A: single-flight + dirty coordinator for re-drive-on-ingest.
/// `active` = a re-drive chain is running; `dirty` = an ingest arrived while a
/// chain was running, so a trailing pass must re-cover it. This is true
/// COALESCING, not throttling: an ingest during an active pass is NEVER dropped
/// — it sets `dirty` and the running chain does one more pass after it finishes
/// (Codex P1-1: a leading-edge cooldown loses the only useful ingest).
#[derive(Default)]
pub(crate) struct RedriveCoord {
    active: bool,
    dirty: bool,
}

/// Sprint 82 Phase A — re-drive-on-ingest. When a node directory is freshly
/// ACCEPTED via gossip (subscription-gated + Ed25519 + anti-rollback), give the
/// boot seed driver another pass so a `keep_online` app whose directory ingests
/// AFTER the daemon booted gets pinned WITHOUT a restart — closing the S75
/// "first-boot dead window" and the S81-G-ESC-1 boot-SEED escalation.
///
/// - Cooldown-coalesced ([`REDRIVE_MIN_INTERVAL`]): the driver is network-heavy
///   (`fetch_and_pin_multi`), so an un-debounced re-drive on the hot ingest
///   path would be a DoS vector; a burst of revision announcements costs at
///   most one pass per window.
/// - Config-only accept-list: the driver iterates ONLY the operator's
///   `configured` apps, never a network-supplied pid, so an ingested directory
///   can trigger a pass but never inject a target.
/// - Duress-safe: `run_boot_seed_driver`'s duress gate is the FIRST statement
///   of the primitive, so a decoy node re-drives nothing. No pre-read of
///   `keep_online` / pid resolution / real `project_id` logging happens here
///   (DURESS-BOOT-LEAK class).
/// - Serialized against the in-flight boot driver via `seed_driver_lock` so the
///   `was_already_announced` read-before-write cannot double-emit
///   `SeedAnnounced`. LOCK SCOPE (review P3-5): the lock covers the boot driver
///   and this re-drive ONLY. `seed_voluntary` (POST /api/daemon/seed) is a third
///   `SeedAnnounced` emitter that does not take it — a concurrent manual seed
///   can still double-emit, deduped best-effort downstream by the seed
///   registry ingest, pre-existing and non-fatal.
/// - Duress-gated FIRST (review P1-2): a decoy node re-drives nothing and does
///   not even clone the real configured list here; `run_boot_seed_driver`
///   re-checks duress defense-in-depth. No observable pre-read (log / DB /
///   resolution / fetch / emit) of real data happens before either gate.
/// - Config-only accept-list: the driver iterates ONLY the operator's
///   `configured` apps, never a network-supplied pid, so an ingested directory
///   can trigger a pass but never inject a target.
/// - Single-flight + dirty-coalesced (Codex P1-1): at most one re-drive chain
///   runs at a time ([`RedriveCoord`]); an ingest arriving during a pass sets
///   `dirty` so the chain does one trailing pass that re-covers it — no ingest
///   is ever lost, and passes never pile up on the lock.
/// - Serialized against the in-flight boot driver via `seed_driver_lock` so the
///   `was_already_announced` read-before-write cannot double-emit
///   `SeedAnnounced`. LOCK SCOPE (review P3-5): the lock covers the boot driver
///   and this re-drive ONLY. `seed_voluntary` (POST /api/daemon/seed) is a third
///   `SeedAnnounced` emitter that does not take it — a concurrent manual seed
///   can still double-emit, deduped best-effort downstream by the seed registry
///   ingest, pre-existing and non-fatal.
/// - **Rate-limited** (Codex round 2), not just single-flight: after each pass
///   the chain STAYS `active` for a `pace` grace window and coalesces the
///   window's ingests into one trailing pass — so a subscribed anchor bumping
///   its revision faster than a pass takes cannot start a fresh unpaced chain
///   per bump. The driver runs at most once per `pace` (prod: [`REDRIVE_MIN_INTERVAL`]).
/// - Spawned (not awaited) so a slow pull never stalls the gossip receive loop.
///
/// Returns the spawned chain's [`JoinHandle`] when this call STARTS a chain, or
/// `None` when it is coalesced into a running chain / empty accept-list /
/// duress. The production caller fires and forgets; tests `await` the handle
/// (with a short `pace` so the grace window does not slow the test).
pub(crate) async fn maybe_redrive_seed_on_ingest(
    boot_driver_state: &Arc<DaemonHttpState>,
    configured: &[String],
    seed_driver_lock: &Arc<tokio::sync::Mutex<()>>,
    coord: &Arc<tokio::sync::Mutex<RedriveCoord>>,
    pace: std::time::Duration,
) -> Option<tokio::task::JoinHandle<()>> {
    if configured.is_empty() {
        return None;
    }
    // Duress gate FIRST (review P1-2): before cloning/using the real configured
    // list. A decoy re-drives nothing.
    if crate::noop_identity::gossip_publish_in_duress(boot_driver_state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return None;
    }
    {
        let mut c = coord.lock().await;
        if c.active {
            // A chain is already running — guarantee it re-covers this ingest
            // with a trailing pass instead of dropping it.
            c.dirty = true;
            return None;
        }
        c.active = true;
    }
    let state = Arc::clone(boot_driver_state);
    let configured = configured.to_vec();
    let lock = Arc::clone(seed_driver_lock);
    let coord = Arc::clone(coord);
    Some(tokio::spawn(async move {
        loop {
            {
                // Serialize against the boot driver / a prior pass so the
                // `was_already_announced` read-before-write cannot double-emit.
                let _guard = lock.lock().await;
                let pinned = crate::http::run_boot_seed_driver(&state, &configured).await;
                if pinned > 0 {
                    info!(pinned, "boot seed driver re-driven on directory ingest");
                }
            }
            // Grace window: STAY `active` so ingests arriving now coalesce into
            // `dirty` (one trailing pass) instead of each starting a fresh
            // UNPACED chain — this rate-limits the driver to at most one pass per
            // `pace`, not just single-flight concurrency (Codex round 2).
            tokio::time::sleep(pace).await;
            {
                let mut c = coord.lock().await;
                if !c.dirty {
                    c.active = false;
                    break;
                }
                // An ingest arrived during the pass or the grace window: consume
                // the flag and run a trailing pass that re-covers it.
                c.dirty = false;
            }
        }
    }))
}

/// Handle a gossip message identified as a node directory announcement
/// (Sprint 75 Phase C). The receive-side sibling of [`handle_announcement`]:
/// fetch + verify + store the referenced `NodeDirectoryEntry` through the shared
/// `SignedList` ingest gate, subscription-gated. Mirrors the curator arm's error
/// logging (non-subscribed/rollback at `debug!`, attribution mismatch at `warn!`)
/// so a flood of routine drops never masks a real spoof attempt.
/// Returns `true` iff the announcement was ACCEPTED (subscription-gated +
/// Ed25519 attribution + anti-rollback all passed and the entry was stored) —
/// the caller uses this to trigger the Sprint 82 Phase A re-drive-on-ingest of
/// the boot seed driver. Every reject path returns `false`.
async fn handle_directory_announcement(
    curator_runtime: &CuratorRuntimeHandle,
    node: &Node,
    content: &[u8],
) -> bool {
    match curator_runtime
        .process_directory_announcement_bytes_throttled(content, node)
        .await
    {
        Ok(entry) => {
            info!(
                node = %hex::encode(entry.node_id),
                revision = entry.directory.revision,
                "node directory accepted via gossip"
            );
            true
        }
        Err(CuratorRuntimeError::NotSubscribed { curator }) => {
            debug!(node = %curator, "dropped node directory from non-subscribed anchor");
            false
        }
        Err(CuratorRuntimeError::EnvelopeMismatch {
            announcement,
            entry,
        }) => {
            warn!(
                announcement = %announcement,
                entry = %entry,
                "node directory attribution mismatch — a peer is stapling a signed directory to a different pubkey"
            );
            false
        }
        Err(CuratorRuntimeError::RevisionRollback { new, stored }) => {
            debug!(new, stored, "ignored node directory revision rollback");
            false
        }
        Err(e) => {
            warn!(error = %e, "failed to process node directory announcement");
            false
        }
    }
}

/// Handle a gossip message identified as a project announcement.
/// Parse, validate, and insert into the browse aggregator.
///
/// Sprint 11 Phase A.
fn wrap_payload_with_pow_static(
    solve_cache: &Arc<PowSolveCache>,
    pow_policy: &Arc<std::sync::RwLock<RelayPowPolicy>>,
    keypair: &Arc<KeyPair>,
    topic: &[u8; 32],
    payload: &[u8],
) -> std::result::Result<Vec<u8>, nexus_core_rs::PowGossipError> {
    let policy = match pow_policy.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let proof = solve_cache.ensure_proof(*topic, keypair.as_ref(), &policy)?;
    nexus_core_rs::PowEnvelope::encode(&proof, payload)
}

/// Open the coordinator's project doc (or create it on first boot) AND
/// enter its iroh-docs sync-set via `start_sync(vec![])`.
///
/// `open_doc`/`create_doc` alone do NOT enter the sync-set (iroh-docs
/// 0.101, recalibrated at the S81 Phase B/C bump — mechanism unchanged
/// from 0.98: only `start_sync` inserts the namespace into `SyncState`,
/// `engine/live.rs:408-414`). A coordinator outside the sync-set (a)
/// never gossip-broadcasts its incremental `task:` writes
/// (`LocalInsert` gated by `is_syncing`, `engine/live.rs:713`) and (b)
/// REJECTS every incoming worker sync with `AbortReason::NotFound`
/// (`engine/state.rs:96-97`). Before Sprint 81 Phase A4 the sync-set
/// was only (re)armed by `share_write()` side-effects — invite mint and
/// the on-demand local-worker bootstrap on task submit
/// (`local_worker.rs` `provision()`, nudged from the submit path) —
/// which left a dead boot->first-submit window observed LIVE on the
/// anchor (S81 Phase A3 baseline: journal "Aborted sync .. NotFound"
/// ~26s after boot) and made WAN task delivery depend on a fragile
/// side-effect. Booting straight into the sync-set closes that window
/// at the root; the worker-side keepalive (`spawn_doc_sync_keepalive`,
/// S77) is complementary and untouched.
///
/// `start_sync(vec![])` dials nothing by itself, but iroh-docs merges
/// the peers PERSISTED in `docs.redb` (`register_useful_peer` /
/// `get_sync_peers`) and re-dials them (`DirectJoin`) — bounded by the
/// store's known-peer list (`PEERS_PER_DOC_CACHE_SIZE = 5`,
/// `store.rs:17`), no new wire surface, no relay in the hot path.
///
/// Under `IdentityMode::Duress` the sync-set entry is SKIPPED
/// (S81 Phase C, `noop_identity::sync_set_entry_in_duress`): the
/// reopened doc is the REAL replica (duress swaps only the node
/// keypair, never the data dir) and entering the sync-set would
/// re-dial the real persisted peers under the decoy key — regressing
/// DURESS-BOOT-LEAK (`THREAT_MODEL.md` §15.1). This is regression-free
/// functionally: no real dispatch happens under duress
/// (`task_dispatch_in_duress` => 503).
pub(crate) async fn open_project_doc_for_dispatch(
    docs_client: &nexus_core_rs::docs::DocsClient,
    identity_mode: nexus_core_rs::IdentityMode,
) -> Result<nexus_core_rs::docs::DocHandle> {
    let existing = docs_client
        .list_docs()
        .await
        .context("failed to list project docs")?;
    let project_doc = if let Some(&first_id) = existing.first() {
        docs_client
            .open_doc(first_id)
            .await
            .context("failed to open project doc")?
            .ok_or_else(|| anyhow!("project doc listed but failed to open"))?
    } else {
        docs_client
            .create_doc()
            .await
            .context("failed to create project doc")?
    };
    match crate::noop_identity::sync_set_entry_in_duress(identity_mode) {
        crate::noop_identity::SyncSetOutcome::Enter => {
            project_doc.start_sync(Vec::new()).await.context(
                "failed to enter the project doc sync-set at boot \
                 (coordinator would neither broadcast task: writes nor accept worker syncs)",
            )?;
        }
        crate::noop_identity::SyncSetOutcome::Skip => {
            // Duress: silent skip, mirroring the boot feed republish
            // (no "duress" marker is ever emitted, even at debug level).
        }
    }
    Ok(project_doc)
}

/// Mint a fresh `BlobTicket` for `hash` from the node's CURRENT endpoint address.
/// Shared by the publish path ([`crate::http::mint_blob_ticket`]) and the replay
/// re-mint helper ([`remint_and_wrap_for_replay`]): a ticket's `EndpointAddr` is a
/// point-in-time snapshot, so every (re-)announce must mint from
/// `my_endpoint_addr()` at announce time and never replay a stored address (a
/// weeks-old snapshot is undialable after a NAT/relay change even with a valid
/// proof — the address half of the Sprint 75 discovery bug). Fails if the blob is
/// no longer held locally: a re-minted ticket to a GC'd blob would advertise an
/// address that serves nothing, so content-addressing stays the truth of
/// reachability rather than the directory claim.
pub(crate) async fn mint_ticket_for_hash(
    node: &Node,
    hash: iroh_blobs::Hash,
) -> anyhow::Result<String> {
    let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
        .my_endpoint_addr()
        .await?;
    mint_ticket_for_hash_with_addr(node, hash, &addr).await
}

/// Like [`mint_ticket_for_hash`] but reuses a PRE-FETCHED endpoint address
/// (Sprint 76 Phase B, B5 hoisting). A replay pass fetches `my_endpoint_addr()`
/// ONCE ([`current_replay_addr`]) and threads it through every entry, instead of
/// querying the address watcher once per outbox entry — a node with N kept-online
/// apps did N redundant watcher round-trips per NeighborUp / browse_request /
/// periodic tick. The blob-presence check stays per-hash (content-addressing is
/// the truth of reachability: a re-minted ticket to a GC'd blob would advertise
/// an address that serves nothing).
pub(crate) async fn mint_ticket_for_hash_with_addr(
    node: &Node,
    hash: iroh_blobs::Hash,
    addr: &iroh::EndpointAddr,
) -> anyhow::Result<String> {
    use iroh_blobs::BlobFormat;
    use iroh_blobs::ticket::BlobTicket;
    let blobs = nexus_core_rs::BlobsClient::new(node.blobs_store());
    if !blobs.has(*hash.as_bytes()).await? {
        anyhow::bail!("blob {hash} no longer in local store");
    }
    Ok(BlobTicket::new(addr.clone(), hash, BlobFormat::Raw).to_string())
}

/// Fetch the node's current endpoint address once for a replay pass (Sprint 76
/// Phase B, B5 hoisting). Returns `None` (and logs at debug) when the address is
/// not yet ready, so the caller skips the whole pass — no entry could re-mint
/// anyway, turning N per-entry failures into one log line.
async fn current_replay_addr(node: &Node) -> Option<iroh::EndpointAddr> {
    match nexus_core_rs::DiscoveryClient::new(node.endpoint())
        .my_endpoint_addr()
        .await
    {
        Ok(addr) => Some(addr),
        Err(e) => {
            debug!(error = %e, "outbox replay skipped: endpoint address not ready");
            None
        }
    }
}

/// Normalize a stored outbox entry to the unwrapped [`publish::ProjectAnnouncement`]
/// gossip bytes. Sprint 75 Phase A persists the UNWRAPPED payload (D2) so every
/// replay re-mints the address + re-stamps the PoW from a live source rather than
/// rebroadcasting a frozen, stale envelope. Entries persisted BEFORE S75 are
/// PoW-wrapped envelopes; we transparently unwrap them so a live node never loses
/// its already-deployed apps on upgrade. This is runtime robustness for the
/// persisted-state transition, NOT a wire-format legacy decoder — the wire is
/// unchanged (a re-wrapped payload is byte-shape-identical on the topic). Returns
/// `None` if neither shape is a project announcement.
fn normalize_outbox_payload(stored: &[u8]) -> Option<Vec<u8>> {
    // New shape (S75+): the stored bytes ARE the announcement payload.
    if publish::is_project_announcement(stored) {
        return Some(stored.to_vec());
    }
    // Legacy shape (pre-S75): a PoW-wrapped envelope of our own — unwrap
    // structurally (no PoW re-verification; these are our trusted local bytes).
    if let Ok((_proof, payload)) = nexus_core_rs::PowEnvelope::decode(stored)
        && publish::is_project_announcement(payload)
    {
        return Some(payload.to_vec());
    }
    None
}

/// Whether an outbox entry is still SERVEABLE by this node — i.e. it carries no
/// archive (metadata-only, nothing to serve) OR its archive blob is still held
/// locally. A non-serveable entry is a stale announcement: the app's archive was
/// GC'd (the app was retired / never kept online — a kept-online app is pinned
/// skip-GC, M18, so it is never GC'd), and re-advertising it surfaces a dead
/// card that fails to fetch on open. Unparseable / malformed entries are treated
/// as non-serveable so the boot prune cleans them too.
async fn outbox_entry_is_serveable(node: &Node, stored: &[u8]) -> bool {
    let Some(payload) = normalize_outbox_payload(stored) else {
        return false;
    };
    let Ok(ann) = publish::ProjectAnnouncement::from_gossip_bytes(&payload) else {
        return false;
    };
    let Some(ticket_str) = ann.archive_ticket.as_deref() else {
        // No archive to serve — a metadata-only announcement is not stale.
        return true;
    };
    use std::str::FromStr;
    let Ok(ticket) = iroh_blobs::ticket::BlobTicket::from_str(ticket_str) else {
        return false;
    };
    let (_addr, hash, _fmt) = ticket.into_parts();
    let blobs = nexus_core_rs::BlobsClient::new(node.blobs_store());
    blobs.has(*hash.as_bytes()).await.unwrap_or(false)
}

/// Drop every stale (non-serveable) entry from the in-memory outbox and rewrite
/// the persisted outbox to match, so the node stops re-advertising apps whose
/// archive it no longer holds AND a fresh peer never hears them. Self-healing,
/// runs once at boot before the replay/restore. Returns the number pruned.
///
/// The DB has no per-row delete (`gossip_outbox` is a replace-the-set table), so
/// a prune rewrites it: `clear_outbox` then re-insert the survivors in order,
/// under one lock so the set is never observed empty. A DB rewrite failure is
/// best-effort — the in-memory filter still takes effect this session.
async fn prune_stale_outbox(
    node: &Node,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    outbox: &mut Vec<Vec<u8>>,
) -> usize {
    let before = outbox.len();
    let mut live = Vec::with_capacity(before);
    for stored in outbox.drain(..) {
        if outbox_entry_is_serveable(node, &stored).await {
            live.push(stored);
        }
    }
    let pruned = before - live.len();
    if pruned > 0 {
        // Rewrite the persisted outbox to the surviving set (best-effort).
        if let Ok(guard) = coordinator_db.lock() {
            if let Err(e) = guard.clear_outbox() {
                warn!(error = %e, "prune: clear_outbox failed");
            } else {
                for entry in &live {
                    if let Err(e) = guard.insert_outbox(entry) {
                        warn!(error = %e, "prune: re-insert_outbox failed");
                    }
                }
            }
        }
    }
    *outbox = live;
    pruned
}

/// Re-mint the address + re-stamp the PoW of an OWN outbox announcement for
/// replay (Sprint 75 Phase A — FIX-A, the fix for the live discovery bug where a
/// fresh peer dropped every announcement older than `MAX_PROOF_AGE_SECS` because
/// the outbox replayed a frozen proof). The `BlobTicket` `EndpointAddr` is
/// re-minted from the current endpoint address, and the envelope is re-wrapped
/// with a FRESH PoW. `MAX_PROOF_AGE_SECS` is unchanged: a re-stamp is a genuinely
/// fresh legitimate proof (the publisher is online now), so the receiver's 30-min
/// window stays intact — we make the window correct, we do not remove it.
///
/// Address re-mint is CONFINED to our OWN announcements (`node_id == ours`):
/// re-pointing a third party's announcement to our address would be a hijack. The
/// outbox is OWN-only by construction ([`handle_project_announcement`] routes
/// third-party announces to the aggregator, never the outbox), so this guard is
/// defense-in-depth. Returns the fresh wire envelope, or `None` if the entry does
/// not parse or the PoW solve fails (a solve failure drops THIS entry from the
/// replay pass, never the whole pass).
async fn remint_and_wrap_for_replay(
    node: &Node,
    solve_cache: &Arc<PowSolveCache>,
    pow_policy: &Arc<std::sync::RwLock<RelayPowPolicy>>,
    keypair: &Arc<KeyPair>,
    topic: &[u8; 32],
    addr: &iroh::EndpointAddr,
    stored: &[u8],
) -> Option<Vec<u8>> {
    let payload = normalize_outbox_payload(stored)?;
    let mut ann = publish::ProjectAnnouncement::from_gossip_bytes(&payload).ok()?;
    if ann.node_id == node.node_id()
        && let Some(stale) = ann.archive_ticket.as_deref()
    {
        use std::str::FromStr;
        if let Ok(ticket) = iroh_blobs::ticket::BlobTicket::from_str(stale) {
            let (_addr, hash, _fmt) = ticket.into_parts();
            // Self-heal (production): an OWN announcement whose archive blob
            // is no longer held (GC'd) must NOT keep being advertised — a
            // re-minted ticket would point at an address that serves nothing,
            // surfacing a dead card that fails on open. Drop it from this
            // replay pass. The boot prune (`prune_stale_outbox`) removes it
            // from the outbox + DB once and for all; this guard additionally
            // catches an app GC'd MID-session. Kept-online apps are pinned
            // (skip-GC tag, M18) so their blob is never GC'd and they are
            // never dropped here — only genuinely-retired apps are.
            //
            // Sprint 76 Phase B (B5): re-mint from the PRE-FETCHED pass
            // address rather than querying the watcher per entry.
            match mint_ticket_for_hash_with_addr(node, hash, addr).await {
                Ok(fresh) => ann.archive_ticket = Some(fresh),
                Err(_) => return None,
            }
        }
    }
    let fresh_payload = ann.to_gossip_bytes().ok()?;
    wrap_payload_with_pow_static(solve_cache, pow_policy, keypair, topic, &fresh_payload).ok()
}

/// Whether a gossiped [`publish::ProjectAnnouncement`] payload claims OUR own
/// node_id. Used to drop a self-impersonating LIVE gossip announcement before it
/// reaches the browse aggregator (Sprint 75 Phase B, Codex round 3): a remote
/// peer announcing `node_id == ours` is always a spoof, since our own apps are
/// added directly by deploy + boot-restore, never via the live gossip path.
/// Returns `false` for an unparseable payload (the handler will skip it anyway).
fn announcement_claims_own_node_id(payload: &[u8], node: &Node) -> bool {
    publish::ProjectAnnouncement::from_gossip_bytes(payload)
        .map(|ann| ann.node_id == node.node_id())
        .unwrap_or(false)
}

fn handle_project_announcement(
    browse_aggregator: &BrowseAggregatorHandle,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    node: &Node,
    content: &[u8],
) {
    match publish::ProjectAnnouncement::from_gossip_bytes(content) {
        Ok(ann) => {
            // Per-app identity from the announcement (blake3(name)). Fall back to
            // node_id only for a legacy announcement that predates the field, so
            // a node hosting N apps shows N distinct cards instead of collapsing
            // them onto one node_id-keyed card.
            let project_id = if ann.project_id.is_empty() {
                ann.node_id.clone()
            } else {
                ann.project_id.clone()
            };
            // Derive the archive hash from the ticket: the hash itself never
            // travels on the announcement (only the ticket does), so without this
            // the shell sees archive_hash=None, marks the app as having no archive
            // (BrowsedProject hasArchive), and never opens it. blob-serve resolves
            // the ticket back from the aggregator to P2P-download the zip.
            let archive_hash = ann
                .archive_ticket
                .as_deref()
                .and_then(crate::http::archive_hash_from_ticket);
            // Remediation #6 (freshness): seed the announcing node's address —
            // carried inside the archive ticket — into our endpoint's address
            // book so the reachability probe (and a later blob fetch) can dial
            // node_id without waiting on a pkarr round-trip. This reconciles
            // gossip discovery with the iroh service layer; mirrors
            // blobs.rs::fetch_ticket.
            if let Some(ticket_str) = ann.archive_ticket.as_deref() {
                use std::str::FromStr;
                match iroh_blobs::ticket::BlobTicket::from_str(ticket_str) {
                    Ok(ticket) => {
                        let (addr, _hash, _format) = ticket.into_parts();
                        node.memory_lookup().add_endpoint_info(addr);
                    }
                    Err(e) => {
                        debug!(error = %e, "could not parse archive ticket for addr-seed");
                    }
                }
            }
            // Sprint 76 Phase B (B2, CARRY-3): downgrade a byzantine
            // `is_open_source=true` carrying no provenance chain to `false` at
            // THIS ingress — the `/browse`-aggregator chokepoint — not only at
            // the search index (`index_browse_entry`, S74 B.6). A gossiped
            // announcement from an untrusted peer can set the flag with a null
            // `provenance_hash`/`repo_url`; without the downgrade the served
            // `/browse` card would carry the spoofable badge, and front "verrou
            // 4" (reads `source=="direct"` + `is_open_source`) would surface it.
            // Same predicate as the index path (THREAT_MODEL §15.1: declarative
            // trust, not a crypto attestation).
            let is_open_source = crate::http::trustworthy_open_source(
                ann.is_open_source,
                ann.provenance_hash.as_deref(),
                ann.repo_url.as_deref(),
            );
            if ann.is_open_source && !is_open_source {
                warn!(
                    project = %project_id,
                    "downgrading is_open_source at /browse ingress: missing provenance_hash/repo_url"
                );
            }
            let entry = BrowseEntry {
                project_id,
                // Remediation #6: the hosting node's dialable identity. The
                // freshness probe dials this, NOT project_id (= blake3(name)).
                node_id: Some(ann.node_id.clone()),
                project_name: ann.project_name,
                category: ann.category,
                description: ann.description,
                curator_pubkey: String::new(),
                curator_name: "Self-published".into(),
                source: BrowseSource::Direct,
                status: BrowseStatus::Unknown,
                last_probed_at: None,
                archive_ticket: ann.archive_ticket,
                archive_hash,
                repo_url: ann.repo_url,
                provenance_hash: ann.provenance_hash,
                is_open_source,
            };
            // Index the gossiped app for search (the gossip path deferred from the
            // search hotfix). Best-effort: a search-index hiccup must never drop
            // the discovered browse entry.
            if let Ok(db) = coordinator_db.lock() {
                crate::http::index_browse_entry(&db, &entry);
            }
            browse_aggregator.add_direct_entry(entry);
            // Path-agnostic: this handler ingests both live gossip arrivals
            // and boot-restored outbox entries (remediation #7).
            info!(
                node_id = %ann.node_id,
                "project announcement ingested"
            );
        }
        Err(e) => {
            warn!(error = %e, "failed to parse project announcement");
        }
    }
}

/// Remediation #7 (Browse boot-restore): repopulate the in-memory Browse
/// aggregator from the node's own persisted gossip outbox at startup.
///
/// The aggregator is in-memory and starts empty on every boot, so a node's
/// own published apps would otherwise disappear from its Browse after a
/// restart — only the outbox (and the feed) survive. Sprint 75 Phase A: each
/// outbox entry is the UNWRAPPED ProjectAnnouncement payload; pre-S75 entries are
/// PoW-wrapped and transparently unwrapped via [`normalize_outbox_payload`] (no
/// PoW re-verification: these are our own trusted local bytes, and a
/// difficulty-policy bump since they were minted must not drop them). We re-ingest
/// every project announcement through
/// [`handle_project_announcement`], which repopulates the aggregator and
/// re-indexes the search corpus with the real `project_name`. Returns the
/// number of project announcements restored. Idempotent: `add_direct_entry`
/// dedups by `project_id` and the search upsert is `INSERT OR REPLACE`.
///
/// Note: after a `daemon.key` identity rotation, restored entries carry the
/// pre-rotation `node_id`, so they probe as remote instead of taking the
/// self-branch — benign, since rotation also invalidates the old
/// announcements (they are re-published under the new identity).
/// Sprint 74 Phase D / Sprint 75 Phase A: should this outbox entry still be
/// re-broadcast to peers? Apps the node has turned OFF (`keep_online` disabled) are
/// skipped; everything else — including an unparseable entry — is replayed, so a
/// decode hiccup never silently drops diffusion. Fast path: an empty disabled set
/// replays all without parsing (the common case). Entries are unwrapped payloads
/// (pre-S75 wrapped entries are normalized via [`normalize_outbox_payload`]).
fn keep_online_allows_rebroadcast(
    stored: &[u8],
    disabled: &std::collections::HashSet<String>,
) -> bool {
    if disabled.is_empty() {
        return true;
    }
    // Sprint 75 Phase A: stored entries are unwrapped payloads (pre-S75 wrapped
    // entries are transparently normalized). A non-parseable entry replays — a
    // decode hiccup must never silently drop diffusion.
    let Some(payload) = normalize_outbox_payload(stored) else {
        return true;
    };
    match publish::ProjectAnnouncement::from_gossip_bytes(&payload) {
        Ok(ann) => {
            // Per-app id (blake3(name)); legacy-empty falls back to node_id, the
            // same key the toggle / disabled list uses.
            let pid = if ann.project_id.is_empty() {
                ann.node_id
            } else {
                ann.project_id
            };
            !disabled.contains(&pid)
        }
        Err(_) => true,
    }
}

/// Load the set of `project_id`s this node has turned OFF (keep_online disabled),
/// for the outbox re-broadcast gate. Best-effort (R6): a poisoned lock or DB
/// error collapses to an EMPTY set = replay-all (the safe default), never an
/// abort. Shared by ALL outbox->peer replay sites (NeighborUp, browse_request,
/// periodic republish) so the OFF gate cannot be bypassed by one path.
fn load_disabled_keep_online(
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
) -> std::collections::HashSet<String> {
    // R6: on ANY failure fall back to an empty set (= replay all, the safe
    // default) — but WARN, never swallow silently, so a real DB/lock fault is
    // observable instead of looking like "nothing is disabled".
    match coordinator_db.lock() {
        Ok(db) => match db.list_keep_online_disabled() {
            Ok(v) => v.into_iter().collect(),
            Err(e) => {
                warn!(error = %e, "keep_online disabled-set read failed; replaying all (R6 fallback)");
                std::collections::HashSet::new()
            }
        },
        Err(_) => {
            warn!("coordinator DB lock poisoned; replaying all keep_online (R6 fallback)");
            std::collections::HashSet::new()
        }
    }
}

fn restore_browse_from_outbox(
    browse_aggregator: &BrowseAggregatorHandle,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    node: &Node,
    outbox: &[Vec<u8>],
) -> usize {
    let mut restored = 0usize;
    for stored in outbox {
        // Sprint 75 Phase A: outbox entries are unwrapped payloads; pre-S75
        // PoW-wrapped entries are transparently normalized (own trusted bytes, no
        // PoW re-verification). A non-announcement entry is skipped, never dropped
        // loudly.
        if let Some(payload) = normalize_outbox_payload(stored) {
            handle_project_announcement(browse_aggregator, coordinator_db, node, &payload);
            restored += 1;
        } else {
            debug!("browse restore: skipping unparseable outbox entry");
        }
    }
    restored
}

/// Backup file left behind by the iroh-docs redb 2->4 tuple migration
/// (`migrate_redb_v2_tuples::run`: the migration writes a temp file,
/// renames `docs.redb` to this sibling, then persists the temp over the
/// original — the backup is KEPT on success). Sprint 81 Phase F: if the
/// process crashes between the rename and the persist, `docs.redb` is
/// ABSENT and the next boot creates a fresh EMPTY store that opens
/// cleanly — every M8 replica then surfaces as a legitimate "Replica
/// not found", which the A2 fail-loud does NOT catch (it only catches
/// non-NotFound errors). The recreate guard below turns that silent
/// data loss into a diagnosable refusal while the backup still holds
/// the data. Filename mirrored (with upstream provenance) by
/// `nexus-core-rs/tests/store_migration.rs`.
pub(crate) fn docs_migration_backup_path(iroh_data_dir: &std::path::Path) -> std::path::PathBuf {
    iroh_data_dir.join("docs.redb.backup-redb-v2-tuples")
}

/// Shared recreate guard for the two boot fns (Sprint 81 Phase F).
/// Called on the "Replica not found" arm ONLY. The exact precondition
/// it reacts to is "backup sibling EXISTS and replica ABSENT" — the
/// signature of an interrupted redb 2->4 migration (fresh empty store
/// after a crash mid-swap), where recreating would silently orphan the
/// data still present in the backup. A backup next to a PRESENT
/// replica is the normal trace of a migration that succeeded — that
/// path never reaches this guard. Flip side (accepted, fail-loud
/// recoverable): while a successful migration's backup lingers, a
/// LATER legitimately-absent replica also refuses instead of
/// self-healing — the remedy is deleting the backup once the migration
/// has been verified (`docs/release/STORE_MIGRATION_OPS.md`), which
/// re-arms the A2 self-heal.
fn refuse_recreate_on_interrupted_migration(
    iroh_data_dir: Option<&std::path::Path>,
    what: &str,
) -> Result<()> {
    if let Some(dir) = iroh_data_dir {
        let backup = docs_migration_backup_path(dir);
        if backup.exists() {
            return Err(anyhow!(
                "{what} replica is absent BUT a redb migration backup exists at {} — an \
                 interrupted redb migration (crash between rename and persist) leaves a fresh \
                 empty docs store; refusing to silently recreate. Restore the backup over \
                 docs.redb (or restore the tar snapshot) and reboot; if the migration is old \
                 and already verified, delete the stale backup to re-arm the self-heal \
                 (docs/release/STORE_MIGRATION_OPS.md)",
                backup.display()
            ));
        }
    }
    Ok(())
}

/// Boot or reopen an iroh-docs storage namespace for a replicated
/// app. Checks the M8 `storage_namespaces` table for a persisted
/// NamespaceId. If found, reopens; otherwise creates a new namespace,
/// generates a Write ticket, and persists both.
///
/// S81 Phase C (P2-SIBLING-SYNC-SET, sibling of the Phase A4 project
/// doc fix): every arm converges on an explicit `start_sync(vec![])`
/// so the reopened namespace ENTERS its sync-set at boot. Before this
/// fix the ticket-persisted reopen arm returned the doc outside the
/// sync-set (broadcasts suppressed, incoming syncs rejected with
/// `AbortReason::NotFound`) — the create arms only entered it via the
/// fragile `share_write()` side-effect. Fail-fast, never warn-only
/// (Phase A2 doctrine): a silently missed sync-set entry would reopen
/// the "silent loss" class A2 closed. Under `IdentityMode::Duress` the
/// entry is SKIPPED (`noop_identity::sync_set_entry_in_duress`): the
/// store is the REAL one and dialing its persisted peers under the
/// decoy key would regress DURESS-BOOT-LEAK (§15.1). The create arms
/// (first-boot / recreate) also `share_write()` past this gate, but on
/// a FRESH namespace — 0 persisted peers to dial, no real content to
/// serve — so they are benign under duress. Residual, out of reach in
/// prod: the ticket-None reopen sub-arm is the only side-effect
/// touching the REAL replica (re-mint via `share_write()`) —
/// unreachable, every M8 write since S58 persists `Some(ticket)`.
pub(crate) async fn boot_storage_namespace(
    docs_client: &nexus_core_rs::docs::DocsClient,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    app_name: &str,
    author: nexus_core_rs::docs::DocsAuthorId,
    identity_mode: nexus_core_rs::IdentityMode,
    iroh_data_dir: Option<&std::path::Path>,
) -> Result<crate::storage_api::StorageNamespaceState> {
    use std::sync::atomic::AtomicU64;

    let existing = {
        let db = coordinator_db
            .lock()
            .map_err(|e| anyhow!("coordinator DB lock failed: {e}"))?;
        db.get_storage_namespace(app_name)
            .map_err(|e| anyhow!("failed to read storage namespace for {app_name}: {e}"))?
    };

    let (doc, ticket_str) = match existing {
        Some(row) => {
            let bytes: [u8; 32] = row
                .namespace_id
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("invalid namespace_id length in DB"))?;
            let ns_id = nexus_core_rs::docs::DocsNamespaceId::from(bytes);
            // Self-heal boundary (S81 Phase A2). The M8 row may point at a
            // namespace whose iroh-docs replica is gone (store reset, a DB
            // carried over from another data dir). In iroh-docs 0.101
            // (re-verified at the S81 Phase B bump: upstream store.rs:24-27
            // Display byte-identical, api.rs:262-265 still hardcodes
            // `Ok(Some)`) `open_doc` NEVER returns `Ok(None)`: a legitimately
            // absent replica surfaces as `Err(OpenError::NotFound)` whose
            // message contains "Replica not found" (the typed variant is
            // erased to a string by the RPC layer, so a message match is the
            // only discriminator available here). Only that absence may recreate;
            // any other error (redb/IO/actor) means the store is corrupted
            // and the boot must fail loudly instead of silently orphaning the
            // replicated entries under a fresh namespace id.
            let opened = match docs_client.open_doc(ns_id).await {
                Ok(opt) => opt,
                Err(e) if e.to_string().contains("Replica not found") => None,
                Err(e) => {
                    return Err(anyhow!(
                        "storage namespace open failed for app {app_name} (ns {ns_id}): {e} \
                         — refusing to silently recreate; restore the iroh store or clear \
                         the M8 storage_namespaces row"
                    ));
                }
            };
            match opened {
                Some(doc) => {
                    let ticket_str = match row.doc_ticket {
                        Some(t) => t,
                        None => {
                            let ticket = doc.share_write().await?;
                            let t = ticket.to_string();
                            let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
                            db.set_storage_namespace(app_name, doc.id().as_bytes(), Some(&t))
                                .map_err(|e| {
                                    anyhow!("failed to persist storage namespace ticket: {e}")
                                })?;
                            t
                        }
                    };
                    (doc, ticket_str)
                }
                None => {
                    refuse_recreate_on_interrupted_migration(
                        iroh_data_dir,
                        &format!("storage namespace for app {app_name} (ns {ns_id})"),
                    )?;
                    warn!(
                        app = %app_name,
                        ns = %ns_id,
                        "previous replica absent from local store — recreating fresh storage namespace"
                    );
                    let doc = docs_client.create_doc().await?;
                    let ticket = doc.share_write().await?;
                    let ticket_str = ticket.to_string();
                    let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
                    db.set_storage_namespace(app_name, doc.id().as_bytes(), Some(&ticket_str))
                        .map_err(|e| {
                            anyhow!("failed to persist recreated storage namespace: {e}")
                        })?;
                    (doc, ticket_str)
                }
            }
        }
        None => {
            // First boot (no M8 row): intentionally NOT migration-guarded —
            // with no row there is nothing an interrupted migration could
            // orphan; the guard only protects the row-present recreate arm.
            let doc = docs_client.create_doc().await?;
            let ticket = doc.share_write().await?;
            let ticket_str = ticket.to_string();
            let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
            db.set_storage_namespace(app_name, doc.id().as_bytes(), Some(&ticket_str))
                .map_err(|e| anyhow!("failed to persist new storage namespace: {e}"))?;
            (doc, ticket_str)
        }
    };

    // S81 Phase C chokepoint: single sync-set entry for ALL arms
    // (idempotent on the already-armed create arms), duress-gated.
    match crate::noop_identity::sync_set_entry_in_duress(identity_mode) {
        crate::noop_identity::SyncSetOutcome::Enter => {
            doc.start_sync(Vec::new()).await.with_context(|| {
                format!(
                    "failed to enter the storage namespace sync-set at boot for app \
                     {app_name} (ns {}) — the namespace would neither broadcast its \
                     writes nor accept incoming syncs",
                    doc.id()
                )
            })?;
        }
        crate::noop_identity::SyncSetOutcome::Skip => {
            // Duress: silent skip (no "duress" marker, even at debug level).
        }
    }

    Ok(crate::storage_api::StorageNamespaceState {
        doc: Arc::new(doc),
        author,
        ticket: ticket_str,
        version: AtomicU64::new(0),
    })
}

/// Boot or reopen the iroh-docs namespace for the public feed
/// P2P sync. Mirrors `boot_storage_namespace` but produces a
/// `FeedSyncState` without the version counter (feed dedup is
/// hash-based, not version-based).
///
/// S81 Phase C: same sync-set chokepoint + duress gate as
/// `boot_storage_namespace` (P2-SIBLING-SYNC-SET). The feed doc is
/// network-visible, so a reopened-but-never-started feed namespace
/// had real reach: the S75 PULL directory is only a partial
/// mitigation when the feed doc itself rejects every sync.
pub(crate) async fn boot_feed_namespace(
    docs_client: &nexus_core_rs::docs::DocsClient,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    author: nexus_core_rs::docs::DocsAuthorId,
    identity_mode: nexus_core_rs::IdentityMode,
    iroh_data_dir: Option<&std::path::Path>,
) -> Result<crate::feed_sync::FeedSyncState> {
    let feed_key = crate::feed_sync::FEED_NAMESPACE_KEY;

    let existing = {
        let db = coordinator_db
            .lock()
            .map_err(|e| anyhow!("coordinator DB lock failed: {e}"))?;
        db.get_storage_namespace(feed_key)
            .map_err(|e| anyhow!("failed to read feed namespace: {e}"))?
    };

    let (doc, ticket_str) = match existing {
        Some(row) => {
            let bytes: [u8; 32] = row
                .namespace_id
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("invalid namespace_id length in DB"))?;
            let ns_id = nexus_core_rs::docs::DocsNamespaceId::from(bytes);
            // Self-heal boundary (S81 Phase A2, mirrors boot_storage_namespace):
            // only a legitimately absent replica ("Replica not found") may
            // recreate; any other docs error fails the boot loudly.
            let opened = match docs_client.open_doc(ns_id).await {
                Ok(opt) => opt,
                Err(e) if e.to_string().contains("Replica not found") => None,
                Err(e) => {
                    return Err(anyhow!(
                        "feed namespace open failed (ns {ns_id}): {e} — refusing to \
                         silently recreate; restore the iroh store or clear the M8 \
                         storage_namespaces row for key {feed_key}"
                    ));
                }
            };
            match opened {
                Some(doc) => {
                    let ticket_str = match row.doc_ticket {
                        Some(t) => t,
                        None => {
                            let ticket = doc.share_write().await?;
                            let t = ticket.to_string();
                            let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
                            db.set_storage_namespace(feed_key, doc.id().as_bytes(), Some(&t))
                                .map_err(|e| {
                                    anyhow!("failed to persist feed namespace ticket: {e}")
                                })?;
                            t
                        }
                    };
                    (doc, ticket_str)
                }
                None => {
                    refuse_recreate_on_interrupted_migration(
                        iroh_data_dir,
                        &format!("feed namespace (ns {ns_id})"),
                    )?;
                    warn!(
                        ns = %ns_id,
                        "previous replica absent from local store — recreating fresh feed namespace"
                    );
                    let doc = docs_client.create_doc().await?;
                    let ticket = doc.share_write().await?;
                    let ticket_str = ticket.to_string();
                    let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
                    db.set_storage_namespace(feed_key, doc.id().as_bytes(), Some(&ticket_str))
                        .map_err(|e| anyhow!("failed to persist recreated feed namespace: {e}"))?;
                    (doc, ticket_str)
                }
            }
        }
        None => {
            // First boot (no M8 row): intentionally NOT migration-guarded —
            // mirror of the storage arm above.
            let doc = docs_client.create_doc().await?;
            let ticket = doc.share_write().await?;
            let ticket_str = ticket.to_string();
            let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
            db.set_storage_namespace(feed_key, doc.id().as_bytes(), Some(&ticket_str))
                .map_err(|e| anyhow!("failed to persist new feed namespace: {e}"))?;
            (doc, ticket_str)
        }
    };

    // S81 Phase C chokepoint: mirrors boot_storage_namespace.
    match crate::noop_identity::sync_set_entry_in_duress(identity_mode) {
        crate::noop_identity::SyncSetOutcome::Enter => {
            doc.start_sync(Vec::new()).await.with_context(|| {
                format!(
                    "failed to enter the feed namespace sync-set at boot (ns {}) — the \
                     public feed would neither broadcast its writes nor accept incoming \
                     syncs",
                    doc.id()
                )
            })?;
        }
        crate::noop_identity::SyncSetOutcome::Skip => {
            // Duress: silent skip (no "duress" marker, even at debug level).
        }
    }

    Ok(crate::feed_sync::FeedSyncState {
        doc: Arc::new(doc),
        author,
        ticket: ticket_str,
    })
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gossip_announcement_populates_archive_hash_from_ticket() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        // Node hosting the zip (provides the ticket address).
        let node = nexus_core_rs::create_node().await.unwrap();
        let blobs = nexus_core_rs::BlobsClient::new(node.blobs_store());
        let hash = blobs.add_bytes(b"zip-bytes".to_vec()).await.unwrap();
        let hash_hex = hex::encode(hash);
        let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let ticket = iroh_blobs::ticket::BlobTicket::new(
            addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();

        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Remote App"));
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "Remote App".into(),
            "tools".into(),
            "d".into(),
            vec![],
        )
        .with_project_id(pid.clone())
        .with_archive_ticket(ticket);
        super::handle_project_announcement(&agg, &db, &node, &ann.to_gossip_bytes().unwrap());
        // archive_hash is derived from the ticket so the shell knows it HAS an
        // archive (the hash never travels on the announcement itself).
        let entry = agg.get_direct_entry(&pid).expect("entry present");
        assert_eq!(entry.archive_hash, Some(hash_hex));
        assert!(entry.archive_ticket.is_some());
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn aggregator_downgrades_open_source_without_provenance() {
        // Sprint 76 Phase B (B2, CARRY-3): a gossiped announcement claiming
        // `is_open_source=true` with NO provenance chain (null provenance_hash /
        // repo_url) is downgraded to `false` at the /browse-aggregator INGRESS,
        // so the SERVED Browse card never carries the spoofable badge — not only
        // the search index (S74 B.6). A full provenance chain is preserved.
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = nexus_core_rs::create_node().await.unwrap();
        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));

        // Byzantine: claims open-source, no provenance/repo → downgraded to false.
        let liar_pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Liar App"));
        let liar = ProjectAnnouncement::new(
            "a".repeat(64),
            "Liar App".into(),
            "tools".into(),
            "d".into(),
            vec![],
        )
        .with_project_id(liar_pid.clone())
        .with_open_source(true);
        super::handle_project_announcement(&agg, &db, &node, &liar.to_gossip_bytes().unwrap());
        let entry = agg.get_direct_entry(&liar_pid).expect("liar entry present");
        assert!(
            !entry.is_open_source,
            "a gossiped open-source claim with no provenance chain must be downgraded at /browse ingress"
        );

        // Honest: full provenance chain (repo_url + provenance_hash) → preserved.
        let honest_pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Honest App"));
        let honest = ProjectAnnouncement::new(
            "b".repeat(64),
            "Honest App".into(),
            "tools".into(),
            "d".into(),
            vec![],
        )
        .with_project_id(honest_pid.clone())
        .with_repo_url("https://codeberg.org/me/app.git".into())
        .with_provenance_hash("ef".repeat(32))
        .with_open_source(true);
        super::handle_project_announcement(&agg, &db, &node, &honest.to_gossip_bytes().unwrap());
        let entry = agg
            .get_direct_entry(&honest_pid)
            .expect("honest entry present");
        assert!(
            entry.is_open_source,
            "an honest open-source claim with full provenance chain must be preserved"
        );

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn live_gossip_drops_self_node_id_spoof() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        // Codex round 3 guard: a LIVE gossip announcement forging OUR own node_id
        // must never enter the aggregator (it would poison own_entries → the
        // signed node directory). A legit remote announcement is still added.
        let node = nexus_core_rs::create_node().await.unwrap();
        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));

        // Spoof: announces OUR own node_id.
        let spoof = ProjectAnnouncement::new(
            node.node_id(),
            "Spoofed".into(),
            "tools".into(),
            "evil".into(),
            vec![],
        )
        .with_project_id(hex::encode(nexus_core_rs::crypto::blake3_hash(b"Spoofed")));
        let spoof_bytes = spoof.to_gossip_bytes().unwrap();
        assert!(
            super::announcement_claims_own_node_id(&spoof_bytes, &node),
            "guard must detect a self-node_id spoof"
        );
        // Mirror the live dispatch: the guard skips the handler for a self-spoof.
        if !super::announcement_claims_own_node_id(&spoof_bytes, &node) {
            super::handle_project_announcement(&agg, &db, &node, &spoof_bytes);
        }
        assert_eq!(
            agg.direct_entry_count(),
            0,
            "a self-node_id spoof must never enter the aggregator"
        );

        // A legit remote announcement (different node_id) IS added.
        let remote = ProjectAnnouncement::new(
            "a".repeat(64),
            "Remote".into(),
            "tools".into(),
            "ok".into(),
            vec![],
        )
        .with_project_id(hex::encode(nexus_core_rs::crypto::blake3_hash(b"Remote")));
        let remote_bytes = remote.to_gossip_bytes().unwrap();
        assert!(!super::announcement_claims_own_node_id(
            &remote_bytes,
            &node
        ));
        if !super::announcement_claims_own_node_id(&remote_bytes, &node) {
            super::handle_project_announcement(&agg, &db, &node, &remote_bytes);
        }
        assert_eq!(
            agg.direct_entry_count(),
            1,
            "a legit remote announcement is still added"
        );
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn gossip_announcement_uses_per_app_id_and_indexes() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = nexus_core_rs::create_node().await.unwrap();
        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let node_id = "a".repeat(64);
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Cool App"));
        let ann = ProjectAnnouncement::new(
            node_id.clone(),
            "Cool App".into(),
            "tools".into(),
            "p2p tool".into(),
            vec![],
        )
        .with_project_id(pid.clone());
        super::handle_project_announcement(&agg, &db, &node, &ann.to_gossip_bytes().unwrap());
        // Browse card keyed by per-app project_id, not node_id.
        assert_eq!(agg.direct_entry_count(), 1);
        assert!(
            agg.get_direct_entry(&pid).is_some(),
            "card keyed by blake3 id"
        );
        assert!(
            agg.get_direct_entry(&node_id).is_none(),
            "card is NOT keyed by node_id"
        );
        // And searchable through the gossip indexing path (deferred from #2).
        let (results, total) =
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "Cool", 20, 0).unwrap();
        assert_eq!(total, 1);
        assert_eq!(results[0].project_id, pid);
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn gossip_two_apps_same_node_are_distinct_cards() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = nexus_core_rs::create_node().await.unwrap();
        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let node_id = "b".repeat(64);
        for name in ["First App", "Second App"] {
            let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(name.as_bytes()));
            let ann = ProjectAnnouncement::new(
                node_id.clone(),
                name.into(),
                "tools".into(),
                "desc".into(),
                vec![],
            )
            .with_project_id(pid);
            super::handle_project_announcement(&agg, &db, &node, &ann.to_gossip_bytes().unwrap());
        }
        // Same node, two apps -> two distinct cards (not collapsed on node_id).
        assert_eq!(agg.direct_entry_count(), 2);
        assert_eq!(
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "First", 20, 0)
                .unwrap()
                .1,
            1
        );
        assert_eq!(
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "Second", 20, 0)
                .unwrap()
                .1,
            1
        );
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn gossip_legacy_announcement_falls_back_to_node_id() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = nexus_core_rs::create_node().await.unwrap();
        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let node_id = "c".repeat(64);
        // No with_project_id() -> empty -> receiver falls back to node_id.
        let ann = ProjectAnnouncement::new(
            node_id.clone(),
            "Legacy App".into(),
            "tools".into(),
            "desc".into(),
            vec![],
        );
        super::handle_project_announcement(&agg, &db, &node, &ann.to_gossip_bytes().unwrap());
        assert!(
            agg.get_direct_entry(&node_id).is_some(),
            "legacy announcement falls back to node_id key"
        );
        node.shutdown().await.ok();
    }

    // multi_thread is mandatory: a real iroh node's gossip actor needs a
    // dedicated thread (P2-A-1, PATTERNS §P54) — current_thread can deadlock.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn gossip_cmd_outbox_persists_to_db() {
        // Sprint 76 Phase B (B4, T6-OUTBOX-DIRECT): the GossipCmd::Outbox handler
        // had no direct test (grep = the handler is its sole occurrence). Drive
        // the REAL gossip subscribe task and assert that an Outbox command
        // persists the unwrapped announcement to the DB outbox — the
        // deterministic, neighbor-independent half (boot-recovery durability, D2).
        // The neighbor-gated broadcast half is exercised by the S75 LIVE
        // cross-node WAN acceptance, not a flaky in-process gossip mesh: NO 2-node
        // NeighborUp test exists in this crate — every cross-node test uses direct
        // ticket/docs connectivity, never gossip-mesh formation.
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = Arc::new(nexus_core_rs::create_node().await.expect("boot node"));
        let coordinator_db = Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().expect("db"),
        ));
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<GossipCmd>(16);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let (boot_done_tx, boot_done_rx) = oneshot::channel::<()>();

        let handle = spawn_gossip_subscribe_task(GossipTaskConfig {
            node: Arc::clone(&node),
            curator_runtime: Arc::new(CuratorRuntime::new(None)),
            browse_aggregator: Arc::new(BrowseAggregator::new()),
            gossip_sender_slot: Arc::new(tokio::sync::RwLock::new(None)),
            pow_verify_cache: Arc::new(PowVerifyCache::new()),
            pow_policy: Arc::new(std::sync::RwLock::new(RelayPowPolicy {
                default_difficulty: 1,
                topic_overrides: std::collections::BTreeMap::new(),
            })),
            shutdown_rx,
            bootstrap_peers: vec![],
            cmd_rx,
            pow_solve_cache: Arc::new(PowSolveCache::new()),
            pow_keypair: Arc::new(KeyPair::generate()),
            curator_topic: curator_topic_id(),
            coordinator_db: Arc::clone(&coordinator_db),
            initial_outbox: vec![],
            boot_replay_done: Some(boot_done_tx),
            // Sprint 82 Phase A: this test drives the Outbox command, not the
            // re-drive-on-ingest path — no boot driver state needed.
            boot_driver_state: None,
            keep_online_projects: vec![],
            seed_driver_lock: Arc::new(tokio::sync::Mutex::new(())),
            redrive_coord: Arc::new(tokio::sync::Mutex::new(RedriveCoord::default())),
        });

        // Wait for the boot replay to finish so the select loop is consuming cmds
        // (bounded — the cmd also buffers in the channel regardless).
        let _ = tokio::time::timeout(std::time::Duration::from_secs(10), boot_done_rx).await;

        let ann = ProjectAnnouncement::new(
            node.node_id(),
            "Outbox App".into(),
            "tools".into(),
            "d".into(),
            vec![],
        )
        .with_project_id(hex::encode(nexus_core_rs::crypto::blake3_hash(
            b"Outbox App",
        )));
        let payload = ann.to_gossip_bytes().unwrap();
        cmd_tx
            .send(GossipCmd::Outbox(payload.clone()))
            .await
            .expect("send Outbox cmd");

        // Poll the DB until the handler has persisted the entry (bounded 5s).
        let mut persisted: Vec<Vec<u8>> = vec![];
        for _ in 0..50 {
            persisted = {
                let db = coordinator_db.lock().unwrap_or_else(|p| p.into_inner());
                db.load_outbox().unwrap_or_default()
            };
            if !persisted.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        assert_eq!(
            persisted.len(),
            1,
            "GossipCmd::Outbox must persist the announcement to the DB outbox"
        );
        assert_eq!(
            persisted[0], payload,
            "the persisted outbox bytes must be the unwrapped announcement payload (D2)"
        );

        let _ = shutdown_tx.send(());
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
        // `node` is an Arc<Node> (GossipTaskConfig owns a clone); shutdown() takes
        // owned self, so we cannot move out of the Arc — drop it and let the gossip
        // task (already signalled) and the node clean up on drop.
        drop(node);
    }

    #[tokio::test]
    async fn freshness_probe_marks_gossiped_remote_app_reachable_e2e() {
        // Remediation #6 real-frontier gate (PATTERNS §P57): a genuine
        // two-node path with NO mock at the discovery<->service boundary.
        // node_a hosts an app and gossips a ProjectAnnouncement carrying a
        // real BlobTicket; node_b ingests it through the production handler
        // (which seeds node_a's addr from the ticket and stores node_a's
        // node_id on the direct entry), then aggregates /browse. The card
        // must flip Unknown -> Reachable, proving the freshness probe dials
        // the hosting node_id — not the per-app project_id (blake3(name)).
        use nexus_shell_daemon_core::browse::{BrowseAggregator, BrowseStatus, DEFAULT_PROBE_TTL};
        use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;

        let node_a = nexus_core_rs::create_node().await.unwrap(); // remote host
        let node_b = nexus_core_rs::create_node().await.unwrap(); // runs /browse

        // node_a mints a real blob + ticket (the ticket carries a_addr).
        let blobs_a = nexus_core_rs::BlobsClient::new(node_a.blobs_store());
        let hash = blobs_a.add_bytes(b"zip-bytes".to_vec()).await.unwrap();
        let a_addr = nexus_core_rs::DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let ticket = iroh_blobs::ticket::BlobTicket::new(
            a_addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();

        // Per-app project_id is blake3(name) — DISTINCT from node_a's id.
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Remote E2E App"));
        assert_ne!(pid, node_a.node_id(), "project_id must differ from node_id");
        let ann = ProjectAnnouncement::new(
            node_a.node_id(),
            "Remote E2E App".into(),
            "tools".into(),
            "discovered over gossip".into(),
            vec![],
        )
        .with_project_id(pid.clone())
        .with_archive_ticket(ticket);

        let agg = std::sync::Arc::new(BrowseAggregator::with_durations(
            DEFAULT_PROBE_TTL,
            std::time::Duration::from_secs(5),
        ));
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));

        // Production ingest on node_b: seeds a_addr from the ticket and
        // stores node_a's node_id on the direct entry.
        super::handle_project_announcement(&agg, &db, &node_b, &ann.to_gossip_bytes().unwrap());

        let curator = CuratorRuntime::new(None);
        let out = agg.aggregate(&curator, &node_b).await;
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].status,
            BrowseStatus::Reachable,
            "a live gossiped remote app must probe Reachable end-to-end"
        );
        assert_eq!(out[0].project_id, pid, "per-app project_id preserved");
        assert!(out[0].last_probed_at.is_some());

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[test]
    fn normalize_outbox_payload_accepts_both_shapes() {
        // Sprint 75 Phase A: the outbox stores unwrapped payloads, but a node
        // upgraded mid-flight may still hold pre-S75 PoW-wrapped entries. Both
        // normalize to the same announcement bytes; junk normalizes to None.
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Norm App"));
        let ann = ProjectAnnouncement::new(
            "b".repeat(64),
            "Norm App".into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(pid);
        let payload = ann.to_gossip_bytes().unwrap();

        // New shape: the stored bytes ARE the payload.
        let n1 = super::normalize_outbox_payload(&payload).expect("unwrapped payload normalizes");
        assert_eq!(n1, payload);

        // Legacy shape: a PoW-wrapped envelope of the same payload.
        let kp = nexus_core_rs::KeyPair::generate();
        let policy = nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        };
        let proof = nexus_core_rs::PowSolveCache::new()
            .ensure_proof([3u8; 32], &kp, &policy)
            .unwrap();
        let wrapped = nexus_core_rs::PowEnvelope::encode(&proof, &payload).unwrap();
        let n2 =
            super::normalize_outbox_payload(&wrapped).expect("legacy wrapped entry normalizes");
        assert_eq!(
            n2, payload,
            "legacy wrapped entry unwraps to the same payload"
        );

        // Junk normalizes to None (never re-broadcast garbage).
        assert!(super::normalize_outbox_payload(b"not an announcement").is_none());
    }

    #[tokio::test]
    async fn replay_restamps_pow_so_a_fresh_receiver_accepts() {
        // FIX-A core: an OWN outbox payload, replayed, yields a FRESH PoW envelope
        // a receiver verifies at "now" — the cure for the live "PoW proof too old"
        // bug where the verbatim replay shipped a >30-min-old proof. The unwrapped
        // payload is NOT itself a verifiable envelope; the replay re-wraps it.
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        use std::sync::{Arc, RwLock};
        let node = nexus_core_rs::create_node().await.unwrap();
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Replay App"));
        let ann = ProjectAnnouncement::new(
            node.node_id(),
            "Replay App".into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(pid.clone());
        let payload = ann.to_gossip_bytes().unwrap();

        let kp = Arc::new(nexus_core_rs::KeyPair::generate());
        let policy = Arc::new(RwLock::new(nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        }));
        let cache = Arc::new(nexus_core_rs::PowSolveCache::new());

        let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let fresh = super::remint_and_wrap_for_replay(
            &node, &cache, &policy, &kp, &[9u8; 32], &addr, &payload,
        )
        .await
        .expect("replay produces a fresh envelope");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let pol = policy.read().unwrap().clone();
        let (_proof, out_payload) = nexus_core_rs::PowVerifyCache::new()
            .verify_envelope(&fresh, &pol, now)
            .expect("a fresh receiver accepts the re-stamped proof at now");
        let out = ProjectAnnouncement::from_gossip_bytes(out_payload).unwrap();
        assert_eq!(out.project_id, pid);
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn replay_remints_own_ticket_to_current_address() {
        // FIX-A address half (positive control): replaying an OWN announcement whose
        // ticket carries a STALE address (here a second node's) re-mints it to OUR
        // current address, preserving the content hash. SENSITIVE: a regression that
        // stopped re-minting would leave the stale ticket and this test would fail.
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        use std::str::FromStr;
        use std::sync::{Arc, RwLock};
        let node = nexus_core_rs::create_node().await.unwrap();
        let other = nexus_core_rs::create_node().await.unwrap();
        // We hold the blob (so the re-mint succeeds), but the stored ticket points at
        // `other`'s address — a stale snapshot the replay must overwrite.
        let hash = nexus_core_rs::BlobsClient::new(node.blobs_store())
            .add_bytes(b"zip".to_vec())
            .await
            .unwrap();
        let stale_addr = nexus_core_rs::DiscoveryClient::new(other.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let stale_ticket = iroh_blobs::ticket::BlobTicket::new(
            stale_addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();
        let ann = ProjectAnnouncement::new(
            node.node_id(),
            "Ticket App".into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(hex::encode(nexus_core_rs::crypto::blake3_hash(
            b"Ticket App",
        )))
        .with_archive_ticket(stale_ticket.clone());
        let payload = ann.to_gossip_bytes().unwrap();

        let kp = Arc::new(nexus_core_rs::KeyPair::generate());
        let policy = Arc::new(RwLock::new(nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        }));
        let cache = Arc::new(nexus_core_rs::PowSolveCache::new());
        let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let fresh = super::remint_and_wrap_for_replay(
            &node, &cache, &policy, &kp, &[5u8; 32], &addr, &payload,
        )
        .await
        .expect("replay envelope");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let pol = policy.read().unwrap().clone();
        let (_proof, out_payload) = nexus_core_rs::PowVerifyCache::new()
            .verify_envelope(&fresh, &pol, now)
            .expect("verify");
        let out = ProjectAnnouncement::from_gossip_bytes(out_payload).unwrap();
        let new_ticket = out.archive_ticket.expect("ticket present after replay");
        assert_ne!(
            new_ticket, stale_ticket,
            "an OWN ticket is re-minted from the current address, not left stale"
        );
        let stale_hash = iroh_blobs::ticket::BlobTicket::from_str(&stale_ticket)
            .unwrap()
            .into_parts()
            .1;
        let new_hash = iroh_blobs::ticket::BlobTicket::from_str(&new_ticket)
            .unwrap()
            .into_parts()
            .1;
        assert_eq!(new_hash, stale_hash, "re-mint preserves the content hash");
        node.shutdown().await.ok();
        other.shutdown().await.ok();
    }

    #[tokio::test]
    async fn endpoint_addr_hoisted_once_per_pass() {
        // Sprint 76 Phase B (B5, WS-3/PD-5 hoisting): `remint_and_wrap_for_replay`
        // re-mints from the PRE-FETCHED pass address it is GIVEN, never a fresh
        // per-entry `my_endpoint_addr()` query. Proof: hand it `other`'s address
        // and assert the re-minted OWN ticket embeds OTHER's endpoint id — a
        // per-entry re-fetch would have stamped our OWN id instead. This is what
        // makes the once-per-pass hoist correct: the replay loop fetches the
        // address once (`current_replay_addr`) and threads it through every entry.
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        use std::str::FromStr;
        use std::sync::{Arc, RwLock};
        let node = nexus_core_rs::create_node().await.unwrap();
        let other = nexus_core_rs::create_node().await.unwrap();
        // We hold the blob (so the re-mint succeeds) and the announcement is OURS.
        let hash = nexus_core_rs::BlobsClient::new(node.blobs_store())
            .add_bytes(b"zip".to_vec())
            .await
            .unwrap();
        let own_addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let stale_ticket = iroh_blobs::ticket::BlobTicket::new(
            own_addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();
        let ann = ProjectAnnouncement::new(
            node.node_id(),
            "Hoist App".into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(hex::encode(nexus_core_rs::crypto::blake3_hash(
            b"Hoist App",
        )))
        .with_archive_ticket(stale_ticket);
        let payload = ann.to_gossip_bytes().unwrap();

        // The address handed to the pass is OTHER's, not ours.
        let pass_addr = nexus_core_rs::DiscoveryClient::new(other.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let other_id = pass_addr.id.to_string();

        let kp = Arc::new(nexus_core_rs::KeyPair::generate());
        let policy = Arc::new(RwLock::new(nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        }));
        let cache = Arc::new(nexus_core_rs::PowSolveCache::new());
        let fresh = super::remint_and_wrap_for_replay(
            &node, &cache, &policy, &kp, &[8u8; 32], &pass_addr, &payload,
        )
        .await
        .expect("replay envelope");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let pol = policy.read().unwrap().clone();
        let (_proof, out_payload) = nexus_core_rs::PowVerifyCache::new()
            .verify_envelope(&fresh, &pol, now)
            .expect("verify");
        let out = ProjectAnnouncement::from_gossip_bytes(out_payload).unwrap();
        let new_ticket = out.archive_ticket.expect("ticket present after replay");
        let minted_id = iroh_blobs::ticket::BlobTicket::from_str(&new_ticket)
            .unwrap()
            .into_parts()
            .0
            .id
            .to_string();
        assert_eq!(
            minted_id, other_id,
            "the re-mint must embed the PASSED pass-address, proving it is not re-fetched per entry"
        );
        assert_ne!(
            minted_id,
            node.node_id(),
            "a per-entry re-fetch would have stamped our OWN id — the hoist prevents that"
        );
        node.shutdown().await.ok();
        other.shutdown().await.ok();
    }

    #[tokio::test]
    async fn replay_does_not_remint_a_third_party_address() {
        // Hijack guard (anti-recentralisation, defense-in-depth): the outbox is
        // OWN-only, but if a THIRD-PARTY announcement (different node_id) were ever
        // in it, replay must NOT re-point its ticket to our address. SENSITIVE: WE
        // also hold the blob, so removing the node_id guard would let the re-mint
        // succeed and rewrite the address — this test would then fail.
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        use std::sync::{Arc, RwLock};
        let node = nexus_core_rs::create_node().await.unwrap();
        let other = nexus_core_rs::create_node().await.unwrap();
        let bytes = b"zip2".to_vec();
        // Both nodes hold the blob, so a (wrongly) un-guarded re-mint on our node
        // WOULD succeed and rewrite the address to ours.
        let hash = nexus_core_rs::BlobsClient::new(node.blobs_store())
            .add_bytes(bytes.clone())
            .await
            .unwrap();
        nexus_core_rs::BlobsClient::new(other.blobs_store())
            .add_bytes(bytes)
            .await
            .unwrap();
        // The foreign ticket points at `other`'s address, under `other`'s node_id.
        let foreign_addr = nexus_core_rs::DiscoveryClient::new(other.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let foreign_ticket = iroh_blobs::ticket::BlobTicket::new(
            foreign_addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();
        let ann = ProjectAnnouncement::new(
            other.node_id(),
            "Foreign App".into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(hex::encode(nexus_core_rs::crypto::blake3_hash(
            b"Foreign App",
        )))
        .with_archive_ticket(foreign_ticket.clone());
        let payload = ann.to_gossip_bytes().unwrap();

        let kp = Arc::new(nexus_core_rs::KeyPair::generate());
        let policy = Arc::new(RwLock::new(nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        }));
        let cache = Arc::new(nexus_core_rs::PowSolveCache::new());
        let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let fresh = super::remint_and_wrap_for_replay(
            &node, &cache, &policy, &kp, &[6u8; 32], &addr, &payload,
        )
        .await
        .expect("replay envelope");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let pol = policy.read().unwrap().clone();
        let (_proof, out_payload) = nexus_core_rs::PowVerifyCache::new()
            .verify_envelope(&fresh, &pol, now)
            .expect("verify");
        let out = ProjectAnnouncement::from_gossip_bytes(out_payload).unwrap();
        assert_eq!(
            out.archive_ticket.as_deref(),
            Some(foreign_ticket.as_str()),
            "a third party's ticket address must NOT be re-minted (hijack guard)"
        );
        node.shutdown().await.ok();
        other.shutdown().await.ok();
    }

    /// Build a project announcement payload for OUR node carrying a ticket that
    /// points at `hash` — held or not depending on whether the caller stored it.
    async fn own_announcement_with_ticket_for_hash(
        node: &Node,
        name: &str,
        hash: iroh_blobs::Hash,
    ) -> Vec<u8> {
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let ticket = iroh_blobs::ticket::BlobTicket::new(addr, hash, iroh_blobs::BlobFormat::Raw)
            .to_string();
        ProjectAnnouncement::new(
            node.node_id(),
            name.into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(hex::encode(nexus_core_rs::crypto::blake3_hash(
            name.as_bytes(),
        )))
        .with_archive_ticket(ticket)
        .to_gossip_bytes()
        .unwrap()
    }

    #[tokio::test]
    async fn replay_drops_announcement_when_blob_is_gone() {
        // Production self-heal (reverses T3): an OWN announcement whose archive
        // blob is no longer held (GC'd / app retired) must NOT keep being
        // re-advertised — re-broadcasting it surfaced a dead card that failed to
        // fetch on open. `remint_and_wrap_for_replay` now returns None so the
        // node stops advertising what it cannot serve. A kept-online app is
        // pinned (skip-GC) so its blob is never GC'd and it is never dropped.
        use std::sync::{Arc, RwLock};
        let node = nexus_core_rs::create_node().await.unwrap();
        // A ticket to a hash that was NEVER stored = the GC'd-blob case.
        let absent_hash =
            iroh_blobs::Hash::from_bytes(nexus_core_rs::crypto::blake3_hash(b"never-stored-blob"));
        let payload = own_announcement_with_ticket_for_hash(&node, "GC App", absent_hash).await;

        let kp = Arc::new(nexus_core_rs::KeyPair::generate());
        let policy = Arc::new(RwLock::new(nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        }));
        let cache = Arc::new(nexus_core_rs::PowSolveCache::new());
        let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let dropped = super::remint_and_wrap_for_replay(
            &node, &cache, &policy, &kp, &[7u8; 32], &addr, &payload,
        )
        .await;
        assert!(
            dropped.is_none(),
            "an announcement whose archive blob is GC'd must be dropped from the replay"
        );
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn prune_stale_outbox_drops_gc_blob_and_rewrites_db() {
        // The boot prune removes outbox entries whose archive blob is gone (and
        // rewrites the persisted outbox), while KEEPING an entry whose blob is
        // still held. This is what makes a retired app self-clean from the node
        // and never reach a fresh peer.
        let node = nexus_core_rs::create_node().await.unwrap();
        let blobs = nexus_core_rs::BlobsClient::new(node.blobs_store());

        // Held app: store its blob first, so its ticket resolves.
        let held_hash = blobs.add_bytes(b"a-real-archive-blob").await.unwrap();
        let held = own_announcement_with_ticket_for_hash(
            &node,
            "Live App",
            iroh_blobs::Hash::from_bytes(held_hash),
        )
        .await;
        // Stale app: a ticket to a never-stored hash.
        let absent =
            iroh_blobs::Hash::from_bytes(nexus_core_rs::crypto::blake3_hash(b"gc-d-archive"));
        let stale = own_announcement_with_ticket_for_hash(&node, "Retired App", absent).await;

        // Per-entry serveable check.
        assert!(super::outbox_entry_is_serveable(&node, &held).await);
        assert!(!super::outbox_entry_is_serveable(&node, &stale).await);

        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        {
            let g = db.lock().unwrap();
            g.insert_outbox(&held).unwrap();
            g.insert_outbox(&stale).unwrap();
        }
        let mut outbox = db.lock().unwrap().load_outbox().unwrap();
        assert_eq!(outbox.len(), 2);

        let pruned = super::prune_stale_outbox(&node, &db, &mut outbox).await;
        assert_eq!(pruned, 1, "exactly the one stale entry is pruned");
        assert_eq!(outbox.len(), 1, "the held entry survives in memory");
        assert_eq!(outbox[0], held);
        // The DB was rewritten to the surviving set (durable prune).
        let persisted = db.lock().unwrap().load_outbox().unwrap();
        assert_eq!(
            persisted,
            vec![held],
            "the persisted outbox is rewritten to the live set"
        );

        node.shutdown().await.ok();
    }

    #[test]
    fn keep_online_gate_handles_unwrapped_payload() {
        // Sprint 75 Phase A hot path (T4): the outbox stores UNWRAPPED payloads, so
        // the OFF gate must suppress a disabled app fed as a raw payload, not only
        // the legacy wrapped envelope covered by keep_online_disabled_app_*.
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        use std::collections::HashSet;
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Unwrapped Disabled"));
        let payload = ProjectAnnouncement::new(
            "a".repeat(64),
            "Unwrapped Disabled".into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(pid.clone())
        .to_gossip_bytes()
        .unwrap();

        let mut disabled = HashSet::new();
        disabled.insert(pid);
        assert!(
            !super::keep_online_allows_rebroadcast(&payload, &disabled),
            "an unwrapped payload for a disabled app is suppressed"
        );
        let mut other = HashSet::new();
        other.insert("zz".repeat(32));
        assert!(
            super::keep_online_allows_rebroadcast(&payload, &other),
            "a different app disabled still replays this one"
        );
        assert!(
            super::keep_online_allows_rebroadcast(&payload, &HashSet::new()),
            "empty disabled set fast-path replays all"
        );
    }

    #[tokio::test]
    async fn browse_boot_restore_from_unwrapped_outbox_e2e() {
        // Sprint 75 Phase A steady-state (T5): the NEW on-disk shape is the
        // UNWRAPPED payload. Persist it through the real DB outbox, load + restore
        // exactly as boot does, and assert the card reappears (Reachable, indexed).
        use nexus_shell_daemon_core::browse::{BrowseAggregator, BrowseStatus};
        use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = nexus_core_rs::create_node().await.unwrap();
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Unwrapped Restored"));
        let payload = ProjectAnnouncement::new(
            node.node_id(),
            "Unwrapped Restored".into(),
            "tools".into(),
            "persisted unwrapped".into(),
            vec![],
        )
        .with_project_id(pid.clone())
        .to_gossip_bytes()
        .unwrap();

        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        // Persist the UNWRAPPED payload (the S75 shape) — not a PoW envelope.
        db.lock().unwrap().insert_outbox(&payload).unwrap();
        let outbox = db.lock().unwrap().load_outbox().unwrap();
        assert_eq!(outbox.len(), 1);

        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let restored = super::restore_browse_from_outbox(&agg, &db, &node, &outbox);
        assert_eq!(restored, 1, "the unwrapped payload restores one card");

        let curator = CuratorRuntime::new(None);
        let out = agg.aggregate(&curator, &node).await;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].project_name, "Unwrapped Restored");
        assert_eq!(out[0].project_id, pid);
        assert_eq!(out[0].status, BrowseStatus::Reachable);
        let (_results, total) =
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "Unwrapped", 20, 0).unwrap();
        assert_eq!(total, 1, "restored unwrapped app is findable by name");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn browse_boot_restore_repopulates_aggregator_from_outbox_e2e() {
        // Remediation #7 real-frontier test (PATTERNS §P57): a node's OWN
        // published app must reappear in its Browse after a daemon restart.
        // We persist a real PoW-wrapped ProjectAnnouncement through the
        // coordinator DB outbox, load it back exactly as boot does
        // (load_outbox), run the restore, and assert the aggregator now
        // carries the card with its real name — and, because the announcement
        // node_id is our own, Reachable. No mock at the persistence frontier:
        // real encode -> real DB round-trip -> real decode -> real ingest.
        use nexus_shell_daemon_core::browse::{BrowseAggregator, BrowseStatus};
        use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;

        let node = nexus_core_rs::create_node().await.unwrap();

        // Our own announcement, with a real ticket minted from our store.
        let blobs = nexus_core_rs::BlobsClient::new(node.blobs_store());
        let hash = blobs.add_bytes(b"zip-bytes".to_vec()).await.unwrap();
        let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let ticket = iroh_blobs::ticket::BlobTicket::new(
            addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"My Restored App"));
        let ann = ProjectAnnouncement::new(
            node.node_id(),
            "My Restored App".into(),
            "tools".into(),
            "persisted across reboot".into(),
            vec![],
        )
        .with_project_id(pid.clone())
        .with_archive_ticket(ticket);
        let payload = ann.to_gossip_bytes().unwrap();

        // Wrap in a real PoW envelope (difficulty 1 -> instant solve; the
        // restore decodes structurally without re-verifying, so the proof
        // difficulty is irrelevant to the path under test) and persist it the
        // way the daemon does.
        let kp = nexus_core_rs::KeyPair::generate();
        let policy = nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        };
        let solve = nexus_core_rs::PowSolveCache::new();
        let proof = solve.ensure_proof([7u8; 32], &kp, &policy).unwrap();
        let envelope = nexus_core_rs::PowEnvelope::encode(&proof, &payload).unwrap();

        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        db.lock().unwrap().insert_outbox(&envelope).unwrap();

        // Load the outbox exactly as boot does, then restore.
        let outbox = db.lock().unwrap().load_outbox().unwrap();
        assert_eq!(outbox.len(), 1, "outbox must persist the envelope");

        let agg = std::sync::Arc::new(BrowseAggregator::new());
        assert_eq!(
            agg.direct_entry_count(),
            0,
            "aggregator starts empty on a fresh boot"
        );
        let restored = super::restore_browse_from_outbox(&agg, &db, &node, &outbox);
        assert_eq!(
            restored, 1,
            "the persisted project announcement must be restored"
        );

        // The card is back, with its REAL name (not the empty feed op), and
        // Reachable because it is our own node.
        let curator = CuratorRuntime::new(None);
        let out = agg.aggregate(&curator, &node).await;
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].project_name, "My Restored App",
            "the real project_name is restored from the announcement"
        );
        assert_eq!(out[0].project_id, pid);
        assert_eq!(
            out[0].status,
            BrowseStatus::Reachable,
            "a node's own restored app must be Reachable (self-branch, no dial)"
        );

        // And the restore re-indexed the search corpus with the real name,
        // fixing the search-by-name gap for a node's own apps after a restart.
        let (results, total) =
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "Restored", 20, 0).unwrap();
        assert_eq!(total, 1, "restored app is findable by name");
        assert_eq!(results[0].project_id, pid);

        node.shutdown().await.ok();
    }

    #[test]
    fn keep_online_disabled_app_not_rebroadcast() {
        // Sprint 74 Phase D: the boot/NeighborUp re-broadcast gate. A real PoW
        // envelope for an app that the node turned OFF must be suppressed; every
        // other case replays (empty set, a different app disabled, undecodable
        // bytes). Per-app id keying is what makes a shared-blob OFF safe.
        use std::collections::HashSet;
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Disabled App"));
        let ann = publish::ProjectAnnouncement::new(
            "a".repeat(64),
            "Disabled App".into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(pid.clone());
        let payload = ann.to_gossip_bytes().unwrap();
        let kp = nexus_core_rs::KeyPair::generate();
        let policy = nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        };
        let proof = nexus_core_rs::PowSolveCache::new()
            .ensure_proof([7u8; 32], &kp, &policy)
            .unwrap();
        let envelope = nexus_core_rs::PowEnvelope::encode(&proof, &payload).unwrap();

        // Empty disabled set -> replay all (fast path, no decode).
        assert!(super::keep_online_allows_rebroadcast(
            &envelope,
            &HashSet::new()
        ));
        // This app disabled -> suppressed.
        let disabled: HashSet<String> = [pid.clone()].into_iter().collect();
        assert!(!super::keep_online_allows_rebroadcast(&envelope, &disabled));
        // A DIFFERENT app disabled -> this one still replays.
        let other: HashSet<String> = ["b".repeat(64)].into_iter().collect();
        assert!(super::keep_online_allows_rebroadcast(&envelope, &other));
        // Undecodable bytes -> never dropped on a decode hiccup.
        assert!(super::keep_online_allows_rebroadcast(
            b"not an envelope",
            &disabled
        ));
    }

    use nexus_shell_daemon_core::registry::write_running as raw_write_running;
    use tempfile::tempdir;

    fn mk_opts(root: &Path) -> DaemonStartOptions {
        let config_file = root.join("config.toml");
        let paths = ShellDaemonPaths::resolve(Some(config_file)).expect("resolve paths");
        DaemonStartOptions {
            paths,
            api_host: "127.0.0.1".to_string(),
            api_port: 0,
            daemon_version: "0.1.0-test".to_string(),
            curator: CuratorConfig::default(),
            seed: SeedConfig::default(),
            // Isolate the boot driver's directory-revision persistence in
            // the test root — never the developer's real ~/.sbfb.
            sbfb_home: Some(root.join(".sbfb")),
            identity_mode: nexus_core_rs::IdentityMode::Normal,
            cors_origins: vec![],
            web_root: None,
        }
    }

    #[tokio::test]
    async fn start_then_shutdown_roundtrip_cleans_up_running_json() {
        let tmp = tempdir().expect("tempdir");
        let opts = mk_opts(tmp.path());
        let running_json = opts.paths.running_json.clone();

        let rt = DaemonRuntime::start(opts).await.expect("start succeeds");
        assert!(
            running_json.exists(),
            "running.json must exist after a successful start"
        );
        assert_ne!(
            rt.bound_addr().port(),
            0,
            "bound_addr must resolve to a real ephemeral port"
        );
        // Curator runtime must be empty at boot.
        assert_eq!(rt.curator_runtime().known_list_count(), 0);

        rt.shutdown().await.expect("shutdown succeeds");
        assert!(
            !running_json.exists(),
            "running.json must be removed after a clean shutdown"
        );
    }

    #[tokio::test]
    async fn second_start_refuses_when_first_still_running() {
        // This unit test works **because** of the hyphen/
        // underscore normalization rule baked into
        // `is_process_alive`. Without it, the test binary
        // (`nexus_shell_daemon-<hash>.exe`) would not match the
        // production `EXPECTED_PROCESS_NAME = "nexus-shell-daemon"`
        // and the singleton enforcement would silently degrade
        // to a no-op inside cargo test. See the Sprint 7 Phase A
        // review note in `registry::EXPECTED_PROCESS_NAME`.
        let tmp = tempdir().expect("tempdir");
        let rt1 = DaemonRuntime::start(mk_opts(tmp.path()))
            .await
            .expect("first start");

        match DaemonRuntime::start(mk_opts(tmp.path())).await {
            Ok(_) => panic!("second start must fail while the first is alive"),
            Err(err) => {
                let message = format!("{err:#}");
                assert!(
                    message.contains("already running"),
                    "error should mention the singleton conflict, got: {message}"
                );
            }
        }

        rt1.shutdown().await.expect("shutdown first");
    }

    #[tokio::test]
    async fn start_overwrites_stale_running_json() {
        let tmp = tempdir().expect("tempdir");
        let opts = mk_opts(tmp.path());
        let running_json = opts.paths.running_json.clone();

        // Write a stale running.json with a pid that cannot
        // possibly be this test binary — pid 0 is either
        // invalid or the scheduler idle task on every platform.
        opts.paths.ensure_dirs().unwrap();
        let stale_state = nexus_shell_daemon_core::registry::RunningState {
            schema_version: 1,
            node_id: "0".repeat(64),
            api_host: "127.0.0.1".to_string(),
            api_port: 1,
            pid: 0,
            started_at: "2000-01-01T00:00:00Z".to_string(),
            daemon_version: "0.0.0-stale".to_string(),
        };
        raw_write_running(&stale_state, &running_json).unwrap();

        let rt = DaemonRuntime::start(opts).await.expect("overwrite stale");
        assert!(running_json.exists());
        rt.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn curator_runtime_persists_subscriptions_across_restart() {
        let tmp = tempdir().expect("tempdir");
        let subscriptions_path = {
            let opts = mk_opts(tmp.path());
            opts.paths.subscriptions_json.clone()
        };

        // First boot — subscribe then shutdown.
        let opts1 = mk_opts(tmp.path());
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        let kp = nexus_core_rs::KeyPair::generate();
        rt1.curator_runtime()
            .subscribe(&hex::encode(kp.public_bytes()))
            .unwrap();
        assert!(subscriptions_path.exists());
        rt1.shutdown().await.unwrap();

        // Second boot against the same fixture — the attention
        // set must be re-populated from the persistence file.
        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(rt2.curator_runtime().is_subscribed(&kp.public_bytes()));
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn auto_subscribe_default_curators_at_boot() {
        let tmp = tempdir().expect("tempdir");
        let kp = nexus_core_rs::KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        let mut opts = mk_opts(tmp.path());
        opts.curator.default_curators = vec![hex_key.clone()];

        let rt = DaemonRuntime::start(opts).await.unwrap();
        assert!(
            rt.curator_runtime().is_subscribed(&kp.public_bytes()),
            "default curator must be auto-subscribed at boot"
        );
        rt.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn auto_subscribe_is_idempotent() {
        let tmp = tempdir().expect("tempdir");
        let kp = nexus_core_rs::KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        // First boot — manually subscribe.
        let opts1 = mk_opts(tmp.path());
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        rt1.curator_runtime().subscribe(&hex_key).unwrap();
        rt1.shutdown().await.unwrap();

        // Second boot — same key in default_curators. Must not
        // double-subscribe or error.
        let mut opts2 = mk_opts(tmp.path());
        opts2.curator.default_curators = vec![hex_key.clone()];
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(rt2.curator_runtime().is_subscribed(&kp.public_bytes()));
        // Only one entry in the attention set, not two.
        assert_eq!(rt2.curator_runtime().subscribed_pubkeys_hex().len(), 1);
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_feed_subscribe_joinhandle_shutdown() {
        let tmp = tempdir().expect("tempdir");
        let opts = mk_opts(tmp.path());
        let rt = DaemonRuntime::start(opts).await.expect("start");
        assert!(
            rt.feed_handle.is_some(),
            "feed subscribe must be spawned at boot"
        );
        rt.shutdown()
            .await
            .expect("shutdown must join feed handle without leak");
    }

    /// 2026-06-05 platform-remediation #6 — the E2E network-execute
    /// anti-recurrence GATE.
    ///
    /// The systemic bug the remediation fights is "discovery vs service
    /// never reconciled, and every test mocks the frontier". This gate
    /// drives the FULL real path with ZERO frontier mock: a real
    /// `DaemonRuntime` (real loopback HTTP + auth + iroh node +
    /// dispatch_loop + result_sync + validator_loop + coordinator DB),
    /// a real `nexus-worker` Engine on a SEPARATE iroh node joined by a
    /// real invite ticket, a task submitted over real HTTP, and the
    /// result polled back over real HTTP. The only mock is the
    /// deterministic `StubBackend` LLM. If any frontier link breaks
    /// (HTTP, auth, iroh cross-node sync, worker claim, the result
    /// bridge, the DB, or retrieval) this fails — which the ~1866 green
    /// unit tests did NOT, because they each mocked one side.
    // multi_thread + serial(sbfb_env): two iroh nodes + the worker pump
    // each need the docs actor on a dedicated thread (P2-A-1), and the
    // test mutates process env (token + the local-worker toggle).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[serial_test::serial(sbfb_env)]
    async fn e2e_network_execute_gate_real_http_no_frontier_mock() {
        use nexus_worker_core::allowlist::{Allowlist, NewProject};
        use nexus_worker_core::config::{Engine as EngineCfg, WorkerConfig};
        use nexus_worker_core::consent::{ConsentConfig, ConsentLevel};
        use nexus_worker_core::engine::{Engine, EngineBoot};
        use nexus_worker_core::llm::StubBackend;
        use std::time::Duration;

        let tmp = tempdir().expect("tempdir");
        // Known bearer token via env (the daemon reads SBFB_AUTH_TOKEN
        // before any disk path), isolate the shell-daemon dir, and
        // disable the OS-process worker auto-spawn — this gate wires its
        // own in-process worker Engine so it stays hermetic (no reliance
        // on a built `nexus-worker` binary next to the test exe).
        let token = "a".repeat(64);
        unsafe {
            std::env::set_var(auth::AUTH_TOKEN_ENV, &token);
            std::env::set_var(
                nexus_shell_daemon_core::paths::NEXUS_GRID_ROOT_ENV,
                tmp.path(),
            );
            std::env::set_var("SBFB_NO_LOCAL_WORKER", "1");
        }

        let rt = DaemonRuntime::start(mk_opts(tmp.path()))
            .await
            .expect("daemon boots");
        let base = format!("http://127.0.0.1:{}", rt.bound_addr().port());
        let client = reqwest::Client::new();
        let hdr = auth::AUTH_HEADER;

        // 1. Mint a worker invite over real HTTP — it carries the write
        //    ticket for the daemon's project doc.
        let inv: serde_json::Value = client
            .post(format!("{base}/api/v1/invite/create"))
            .header(hdr, &token)
            .json(&serde_json::json!({"scope": "worker"}))
            .send()
            .await
            .expect("invite request")
            .json()
            .await
            .expect("invite json");
        let wire = inv["wire"].as_str().expect("invite carries a wire");
        let invite = nexus_worker_core::invite::Invite::decode(wire).expect("decode invite");
        let project_id = invite.payload.project_id.clone();
        let ticket = invite
            .payload
            .tasks_doc_ticket
            .clone()
            .expect("worker invite carries the project doc ticket");

        // 2. A real worker Engine on its OWN iroh node, joined by the
        //    ticket (the production cross-node join path).
        let allowlist = Allowlist::open_in_memory().expect("allowlist");
        allowlist
            .enroll(NewProject {
                id: project_id.clone(),
                name: "gate".into(),
                enabled: true,
                budget_joules: 0,
                tasks_doc_ticket: Some(ticket),
            })
            .expect("enroll");
        let sbfb_tmp = tempdir().expect("sbfb tmp");
        let mut consent = ConsentConfig::default_for("gate-worker");
        consent.level = ConsentLevel::All;
        consent
            .save_atomic(&sbfb_tmp.path().join("consent.json"))
            .expect("consent");
        let boot = EngineBoot {
            worker_config: WorkerConfig {
                engine: EngineCfg {
                    task_poll_interval_ms: 100,
                    max_concurrent_tasks: 1,
                    state_flush_secs: 5,
                },
                ..WorkerConfig::default()
            },
            keypair: nexus_core_rs::KeyPair::generate(),
            allowlist,
            data_dir: None,
            llm_override: Some(Box::new(StubBackend::new())),
            sbfb_home_override: Some(sbfb_tmp.path().to_path_buf()),
            rate_limit_policy_path_override: None,
        };
        let mut engine = Engine::new(boot).await.expect("worker engine boots");
        let w_stop = engine.take_shutdown_sender().expect("shutdown sender");
        let worker = tokio::spawn(async move { engine.run_until_shutdown().await });

        // 3. Submit a task over real HTTP.
        let sub: serde_json::Value = client
            .post(format!("{base}/api/v1/tasks/submit"))
            .header(hdr, &token)
            .json(&serde_json::json!({
                "project_id": project_id,
                "task_type": "inference",
                "prompt": "ping",
                "model": "llama3",
                "is_open_source": true,
            }))
            .send()
            .await
            .expect("submit request")
            .json()
            .await
            .expect("submit json");
        let task_id = sub["task"]["task_id"]
            .as_str()
            .expect("submit returns a task id")
            .to_string();

        // 4. Poll real GET /result until the worker's output has flowed
        //    back across iroh, through the result bridge + validator,
        //    into the DB, and out the retrieval endpoint.
        let mut result_text = None;
        for _ in 0..150 {
            let resp = client
                .get(format!("{base}/api/v1/tasks/{task_id}/result"))
                .header(hdr, &token)
                .send()
                .await
                .expect("result request");
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.expect("result json");
                if let Some(t) = body["result_text"].as_str() {
                    result_text = Some(t.to_string());
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(
            result_text.is_some(),
            "result_text must return over the real HTTP path within 30s — \
             the full submit -> dispatch -> iroh sync -> worker -> result \
             bridge -> DB -> retrieval frontier, with no mock"
        );

        let _ = w_stop.send(());
        let _ = worker.await;
        rt.shutdown().await.expect("daemon shutdown");
        unsafe {
            std::env::remove_var(auth::AUTH_TOKEN_ENV);
            std::env::remove_var(nexus_shell_daemon_core::paths::NEXUS_GRID_ROOT_ENV);
            std::env::remove_var("SBFB_NO_LOCAL_WORKER");
        }
    }

    #[test]
    fn jitter_bounds_are_within_range() {
        for _ in 0..200 {
            let d = jittered_republish_duration();
            assert!(
                d.as_secs() >= 30 && d.as_secs() <= 60,
                "jitter {d:?} out of [30s, 60s]"
            );
        }
    }

    #[tokio::test]
    async fn boot_storage_namespace_persistent_reopen() {
        let tmp = tempdir().expect("tempdir");
        let opts1 = mk_opts(tmp.path());
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        rt1.shutdown().await.unwrap();

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn boot_feed_namespace_persistent_reopen() {
        let tmp = tempdir().expect("tempdir");
        let opts1 = mk_opts(tmp.path());
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        assert!(rt1.feed_handle.is_some());
        rt1.shutdown().await.unwrap();

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(rt2.feed_handle.is_some());
        rt2.shutdown().await.unwrap();
    }

    // S81 Phase A2: the self-heal boundary is "Replica not found" (legitimate
    // absence -> recreate loudly) vs any other docs error (fail fast, M8 row
    // untouched). In iroh-docs 0.101 (re-verified at the S81 Phase B bump)
    // open_doc never returns Ok(None), so the absence path is exercised
    // through the Err(NotFound) discriminator.

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_storage_namespace_recreates_loud_on_absent_replica() {
        let node = nexus_core_rs::create_node().await.unwrap();
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs_client.author_create().await.unwrap();
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        // M8 row pointing at a namespace that was never created in this store
        // (DB carried over from another data dir) -> Err("Replica not found").
        let stale: [u8; 32] = [0xAB; 32];
        db.lock()
            .unwrap()
            .set_storage_namespace("sbfb-ideas", &stale, None)
            .unwrap();

        // Some(empty dir) pins the PROD branch of the Phase F guard: a
        // data dir WITHOUT the migration backup sibling must still
        // self-heal (the guard only refuses when the backup exists).
        let iroh_dir = tempfile::tempdir().unwrap();
        let state = boot_storage_namespace(
            &docs_client,
            &db,
            "sbfb-ideas",
            author,
            nexus_core_rs::IdentityMode::Normal,
            Some(iroh_dir.path()),
        )
        .await
        .expect("absent replica (NotFound) must self-heal, not fail fast");

        let row = db
            .lock()
            .unwrap()
            .get_storage_namespace("sbfb-ideas")
            .unwrap()
            .expect("M8 row must still exist");
        assert_ne!(
            row.namespace_id,
            stale.to_vec(),
            "stale pointer must be overwritten by the recreated namespace"
        );
        assert_eq!(row.namespace_id, state.doc.id().as_bytes().to_vec());
        node.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_feed_namespace_recreates_loud_on_absent_replica() {
        let node = nexus_core_rs::create_node().await.unwrap();
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs_client.author_create().await.unwrap();
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let feed_key = crate::feed_sync::FEED_NAMESPACE_KEY;
        let stale: [u8; 32] = [0xCD; 32];
        db.lock()
            .unwrap()
            .set_storage_namespace(feed_key, &stale, None)
            .unwrap();

        // Some(empty dir): prod-branch pin, mirror of the storage test.
        let iroh_dir = tempfile::tempdir().unwrap();
        let state = boot_feed_namespace(
            &docs_client,
            &db,
            author,
            nexus_core_rs::IdentityMode::Normal,
            Some(iroh_dir.path()),
        )
        .await
        .expect("absent replica (NotFound) must self-heal, not fail fast");

        let row = db
            .lock()
            .unwrap()
            .get_storage_namespace(feed_key)
            .unwrap()
            .expect("M8 row must still exist");
        assert_ne!(row.namespace_id, stale.to_vec());
        assert_eq!(row.namespace_id, state.doc.id().as_bytes().to_vec());
        node.shutdown().await.unwrap();
    }

    /// Sprint 81 Phase K — hermetic close of the T1(3) "self-heal NOT
    /// triggered" gap: until now only the env-gated real-store gate
    /// (`store_migration.rs`, tarball absent in CI) proved that a boot
    /// over a PRESENT namespace does not recreate it. This pins it in
    /// CI: a second boot over the same store + M8 row must return the
    /// SAME namespace id, row untouched. Composed with the hermetic
    /// core proof that the redb 2→4 migration preserves `namespaces-2`
    /// (`store_migration.rs::docs_store_with_legacy_tuple_tags_migrates_on_open`),
    /// this covers "self-heal not triggered across the migration
    /// window" without the gitignored tarball.
    ///
    /// Honesty note: the "second boot" here is a second CALL in the same
    /// process over the SAME open node/store — it proves the reuse
    /// branch, not a full daemon restart over a closed-and-reopened
    /// store (that lives in the live flip runbook paliers).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_storage_namespace_reuses_existing_namespace_without_self_heal() {
        let node = nexus_core_rs::create_node().await.unwrap();
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs_client.author_create().await.unwrap();
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let iroh_dir = tempfile::tempdir().unwrap();

        let first = boot_storage_namespace(
            &docs_client,
            &db,
            "sbfb-ideas",
            author,
            nexus_core_rs::IdentityMode::Normal,
            Some(iroh_dir.path()),
        )
        .await
        .expect("first boot creates the namespace");
        let created_id = first.doc.id().as_bytes().to_vec();

        let second = boot_storage_namespace(
            &docs_client,
            &db,
            "sbfb-ideas",
            author,
            nexus_core_rs::IdentityMode::Normal,
            Some(iroh_dir.path()),
        )
        .await
        .expect("second boot reopens, never fails");
        assert_eq!(
            second.doc.id().as_bytes().to_vec(),
            created_id,
            "a present namespace must be REUSED — any new id here is a \
             silently-triggered self-heal"
        );
        let row = db
            .lock()
            .unwrap()
            .get_storage_namespace("sbfb-ideas")
            .unwrap()
            .expect("M8 row present");
        assert_eq!(
            row.namespace_id, created_id,
            "the M8 row must be untouched by a reuse boot"
        );
        node.shutdown().await.unwrap();
    }

    /// Feed mirror of the reuse pin — the second A2 self-heal site.
    /// Same honesty note as the storage twin above: second CALL
    /// same-process over the same open store, not a real daemon restart.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_feed_namespace_reuses_existing_namespace_without_self_heal() {
        let node = nexus_core_rs::create_node().await.unwrap();
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs_client.author_create().await.unwrap();
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let iroh_dir = tempfile::tempdir().unwrap();

        let first = boot_feed_namespace(
            &docs_client,
            &db,
            author,
            nexus_core_rs::IdentityMode::Normal,
            Some(iroh_dir.path()),
        )
        .await
        .expect("first boot creates the feed namespace");
        let created_id = first.doc.id().as_bytes().to_vec();

        let second = boot_feed_namespace(
            &docs_client,
            &db,
            author,
            nexus_core_rs::IdentityMode::Normal,
            Some(iroh_dir.path()),
        )
        .await
        .expect("second boot reopens, never fails");
        assert_eq!(
            second.doc.id().as_bytes().to_vec(),
            created_id,
            "a present feed namespace must be REUSED, never self-healed anew"
        );
        let row = db
            .lock()
            .unwrap()
            .get_storage_namespace(crate::feed_sync::FEED_NAMESPACE_KEY)
            .unwrap()
            .expect("feed M8 row present");
        assert_eq!(row.namespace_id, created_id);
        node.shutdown().await.unwrap();
    }

    /// Sprint 81 Phase F: an absent replica while the redb migration
    /// backup sibling exists is an INTERRUPTED migration (crash between
    /// rename and persist -> fresh empty store), not a legitimate
    /// NotFound — the boot must refuse the silent recreate while the
    /// backup still holds the data. NB: the guard keys on the backup's
    /// PRESENCE alone (it cannot distinguish "interrupted" from "a
    /// successful migration whose backup was never cleaned up" — both
    /// refuse, the remedy line in the error covers both).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_storage_namespace_refuses_recreate_on_interrupted_migration() {
        let node = nexus_core_rs::create_node().await.unwrap();
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs_client.author_create().await.unwrap();
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let stale: [u8; 32] = [0xAB; 32];
        db.lock()
            .unwrap()
            .set_storage_namespace("sbfb-ideas", &stale, None)
            .unwrap();

        let iroh_dir = tempfile::tempdir().unwrap();
        std::fs::write(docs_migration_backup_path(iroh_dir.path()), b"backup").unwrap();

        let err = boot_storage_namespace(
            &docs_client,
            &db,
            "sbfb-ideas",
            author,
            nexus_core_rs::IdentityMode::Normal,
            Some(iroh_dir.path()),
        )
        .await
        .expect_err("absent replica + migration backup must refuse the recreate");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("interrupted redb migration"),
            "diagnosable interrupted-migration marker expected, got: {msg}"
        );
        let row = db
            .lock()
            .unwrap()
            .get_storage_namespace("sbfb-ideas")
            .unwrap()
            .expect("M8 row must still exist");
        assert_eq!(
            row.namespace_id,
            stale.to_vec(),
            "M8 row must be untouched when the recreate is refused"
        );
        node.shutdown().await.unwrap();
    }

    /// Sprint 81 Phase F: feed mirror of the interrupted-migration
    /// recreate guard (same boundary, sibling arm — never a silent
    /// sibling gap).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_feed_namespace_refuses_recreate_on_interrupted_migration() {
        let node = nexus_core_rs::create_node().await.unwrap();
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs_client.author_create().await.unwrap();
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let feed_key = crate::feed_sync::FEED_NAMESPACE_KEY;
        let stale: [u8; 32] = [0xCD; 32];
        db.lock()
            .unwrap()
            .set_storage_namespace(feed_key, &stale, None)
            .unwrap();

        let iroh_dir = tempfile::tempdir().unwrap();
        std::fs::write(docs_migration_backup_path(iroh_dir.path()), b"backup").unwrap();

        let err = boot_feed_namespace(
            &docs_client,
            &db,
            author,
            nexus_core_rs::IdentityMode::Normal,
            Some(iroh_dir.path()),
        )
        .await
        .expect_err("absent replica + migration backup must refuse the recreate");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("interrupted redb migration"),
            "diagnosable interrupted-migration marker expected, got: {msg}"
        );
        let row = db
            .lock()
            .unwrap()
            .get_storage_namespace(feed_key)
            .unwrap()
            .expect("M8 row must still exist");
        assert_eq!(
            row.namespace_id,
            stale.to_vec(),
            "M8 row must be untouched when the recreate is refused"
        );
        node.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_storage_namespace_fail_fast_on_docs_error() {
        let node = nexus_core_rs::create_node().await.unwrap();
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs_client.author_create().await.unwrap();
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        // Persist a VALID namespace, then kill the node so open_doc fails
        // with an actor/transport error that is NOT "Replica not found".
        let doc = docs_client.create_doc().await.unwrap();
        let valid_ns = doc.id().as_bytes().to_vec();
        db.lock()
            .unwrap()
            .set_storage_namespace("sbfb-ideas", &valid_ns, None)
            .unwrap();
        drop(doc);
        node.shutdown().await.unwrap();

        let err = boot_storage_namespace(
            &docs_client,
            &db,
            "sbfb-ideas",
            author,
            nexus_core_rs::IdentityMode::Normal,
            None,
        )
        .await
        .expect_err("docs error after shutdown must fail fast, not recreate");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("refusing to silently recreate"),
            "diagnosable fail-fast marker expected, got: {msg}"
        );
        let row = db
            .lock()
            .unwrap()
            .get_storage_namespace("sbfb-ideas")
            .unwrap()
            .expect("M8 row must still exist");
        assert_eq!(
            row.namespace_id, valid_ns,
            "M8 row must be untouched on fail-fast"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_feed_namespace_fail_fast_on_docs_error() {
        let node = nexus_core_rs::create_node().await.unwrap();
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs_client.author_create().await.unwrap();
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let feed_key = crate::feed_sync::FEED_NAMESPACE_KEY;
        let doc = docs_client.create_doc().await.unwrap();
        let valid_ns = doc.id().as_bytes().to_vec();
        db.lock()
            .unwrap()
            .set_storage_namespace(feed_key, &valid_ns, None)
            .unwrap();
        drop(doc);
        node.shutdown().await.unwrap();

        let err = boot_feed_namespace(
            &docs_client,
            &db,
            author,
            nexus_core_rs::IdentityMode::Normal,
            None,
        )
        .await
        .expect_err("docs error after shutdown must fail fast, not recreate");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("refusing to silently recreate"),
            "diagnosable fail-fast marker expected, got: {msg}"
        );
        let row = db
            .lock()
            .unwrap()
            .get_storage_namespace(feed_key)
            .unwrap()
            .expect("M8 row must still exist");
        assert_eq!(
            row.namespace_id, valid_ns,
            "M8 row must be untouched on fail-fast"
        );
    }

    #[tokio::test]
    async fn test_feed_republish_at_boot() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        rt1.shutdown().await.unwrap();

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let kp = nexus_core_rs::KeyPair::generate();
            let pubkey = hex::encode(kp.public_bytes());
            let op = serde_json::json!({
                "type": "ReleasePublished",
                "project_id": "test-republish",
                "version": "1.0.0"
            });
            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
                kp.sign(data).to_vec()
            })
            .expect("insert feed op");
            let entries = nexus_coordinator_rs::public_feed::replay_all(&db).unwrap();
            assert_eq!(entries.len(), 1, "SQLite must have 1 entry before reboot");
        }

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(
            rt2.feed_handle.is_some(),
            "feed subscribe must be active after republish boot"
        );
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_feed_join_handles_tracked_and_shutdown() {
        let tmp = tempdir().expect("tempdir");
        let opts = mk_opts(tmp.path());
        let rt = DaemonRuntime::start(opts).await.unwrap();
        assert!(
            rt.feed_join_handles.is_some(),
            "feed_join_handles must be initialized at boot"
        );
        assert!(
            rt.feed_join_shutdown.is_some(),
            "feed_join_shutdown must be initialized at boot"
        );
        let handles = rt.feed_join_handles.as_ref().unwrap();
        assert_eq!(
            handles.lock().unwrap().len(),
            0,
            "no feed_join calls yet, Vec must be empty"
        );
        rt.shutdown()
            .await
            .expect("shutdown must join feed_join handles without leak");
    }

    #[tokio::test]
    async fn test_orphan_republish_recovery() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        rt1.shutdown().await.unwrap();

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let kp = nexus_core_rs::KeyPair::generate();
            let pubkey = hex::encode(kp.public_bytes());
            let op = serde_json::json!({
                "type": "ReleasePublished",
                "project_id": "test-orphan",
                "version": "2.0.0"
            });
            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
                kp.sign(data).to_vec()
            })
            .expect("insert feed op");
            assert_eq!(
                db.count_feed_entries().unwrap(),
                1,
                "SQLite must have 1 entry before recovery boot"
            );
        }

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(
            rt2.feed_handle.is_some(),
            "feed sync must be active after orphan recovery"
        );
        rt2.shutdown().await.unwrap();

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open DB");
            assert_eq!(
                db.count_feed_entries().unwrap(),
                1,
                "feed entry must survive recovery boot without data loss"
            );
        }

        let opts3 = mk_opts(tmp.path());
        let rt3 = DaemonRuntime::start(opts3).await.unwrap();
        assert!(
            rt3.feed_handle.is_some(),
            "feed sync must remain active after second boot"
        );
        rt3.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_key_rotation_persistence_survives_reboot() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        assert_eq!(
            rt1.revocation_cache().read().unwrap().len(),
            0,
            "no rotations before insert"
        );
        rt1.shutdown().await.unwrap();

        let kp = nexus_core_rs::KeyPair::generate();
        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let new_kp = nexus_core_rs::KeyPair::generate();
            db.insert_key_rotation(
                &hex::encode(kp.public_bytes()),
                &hex::encode(new_kp.public_bytes()),
                1_700_000_000,
                7,
                "test_sig",
                "test reason",
            )
            .expect("insert key rotation");
        }

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert_eq!(
            rt2.revocation_cache().read().unwrap().len(),
            1,
            "RevocationCache must contain the persisted rotation after reboot"
        );
        assert!(
            rt2.revocation_cache()
                .read()
                .unwrap()
                .is_in_transition(&kp.public_bytes(), 1_700_000_000),
            "old key must be in transition after restore"
        );
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_e2e_restart_full_cycle() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();

        let node_id_1 = rt1.node.as_ref().unwrap().node_id();

        let kp = nexus_core_rs::KeyPair::generate();
        rt1.curator_runtime()
            .subscribe(&hex::encode(kp.public_bytes()))
            .unwrap();

        let blobs = nexus_core_rs::BlobsClient::new(rt1.node.as_ref().unwrap().blobs_store());
        let blob_hash = blobs.add_bytes(b"e2e-restart-payload").await.unwrap();
        let retrieved = blobs.get_bytes(blob_hash).await.unwrap();
        assert_eq!(retrieved, b"e2e-restart-payload");

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let feed_kp = nexus_core_rs::KeyPair::generate();
            let pubkey = hex::encode(feed_kp.public_bytes());
            let op = serde_json::json!({
                "type": "ReleasePublished",
                "project_id": "e2e-restart-proj",
                "version": "1.0.0"
            });
            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
                feed_kp.sign(data).to_vec()
            })
            .expect("insert feed op");
        }

        rt1.shutdown().await.unwrap();

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();

        let node_id_2 = rt2.node.as_ref().unwrap().node_id();
        assert_eq!(
            node_id_1, node_id_2,
            "node_id must be identical across restart (persistent identity)"
        );

        assert!(
            rt2.curator_runtime().is_subscribed(&kp.public_bytes()),
            "curator subscription must survive restart"
        );

        let blobs2 = nexus_core_rs::BlobsClient::new(rt2.node.as_ref().unwrap().blobs_store());
        let data2 = blobs2.get_bytes(blob_hash).await.unwrap();
        assert_eq!(
            data2, b"e2e-restart-payload",
            "blob must survive restart (FsStore persistence)"
        );

        assert!(
            rt2.feed_handle.is_some(),
            "feed sync must be active after restart"
        );

        {
            let db =
                nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open DB post-boot");
            let entries = nexus_coordinator_rs::public_feed::replay_all(&db).unwrap();
            assert_eq!(
                entries.len(),
                1,
                "feed entry must survive restart in SQLite"
            );
        }

        assert_eq!(
            rt2.revocation_cache().read().unwrap().len(),
            0,
            "revocation cache must be initialized (empty, no rotations inserted)"
        );

        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_e2e_crash_recovery() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let running_json = opts1.paths.running_json.clone();
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let kp = nexus_core_rs::KeyPair::generate();
            let pubkey = hex::encode(kp.public_bytes());
            let op = serde_json::json!({
                "type": "ReleasePublished",
                "project_id": "crash-recovery",
                "version": "1.0.0"
            });
            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
                kp.sign(data).to_vec()
            })
            .expect("insert feed op");
        }

        rt1.shutdown().await.unwrap();

        // Simulate crash aftermath: a stale running.json left behind
        // by a process that died before cleanup. The singleton check
        // must detect the stale marker and proceed.
        let stale = nexus_shell_daemon_core::registry::RunningState {
            schema_version: 1,
            node_id: "0".repeat(64),
            api_host: "127.0.0.1".to_string(),
            api_port: 1,
            pid: 0,
            started_at: "2000-01-01T00:00:00Z".to_string(),
            daemon_version: "0.0.0-crashed".to_string(),
        };
        raw_write_running(&stale, &running_json).unwrap();
        assert!(running_json.exists(), "stale running.json must be present");

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();

        assert!(
            rt2.feed_handle.is_some(),
            "feed sync must recover after crash"
        );

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path)
                .expect("open DB post-crash");
            let entries = nexus_coordinator_rs::public_feed::replay_all(&db).unwrap();
            assert_eq!(
                entries.len(),
                1,
                "feed entry must survive crash (SQLite WAL + FULL pragma)"
            );
        }

        rt2.shutdown().await.unwrap();
        assert!(
            !running_json.exists(),
            "running.json must be cleaned after recovery"
        );
    }
}
