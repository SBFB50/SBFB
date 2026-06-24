// SPDX-License-Identifier: AGPL-3.0-or-later
//! End-to-end tests for the `nexus-worker` binary.
//!
//! These tests spawn the real compiled binary (via
//! `env!("CARGO_BIN_EXE_nexus-worker")`) as a subprocess,
//! feed it an isolated `--config` fixture directory, and
//! assert on the CLI surface. They exercise the full stack:
//!
//!   CLI parser → WorkerPaths → WorkerConfig TOML round-trip
//!     → KeyPair::load_or_generate → Allowlist SQLite → Invite
//!     → Engine boot path
//!
//! The Sprint 3 plan originally called for a Python
//! coordinator + Rust worker subprocess + 10 signed tasks end
//! to end. That test requires the coordinator side AND the task
//! claim/execute flow, neither of which is wired here. This suite
//! instead locks every CLI-visible path the coordinator side
//! relies on: register, join a valid invite, projects CRUD, a
//! bounded `start --headless` sanity run.
//!
//! Every test uses `tempfile::TempDir` for the `--config`
//! path so the suite is reproducible across machines and
//! leaves no artifacts behind.

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nexus_core_rs::KeyPair;
use nexus_worker_core::invite::{INVITE_PREFIX, Invite, InviteScope};
use tempfile::TempDir;

// -----------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------

/// Path to the binary that `cargo test` just built. Cargo
/// exports this environment variable automatically for every
/// integration test in a crate that has a `[[bin]]` target.
fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nexus-worker"))
}

/// Allocate a fresh fixture directory + config file path.
fn fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir created");
    let config_file = dir.path().join("worker.toml");
    (dir, config_file)
}

/// Run `nexus-worker --config <path> <args...>` synchronously
/// and return the full `Output`. Panics on spawn failure.
fn run_cli(config_file: &PathBuf, args: &[&str]) -> Output {
    let bin = binary_path();
    Command::new(&bin)
        .arg("--config")
        .arg(config_file)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {bin:?}: {e}"))
}

fn assert_success(output: &Output, ctx: &str) {
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "{ctx} failed with status {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            output.status
        );
    }
}

fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

// -----------------------------------------------------------------
// Tests
// -----------------------------------------------------------------

#[test]
fn version_flag_prints_version() {
    let (_dir, config_file) = fixture();
    let out = run_cli(&config_file, &["--version"]);
    assert_success(&out, "--version");
    let stdout = stdout_str(&out);
    assert!(
        stdout.contains("nexus-worker") && stdout.contains(env!("CARGO_PKG_VERSION")),
        "expected version string, got: {stdout}"
    );
}

#[test]
fn help_flag_lists_every_subcommand() {
    let (_dir, config_file) = fixture();
    let out = run_cli(&config_file, &["--help"]);
    assert_success(&out, "--help");
    let help = stdout_str(&out);
    for expected in [
        "register", "start", "join", "projects", "browse", "stats", "config",
    ] {
        assert!(
            help.contains(expected),
            "expected help to mention `{expected}`, got:\n{help}"
        );
    }
}

#[test]
fn register_creates_config_and_keypair() {
    let (dir, config_file) = fixture();
    let out = run_cli(&config_file, &["register", "--name", "e2e-worker"]);
    assert_success(&out, "register");

    assert!(
        config_file.exists(),
        "register must create the config file at {}",
        config_file.display()
    );

    let cfg_body = std::fs::read_to_string(&config_file).unwrap();
    assert!(cfg_body.contains("[identity]"));
    assert!(cfg_body.contains("e2e-worker"));

    // The keypair lands next to the config in the fixture's
    // derived data dir (WorkerPaths::resolve uses parent /data
    // when --config is set).
    let data_dir = dir.path().join("data");
    assert!(
        data_dir.join("worker.key").exists(),
        "expected worker.key in {}",
        data_dir.display()
    );

    let stdout = stdout_str(&out);
    assert!(stdout.contains("registered as:"));
    assert!(stdout.contains("public key (hex):"));
}

#[test]
fn register_twice_is_rejected() {
    let (_dir, config_file) = fixture();
    let first = run_cli(&config_file, &["register", "--name", "w1"]);
    assert_success(&first, "first register");

    let second = run_cli(&config_file, &["register", "--name", "w2"]);
    assert!(
        !second.status.success(),
        "second register must fail; stdout={}, stderr={}",
        stdout_str(&second),
        String::from_utf8_lossy(&second.stderr)
    );
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(
        stderr.contains("already registered"),
        "stderr should explain the refusal, got: {stderr}"
    );
}

#[test]
fn projects_list_on_empty_allowlist_reports_empty() {
    let (_dir, config_file) = fixture();
    assert_success(
        &run_cli(&config_file, &["register", "--name", "empty"]),
        "register",
    );

    let out = run_cli(&config_file, &["projects", "list"]);
    assert_success(&out, "projects list");
    let stdout = stdout_str(&out);
    assert!(
        stdout.contains("no projects enrolled yet"),
        "expected empty hint, got: {stdout}"
    );
}

#[test]
fn join_then_projects_list_shows_the_project() {
    let (_dir, config_file) = fixture();
    assert_success(
        &run_cli(&config_file, &["register", "--name", "joiner"]),
        "register",
    );

    // Mint a valid invite signed by a fresh coordinator.
    let coord = KeyPair::generate();
    let far_future_expiry = 2_500_000_000; // well past 2026
    let invite = Invite::mint(
        &coord,
        "proj-e2e-001",
        "End-to-end fixture",
        Some("https://relay.example.org/".to_string()),
        Some("fake-doc-ticket-e2e".to_string()),
        InviteScope::Worker,
        far_future_expiry,
    )
    .expect("well-formed Worker invite");
    let wire = invite.encode();
    assert!(wire.starts_with(INVITE_PREFIX));

    let join_out = run_cli(&config_file, &["join", wire.as_str()]);
    assert_success(&join_out, "join");
    let stdout = stdout_str(&join_out);
    assert!(stdout.contains("joined project:"));
    assert!(stdout.contains("End-to-end fixture"));

    let list_out = run_cli(&config_file, &["projects", "list"]);
    assert_success(&list_out, "projects list");
    let list_stdout = stdout_str(&list_out);
    assert!(
        list_stdout.contains("proj-e2e-001"),
        "expected listed id, got:\n{list_stdout}"
    );
    assert!(list_stdout.contains("End-to-end fixture"));
}

#[test]
fn join_expired_invite_is_refused() {
    let (_dir, config_file) = fixture();
    assert_success(
        &run_cli(&config_file, &["register", "--name", "expiry-check"]),
        "register",
    );

    let coord = KeyPair::generate();
    let past_expiry = 1_000_000_000; // year 2001
    let invite = Invite::mint(
        &coord,
        "proj-expired",
        "Expired fixture",
        None,
        Some("fake-doc-ticket-exp".to_string()),
        InviteScope::Worker,
        past_expiry,
    )
    .expect("well-formed Worker invite");
    let wire = invite.encode();

    let out = run_cli(&config_file, &["join", wire.as_str()]);
    assert!(
        !out.status.success(),
        "expired invite must be rejected; stdout={}, stderr={}",
        stdout_str(&out),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("expired"),
        "stderr should mention expiry, got: {stderr}"
    );
}

#[test]
fn join_bad_prefix_is_refused() {
    let (_dir, config_file) = fixture();
    assert_success(
        &run_cli(&config_file, &["register", "--name", "bad-prefix"]),
        "register",
    );

    let out = run_cli(&config_file, &["join", "garbage-that-is-not-an-invite"]);
    assert!(!out.status.success(), "garbage invite must be rejected");
}

#[test]
fn projects_enable_disable_and_budget_round_trip() {
    let (_dir, config_file) = fixture();
    assert_success(
        &run_cli(&config_file, &["register", "--name", "crud"]),
        "register",
    );

    // Enroll one project.
    let coord = KeyPair::generate();
    let invite = Invite::mint(
        &coord,
        "proj-crud-001",
        "Crud fixture",
        None,
        Some("fake-doc-ticket-crud".to_string()),
        InviteScope::Worker,
        2_500_000_000,
    )
    .expect("well-formed Worker invite");
    assert_success(
        &run_cli(&config_file, &["join", invite.encode().as_str()]),
        "join",
    );

    // Disable.
    assert_success(
        &run_cli(&config_file, &["projects", "disable", "proj-crud-001"]),
        "projects disable",
    );
    let list = stdout_str(&run_cli(&config_file, &["projects", "list"]));
    assert!(
        list.contains("false"),
        "expected disabled row, got:\n{list}"
    );

    // Re-enable.
    assert_success(
        &run_cli(&config_file, &["projects", "enable", "proj-crud-001"]),
        "projects enable",
    );
    let list = stdout_str(&run_cli(&config_file, &["projects", "list"]));
    assert!(list.contains("true"), "expected enabled row, got:\n{list}");

    // Set a budget.
    assert_success(
        &run_cli(
            &config_file,
            &["projects", "budget", "proj-crud-001", "1800000"],
        ),
        "projects budget",
    );
    let list = stdout_str(&run_cli(&config_file, &["projects", "list"]));
    assert!(
        list.contains("1800000"),
        "expected budget column to show 1800000, got:\n{list}"
    );

    // Clear the budget.
    assert_success(
        &run_cli(&config_file, &["projects", "budget", "proj-crud-001", "0"]),
        "projects budget clear",
    );
    let list = stdout_str(&run_cli(&config_file, &["projects", "list"]));
    assert!(
        list.contains("unlimited"),
        "budget == 0 should render as 'unlimited', got:\n{list}"
    );
}

#[test]
fn stats_reports_registered_worker() {
    let (_dir, config_file) = fixture();
    assert_success(
        &run_cli(&config_file, &["register", "--name", "stats-check"]),
        "register",
    );

    let out = run_cli(&config_file, &["stats"]);
    assert_success(&out, "stats");
    let stdout = stdout_str(&out);
    assert!(
        stdout.contains("stats-check"),
        "expected worker name, got:\n{stdout}"
    );
    assert!(
        stdout.contains("subcommand:"),
        "expected stub banner, got:\n{stdout}"
    );
}

/// Boot the engine in headless mode and kill it after a short
/// window. Proves that the start handler wires the full engine
/// up without panicking, even though the loop does no real
/// work without a coordinator. Acts as a regression test for
/// every W1..W11 touch-point.
#[cfg(unix)]
#[test]
fn start_headless_boots_and_shuts_down_on_signal() {
    use std::os::unix::process::CommandExt;
    use std::time::Duration;

    let (_dir, config_file) = fixture();
    assert_success(
        &run_cli(&config_file, &["register", "--name", "start-check"]),
        "register",
    );

    let bin = binary_path();
    let mut child = Command::new(&bin)
        .arg("--config")
        .arg(&config_file)
        .arg("start")
        .arg("--headless")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .expect("failed to spawn start --headless");

    // Let the engine boot and run a few ticks.
    std::thread::sleep(Duration::from_millis(800));

    // Send SIGINT; the binary installs a tokio::signal::ctrl_c
    // task that forwards it to the engine shutdown oneshot.
    let pid = child.id() as i32;
    unsafe {
        libc::kill(-pid, libc::SIGINT);
    }

    let status = child.wait().expect("child process joins");
    assert!(
        status.success(),
        "headless start should exit cleanly after SIGINT; status={status:?}"
    );
}
