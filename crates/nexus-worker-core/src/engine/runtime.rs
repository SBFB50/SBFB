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
//! ## Scope for Sprint 3 W9
//!
//! The W9 milestone targets the *boot* path and the *loop
//! shell*:
//!
//! - Construct the engine from a fully-populated
//!   [`EngineConfig`]
//! - Boot the iroh [`Node`] with the persistent worker keypair
//! - Run an initial Ollama health-check and GPU probe (both
//!   logged, neither fatal)
//! - Apply `Start` to the state machine, broadcasting the
//!   resulting `WorkerState` on a `tokio::sync::watch` channel
//! - Iterate the allowlist on every poll tick and log enrolled
//!   projects (so the TUI has something real to show)
//! - Handle graceful shutdown via a `oneshot` channel
//!
//! The actual task claim / execute / result write-back flow is
//! intentionally **not** in this commit. That code depends on
//! the Sprint 4 coordinator writing `task:*` entries into a
//! project doc and on the invite carrying a [`DocTicket`]
//! string. Both arrive in a W9.1 follow-up. The engine here
//! reserves a clean place for that code to drop in — search
//! for the `TODO(W9.1)` markers in [`Engine::run_until_shutdown`].
//!
//! ## Channels
//!
//! The engine exposes two channels:
//!
//! - `state_rx: watch::Receiver<WorkerState>` — the current
//!   state, updated every transition. W10 subscribes.
//! - `shutdown_tx: oneshot::Sender<()>` — signal graceful stop.
//!   The `nexus-worker` binary wires this to a SIGINT handler.

use std::sync::Arc;
use std::time::Duration;

use nexus_core_rs::{create_node_with_config, KeyPair, Node, NodeConfig};
use tokio::sync::{oneshot, watch, Mutex};
use tracing::{debug, error, info, warn};

use crate::allowlist::Allowlist;
use crate::config::WorkerConfig;
use crate::engine::state::{StateMachine, WorkerEvent, WorkerState};
use crate::gpu::{create_monitor, GpuInfo, GpuMonitor};
use crate::ollama::{HealthCheck, OllamaClient, OllamaHttpClient};

// =================================================================
// Boot-time configuration
// =================================================================

/// Everything the engine needs to boot.
///
/// Constructed by the `nexus-worker` binary (which has already
/// loaded `worker.toml`, resolved [`crate::config::WorkerPaths`],
/// and loaded the persistent Ed25519 keypair from disk) and
/// passed to [`Engine::new`].
pub struct EngineBoot {
    pub worker_config: WorkerConfig,
    pub keypair: KeyPair,
    pub allowlist: Allowlist,
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
    ollama: Box<dyn OllamaClient>,
    gpu: Box<dyn GpuMonitor>,
    gpu_info: Vec<GpuInfo>,
    worker_config: WorkerConfig,
    shutdown_rx: Option<oneshot::Receiver<()>>,
    shutdown_tx_handle: Option<oneshot::Sender<()>>,
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
        } = boot;

        info!(
            worker = %worker_config.identity.name,
            "booting nexus-worker engine"
        );

        // --- iroh Node ---
        let node_cfg = NodeConfig::default().with_secret_key(keypair.secret_bytes());
        let node = create_node_with_config(node_cfg)
            .await
            .map_err(|e| anyhow::anyhow!("failed to boot iroh node for worker keypair: {e}"))?;
        info!(node_id = %node.node_id(), "iroh endpoint ready");

        // --- Ollama ---
        let ollama: Box<dyn OllamaClient> =
            Box::new(OllamaHttpClient::from_config(&worker_config.ollama)?);
        match ollama.healthcheck().await {
            HealthCheck::Ready { models } => {
                info!(
                    endpoint = %worker_config.ollama.endpoint,
                    model_count = models.len(),
                    "ollama healthcheck passed"
                );
            }
            HealthCheck::NotRunning { endpoint, hint, .. } => {
                warn!(
                    %endpoint,
                    %hint,
                    "ollama is not running; engine will continue but cannot serve tasks until it comes up"
                );
            }
            HealthCheck::Error { endpoint, reason } => {
                warn!(%endpoint, %reason, "ollama healthcheck returned an error");
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

        Ok(Self {
            node,
            state: Arc::new(Mutex::new(state)),
            state_tx,
            state_rx,
            allowlist,
            ollama,
            gpu,
            gpu_info,
            worker_config,
            shutdown_rx: Some(shutdown_rx),
            shutdown_tx_handle: Some(shutdown_tx),
        })
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

        // The cached initial state after Start.
        let mut shutdown_rx = shutdown_rx;

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
                    if self.state_rx.borrow().is_terminal() {
                        break;
                    }
                }
            }
        }

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
    /// 2. List enabled projects from the allowlist. The W9.1
    ///    follow-up will dial each project's coordinator and
    ///    import the task doc here.
    /// 3. Task claim / execute / result write-back — deferred
    ///    to W9.1 (requires a DocTicket in the invite).
    async fn tick(&self) {
        // Snapshot state before the tick so we can decide
        // transitions without holding the lock across awaits.
        let current = self.state_rx.borrow().clone();

        match current {
            WorkerState::Connecting => {
                // Transition to Processing as soon as Ollama is
                // reachable and at least one project is
                // enrolled. Otherwise stay in Connecting; the
                // TUI will show the user what's missing.
                let hc = self.ollama.healthcheck().await;
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
                // TODO(W9.1): scan each enabled project doc for
                // task: entries, claim and execute them via
                // self.ollama.generate(), sign the result and
                // write back via doc.set("result:{id}", ...).
                //
                // For W9 we just re-verify the preconditions and
                // back off to Connecting if Ollama went away.
                let hc = self.ollama.healthcheck().await;
                if !hc.is_ready() {
                    warn!("ollama is no longer reachable, dropping back to Connecting");
                    self.apply_event(WorkerEvent::Pause).await;
                    self.apply_event(WorkerEvent::Resume).await;
                }
            }
            WorkerState::PullingModel { .. } => {
                // W5 retry policy already handles this via
                // exponential backoff inside OllamaHttpClient;
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
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ollama::{GenerateParams, GenerateResponse, OllamaResult};
    use async_trait::async_trait;

    /// Deterministic stub that behaves like a healthy Ollama
    /// without touching the network.
    struct StubOllama {
        models: Vec<String>,
    }

    #[async_trait]
    impl OllamaClient for StubOllama {
        async fn healthcheck(&self) -> HealthCheck {
            HealthCheck::Ready {
                models: self.models.clone(),
            }
        }

        async fn generate(&self, params: GenerateParams) -> OllamaResult<GenerateResponse> {
            Ok(GenerateResponse {
                text: format!("STUB[{}]: {}", params.model, params.prompt),
                model: params.model,
                prompt_tokens: Some(4),
                completion_tokens: Some(10),
            })
        }
    }

    async fn build_engine_with_stub_ollama() -> Engine {
        let worker_config = WorkerConfig::default();
        let keypair = KeyPair::generate();
        let allowlist = Allowlist::open_in_memory().unwrap();

        let boot = EngineBoot {
            worker_config: worker_config.clone(),
            keypair,
            allowlist,
        };
        let mut engine = Engine::new(boot).await.expect("engine boots");
        // Replace the real Ollama client with a stub so tests
        // do not depend on the daemon being reachable.
        engine.ollama = Box::new(StubOllama {
            models: vec!["stub-model:latest".into()],
        });
        engine
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

    #[tokio::test]
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

    #[tokio::test]
    async fn engine_transitions_to_processing_when_project_is_enrolled() {
        // Build an engine whose allowlist has one project, so
        // the tick's "ollama_ready && enabled > 0" branch
        // fires and the state machine moves Connecting →
        // Processing within a handful of ticks.
        let worker_config = WorkerConfig {
            engine: crate::config::Engine {
                task_poll_interval_ms: 100,
                max_concurrent_tasks: 1,
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
        };
        let mut engine = Engine::new(boot).await.expect("engine boots");
        engine.ollama = Box::new(StubOllama {
            models: vec!["stub-model:latest".into()],
        });

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

    #[tokio::test]
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
