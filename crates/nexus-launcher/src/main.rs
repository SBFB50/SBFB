// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Minimal SBFB launcher — spawns the shell daemon, opens the
//! browser, waits for Ctrl+C, then shuts down gracefully.
//!
//! Sprint 13 Phase D (D4). No Tauri, no native window — the
//! browser IS the client.
//!
//! Launcher and daemon share `<nexus-grid-root>/logs/` (S37 D1).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use nexus_shell_daemon_core::auth::{TokenRotator, tokens_file_path};
use serde::Deserialize;
use tokio::sync::RwLock;

mod auth;
mod driver_check;
mod token_rotation;
mod tray;
mod unlock;

#[cfg(windows)]
fn error_msgbox(title: &str, msg: &str) {
    unsafe extern "system" {
        fn MessageBoxW(
            hwnd: *mut core::ffi::c_void,
            text: *const u16,
            caption: *const u16,
            utype: u32,
        ) -> i32;
    }
    const MB_ICONERROR: u32 = 0x10;
    const MB_OK: u32 = 0x0;
    let text: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
    let caption: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: text and caption are valid null-terminated UTF-16 slices,
    // hwnd is null (no parent window).
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_ICONERROR | MB_OK,
        );
    }
}

#[cfg(not(windows))]
fn error_msgbox(_title: &str, msg: &str) {
    eprintln!("{msg}");
}

#[cfg(test)]
mod test_util {
    //! Shared test plumbing for the launcher. Right now: a
    //! single process-wide `SBFB_HOME` mutex so `auth::tests`
    //! and `token_rotation::tests` do not race the env var
    //! with each other when cargo runs both modules in
    //! parallel threads.
    pub fn env_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }
}

/// Production token rotation cadence. Overridable via
/// `SBFB_TOKEN_ROTATION_INTERVAL_SECS` for manual burn-in tests
/// on a running launcher; absent → 24 h.
const DEFAULT_ROTATION_INTERVAL_SECS: u64 = 86_400;

// =================================================================
// Structured logging (tracing-appender, shared log dir with daemon)
// =================================================================

fn launcher_log_dir() -> PathBuf {
    nexus_shell_daemon_core::paths::log_dir().unwrap_or_else(|| {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".nexus-grid").join("logs")
    })
}

#[must_use = "dropping the guard stops the background file writer"]
struct LauncherLogGuard {
    _file_guard: tracing_appender::non_blocking::WorkerGuard,
}

fn setup_tracing() -> Option<LauncherLogGuard> {
    use tracing_subscriber::Layer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let log_dir = launcher_log_dir();
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!(
            "[launcher] failed to create log dir {}: {e}",
            log_dir.display()
        );
        return None;
    }

    let file_appender = tracing_appender::rolling::daily(&log_dir, "launcher.log");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_target(true)
        .with_writer(file_writer)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    #[cfg(debug_assertions)]
    {
        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_writer(std::io::stdout)
            .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
        tracing_subscriber::registry()
            .with(stdout_layer)
            .with(file_layer)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry().with(file_layer).init();
    }

    let panic_dir = log_dir;
    std::panic::set_hook(Box::new(move |info| {
        let _ = std::fs::write(
            panic_dir.join("launcher-panic.log"),
            format!("[launcher] PANIC: {info}\n"),
        );
    }));

    Some(LauncherLogGuard {
        _file_guard: file_guard,
    })
}

// =================================================================
// running.json schema
// =================================================================

/// Subset of the daemon's `running.json` that the launcher needs.
#[derive(Debug, Deserialize)]
pub struct RunningInfo {
    pub api_host: String,
    pub api_port: u16,
    pub pid: u32,
}

/// Read and parse `running.json` from the given path.
/// Returns `None` if the file doesn't exist or can't be parsed.
pub fn read_running_info(path: &std::path::Path) -> Option<RunningInfo> {
    let body = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

/// Return the expected path of `running.json` — delegates to
/// `nexus_shell_daemon_core::paths::running_json_path()` so
/// launcher and daemon always agree on the location (platform
/// `data_dir` via `dirs`, overridable via `NEXUS_GRID_ROOT`).
pub fn find_running_json() -> PathBuf {
    nexus_shell_daemon_core::paths::running_json_path().unwrap_or_else(|| {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".nexus-grid")
            .join("shell-daemon")
            .join("running.json")
    })
}

/// Poll `running.json` until it appears and parses correctly,
/// or until the timeout elapses.
async fn wait_for_running(path: &std::path::Path, timeout: Duration) -> Option<RunningInfo> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if let Some(info) = read_running_info(path) {
            return Some(info);
        }
        if tokio::time::Instant::now() >= deadline {
            return None;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// Check if the daemon is actually alive by attempting a TCP
/// connection to its API port (2s timeout).  Avoids the stale
/// `running.json` scenario where the daemon crashed but left the
/// file behind.
async fn is_daemon_alive(info: &RunningInfo) -> bool {
    let addr = format!("{}:{}", info.api_host, info.api_port);
    tokio::time::timeout(
        Duration::from_secs(2),
        tokio::net::TcpStream::connect(&addr),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false)
}

/// Spawn the daemon, wait for `running.json`, return the info.
/// Exits the process on failure.
async fn spawn_and_wait(
    running_path: &std::path::Path,
    spawned_child: &mut Option<std::process::Child>,
) -> RunningInfo {
    tracing::info!("spawning nexus-shell-daemon start");
    match spawn_daemon() {
        Ok(child) => {
            *spawned_child = Some(child);
        }
        Err(e) => {
            let msg = format!(
                "Failed to spawn nexus-shell-daemon: {e}\n\n\
                 Make sure nexus-shell-daemon is in PATH or next to the launcher."
            );
            tracing::error!("{msg}");
            error_msgbox("SBFB Launcher", &msg);
            std::process::exit(1);
        }
    }

    tracing::info!("waiting for daemon to start (max 15s)");
    match wait_for_running(running_path, Duration::from_secs(15)).await {
        Some(info) => {
            tracing::info!(
                host = %info.api_host,
                port = info.api_port,
                "daemon ready"
            );
            info
        }
        None => {
            let msg = "Daemon did not start within 15 seconds.\n\n\
                       Check logs in ~/.nexus-grid/logs/ for details.";
            tracing::error!("{msg}");
            error_msgbox("SBFB Launcher", msg);
            if let Some(child) = spawned_child {
                let _ = child.kill();
            }
            std::process::exit(1);
        }
    }
}

/// Spawn `nexus-shell-daemon start` as a child process.
fn spawn_daemon() -> std::io::Result<std::process::Child> {
    use std::process::Command;

    // Look for the daemon binary next to the launcher first,
    // then fall back to PATH.
    let exe = std::env::current_exe().ok();
    let sibling = exe
        .as_ref()
        .and_then(|p| p.parent())
        .map(|dir| dir.join("nexus-shell-daemon"));

    let program = match &sibling {
        Some(p) if p.exists() => p.as_os_str().to_owned(),
        _ => "nexus-shell-daemon".into(),
    };

    let mut cmd = Command::new(program);
    cmd.arg("start");

    if let Some(web_root) = resolve_web_root() {
        cmd.arg("--web-root").arg(&web_root);
        tracing::info!(path = %web_root.display(), "passing --web-root to daemon");
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn()
}

/// Locate the built React shell (`web/dist/`). Search order:
/// 1. `SBFB_WEB_ROOT` env var
/// 2. `<launcher_dir>/web-dist/` (bundled deployment)
/// 3. `<launcher_dir>/../../web/dist/` (dev: target/{debug,release}/
///    → project root → web/dist)
fn resolve_web_root() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("SBFB_WEB_ROOT") {
        let pb = std::path::PathBuf::from(p);
        if pb.join("index.html").exists() {
            return Some(pb);
        }
    }
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let installed = dir.join("web");
    if installed.join("index.html").exists() {
        return Some(installed);
    }
    let bundled = dir.join("web-dist");
    if bundled.join("index.html").exists() {
        return Some(bundled);
    }
    let dev = dir.join("../../web/dist");
    if dev.join("index.html").exists() {
        return Some(dev.canonicalize().unwrap_or(dev));
    }
    None
}

// =================================================================
// Main
// =================================================================

fn main() {
    let _log_guard = setup_tracing();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("nexus-launcher — SBFB launcher");
        println!();
        println!("Spawns nexus-shell-daemon, opens the default browser,");
        println!("shows a tray icon for control.");
        println!();
        println!("Usage:");
        println!(
            "  nexus-launcher                       # spawn daemon (legacy ephemeral keypair)"
        );
        println!("  nexus-launcher init   --pin <pin>    # generate + encrypt identity, exit");
        println!("  nexus-launcher unlock --pin <pin>    # decrypt identity, spawn daemon with it");
        return;
    }

    match unlock::parse_subcommand(&args) {
        Some(unlock::Subcommand::Init { pin }) => {
            std::process::exit(unlock::run_init(&pin));
        }
        Some(unlock::Subcommand::InitDuress { pin }) => {
            std::process::exit(unlock::run_init_duress(&pin));
        }
        Some(unlock::Subcommand::Unlock { pin }) => {
            if let Err(code) = unlock::run_unlock_and_export_env(&pin) {
                std::process::exit(code);
            }
        }
        None => {}
    }

    // Pre-thread: resolve token + set env before any worker threads.
    let token = match resolve_token_for_child() {
        Ok(t) => t,
        Err(e) => {
            let msg = format!("Failed to prepare auth token: {e}");
            tracing::error!("{msg}");
            error_msgbox("SBFB Launcher", &msg);
            std::process::exit(1);
        }
    };
    // SAFETY: called before tokio runtime creation, single-threaded.
    unsafe { std::env::set_var(nexus_shell_daemon_core::auth::AUTH_TOKEN_ENV, &token) };

    if let Err(e) = auth::ensure_run_dir() {
        let msg = format!("Failed to prepare run directory: {e}");
        tracing::error!("{msg}");
        error_msgbox("SBFB Launcher", &msg);
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    // Ctrl+C forwarding: spawned on the runtime, signals the main
    // thread's tray event loop via a std channel.
    let (ctrl_c_tx, ctrl_c_rx) = std::sync::mpsc::channel::<()>();
    rt.spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = ctrl_c_tx.send(());
    });

    // --- Async init phase ---
    let (url, spawned_child, auth_server, rotation_handle) = rt.block_on(async {
        tokio::spawn(async move {
            let report = driver_check::check_nvidia_drivers().await;
            let source = if report.fetched_from_cache {
                "cache"
            } else if report.fetch_failed {
                "fetch-failed"
            } else {
                "nvd"
            };
            match report.local_version.as_deref() {
                Some(v) => tracing::info!(
                    source,
                    local = v,
                    cves = report.cves_affecting.len(),
                    critical = report.critical_count,
                    "driver check complete"
                ),
                None => {
                    tracing::info!(source, "no NVIDIA driver detected, skipping")
                }
            }
            if report.critical_count > 0
                && let Some(ref v) = report.local_version
            {
                tracing::warn!(
                    driver = %v,
                    critical = report.critical_count,
                    "NVIDIA driver affected by Critical CVE — consider updating"
                );
            }
        });

        let auth_server = match auth::AuthServer::start(token.clone()).await {
            Ok(s) => {
                tracing::info!(addr = %s.bound(), "auth server listening");
                Some(s)
            }
            Err(e) => {
                let msg = format!("Failed to start auth server: {e}");
                tracing::error!("{msg}");
                error_msgbox("SBFB Launcher", &msg);
                std::process::exit(1);
            }
        };

        let rotation_handle = match tokens_file_path() {
            Some(path) => {
                let rotator = match TokenRotator::load(&path) {
                    Ok(Some(r)) => r,
                    Ok(None) => {
                        let r = TokenRotator::new(token.clone());
                        if let Err(e) = r.write_atomic(&path) {
                            tracing::warn!(
                                path = %path.display(),
                                "failed to seed tokens.json: {e}"
                            );
                        }
                        r
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = %path.display(),
                            "tokens.json malformed ({e}); reseeding"
                        );
                        let r = TokenRotator::new(token.clone());
                        if let Err(e) = r.write_atomic(&path) {
                            tracing::warn!("failed to rewrite tokens.json: {e}");
                        }
                        r
                    }
                };
                let rotator = Arc::new(RwLock::new(rotator));
                let interval_secs = std::env::var("SBFB_TOKEN_ROTATION_INTERVAL_SECS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_ROTATION_INTERVAL_SECS);
                tracing::info!(
                    interval_secs,
                    path = %path.display(),
                    "token rotation loop started"
                );
                Some(token_rotation::spawn_rotation_loop(
                    rotator,
                    path,
                    Duration::from_secs(interval_secs),
                ))
            }
            None => {
                tracing::warn!("could not resolve tokens.json path; rotation disabled");
                None
            }
        };

        let running_path = find_running_json();
        tracing::info!(path = %running_path.display(), "looking for daemon");

        let mut spawned_child: Option<std::process::Child> = None;
        let info = if let Some(info) = read_running_info(&running_path) {
            if is_daemon_alive(&info).await {
                tracing::info!(
                    host = %info.api_host,
                    port = info.api_port,
                    "daemon already running"
                );
                info
            } else {
                tracing::warn!(
                    pid = info.pid,
                    host = %info.api_host,
                    port = info.api_port,
                    "stale running.json, removing"
                );
                let _ = std::fs::remove_file(&running_path);
                spawn_and_wait(&running_path, &mut spawned_child).await
            }
        } else {
            spawn_and_wait(&running_path, &mut spawned_child).await
        };

        let url = format!("http://{}:{}", info.api_host, info.api_port);
        tracing::info!(url = %url, "opening browser");
        if let Err(e) = open::that(&url) {
            tracing::warn!("failed to open browser: {e}");
        }

        (url, spawned_child, auth_server, rotation_handle)
    });

    // --- Tray event loop (main thread) ---
    match tray::create_tray() {
        Ok(state) => {
            tracing::info!("tray icon active, right-click for menu");
            tray::run_event_loop(&state, &url, &ctrl_c_rx);
        }
        Err(e) => {
            tracing::warn!("tray icon init failed ({e}), falling back to Ctrl+C");
            tracing::info!("press Ctrl+C to stop");
            let _ = ctrl_c_rx.recv();
        }
    }

    // --- Shutdown ---
    tracing::info!("shutting down");

    rt.block_on(async {
        if let Some(mut child) = spawned_child {
            #[cfg(unix)]
            {
                // SAFETY: child.id() is a valid PID of a process we spawned.
                unsafe {
                    libc::kill(child.id() as i32, libc::SIGTERM);
                }
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill();
            }

            let wait_result = tokio::time::timeout(
                Duration::from_secs(5),
                tokio::task::spawn_blocking(move || child.wait()),
            )
            .await;

            match wait_result {
                Ok(Ok(Ok(status))) => tracing::info!(%status, "daemon exited"),
                Ok(Ok(Err(e))) => tracing::error!("daemon wait error: {e}"),
                Ok(Err(e)) => tracing::error!("daemon join error: {e}"),
                Err(_) => tracing::warn!("daemon did not exit within 5s, abandoning"),
            }
        }

        if let Some(server) = auth_server {
            server.shutdown().await;
        }

        if let Some(handle) = rotation_handle {
            handle.abort();
        }
    });

    tracing::info!("goodbye");
}

/// Resolve the loopback bearer token: prefer an existing
/// `~/.sbfb/auth_token`, otherwise generate a fresh 256-bit
/// token and persist it at mode `0600` (Unix) / user-owned
/// ACL (Windows).
fn resolve_token_for_child() -> anyhow::Result<String> {
    use anyhow::{Context, anyhow};
    let path = nexus_shell_daemon_core::auth::auth_token_path()
        .ok_or_else(|| anyhow!("cannot resolve ~/.sbfb/auth_token path for this platform"))?;
    nexus_shell_daemon_core::auth::load_or_generate_token(&path)
        .with_context(|| format!("load or generate auth token at {}", path.display()))
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_running_info_parses_valid_json() {
        let dir = std::env::temp_dir().join("nexus-launcher-test-valid");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("running.json");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"{{"schema_version":1,"node_id":"aa","api_host":"127.0.0.1","api_port":7777,"pid":1234,"started_at":"2026-01-01T00:00:00Z","daemon_version":"1.0.0"}}"#
        )
        .unwrap();

        let info = read_running_info(&path).expect("should parse");
        assert_eq!(info.api_host, "127.0.0.1");
        assert_eq!(info.api_port, 7777);
        assert_eq!(info.pid, 1234);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_read_running_info_returns_none_for_missing_file() {
        let path = std::env::temp_dir().join("nexus-launcher-test-missing/running.json");
        assert!(read_running_info(&path).is_none());
    }

    #[test]
    fn test_find_running_json_returns_expected_path() {
        let path = find_running_json();
        assert!(
            path.ends_with("shell-daemon/running.json")
                || path.ends_with("shell-daemon\\running.json"),
            "unexpected path: {path:?}"
        );
    }

    #[tokio::test]
    async fn test_is_daemon_alive_returns_false_for_dead_port() {
        let info = RunningInfo {
            api_host: "127.0.0.1".to_string(),
            api_port: 19999, // nobody listens here
            pid: 99999,
        };
        assert!(!is_daemon_alive(&info).await);
    }

    #[test]
    fn resolve_web_root_respects_env_var() {
        let _guard = crate::test_util::env_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("index.html"), "<!doctype html>").unwrap();

        unsafe { std::env::set_var("SBFB_WEB_ROOT", tmp.path()) };
        let result = resolve_web_root();
        unsafe { std::env::remove_var("SBFB_WEB_ROOT") };

        assert_eq!(
            result.expect("should find web root via env var"),
            tmp.path()
        );
    }

    #[test]
    fn launcher_log_dir_matches_daemon_log_dir() {
        let _guard = crate::test_util::env_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tempdir");
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe {
            std::env::set_var(
                nexus_shell_daemon_core::paths::NEXUS_GRID_ROOT_ENV,
                tmp.path(),
            )
        };

        let launcher = launcher_log_dir();
        let daemon = nexus_shell_daemon_core::paths::log_dir().expect("log_dir");
        assert_eq!(
            launcher, daemon,
            "launcher and daemon must share the same log directory"
        );

        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(nexus_shell_daemon_core::paths::NEXUS_GRID_ROOT_ENV) };
    }
}
