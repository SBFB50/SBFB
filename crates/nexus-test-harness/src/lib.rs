// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-daemon integration test harness for SBFB.
//!
//! Spawns N isolated `nexus-shell-daemon` processes, each in its
//! own temporary directory with distinct iroh keypairs. Provides
//! health checks, port discovery via `running.json`, and graceful
//! shutdown via SIGINT / TerminateProcess.

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

fn daemon_binary() -> PathBuf {
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
    _sbfb_home: TempDir,
    pub http_port: u16,
    pub node_id: String,
    pub auth_token: String,
}

impl DaemonHandle {
    async fn spawn() -> Result<Self> {
        let root = TempDir::new().context("create NEXUS_GRID_ROOT tempdir")?;
        let sbfb_home = TempDir::new().context("create SBFB_HOME tempdir")?;

        let bin = daemon_binary();
        if !bin.exists() {
            bail!(
                "daemon binary not found at {}; build with `cargo build -p nexus-shell-daemon`",
                bin.display()
            );
        }

        let proc = Command::new(&bin)
            .args(["start", "--headless"])
            .env("NEXUS_GRID_ROOT", root.path())
            .env("SBFB_HOME", sbfb_home.path())
            .env("RUST_LOG", "warn")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("spawn {}", bin.display()))?;

        let running_json = root.path().join("shell-daemon").join("running.json");

        let state = timeout(HEALTH_TIMEOUT, wait_for_running_json(&running_json))
            .await
            .context("timeout waiting for running.json")?
            .context("failed to read running.json")?;

        let auth_token = read_auth_token(sbfb_home.path()).await?;

        let mut handle = Self {
            proc,
            root,
            _sbfb_home: sbfb_home,
            http_port: state.api_port,
            node_id: state.node_id,
            auth_token,
        };

        handle.wait_for_health().await?;
        Ok(handle)
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
