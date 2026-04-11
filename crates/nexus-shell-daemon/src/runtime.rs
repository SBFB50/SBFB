//! Phase A runtime: boot, HTTP serve, graceful shutdown.
//!
//! [`DaemonRuntime::start`] is the single entry point the binary
//! uses to bring up the daemon. It performs the ordered Phase A
//! boot sequence:
//!
//! 1. [`nexus_shell_daemon_core::registry::check_stale_or_bail`]
//!    — refuse to boot if a live daemon is already running.
//! 2. `nexus_core_rs::create_node()` — spin up the iroh endpoint +
//!    protocol router + blobs/docs/gossip stack. Phase A does not
//!    consume any of those protocols, but the boot ordering (node
//!    first, HTTP second) matches what Phase C will need.
//! 3. Bind a TCP listener on `(api_host, 0)` so the OS picks an
//!    ephemeral port. The real port is then written into
//!    `running.json` alongside the pid.
//! 4. Write `running.json`.
//! 5. Spawn an axum `serve` task on a oneshot shutdown channel.
//!
//! [`DaemonRuntime::wait_shutdown`] yields on ctrl+c via
//! `tokio::signal::ctrl_c`. [`DaemonRuntime::shutdown`] drives
//! the reverse order: stop HTTP → shutdown iroh → remove
//! `running.json`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{anyhow, Context, Result};
use nexus_core_rs::Node;
use nexus_shell_daemon_core::config::ShellDaemonPaths;
use nexus_shell_daemon_core::registry::{
    self, new_running_state, remove_running, write_running, StaleOutcome,
};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::http::{build_router, DaemonHttpState};

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
/// Owns the iroh `Node`, the running.json path (so it can
/// remove it on shutdown), the HTTP serve task, and a oneshot
/// sender that signals axum to stop accepting new connections.
pub struct DaemonRuntime {
    node: Option<Node>,
    running_json: PathBuf,
    http_handle: JoinHandle<()>,
    http_shutdown: Option<oneshot::Sender<()>>,
    bound_addr: std::net::SocketAddr,
}

impl DaemonRuntime {
    /// Execute the Phase A boot sequence and return a live
    /// [`DaemonRuntime`].
    ///
    /// The caller is expected to either:
    /// - keep the returned handle and call
    ///   [`DaemonRuntime::wait_shutdown`] + [`DaemonRuntime::shutdown`]
    ///   in sequence (the binary's normal path), or
    /// - drop it, which triggers `Drop` but cannot drive the
    ///   graceful iroh shutdown — that only works through the
    ///   async `shutdown()` method.
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

        // 2. Boot the iroh endpoint + protocol router.
        //
        // Phase A doesn't consume any of docs/gossip/blobs yet,
        // but we still boot the full stack so Phase C can plug
        // its curator pipeline in without a boot-order shuffle.
        let node = nexus_core_rs::create_node()
            .await
            .context("failed to boot iroh node for shell daemon")?;
        let node_id = node.node_id();
        info!(node_id = %node_id, "shell daemon iroh node ready");

        // 3. Bind the TCP listener. An empty host in the config
        // was clamped to 127.0.0.1 at load time (see
        // `ShellDaemonConfig::clamped`); defend-in-depth here
        // too so a future bypass of `load` cannot slip through.
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

        // 5. Build the shared HTTP state + spawn the serve task.
        let http_state = Arc::new(DaemonHttpState {
            node_id,
            daemon_version: opts.daemon_version.clone(),
            boot_time: SystemTime::now(),
            api_host: host,
            api_port: bound_addr.port(),
        });
        let router = build_router(http_state);

        let (http_shutdown_tx, http_shutdown_rx) = oneshot::channel::<()>();
        let http_handle = tokio::spawn(async move {
            let serve = axum::serve(listener, router).with_graceful_shutdown(async move {
                // On receiving the shutdown signal (or the sender
                // being dropped, which `.await` also unblocks on)
                // axum stops accepting new connections and lets
                // in-flight ones drain.
                let _ = http_shutdown_rx.await;
            });
            if let Err(e) = serve.await {
                warn!(error = %e, "axum serve exited with an error");
            }
        });

        Ok(Self {
            node: Some(node),
            running_json: running_json_path,
            http_handle,
            http_shutdown: Some(http_shutdown_tx),
            bound_addr,
        })
    }

    /// Return the real bound socket address. Exposed so the
    /// binary's handler can print the listening port on
    /// boot for operators.
    pub fn bound_addr(&self) -> std::net::SocketAddr {
        self.bound_addr
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
    /// 1. Signal axum to stop + await the serve task.
    /// 2. Shutdown the iroh node (drains router + closes
    ///    endpoint, per the Sprint 2 audit fix).
    /// 3. Remove `running.json`.
    ///
    /// Any individual step may fail; the function logs the
    /// failure and still runs the remaining steps so the user
    /// never sees a dangling `running.json` + dangling iroh
    /// endpoint combo from a half-executed shutdown.
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(tx) = self.http_shutdown.take() {
            let _ = tx.send(());
        }
        if let Err(e) = (&mut self.http_handle).await {
            warn!(error = %e, "HTTP serve task join failed");
        }

        if let Some(node) = self.node.take() {
            if let Err(e) = node.shutdown().await {
                warn!(error = %e, "iroh node shutdown returned an error");
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
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
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

        // `expect_err` would require `DaemonRuntime: Debug`;
        // match on the Result directly instead so the Ok arm
        // can panic cleanly on failure.
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

        // Write a stale running.json with a pid that is
        // guaranteed not to be our test binary.
        opts.paths.ensure_dirs().unwrap();
        let stale_state = new_running_state(
            "0".repeat(64),
            "127.0.0.1".to_string(),
            1,
            "0.0.0-stale".to_string(),
        );
        // Force pid 0 which is never a live shell daemon.
        let stale_state = nexus_shell_daemon_core::registry::RunningState {
            pid: 0,
            ..stale_state
        };
        write_running(&stale_state, &running_json).unwrap();

        // A fresh start should overwrite the stale file and
        // bring up a live daemon.
        let rt = DaemonRuntime::start(opts).await.expect("overwrite stale");
        assert!(running_json.exists());
        rt.shutdown().await.unwrap();
    }
}
