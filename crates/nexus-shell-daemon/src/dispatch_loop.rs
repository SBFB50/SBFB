// SPDX-License-Identifier: AGPL-3.0-or-later
//! MPSC dispatch loop — writes TaskEntry values to the project iroh doc.
//!
//! Sprint 49 Phase A (G1 D2 ack): the dispatch loop is the **sole writer**
//! to the project doc. HTTP endpoints submit signed TaskEntry values into
//! the MPSC channel; this loop drains them and writes sequentially.

use std::sync::Arc;

use nexus_core_rs::docs::{DocHandle, DocsAuthorId};
use nexus_core_rs::task::TaskEntry;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};

pub type TaskEntrySender = mpsc::Sender<TaskEntry>;

const CHANNEL_CAPACITY: usize = 64;

pub fn create_dispatch_channel() -> (TaskEntrySender, mpsc::Receiver<TaskEntry>) {
    mpsc::channel(CHANNEL_CAPACITY)
}

pub async fn run(
    mut rx: mpsc::Receiver<TaskEntry>,
    doc: Arc<DocHandle>,
    author: DocsAuthorId,
    shutdown: oneshot::Receiver<()>,
) {
    info!("dispatch_loop started");
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            entry = rx.recv() => {
                let Some(entry) = entry else { break };
                // Key prefix MUST match the worker scan in nexus-worker-core
                // (`get_many_by_prefix(b"task:")` + `strip_prefix("task:")`,
                // engine/runtime.rs). Sprint 71 Phase A (B-1) aligned this
                // writer onto the long-standing `task:` reader — before, the
                // `tasks/` prefix meant no dispatched task was ever claimed by
                // a real worker (the flow only ran in in-process tests).
                let key = format!("task:{}", entry.task.task_id);
                let value = match serde_json::to_vec(&entry) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(task_id = %entry.task.task_id, error = %e, "failed to serialize task entry");
                        continue;
                    }
                };
                if let Err(e) = doc.set(author, key.as_bytes().to_vec(), value).await {
                    warn!(task_id = %entry.task.task_id, error = %e, "failed to write task entry to project doc");
                }
            }
            _ = &mut shutdown => {
                info!("dispatch_loop received shutdown signal");
                break;
            }
        }
    }
    info!("dispatch_loop exiting");
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_lite::StreamExt;
    use nexus_core_rs::create_node;
    use nexus_core_rs::crypto::KeyPair;
    use nexus_core_rs::discovery::DiscoveryClient;
    use nexus_core_rs::docs::{DocsClient, DocsLiveEvent};
    use nexus_core_rs::task::{TASK_FORMAT_VERSION, Task};
    use std::time::Duration;

    fn make_test_entry() -> TaskEntry {
        let task = Task {
            version: TASK_FORMAT_VERSION,
            task_id: "test-dispatch-001".into(),
            task_type: "analysis".into(),
            prompt: "test prompt".into(),
            system_prompt: String::new(),
            model: "llama3".into(),
            priority: 5,
            created_at: 1714300000,
            parent_task_id: String::new(),
            metadata: std::collections::BTreeMap::new(),
            is_open_source: false,
            estimated_watts: 0,
            estimated_vram_mb: 0,
            estimated_hours: 0.0,
            redundancy_factor: 1,
            verifiable: false,
            watermark_seed: Vec::new(),
            required_runtime: None,
        };
        let kp = KeyPair::generate();
        TaskEntry::sign(task, &kp).expect("sign")
    }

    // P2-A-1 (S71->S73): spawns the dispatch loop concurrently with an
    // iroh-docs actor read. current_thread deadlocks under Windows
    // `cargo test` shared-process scheduling (tokio #7049); multi_thread
    // matches the prod runtime (worker binary, daemon). See PATTERNS §P54.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dispatch_loop_writes_to_doc() {
        let node = nexus_core_rs::create_node().await.expect("boot");
        let docs = nexus_core_rs::docs::DocsClient::new(node.docs());
        let doc = docs.create_doc().await.expect("create doc");
        let author = docs.author_default().await.expect("author");
        let doc = Arc::new(doc);

        let (tx, rx) = create_dispatch_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let doc_clone = Arc::clone(&doc);
        let handle = tokio::spawn(run(rx, doc_clone, author, shutdown_rx));

        let entry = make_test_entry();
        let task_id = entry.task.task_id.clone();
        tx.send(entry).await.expect("send");

        // P2-A-1 (S73 Phase B): `run`'s `select!` races `rx.recv()` against the
        // shutdown signal — on `current_thread` the buffered message happened to
        // win deterministically, but under `multi_thread` (the hang fix) the
        // shutdown can preempt it and drop the buffered task. Synchronise on the
        // observable write instead of assuming recv wins: wait for the task to
        // land, THEN signal shutdown.
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                let n = doc
                    .get_many_by_prefix(b"task:")
                    .await
                    .expect("get entries")
                    .len();
                if n == 1 {
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        })
        .await
        .expect("dispatcher should write the task within 10s");
        drop(tx);
        let _ = shutdown_tx.send(());
        handle.await.expect("dispatch loop joins");

        let entries = doc.get_many_by_prefix(b"task:").await.expect("get entries");
        assert_eq!(
            entries.len(),
            1,
            "dispatch loop must write exactly one entry"
        );

        let stored_key = std::str::from_utf8(entries[0].key()).unwrap();
        // B-1: the dispatched key MUST carry the `task:` prefix the worker
        // scans for, otherwise no dispatched task is ever claimed.
        assert!(
            stored_key.starts_with("task:"),
            "dispatched key must use the worker `task:` prefix, got {stored_key}"
        );
        assert_eq!(stored_key, format!("task:{task_id}"));
    }

    /// Sprint 71 Phase A (B-3): the first end-to-end test that wires the
    /// **production dispatch loop** to a **real worker engine**.
    ///
    /// Before the B-1 fix the dispatcher wrote `tasks/{id}` while the
    /// worker scanned the `task:` prefix, so no dispatched task was ever
    /// claimed — yet the worker's own test
    /// (`engine_claims_and_executes_tasks_on_registered_doc`) passed
    /// because it emulates the coordinator with a hand-written `task:`
    /// key. This test closes that blind spot: it writes the task through
    /// `dispatch_loop::run` and asserts a real engine claims and executes
    /// it. Execution uses the deterministic `StubBackend`, so the test is
    /// hermetic (no Ollama). Cross-machine/cross-node sync is S75.
    // P2-A-1 (S71->S73) MANDATORY: the canonical worker-pump E2E. It spawns
    // the engine pump (which polls the iroh-docs actor via
    // `get_many_by_prefix`) and waits on a real-time loop for `result:`.
    // current_thread deadlocks under Windows `cargo test` shared-process
    // teardown (tokio #7049); multi_thread matches prod and the only working
    // 2-node sync example (two_nodes_docs_sync.rs). The 10s timeout below is
    // defence-in-depth so a future regression fails fast instead of hanging
    // the whole nextest run. See PATTERNS §P54.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dispatched_task_is_claimed_and_executed_by_worker_engine() {
        use nexus_worker_core::allowlist::{Allowlist, NewProject};
        use nexus_worker_core::config::{Engine as EngineCfg, WorkerConfig};
        use nexus_worker_core::consent::{ConsentConfig, ConsentLevel};
        use nexus_worker_core::engine::{Engine, EngineBoot};
        use nexus_worker_core::llm::StubBackend;
        use std::time::Duration;

        // Worker engine: deterministic backend, fast poll, consent L4 so
        // the synthetic project id is admitted by the consent filter.
        let worker_config = WorkerConfig {
            engine: EngineCfg {
                task_poll_interval_ms: 100,
                max_concurrent_tasks: 1,
                state_flush_secs: 5,
            },
            ..WorkerConfig::default()
        };
        let allowlist = Allowlist::open_in_memory().expect("allowlist");
        allowlist
            .enroll(NewProject {
                id: "proj-dispatch-e2e".into(),
                name: "dispatch e2e".into(),
                enabled: true,
                budget_joules: 0,
                tasks_doc_ticket: None,
            })
            .expect("enroll");

        let sbfb_tmp = tempfile::tempdir().expect("tempdir");
        let mut consent = ConsentConfig::default_for("dispatch-e2e-worker");
        consent.level = ConsentLevel::All;
        consent
            .save_atomic(&sbfb_tmp.path().join("consent.json"))
            .expect("save consent");

        let boot = EngineBoot {
            worker_config,
            keypair: KeyPair::generate(),
            allowlist,
            data_dir: None,
            // Sprint 76 Phase E: a 5 ms inference delay gives the worker's
            // Instant-measured `generation_time_ms` a deterministic non-zero
            // floor, so the assertion below proves it is a real measurement
            // (not the pre-S76-E hardcoded 0).
            llm_override: Some(Box::new(StubBackend::new().with_delay_ms(5))),
            sbfb_home_override: Some(sbfb_tmp.path().to_path_buf()),
            rate_limit_policy_path_override: None,
        };
        let mut engine = Engine::new(boot).await.expect("engine boots");

        // Create a doc on the worker's own node, then write a task onto it
        // through the REAL dispatch loop (not a hand-written `doc.set`).
        let docs = engine.docs();
        let author = docs.author_create().await.expect("author");
        let doc = Arc::new(docs.create_doc().await.expect("create doc"));

        let (tx, rx) = create_dispatch_channel();
        let (d_stop_tx, d_stop_rx) = oneshot::channel::<()>();
        let dispatch = tokio::spawn(run(rx, Arc::clone(&doc), author, d_stop_rx));

        let entry = make_test_entry();
        tx.send(entry).await.expect("send task to dispatcher");
        // P2-A-1 (S73 Phase B): wait for the observable write before signalling
        // shutdown — `run`'s `select!` races recv against shutdown, and under
        // `multi_thread` shutdown can preempt the buffered task (current_thread
        // happened to be deterministic).
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if doc.get_many_by_prefix(b"task:").await.expect("tasks").len() == 1 {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .expect("dispatcher should write the task within 10s");
        drop(tx);
        let _ = d_stop_tx.send(());
        dispatch.await.expect("dispatch loop joins");

        // The dispatcher wrote exactly one entry under the `task:` prefix.
        let tasks = doc.get_many_by_prefix(b"task:").await.expect("tasks");
        assert_eq!(tasks.len(), 1, "dispatcher wrote one task: entry");

        // Hand the doc to the worker and run until it emits a result.
        engine.register_task_doc("proj-dispatch-e2e", (*doc).clone());
        let w_stop = engine.take_shutdown_sender().expect("shutdown sender");
        // P2-A-2 (Sprint 72 Phase B): capture an owned clone of the worker's
        // blob store *before* the engine is moved into the task below. The
        // clone shares the same content-addressed backend, so it still sees
        // the result blob the worker writes once it runs — letting us verify
        // the signature on the stored result, not just its presence.
        let blob_store = engine.blob_store();
        let worker = tokio::spawn(async move { engine.run_until_shutdown().await });

        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let results = doc.get_many_by_prefix(b"result:").await.expect("results");
                if !results.is_empty() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .expect("worker should claim+execute the dispatched task within 10s");

        let claims = doc.get_many_by_prefix(b"claim:").await.expect("claims");
        assert_eq!(claims.len(), 1, "worker claimed the dispatched task");
        let results = doc.get_many_by_prefix(b"result:").await.expect("results");
        assert_eq!(results.len(), 1, "worker produced exactly one result");

        // P2-A-2: the result must be an authentically signed `ResultEntry`,
        // not merely *a* blob under the `result:` prefix. Fetch the stored
        // bytes and verify the worker's Ed25519 signature over the canonical
        // payload — closing the S71 B-3 gap where the E2E only counted
        // results without proving authenticity.
        let blobs = nexus_core_rs::BlobsClient::new(&blob_store);
        let result_bytes = blobs
            .get_bytes(*results[0].content_hash().as_bytes())
            .await
            .expect("fetch result blob");
        let result: nexus_core_rs::ResultEntry =
            serde_json::from_slice(&result_bytes).expect("decode ResultEntry");
        result
            .verify_signature()
            .expect("worker result carries a valid Ed25519 signature");

        // Sprint 76 Phase E (D4-Q): the worker must stamp a REAL measured
        // inference duration, not the pre-S76-E hardcoded 0. With the 5 ms
        // stub delay the monotonic measurement is reliably >= 1 ms; a
        // regression to the hardcoded 0 would fail here. This is the field
        // the coordinator's kudos sanity-bound reads to clamp token claims.
        assert!(
            result.payload.generation_time_ms >= 1,
            "worker must stamp a real generation_time_ms (got {})",
            result.payload.generation_time_ms
        );
        assert!(
            result.payload.finished_at >= result.payload.started_at,
            "finished_at must not precede started_at"
        );

        let _ = w_stop.send(());
        worker.await.expect("worker joins").expect("worker ok");
    }

    // -----------------------------------------------------------------
    // Sprint 77 Phase A — WAN task delivery convergence (2-node)
    // -----------------------------------------------------------------

    /// Seed `node_b`'s address lookup with `node_a`'s current address so
    /// the dial resolves without depending on live pkarr DHT timing
    /// (mirrors the existing 2-node tests and `blobs.rs::fetch_ticket`).
    async fn seed_addr(node_a: &nexus_core_rs::Node, node_b: &nexus_core_rs::Node) {
        let a_addr = DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("node A publishes its address");
        node_b.memory_lookup().add_endpoint_info(a_addr);
    }

    /// Block until `doc` reports a gossip neighbor (`NeighborUp`) or the
    /// deadline elapses, so the write under test is a genuine *live
    /// incremental* delivery rather than the initial bulk sync.
    async fn await_neighbor(doc: &DocHandle, within: Duration) {
        let mut events = doc.subscribe().await.expect("subscribe for neighbor wait");
        tokio::time::timeout(within, async {
            while let Some(ev) = events.next().await {
                if matches!(ev, Ok(DocsLiveEvent::NeighborUp(_))) {
                    return;
                }
            }
        })
        .await
        .expect("a gossip neighbor must form within the deadline");
    }

    /// Poll `doc` until an entry under `prefix` whose key equals
    /// `exact_key` appears, or the deadline elapses.
    async fn await_exact_key(
        doc: &DocHandle,
        prefix: &[u8],
        exact_key: &[u8],
        within: Duration,
    ) -> bool {
        let deadline = std::time::Instant::now() + within;
        loop {
            let entries = doc.get_many_by_prefix(prefix).await.expect("scan prefix");
            if entries.iter().any(|e| e.key() == exact_key) {
                return true;
            }
            if std::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Convergence #1: a `task:` entry written by the REAL dispatch loop
    /// **after** a remote replica has imported and joined the doc must
    /// reach that replica via live incremental sync.
    ///
    /// This is the first cross-node test to exercise *incremental
    /// post-subscribe* delivery through `dispatch_loop::run` — every
    /// prior cross-node test wrote before the replica booted, covering
    /// only the initial bulk sync (the blind spot that hid the live
    /// `recv:0` blocker, `sprint76_verification.md` §5.1). In-process the
    /// gossip neighbor forms trivially, so this is the GREEN
    /// non-regression guard; the dropped-neighbor recovery (the actual
    /// prod fix) is proven red→green by
    /// `nexus_core_rs::doc_sync::tests::keepalive_rejoins_doc_after_neighbor_loss`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn convergence_incremental_task_reaches_remote_replica() {
        let node_a = create_node().await.expect("boot node A (coordinator)");
        let node_b = create_node().await.expect("boot node B (worker replica)");

        let docs_a = DocsClient::new(node_a.docs());
        let docs_b = DocsClient::new(node_b.docs());
        let author_a = docs_a.author_default().await.expect("author A");
        let doc_a = docs_a.create_doc().await.expect("create project doc on A");

        seed_addr(&node_a, &node_b).await;

        // B imports A's write ticket → start_sync → gossip neighbor forms.
        let ticket = doc_a.share_write().await.expect("share write ticket");
        let doc_b = docs_b.import_ticket(ticket).await.expect("B imports doc");

        // Ensure the neighbor is up so the dispatch write is live, not bulk.
        await_neighbor(&doc_b, Duration::from_secs(20)).await;

        // A writes a task through the PRODUCTION dispatch loop AFTER B joined.
        let doc_a = Arc::new(doc_a);
        let (tx, rx) = create_dispatch_channel();
        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        let dispatch = tokio::spawn(run(rx, Arc::clone(&doc_a), author_a, stop_rx));

        let entry = make_test_entry();
        let task_id = entry.task.task_id.clone();
        tx.send(entry).await.expect("queue task to dispatcher");

        let want_key = format!("task:{task_id}").into_bytes();
        assert!(
            await_exact_key(&doc_b, b"task:", &want_key, Duration::from_secs(15)).await,
            "an incremental task: entry written post-subscribe must reach the remote replica"
        );

        drop(tx);
        let _ = stop_tx.send(());
        dispatch.await.expect("dispatch loop joins");
        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    /// Convergence #2 (non-regression): a `task:` entry written **before**
    /// the replica imports must still reach it via the initial bulk sync.
    /// Guards against a keepalive/sync change breaking boot catch-up.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn convergence_boot_catchup_still_works() {
        let node_a = create_node().await.expect("boot node A (coordinator)");
        let node_b = create_node().await.expect("boot node B (worker replica)");

        let docs_a = DocsClient::new(node_a.docs());
        let docs_b = DocsClient::new(node_b.docs());
        let author_a = docs_a.author_default().await.expect("author A");
        let doc_a = docs_a.create_doc().await.expect("create project doc on A");

        seed_addr(&node_a, &node_b).await;

        // Write the task BEFORE B imports — only the bulk catch-up can find it.
        let entry = make_test_entry();
        let task_id = entry.task.task_id.clone();
        let value = serde_json::to_vec(&entry).expect("serialize entry");
        doc_a
            .set(author_a, format!("task:{task_id}").into_bytes(), value)
            .await
            .expect("A writes task before share");

        let ticket = doc_a.share_write().await.expect("share write ticket");
        let doc_b = docs_b.import_ticket(ticket).await.expect("B imports doc");

        let want_key = format!("task:{task_id}").into_bytes();
        assert!(
            await_exact_key(&doc_b, b"task:", &want_key, Duration::from_secs(15)).await,
            "a task: written before import must still reach the replica via bulk catch-up"
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    /// Convergence #3 (symmetry): a `result:` entry written by the worker
    /// replica must reach the coordinator's subscriber — the inverse
    /// direction of the dispatch path, the leg `result_sync` relies on.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn convergence_remote_write_visible_to_local_subscriber() {
        let node_a = create_node().await.expect("boot node A (coordinator)");
        let node_b = create_node().await.expect("boot node B (worker replica)");

        let docs_a = DocsClient::new(node_a.docs());
        let docs_b = DocsClient::new(node_b.docs());
        let doc_a = docs_a.create_doc().await.expect("create project doc on A");
        let author_b = docs_b.author_default().await.expect("author B");

        seed_addr(&node_a, &node_b).await;

        let ticket = doc_a.share_write().await.expect("share write ticket");
        let doc_b = docs_b.import_ticket(ticket).await.expect("B imports doc");

        // Wait until A has formed a neighbor so the result: write is live.
        await_neighbor(&doc_a, Duration::from_secs(20)).await;

        // The worker writes a result: entry; the coordinator must observe it.
        doc_b
            .set(author_b, b"result:rid-conv".to_vec(), b"{}".to_vec())
            .await
            .expect("B writes result entry");

        assert!(
            await_exact_key(
                &doc_a,
                b"result:",
                b"result:rid-conv",
                Duration::from_secs(15)
            )
            .await,
            "a result: entry written by the worker must reach the coordinator subscriber"
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    /// Boot a persistent coordinator node on `data_dir` with a fixed
    /// secret key, so a test can shut it down and boot a successor on
    /// the SAME store + identity (the restart / hot-binary-swap mode
    /// of `sprint76_verification.md` §5.1).
    async fn boot_persistent_coordinator(
        data_dir: &std::path::Path,
        secret: [u8; 32],
    ) -> nexus_core_rs::Node {
        nexus_core_rs::create_node_with_config(
            nexus_core_rs::NodeConfig::default()
                .with_secret_key(secret)
                .with_data_dir(data_dir),
        )
        .await
        .expect("boot persistent coordinator node")
    }

    /// Publish `node`'s current address into `other`'s memory lookup
    /// AND return it, so a test can hand it to a worker-side
    /// `start_sync` exactly like the S77 keepalive does with the
    /// ticket peers.
    async fn addr_of(node: &nexus_core_rs::Node) -> iroh::EndpointAddr {
        DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .expect("node publishes its address")
    }

    /// Convergence #4 (Sprint 81 Phase A4 — CONTROL, pins the hole):
    /// a coordinator that REOPENS its persisted project doc after a
    /// restart via `open_doc` alone sits OUTSIDE the doc's sync-set
    /// (iroh-docs 0.101, recalibrated at the S81 Phase B/C bump —
    /// mechanism unchanged from 0.98: only `start_sync` inserts into
    /// `SyncState`, `engine/live.rs:408-414`).
    /// Consequences pinned here, both observed LIVE on the anchor in
    /// the S81 Phase A3 baseline: (a) its incremental `task:` write is
    /// never gossip-broadcast, and (b) the worker's keepalive re-dial
    /// (`start_sync(peers)` from the worker side — the S77 fix) is
    /// REJECTED (`AbortReason::NotFound`), so the keepalive CANNOT
    /// compensate. If this control starts CONVERGING after an
    /// iroh-docs bump, the upstream sync-set behaviour changed and the
    /// A4 boot fix premise must be recalibrated (tripwire re-verified
    /// non-convergent under 0.101 at Phase B).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn reopened_project_doc_without_start_sync_does_not_deliver() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let store = dir.path().join("coordinator");
        let secret = [7u8; 32];

        // Boot #1: create the doc, enroll B, prove baseline delivery.
        let node_a1 = boot_persistent_coordinator(&store, secret).await;
        let node_b = create_node().await.expect("boot node B (worker replica)");
        let docs_a1 = DocsClient::new(node_a1.docs());
        let docs_b = DocsClient::new(node_b.docs());
        let author_a1 = docs_a1.author_default().await.expect("author A1");
        let doc_a1 = docs_a1.create_doc().await.expect("create project doc");
        let doc_id = doc_a1.id();

        seed_addr(&node_a1, &node_b).await;
        let ticket = doc_a1.share_write().await.expect("share write ticket");
        let doc_b = docs_b.import_ticket(ticket).await.expect("B imports doc");
        await_neighbor(&doc_b, Duration::from_secs(20)).await;

        doc_a1
            .set(author_a1, b"task:restart-base".to_vec(), b"{}".to_vec())
            .await
            .expect("baseline write");
        assert!(
            await_exact_key(
                &doc_b,
                b"task:",
                b"task:restart-base",
                Duration::from_secs(15)
            )
            .await,
            "pre-restart baseline write must converge (share_write armed the sync-set)"
        );

        // Restart: same store, same identity, reopen WITHOUT start_sync.
        node_a1.shutdown().await.expect("A1 shuts down");
        let node_a2 = boot_persistent_coordinator(&store, secret).await;
        let docs_a2 = DocsClient::new(node_a2.docs());
        let author_a2 = docs_a2
            .author_default()
            .await
            .expect("author survives reopen");
        let doc_a2 = docs_a2
            .open_doc(doc_id)
            .await
            .expect("open persisted doc")
            .expect("doc survives the restart");

        // The worker keepalive re-dials the rebooted coordinator — and
        // must be rejected, because A2 never entered its sync-set.
        let a2_addr = addr_of(&node_a2).await;
        node_b.memory_lookup().add_endpoint_info(a2_addr.clone());
        doc_b
            .start_sync(vec![a2_addr])
            .await
            .expect("worker-side start_sync call itself succeeds (rejection is remote)");

        doc_a2
            .set(author_a2, b"task:restart-control".to_vec(), b"{}".to_vec())
            .await
            .expect("post-restart write");
        assert!(
            !await_exact_key(
                &doc_b,
                b"task:",
                b"task:restart-control",
                Duration::from_secs(8)
            )
            .await,
            "an incremental write from a reopened-but-never-started doc must NOT reach the \
             replica, even with the worker keepalive re-dialing — if this starts converging \
             the upstream sync-set behaviour changed: recalibrate the A4 boot fix"
        );

        node_a2.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    /// Convergence #5 (Sprint 81 Phase A4 — the boot fix, red→green
    /// against #4): same restart scenario, but the successor reopens
    /// through `open_project_doc_for_dispatch` (the PRODUCTION boot
    /// path) which enters the sync-set at boot. The worker keepalive
    /// re-dial is now ACCEPTED and the post-restart incremental write
    /// converges WITHOUT any invite mint or submit-path share_write
    /// side-effect (the fragile dependency the A3 baseline exposed).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_path_reenters_sync_set_and_delivers_after_reopen() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let store = dir.path().join("coordinator");
        let secret = [9u8; 32];

        // Boot #1: create the doc, enroll B, prove baseline delivery.
        let node_a1 = boot_persistent_coordinator(&store, secret).await;
        let node_b = create_node().await.expect("boot node B (worker replica)");
        let docs_a1 = DocsClient::new(node_a1.docs());
        let docs_b = DocsClient::new(node_b.docs());
        let author_a1 = docs_a1.author_default().await.expect("author A1");
        let doc_a1 = docs_a1.create_doc().await.expect("create project doc");
        let doc_id = doc_a1.id();

        seed_addr(&node_a1, &node_b).await;
        let ticket = doc_a1.share_write().await.expect("share write ticket");
        let doc_b = docs_b.import_ticket(ticket).await.expect("B imports doc");
        await_neighbor(&doc_b, Duration::from_secs(20)).await;

        doc_a1
            .set(author_a1, b"task:restart-base".to_vec(), b"{}".to_vec())
            .await
            .expect("baseline write");
        assert!(
            await_exact_key(
                &doc_b,
                b"task:",
                b"task:restart-base",
                Duration::from_secs(15)
            )
            .await,
            "pre-restart baseline write must converge"
        );

        // Restart: same store, same identity — reopen through the
        // production boot path (open/create + start_sync).
        node_a1.shutdown().await.expect("A1 shuts down");
        let node_a2 = boot_persistent_coordinator(&store, secret).await;
        // Seed B's address BEFORE the boot path runs: start_sync
        // re-dials the peers persisted in docs.redb, and in-process
        // there is no pkarr to resolve them (prod resolves via
        // discovery).
        seed_addr(&node_b, &node_a2).await;
        let docs_a2 = DocsClient::new(node_a2.docs());
        let author_a2 = docs_a2
            .author_default()
            .await
            .expect("author survives reopen");
        let doc_a2 = crate::runtime::open_project_doc_for_dispatch(
            &docs_a2,
            nexus_core_rs::IdentityMode::Normal,
        )
        .await
        .expect("production boot path opens + enters the sync-set");
        assert_eq!(
            doc_a2.id(),
            doc_id,
            "boot path must reopen the SAME persisted doc, never create a fresh one"
        );

        // The worker keepalive re-dials the rebooted coordinator — now
        // ACCEPTED because the boot path entered the sync-set.
        let a2_addr = addr_of(&node_a2).await;
        node_b.memory_lookup().add_endpoint_info(a2_addr.clone());
        doc_b
            .start_sync(vec![a2_addr])
            .await
            .expect("worker-side keepalive re-dial");

        doc_a2
            .set(author_a2, b"task:restart-fixed".to_vec(), b"{}".to_vec())
            .await
            .expect("post-restart write");
        assert!(
            await_exact_key(
                &doc_b,
                b"task:",
                b"task:restart-fixed",
                Duration::from_secs(20)
            )
            .await,
            "with the A4 boot fix the post-restart incremental write must reach the replica \
             (worker keepalive accepted, no share_write side-effect needed)"
        );

        node_a2.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    // -----------------------------------------------------------------
    // Sprint 81 Phase C — sibling sync-set fix (P2-SIBLING-SYNC-SET)
    // -----------------------------------------------------------------

    /// Which sibling namespace a reopen scenario boots.
    enum SiblingKind {
        Storage,
        Feed,
    }

    /// Which reopen path the scenario exercises after the restart.
    enum SiblingReopen {
        /// The production boot fn in Normal mode — the Phase C fix
        /// (`start_sync(vec![])` chokepoint, all arms).
        BootFnNormal,
        /// The production boot fn under Duress — must SKIP the
        /// sync-set entry (DURESS-BOOT-LEAK §15.1: real store, decoy
        /// key; no dial, no serve).
        BootFnDuress,
        /// Raw `open_doc`, mimicking the pre-C ticket-persisted reopen
        /// arm (CONTROL — and upstream tripwire: only `start_sync`
        /// inserts into `SyncState` under iroh-docs 0.101; if this
        /// path starts converging, recalibrate the sibling fix
        /// premise, same discipline as convergence #4).
        OpenDocDirect,
    }

    /// Shared 2-node harness for the six sibling scenarios: first-boot
    /// the namespace through the PRODUCTION boot fn (creates + persists
    /// the M8 row, `share_write` arms the sync-set), enroll a remote
    /// replica, prove baseline convergence, then restart the
    /// coordinator on the SAME store + identity and reopen through
    /// `reopen`. Returns whether a post-restart incremental write
    /// reached the replica.
    async fn run_sibling_reopen_scenario(kind: SiblingKind, reopen: SiblingReopen) -> bool {
        let dir = tempfile::tempdir().expect("tmpdir");
        let store = dir.path().join("coordinator");
        let secret_base = match kind {
            SiblingKind::Storage => 20u8,
            SiblingKind::Feed => 30u8,
        };
        let secret = [secret_base
            + match reopen {
                SiblingReopen::BootFnNormal => 1,
                SiblingReopen::BootFnDuress => 2,
                SiblingReopen::OpenDocDirect => 3,
            }; 32];
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().expect("in-memory DB"),
        ));

        // Boot #1: first-boot arm — create, persist M8 (Some(ticket)),
        // share_write side-effect arms the sync-set.
        let node_a1 = boot_persistent_coordinator(&store, secret).await;
        let node_b = create_node().await.expect("boot node B (remote replica)");
        let docs_a1 = DocsClient::new(node_a1.docs());
        let docs_b = DocsClient::new(node_b.docs());
        let author_a1 = docs_a1.author_default().await.expect("author A1");

        let (doc_a1, ticket_str) = match kind {
            SiblingKind::Storage => {
                let st = crate::runtime::boot_storage_namespace(
                    &docs_a1,
                    &db,
                    "sbfb-ideas",
                    author_a1,
                    nexus_core_rs::IdentityMode::Normal,
                    None,
                )
                .await
                .expect("first-boot storage namespace");
                (st.doc.clone(), st.ticket.clone())
            }
            SiblingKind::Feed => {
                let fs = crate::runtime::boot_feed_namespace(
                    &docs_a1,
                    &db,
                    author_a1,
                    nexus_core_rs::IdentityMode::Normal,
                    None,
                )
                .await
                .expect("first-boot feed namespace");
                (fs.doc.clone(), fs.ticket.clone())
            }
        };
        let ns_id = doc_a1.id();

        seed_addr(&node_a1, &node_b).await;
        let ticket: nexus_core_rs::docs::DocsTicket = ticket_str
            .parse()
            .expect("the persisted state ticket string parses");
        let doc_b = docs_b.import_ticket(ticket).await.expect("B imports doc");
        // Positive deadlines below are generous: these six scenarios run
        // CONCURRENTLY with the rest of the workspace suite (real QUIC
        // 2-node convergence is CPU-bound — observed green solo in ~2s
        // but past 30s under the initial spike of a full parallel run).
        // `await_exact_key`/`await_neighbor` return as soon as the event
        // lands, so slack costs nothing; the two-node-convergence nextest
        // group carries a matching wider slow-timeout.
        await_neighbor(&doc_b, Duration::from_secs(60)).await;

        doc_a1
            .set(author_a1, b"kv:base".to_vec(), b"{}".to_vec())
            .await
            .expect("baseline write");
        assert!(
            await_exact_key(&doc_b, b"kv:", b"kv:base", Duration::from_secs(30)).await,
            "pre-restart baseline write must converge (share_write armed the sync-set)"
        );

        // Restart: same store + identity; the M8 row now carries
        // Some(ticket), so the boot fn takes the ticket-persisted
        // reopen arm — the arm the Phase C fix targets.
        node_a1.shutdown().await.expect("A1 shuts down");
        let node_a2 = boot_persistent_coordinator(&store, secret).await;
        // Seed B's address BEFORE the reopen path runs: start_sync
        // re-dials the peers persisted in docs.redb, and in-process
        // there is no pkarr to resolve them.
        seed_addr(&node_b, &node_a2).await;
        let docs_a2 = DocsClient::new(node_a2.docs());
        let author_a2 = docs_a2
            .author_default()
            .await
            .expect("author survives reopen");

        let doc_a2 = match reopen {
            SiblingReopen::BootFnNormal | SiblingReopen::BootFnDuress => {
                let mode = match reopen {
                    SiblingReopen::BootFnDuress => nexus_core_rs::IdentityMode::Duress,
                    _ => nexus_core_rs::IdentityMode::Normal,
                };
                match kind {
                    SiblingKind::Storage => crate::runtime::boot_storage_namespace(
                        &docs_a2,
                        &db,
                        "sbfb-ideas",
                        author_a2,
                        mode,
                        None,
                    )
                    .await
                    .expect("reopen through the production boot fn")
                    .doc
                    .clone(),
                    SiblingKind::Feed => {
                        crate::runtime::boot_feed_namespace(&docs_a2, &db, author_a2, mode, None)
                            .await
                            .expect("reopen through the production boot fn")
                            .doc
                            .clone()
                    }
                }
            }
            SiblingReopen::OpenDocDirect => Arc::new(
                docs_a2
                    .open_doc(ns_id)
                    .await
                    .expect("open persisted doc")
                    .expect("doc survives the restart"),
            ),
        };
        assert_eq!(
            doc_a2.id(),
            ns_id,
            "reopen must yield the SAME persisted namespace, never a fresh one"
        );

        // Worker-style keepalive re-dial (S77) — accepted only if the
        // reopened coordinator entered its sync-set. The PRODUCTION
        // keepalive re-issues start_sync until the neighbor holds
        // (`spawn_doc_sync_keepalive`); a single dial can be lost under
        // the parallel workspace load spike, so the harness re-dials
        // periodically like prod does. This does not weaken the negative
        // paths: a doc outside its sync-set rejects EVERY re-dial
        // (`AbortReason::NotFound`), which is exactly what they pin.
        let a2_addr = addr_of(&node_a2).await;
        node_b.memory_lookup().add_endpoint_info(a2_addr.clone());

        doc_a2
            .set(author_a2, b"kv:after-restart".to_vec(), b"{}".to_vec())
            .await
            .expect("post-restart write");
        let deadline = std::time::Instant::now()
            + match reopen {
                // Convergence expected: generous deadline (see the load
                // note above — single-dial 20s/45s runs each flaked once
                // under the full parallel workspace).
                SiblingReopen::BootFnNormal => Duration::from_secs(45),
                // Negative assertion: bounded wait, mirrors convergence
                // #4. The CONTROL's real safety is the categorical
                // reject (`AbortReason::NotFound` on every re-dial,
                // state.rs:96-97), not this timing. Known residual for
                // the DURESS path (review P2-B, carry K): a future
                // miswire that re-enters the sync-set under duress and
                // converges in the 8-45s load window would slip past
                // this bound (it is still caught on any idle/solo run,
                // where convergence takes ~2s).
                _ => Duration::from_secs(8),
            };
        let converged = loop {
            doc_b
                .start_sync(vec![a2_addr.clone()])
                .await
                .expect("worker-side start_sync call itself succeeds (rejection is remote)");
            if await_exact_key(&doc_b, b"kv:", b"kv:after-restart", Duration::from_secs(5)).await {
                break true;
            }
            if std::time::Instant::now() >= deadline {
                break false;
            }
        };

        node_a2.shutdown().await.ok();
        node_b.shutdown().await.ok();
        converged
    }

    /// CONTROL (pins the pre-C hole on the STORAGE sibling): a
    /// ticket-persisted storage namespace reopened via `open_doc`
    /// alone — what `boot_storage_namespace`'s reopen arm did before
    /// Phase C — sits outside the sync-set: broadcasts suppressed,
    /// worker keepalive rejected.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn storage_namespace_reopen_without_start_sync_does_not_deliver() {
        assert!(
            !run_sibling_reopen_scenario(SiblingKind::Storage, SiblingReopen::OpenDocDirect).await,
            "a reopened-but-never-started storage namespace must NOT deliver — if this \
             converges the upstream sync-set behaviour changed: recalibrate the sibling fix"
        );
    }

    /// GREEN (the Phase C fix, red→green against the CONTROL above):
    /// reopening through `boot_storage_namespace` enters the sync-set
    /// at boot, so the post-restart incremental write converges and
    /// the worker keepalive re-dial is accepted.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_storage_namespace_reenters_sync_set_and_delivers_after_reopen() {
        assert!(
            run_sibling_reopen_scenario(SiblingKind::Storage, SiblingReopen::BootFnNormal).await,
            "with the Phase C sibling fix the reopened storage namespace must re-enter \
             its sync-set at boot and deliver incremental writes"
        );
    }

    /// Duress no-op (DURESS-BOOT-LEAK §15.1): under duress the store
    /// is the REAL one and only the node keypair is a decoy — the
    /// boot fn must SKIP the sync-set entry: no persisted-peer
    /// re-dial, no serving the real replica, worker sync rejected.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_storage_namespace_duress_skips_sync_set_entry() {
        assert!(
            !run_sibling_reopen_scenario(SiblingKind::Storage, SiblingReopen::BootFnDuress).await,
            "under duress the storage namespace must stay OUT of the sync-set (0 dial, \
             0 serve) — convergence here means the duress gate regressed"
        );
    }

    /// CONTROL — feed sibling (mirror of the storage CONTROL). The
    /// feed doc is network-visible, so this hole had real reach.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn feed_namespace_reopen_without_start_sync_does_not_deliver() {
        assert!(
            !run_sibling_reopen_scenario(SiblingKind::Feed, SiblingReopen::OpenDocDirect).await,
            "a reopened-but-never-started feed namespace must NOT deliver — if this \
             converges the upstream sync-set behaviour changed: recalibrate the sibling fix"
        );
    }

    /// GREEN — feed sibling (mirror of the storage GREEN).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_feed_namespace_reenters_sync_set_and_delivers_after_reopen() {
        assert!(
            run_sibling_reopen_scenario(SiblingKind::Feed, SiblingReopen::BootFnNormal).await,
            "with the Phase C sibling fix the reopened feed namespace must re-enter its \
             sync-set at boot and deliver incremental writes"
        );
    }

    /// Duress no-op — feed sibling (mirror of the storage duress
    /// no-op).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn boot_feed_namespace_duress_skips_sync_set_entry() {
        assert!(
            !run_sibling_reopen_scenario(SiblingKind::Feed, SiblingReopen::BootFnDuress).await,
            "under duress the feed namespace must stay OUT of the sync-set (0 dial, \
             0 serve) — convergence here means the duress gate regressed"
        );
    }

    /// Duress no-op — PROJECT doc (closes review P2-A): the A4 boot
    /// path under duress must reopen the SAME persisted doc but SKIP
    /// the sync-set entry. The project doc is the most sensitive one
    /// (task/result history) and the preflight §5.2 named its A4-era
    /// unconditional `start_sync` as a probably-shipped leak; this is
    /// the integration tripwire for that branch (the unit test only
    /// covers the pure predicate). Same 8s negative bound as the
    /// sibling duress tests (review P2-B residual, carry K).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn open_project_doc_for_dispatch_duress_skips_sync_set() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let store = dir.path().join("coordinator");
        let secret = [13u8; 32];

        // Boot #1: create the doc, enroll B, prove baseline delivery.
        let node_a1 = boot_persistent_coordinator(&store, secret).await;
        let node_b = create_node().await.expect("boot node B (worker replica)");
        let docs_a1 = DocsClient::new(node_a1.docs());
        let docs_b = DocsClient::new(node_b.docs());
        let author_a1 = docs_a1.author_default().await.expect("author A1");
        let doc_a1 = docs_a1.create_doc().await.expect("create project doc");
        let doc_id = doc_a1.id();

        seed_addr(&node_a1, &node_b).await;
        let ticket = doc_a1.share_write().await.expect("share write ticket");
        let doc_b = docs_b.import_ticket(ticket).await.expect("B imports doc");
        await_neighbor(&doc_b, Duration::from_secs(60)).await;

        doc_a1
            .set(author_a1, b"task:duress-base".to_vec(), b"{}".to_vec())
            .await
            .expect("baseline write");
        assert!(
            await_exact_key(
                &doc_b,
                b"task:",
                b"task:duress-base",
                Duration::from_secs(30)
            )
            .await,
            "pre-restart baseline write must converge"
        );

        // Restart under DURESS through the production boot path.
        node_a1.shutdown().await.expect("A1 shuts down");
        let node_a2 = boot_persistent_coordinator(&store, secret).await;
        seed_addr(&node_b, &node_a2).await;
        let docs_a2 = DocsClient::new(node_a2.docs());
        let author_a2 = docs_a2
            .author_default()
            .await
            .expect("author survives reopen");
        let doc_a2 = crate::runtime::open_project_doc_for_dispatch(
            &docs_a2,
            nexus_core_rs::IdentityMode::Duress,
        )
        .await
        .expect("duress boot path opens the doc without entering the sync-set");
        assert_eq!(
            doc_a2.id(),
            doc_id,
            "duress must reopen the SAME persisted doc, never create a fresh one"
        );

        // Worker keepalive re-dials — must stay rejected; the
        // post-restart write must NOT reach the replica.
        let a2_addr = addr_of(&node_a2).await;
        node_b.memory_lookup().add_endpoint_info(a2_addr.clone());
        doc_a2
            .set(author_a2, b"task:duress-control".to_vec(), b"{}".to_vec())
            .await
            .expect("post-restart write");
        let deadline = std::time::Instant::now() + Duration::from_secs(8);
        let converged = loop {
            doc_b
                .start_sync(vec![a2_addr.clone()])
                .await
                .expect("worker-side start_sync call itself succeeds (rejection is remote)");
            if await_exact_key(
                &doc_b,
                b"task:",
                b"task:duress-control",
                Duration::from_secs(5),
            )
            .await
            {
                break true;
            }
            if std::time::Instant::now() >= deadline {
                break false;
            }
        };
        assert!(
            !converged,
            "under duress the project doc must stay OUT of the sync-set — convergence \
             here means the duress gate regressed (DURESS-BOOT-LEAK §15.1)"
        );

        node_a2.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }
}
