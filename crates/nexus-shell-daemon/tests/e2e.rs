//! End-to-end tests for the `nexus-shell-daemon` binary.
//!
//! These tests spawn the real compiled binary (via
//! `env!("CARGO_BIN_EXE_nexus-shell-daemon")`) as a subprocess,
//! point it at an isolated `NEXUS_GRID_ROOT` fixture directory,
//! and assert on its singleton + HTTP behaviour.
//!
//! The fixture strategy is different from `nexus-worker/tests`:
//! the shell daemon honours `NEXUS_GRID_ROOT` as the single
//! override point, so we set that environment variable on the
//! spawned child instead of passing `--config`. This exercises
//! the production path more faithfully (the binary's
//! `ShellDaemonPaths::resolve(None)` branch) and keeps the
//! fixture setup simple.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;

// -----------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nexus-shell-daemon"))
}

/// Drain `child.stdout` on a background thread and push every
/// line into a shared `Vec<String>`. Used so a child that hangs
/// on a full pipe cannot deadlock the test.
fn drain_lines(
    name: &'static str,
    reader: impl std::io::Read + Send + 'static,
) -> thread::JoinHandle<Vec<String>> {
    thread::spawn(move || {
        let mut lines = Vec::new();
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            eprintln!("[{name}] {line}");
            lines.push(line);
        }
        lines
    })
}

/// Poll `running.json` under `<root>/shell-daemon/running.json`
/// until it shows up or the deadline elapses.
fn wait_for_running_json(root: &std::path::Path, timeout: Duration) -> Option<PathBuf> {
    let candidate = root.join("shell-daemon").join("running.json");
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if candidate.exists() {
            return Some(candidate);
        }
        thread::sleep(Duration::from_millis(50));
    }
    None
}

fn read_port_from_running_json(path: &std::path::Path) -> Option<u16> {
    let body = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&body).ok()?;
    value.get("api_port")?.as_u64().map(|v| v as u16)
}

/// Try to GET `http://127.0.0.1:<port>/health` using `std::net`
/// with a hand-rolled HTTP/1.1 request. Pulling a full HTTP
/// client crate into dev-dependencies just to curl a single
/// loopback URL would be overkill — the request is 4 lines and
/// the response is parsed with `lines().take_while`.
fn http_get_health(port: u16) -> Option<(u16, String)> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok()?;
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .ok()?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    let mut split = text.splitn(2, "\r\n\r\n");
    let head = split.next()?;
    let body = split.next().unwrap_or("").to_string();
    let status = head.lines().next()?;
    let code = status.split_whitespace().nth(1)?.parse::<u16>().ok()?;
    Some((code, body))
}

// -----------------------------------------------------------------
// Tests
// -----------------------------------------------------------------

#[test]
fn version_flag_prints_version() {
    let out = Command::new(binary_path())
        .arg("--version")
        .output()
        .expect("spawn --version");
    assert!(out.status.success(), "--version must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("nexus-shell-daemon") && stdout.contains(env!("CARGO_PKG_VERSION")),
        "expected version line, got: {stdout}"
    );
}

#[test]
fn help_flag_lists_every_subcommand() {
    let out = Command::new(binary_path())
        .arg("--help")
        .output()
        .expect("spawn --help");
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    for expected in ["start", "stop", "status", "config"] {
        assert!(
            help.contains(expected),
            "expected help to mention `{expected}`, got:\n{help}"
        );
    }
}

#[test]
fn stop_stub_prints_phase_marker() {
    let tmp = TempDir::new().expect("tempdir");
    let out = Command::new(binary_path())
        .env("NEXUS_GRID_ROOT", tmp.path())
        .arg("stop")
        .output()
        .expect("spawn stop");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("subcommand:   stop"),
        "expected stub banner, got:\n{stdout}"
    );
    assert!(
        stdout.contains("Phase E"),
        "stub banner must point at the Phase that will implement it, got:\n{stdout}"
    );
}

#[test]
fn status_stub_prints_phase_marker() {
    let tmp = TempDir::new().expect("tempdir");
    let out = Command::new(binary_path())
        .env("NEXUS_GRID_ROOT", tmp.path())
        .arg("status")
        .output()
        .expect("spawn status");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("subcommand:   status"));
}

/// Boot the daemon in headless mode, poll `running.json`, curl
/// `/health`, then kill the child. Proves the full start path
/// wires up singleton + iroh + HTTP without panicking. Runs on
/// every platform the workspace targets.
#[test]
fn start_writes_running_json_and_responds_to_health() {
    let tmp = TempDir::new().expect("tempdir");

    let mut child = Command::new(binary_path())
        .env("NEXUS_GRID_ROOT", tmp.path())
        .arg("start")
        .arg("--headless")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn start --headless");

    let stdout_reader = child.stdout.take().expect("piped stdout");
    let stderr_reader = child.stderr.take().expect("piped stderr");
    let _stdout_handle = drain_lines("stdout", stdout_reader);
    let _stderr_handle = drain_lines("stderr", stderr_reader);

    let running_json = wait_for_running_json(tmp.path(), Duration::from_secs(10))
        .expect("running.json should appear within 10s of start");
    let port = read_port_from_running_json(&running_json)
        .expect("running.json must carry an api_port field");
    assert!(port > 0, "bound port must be non-zero");

    // Give axum a tick to install the router — start() returns
    // as soon as the listener is bound and write_running lands,
    // but the serve task may still be mid-`spawn`.
    let (code, body) = loop_until_health(port, Duration::from_secs(5))
        .expect("GET /health must succeed within 5s");
    assert_eq!(code, 200, "expected /health to return 200");
    assert!(
        body.contains("\"status\":\"ok\""),
        "unexpected /health body: {body}"
    );
    assert!(
        body.contains("\"schema_version\":1"),
        "unexpected /health body: {body}"
    );

    // Kill the child — on Windows this is a hard kill which
    // skips the Drop-based cleanup, but the next test recreates
    // its own TempDir so that's fine.
    child.kill().expect("kill child");
    let _ = child.wait();
}

fn loop_until_health(port: u16, timeout: Duration) -> Option<(u16, String)> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(resp) = http_get_health(port) {
            return Some(resp);
        }
        thread::sleep(Duration::from_millis(50));
    }
    None
}

/// Spawn two daemons in sequence against the same
/// `NEXUS_GRID_ROOT` fixture. The second must refuse to boot,
/// citing the singleton conflict.
#[test]
fn second_start_refuses_when_first_still_running() {
    let tmp = TempDir::new().expect("tempdir");

    let mut first = Command::new(binary_path())
        .env("NEXUS_GRID_ROOT", tmp.path())
        .arg("start")
        .arg("--headless")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn first daemon");

    let first_out = first.stdout.take().unwrap();
    let first_err = first.stderr.take().unwrap();
    let _first_out_handle = drain_lines("first-stdout", first_out);
    let _first_err_handle = drain_lines("first-stderr", first_err);

    // Wait for the first daemon to be fully up before we race
    // the second one against it.
    let _running_json = wait_for_running_json(tmp.path(), Duration::from_secs(10))
        .expect("first daemon's running.json should appear");

    let second_out = Command::new(binary_path())
        .env("NEXUS_GRID_ROOT", tmp.path())
        .arg("start")
        .arg("--headless")
        .output()
        .expect("spawn second daemon");

    assert!(
        !second_out.status.success(),
        "second start must fail; stdout={}, stderr={}",
        String::from_utf8_lossy(&second_out.stdout),
        String::from_utf8_lossy(&second_out.stderr)
    );
    let stderr = String::from_utf8_lossy(&second_out.stderr);
    let stdout = String::from_utf8_lossy(&second_out.stdout);
    let combined = format!("{stdout}\n{stderr}");
    assert!(
        combined.contains("already running"),
        "second start should explain the singleton conflict, got:\n{combined}"
    );

    first.kill().expect("kill first daemon");
    let _ = first.wait();
}

/// Unix-only: send SIGINT to a running daemon and verify it
/// exits cleanly with `running.json` removed. The Windows
/// equivalent is harder (no signal-based shutdown path in
/// `std::process::Command`) so we skip it and cover the same
/// graceful path via the inner-crate `DaemonRuntime::shutdown`
/// unit test in `runtime.rs`.
#[cfg(unix)]
#[test]
fn sigint_triggers_graceful_shutdown_and_removes_running_json() {
    use std::os::unix::process::CommandExt;

    let tmp = TempDir::new().expect("tempdir");

    let mut child = Command::new(binary_path())
        .env("NEXUS_GRID_ROOT", tmp.path())
        .arg("start")
        .arg("--headless")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .expect("spawn start --headless");

    let stdout_reader = child.stdout.take().unwrap();
    let stderr_reader = child.stderr.take().unwrap();
    let _stdout_handle = drain_lines("stdout", stdout_reader);
    let _stderr_handle = drain_lines("stderr", stderr_reader);

    let running_json = wait_for_running_json(tmp.path(), Duration::from_secs(10))
        .expect("running.json should appear within 10s");

    let pid = child.id() as i32;
    unsafe {
        libc::kill(-pid, libc::SIGINT);
    }

    let status = child.wait().expect("child process joins");
    assert!(
        status.success(),
        "daemon should exit cleanly on SIGINT; status={status:?}"
    );
    assert!(
        !running_json.exists(),
        "running.json must be removed after a clean shutdown"
    );
}
