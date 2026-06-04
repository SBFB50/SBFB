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
    use nexus_core_rs::crypto::KeyPair;
    use nexus_core_rs::task::{TASK_FORMAT_VERSION, Task};

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
            llm_override: Some(Box::new(StubBackend::new())),
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

        let _ = w_stop.send(());
        worker.await.expect("worker joins").expect("worker ok");
    }
}
