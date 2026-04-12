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

use anyhow::{anyhow, Context, Result};
use nexus_core_rs::{create_node, GossipClient, GossipEvent, Node};
use nexus_shell_daemon_core::browse::{
    BrowseAggregator, BrowseAggregatorHandle, BrowseEntry, BrowseSource, BrowseStatus,
};
use nexus_shell_daemon_core::config::ShellDaemonPaths;
use nexus_shell_daemon_core::iroh_runtime::{
    curator_topic_id, CuratorRuntime, CuratorRuntimeError, CuratorRuntimeHandle,
};
use nexus_shell_daemon_core::publish;
use nexus_shell_daemon_core::registry::{
    self, new_running_state, remove_running, write_running, StaleOutcome,
};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::http::{build_router, DaemonHttpState, GossipSenderHandle};

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
/// test harness and Phase D browse path need access to it
/// through [`DaemonRuntime::curator_runtime`], so the field is
/// explicitly allowed as dead_code. Phase D will reach into it
/// to filter browse entries by the current attention set.
pub struct DaemonRuntime {
    node: Option<Arc<Node>>,
    #[allow(dead_code)]
    curator_runtime: CuratorRuntimeHandle,
    running_json: PathBuf,
    http_handle: JoinHandle<()>,
    http_shutdown: Option<oneshot::Sender<()>>,
    gossip_handle: JoinHandle<()>,
    gossip_shutdown: Option<oneshot::Sender<()>>,
    bound_addr: std::net::SocketAddr,
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
        let node = create_node()
            .await
            .context("failed to boot iroh node for shell daemon")?;
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

        // 5b. Construct the Phase D browse aggregator. It is
        //     cheap (just a DashMap + two duration knobs) so
        //     it is always instantiated at boot; GET /browse
        //     reaches through this handle + the Arc<Node> to
        //     probe each project's reachability on demand.
        let browse_aggregator: BrowseAggregatorHandle = Arc::new(BrowseAggregator::new());

        // 5c. Sprint 11 Phase A: shared gossip sender slot. Set to
        //     `Some(sender)` once the gossip task joins the topic.
        let gossip_sender: GossipSenderHandle = Arc::new(tokio::sync::RwLock::new(None));

        // 6. Build the shared HTTP state + spawn the serve task.
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
        });
        let router = build_router(http_state);

        let (http_shutdown_tx, http_shutdown_rx) = oneshot::channel::<()>();
        let http_handle = tokio::spawn(async move {
            let serve = axum::serve(listener, router).with_graceful_shutdown(async move {
                let _ = http_shutdown_rx.await;
            });
            if let Err(e) = serve.await {
                warn!(error = %e, "axum serve exited with an error");
            }
        });

        // 7. Spawn the gossip subscribe task. It joins the
        //    curator topic, streams events, and forwards each
        //    message body to the curator runtime. The oneshot
        //    lets shutdown signal a clean exit.
        let (gossip_shutdown_tx, gossip_shutdown_rx) = oneshot::channel::<()>();
        let gossip_handle = spawn_gossip_subscribe_task(
            Arc::clone(&node),
            Arc::clone(&curator_runtime),
            Arc::clone(&browse_aggregator),
            Arc::clone(&gossip_sender),
            gossip_shutdown_rx,
        );

        Ok(Self {
            node: Some(node),
            curator_runtime,
            running_json: running_json_path,
            http_handle,
            http_shutdown: Some(http_shutdown_tx),
            gossip_handle,
            gossip_shutdown: Some(gossip_shutdown_tx),
            bound_addr,
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

    /// Block on ctrl+c, returning when the user (or the test
    /// harness) signals shutdown.
    pub async fn wait_shutdown(&self) -> Result<()> {
        tokio::signal::ctrl_c()
            .await
            .context("failed to install ctrl+c handler")?;
        info!("ctrl+c received, initiating shell daemon shutdown");
        Ok(())
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
        if let Some(tx) = self.gossip_shutdown.take() {
            let _ = tx.send(());
        }
        if let Err(e) = (&mut self.gossip_handle).await {
            warn!(error = %e, "gossip subscribe task join failed");
        }

        if let Some(tx) = self.http_shutdown.take() {
            let _ = tx.send(());
        }
        if let Err(e) = (&mut self.http_handle).await {
            warn!(error = %e, "HTTP serve task join failed");
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
// Gossip subscribe task
// =================================================================

/// Spawn the background task that joins the curator gossip
/// topic and forwards every message body to the curator
/// runtime.
///
/// Lives in its own function (rather than inlined into
/// `DaemonRuntime::start`) so the long boot body stays readable
/// and so the task's precise lifecycle is easy to audit.
fn spawn_gossip_subscribe_task(
    node: Arc<Node>,
    curator_runtime: CuratorRuntimeHandle,
    browse_aggregator: BrowseAggregatorHandle,
    gossip_sender_slot: GossipSenderHandle,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        // Step 1: derive the topic id and join. We do not pass
        // any bootstrap peer ids — the daemon relies on pkarr
        // discovery + already-open peer connections to find
        // neighbours on the topic swarm. If `join_topic` hangs
        // because there are zero peers reachable, the shutdown
        // oneshot will wake us up.
        let gossip = GossipClient::new(node.gossip());
        let topic_id = curator_topic_id();

        let topic = tokio::select! {
            join = gossip.join_topic(topic_id, vec![]) => match join {
                Ok(t) => t,
                Err(e) => {
                    warn!(error = %e, "curator gossip join_topic failed — subscribe task exits");
                    return;
                }
            },
            _ = &mut shutdown_rx => {
                debug!("curator gossip task shut down before join_topic completed");
                return;
            }
        };
        info!("curator gossip topic joined");

        let (sender, mut receiver) = topic.split();

        // Sprint 11 Phase A: store the sender in the shared slot
        // so POST /publish can broadcast project announcements.
        {
            let mut lock = gossip_sender_slot.write().await;
            *lock = Some(sender);
        }

        // Step 2: drain events until shutdown is signalled or
        // the receiver stream ends.
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
                            // Sprint 11 Phase A: dispatch based on
                            // message type before curator processing.
                            if publish::is_project_announcement(&content) {
                                handle_project_announcement(&browse_aggregator, &content);
                            } else {
                                handle_announcement(&curator_runtime, &node, &content).await;
                            }
                        }
                        Ok(Some(GossipEvent::NeighborUp { node_id })) => {
                            debug!(neighbor = %node_id, "gossip neighbor up");
                        }
                        Ok(Some(GossipEvent::NeighborDown { node_id })) => {
                            debug!(neighbor = %node_id, "gossip neighbor down");
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
                _ = &mut shutdown_rx => {
                    info!("gossip task shut down on signal");
                    break;
                }
            }
        }
    })
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

/// Handle a gossip message identified as a project announcement.
/// Parse, validate, and insert into the browse aggregator.
///
/// Sprint 11 Phase A.
fn handle_project_announcement(browse_aggregator: &BrowseAggregatorHandle, content: &[u8]) {
    match publish::ProjectAnnouncement::from_gossip_bytes(content) {
        Ok(ann) => {
            let entry = BrowseEntry {
                project_id: ann.node_id.clone(),
                project_name: ann.project_name,
                category: ann.category,
                description: ann.description,
                curator_pubkey: String::new(),
                curator_name: "Self-published".into(),
                source: BrowseSource::Direct,
                status: BrowseStatus::Unknown,
                last_probed_at: None,
            };
            browse_aggregator.add_direct_entry(entry);
            info!(
                node_id = %ann.node_id,
                "project announcement accepted via gossip"
            );
        }
        Err(e) => {
            warn!(error = %e, "failed to parse project announcement");
        }
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
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
}
