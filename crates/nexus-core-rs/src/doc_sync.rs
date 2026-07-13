// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doc-sync keepalive — re-form the iroh-docs gossip neighborhood for a
//! shared document when it drops.
//!
//! ## Why this exists (Sprint 77 Phase A — WAN task delivery convergence)
//!
//! The live cross-machine attempt at the end of Sprint 76
//! (`sprint76_verification.md` §5.1) surfaced a hard blocker: a `task:`
//! entry written onto the project doc **after** a remote worker had
//! imported and subscribed never reached the worker's replica
//! (`recv:0`, "gossip neighborhood non formé"), reproduced on LAN and
//! WAN. The Sprint 77 Phase A preflight root-caused it against the
//! installed iroh-docs 0.98 source:
//!
//! - The coordinator's incremental broadcast (`LocalInsert ->
//!   gossip.broadcast`) is **gated by `is_syncing(namespace)`**
//!   (`iroh-docs-0.98.0 live.rs:711-718`), and `is_syncing` is only
//!   inserted into the sync-set by `start_sync` (`live.rs:409-414`).
//! - The coordinator opens the project doc via `create_doc`/`open_doc`
//!   and never calls `start_sync`; it relies entirely on the worker's
//!   **incoming dial** forming a gossip neighbor (`NeighborUp ->
//!   sync_with_peer`) to flip `is_syncing` true.
//! - `DocsApi::import(ticket)` calls `doc.start_sync(ticket.nodes)`
//!   exactly **once at boot** (`api.rs:220-225`), seeding the dial with
//!   the addresses **frozen into the ticket at share time**. On real
//!   transport (NAT rebind, relay change, stale ticket addresses, a
//!   hot binary swap over persistent `docs.redb`), that one dial does
//!   not form/maintain a neighbor, so the swarm for the namespace stays
//!   empty: the coordinator broadcasts nothing and the worker only ever
//!   receives the initial bulk sync — never the incremental writes.
//!
//! Recalibrated against iroh-docs 0.101 at the S81 Phase B/C bump —
//! mechanism unchanged: broadcast gate `is_syncing` (`live.rs:713`),
//! sync-set insert only via `start_sync` (`live.rs:408-414`),
//! incoming-sync reject `AbortReason::NotFound` (`state.rs:96-97`).
//! The line numbers cited in the S77 narrative above are the 0.98
//! ones it was root-caused against.
//!
//! The golden in-process example (`examples/two_nodes_docs_sync.rs`)
//! converges post-subscribe (verified under 0.98; the 2-node
//! convergence suite is green under 0.101), so the convergence
//! primitive is **not** broken — the gap is keeping the namespace's
//! gossip neighbor alive across transport churn. This module is that keepalive: it
//! observes the doc's `NeighborUp`/`NeighborDown` events and, whenever
//! the neighbor is absent, re-issues `Doc::start_sync(peers)`. Passing
//! the coordinator's [`EndpointAddr`] (which carries the endpoint id)
//! lets the `presets::N0` discovery wired at node boot re-resolve the
//! coordinator's **current** address via pkarr instead of staying stuck
//! on the ticket's frozen addresses (`crate::discovery`).
//!
//! It is deliberately **observability-only** on the read path: the
//! worker's task claim stays poll-based (`get_many_by_prefix`), and the
//! subscription here exists purely to detect a dropped neighbor. The
//! `LiveEvent` stream is drained best-effort so it can never apply
//! backpressure to the engine's hot path (PATTERNS §P54).
//!
//! ## Rejected alternatives (Sprint 77 D1)
//!
//! Periodic polling instead of subscribe, a parallel HTTP push channel,
//! and an N0 relay in the delivery hot path were all rejected at the
//! kickoff: this keepalive uses iroh's native pkarr/relay resolution via
//! the existing `start_sync` primitive, adds no new wire format, and
//! does not touch the `task:` doc key or any canonical bytes.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use futures_lite::{Stream, StreamExt};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::Result;
use crate::docs::{DocHandle, DocsLiveEvent};

/// Re-exported so callers (e.g. `nexus-worker-core`) can name the peer
/// address type the keepalive re-dials without taking a direct iroh
/// dependency. It is exactly the type carried by `DocTicket::nodes`.
pub use iroh::EndpointAddr;

/// Default cadence at which a doc with no live neighbor re-issues
/// `start_sync`. Long enough to avoid churn on a healthy sync, short
/// enough that a worker which lost its neighbor recovers well within the
/// minutes-long batch/async budget the sharding pipeline targets.
pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(15);

/// Minimum spacing between two re-join attempts. Bounds the rate of
/// `start_sync` calls when `NeighborDown` events arrive in a burst.
pub const DEFAULT_MIN_REJOIN_INTERVAL: Duration = Duration::from_secs(5);

/// Cold-boot warmup cadence: until the FIRST `NeighborUp` forms after a
/// (re)start, a doc with no live neighbor re-issues `start_sync` this often
/// instead of at `check_interval`. Sprint 82 Phase A: the S81-K live gap was
/// a worker cold-booted a few seconds before an incremental `task:` write
/// whose gossip neighbor had not yet formed — with only the 15s backstop the
/// namespace took minutes to converge (neighbor-formation latency, not a
/// missing `start_sync`). A ~1s warmup re-dials aggressively while the
/// neighborhood is still forming (pkarr/relay resolution retries), then
/// relaxes to the steady backstop once the first neighbor is up.
pub const DEFAULT_COLD_BOOT_CHECK_INTERVAL: Duration = Duration::from_secs(1);

/// Re-join cooldown during the cold-boot warmup (bounds the aggressive
/// cadence above so it cannot spin `start_sync`). Paired with
/// [`DEFAULT_COLD_BOOT_CHECK_INTERVAL`].
pub const DEFAULT_COLD_BOOT_MIN_REJOIN_INTERVAL: Duration = Duration::from_secs(1);

/// Wall-clock cap on the cold-boot warmup: relax to the steady backstop after
/// this long even if the first `NeighborUp` edge is never observed (Sprint 82
/// Phase A review P2). The subscription is edge-triggered, so an initial
/// `NeighborUp` fired before we subscribed — or a coordinator that never
/// becomes reachable — could otherwise pin the aggressive ~1s cadence forever.
/// Comfortably longer than a healthy cold-boot neighbor formation, short enough
/// that a stuck doc stops re-dialing aggressively within a minute.
pub const DEFAULT_COLD_BOOT_WINDOW: Duration = Duration::from_secs(60);

/// Tunables for [`spawn_doc_sync_keepalive`].
#[derive(Debug, Clone)]
pub struct KeepaliveConfig {
    /// Periodic backstop cadence: every `check_interval`, a doc that is
    /// not currently confirmed to have a live neighbor re-issues
    /// `start_sync`. Catches the case where the initial `NeighborUp`
    /// fired before this keepalive subscribed. Keep it `>=`
    /// [`KeepaliveConfig::min_rejoin_interval`] — a shorter cadence is
    /// harmless but its extra ticks are absorbed by the cooldown.
    pub check_interval: Duration,
    /// Cooldown between re-join attempts (rate-limits the `NeighborDown`
    /// fast path and the periodic backstop together).
    pub min_rejoin_interval: Duration,
    /// Backstop cadence used ONLY during the cold-boot warmup — before the
    /// first `NeighborUp` since (re)start. Once a neighbor forms the task
    /// relaxes to `check_interval`. Defaults to `check_interval` (no
    /// acceleration) so a caller that does not opt in keeps the exact Sprint
    /// 77 behavior; the worker opts into acceleration via
    /// [`KeepaliveConfig::cold_boot_aggressive`]. (Sprint 82 Phase A.)
    pub cold_boot_check_interval: Duration,
    /// Re-join cooldown during the cold-boot warmup. Defaults to
    /// `min_rejoin_interval`. (Sprint 82 Phase A.)
    pub cold_boot_min_rejoin_interval: Duration,
}

impl Default for KeepaliveConfig {
    fn default() -> Self {
        Self {
            check_interval: DEFAULT_CHECK_INTERVAL,
            min_rejoin_interval: DEFAULT_MIN_REJOIN_INTERVAL,
            // No warmup acceleration by default: the cold-boot cadence equals
            // the steady cadence, so an unaware caller keeps the exact Sprint
            // 77 behavior (0 observable change).
            cold_boot_check_interval: DEFAULT_CHECK_INTERVAL,
            cold_boot_min_rejoin_interval: DEFAULT_MIN_REJOIN_INTERVAL,
        }
    }
}

impl KeepaliveConfig {
    /// Steady backstop plus an aggressive cold-boot warmup: until the first
    /// `NeighborUp` since (re)start, re-issue `start_sync` every
    /// [`DEFAULT_COLD_BOOT_CHECK_INTERVAL`] instead of every
    /// [`DEFAULT_CHECK_INTERVAL`], then relax to the steady backstop. Closes
    /// the S81-K cold-boot convergence gap (Sprint 82 Phase A): a worker that
    /// boots seconds before an incremental `task:` write re-forms its gossip
    /// neighbor in seconds instead of minutes, without changing steady-state
    /// behavior. 0 dep, 0 wire — a cadence choice over the existing
    /// `start_sync` primitive.
    pub fn cold_boot_aggressive() -> Self {
        Self {
            cold_boot_check_interval: DEFAULT_COLD_BOOT_CHECK_INTERVAL,
            cold_boot_min_rejoin_interval: DEFAULT_COLD_BOOT_MIN_REJOIN_INTERVAL,
            ..Self::default()
        }
    }

    /// The backstop cadence for the current warmth: the aggressive cold-boot
    /// interval until the first neighbor forms (`warm == false`), then the
    /// steady interval. Used by [`spawn_doc_sync_keepalive`]'s ticker.
    fn check_interval_for(&self, warm: bool) -> Duration {
        if warm {
            self.check_interval
        } else {
            self.cold_boot_check_interval
        }
    }

    /// The re-join cooldown for the current warmth (aggressive while cold,
    /// steady once a neighbor has formed).
    fn min_rejoin_for(&self, warm: bool) -> Duration {
        if warm {
            self.min_rejoin_interval
        } else {
            self.cold_boot_min_rejoin_interval
        }
    }
}

/// One pulled item from the observability subscription.
enum Pulled {
    Event(DocsLiveEvent),
    /// The stream errored or ended — the caller should reconnect.
    Ended,
}

/// Poll the (optional) live-event stream. When there is no stream
/// (subscribe failed) this never resolves, so the periodic timer drives
/// the loop on its own.
async fn pull<S>(stream: &mut Option<S>) -> Pulled
where
    S: Stream<Item = Result<DocsLiveEvent>> + Unpin,
{
    match stream {
        Some(s) => match s.next().await {
            Some(Ok(ev)) => Pulled::Event(ev),
            Some(Err(_)) | None => Pulled::Ended,
        },
        None => std::future::pending().await,
    }
}

/// Re-issue `start_sync(peers)` if the cooldown has elapsed, advancing
/// `last_rejoin` on success. `start_sync` is idempotent on an already
/// syncing doc; passing the coordinator's `EndpointAddr` lets iroh
/// re-resolve its current address via pkarr (`presets::N0`).
async fn rejoin(
    doc: &DocHandle,
    peers: &[EndpointAddr],
    min_rejoin: Duration,
    last_rejoin: &mut Instant,
) {
    if last_rejoin.elapsed() < min_rejoin {
        return;
    }
    match doc.inner().start_sync(peers.to_vec()).await {
        Ok(()) => {
            *last_rejoin = Instant::now();
            debug!(doc = %doc.id(), "doc-sync keepalive re-joined gossip neighborhood");
        }
        Err(e) => {
            warn!(doc = %doc.id(), error = %e, "doc-sync keepalive start_sync re-join failed")
        }
    }
}

/// Spawn a keepalive task that keeps `doc`'s gossip neighborhood alive
/// by re-issuing `start_sync(peers)` whenever the neighbor is absent.
///
/// `peers` are the coordinator addresses from the doc ticket
/// (`DocTicket::nodes`). With an empty `peers` list there is nothing to
/// re-dial, so the task exits immediately (e.g. docs injected directly
/// in tests via `register_task_doc`).
///
/// `shutdown` is a `watch` flipped by the owner on teardown; any change
/// (including the sender being dropped) stops the task, mirroring
/// [`crate`]'s `spawn_result_subscribe` shutdown convention.
pub fn spawn_doc_sync_keepalive(
    doc: DocHandle,
    peers: Vec<EndpointAddr>,
    config: KeepaliveConfig,
    mut shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if peers.is_empty() {
            debug!(doc = %doc.id(), "doc-sync keepalive: no peers to re-dial; not started");
            return;
        }

        let mut backoff = Duration::from_millis(500);
        let max_backoff = Duration::from_secs(30);
        // Initialise so the very first periodic tick is allowed to
        // re-join immediately — recovering a doc that was imported
        // before its coordinator became reachable.
        let mut last_rejoin = Instant::now()
            .checked_sub(config.min_rejoin_interval)
            .unwrap_or_else(Instant::now);
        // Sprint 82 Phase A: `false` until the cold-boot warmup is over —
        // whichever comes first, the FIRST `NeighborUp` since this task started
        // OR the `DEFAULT_COLD_BOOT_WINDOW` wall-clock deadline (review P2: the
        // edge-triggered subscription can miss the initial NeighborUp, which
        // must not pin the aggressive ~1s cadence forever). While cold, the
        // ticker + re-join cooldown use the aggressive `cold_boot_*` cadence so
        // a cold-booted replica forms its neighbor in seconds; once warm we
        // relax to the steady backstop. Persisted across subscription
        // reconnects so a reconnect before warmup completes stays aggressive.
        let mut warm = false;
        let started_at = Instant::now();

        loop {
            // (Re)subscribe for observability. Best-effort: if it fails
            // the periodic timer still drives re-joins.
            let mut stream = match tokio::time::timeout(Duration::from_secs(30), doc.subscribe())
                .await
            {
                Ok(Ok(s)) => Some(s),
                Ok(Err(e)) => {
                    warn!(doc = %doc.id(), error = %e, "doc-sync keepalive subscribe failed; timer-only");
                    None
                }
                Err(_) => {
                    warn!(doc = %doc.id(), "doc-sync keepalive subscribe timed out; timer-only");
                    None
                }
            };

            // Reset the reconnect backoff once a subscription is healthy
            // again, so a later transient drop restarts from the short
            // delay rather than the capped one (mirrors the reconnect
            // contract of `nexus-shell-daemon::result_sync`).
            if stream.is_some() {
                backoff = Duration::from_millis(500);
            }

            // A fresh interval per (re)subscribe. `interval` fires its
            // first tick immediately, so re-entering the loop triggers an
            // (idempotent, cooldown-gated) re-join. Sprint 82 Phase A: use the
            // aggressive cadence until the first neighbor forms (`!warm`),
            // then the steady backstop.
            let mut ticker = tokio::time::interval(config.check_interval_for(warm));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            // Currently-up neighbors, keyed by their endpoint id string.
            let mut neighbors: HashSet<String> = HashSet::new();

            loop {
                tokio::select! {
                    biased;

                    _ = shutdown.changed() => {
                        info!(doc = %doc.id(), "doc-sync keepalive shutting down");
                        return;
                    }

                    pulled = pull(&mut stream) => {
                        match pulled {
                            Pulled::Event(DocsLiveEvent::NeighborUp(pk)) => {
                                neighbors.insert(pk.to_string());
                                // Sprint 82 Phase A: the cold-boot neighborhood
                                // has formed — relax from the aggressive warmup
                                // cadence to the steady backstop. Rebuilding the
                                // ticker fires an immediate first tick, but a
                                // neighbor is now up so the `is_empty()` guard
                                // skips the re-join.
                                if !warm {
                                    warm = true;
                                    ticker = tokio::time::interval(config.check_interval);
                                    ticker.set_missed_tick_behavior(
                                        tokio::time::MissedTickBehavior::Skip,
                                    );
                                }
                            }
                            Pulled::Event(DocsLiveEvent::NeighborDown(pk)) => {
                                neighbors.remove(&pk.to_string());
                                // Fast path: a dropped neighbor is exactly
                                // the prod failure — re-join immediately
                                // (cooldown-gated) rather than waiting for
                                // the next periodic tick.
                                if neighbors.is_empty() {
                                    rejoin(&doc, &peers, config.min_rejoin_for(warm), &mut last_rejoin)
                                        .await;
                                }
                            }
                            // Drain every other live event best-effort so the
                            // 64-slot subscription buffer never backs up.
                            Pulled::Event(_) => {}
                            Pulled::Ended => break, // reconnect the subscription
                        }
                    }

                    _ = ticker.tick() => {
                        // Sprint 82 Phase A (review P2): close the cold-boot
                        // warmup on the wall-clock deadline even if no
                        // NeighborUp edge was ever observed, so a stuck doc
                        // relaxes to the steady backstop instead of re-dialing
                        // at ~1s forever. The rebuilt ticker's immediate first
                        // tick is harmless (cooldown-gated, and warm is now set).
                        if !warm && started_at.elapsed() >= DEFAULT_COLD_BOOT_WINDOW {
                            warm = true;
                            ticker = tokio::time::interval(config.check_interval);
                            ticker.set_missed_tick_behavior(
                                tokio::time::MissedTickBehavior::Skip,
                            );
                        }
                        // Periodic backstop: re-join when no neighbor is
                        // currently tracked. Covers the case where the
                        // initial NeighborUp fired before we subscribed.
                        if neighbors.is_empty() {
                            rejoin(&doc, &peers, config.min_rejoin_for(warm), &mut last_rejoin).await;
                        }
                    }
                }
            }

            // Reconnect gap with exponential backoff, still responsive to
            // shutdown.
            tokio::select! {
                _ = tokio::time::sleep(backoff) => {}
                _ = shutdown.changed() => { return; }
            }
            backoff = (backoff * 2).min(max_backoff);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_node;
    use crate::discovery::DiscoveryClient;
    use crate::docs::DocsClient;
    use std::time::Duration;

    /// Poll `get_latest_by_key` until the key is present or the deadline
    /// elapses. Returns `true` if the entry converged, `false` on
    /// timeout — author-agnostic so it works for entries written under a
    /// remote author.
    async fn key_converges(doc: &DocHandle, key: &[u8], within: Duration) -> bool {
        let deadline = Instant::now() + within;
        loop {
            if let Ok(Some(_)) = doc.get_latest_by_key(key).await {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Phase A red→green, deterministic, in one test (the "control"
    /// phase is the revert-proof: without a re-join the detached replica
    /// never converges; with the keepalive it does).
    ///
    /// Models the prod failure faithfully: a worker whose gossip neighbor
    /// with the coordinator drops (here forced by `leave()`, prod cause =
    /// NAT rebind / stale ticket addr / binary swap) stops receiving the
    /// coordinator's incremental writes — exactly the `recv:0` symptom —
    /// and the keepalive's `start_sync` re-join recovers convergence.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn keepalive_rejoins_doc_after_neighbor_loss() {
        let node_a = create_node().await.expect("boot node A (coordinator)");
        let node_b = create_node().await.expect("boot node B (worker replica)");

        let docs_a = DocsClient::new(node_a.docs());
        let docs_b = DocsClient::new(node_b.docs());
        let author_a = docs_a.author_default().await.expect("author A");
        let doc_a = docs_a.create_doc().await.expect("create doc on A");

        // Seed A's address into B's lookup so the dial resolves without
        // depending on live pkarr DHT timing (same trick the existing
        // 2-node tests and `blobs.rs::fetch_ticket` use).
        let a_addr = DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("A publishes its address");
        node_b.memory_lookup().add_endpoint_info(a_addr.clone());

        // B imports A's write ticket → start_sync → gossip neighbor forms.
        let ticket = doc_a.share_write().await.expect("share write ticket");
        let doc_b = docs_b.import_ticket(ticket).await.expect("B imports doc");

        // Baseline: an incremental write reaches B live (proves the
        // neighbor is up and the wiring delivers post-subscribe writes).
        doc_a
            .set(author_a, b"k1".to_vec(), b"v1".to_vec())
            .await
            .expect("A writes k1");
        assert!(
            key_converges(&doc_b, b"k1", Duration::from_secs(10)).await,
            "baseline: B must receive the incremental write k1 while the neighbor is up"
        );

        // Neighbor loss: B leaves the swarm. Stand-in for the prod cause
        // (dropped connection / stale ticket addr / hot binary swap).
        doc_b
            .inner()
            .leave()
            .await
            .expect("B leaves the gossip swarm");

        // A writes k2 while B is detached.
        doc_a
            .set(author_a, b"k2".to_vec(), b"v2".to_vec())
            .await
            .expect("A writes k2");

        // CONTROL (the red proof): with no re-join, B must NOT converge
        // on k2. This is the `recv:0` bug reproduced deterministically.
        assert!(
            !key_converges(&doc_b, b"k2", Duration::from_secs(2)).await,
            "control: a detached replica must NOT receive k2 without a re-join (reproduces recv:0)"
        );

        // FIX: spawn the keepalive → it re-joins the gossip neighborhood
        // and B converges on k2.
        let (stop_tx, stop_rx) = watch::channel(false);
        // cold_boot_* == steady here: this test exercises the NeighborDown
        // fast path (post-warm), not the cold-boot warmup, so the two
        // cadences are kept equal to preserve the original timing.
        let cfg = KeepaliveConfig {
            check_interval: Duration::from_millis(500),
            min_rejoin_interval: Duration::from_millis(200),
            cold_boot_check_interval: Duration::from_millis(500),
            cold_boot_min_rejoin_interval: Duration::from_millis(200),
        };
        let handle = spawn_doc_sync_keepalive(doc_b.clone(), vec![a_addr], cfg, stop_rx);

        assert!(
            key_converges(&doc_b, b"k2", Duration::from_secs(15)).await,
            "fix: the keepalive must re-join and converge k2 onto the detached replica"
        );

        // Clean shutdown.
        stop_tx.send(true).expect("signal keepalive shutdown");
        handle.await.expect("keepalive task joins");
        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    /// Sprint 82 Phase A — the cold-boot cadence, deterministic revert-proof.
    ///
    /// The convergence BENEFIT of the aggressive cold-boot cadence is a
    /// real-transport property (a failed pkarr/relay dial must be re-issued,
    /// and 1s re-issues form the neighbor minutes faster than 15s ones); it is
    /// NOT reproducible in-process, where a single `start_sync`'s pending dial
    /// resolves in the background the instant the address appears in the memory
    /// lookup — the rejoin frequency is then irrelevant. So the live proof is
    /// the T2 rig re-jeu (`b3_live_pc_vps.sh`, PASS<30s); here we lock the
    /// cadence-selection LOGIC that the keepalive loop uses via
    /// `check_interval_for` / `min_rejoin_for`. SCOPE (review P2-4): this
    /// revert-proofs the CONSTRUCTOR contract — replace `cold_boot_aggressive()`
    /// with `default()` on line one and the strict inequalities below fail. It
    /// does NOT guard the worker's opt-in call-site
    /// (`nexus-worker-core/engine/runtime.rs`); that one behavioral line closing
    /// S81-K is proven only by the T2 live re-jeu.
    #[test]
    fn cold_boot_config_accelerates_only_the_cold_window() {
        let cold = KeepaliveConfig::cold_boot_aggressive();
        // Cold window (before the first NeighborUp) is strictly more aggressive
        // than the steady backstop, on both the ticker cadence and the cooldown.
        assert_eq!(
            cold.check_interval_for(false),
            DEFAULT_COLD_BOOT_CHECK_INTERVAL
        );
        assert!(
            cold.check_interval_for(false) < cold.check_interval_for(true),
            "cold-boot must re-dial faster than the steady backstop"
        );
        assert!(cold.min_rejoin_for(false) <= cold.min_rejoin_for(true));
        // Once warm it relaxes to the EXACT Sprint 77 steady backstop.
        assert_eq!(cold.check_interval_for(true), DEFAULT_CHECK_INTERVAL);
        assert_eq!(cold.min_rejoin_for(true), DEFAULT_MIN_REJOIN_INTERVAL);

        // The default keeps the Sprint 77 behavior verbatim: cold == steady (no
        // acceleration), so any caller that does not opt in is unaffected — 0
        // observable change (the constructor contract, not the worker call-site).
        let d = KeepaliveConfig::default();
        assert_eq!(d.check_interval_for(false), d.check_interval_for(true));
        assert_eq!(d.min_rejoin_for(false), d.min_rejoin_for(true));
    }

    /// With no peers there is nothing to re-dial, so the keepalive must
    /// exit immediately instead of spinning.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn keepalive_without_peers_exits_immediately() {
        let node = create_node().await.expect("boot node");
        let docs = DocsClient::new(node.docs());
        let doc = docs.create_doc().await.expect("create doc");

        let (_stop_tx, stop_rx) = watch::channel(false);
        let handle = spawn_doc_sync_keepalive(doc, Vec::new(), KeepaliveConfig::default(), stop_rx);
        // Must finish on its own without any shutdown signal.
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("keepalive with no peers must exit promptly")
            .expect("keepalive task joins");

        node.shutdown().await.ok();
    }
}
