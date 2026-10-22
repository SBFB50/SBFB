// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-daemon integration test harness for SBFB.
//!
//! Spawns N isolated `nexus-shell-daemon` processes, each in its
//! own temporary directory with distinct iroh keypairs. Provides
//! health checks, port discovery via `running.json`, and graceful
//! shutdown via SIGINT / TerminateProcess.
//!
//! [`DaemonHandle::spawn_with`] adds what a test needs to stay off the
//! public network and to model a node going offline: a `config.toml`
//! written before the first boot (so the first-boot public curator and
//! mailbox defaults are never added), a hermetic discovery posture
//! ([`HERMETIC_ENV`]), and [`DaemonHandle::restart`] on the same data
//! directories.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tempfile::TempDir;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(250);
const HEALTH_TIMEOUT: Duration = Duration::from_secs(30);

/// Overrides the daemon binary the harness launches (default: the
/// `nexus-shell-daemon` next to the test binary's profile directory).
pub const DAEMON_BIN_ENV: &str = "SBFB_TEST_DAEMON_BIN";

/// `RUST_LOG` handed to every daemon (default `warn`); its logs land in
/// `<root>/logs/` and `<root>/daemon-<boot>.log` (stdout + stderr).
pub const DAEMON_LOG_ENV: &str = "SBFB_TEST_DAEMON_LOG";

/// Environment of a hermetic daemon: zero-n0 discovery pointed at a
/// loopback port nothing listens on, so the endpoint neither homes on a
/// public relay nor publishes to / resolves from a public pkarr relay.
/// Peers are then reachable only through the direct addresses carried by
/// tickets. `SBFB_DEV_MODE=1` is what lets the URL policy accept a
/// loopback relay; it has no other effect.
pub const HERMETIC_ENV: &[(&str, &str)] = &[
    ("SBFB_ZERO_N0", "1"),
    ("SBFB_ZERO_N0_PKARR_RELAYS", "https://127.0.0.1:9/pkarr"),
    ("SBFB_CUSTOM_RELAYS", "https://127.0.0.1:9"),
    ("SBFB_DEV_MODE", "1"),
];

/// How a daemon is launched by [`DaemonHandle::spawn_with`].
#[derive(Debug, Clone, Default)]
pub struct SpawnOptions {
    /// Written to `<root>/shell-daemon/config.toml` before the first
    /// boot. The daemon only adds its public first-boot defaults when no
    /// config file exists, so any value here keeps them out.
    pub config_toml: Option<String>,
    /// Launch with [`HERMETIC_ENV`].
    pub hermetic: bool,
}

fn daemon_binary() -> PathBuf {
    if let Some(bin) = std::env::var_os(DAEMON_BIN_ENV) {
        return PathBuf::from(bin);
    }
    let mut path = std::env::current_exe()
        .expect("current_exe")
        .parent()
        .expect("parent")
        .parent()
        .expect("grandparent")
        .to_path_buf();
    if cfg!(windows) {
        path.push("nexus-shell-daemon.exe");
    } else {
        path.push("nexus-shell-daemon");
    }
    path
}

#[derive(Debug, Deserialize)]
struct RunningState {
    pub node_id: String,
    pub api_port: u16,
}

#[derive(Debug, Deserialize)]
struct HealthResponse {
    pub status: String,
}

pub struct DaemonHandle {
    proc: Child,
    root: TempDir,
    sbfb_home: TempDir,
    opts: SpawnOptions,
    boots: u32,
    pub http_port: u16,
    pub node_id: String,
    pub auth_token: String,
}

impl DaemonHandle {
    async fn spawn() -> Result<Self> {
        Self::spawn_with(SpawnOptions::default()).await
    }

    /// Launch one daemon in fresh `NEXUS_GRID_ROOT` / `SBFB_HOME`
    /// temporary directories and wait until it answers `/health`.
    pub async fn spawn_with(opts: SpawnOptions) -> Result<Self> {
        let root = TempDir::new().context("create NEXUS_GRID_ROOT tempdir")?;
        let sbfb_home = TempDir::new().context("create SBFB_HOME tempdir")?;

        if let Some(cfg) = &opts.config_toml {
            let dir = root.path().join("shell-daemon");
            std::fs::create_dir_all(&dir).context("create shell-daemon dir")?;
            std::fs::write(dir.join("config.toml"), cfg).context("write config.toml")?;
        }

        let proc = launch(root.path(), sbfb_home.path(), &opts, 0)?;
        let mut handle = Self {
            proc,
            root,
            sbfb_home,
            opts,
            boots: 0,
            http_port: 0,
            node_id: String::new(),
            auth_token: String::new(),
        };
        handle.attach().await?;
        Ok(handle)
    }

    /// Stop the process and boot it again on the SAME data directories
    /// (same node key, same stores). The HTTP port changes; the node id
    /// must not.
    pub async fn restart(&mut self) -> Result<()> {
        self.shutdown().await?;
        // A killed daemon leaves its running.json behind: remove it so
        // the wait below reads the new boot's port, not the old one.
        let _ = std::fs::remove_file(self.running_json());
        self.boots += 1;
        self.proc = launch(
            self.root.path(),
            self.sbfb_home.path(),
            &self.opts,
            self.boots,
        )?;
        self.attach().await
    }

    fn running_json(&self) -> PathBuf {
        self.root.path().join("shell-daemon").join("running.json")
    }

    async fn attach(&mut self) -> Result<()> {
        let state = timeout(HEALTH_TIMEOUT, wait_for_running_json(&self.running_json()))
            .await
            .context("timeout waiting for running.json")?
            .context("failed to read running.json")?;
        self.auth_token = read_auth_token(self.sbfb_home.path()).await?;
        self.http_port = state.api_port;
        self.node_id = state.node_id;
        self.wait_for_health().await
    }

    /// `SBFB_HOME` of this daemon.
    pub fn sbfb_home_path(&self) -> &Path {
        self.sbfb_home.path()
    }

    /// Authenticated loopback call; returns the status and the JSON body
    /// (`Null` when the body is empty or not JSON).
    pub async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<(reqwest::StatusCode, serde_json::Value)> {
        let mut req = reqwest::Client::new()
            .request(method, format!("{}{}", self.http_url(), path))
            .header("X-SBFB-Token", &self.auth_token)
            .header("Host", format!("127.0.0.1:{}", self.http_port))
            .timeout(Duration::from_secs(30));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .with_context(|| format!("request {path}"))?;
        let status = resp.status();
        let bytes = resp.bytes().await.context("read body")?;
        let value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        Ok((status, value))
    }

    pub fn http_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.http_port)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.http_url());
        let resp = reqwest::get(&url).await?;
        if !resp.status().is_success() {
            return Ok(false);
        }
        let body: HealthResponse = resp.json().await?;
        Ok(body.status == "ok")
    }

    async fn wait_for_health(&mut self) -> Result<()> {
        let deadline = tokio::time::Instant::now() + HEALTH_TIMEOUT;
        loop {
            if tokio::time::Instant::now() > deadline {
                bail!("health check timeout for daemon on port {}", self.http_port);
            }
            match self.health_check().await {
                Ok(true) => return Ok(()),
                _ => sleep(HEALTH_POLL_INTERVAL).await,
            }
        }
    }

    pub async fn get_info(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/daemon/info", self.http_url());
        let resp = reqwest::Client::new()
            .get(&url)
            .header("X-SBFB-Token", &self.auth_token)
            .header("Host", format!("127.0.0.1:{}", self.http_port))
            .send()
            .await?;
        let body: serde_json::Value = resp.json().await?;
        Ok(body)
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        #[cfg(unix)]
        {
            if let Some(pid) = self.proc.id() {
                // SAFETY: pid is a valid PID of a daemon process we spawned;
                // SIGINT triggers the graceful ctrl-c shutdown handler.
                unsafe {
                    libc::kill(pid as i32, libc::SIGINT);
                }
            }
        }
        #[cfg(windows)]
        {
            let _ = self.proc.kill().await;
        }

        let _ = timeout(Duration::from_secs(10), self.proc.wait()).await;
        Ok(())
    }

    pub fn root_path(&self) -> &Path {
        self.root.path()
    }

    pub async fn subscribe_curator(&self, pubkey_hex: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/daemon/curators/subscribe", self.http_url());
        let resp = reqwest::Client::new()
            .post(&url)
            .header("X-SBFB-Token", &self.auth_token)
            .header("Host", format!("127.0.0.1:{}", self.http_port))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "curator_pubkey_hex": pubkey_hex }))
            .send()
            .await
            .context("subscribe_curator request")?;
        let body: serde_json::Value = resp.json().await.context("subscribe_curator parse")?;
        Ok(body)
    }

    pub async fn publish_project(&self, name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/daemon/publish", self.http_url());
        let resp = reqwest::Client::new()
            .post(&url)
            .header("X-SBFB-Token", &self.auth_token)
            .header("Host", format!("127.0.0.1:{}", self.http_port))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "project_name": name,
                "category": "test",
                "description": format!("E2E test project {}", name),
            }))
            .send()
            .await
            .context("publish_project request")?;
        let body: serde_json::Value = resp.json().await.context("publish_project parse")?;
        Ok(body)
    }

    pub async fn browse_projects(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/api/daemon/browse", self.http_url());
        let resp = reqwest::Client::new()
            .get(&url)
            .header("X-SBFB-Token", &self.auth_token)
            .header("Host", format!("127.0.0.1:{}", self.http_port))
            .send()
            .await
            .context("browse_projects request")?;
        let body: serde_json::Value = resp.json().await.context("browse_projects parse")?;
        let entries = body["entries"].as_array().cloned().unwrap_or_default();
        Ok(entries)
    }
}

/// Start the daemon process. stdout/stderr go to a file under the root (a
/// pipe nobody drains would block the daemon once its buffer fills).
fn launch(root: &Path, sbfb_home: &Path, opts: &SpawnOptions, boot: u32) -> Result<Child> {
    let bin = daemon_binary();
    if !bin.exists() {
        bail!(
            "daemon binary not found at {}; build with `cargo build -p nexus-shell-daemon`",
            bin.display()
        );
    }
    let log = std::fs::File::create(root.join(format!("daemon-{boot}.log")))
        .context("create daemon log")?;
    let log2 = log.try_clone().context("clone daemon log")?;
    let mut cmd = Command::new(&bin);
    cmd.args(["start", "--headless"])
        .env("NEXUS_GRID_ROOT", root)
        .env("SBFB_HOME", sbfb_home)
        .env(
            "RUST_LOG",
            std::env::var(DAEMON_LOG_ENV).unwrap_or_else(|_| "warn".to_owned()),
        )
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log2))
        .kill_on_drop(true);
    if opts.hermetic {
        cmd.envs(HERMETIC_ENV.iter().copied());
    }
    cmd.spawn()
        .with_context(|| format!("spawn {}", bin.display()))
}

pub struct DaemonCluster {
    pub nodes: Vec<DaemonHandle>,
}

impl DaemonCluster {
    pub async fn spawn(n: usize) -> Result<Self> {
        let mut nodes = Vec::with_capacity(n);
        for i in 0..n {
            tracing::info!(index = i, "spawning daemon {}/{}", i + 1, n);
            let handle = DaemonHandle::spawn()
                .await
                .with_context(|| format!("failed to spawn daemon {}", i))?;
            tracing::info!(
                index = i,
                port = handle.http_port,
                node_id = %handle.node_id,
                "daemon ready"
            );
            nodes.push(handle);
        }
        Ok(Self { nodes })
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            tracing::info!(index = i, "shutting down daemon");
            node.shutdown().await?;
        }
        Ok(())
    }
}

async fn wait_for_running_json(path: &Path) -> Result<RunningState> {
    loop {
        if path.exists() {
            let contents = tokio::fs::read_to_string(path).await?;
            match serde_json::from_str::<RunningState>(&contents) {
                Ok(state) if !state.node_id.is_empty() => return Ok(state),
                _ => {}
            }
        }
        sleep(HEALTH_POLL_INTERVAL).await;
    }
}

async fn read_auth_token(sbfb_home: &Path) -> Result<String> {
    let token_path = sbfb_home.join("auth_token");
    let deadline = tokio::time::Instant::now() + HEALTH_TIMEOUT;
    loop {
        if token_path.exists() {
            let token = tokio::fs::read_to_string(&token_path).await?;
            let token = token.trim().to_string();
            if !token.is_empty() {
                return Ok(token);
            }
        }
        if tokio::time::Instant::now() > deadline {
            bail!("timeout waiting for auth_token at {}", token_path.display());
        }
        sleep(HEALTH_POLL_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_binary_path_is_constructed() {
        let path = daemon_binary();
        assert!(
            path.to_string_lossy().contains("nexus-shell-daemon"),
            "expected daemon binary in path: {}",
            path.display()
        );
    }
}
