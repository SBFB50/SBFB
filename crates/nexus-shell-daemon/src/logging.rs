// SPDX-License-Identifier: AGPL-3.0-or-later
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
    Layer,
    filter::EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
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

/// The iroh targets whose WARNs carry socket addresses, at `error`, so
/// none reaches a log someone may later share (chat plan, Phase I):
/// peers' paths (`iroh::socket::remote_map`: failed path pings, path
/// status), our own source IP on a dropped transmit
/// (`iroh::socket::transports`), our public IP as the relays see it
/// (`iroh::net_report`, QAD mismatch). The relay client keeps its WARNs
/// (relay URLs only, the operator's main diagnostic). `iroh::socket`
/// stays at `warn`: it opens an INFO span per packet send, which floods
/// stdout / journald under `FmtSpan::CLOSE`; so does `iroh_relay::client`
/// once per relay dial, naming the relay's IP (`connect{addr=…}`).
/// Targets match by string PREFIX and the most specific directive wins;
/// a bare `iroh` would also cover iroh_docs, iroh_gossip, iroh_blobs.
const IROH_ADDRESS_LOGS_OFF: &str = "iroh::socket=warn,\
iroh::socket::remote_map=error,\
iroh::socket::transports=error,\
iroh::socket::transports::relay=warn,\
iroh::net_report=error,\
iroh_relay::client=warn";

/// Build the terminal `EnvFilter` from the highest-priority
/// source available. Order: `$RUST_LOG` → verbose override →
/// config `[logging] level`.
fn terminal_filter(level: &str, verbose: u8) -> Result<EnvFilter> {
    if let Ok(env) = std::env::var(EnvFilter::DEFAULT_ENV)
        && !env.trim().is_empty()
    {
        let directive = env_directive(&env);
        return EnvFilter::try_new(&directive)
            .with_context(|| format!("invalid {}: {directive:?}", EnvFilter::DEFAULT_ENV));
    }
    let directive = terminal_directive(level, verbose);
    EnvFilter::try_new(&directive)
        .with_context(|| format!("invalid tracing filter directive: {directive:?}"))
}

/// `$RUST_LOG` keeps the address directives (a unit that sets
/// `RUST_LOG=info` must not bring the addresses back) unless it names an
/// `iroh` target itself: then the operator is debugging iroh on purpose.
fn env_directive(env: &str) -> String {
    let names_iroh = env
        .split(',')
        .any(|d| d.trim().starts_with("iroh=") || d.trim().starts_with("iroh::"));
    if names_iroh {
        env.to_string()
    } else {
        format!("{env},{IROH_ADDRESS_LOGS_OFF}")
    }
}

/// `-vv` and above is an explicit debugging session: everything.
fn terminal_directive(level: &str, verbose: u8) -> String {
    let base = match verbose {
        0 => level,
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    if verbose >= 2 {
        base.to_string()
    } else {
        format!("{base},{IROH_ADDRESS_LOGS_OFF}")
    }
}

/// File filter is fixed at info+, regardless of `-v`, so a
/// verbose interactive session does not blow up the on-disk
/// logs with trace spam.
fn file_filter() -> Result<EnvFilter> {
    EnvFilter::try_new(file_directive()).context("static file filter should always parse")
}

fn file_directive() -> String {
    format!("info,{IROH_ADDRESS_LOGS_OFF}")
}

/// Poor-man's TTY detection so CI / piped invocations get
/// plain-text logs without ANSI escape codes.
fn atty_stdout() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_log_keeps_address_directives_unless_it_targets_iroh() {
        assert_eq!(
            env_directive("info"),
            format!("info,{IROH_ADDRESS_LOGS_OFF}")
        );
        assert_eq!(
            env_directive("info,iroh::socket=trace"),
            "info,iroh::socket=trace"
        );
        assert_eq!(env_directive("iroh=debug"), "iroh=debug");
        // A crate whose name starts with "iroh" is not iroh itself.
        assert_eq!(
            env_directive("iroh_docs=debug"),
            format!("iroh_docs=debug,{IROH_ADDRESS_LOGS_OFF}")
        );
    }

    /// Whether `directive` lets an event of `level` at `target` through,
    /// asked of a real subscriber (targets match by prefix, the most
    /// specific directive wins — the reason this is not a string check).
    fn passes(directive: &str, target_level: (&str, tracing::Level)) -> bool {
        use tracing::Level;
        let filter = EnvFilter::try_new(directive).expect("directive parses");
        let subscriber = tracing_subscriber::registry()
            .with(fmt::layer().with_writer(std::io::sink).with_filter(filter));
        tracing::subscriber::with_default(subscriber, || match target_level {
            ("iroh::socket::remote_map::remote_state", Level::WARN) => {
                tracing::enabled!(target: "iroh::socket::remote_map::remote_state", Level::WARN)
            }
            ("iroh::socket::transports", Level::WARN) => {
                tracing::enabled!(target: "iroh::socket::transports", Level::WARN)
            }
            ("iroh::net_report::report", Level::WARN) => {
                tracing::enabled!(target: "iroh::net_report::report", Level::WARN)
            }
            ("iroh::socket::transports::relay::actor", Level::WARN) => {
                tracing::enabled!(target: "iroh::socket::transports::relay::actor", Level::WARN)
            }
            ("iroh::socket", Level::INFO) => tracing::enabled!(target: "iroh::socket", Level::INFO),
            ("iroh_docs::engine", Level::INFO) => {
                tracing::enabled!(target: "iroh_docs::engine", Level::INFO)
            }
            ("iroh_relay::client::tls", Level::INFO) => {
                tracing::enabled!(target: "iroh_relay::client::tls", Level::INFO)
            }
            other => panic!("no callsite for {other:?}"),
        })
    }

    #[test]
    fn log_filters_silence_iroh_address_logs() {
        use tracing::Level;
        for directive in [
            file_directive(),
            terminal_directive("info", 0),
            terminal_directive("warn", 1),
            env_directive("info"),
        ] {
            for addr_logging in [
                ("iroh::socket::remote_map::remote_state", Level::WARN),
                ("iroh::socket::transports", Level::WARN),
                ("iroh::net_report::report", Level::WARN),
                ("iroh::socket", Level::INFO),
                ("iroh_relay::client::tls", Level::INFO),
            ] {
                assert!(
                    !passes(&directive, addr_logging),
                    "{addr_logging:?} under {directive}"
                );
            }
            assert!(
                passes(
                    &directive,
                    ("iroh::socket::transports::relay::actor", Level::WARN)
                ),
                "relay diagnostics muted under {directive}"
            );
        }
        // Prefix trap: the other iroh crates keep their INFO.
        assert!(passes(
            &file_directive(),
            ("iroh_docs::engine", Level::INFO)
        ));
        assert_eq!(terminal_directive("info", 2), "debug");
    }
}
