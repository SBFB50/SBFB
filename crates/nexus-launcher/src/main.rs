// SPDX-License-Identifier: AGPL-3.0-or-later
//! Minimal SBFB launcher — spawns the shell daemon, opens the
//! browser, waits for Ctrl+C, then shuts down gracefully.
//!
//! Sprint 13 Phase D (D4). No Tauri, no native window — the
//! browser IS the client.

use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

mod auth;

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

/// Return the expected path of `running.json` under the SBFB
/// grid root. Checks `NEXUS_GRID_ROOT` env var first, then
/// falls back to `~/.nexus-grid`.
pub fn find_running_json() -> PathBuf {
    let root = match std::env::var("NEXUS_GRID_ROOT") {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => {
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".nexus-grid")
        }
    };
    root.join("shell-daemon").join("running.json")
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
    println!("[launcher] spawning nexus-shell-daemon start...");
    match spawn_daemon() {
        Ok(child) => {
            *spawned_child = Some(child);
        }
        Err(e) => {
            eprintln!("[launcher] failed to spawn daemon: {e}");
            eprintln!("[launcher] make sure nexus-shell-daemon is in PATH or next to the launcher");
            std::process::exit(1);
        }
    }

    println!("[launcher] waiting for daemon to start (max 15s)...");
    match wait_for_running(running_path, Duration::from_secs(15)).await {
        Some(info) => {
            println!(
                "[launcher] daemon ready on {}:{}",
                info.api_host, info.api_port
            );
            info
        }
        None => {
            eprintln!("[launcher] daemon did not produce running.json within 15s");
            if let Some(ref mut child) = spawned_child {
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

    Command::new(program).arg("start").spawn()
}

// =================================================================
// Main
// =================================================================

#[tokio::main]
async fn main() {
    // Check --help / --version.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("nexus-launcher — Minimal SBFB launcher");
        println!();
        println!("Spawns nexus-shell-daemon, opens the default browser,");
        println!("waits for Ctrl+C, then shuts down gracefully.");
        println!();
        println!("Usage: nexus-launcher [--help]");
        return;
    }

    let running_path = find_running_json();
    println!(
        "[launcher] looking for daemon at {}",
        running_path.display()
    );

    // 0. Sprint 16 Phase A (D1): resolve the loopback bearer
    //    token before anything else. Generates + persists
    //    ~/.sbfb/auth_token on first boot, reuses the existing
    //    file on subsequent runs. The daemon child will pick up
    //    the same token either via the SBFB_AUTH_TOKEN env (set
    //    below) or by reading the same file.
    let token = match resolve_token_for_child() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[launcher] failed to prepare auth token: {e}");
            std::process::exit(1);
        }
    };
    std::env::set_var(nexus_shell_daemon_core::auth::AUTH_TOKEN_ENV, &token);

    // 0b. Sprint 16 Phase B (D2): create ~/.sbfb/run/ at mode 0700
    //     so the daemon can drop daemon.sock there and the
    //     coordinator can drop coordinator.sock. On Windows the
    //     dir lives in the user profile and inherits the user
    //     ACL; the kernel Named Pipe namespace ignores filesystem
    //     paths, but we still create the dir for symmetry.
    if let Err(e) = auth::ensure_run_dir() {
        eprintln!("[launcher] failed to prepare ~/.sbfb/run/: {e}");
        std::process::exit(1);
    }

    let auth_server = match auth::AuthServer::start(token.clone()).await {
        Ok(s) => {
            println!("[launcher] auth server listening on {}", s.bound());
            Some(s)
        }
        Err(e) => {
            eprintln!("[launcher] failed to start auth server: {e}");
            std::process::exit(1);
        }
    };

    // 1. Check if daemon already running (with stale detection).
    let mut spawned_child: Option<std::process::Child> = None;
    let info = if let Some(info) = read_running_info(&running_path) {
        // Verify the daemon is actually alive by probing its TCP port.
        if is_daemon_alive(&info).await {
            println!(
                "[launcher] daemon already running on {}:{}",
                info.api_host, info.api_port
            );
            info
        } else {
            eprintln!(
                "[launcher] stale running.json (pid {} not responding on {}:{}), removing",
                info.pid, info.api_host, info.api_port
            );
            let _ = std::fs::remove_file(&running_path);
            // Fall through to spawn a new daemon below.
            spawn_and_wait(&running_path, &mut spawned_child).await
        }
    } else {
        // 2. No running.json — spawn fresh.
        spawn_and_wait(&running_path, &mut spawned_child).await
    };

    // 4. Open the browser.
    let url = format!("http://{}:{}", info.api_host, info.api_port);
    println!("[launcher] opening {url}");
    if let Err(e) = open::that(&url) {
        eprintln!("[launcher] failed to open browser: {e}");
        // Non-fatal — the daemon is running, the user can open manually.
    }

    // 5. Wait for Ctrl+C.
    println!("[launcher] press Ctrl+C to stop");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for Ctrl+C");

    println!("\n[launcher] shutting down...");

    // 6. If we spawned the daemon, kill it.
    if let Some(ref mut child) = spawned_child {
        #[cfg(unix)]
        {
            // Send SIGTERM for graceful shutdown.
            unsafe {
                libc::kill(child.id() as i32, libc::SIGTERM);
            }
        }
        #[cfg(not(unix))]
        {
            let _ = child.kill();
        }

        // Wait for the child to exit (max 5s).
        let wait_result = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::task::spawn_blocking({
                let mut child = spawned_child.take().unwrap();
                move || child.wait()
            }),
        )
        .await;

        match wait_result {
            Ok(Ok(Ok(status))) => println!("[launcher] daemon exited with {status}"),
            Ok(Ok(Err(e))) => eprintln!("[launcher] daemon wait error: {e}"),
            Ok(Err(e)) => eprintln!("[launcher] daemon join error: {e}"),
            Err(_) => eprintln!("[launcher] daemon did not exit within 5s, abandoning"),
        }
    }

    // 7. Shut down the launcher auth server + remove launcher.json.
    if let Some(server) = auth_server {
        server.shutdown().await;
    }

    println!("[launcher] goodbye");
}

/// Resolve the loopback bearer token: prefer an existing
/// `~/.sbfb/auth_token`, otherwise generate a fresh 256-bit
/// token and persist it at mode `0600` (Unix) / user-owned
/// ACL (Windows).
fn resolve_token_for_child() -> anyhow::Result<String> {
    use anyhow::{anyhow, Context};
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
}
