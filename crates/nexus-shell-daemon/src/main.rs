// SPDX-License-Identifier: AGPL-3.0-or-later
//! # nexus-shell-daemon
//!
//! SBFB shell daemon: the long-lived P2P process that backs the
//! React shell's Browse / Curators pages. One instance per user,
//! reached by the shell exclusively via the coordinator
//! `/daemon/*` proxy (Sprint 7 D1 — HTTP loopback, not Unix
//! socket, not cross-origin direct call).
//!
//! ## Sprint 7 Phase A architecture
//!
//! This binary is a thin CLI + runtime wrapper around the
//! headless engine living in [`nexus_shell_daemon_core`]. The
//! split mirrors the Sprint 3 worker-core / worker pair: the
//! engine must run and be fully testable without any axum
//! server or clap parser.
//!
//! ## Phase A scope
//!
//! - `start` — singleton check → iroh node boot → HTTP serve on
//!   an ephemeral loopback port → `running.json` write → block
//!   on ctrl+c → graceful shutdown.
//! - `stop` / `status` / `config` — stubs that land in Phase E
//!   alongside the coordinator proxy.
//!
//! Phase C adds the curator gossip subscribe pipeline; Phase D
//! adds pkarr browse resolution; Phase E adds the coordinator
//! proxy + the shell pages. **None of those are in Phase A.**

mod apps;
mod canary_api;
mod cli;
mod consent;
mod contributor_api;
mod deploy;
mod diagnostic_api;
mod files;
mod health_api;
mod http;
mod invite_api;
mod kudos_api;
mod logging;
#[cfg(windows)]
mod named_pipe_server;
mod noop_identity;
mod panic;
mod quarantine_api;
mod runtime;
mod shell_api;
mod tasks_api;
#[cfg(unix)]
mod uds_server;
mod validator_loop;
mod worker_state_api;

use anyhow::{Context, Result};
use clap::Parser;
use nexus_shell_daemon_core::config::{ShellDaemonConfig, ShellDaemonPaths};

use cli::{CanaryCommand, Cli, Command, ConfigCommand, FrostCommand};
use runtime::{DaemonRuntime, DaemonStartOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let paths = ShellDaemonPaths::resolve(cli.config.clone())
        .context("could not resolve shell-daemon paths for this platform")?;

    // Initialize structured logging. Try to honour the config's
    // log level if it exists; fall back to "info" before the
    // config is written (e.g. the very first `start` on a fresh
    // machine) so early errors still surface.
    let level = match ShellDaemonConfig::load(&paths.config_file) {
        Ok(cfg) => cfg.logging.level,
        Err(_) => "info".to_string(),
    };
    paths
        .ensure_dirs()
        .context("failed to create shell-daemon directories")?;
    let _log_guard = logging::init_logging(&paths.log_dir, &level, cli.verbose)
        .context("failed to initialize tracing subscriber")?;

    tracing::debug!(
        core_version = nexus_shell_daemon_core::VERSION,
        "nexus-shell-daemon parsed CLI"
    );

    match cli.command {
        Command::Start {
            headless,
            cors_origins,
        } => handle_start(paths, headless, cors_origins).await,
        Command::Stop => handle_stop(&paths).await,
        Command::Status => handle_status(&paths).await,
        Command::Config(cmd) => handle_config(&paths, cmd).await,
        Command::Canary(cmd) => handle_canary(cmd).await,
    }
}

async fn handle_start(
    paths: ShellDaemonPaths,
    _headless: bool,
    cli_cors_origins: Vec<String>,
) -> Result<()> {
    // Load the config to pick up any user-tuned bind host / port.
    // A missing file is fine — Phase A defaults to 127.0.0.1:0.
    let cfg = ShellDaemonConfig::load(&paths.config_file)
        .context("failed to load shell-daemon config")?;

    println!("nexus-shell-daemon v{}", env!("CARGO_PKG_VERSION"));
    println!("  core version: {}", nexus_shell_daemon_core::VERSION);
    println!("  config:       {}", paths.config_file.display());
    println!("  running.json: {}", paths.running_json.display());
    println!();

    // Sprint 20 Phase B : the launcher sets `SBFB_IDENTITY_MODE=
    // duress` in the child's environment when `sbfb unlock`
    // matched the duress blob. Any other value (including unset)
    // falls through to `Normal`. The env var is read inside the
    // daemon process and never persisted on disk, matching the
    // Phase A pattern for `SBFB_IDENTITY_SECRET_HEX`.
    let cors_origins = if cli_cors_origins.is_empty() {
        std::env::var("NEXUS_DAEMON_CORS_ORIGINS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        cli_cors_origins
    };
    for origin in &cors_origins {
        if !crate::http::is_valid_origin(origin) {
            anyhow::bail!(
                "invalid --cors-origin value: {origin:?} (expected http(s)://host[:port])"
            );
        }
    }

    let identity_mode = match std::env::var("SBFB_IDENTITY_MODE").ok().as_deref() {
        Some("duress") => nexus_core_rs::IdentityMode::Duress,
        _ => nexus_core_rs::IdentityMode::Normal,
    };
    std::env::remove_var("SBFB_IDENTITY_MODE");

    let opts = DaemonStartOptions {
        paths,
        api_host: cfg.network.api_host.clone(),
        api_port: cfg.network.api_port,
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
        curator: cfg.curator.clone(),
        identity_mode,
        cors_origins,
    };

    let runtime = DaemonRuntime::start(opts)
        .await
        .context("daemon start failed")?;

    let addr = runtime.bound_addr();
    println!("  listening on: http://{}", addr);
    println!("  (press ctrl+c to shut down)");
    println!();

    runtime
        .wait_shutdown()
        .await
        .context("ctrl+c handler failed")?;

    println!();
    println!("nexus-shell-daemon shutting down...");
    runtime.shutdown().await.context("shutdown failed")?;
    println!("nexus-shell-daemon exited cleanly.");
    Ok(())
}

async fn handle_stop(paths: &ShellDaemonPaths) -> Result<()> {
    print_stub(
        "stop",
        "Phase E (coordinator proxy + process signaling)",
        &[("running_json", &paths.running_json.display().to_string())],
    );
    Ok(())
}

async fn handle_status(paths: &ShellDaemonPaths) -> Result<()> {
    print_stub(
        "status",
        "Phase E (coordinator proxy + /info passthrough)",
        &[("running_json", &paths.running_json.display().to_string())],
    );
    Ok(())
}

async fn handle_config(_paths: &ShellDaemonPaths, cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Get { key } => print_stub(
            "config get",
            "Phase E (config persist + dotted key lookup)",
            &[("key", key.as_str())],
        ),
        ConfigCommand::Set { key, value } => print_stub(
            "config set",
            "Phase E (config persist + dotted key lookup)",
            &[("key", key.as_str()), ("value", value.as_str())],
        ),
    }
    Ok(())
}

async fn handle_canary(cmd: CanaryCommand) -> Result<()> {
    use nexus_shell_daemon_core::canary::{
        build_canary, format_canary_txt, parse_canary_txt, publish_canary, today_utc,
        warrant_canary_topic_id, CanaryBroadcaster, Ed25519CanarySigner,
    };

    match cmd {
        CanaryCommand::Publish {
            headline,
            output,
            no_gossip,
        } => {
            // 1. Load (or create) the maintainer's persistent
            //    canary key. Separate from the daemon's ephemeral
            //    node identity on purpose — see the `canary_key_path`
            //    doc for the rationale.
            let key_path = nexus_shell_daemon_core::auth::canary_key_path().with_context(|| {
                "could not resolve SBFB home dir — set $SBFB_HOME or $HOME/$USERPROFILE"
            })?;
            let keypair =
                nexus_core_rs::KeyPair::load_or_generate(&key_path).with_context(|| {
                    format!(
                        "failed to load or create canary key at {}",
                        key_path.display()
                    )
                })?;
            // Wrap the keypair in the Sprint 20 Phase E.1
            // CanarySigner trait so the build_canary path stays
            // algorithm-agnostic (FrostCanarySigner is the opt-in
            // K-of-N alternative for cross-juridiction maintainer
            // federation).
            let signer = Ed25519CanarySigner::new(keypair);

            // 2. Build + sign the canary.
            let canary = build_canary(today_utc(), headline, &signer)
                .context("failed to build signed canary")?;

            // 3. Write the human-readable mirror.
            let txt = format_canary_txt(&canary);
            std::fs::write(&output, &txt).with_context(|| {
                format!("failed to write canary mirror to {}", output.display())
            })?;

            println!("SBFB canary written to {}", output.display());
            println!("  date:         {}", canary.signed.date);
            println!("  headline:     {}", canary.signed.headline);
            println!("  next update:  {}", canary.signed.next_update);
            println!("  pubkey:       {}", canary.signed.pubkey_hex);
            println!("  key file:     {}", key_path.display());

            // 4. Broadcast on gossip unless opted out. Booting an
            //    iroh node is slow, so keep the noop fast path
            //    when CI just wants to refresh the repo-side file.
            if no_gossip {
                println!("  gossip:       skipped (--no-gossip)");
                return Ok(());
            }

            let node = nexus_core_rs::create_node()
                .await
                .context("failed to boot iroh endpoint for canary broadcast")?;
            let gossip = nexus_core_rs::GossipClient::new(node.gossip());
            let mut topic = gossip
                .join_topic(warrant_canary_topic_id(), Vec::new())
                .await
                .context("failed to join warrant canary gossip topic")?;

            struct TopicBroadcaster<'a> {
                inner: &'a mut nexus_core_rs::TopicHandle,
            }
            #[async_trait::async_trait]
            impl<'a> CanaryBroadcaster for TopicBroadcaster<'a> {
                async fn broadcast(&mut self, bytes: Vec<u8>) -> Result<(), String> {
                    self.inner.broadcast(bytes).await.map_err(|e| e.to_string())
                }
            }

            let mut broadcaster = TopicBroadcaster { inner: &mut topic };
            publish_canary(&canary, &mut broadcaster)
                .await
                .context("gossip broadcast of canary failed")?;
            node.shutdown().await.ok();

            println!("  gossip:       broadcast on warrant-canary/v1");
            Ok(())
        }

        CanaryCommand::Frost(frost_cmd) => handle_frost(frost_cmd).await,

        CanaryCommand::Verify { input } => {
            let text = std::fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))?;
            let canary =
                parse_canary_txt(&text).with_context(|| "canary file is not in SBFB format")?;
            nexus_shell_daemon_core::canary::verify_canary(&canary)
                .with_context(|| "signature does not validate")?;

            println!("canary OK");
            println!("  date:         {}", canary.signed.date);
            println!("  headline:     {}", canary.signed.headline);
            println!("  next update:  {}", canary.signed.next_update);
            println!("  pubkey:       {}", canary.signed.pubkey_hex);
            Ok(())
        }
    }
}

async fn handle_frost(cmd: FrostCommand) -> Result<()> {
    use nexus_core_rs::canonical::{canonical_bytes, DOMAIN_WARRANT_CANARY_V1};
    use nexus_shell_daemon_core::canary::{
        build_signing_package, ceremony_aggregate, ceremony_round1, ceremony_round2,
        dkg::{generate_dkg, load_pubkey, load_share},
        format_canary_txt, today_utc, verify_canary, Canary, CanarySigned, CANARY_VALIDITY_DAYS,
        CANARY_VERSION,
    };
    use time::Duration;

    match cmd {
        FrostCommand::TrustedDealer { k, n, output_dir } => {
            std::fs::create_dir_all(&output_dir)
                .with_context(|| format!("failed to create output dir {}", output_dir.display()))?;

            let (share_files, pubkey_file) = generate_dkg(k, n)
                .with_context(|| format!("FROST trusted dealer DKG failed (K={k}, N={n})"))?;

            for sf in &share_files {
                let path = output_dir.join(format!("canary-share-{}.frost.json", sf.participant));
                let json = serde_json::to_string_pretty(sf).context("serialize share file")?;
                std::fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
                println!("  share {}: {}", sf.participant, path.display());
            }

            let pp_path = output_dir.join("canary-pubkey-package.frost.json");
            let pp_json =
                serde_json::to_string_pretty(&pubkey_file).context("serialize pubkey file")?;
            std::fs::write(&pp_path, pp_json)
                .with_context(|| format!("write {}", pp_path.display()))?;

            println!("  pubkey:   {}", pp_path.display());
            println!("  K={k}, N={n}");
            println!("  verifying key: {}", pubkey_file.verifying_key_hex);
            println!();
            println!("Distribute each share file to its participant via a");
            println!("separate secure channel. Destroy this machine's RNG");
            println!("seed after distribution (cf. WARRANT_CANARY_HARDENING §4.2).");
            Ok(())
        }

        FrostCommand::Round1 {
            share,
            commitment,
            nonces,
        } => {
            let share_json = std::fs::read_to_string(&share)
                .with_context(|| format!("read {}", share.display()))?;
            let share_file: nexus_shell_daemon_core::canary::DkgShareFile =
                serde_json::from_str(&share_json).context("parse share file")?;
            let frost_share = load_share(&share_file).context("load FROST share")?;

            let (c, n) = ceremony_round1(share_file.participant, &frost_share.key_package)
                .context("round 1 failed")?;

            let c_json = serde_json::to_string_pretty(&c).context("serialize commitment")?;
            std::fs::write(&commitment, &c_json)
                .with_context(|| format!("write {}", commitment.display()))?;

            let n_json = serde_json::to_string_pretty(&n).context("serialize nonces")?;
            std::fs::write(&nonces, &n_json)
                .with_context(|| format!("write {}", nonces.display()))?;

            println!("  participant: {}", share_file.participant);
            println!("  commitment:  {}", commitment.display());
            println!(
                "  nonces:      {} (SECRET — do not share)",
                nonces.display()
            );
            Ok(())
        }

        FrostCommand::BuildSigningPackage {
            commitments,
            pubkey_package: pp_path,
            headline,
            output,
        } => {
            let mut commitment_list = Vec::with_capacity(commitments.len());
            for path in &commitments {
                let json = std::fs::read_to_string(path)
                    .with_context(|| format!("read {}", path.display()))?;
                let c: nexus_shell_daemon_core::canary::CeremonyCommitment =
                    serde_json::from_str(&json).context("parse commitment")?;
                commitment_list.push(c);
            }

            let pp_json = std::fs::read_to_string(&pp_path)
                .with_context(|| format!("read {}", pp_path.display()))?;
            let pp_file: nexus_shell_daemon_core::canary::DkgPubkeyFile =
                serde_json::from_str(&pp_json).context("parse pubkey package")?;

            let date = today_utc();
            let next_update = date.saturating_add(Duration::days(CANARY_VALIDITY_DAYS));
            let signed = CanarySigned {
                version: CANARY_VERSION,
                date: format!(
                    "{:04}-{:02}-{:02}",
                    date.year(),
                    u8::from(date.month()),
                    date.day()
                ),
                headline: headline.clone(),
                next_update: format!(
                    "{:04}-{:02}-{:02}",
                    next_update.year(),
                    u8::from(next_update.month()),
                    next_update.day()
                ),
                pubkey_hex: pp_file.verifying_key_hex,
            };
            let canonical =
                canonical_bytes(&signed, DOMAIN_WARRANT_CANARY_V1).context("canonical bytes")?;

            let sp = build_signing_package(&commitment_list, &canonical)
                .context("build signing package")?;

            let sp_json = serde_json::to_string_pretty(&sp).context("serialize signing package")?;
            std::fs::write(&output, &sp_json)
                .with_context(|| format!("write {}", output.display()))?;

            println!("  signing package: {}", output.display());
            println!("  headline:        {}", headline);
            println!("  commitments:     {}", commitments.len());
            Ok(())
        }

        FrostCommand::Round2 {
            share,
            nonces,
            signing_package,
            output,
        } => {
            let share_json = std::fs::read_to_string(&share)
                .with_context(|| format!("read {}", share.display()))?;
            let share_file: nexus_shell_daemon_core::canary::DkgShareFile =
                serde_json::from_str(&share_json).context("parse share")?;
            let frost_share = load_share(&share_file).context("load share")?;

            let nonces_json = std::fs::read_to_string(&nonces)
                .with_context(|| format!("read {}", nonces.display()))?;
            let nonces_data: nexus_shell_daemon_core::canary::CeremonyNonces =
                serde_json::from_str(&nonces_json).context("parse nonces")?;

            let sp_json = std::fs::read_to_string(&signing_package)
                .with_context(|| format!("read {}", signing_package.display()))?;
            let sp: nexus_shell_daemon_core::canary::CeremonySigningPackage =
                serde_json::from_str(&sp_json).context("parse signing package")?;

            let ss = ceremony_round2(&nonces_data, &sp, &frost_share.key_package)
                .context("round 2 failed")?;

            let ss_json = serde_json::to_string_pretty(&ss).context("serialize sig share")?;
            std::fs::write(&output, &ss_json)
                .with_context(|| format!("write {}", output.display()))?;

            println!("  participant:  {}", share_file.participant);
            println!("  sig share:    {}", output.display());
            println!("  (destroy nonces file now)");
            Ok(())
        }

        FrostCommand::Aggregate {
            pubkey_package,
            signing_package,
            shares,
            headline,
            output,
        } => {
            let pp_json = std::fs::read_to_string(&pubkey_package)
                .with_context(|| format!("read {}", pubkey_package.display()))?;
            let pp_file: nexus_shell_daemon_core::canary::DkgPubkeyFile =
                serde_json::from_str(&pp_json).context("parse pubkey package")?;
            let pubkey = load_pubkey(&pp_file).context("load pubkey")?;

            let sp_json = std::fs::read_to_string(&signing_package)
                .with_context(|| format!("read {}", signing_package.display()))?;
            let sp: nexus_shell_daemon_core::canary::CeremonySigningPackage =
                serde_json::from_str(&sp_json).context("parse signing package")?;

            let mut sig_shares = Vec::with_capacity(shares.len());
            for path in &shares {
                let json = std::fs::read_to_string(path)
                    .with_context(|| format!("read {}", path.display()))?;
                let ss: nexus_shell_daemon_core::canary::CeremonySignatureShare =
                    serde_json::from_str(&json).context("parse sig share")?;
                sig_shares.push(ss);
            }

            let sig = ceremony_aggregate(&sp, &sig_shares, pubkey.package())
                .context("aggregate failed")?;

            let date = today_utc();
            let next_update = date.saturating_add(Duration::days(CANARY_VALIDITY_DAYS));
            let canary = Canary {
                signed: CanarySigned {
                    version: CANARY_VERSION,
                    date: format!(
                        "{:04}-{:02}-{:02}",
                        date.year(),
                        u8::from(date.month()),
                        date.day()
                    ),
                    headline,
                    next_update: format!(
                        "{:04}-{:02}-{:02}",
                        next_update.year(),
                        u8::from(next_update.month()),
                        next_update.day()
                    ),
                    pubkey_hex: pp_file.verifying_key_hex.clone(),
                },
                signature_hex: hex::encode(sig),
            };

            verify_canary(&canary).context("self-verification of aggregated canary")?;

            let txt = format_canary_txt(&canary);
            std::fs::write(&output, &txt).with_context(|| format!("write {}", output.display()))?;

            println!("FROST canary aggregated and verified.");
            println!("  date:         {}", canary.signed.date);
            println!("  headline:     {}", canary.signed.headline);
            println!("  next update:  {}", canary.signed.next_update);
            println!("  pubkey:       {}", canary.signed.pubkey_hex);
            println!("  output:       {}", output.display());
            Ok(())
        }
    }
}

/// Uniform placeholder output for unimplemented subcommands.
///
/// Kept as a free function so the stub format is trivially
/// grep-able across every handler: when a real implementation
/// lands, `grep print_stub` flags any handler still on the stub
/// path.
fn print_stub(name: &str, phase: &str, args: &[(&str, &str)]) {
    println!("nexus-shell-daemon v{}", env!("CARGO_PKG_VERSION"));
    println!("  core version: {}", nexus_shell_daemon_core::VERSION);
    println!("  subcommand:   {name}");
    println!("  status:       not yet implemented, see Sprint 7 {phase}");
    if !args.is_empty() {
        println!("  args:");
        for (k, v) in args {
            println!("    {k} = {v}");
        }
    }
}
