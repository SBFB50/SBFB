//! Tracing subscriber wiring for the `nexus-shell-daemon` binary.
//!
//! Direct clone of `nexus-worker/src/logging.rs` adapted for the
//! daemon: the file appender writes `daemon.log` instead of
//! `worker.log`, and the initial info message identifies the
//! daemon. Everything else — two layers (stdout + rotating
//! JSON file), the verbose `-v` override, the `$RUST_LOG` wins
//! path — is identical.
//!
//! Keeping the file a near-copy is a deliberate choice: the two
//! binaries serve different processes, they evolve on different
//! schedules, and making one depend on the other's subscriber
//! wiring would couple them for reasons unrelated to their
//! purpose. The Sprint 3 worker log config and the Sprint 7
//! daemon log config diverge over time — that's fine.

use std::path::Path;

use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

/// Handle that keeps the background file-writer thread alive.
///
/// The caller must store it for the whole program lifetime
/// (normally as a local binding in `main`). Dropping it blocks
/// until every queued log line has been flushed to disk, which
/// is exactly the shutdown behaviour we want.
#[must_use = "dropping the LogGuard stops the background file writer"]
pub struct LogGuard {
    _file_guard: WorkerGuard,
}

/// Initialize the global tracing subscriber.
///
/// `level` comes from the loaded shell daemon config
/// (`[logging] level`, default `"info"`). `verbose` is the CLI
/// `-v` repetition count: 0 = use level as-is, 1 = force `info`,
/// 2 = `debug`, 3+ = `trace`.
///
/// `log_dir` must exist or be creatable — the function calls
/// `std::fs::create_dir_all` before opening the appender so the
/// first-run bootstrap path on a fresh machine works.
///
/// `$RUST_LOG` wins over both sources when set: operators can
/// always override from the shell without editing the config
/// file.
pub fn init_logging(log_dir: &Path, level: &str, verbose: u8) -> Result<LogGuard> {
    std::fs::create_dir_all(log_dir)
        .with_context(|| format!("failed to create log directory {}", log_dir.display()))?;

    let terminal_filter = terminal_filter(level, verbose)?;
    let file_filter = file_filter()?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "daemon.log");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    let terminal_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(atty_stdout())
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(std::io::stdout)
        .with_filter(terminal_filter);

    let file_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(false)
        .with_target(true)
        .with_writer(file_writer)
        .with_filter(file_filter);

    tracing_subscriber::registry()
        .with(terminal_layer)
        .with(file_layer)
        .try_init()
        .context("failed to install global tracing subscriber")?;

    tracing::info!(
        log_dir = %log_dir.display(),
        "shell daemon logging initialized"
    );

    Ok(LogGuard {
        _file_guard: file_guard,
    })
}

/// Build the terminal `EnvFilter` from the highest-priority
/// source available. Order: `$RUST_LOG` → verbose override →
/// config `[logging] level`.
fn terminal_filter(level: &str, verbose: u8) -> Result<EnvFilter> {
    if let Ok(filter) = EnvFilter::try_from_default_env() {
        return Ok(filter);
    }
    let directive = match verbose {
        0 => level.to_string(),
        1 => "info".to_string(),
        2 => "debug".to_string(),
        _ => "trace".to_string(),
    };
    EnvFilter::try_new(&directive)
        .with_context(|| format!("invalid tracing filter directive: {directive:?}"))
}

/// File filter is fixed at info+, regardless of `-v`, so a
/// verbose interactive session does not blow up the on-disk
/// logs with trace spam.
fn file_filter() -> Result<EnvFilter> {
    EnvFilter::try_new("info").context("static file filter 'info' should always parse")
}

/// Poor-man's TTY detection so CI / piped invocations get
/// plain-text logs without ANSI escape codes.
fn atty_stdout() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}
