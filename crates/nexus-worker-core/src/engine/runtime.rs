// SPDX-License-Identifier: AGPL-3.0-or-later
//! Async engine runtime: the loop that drives the state
//! machine and talks to all the other worker-core modules.
//!
//! This is the piece that turns the collection of typed, loosely
//! coupled modules (config, allowlist, gpu, ollama, invite,
//! state machine) into a running daemon. It is deliberately
//! `Send + 'static` so the `nexus-worker` binary can spawn it
//! on any tokio runtime and so the W10 TUI layer can observe
//! state from a different task.
//!
//! ## Scope — Sprint 3 W9 boot shell + Sprint 4 Phase D task pump
//!
//! Sprint 3 W9 landed the boot path and the loop shell:
//!
//! - Construct the engine from a fully-populated `EngineBoot`
//! - Boot the iroh [`Node`] with the persistent worker keypair
//! - Run an initial Ollama health-check and GPU probe (both
//!   logged, neither fatal)
//! - Apply `Start` to the state machine, broadcasting the
//!   resulting `WorkerState` on a `tokio::sync::watch` channel
//! - Iterate the allowlist on every poll tick and log enrolled
//!   projects (so the TUI has something real to show)
//! - Handle graceful shutdown via a `oneshot` channel
//!
//! Sprint 4 Phase D replaced the placeholder TODO in the
//! Processing branch with the real task pump in
//! [`Engine::scan_and_execute_tasks`]. The pump imports
//! coordinator docs (via the `tasks_doc_ticket` field on invite
//! v2 or a test injection), scans `task:*` entries, verifies the
//! coordinator signature, signs and writes a [`ClaimEntry`],
//! calls [`LlmBackend::generate`], and writes the signed
//! [`ResultEntry`] back.
//!
//! ## Channels
//!
//! The engine exposes two channels:
//!
//! - `state_rx: watch::Receiver<WorkerState>` — the current
//!   state, updated every transition. W10 subscribes.
//! - `shutdown_tx: oneshot::Sender<()>` — signal graceful stop.
//!   The `nexus-worker` binary wires this to a SIGINT handler.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use nexus_core_rs::docs::{DocHandle, DocsAuthorId, DocsClient, DocsTicket};
use nexus_core_rs::task::{
    Claim, ClaimEntry, ResultEntry, ResultPayload, TASK_FORMAT_VERSION, Task, TaskEntry,
};
use nexus_core_rs::{BlobsClient, KeyPair, Node, NodeConfig, blake3_hash, create_node_with_config};
use tokio::sync::{Mutex, oneshot, watch};
use tracing::{debug, error, info, warn};

use crate::allowlist::Allowlist;
use crate::config::{WatermarkConfig, WorkerConfig};
use crate::consent::{self, AllowOutcome, ConsentWatcher, RejectReason, TaskContext, UsageTracker};
use crate::engine::state::{StateMachine, WorkerEvent, WorkerState};
use crate::engine::state_writer::{self, LastTask, SnapshotInputs};
use crate::ephemeral::{EphemeralLifecycle, LifecycleState};
use crate::gpu::{GpuInfo, GpuMonitor, create_monitor};
use crate::llm::factory::build_backend;
use crate::llm::{GenerateParams, HealthCheck, LlmBackend};
use crate::paths::worker_state_file;
use crate::rate_limit::{RateKey, RateLimitError, RateLimiter};
use crate::rate_limit_policy_loader::RateLimitPolicyWatcher;

// =================================================================
// Boot-time configuration
// =================================================================

/// Everything the engine needs to boot.
///
/// Constructed by the `nexus-worker` binary (which has already
/// loaded `worker.toml`, resolved [`crate::config::WorkerPaths`],
/// and loaded the persistent Ed25519 keypair from disk) and
/// passed to [`Engine::new`].
///
/// Sprint 4 Phase D W9.1 fields (renamed Sprint 20 Phase D):
/// - `data_dir`: when Some, passed to [`NodeConfig::with_data_dir`]
///   so the worker's iroh-docs replica and default author survive
///   process restarts. The W9.1 task flow stores imported
///   coordinator docs through this same store.
/// - `llm_override`: when Some, replaces the backend built from
///   `worker_config.llm` (which pre-S20 was called `ollama`). The
///   nexus-worker binary uses this to wire
///   [`crate::llm::StubBackend`] when the operator passes
///   `--stub-llm` for hermetic e2e runs.
pub struct EngineBoot {
    pub worker_config: WorkerConfig,
    pub keypair: KeyPair,
    pub allowlist: Allowlist,
    pub data_dir: Option<PathBuf>,
    pub llm_override: Option<Box<dyn LlmBackend>>,
    /// Sprint 16 Phase C: override for the `~/.sbfb/` root the
    /// engine uses to locate `consent.json` and `usage.json`.
    /// `None` (the prod default) resolves to
    /// [`consent::sbfb_home`] which honours the `SBFB_HOME` env
    /// var. Integration tests pass `Some(tempdir)` so they do
    /// not touch the developer's real home dir.
    pub sbfb_home_override: Option<PathBuf>,
    /// Sprint 22 Phase A: override for the rate-limit policy file
    /// path. `None` (the prod default) resolves to
    /// `<sbfb_home>/rate_limit_policy.toml`. Integration tests set
    /// this to a tempdir file so they can (a) exercise the engine
    /// gate against a pre-seeded policy without touching the real
    /// `~/.sbfb/` and (b) rewrite the file to trigger hot-reload
    /// without racing the operator's config.
    pub rate_limit_policy_path_override: Option<PathBuf>,
}

impl EngineBoot {
    /// Convenience constructor that mirrors the Sprint 3 W9 shape
    /// (no data_dir, default backend from config) so existing
    /// callers keep compiling after the Phase D struct extension.
    pub fn new(worker_config: WorkerConfig, keypair: KeyPair, allowlist: Allowlist) -> Self {
        Self {
            worker_config,
            keypair,
            allowlist,
            data_dir: None,
            llm_override: None,
            sbfb_home_override: None,
            rate_limit_policy_path_override: None,
        }
    }
}

// =================================================================
// Engine
// =================================================================

/// A running worker engine.
///
/// Owns the iroh [`Node`], the state machine, the Ollama client,
/// the GPU monitor, and the allowlist. Callers construct one
/// via [`Engine::new`] and drive it with
/// [`Engine::run_until_shutdown`]. Use [`Engine::state_rx`] to
/// subscribe the TUI / telemetry to live state updates.
pub struct Engine {
    node: Node,
    state: Arc<Mutex<StateMachine>>,
    state_tx: watch::Sender<WorkerState>,
    state_rx: watch::Receiver<WorkerState>,
    allowlist: Allowlist,
    llm: Box<dyn LlmBackend>,
    gpu: Box<dyn GpuMonitor>,
    gpu_info: Vec<GpuInfo>,
    worker_config: WorkerConfig,
    shutdown_rx: Option<oneshot::Receiver<()>>,
    shutdown_tx_handle: Option<oneshot::Sender<()>>,
    /// Sprint 4 Phase D W9.1 fields.
    keypair: KeyPair,
    /// Docs the engine watches for `task:*` entries, keyed by
    /// project_id. Populated at boot from every allowlist entry
    /// that has a non-None `tasks_doc_ticket`, plus tests can
    /// inject docs directly via [`Engine::register_task_doc`].
    task_docs: HashMap<String, DocHandle>,
    /// Default author id on this worker's local docs store,
    /// used to write `claim:*` and `result:*` entries.
    worker_author: Option<DocsAuthorId>,
    /// task_ids already processed in this process instance,
    /// used to dedupe within a single tick and across ticks.
    completed_task_ids: HashSet<String>,
    /// Sprint 5 Phase A: wall-clock time the engine was
    /// constructed, used by the state_writer to compute
    /// `uptime_secs` and `started_at` for the shell snapshot.
    boot_time: SystemTime,
    /// Sprint 5 Phase A: most recent task observed in the
    /// completion path, reported in the shell snapshot so the
    /// user sees a "Last task" card in /my-network. Updated
    /// every time `scan_and_execute_tasks` writes a result.
    last_task: Option<LastTask>,
    /// Sprint 5 Phase A: override for the state flush destination,
    /// used by integration tests that cannot write to the default
    /// `~/.nexus-grid/worker/state.json` path. `None` means use
    /// [`crate::paths::worker_state_file`] at flush time.
    state_flush_path_override: Option<PathBuf>,
    /// Sprint 16 Phase C: live view of the user's consent
    /// preferences. `None` means the `~/.sbfb/` root could not be
    /// resolved (CI sandbox) or the watcher thread failed to
    /// start — the engine logs a warning and skips the consent
    /// filter in that case (equivalent to "accept everything the
    /// allowlist already enrolled").
    consent: Option<ConsentWatcher>,
    /// Sprint 16 Phase C: per-day wall-clock usage tracker
    /// updated after every completed task. `None` when the root
    /// is unresolvable; the consent filter then skips the hours
    /// cap but still enforces level / watts / vram.
    usage: Option<Arc<Mutex<UsageTracker>>>,
    /// Sprint 22 Phase A : worker-engine rate-limit gate. Invoked
    /// just before claim sign / broadcast in
    /// [`Engine::scan_and_execute_tasks`] against the tuple
    /// `(task_entry.author_pubkey, self.keypair.public, task.model)`.
    /// Wrapped in `Arc` so the policy watcher callback (held by
    /// `_rate_limit_watcher` below) can keep a cheap clone across
    /// hot-reload events.
    rate_limiter: Arc<RateLimiter>,
    /// Sprint 22 Phase A : watcher thread backing
    /// [`Self::rate_limiter`] hot-reload. Held in the struct so its
    /// `Drop` runs at engine shutdown, joining the background
    /// reload thread. `None` when `~/.sbfb/` could not be resolved
    /// — the engine then runs against the in-memory default
    /// policy (60 req/min, no overrides) and never picks up operator
    /// edits, which matches the consent-filter-disabled fallback
    /// pattern from Sprint 16 Phase C.
    _rate_limit_watcher: Option<RateLimitPolicyWatcher>,
    /// Sprint 23 Phase B : ephemeral worker lifecycle tracker.
    /// Counts completed tasks, triggers VRAM wipe between tasks,
    /// and signals process exit when `max_tasks` is reached.
    ephemeral: EphemeralLifecycle,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("state", &self.state_rx.borrow())
            .field("gpu_count", &self.gpu_info.len())
            .finish()
    }
}

impl Engine {
    /// Boot the engine.
    ///
    /// This is the full init path: builds the iroh Node with the
    /// persistent secret key, constructs the Ollama HTTP client,
    /// creates the GPU monitor, and runs an initial healthcheck
    /// plus GPU probe. Both are logged, neither is fatal: the
    /// engine still boots even if Ollama is down or no GPU is
    /// visible.
    ///
    /// The state machine is initialized in `Idle` and the boot
    /// sequence does NOT apply the `Start` event. Callers must
    /// drive [`Engine::run_until_shutdown`] to transition into
    /// `Connecting` / `Processing`.
    pub async fn new(boot: EngineBoot) -> anyhow::Result<Self> {
        let EngineBoot {
            worker_config,
            keypair,
            allowlist,
            data_dir,
            llm_override,
            sbfb_home_override,
            rate_limit_policy_path_override,
        } = boot;

        info!(
            worker = %worker_config.identity.name,
            "booting nexus-worker engine"
        );

        // --- iroh Node ---
        let mut node_cfg = NodeConfig::default().with_secret_key(keypair.secret_bytes());
        if let Some(dir) = data_dir.as_ref() {
            node_cfg = node_cfg.with_data_dir(dir.clone());
        }
        let node = create_node_with_config(node_cfg)
            .await
            .map_err(|e| anyhow::anyhow!("failed to boot iroh node for worker keypair: {e}"))?;
        info!(node_id = %node.node_id(), "iroh endpoint ready");

        // --- LLM backend (Sprint 20 Phase D : dual-backend) ---
        let llm: Box<dyn LlmBackend> = match llm_override {
            Some(stub) => {
                info!("using injected LlmBackend override (stub mode)");
                stub
            }
            None => build_backend(&worker_config.llm)
                .map_err(|e| anyhow::anyhow!("failed to build LLM backend: {e}"))?,
        };
        match llm.healthcheck().await {
            HealthCheck::Ready { models } => {
                info!(
                    backend = ?worker_config.llm.backend,
                    model_count = models.len(),
                    "llm healthcheck passed"
                );
            }
            HealthCheck::NotRunning { endpoint, hint, .. } => {
                warn!(
                    %endpoint,
                    %hint,
                    "llm backend is not running; engine will continue but cannot serve tasks until it comes up"
                );
            }
            HealthCheck::Error { endpoint, reason } => {
                warn!(%endpoint, %reason, "llm healthcheck returned an error");
            }
        }

        // --- GPU ---
        let gpu: Box<dyn GpuMonitor> = create_monitor();
        let gpu_info = match gpu.probe() {
            Ok(infos) => {
                if infos.is_empty() {
                    info!("gpu probe: no devices visible (CPU-only mode)");
                } else {
                    for info in &infos {
                        info!(
                            index = info.index,
                            name = %info.name,
                            backend = %info.backend,
                            vram_gb = info.vram_total_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
                            "gpu device discovered"
                        );
                    }
                }
                infos
            }
            Err(e) => {
                warn!(error = %e, "gpu probe failed");
                Vec::new()
            }
        };

        let state = StateMachine::new();
        let initial = state.state().clone();
        let (state_tx, state_rx) = watch::channel(initial);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // --- Sprint 4 Phase D W9.1: worker author + imported task docs ---
        //
        // The engine writes claim:* and result:* entries under the
        // node's default author id. With data_dir set, iroh-docs
        // persists this author across reboots so every worker write
        // carries a stable identity.
        let docs_client = DocsClient::new(node.docs());
        let worker_author = match docs_client.author_default().await {
            Ok(author) => Some(author),
            Err(e) => {
                warn!(error = %e, "could not obtain default author; engine will skip claim/result writes");
                None
            }
        };

        // Import every allowlist project with a tasks_doc_ticket.
        // Failed imports are logged but non-fatal so a single broken
        // project can't prevent the engine from booting.
        let mut task_docs: HashMap<String, DocHandle> = HashMap::new();
        match allowlist.list_enabled() {
            Ok(projects) => {
                for p in projects {
                    let Some(ticket_str) = p.tasks_doc_ticket.as_ref() else {
                        continue;
                    };
                    match ticket_str.parse::<DocsTicket>() {
                        Ok(ticket) => match docs_client.import_ticket(ticket).await {
                            Ok(doc) => {
                                info!(
                                    project = %p.id,
                                    doc_id = %doc.id(),
                                    "imported project task doc",
                                );
                                task_docs.insert(p.id.clone(), doc);
                            }
                            Err(e) => {
                                warn!(
                                    project = %p.id,
                                    error = %e,
                                    "failed to import task doc; project will be skipped",
                                );
                            }
                        },
                        Err(e) => {
                            warn!(
                                project = %p.id,
                                error = %e,
                                "invalid tasks_doc_ticket; project will be skipped",
                            );
                        }
                    }
                }
            }
            Err(e) => warn!(error = %e, "list_enabled failed at boot"),
        }

        // --- Sprint 16 Phase C: consent + usage tracker ---
        //
        // Resolve `~/.sbfb/` (or the boot override). The worker
        // keeps booting even if the path is unresolvable or the
        // notify watcher thread fails — the engine just skips
        // the consent filter in that case and relies on the
        // existing allowlist SQLite enrollment for task
        // admission. A log line records the fallback so ops can
        // tell "consent filter silently off" from "filter on and
        // accepting everything".
        let sbfb_home = sbfb_home_override.or_else(consent::sbfb_home);
        let own_node_id_hex = hex::encode(keypair.public_bytes());
        let consent_handle = match sbfb_home.as_ref() {
            Some(root) => {
                match ConsentWatcher::spawn(root.join("consent.json"), &own_node_id_hex) {
                    Ok(w) => Some(w),
                    Err(e) => {
                        warn!(error = %e, "consent watcher failed to start; filter disabled");
                        None
                    }
                }
            }
            None => {
                warn!("cannot resolve ~/.sbfb/ root; consent filter disabled");
                None
            }
        };
        let usage_handle = match sbfb_home.as_ref() {
            Some(root) => match UsageTracker::load_or_default(root.join("usage.json")) {
                Ok(u) => Some(Arc::new(Mutex::new(u))),
                Err(e) => {
                    warn!(error = %e, "usage tracker failed to load; hours cap disabled");
                    None
                }
            },
            None => None,
        };

        // --- Sprint 22 Phase A: rate-limit engine gate + hot-reload ---
        //
        // Resolve the policy path : explicit override wins (tests),
        // else `<sbfb_home>/rate_limit_policy.toml`, else fall back
        // to in-memory default. The same fail-open pattern as the
        // consent watcher : a worker with no resolvable home dir
        // still boots, but its gate runs on the default policy
        // (60 req/min + burst x2) and cannot pick up operator edits.
        let rate_limit_path = rate_limit_policy_path_override.or_else(|| {
            sbfb_home
                .as_ref()
                .map(|root| root.join("rate_limit_policy.toml"))
        });
        let (rate_limiter, rate_limit_watcher) = match rate_limit_path {
            Some(path) => {
                // Build the rate limiter from the on-disk snapshot
                // (or default if the file does not yet exist) BEFORE
                // spawning the watcher, so `check` calls on the
                // first tick see a coherent state. The watcher's
                // `spawn_with_on_reload` then synchronously applies
                // the same snapshot via the callback and installs
                // the notify observer for subsequent edits.
                let rl_arc =
                    match crate::rate_limit_policy_loader::load_rate_limit_policy_from(&path) {
                        Ok(initial) => match RateLimiter::from_policy_value(initial) {
                            Ok(r) => Arc::new(r),
                            Err(e) => {
                                warn!(
                                    error = %e,
                                    "rate-limit policy invalid at boot; falling back to default"
                                );
                                Arc::new(
                                    RateLimiter::from_policy_value(
                                        crate::rate_limit::RateLimitPolicy::default(),
                                    )
                                    .expect("default policy must build"),
                                )
                            }
                        },
                        Err(e) => {
                            warn!(
                                error = %e,
                                path = %path.display(),
                                "rate-limit policy load failed at boot; falling back to default"
                            );
                            Arc::new(
                                RateLimiter::from_policy_value(
                                    crate::rate_limit::RateLimitPolicy::default(),
                                )
                                .expect("default policy must build"),
                            )
                        }
                    };
                let rl_for_callback = Arc::clone(&rl_arc);
                let watcher = match RateLimitPolicyWatcher::spawn_with_on_reload(
                    path,
                    move |fresh| {
                        if let Err(e) = rl_for_callback.swap_policy(fresh.clone()) {
                            warn!(
                                error = %e,
                                "rate-limit policy swap rejected — keeping previous GCRA state"
                            );
                        }
                    },
                ) {
                    Ok(w) => Some(w),
                    Err(e) => {
                        warn!(error = %e, "rate-limit watcher failed to spawn; hot-reload disabled");
                        None
                    }
                };
                (rl_arc, watcher)
            }
            None => {
                warn!(
                    "cannot resolve ~/.sbfb/ root; rate-limit runs on default policy, no hot-reload"
                );
                (
                    Arc::new(
                        RateLimiter::from_policy_value(
                            crate::rate_limit::RateLimitPolicy::default(),
                        )
                        .expect("default policy must build"),
                    ),
                    None,
                )
            }
        };

        let ephemeral = EphemeralLifecycle::new(worker_config.ephemeral.clone());

        Ok(Self {
            node,
            state: Arc::new(Mutex::new(state)),
            state_tx,
            state_rx,
            allowlist,
            llm,
            gpu,
            gpu_info,
            worker_config,
            shutdown_rx: Some(shutdown_rx),
            shutdown_tx_handle: Some(shutdown_tx),
            keypair,
            task_docs,
            worker_author,
            completed_task_ids: HashSet::new(),
            boot_time: SystemTime::now(),
            last_task: None,
            state_flush_path_override: None,
            consent: consent_handle,
            usage: usage_handle,
            rate_limiter,
            _rate_limit_watcher: rate_limit_watcher,
            ephemeral,
        })
    }

    /// Test helper: redirect the state_writer flush destination
    /// to a caller-supplied path so integration tests don't
    /// clobber the real `~/.nexus-grid/worker/state.json`.
    pub fn set_state_flush_path(&mut self, path: PathBuf) {
        self.state_flush_path_override = Some(path);
    }

    /// Test helper: register a doc directly on the engine without
    /// going through the allowlist's `tasks_doc_ticket` ↔ iroh-docs
    /// import round trip. The Phase D Rust integration test uses
    /// this to inject a doc both sides of the handshake have
    /// access to.
    pub fn register_task_doc(&mut self, project_id: impl Into<String>, doc: DocHandle) {
        self.task_docs.insert(project_id.into(), doc);
    }

    /// Return a [`DocsClient`](nexus_core_rs::docs::DocsClient) backed by
    /// this engine's node.
    ///
    /// Sprint 71 Phase A (B-3): lets an integration test create a doc on
    /// the worker's own node and write a `TaskEntry` onto it through the
    /// **real** dispatch loop, then [`register_task_doc`] it — proving end
    /// to end that a dispatched task is claimed and executed (the
    /// dispatcher key and the worker scan prefix are aligned, B-1). The
    /// pre-S71 worker tests only emulate the coordinator with a hand-written
    /// `task:` key, so they never exercise the production writer.
    pub fn docs(&self) -> nexus_core_rs::docs::DocsClient {
        nexus_core_rs::docs::DocsClient::new(self.node.docs())
    }

    /// Return an **owned** clone of this engine's blob store handle.
    ///
    /// Sprint 72 Phase B (P2-A-2): the cross-process E2E
    /// (`dispatched_task_is_claimed_and_executed_by_worker_engine`) moves the
    /// engine into a `tokio::spawn` to drive it, so it cannot keep a borrowed
    /// [`BlobsClient`](nexus_core_rs::BlobsClient) (which holds `&Store`)
    /// alive across that move. The blob store is a cheap, `Clone`-able handle
    /// over a shared content-addressed backend, so a clone captured *before*
    /// the move still observes the result blob the worker writes *after* it —
    /// letting the test fetch the stored [`ResultEntry`](nexus_core_rs::ResultEntry)
    /// and assert its Ed25519 signature with
    /// [`verify_signature`](nexus_core_rs::ResultEntry::verify_signature). The
    /// S71 B-3 test only asserted that exactly one result entry was produced.
    pub fn blob_store(&self) -> nexus_core_rs::Store {
        self.node.blobs_store().clone()
    }

    /// Return a `watch::Receiver` for the current engine state.
    ///
    /// Every subscriber sees the latest value on first poll and
    /// then receives an update on every transition. W10 TUI
    /// uses this to drive the dashboard render loop.
    pub fn state_rx(&self) -> watch::Receiver<WorkerState> {
        self.state_rx.clone()
    }

    /// Take the shutdown handle so an external signal handler
    /// can trigger graceful termination. The returned
    /// `oneshot::Sender<()>` is armed once; sending `()` causes
    /// [`Engine::run_until_shutdown`] to apply the `Shutdown`
    /// event and return.
    ///
    /// Returns `None` if the handle was already taken.
    pub fn take_shutdown_sender(&mut self) -> Option<oneshot::Sender<()>> {
        self.shutdown_tx_handle.take()
    }

    /// The iroh node's short id (hex-encoded public key). Shown
    /// in `stats` / `start` CLI output and in the TUI header.
    pub fn node_id(&self) -> String {
        self.node.node_id()
    }

    /// List of GPUs probed at boot.
    pub fn gpu_info(&self) -> &[GpuInfo] {
        &self.gpu_info
    }

    /// Take a fresh [`crate::gpu::GpuStats`] snapshot for the
    /// device at `index`. Returns an error if the backend
    /// reports no device at that index.
    ///
    /// Used by the CLI `stats` command and by the W10 TUI to
    /// render live GPU load / temperature / VRAM counters.
    pub fn gpu_snapshot(&self, index: u32) -> Result<crate::gpu::GpuStats, crate::gpu::GpuError> {
        self.gpu.snapshot(index)
    }

    /// Mutate the state machine and broadcast the resulting
    /// state on the watch channel. Returns the previous state
    /// on a legal transition; logs a warning and keeps the
    /// current state on an illegal one.
    async fn apply_event(&self, event: WorkerEvent) {
        let mut sm = self.state.lock().await;
        match sm.apply(event.clone()) {
            Ok(prev) => {
                let current = sm.state().clone();
                debug!(
                    previous = %prev,
                    next = %current,
                    event = %event.label(),
                    "engine state transition"
                );
                let _ = self.state_tx.send(current);
            }
            Err(e) => {
                warn!(
                    state = %sm.state(),
                    event = %event.label(),
                    error = %e,
                    "engine state machine rejected event"
                );
            }
        }
    }

    /// Run the main engine loop until either the state machine
    /// enters `Shutdown` or an external shutdown signal arrives
    /// on the `shutdown_tx` channel.
    ///
    /// Consumes `self` so the engine cannot be reused after
    /// shutdown — this mirrors `Node::shutdown` and prevents
    /// leaking the iroh endpoint.
    pub async fn run_until_shutdown(mut self) -> anyhow::Result<()> {
        // Apply Start now that we are actually about to loop.
        self.apply_event(WorkerEvent::Start).await;

        let shutdown_rx = self
            .shutdown_rx
            .take()
            .expect("engine shutdown receiver already taken");

        let poll_interval = Duration::from_millis(self.worker_config.engine.task_poll_interval_ms);
        let flush_interval = Duration::from_secs(self.worker_config.engine.state_flush_secs);

        // The cached initial state after Start.
        let mut shutdown_rx = shutdown_rx;

        // Sprint 5 Phase A: flush a first snapshot immediately so
        // the shell has something to read before the first
        // flush_interval elapses.
        self.flush_state_snapshot();
        let mut last_flush = Instant::now();

        loop {
            // React to shutdown signal OR the poll interval
            // firing, whichever comes first. The biased=true
            // select prefers shutdown so a pending poll cannot
            // starve the signal.
            tokio::select! {
                biased;

                res = &mut shutdown_rx => {
                    match res {
                        Ok(()) => info!("engine received shutdown signal"),
                        Err(_) => info!("engine shutdown sender dropped; exiting loop"),
                    }
                    self.apply_event(WorkerEvent::Shutdown).await;
                    break;
                }

                _ = tokio::time::sleep(poll_interval) => {
                    self.tick().await;

                    // Sprint 5 Phase A: flush a shell snapshot
                    // every `state_flush_secs`. Runs after the
                    // tick so the snapshot reflects any task
                    // completion we just recorded.
                    if last_flush.elapsed() >= flush_interval {
                        self.flush_state_snapshot();
                        last_flush = Instant::now();
                    }

                    if self.state_rx.borrow().is_terminal() {
                        break;
                    }
                }
            }
        }

        // Sprint 5 Phase A: one last flush on the way out so the
        // shell's last view of the worker reflects the shutdown.
        self.flush_state_snapshot();

        info!("engine loop exited, shutting down iroh node");
        // Best-effort shutdown; log on failure but do not
        // propagate because the loop already committed to
        // terminating.
        if let Err(e) = self.node.shutdown().await {
            error!(error = %e, "iroh node shutdown reported an error");
        }

        Ok(())
    }

    /// Single tick of the main loop.
    ///
    /// Runs every `task_poll_interval_ms`. Responsibilities:
    ///
    /// 1. Probe Ollama health (cheap: `list_local_models`). If
    ///    the daemon just came back up, transition from
    ///    `Connecting` → `Processing`. If it just went down,
    ///    transition back to `Connecting`.
    /// 2. List enabled projects from the allowlist. Imported
    ///    docs are populated at boot from each project's
    ///    `tasks_doc_ticket`.
    /// 3. Task claim / execute / result write-back — handled by
    ///    [`Engine::scan_and_execute_tasks`] when the state
    ///    machine is in `Processing`.
    async fn tick(&mut self) {
        // Snapshot state before the tick so we can decide
        // transitions without holding the lock across awaits.
        let current = self.state_rx.borrow().clone();

        match current {
            WorkerState::Connecting => {
                // Transition to Processing as soon as Ollama is
                // reachable and at least one project is
                // enrolled. Otherwise stay in Connecting; the
                // TUI will show the user what's missing.
                let hc = self.llm.healthcheck().await;
                let enabled_count = match self.allowlist.list_enabled() {
                    Ok(list) => list.len(),
                    Err(e) => {
                        error!(error = %e, "failed to list enabled projects");
                        0
                    }
                };

                if hc.is_ready() && enabled_count > 0 {
                    info!(
                        enabled_projects = enabled_count,
                        "ollama ready and projects enrolled; entering Processing"
                    );
                    self.apply_event(WorkerEvent::Connected).await;
                } else {
                    debug!(
                        ollama_ready = hc.is_ready(),
                        enabled_projects = enabled_count,
                        "engine waiting for ollama + enrolled projects"
                    );
                }
            }
            WorkerState::Processing { .. } => {
                // Sprint 4 Phase D W9.1 — the full drop-in:
                // scan every imported project doc for task:* entries,
                // skip anything we've already processed, claim the
                // rest, run inference, write the signed result back.
                //
                // Ollama precondition: if the daemon went away the
                // engine cannot serve tasks this tick. Bounce back
                // through Pause → Resume so the state machine
                // reflects the degraded state.
                let hc = self.llm.healthcheck().await;
                if !hc.is_ready() {
                    warn!("ollama is no longer reachable, dropping back to Connecting");
                    self.apply_event(WorkerEvent::Pause).await;
                    self.apply_event(WorkerEvent::Resume).await;
                    return;
                }

                if self.worker_author.is_none() || self.task_docs.is_empty() {
                    return;
                }

                if let Err(e) = self.scan_and_execute_tasks().await {
                    error!(error = %e, "task scan/execute failed this tick");
                }
            }
            WorkerState::PullingModel { .. } => {
                // W5 retry policy already handles this via
                // exponential backoff inside the LlmBackend;
                // nothing to do in the tick until W9.1 wires
                // the actual pull_model stream through.
            }
            WorkerState::Paused => {
                // User pause — tick does nothing until Resume.
            }
            WorkerState::Error { ref reason } => {
                warn!(error = %reason, "engine is in Error state; awaiting manual Clear");
            }
            WorkerState::Idle | WorkerState::Shutdown => {
                // Idle shouldn't actually happen here because
                // the loop always applies Start first. Shutdown
                // is terminal and the select! above covers it.
            }
        }
    }

    /// Sprint 4 Phase D W9.1 — main task-execution pump.
    ///
    /// For every doc registered in [`Engine::task_docs`]:
    ///
    /// 1. Scan `task:*` entries.
    /// 2. Skip tasks we already completed in this process (local
    ///    dedupe set) or ones we can see an existing `claim:<id>`
    ///    or `result:<id>` entry for on the same doc.
    /// 3. Read the blob content for the task entry, deserialize
    ///    to [`TaskEntry`], verify the coordinator signature.
    /// 4. Mint a [`ClaimEntry`] and write it under
    ///    `claim:<task_id>`.
    /// 5. Call [`LlmBackend::generate`] with the prompt.
    /// 6. Build a [`ResultPayload`] (deterministic digests in
    ///    stub mode; hashed model digest + logprob placeholder
    ///    otherwise) and sign it into a [`ResultEntry`], write
    ///    under `result:<task_id>`.
    /// 7. Mark the task_id as completed and update the allowlist
    ///    counters.
    ///
    /// Errors are logged per-task and the loop keeps going so one
    /// bad task does not starve the rest.
    async fn scan_and_execute_tasks(&mut self) -> anyhow::Result<()> {
        let author = match self.worker_author {
            Some(a) => a,
            None => return Ok(()),
        };

        // Snapshot the doc list so we don't hold a borrow on
        // self.task_docs across the await points inside the inner
        // loop. DocHandle is cheaply cloneable (iroh-docs::api::Doc
        // is an Arc inside).
        let doc_snapshot: Vec<(String, DocHandle)> = self
            .task_docs
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (project_id, doc) in doc_snapshot {
            let entries = match doc.get_many_by_prefix(b"task:").await {
                Ok(e) => e,
                Err(e) => {
                    warn!(project = %project_id, error = %e, "get_many_by_prefix failed");
                    continue;
                }
            };

            for entry in entries {
                let key_bytes = entry.key();
                let task_id = match std::str::from_utf8(key_bytes)
                    .ok()
                    .and_then(|s| s.strip_prefix("task:"))
                {
                    Some(s) => s.to_string(),
                    None => continue,
                };

                if self.completed_task_ids.contains(&task_id) {
                    continue;
                }
                if self.task_already_handled_on_doc(&doc, &task_id).await {
                    // Another worker has already claimed or
                    // resulted this task. Cache locally so we
                    // stop re-checking on every tick.
                    self.completed_task_ids.insert(task_id);
                    continue;
                }

                // Fetch the blob content backing this entry.
                let content_hash: [u8; 32] = *entry.content_hash().as_bytes();
                let blobs = BlobsClient::new(self.node.blobs_store());
                let blob = match blobs.get_bytes(content_hash).await {
                    Ok(b) => b,
                    Err(e) => {
                        warn!(
                            task_id = %task_id,
                            error = %e,
                            "task blob not yet available; will retry next tick",
                        );
                        continue;
                    }
                };

                // Parse + verify TaskEntry.
                let task_entry: TaskEntry = match serde_json::from_slice(&blob) {
                    Ok(t) => t,
                    Err(e) => {
                        warn!(task_id = %task_id, error = %e, "task JSON invalid");
                        continue;
                    }
                };
                if let Err(e) = task_entry.verify_signature() {
                    warn!(task_id = %task_id, error = %e, "task signature invalid");
                    continue;
                }

                // Sprint 16 audit fix C-1 + C-2: read the four v2
                // Task fields directly from the verified TaskEntry.
                // `is_open_source` comes from the coordinator at
                // task craft time (set from the project's PA v5
                // flag). The three `estimated_*` fields come from
                // the app submitting the task (Option A: the app
                // knows its model best). Zero means "unknown" —
                // the corresponding cap stays inert for that task.
                let task_ctx = TaskContext {
                    project_id: &project_id,
                    is_open_source: task_entry.task.is_open_source,
                    estimated_watts: task_entry.task.estimated_watts,
                    estimated_vram_mb: task_entry.task.estimated_vram_mb,
                    estimated_hours: task_entry.task.estimated_hours,
                };
                if let Some(watcher) = self.consent.as_ref() {
                    match watcher.current() {
                        Ok(consent_cfg) => {
                            let outcome = if let Some(usage) = self.usage.as_ref() {
                                let mut guard = usage.lock().await;
                                consent::should_accept_task(&task_ctx, &consent_cfg, &mut guard)
                            } else {
                                // No usage tracker — build a
                                // throwaway one so the hours
                                // cap check always passes. The
                                // level + watts + vram filter
                                // still runs normally.
                                let mut throwaway = match UsageTracker::load_or_default(
                                    std::env::temp_dir().join("sbfb-usage-noop.json"),
                                ) {
                                    Ok(u) => u,
                                    Err(_) => continue,
                                };
                                consent::should_accept_task(&task_ctx, &consent_cfg, &mut throwaway)
                            };
                            if let AllowOutcome::Reject(reason) = outcome {
                                let reason_str = match reason {
                                    RejectReason::NotOwnProject => "not_own_project",
                                    RejectReason::NotOpenSource => "not_open_source",
                                    RejectReason::NotInWhitelist => "not_in_whitelist",
                                    RejectReason::CapWatts => "cap_watts",
                                    RejectReason::CapVram => "cap_vram",
                                    RejectReason::CapHoursToday => "cap_hours_today",
                                };
                                debug!(
                                    project = %project_id,
                                    task_id = %task_id,
                                    reason = reason_str,
                                    "task rejected by consent filter",
                                );
                                continue;
                            }
                        }
                        Err(e) => {
                            warn!(
                                error = %e,
                                "consent state unreadable; rejecting task (fail-closed)"
                            );
                            continue;
                        }
                    }
                }

                // Sprint 22 Phase A : rate-limit engine gate.
                // Consent filter has cleared the task ; now bound
                // its admission rate by the tuple
                // `(coordinator_that_signed, self, model)`. A
                // saturated bucket defers the task — we `continue`
                // without emitting a `ClaimEntry`, so the TaskEntry
                // stays live on the doc and the next tick (after
                // GCRA replenish) has a fresh shot. Defends
                // `HARDENING_ROADMAP §3 C-ModelExtract` model-
                // extraction paper-flood + `C-DosFlood` §7 DoS
                // flood at the runtime layer, making the S21 Phase
                // A primitive effective on the hot path.
                let rate_key = RateKey::new(
                    hex::encode(task_entry.author_pubkey),
                    hex::encode(self.keypair.public_bytes()),
                    task_entry.task.model.clone(),
                );
                match self.rate_limiter.check(&rate_key) {
                    Ok(()) => {}
                    Err(RateLimitError::Saturated {
                        consumer,
                        worker,
                        model,
                    }) => {
                        debug!(
                            task_id = %task_id,
                            consumer = %consumer,
                            worker = %worker,
                            model = %model,
                            "rate-limit saturated; deferring task (no claim emitted this tick)"
                        );
                        continue;
                    }
                    Err(e) => {
                        // InvalidQuota surfaces only on a mis-
                        // configured policy — the watcher rejects
                        // malformed swaps and keeps the previous
                        // known-good state, so reaching this arm
                        // means the bootstrap quota itself was
                        // invalid and the engine fell back to
                        // default. We `warn!` and continue (the
                        // default policy's quotas are always
                        // valid) rather than crash the engine.
                        warn!(task_id = %task_id, error = %e, "rate-limit gate errored; skipping task");
                        continue;
                    }
                }

                // Sprint 76 Phase C (D3 etage 1): cohort-homogeneity
                // claim-gate. When the coordinator pinned a
                // `required_runtime` tuple (deterministic-quorum
                // dispatch — `verifiable` + redundancy>1), a worker
                // claims only if its local runtime fingerprint
                // satisfies the requirement. A mismatch defers the
                // task with NO claim emitted — exactly like the
                // rate-limit defer above — so it stays live on the doc
                // for a homogeneous peer. This is the PULL-correct
                // point of application: the worker self-selects; the
                // coordinator never assigns. The gate is advisory
                // routing, not a trust boundary — the unchanged
                // exact-match quorum (`validate_quorum_pre_guardrail`)
                // still rejects a divergent result as an outlier.
                if let Some(required) = task_entry.task.required_runtime.as_ref() {
                    let local = self.llm.runtime_tuple(&task_entry.task.model).await;
                    if !local.matches(required) {
                        debug!(
                            task_id = %task_id,
                            required_family = %required.runtime_family,
                            required_quant = %required.quant,
                            local_family = %local.runtime_family,
                            local_quant = %local.quant,
                            "runtime tuple mismatch; deferring task (cohort homogeneity, no claim emitted)"
                        );
                        continue;
                    }
                }

                let task_started_at = Instant::now();

                // Sign + write claim.
                let claim = Claim::new(
                    task_entry.task.task_id.clone(),
                    self.keypair.public_bytes(),
                    now_unix_secs(),
                );
                let claim_entry = match ClaimEntry::sign(claim, &self.keypair) {
                    Ok(e) => e,
                    Err(e) => {
                        warn!(task_id = %task_id, error = %e, "claim sign failed");
                        continue;
                    }
                };
                let claim_json = match serde_json::to_vec(&claim_entry) {
                    Ok(j) => j,
                    Err(e) => {
                        warn!(task_id = %task_id, error = %e, "claim serialize failed");
                        continue;
                    }
                };
                let claim_key = format!("claim:{}", task_entry.task.task_id);
                if let Err(e) = doc.set(author, claim_key.into_bytes(), claim_json).await {
                    warn!(task_id = %task_id, error = %e, "claim write failed");
                    continue;
                }

                // Run the LLM. A `verifiable` task gets deterministic
                // (greedy + fixed-seed) params so independent honest
                // workers converge on the same `result_text` for
                // hash-exact quorum (Sprint 71 Phase B, B-2 / D2).
                let params = build_generate_params(&task_entry.task, &self.worker_config.watermark);

                // Measure the real inference wall-clock so the signed payload
                // carries a truthful `generation_time_ms` (Sprint 76 Phase E,
                // D4-Q). The coordinator's kudos sanity-bound clamps the
                // self-declared token count against this duration, so a
                // hardcoded 0 would collapse every honest credit > the per-ms
                // ceiling. `Instant` is monotonic (immune to wall-clock jumps);
                // `started_at`/`finished_at` bracket the same call in epoch secs.
                let started_at = now_unix_secs();
                let gen_start = Instant::now();
                let generated = match self.llm.generate(params).await {
                    Ok(r) => r,
                    Err(e) => {
                        warn!(task_id = %task_id, error = %e, "ollama.generate failed");
                        continue;
                    }
                };
                let generation_time_ms = gen_start.elapsed().as_millis() as u64;

                // Build the result payload.
                //
                // model_digest: BLAKE3 of the model NAME string (see
                //   `model_name_digest`). Sprint 76 Phase C doc-note
                //   (D3 etage 1): this is a name hash, NOT a GGUF
                //   weight-file hash — Ollama exposes no clean file
                //   digest and `Verifier` has no prod caller, so the
                //   live path (hash-exact quorum over result_text) is
                //   unaffected. A real weight digest is gated on a
                //   file-exposing backend (`llm_llama_cpp`, S77 / D3
                //   etage 2). Matches what the coordinator's stub
                //   verifier expects when no whitelist was loaded
                //   (unprofiled_model_passes_digest test).
                // logprobs_hash: 32 zero bytes — "logprobs not
                //   provided" in the Sprint 3 Verifier semantics.
                let model_digest: [u8; 32] = model_name_digest(&task_entry.task.model);
                let payload = ResultPayload {
                    version: TASK_FORMAT_VERSION,
                    task_id: task_entry.task.task_id.clone(),
                    result_text: generated.text,
                    tokens_generated: generated.completion_tokens.unwrap_or(0),
                    generation_time_ms,
                    model_digest,
                    logprobs_hash: [0u8; 32],
                    started_at,
                    finished_at: now_unix_secs(),
                    output_token_ids: generated.output_token_ids,
                };
                let result_entry = match ResultEntry::sign(payload, &self.keypair) {
                    Ok(e) => e,
                    Err(e) => {
                        warn!(task_id = %task_id, error = %e, "result sign failed");
                        continue;
                    }
                };
                let result_json = match serde_json::to_vec(&result_entry) {
                    Ok(j) => j,
                    Err(e) => {
                        warn!(task_id = %task_id, error = %e, "result serialize failed");
                        continue;
                    }
                };
                let result_key = format!("result:{}", task_entry.task.task_id);
                if let Err(e) = doc.set(author, result_key.into_bytes(), result_json).await {
                    warn!(task_id = %task_id, error = %e, "result write failed");
                    continue;
                }

                // Success: mark the task done and record the
                // usage counter on the project allowlist row.
                info!(
                    project = %project_id,
                    task_id = %task_entry.task.task_id,
                    "task completed and result written",
                );
                self.completed_task_ids
                    .insert(task_entry.task.task_id.clone());
                if let Err(e) = self.allowlist.record_task(&project_id, 0) {
                    debug!(error = %e, "allowlist.record_task failed (non-fatal)");
                }

                // Sprint 16 Phase C: record the wall-clock time
                // this task ate into the per-day hours cap. Non-
                // fatal: a write error just means the next task
                // check reads a stale counter.
                if let Some(usage) = self.usage.as_ref() {
                    let duration_hours = task_started_at.elapsed().as_secs_f64() / 3600.0;
                    let mut guard = usage.lock().await;
                    if let Err(e) = guard.record_task(duration_hours) {
                        debug!(error = %e, "usage.record_task failed (non-fatal)");
                    }
                }

                // Sprint 5 Phase A: record this as the "last
                // task" for the shell snapshot. Uses the project
                // id as the project_name because the allowlist
                // row is keyed by id — the shell can resolve the
                // human-readable project_name through /project.
                self.last_task = Some(LastTask {
                    task_id: task_entry.task.task_id.clone(),
                    project_name: project_id.clone(),
                    prompt_preview: preview_prompt(&task_entry.task.prompt, 120),
                    status: "completed".to_string(),
                    completed_at: rfc3339_now(),
                });

                // Sprint 23 Phase B : ephemeral lifecycle post-task.
                self.ephemeral.start_task();
                self.ephemeral.complete_task();

                if self.ephemeral.state() == LifecycleState::WipePending {
                    if let Err(e) = crate::ephemeral::wipe_vram().await {
                        warn!(error = %e, "ephemeral VRAM wipe failed (non-fatal)");
                    }
                    self.ephemeral.wipe_done();
                }

                if self.ephemeral.state() == LifecycleState::RestartPending {
                    self.ephemeral.request_exit();
                    info!(
                        completed = self.ephemeral.completed_count(),
                        "ephemeral max_tasks reached; requesting graceful shutdown"
                    );
                    self.apply_event(WorkerEvent::Shutdown).await;
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    /// Sprint 5 Phase A: build a snapshot from the live engine
    /// state and flush it to disk. Called once at loop start,
    /// once per `state_flush_secs` thereafter, and once on
    /// graceful shutdown. Errors are logged inside
    /// [`state_writer::flush`] and never propagate.
    fn flush_state_snapshot(&self) {
        let dest = match self.state_flush_path_override.clone() {
            Some(p) => p,
            None => match worker_state_file() {
                Some(p) => p,
                None => {
                    debug!("no BaseDirs available; skipping worker state flush");
                    return;
                }
            },
        };

        // Best-effort probe of the first GPU device so the
        // snapshot carries a live VRAM / utilization reading.
        // A probe failure collapses both fields to None — the
        // plan §2.3 explicitly allows `gpu: null`.
        let (gpu_info, gpu_stats) = match self.gpu_info.first() {
            Some(info) => match self.gpu.snapshot(info.index) {
                Ok(stats) => (Some(info), Some(stats)),
                Err(e) => {
                    debug!(error = %e, "gpu snapshot failed; reporting null gpu");
                    (None, None)
                }
            },
            None => (None, None),
        };

        let inputs = SnapshotInputs {
            node_id: self.node.node_id(),
            worker_version: crate::VERSION,
            boot_time: self.boot_time,
            gpu_info,
            gpu_stats: gpu_stats.as_ref(),
            allowlist: &self.allowlist,
            last_task: self.last_task.clone(),
            consent: self.consent_snapshot(),
        };

        state_writer::flush(inputs, &dest);
    }

    /// Sprint 76 Phase A (D1): build the optional consent snapshot for
    /// the state file — the active sharing level + caps from the
    /// consent watcher plus today's hours from the usage tracker, so
    /// the "offer my power" panel renders a live caps gauge without a
    /// new endpoint. Returns `None` when no consent watcher is wired
    /// (the worker shares nothing), leaving the snapshot field absent.
    fn consent_snapshot(&self) -> Option<state_writer::ConsentSnapshot> {
        let cfg = self.consent.as_ref()?.current().ok()?;
        // Usage is read non-blocking: the flush runs on the same task
        // as the claim pump, so the tokio mutex is virtually never
        // contended; a rare miss reports 0h for this tick and the next
        // flush recovers the real value.
        let hours_used_today = self
            .usage
            .as_ref()
            .and_then(|u| u.try_lock().ok().map(|mut g| g.hours_used_today()))
            .unwrap_or(0.0);
        Some(state_writer::ConsentSnapshot {
            level: u8::from(cfg.level),
            max_hours_day: cfg.caps.max_hours_day,
            hours_used_today,
            max_watts: cfg.caps.max_watts,
            max_vram_mb: cfg.caps.max_vram_mb,
        })
    }

    /// Returns true if the doc already has a `claim:<id>` or
    /// `result:<id>` entry for this task. Used to avoid
    /// re-claiming tasks another worker has taken.
    async fn task_already_handled_on_doc(&self, doc: &DocHandle, task_id: &str) -> bool {
        let claim_key = format!("claim:{task_id}");
        let result_key = format!("result:{task_id}");
        let Some(author) = self.worker_author else {
            return false;
        };

        // get_exact returns Ok(None) when the key is absent. We
        // specifically check the worker's own author because
        // iroh-docs' single-author get_exact is the cheapest query.
        // If the key exists under a different author the check
        // correctly returns false → we attempt a write, which is
        // fine because our ClaimEntry signature makes it unique.
        if let Ok(Some(_)) = doc.get_exact(author, &claim_key).await {
            return true;
        }
        if let Ok(Some(_)) = doc.get_exact(author, &result_key).await {
            return true;
        }
        false
    }
}

/// Build the backend [`GenerateParams`] for `task`.
///
/// When [`Task::verifiable`] is set, the params force deterministic
/// (greedy, fixed-seed) inference via [`GenerateParams::deterministic`]
/// so two honest workers reproduce the same `result_text` and the
/// coordinator's hash-exact quorum (`validate_quorum`) can accept by
/// majority. Otherwise the worker keeps the pre-Sprint-71 best-effort
/// sampling. The watermark config is threaded through unchanged.
/// (Sprint 71 Phase B, B-2 / D2.)
fn build_generate_params(task: &Task, watermark: &WatermarkConfig) -> GenerateParams {
    let params = GenerateParams::new(task.model.clone(), task.prompt.clone())
        .with_system(task.system_prompt.clone())
        .with_watermark(
            watermark.enabled,
            task.watermark_seed.clone(),
            watermark.delta_logit,
            watermark.window_size,
        );
    if task.verifiable {
        params.deterministic(deterministic_seed(&task.task_id))
    } else {
        params
    }
}

/// Derive a stable `u32` seed from the task id. Every honest worker
/// computing the same task derives the same seed, so a fixed-seed
/// greedy decode reproduces the same tokens across workers. The seed
/// is NOT a secret — it is determinism only, distinct from the
/// per-task watermark PRF seed (`Task::watermark_seed`).
/// (Sprint 71 Phase B, B-2.)
fn deterministic_seed(task_id: &str) -> u32 {
    let digest = blake3_hash(task_id.as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

/// BLAKE3 digest the worker writes into [`ResultPayload::model_digest`].
///
/// **Sprint 76 Phase C doc-note (D3 etage 1):** this hashes the model
/// *name* string, NOT the GGUF weight file. The Ollama HTTP backend
/// (ollama-rs 0.3.4) exposes no clean file-digest accessor, and
/// `Verifier` (the sole consumer of the layer-2 `model_digest`) has no
/// production caller, so the live result path — the hash-exact quorum
/// over `result_text` (`validate_quorum_pre_guardrail`) — is
/// unaffected by the name-vs-file distinction. A real weight digest is
/// gated on a file-exposing backend (`llm_llama_cpp` C-API, Sprint 77 /
/// D3 etage 2). This helper is the single seam that pins the contract:
/// changing it to a file hash must be a deliberate, reviewed break.
fn model_name_digest(model: &str) -> [u8; 32] {
    blake3_hash(model.as_bytes())
}

/// Unix seconds with a graceful fallback on clock failure. Used
/// by the W9.1 task flow for `Claim::claimed_at` and
/// `ResultPayload::started_at` / `finished_at`.
fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Sprint 5 Phase A: RFC 3339 "now" for the shell snapshot's
/// `last_task.completed_at` field. Falls back to the Unix epoch
/// on a clock failure — the shell tolerates "1970" better than a
/// missing field.
fn rfc3339_now() -> String {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    let secs = now_unix_secs() as i64;
    OffsetDateTime::from_unix_timestamp(secs)
        .ok()
        .and_then(|dt| dt.format(&Rfc3339).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// Sprint 5 Phase A: truncate a prompt to a UTF-8-safe preview
/// of at most `max_chars` characters, appending `…` if the
/// prompt was longer.
fn preview_prompt(prompt: &str, max_chars: usize) -> String {
    let mut out = String::with_capacity(max_chars + 3);
    for (i, c) in prompt.chars().enumerate() {
        if i >= max_chars {
            out.push('…');
            return out;
        }
        out.push(c);
    }
    out
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consent::{ConsentConfig, ConsentLevel};
    use crate::llm::StubBackend;
    use nexus_core_rs::docs::DocsClient as RsDocsClient;
    use nexus_core_rs::task::{RuntimeTuple, Task, TaskEntry};
    use tempfile::TempDir;

    #[test]
    fn verifiable_task_uses_greedy_seed() {
        let wm = WatermarkConfig::default();

        // A verifiable task forces greedy decoding (temperature 0) +
        // a fixed seed derived deterministically from the task id, so
        // every honest worker on this task pins the same sampling.
        let det = Task::new("task-det", "analysis", "p", "llama3", 5, 0).with_verifiable(true);
        let params = build_generate_params(&det, &wm);
        assert_eq!(params.temperature, Some(0.0), "verifiable => greedy");
        assert_eq!(
            params.seed,
            Some(deterministic_seed("task-det")),
            "verifiable => fixed seed derived from the task id"
        );

        // The derived seed is stable (same task id => same seed),
        // which is exactly what lets two independent workers converge.
        assert_eq!(
            deterministic_seed("task-det"),
            deterministic_seed("task-det")
        );
        assert_ne!(
            deterministic_seed("task-det"),
            deterministic_seed("task-other"),
            "different tasks get different seeds"
        );

        // A best-effort task leaves sampling to the backend default.
        let plain = Task::new("task-plain", "analysis", "p", "llama3", 5, 0);
        let plain_params = build_generate_params(&plain, &wm);
        assert_eq!(plain_params.temperature, None);
        assert_eq!(plain_params.seed, None);
    }

    /// Sprint 76 Phase D: the `verifiable` determinism seed is
    /// **cross-worker stable** — every honest worker computing the same
    /// task derives the SAME seed, the premise the redundancy>1 quorum
    /// relies on. This test pins the HONEST contract the plan's shorthand
    /// "seed = blake3(task_id)" abbreviates: the seed is the `u32`
    /// little-endian of the **first 4 bytes** of `blake3(task_id)`, a
    /// truncation of the 32-byte digest, not the whole hash.
    #[test]
    fn verifiable_seed_is_cross_worker_stable() {
        let task_id = "task-quorum-seed";

        // Two independent "workers" derive the identical seed for the task.
        let worker_a_seed = deterministic_seed(task_id);
        let worker_b_seed = deterministic_seed(task_id);
        assert_eq!(
            worker_a_seed, worker_b_seed,
            "two honest workers on the same task must derive the same seed"
        );

        // Honest contract: the seed is the u32 LE of blake3(task_id)[..4],
        // a truncation of the digest — NOT the full 32-byte hash.
        let digest = blake3_hash(task_id.as_bytes());
        let expected = u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]]);
        assert_eq!(
            worker_a_seed, expected,
            "seed must be the u32 LE truncation of the first 4 bytes of blake3(task_id)"
        );

        // Distinct task ids derive distinct seeds (no accidental constant).
        assert_ne!(
            deterministic_seed(task_id),
            deterministic_seed("task-quorum-seed-other"),
            "different task ids derive different seeds"
        );
    }

    async fn build_engine_with_stub_ollama() -> Engine {
        let worker_config = WorkerConfig::default();
        let keypair = KeyPair::generate();
        let allowlist = Allowlist::open_in_memory().unwrap();

        let boot = EngineBoot {
            worker_config: worker_config.clone(),
            keypair,
            allowlist,
            data_dir: None,
            llm_override: Some(Box::new(StubBackend::new())),
            sbfb_home_override: None,
            rate_limit_policy_path_override: None,
        };
        Engine::new(boot).await.expect("engine boots")
    }

    #[tokio::test]
    async fn engine_boots_in_idle_state_and_exposes_node_id() {
        let engine = build_engine_with_stub_ollama().await;
        assert_eq!(engine.state_rx().borrow().label(), "idle");
        assert!(!engine.node_id().is_empty());
        engine
            .node
            .shutdown()
            .await
            .expect("shutdown the engine's node after the assertions");
    }

    // P2-A-1 (S71->S73): spawns the engine pump; current_thread deadlocks
    // under Windows `cargo test` shared-process teardown (tokio #7049).
    // multi_thread matches prod (worker binary). See PATTERNS §P54.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn engine_applies_start_event_on_run() {
        let mut engine = build_engine_with_stub_ollama().await;
        // Cancel the shutdown mechanism before run: hold a
        // sender, send right away so the loop exits on the
        // first select poll.
        let tx = engine.take_shutdown_sender().expect("first take succeeds");
        let rx = engine.state_rx();

        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        // Give the loop a tick to apply Start and see the
        // watch channel update. Use a bounded timeout so a
        // regression does not hang the test suite.
        let first_non_idle = tokio::time::timeout(Duration::from_secs(2), async {
            let mut rx = rx.clone();
            loop {
                let s = rx.borrow().clone();
                if !matches!(s, WorkerState::Idle) {
                    return s;
                }
                rx.changed().await.unwrap();
            }
        })
        .await
        .expect("state machine should leave Idle within 2s");

        // The first non-Idle state must be Connecting (engine
        // applies Start → Connecting before anything else).
        assert_eq!(first_non_idle.label(), "connecting");

        let _ = tx.send(());
        handle.await.expect("run task joins").expect("run ok");
    }

    // P2-A-1 (S71->S73): spawns the engine pump; current_thread deadlocks
    // under Windows `cargo test` shared-process teardown (tokio #7049).
    // multi_thread matches prod (worker binary). See PATTERNS §P54.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn engine_transitions_to_processing_when_project_is_enrolled() {
        // Build an engine whose allowlist has one project, so
        // the tick's "ollama_ready && enabled > 0" branch
        // fires and the state machine moves Connecting →
        // Processing within a handful of ticks.
        let worker_config = WorkerConfig {
            engine: crate::config::Engine {
                task_poll_interval_ms: 100,
                max_concurrent_tasks: 1,
                state_flush_secs: 5,
            },
            ..WorkerConfig::default()
        };
        let keypair = KeyPair::generate();
        let allowlist = Allowlist::open_in_memory().unwrap();
        allowlist
            .enroll(crate::allowlist::NewProject {
                id: "proj-x".into(),
                name: "X".into(),
                enabled: true,
                budget_joules: 0,
                tasks_doc_ticket: None,
            })
            .unwrap();

        let boot = EngineBoot {
            worker_config,
            keypair,
            allowlist,
            data_dir: None,
            llm_override: Some(Box::new(StubBackend::new())),
            sbfb_home_override: None,
            rate_limit_policy_path_override: None,
        };
        let mut engine = Engine::new(boot).await.expect("engine boots");

        let tx = engine.take_shutdown_sender().unwrap();
        let rx = engine.state_rx();

        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        tokio::time::timeout(Duration::from_secs(5), async {
            let mut rx = rx.clone();
            loop {
                if matches!(*rx.borrow(), WorkerState::Processing { .. }) {
                    return;
                }
                rx.changed().await.unwrap();
            }
        })
        .await
        .expect("engine should reach Processing within 5s");

        let _ = tx.send(());
        handle.await.unwrap().unwrap();
    }

    // P2-A-1 (S71->S73) MANDATORY: the worker-side mirror of the dispatch
    // E2E. Spawns the engine pump (polls the iroh-docs actor) and waits on a
    // real-time loop for `result:`. current_thread deadlocks under Windows
    // `cargo test` shared-process teardown (tokio #7049); multi_thread
    // matches prod and the only working 2-node sync example. The 10s timeout
    // is defence-in-depth. See PATTERNS §P54.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn engine_claims_and_executes_tasks_on_registered_doc() {
        // Sprint 4 Phase D W9.1 end-to-end in a single process.
        // The test emulates a coordinator by directly creating a
        // doc on the worker's own Node, signing a TaskEntry with
        // a throwaway coordinator keypair, and writing it under
        // `task:<id>`. The worker engine then ticks, picks the
        // task up via its StubOllama client, and writes back a
        // signed ResultEntry that the test reads off the same
        // doc.
        //
        // This is stronger than the Sprint 3 runtime tests because
        // it exercises the full claim → execute → write path that
        // the production worker will run against a real imported
        // DocTicket. The ticket-import branch is covered by the
        // separate `persistent_data_dir_reboots_with_same_doc_and_author`
        // test in crates/nexus-core-rs/src/node.rs.
        let worker_config = WorkerConfig {
            engine: crate::config::Engine {
                task_poll_interval_ms: 100,
                max_concurrent_tasks: 1,
                state_flush_secs: 5,
            },
            ..WorkerConfig::default()
        };
        let keypair = KeyPair::generate();
        let allowlist = Allowlist::open_in_memory().unwrap();
        allowlist
            .enroll(crate::allowlist::NewProject {
                id: "proj-w91".into(),
                name: "W9.1".into(),
                enabled: true,
                budget_joules: 0,
                tasks_doc_ticket: None,
            })
            .unwrap();

        // Sprint 16 Phase C: the consent filter defaults to L1
        // (own projects only) which would reject `proj-w91`
        // because that id doesn't match the worker's own
        // node_id hex. Pre-seed a tempdir override with an L4
        // consent so this test keeps its pre-S16 semantics.
        let sbfb_tmp: TempDir = tempfile::tempdir().unwrap();
        let mut consent = ConsentConfig::default_for("test-worker");
        consent.level = ConsentLevel::All;
        consent
            .save_atomic(&sbfb_tmp.path().join("consent.json"))
            .unwrap();

        let boot = EngineBoot {
            worker_config,
            keypair,
            allowlist,
            data_dir: None,
            llm_override: Some(Box::new(StubBackend::new())),
            sbfb_home_override: Some(sbfb_tmp.path().to_path_buf()),
            rate_limit_policy_path_override: None,
        };
        let mut engine = Engine::new(boot).await.expect("engine boots");

        // Create a doc on the worker's own Node (the test doesn't
        // use import_ticket; it just injects the doc directly to
        // keep the test hermetic).
        let docs = RsDocsClient::new(engine.node.docs());
        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();

        // Write a signed TaskEntry under `task:t-1`.
        let coord_kp = KeyPair::generate();
        let mut task = Task::new(
            "t-1",
            "analysis",
            "Echo hello from the e2e test",
            "stub-model:latest",
            5,
            1_000_000_000,
        );
        task.system_prompt = "".into();
        let task_entry = TaskEntry::sign(task, &coord_kp).unwrap();
        let task_json = serde_json::to_vec(&task_entry).unwrap();
        doc.set(author, b"task:t-1".to_vec(), task_json)
            .await
            .unwrap();

        engine.register_task_doc("proj-w91", doc.clone());

        // Drive the loop: run until a `result:t-1` entry appears
        // on the doc, with a 10s safety timeout.
        let tx = engine.take_shutdown_sender().unwrap();
        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let entries = doc.get_many_by_prefix(b"result:").await.unwrap();
                if !entries.is_empty() {
                    return entries;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .expect("worker should emit a result within 10s");

        // Read and verify the result entry metadata. The blob
        // content itself lives in the same Node's blob store and is
        // easy to fetch from the test side, but for a correctness
        // check on the W9.1 pump we just need the entry to exist
        // alongside a matching claim entry.
        let result_entries = doc.get_many_by_prefix(b"result:").await.unwrap();
        assert_eq!(result_entries.len(), 1);
        assert_eq!(result_entries[0].key(), b"result:t-1");

        let claim_entries = doc.get_many_by_prefix(b"claim:").await.unwrap();
        assert_eq!(claim_entries.len(), 1, "expected exactly one claim entry");
        assert_eq!(claim_entries[0].key(), b"claim:t-1");

        let _ = tx.send(());
        handle.await.unwrap().unwrap();
    }

    // =============================================================
    // Sprint 22 Phase A — rate-limit engine gate integration tests
    // =============================================================

    /// Helper : spawn an engine pre-wired with a rate-limit policy
    /// written to a tempdir TOML file. Returns the engine plus
    /// handles the test needs to inject tasks (doc + author + coord
    /// keypair + project id) and to rewrite the policy for the
    /// hot-reload test. The tempdir is kept alive for the engine's
    /// lifetime.
    async fn build_engine_with_rate_limit_policy(
        policy_toml: &str,
    ) -> (
        Engine,
        TempDir,
        std::path::PathBuf,
        RsDocsClient,
        KeyPair,
        String,
    ) {
        let worker_config = WorkerConfig {
            engine: crate::config::Engine {
                task_poll_interval_ms: 100,
                max_concurrent_tasks: 1,
                state_flush_secs: 5,
            },
            ..WorkerConfig::default()
        };
        let keypair = KeyPair::generate();
        let allowlist = Allowlist::open_in_memory().unwrap();
        let project_id = "proj-rl".to_string();
        allowlist
            .enroll(crate::allowlist::NewProject {
                id: project_id.clone(),
                name: "rate-limit test".into(),
                enabled: true,
                budget_joules: 0,
                tasks_doc_ticket: None,
            })
            .unwrap();

        // Consent L4 so the filter admits any project id (the test
        // project id has nothing to do with the worker node id).
        let sbfb_tmp: TempDir = tempfile::tempdir().unwrap();
        let mut consent = ConsentConfig::default_for("rate-limit-test");
        consent.level = ConsentLevel::All;
        consent
            .save_atomic(&sbfb_tmp.path().join("consent.json"))
            .unwrap();

        let policy_path = sbfb_tmp.path().join("rate_limit_policy.toml");
        std::fs::write(&policy_path, policy_toml).unwrap();

        let boot = EngineBoot {
            worker_config,
            keypair: keypair.clone(),
            allowlist,
            data_dir: None,
            llm_override: Some(Box::new(StubBackend::new())),
            sbfb_home_override: Some(sbfb_tmp.path().to_path_buf()),
            rate_limit_policy_path_override: Some(policy_path.clone()),
        };
        let engine = Engine::new(boot).await.expect("engine boots");

        let docs = RsDocsClient::new(engine.node.docs());
        let coord_kp = KeyPair::generate();
        (engine, sbfb_tmp, policy_path, docs, coord_kp, project_id)
    }

    fn sign_test_task(id: &str, coord_kp: &KeyPair) -> Vec<u8> {
        let mut task = Task::new(
            id,
            "analysis",
            "hello",
            "stub-model:latest",
            5,
            1_000_000_000,
        );
        task.system_prompt = "".into();
        let task_entry = TaskEntry::sign(task, coord_kp).unwrap();
        serde_json::to_vec(&task_entry).unwrap()
    }

    // Sprint 76 Phase C: like `sign_test_task` but pins a cohort
    // `required_runtime` so the claim-gate is exercised. `model` stays
    // `stub-model:latest` (the StubBackend's model) so a worker's local
    // tuple resolves consistently.
    fn sign_test_task_with_runtime(
        id: &str,
        coord_kp: &KeyPair,
        required: Option<RuntimeTuple>,
    ) -> Vec<u8> {
        let mut task = Task::new(
            id,
            "analysis",
            "hello",
            "stub-model:latest",
            5,
            1_000_000_000,
        );
        task.system_prompt = "".into();
        task.required_runtime = required;
        let task_entry = TaskEntry::sign(task, coord_kp).unwrap();
        serde_json::to_vec(&task_entry).unwrap()
    }

    #[test]
    fn model_digest_is_name_hash_doc_note_s77() {
        // Sprint 76 Phase C doc-note (D3 etage 1): the worker computes
        // `model_digest = blake3(model NAME)` (`model_name_digest`),
        // NOT a GGUF weight-file hash. Ollama (ollama-rs 0.3.4) exposes
        // no clean file digest and `Verifier` has no prod caller, so
        // the name-hash regresses nothing; a real weight digest is
        // gated on `llm_llama_cpp` (Sprint 77 / D3 etage 2). This test
        // PINS the current contract so a future switch to a file hash
        // is a deliberate, reviewed break — not a silent drift.
        let model = "llama3.1:8b";
        // The seam the engine uses is `model_name_digest`, which is
        // exactly blake3 over the NAME bytes.
        assert_eq!(
            model_name_digest(model),
            blake3_hash(model.as_bytes()),
            "model_digest must remain blake3(model NAME) until a file-digest backend (S77)"
        );
        // And it is NOT a content hash of any weight bytes: a distinct
        // byte string (standing in for GGUF file bytes) must not
        // collide with the name digest — the documented discordance.
        let pretend_weight_bytes = b"GGUF\x00 pretend weight file contents";
        assert_ne!(
            model_name_digest(model),
            blake3_hash(pretend_weight_bytes),
            "documented discordance: name-hash != weight-file-hash (tighten in S77)"
        );
    }

    // P2-A-1 (S71->S73): spawns the engine pump and waits on a real-time loop
    // for `result:`. current_thread deadlocks under Windows `cargo test`
    // shared-process teardown (tokio #7049); multi_thread matches prod.
    // NB: unlike `rate_limit_gate_rejects/defer`, this test uses real time
    // (no tokio::time::pause), so multi_thread is both safe and required.
    // See PATTERNS §P54.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn rate_limit_gate_admits_fresh_tuple() {
        // A fresh (coord, worker, model) tuple must clear the gate
        // and produce a `claim:*` + `result:*` entry just like the
        // pre-S22 flow. This is the no-regression baseline : the
        // gate must not starve tasks under their normal budget.
        let (mut engine, _home, _policy_path, docs, coord_kp, project_id) =
            build_engine_with_rate_limit_policy(
                r#"
[default]
per_min = 60
burst_multiplier = 2.0
"#,
            )
            .await;

        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();
        let task_json = sign_test_task("t-fresh", &coord_kp);
        doc.set(author, b"task:t-fresh".to_vec(), task_json)
            .await
            .unwrap();
        engine.register_task_doc(&project_id, doc.clone());

        let tx = engine.take_shutdown_sender().unwrap();
        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        let result = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let entries = doc.get_many_by_prefix(b"result:").await.unwrap();
                if !entries.is_empty() {
                    return entries;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .expect("fresh tuple must produce a result within 10s");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key(), b"result:t-fresh");

        let claims = doc.get_many_by_prefix(b"claim:").await.unwrap();
        assert_eq!(claims.len(), 1, "claim must be emitted for fresh tuple");

        let _ = tx.send(());
        handle.await.unwrap().unwrap();
    }

    // Sprint 76 Phase C (D3 etage 1): a worker whose local runtime
    // tuple SATISFIES the task's `required_runtime` claims + executes
    // just like a no-cohort task — the gate must not starve a
    // homogeneous cohort. Real time + multi_thread for the same reason
    // as `rate_limit_gate_admits_fresh_tuple` (waits on `result:`).
    // See PATTERNS §P54.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cohort_gate_admits_homogeneous_worker() {
        let (mut engine, _home, _policy_path, docs, coord_kp, project_id) =
            build_engine_with_rate_limit_policy(
                r#"
[default]
per_min = 60
burst_multiplier = 2.0
"#,
            )
            .await;

        // The StubBackend reports runtime_family = "stub". A
        // requirement pinning family "stub" (other dims wildcard) is
        // satisfied, so the worker claims and produces a result.
        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();
        let required = Some(RuntimeTuple {
            model: String::new(),
            quant: String::new(),
            runtime_family: "stub".to_string(),
        });
        let task_json = sign_test_task_with_runtime("t-homog", &coord_kp, required);
        doc.set(author, b"task:t-homog".to_vec(), task_json)
            .await
            .unwrap();
        engine.register_task_doc(&project_id, doc.clone());

        let tx = engine.take_shutdown_sender().unwrap();
        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        let result = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let entries = doc.get_many_by_prefix(b"result:").await.unwrap();
                if !entries.is_empty() {
                    return entries;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .expect("homogeneous worker must produce a result within 10s");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key(), b"result:t-homog");
        let claims = doc.get_many_by_prefix(b"claim:").await.unwrap();
        assert_eq!(claims.len(), 1, "homogeneous worker must emit a claim");

        let _ = tx.send(());
        handle.await.unwrap().unwrap();
    }

    // P2-A-1 note: this test MUST stay current_thread (virtual time via
    // `tokio::time::pause` + `advance`, deterministic and immune to the
    // Windows real-time poll-loop hang). See PATTERNS §P54.
    #[tokio::test]
    async fn cohort_gate_blocks_non_homogeneous_worker() {
        // Sprint 76 Phase C (D3 etage 1): a worker whose local runtime
        // tuple does NOT satisfy the task's `required_runtime` must NOT
        // claim — the task stays live on the doc for a homogeneous
        // peer, exactly like the rate-limit defer. A generous rate
        // budget isolates the cohort gate as the sole cause (cf.
        // `cohort_gate_admits_homogeneous_worker`, same harness,
        // matching tuple => claims).
        let (mut engine, _home, _policy_path, docs, coord_kp, project_id) =
            build_engine_with_rate_limit_policy(
                r#"
[default]
per_min = 60
burst_multiplier = 2.0
"#,
            )
            .await;

        // The StubBackend reports runtime_family = "stub"; require
        // "ollama" => mismatch => no claim.
        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();
        let required = Some(RuntimeTuple {
            model: String::new(),
            quant: String::new(),
            runtime_family: "ollama".to_string(),
        });
        let task_json = sign_test_task_with_runtime("t-heterog", &coord_kp, required);
        doc.set(author, b"task:t-heterog".to_vec(), task_json)
            .await
            .unwrap();
        engine.register_task_doc(&project_id, doc.clone());

        let tx = engine.take_shutdown_sender().unwrap();
        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        tokio::time::pause();
        for _ in 0..15 {
            tokio::time::advance(Duration::from_millis(100)).await;
            tokio::task::yield_now().await;
        }

        // Task stays live; no claim, no result.
        let tasks = doc.get_many_by_prefix(b"task:").await.unwrap();
        assert_eq!(
            tasks.len(),
            1,
            "non-homogeneous worker must leave the task live for a peer"
        );
        assert_eq!(tasks[0].key(), b"task:t-heterog");
        let claims = doc.get_many_by_prefix(b"claim:").await.unwrap();
        assert!(
            claims.is_empty(),
            "non-homogeneous worker must not emit a claim"
        );
        let results = doc.get_many_by_prefix(b"result:").await.unwrap();
        assert!(
            results.is_empty(),
            "non-homogeneous worker must not emit a result"
        );

        let _ = tx.send(());
        tokio::time::resume();
        handle.await.unwrap().unwrap();
    }

    // P2-A-1 note: this test MUST stay current_thread. It drives the pump
    // with virtual time (`tokio::time::pause` + `advance`), which is
    // current_thread-only and fully deterministic — so it is immune to the
    // Windows real-time poll-loop hang and must NOT be switched to
    // multi_thread. See PATTERNS §P54.
    #[tokio::test]
    async fn rate_limit_gate_rejects_saturated_tuple() {
        // Saturate the tuple through direct `rate_limiter.check`
        // calls BEFORE the engine ticks. The tuple key must match
        // what the engine derives at admission time :
        // `(hex(coord_pubkey), hex(self.worker_pubkey), model)`.
        // A saturated tuple must NOT get a `claim:*` entry — the
        // engine defers the task and moves on.
        let policy_toml = r#"
[default]
per_min = 2
burst_multiplier = 1.0
"#;
        let (mut engine, _home, _policy_path, docs, coord_kp, project_id) =
            build_engine_with_rate_limit_policy(policy_toml).await;

        // Pre-saturate the engine's rate limiter on the exact tuple
        // the engine will see when it admits `t-saturated` below.
        let rl = Arc::clone(&engine.rate_limiter);
        let rate_key = RateKey::new(
            hex::encode(coord_kp.public_bytes()),
            hex::encode(engine.keypair.public_bytes()),
            "stub-model:latest",
        );
        rl.check(&rate_key).unwrap();
        rl.check(&rate_key).unwrap();
        assert!(
            rl.check(&rate_key).is_err(),
            "tuple must be saturated before the task lands"
        );

        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();
        let task_json = sign_test_task("t-saturated", &coord_kp);
        doc.set(author, b"task:t-saturated".to_vec(), task_json)
            .await
            .unwrap();
        engine.register_task_doc(&project_id, doc.clone());

        let tx = engine.take_shutdown_sender().unwrap();
        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        // Advance time deterministically so the engine sees enough
        // ticks to observe the task. Using tokio::time::pause +
        // advance avoids wall-clock sensitivity across fast/slow CI.
        tokio::time::pause();
        for _ in 0..15 {
            tokio::time::advance(Duration::from_millis(100)).await;
            tokio::task::yield_now().await;
        }

        let claims = doc.get_many_by_prefix(b"claim:").await.unwrap();
        assert!(
            claims.is_empty(),
            "saturated tuple must not produce a claim entry (got {})",
            claims.len()
        );

        let _ = tx.send(());
        tokio::time::resume();
        handle.await.unwrap().unwrap();
    }

    // P2-A-1 note: this test MUST stay current_thread (virtual time via
    // `tokio::time::pause` + `advance`, current_thread-only and
    // deterministic). It is immune to the Windows real-time poll-loop hang
    // and must NOT be switched to multi_thread. See PATTERNS §P54.
    #[tokio::test]
    async fn rate_limit_gate_defer_preserves_task() {
        // A rate-limited task must remain live on the doc — no
        // claim emitted, no result emitted, but the TaskEntry still
        // exists. This is the contract that keeps throughput lossy
        // but not wasteful : once the GCRA bucket replenishes, a
        // future tick can pick the same task up.
        let (mut engine, _home, _policy_path, docs, coord_kp, project_id) =
            build_engine_with_rate_limit_policy(
                r#"
[default]
per_min = 1
burst_multiplier = 1.0
"#,
            )
            .await;

        let rl = Arc::clone(&engine.rate_limiter);
        let rate_key = RateKey::new(
            hex::encode(coord_kp.public_bytes()),
            hex::encode(engine.keypair.public_bytes()),
            "stub-model:latest",
        );
        rl.check(&rate_key).unwrap();
        assert!(rl.check(&rate_key).is_err());

        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();
        let task_json = sign_test_task("t-deferred", &coord_kp);
        doc.set(author, b"task:t-deferred".to_vec(), task_json)
            .await
            .unwrap();
        engine.register_task_doc(&project_id, doc.clone());

        let tx = engine.take_shutdown_sender().unwrap();
        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        tokio::time::pause();
        for _ in 0..15 {
            tokio::time::advance(Duration::from_millis(100)).await;
            tokio::task::yield_now().await;
        }

        let tasks = doc.get_many_by_prefix(b"task:").await.unwrap();
        assert_eq!(
            tasks.len(),
            1,
            "deferred task must remain on the doc for a future tick"
        );
        assert_eq!(tasks[0].key(), b"task:t-deferred");

        let claims = doc.get_many_by_prefix(b"claim:").await.unwrap();
        assert!(
            claims.is_empty(),
            "deferred task must not produce a claim entry"
        );
        let results = doc.get_many_by_prefix(b"result:").await.unwrap();
        assert!(
            results.is_empty(),
            "deferred task must not produce a result entry"
        );

        let _ = tx.send(());
        tokio::time::resume();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn rate_limit_gate_reloads_live_policy() {
        // Boot with a cramped policy that saturates the tuple after
        // a single check, rewrite the TOML with a generous budget,
        // wait for the notify watcher to propagate the reload via
        // the `swap_policy` callback, and confirm the same tuple is
        // now admissible. This exercises the full hot-reload path
        // (file watch → parse → swap_policy → GCRA rebuild).
        let (engine, _home, policy_path, _docs, coord_kp, _project_id) =
            build_engine_with_rate_limit_policy(
                r#"
[default]
per_min = 1
burst_multiplier = 1.0
"#,
            )
            .await;

        let rl = Arc::clone(&engine.rate_limiter);
        let rate_key = RateKey::new(
            hex::encode(coord_kp.public_bytes()),
            hex::encode(engine.keypair.public_bytes()),
            "stub-model:latest",
        );
        // Saturate under the initial policy.
        rl.check(&rate_key).unwrap();
        assert!(rl.check(&rate_key).is_err());

        // Rewrite the TOML with a far higher budget. The watcher
        // picks up the Modify event (debounced 50 ms) and invokes
        // the on_reload callback which calls
        // `rate_limiter.swap_policy`, rebuilding the GCRA state.
        std::fs::write(
            &policy_path,
            r#"
[default]
per_min = 600
burst_multiplier = 1.0
"#,
        )
        .unwrap();

        // Wait for the reload — bounded to 3s to surface a
        // regression quickly without flaking on slow CI.
        let reloaded = {
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            let mut applied = false;
            while std::time::Instant::now() < deadline {
                if rl.check(&rate_key).is_ok() {
                    applied = true;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            applied
        };
        assert!(
            reloaded,
            "hot-reload via swap_policy must free the saturated tuple within 3s"
        );

        // Engine shutdown path. This test never injects a real
        // task — the assertion is purely on the in-process
        // rate_limiter state.
        let _ = engine.node.shutdown().await;
    }

    #[test]
    fn rate_limit_policy_sample_loader_smoke() {
        // Parse the checked-in sample TOML to catch schema drift
        // between `RateLimitPolicy` and the operator-facing example.
        // A regression here means the sample file documents a
        // shape the parser refuses — very confusing for operators.
        let sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("configs/rate_limit_policy.toml.sample");
        let parsed =
            crate::rate_limit_policy_loader::load_rate_limit_policy_from(&sample_path).unwrap();
        assert!(
            parsed.default.per_min > 0,
            "sample default tier must have a positive per_min"
        );
        assert!(
            parsed.default.burst_multiplier > 0.0,
            "sample default tier must have a positive burst multiplier"
        );
    }

    // P2-A-1 (S71->S73): spawns the engine pump; current_thread deadlocks
    // under Windows `cargo test` shared-process teardown (tokio #7049).
    // multi_thread matches prod (worker binary). See PATTERNS §P54.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn engine_shuts_down_gracefully() {
        let mut engine = build_engine_with_stub_ollama().await;
        let tx = engine.take_shutdown_sender().unwrap();
        let rx = engine.state_rx();
        let handle = tokio::spawn(async move { engine.run_until_shutdown().await });

        let _ = tx.send(());

        handle
            .await
            .expect("engine task joins")
            .expect("engine exits cleanly");

        // Final state must be Shutdown (the loop applied it
        // on the shutdown branch).
        assert_eq!(rx.borrow().label(), "shutdown");
    }
}
