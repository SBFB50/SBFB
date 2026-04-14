// SPDX-License-Identifier: AGPL-3.0-or-later
//! Launcher-side auth server. Sprint 16 Phase A (D1).
//!
//! The launcher owns the bearer token the rest of the stack
//! validates against. Responsibilities:
//!
//! 1. At boot, load-or-generate the 256-bit hex token via
//!    [`nexus_shell_daemon_core::auth::load_or_generate_token`]
//!    (single on-disk source of truth at `~/.sbfb/auth_token`).
//! 2. Expose the token to the React shell via a minimal axum
//!    server bound on an ephemeral loopback port. The shell
//!    issues `GET /auth/token` at startup, caches the result,
//!    and injects it into every fetch to daemon / coordinator.
//! 3. Persist the launcher's bound port at
//!    `~/.sbfb/launcher.json` so the shell can discover it
//!    without hard-coding.
//!
//! The server only binds on `127.0.0.1`. Even though the
//! `/auth/token` handler would reveal the token to anyone who
//! can reach the port, the kernel refuses non-loopback
//! connections at the socket level. Everything else in the
//! stack is still bearer-authenticated — the launcher is the
//! single distribution point.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use nexus_shell_daemon_core::auth::{self as core_auth};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// Filename of the on-disk discovery record the React shell
/// reads to locate the running launcher. Matches the Sprint 13
/// launcher design doc naming convention (`launcher.json`).
pub const LAUNCHER_JSON_NAME: &str = "launcher.json";

/// Sprint 16 Phase B (D2): make sure `~/.sbfb/run/` exists before
/// the daemon and coordinator try to bind their UDS sockets in
/// it. Mode `0700` on Unix; on Windows the dir lives in the user
/// profile and inherits the user-only ACL, plus the kernel
/// Named Pipe namespace ignores filesystem layout for pipe
/// names — but we still create the dir for symmetry so a future
/// CLI command can drop a per-process state file there.
///
/// Idempotent: existing dir is kept, mode is re-applied.
pub fn ensure_run_dir() -> Result<PathBuf> {
    let dir = core_auth::sbfb_run_dir()
        .ok_or_else(|| anyhow!("cannot resolve ~/.sbfb/run path for this platform"))?;
    std::fs::create_dir_all(&dir).with_context(|| format!("create_dir_all {}", dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o700);
        std::fs::set_permissions(&dir, perms)
            .with_context(|| format!("set 0700 on {}", dir.display()))?;
    }
    Ok(dir)
}

/// Shape of `~/.sbfb/launcher.json`. Pinned so a future launcher
/// version can extend it without breaking the shell parser.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LauncherInfo {
    pub schema_version: u32,
    pub api_host: String,
    pub api_port: u16,
    pub pid: u32,
}

/// Response body of `GET /auth/token`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TokenResponse {
    pub token: String,
}

/// Return `~/.sbfb/launcher.json` for the current user. Uses the
/// same `SBFB_HOME` env override as the daemon so tests redirect
/// both files at the same tempdir.
pub fn launcher_json_path() -> Option<PathBuf> {
    core_auth::sbfb_home().map(|d| d.join(LAUNCHER_JSON_NAME))
}

/// Write the `launcher.json` record atomically (tempfile + rename)
/// so a partially written file is never visible to the shell.
pub fn write_launcher_json(path: &Path, info: &LauncherInfo) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create_dir_all {}", parent.display()))?;
    }
    let json = serde_json::to_string(info).context("serialize LauncherInfo")?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, json).with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("rename to {}", path.display()))?;
    Ok(())
}

/// Best-effort cleanup on shutdown. Failing to remove the file
/// is logged but non-fatal: the next launcher boot overwrites it.
pub fn remove_launcher_json(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

/// Shared state for the two HTTP routes.
#[derive(Debug, Clone)]
struct AuthServerState {
    token: String,
}

async fn auth_token(State(state): State<AuthServerState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(TokenResponse {
            token: state.token.clone(),
        }),
    )
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

/// Minimal router: `GET /auth/token` + `GET /health`.
/// Unlike the daemon router, the launcher does NOT apply the
/// bearer-auth middleware — the whole point of this surface is
/// to hand out the token. Host + Origin checks are still
/// applied so a malicious page cannot exfiltrate via a
/// cross-origin fetch.
pub fn build_router(token: String) -> Router {
    use axum::{body::Body, extract::Request, middleware::Next, response::Response};

    let state = AuthServerState { token };

    async fn host_origin_gate(req: Request<Body>, next: Next) -> Response {
        let host_ok = req
            .headers()
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .map(core_auth::is_loopback_host)
            .unwrap_or(false);
        if !host_ok {
            return (StatusCode::FORBIDDEN, "host not allowed").into_response();
        }
        if let Some(origin) = req.headers().get(axum::http::header::ORIGIN) {
            let ok = origin
                .to_str()
                .ok()
                .map(core_auth::is_loopback_origin)
                .unwrap_or(false);
            if !ok {
                return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
            }
        }
        next.run(req).await
    }

    Router::new()
        .route("/auth/token", get(auth_token))
        .route("/health", get(health))
        .with_state(state)
        .layer(axum::middleware::from_fn(host_origin_gate))
}

/// A live launcher auth server.
pub struct AuthServer {
    bound: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
    launcher_json: PathBuf,
}

impl AuthServer {
    /// Start the server on `127.0.0.1:0` (ephemeral port),
    /// persist `launcher.json`, and return a handle the caller
    /// can use to query the bound port or drive shutdown.
    pub async fn start(token: String) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("bind launcher HTTP listener on 127.0.0.1:0")?;
        let bound = listener
            .local_addr()
            .context("read local_addr of launcher HTTP listener")?;

        let launcher_json = launcher_json_path()
            .ok_or_else(|| anyhow!("cannot resolve launcher.json path for this platform"))?;
        let info = LauncherInfo {
            schema_version: 1,
            api_host: bound.ip().to_string(),
            api_port: bound.port(),
            pid: std::process::id(),
        };
        write_launcher_json(&launcher_json, &info)
            .with_context(|| format!("write {}", launcher_json.display()))?;

        let router = build_router(token);
        let (tx, rx) = oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            let serve = axum::serve(listener, router).with_graceful_shutdown(async move {
                let _ = rx.await;
            });
            if let Err(e) = serve.await {
                eprintln!("[launcher] auth server exited with error: {e}");
            }
        });

        Ok(Self {
            bound,
            shutdown: Some(tx),
            handle,
            launcher_json,
        })
    }

    pub fn bound(&self) -> SocketAddr {
        self.bound
    }

    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        let _ = (&mut self.handle).await;
        remove_launcher_json(&self.launcher_json);
    }
}

impl Drop for AuthServer {
    fn drop(&mut self) {
        // If the owner forgot to call `shutdown().await`, tear
        // down the HTTP task and remove launcher.json synchronously.
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        remove_launcher_json(&self.launcher_json);
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Every test in this module mutates the `SBFB_HOME` env var.
    /// Serialize them through a shared mutex so cargo's parallel
    /// runner cannot observe a racing value (see the matching
    /// pattern in `nexus-shell-daemon-core::paths::tests`).
    fn env_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }

    struct SbfbHomeGuard {
        prev: Option<String>,
        _guard: std::sync::MutexGuard<'static, ()>,
    }

    impl SbfbHomeGuard {
        fn new(path: &Path) -> Self {
            let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
            let prev = std::env::var("SBFB_HOME").ok();
            std::env::set_var("SBFB_HOME", path);
            Self { prev, _guard }
        }
    }

    impl Drop for SbfbHomeGuard {
        fn drop(&mut self) {
            match self.prev.take() {
                Some(v) => std::env::set_var("SBFB_HOME", v),
                None => std::env::remove_var("SBFB_HOME"),
            }
        }
    }

    #[test]
    fn ensure_run_dir_creates_path_idempotently() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = SbfbHomeGuard::new(dir.path());

        let run = ensure_run_dir().unwrap();
        assert!(run.exists(), "run dir must exist after first call");
        assert_eq!(run, dir.path().join("run"));

        // Second call is a no-op: same path, no error.
        let run2 = ensure_run_dir().unwrap();
        assert_eq!(run, run2);
    }

    #[cfg(unix)]
    #[test]
    fn ensure_run_dir_sets_mode_0700_on_unix() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let _guard = SbfbHomeGuard::new(dir.path());

        let run = ensure_run_dir().unwrap();
        let mode = std::fs::metadata(&run).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "run dir must be 0700 on Unix");
    }

    #[test]
    fn write_then_parse_launcher_json_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = SbfbHomeGuard::new(dir.path());
        let path = launcher_json_path().unwrap();

        let info = LauncherInfo {
            schema_version: 1,
            api_host: "127.0.0.1".to_string(),
            api_port: 54321,
            pid: 1234,
        };
        write_launcher_json(&path, &info).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let parsed: LauncherInfo = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed, info);
    }

    #[test]
    fn launcher_json_rejects_extra_fields() {
        let raw =
            r#"{"schema_version":1,"api_host":"127.0.0.1","api_port":1,"pid":1,"extra":"nope"}"#;
        let err = serde_json::from_str::<LauncherInfo>(raw).unwrap_err();
        assert!(err.to_string().contains("unknown field"), "got: {err}");
    }

    #[tokio::test]
    async fn auth_token_endpoint_returns_token() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = SbfbHomeGuard::new(dir.path());
        let server = AuthServer::start("deadbeef".to_string()).await.unwrap();
        let url = format!("http://{}/auth/token", server.bound());

        let resp = reqwest_get(&url, &[("host", &server.bound().to_string())])
            .await
            .unwrap();
        assert_eq!(resp.0, 200);
        let body: TokenResponse = serde_json::from_str(&resp.1).unwrap();
        assert_eq!(body.token, "deadbeef");

        server.shutdown().await;
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = SbfbHomeGuard::new(dir.path());
        let server = AuthServer::start("deadbeef".to_string()).await.unwrap();
        let url = format!("http://{}/health", server.bound());
        let resp = reqwest_get(&url, &[("host", &server.bound().to_string())])
            .await
            .unwrap();
        assert_eq!(resp.0, 200);
        assert!(resp.1.contains("ok"));
        server.shutdown().await;
    }

    #[tokio::test]
    async fn rebound_host_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = SbfbHomeGuard::new(dir.path());
        let server = AuthServer::start("deadbeef".to_string()).await.unwrap();
        let url = format!("http://{}/auth/token", server.bound());
        let resp = reqwest_get(&url, &[("host", "attacker.com")])
            .await
            .unwrap();
        assert_eq!(resp.0, 403);
        server.shutdown().await;
    }

    #[tokio::test]
    async fn cross_origin_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = SbfbHomeGuard::new(dir.path());
        let server = AuthServer::start("deadbeef".to_string()).await.unwrap();
        let url = format!("http://{}/auth/token", server.bound());
        let resp = reqwest_get(
            &url,
            &[
                ("host", &server.bound().to_string()),
                ("origin", "https://attacker.com"),
            ],
        )
        .await
        .unwrap();
        assert_eq!(resp.0, 403);
        server.shutdown().await;
    }

    #[tokio::test]
    async fn start_persists_launcher_json_with_bound_port() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = SbfbHomeGuard::new(dir.path());
        let server = AuthServer::start("deadbeef".to_string()).await.unwrap();
        let path = launcher_json_path().unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let info: LauncherInfo = serde_json::from_str(&raw).unwrap();
        assert_eq!(info.schema_version, 1);
        assert_eq!(info.api_port, server.bound().port());
        assert_eq!(info.pid, std::process::id());
        server.shutdown().await;
        assert!(!path.exists(), "launcher.json must be removed on shutdown");
    }

    /// Tiny raw HTTP GET helper that writes the request bytes
    /// over a TCP connection so we can control exactly which
    /// headers are sent (unlike `reqwest`, which auto-attaches
    /// a `Host:` header).
    async fn reqwest_get(url: &str, headers: &[(&str, &str)]) -> std::io::Result<(u16, String)> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let parsed = url::Url::parse(url).unwrap();
        let host = parsed.host_str().unwrap();
        let port = parsed.port().unwrap_or(80);
        let path = parsed.path();

        let mut stream = tokio::net::TcpStream::connect((host, port)).await?;
        let mut req = format!("GET {path} HTTP/1.1\r\n");
        let mut saw_host = false;
        for (k, v) in headers {
            if k.eq_ignore_ascii_case("host") {
                saw_host = true;
            }
            req.push_str(&format!("{k}: {v}\r\n"));
        }
        if !saw_host {
            req.push_str(&format!("Host: {host}:{port}\r\n"));
        }
        req.push_str("Connection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).await?;

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;
        let raw = String::from_utf8_lossy(&buf).to_string();
        let mut lines = raw.lines();
        let first = lines.next().unwrap_or("");
        let status: u16 = first
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let body_start = raw.find("\r\n\r\n").map(|i| i + 4).unwrap_or(raw.len());
        let body = raw[body_start..].to_string();
        Ok((status, body))
    }
}
