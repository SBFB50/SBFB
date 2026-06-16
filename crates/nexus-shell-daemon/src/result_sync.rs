// SPDX-License-Identifier: AGPL-3.0-or-later
//! Result-sync bridge — forwards worker-written `result:` entries on
//! the project doc into the validator loop.
//!
//! ## Why this exists (2026-06-05 platform remediation, hotfix #5)
//!
//! The dispatch loop ([`crate::dispatch_loop`]) writes `task:{id}` to
//! the project doc; a worker (the co-located on-demand worker, or any
//! remote worker that joined via an invite) claims it, runs inference,
//! and writes `result:{id}` back onto the same iroh-docs namespace.
//! iroh-docs replicates that entry to the coordinator's node.
//!
//! Before this loop existed nothing on the coordinator read those
//! replicated `result:` entries. The validator loop
//! ([`crate::validator_loop`]) drains a broadcast channel of
//! [`ResultEvent`]s, but the channel's only producer was the
//! synchronous `POST /api/v1/results/submit` HTTP handler. A worker
//! that wrote its result to the doc therefore never reached the
//! coordinator DB, so `GET /api/v1/tasks/{id}/result` 404'd until the
//! network execute arm gave up at its global timeout.
//!
//! This loop is the missing producer: it observes `result:` entries
//! arriving on the project doc (live via `InsertRemote`, plus a boot
//! catch-up scan for entries that landed while the daemon was down),
//! decodes the [`ResultEntry`], and forwards it as a
//! [`ResultEvent::NewResult`]. **All verification stays in the
//! validator loop** — signature check, output guardrail
//! (guardrail-before-persist, Sprint 73 Phase A D5), DB persist and
//! kudos credit. This module is intentionally a thin, single-purpose
//! bridge so the validator loop remains the one place a result can be
//! accepted.
//!
//! The structure mirrors [`crate::feed_sync::spawn_feed_subscribe`]:
//! a reconnecting subscribe loop with backoff, a shutdown watch, and a
//! blob fetch that retries because iroh-docs syncs entry metadata
//! before the content blob is available locally.

use std::collections::HashSet;
use std::sync::Arc;

use futures_lite::StreamExt;
use tracing::{debug, info, warn};

use nexus_core_rs::BlobsClient;
use nexus_core_rs::docs::{DocHandle, DocsEntry, DocsLiveEvent};
use nexus_core_rs::task::ResultEntry;

use crate::validator_loop::{ResultEvent, ResultEventSender};

const RESULT_PREFIX: &[u8] = b"result:";

// ---------------------------------------------------------------------------
// Forward a single `result:` doc entry into the validator loop
// ---------------------------------------------------------------------------

/// Decode the [`ResultEntry`] behind a `result:` doc entry and forward
/// it to the validator loop. No-op for any other key prefix.
///
/// `seen` deduplicates by `task_id`: the validator loop is already
/// idempotent (a second result for a completed task is rejected with
/// no double-credit), but `InsertRemote` can refire for the same key
/// and the boot catch-up overlaps the live stream, so skipping avoids
/// channel and log noise. A send failure un-marks the id so a later
/// retry can re-forward.
async fn forward_result_entry(
    doc_entry: &DocsEntry,
    node: &nexus_core_rs::Node,
    tx: &ResultEventSender,
    seen: &mut HashSet<String>,
) {
    let key_str = String::from_utf8_lossy(doc_entry.key());
    if !key_str.as_bytes().starts_with(RESULT_PREFIX) {
        return;
    }

    // Blob content may not be downloaded yet when InsertRemote fires
    // (iroh-docs syncs metadata before content). Retry with backoff —
    // identical shape to feed_sync::ingest_doc_entry.
    let blobs = BlobsClient::new(node.blobs_store());
    let hash_bytes = *doc_entry.content_hash().as_bytes();
    let content = {
        let mut backoff = std::time::Duration::from_millis(50);
        let max_backoff = std::time::Duration::from_secs(2);
        loop {
            match blobs.get_bytes(hash_bytes).await {
                Ok(b) => break b,
                Err(e) => {
                    if backoff > max_backoff {
                        warn!(key = %key_str, error = %e, "result blob unavailable after retries");
                        return;
                    }
                    debug!(
                        key = %key_str,
                        wait_ms = backoff.as_millis(),
                        "result blob not ready, retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff = backoff.saturating_mul(3);
                }
            }
        }
    };

    let entry: ResultEntry = match serde_json::from_slice(&content) {
        Ok(e) => e,
        Err(e) => {
            warn!(key = %key_str, error = %e, "invalid result entry JSON, skipping");
            return;
        }
    };

    let task_id = entry.payload.task_id.clone();
    if !seen.insert(task_id.clone()) {
        debug!(task_id = %task_id, "result already forwarded, skipping");
        return;
    }

    match tx.send(ResultEvent::NewResult(entry)) {
        Ok(_) => info!(task_id = %task_id, "forwarded worker result to validator loop"),
        Err(_) => {
            // No active receiver — the validator loop is gone. Un-mark
            // so a future entry for this task can retry.
            seen.remove(&task_id);
            warn!(
                task_id = %task_id,
                "validator loop receiver dropped; result not forwarded"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Subscribe — forward remote `result:` entries into the validator loop
// ---------------------------------------------------------------------------

/// Spawn the result-sync loop on the project `doc`.
///
/// Runs a boot catch-up scan (forwarding `result:` entries already on
/// the doc) and then a reconnecting `subscribe()` loop that forwards
/// each `InsertRemote` `result:` entry. Returns the join handle so the
/// runtime can await a clean shutdown.
pub fn spawn_result_subscribe(
    doc: Arc<DocHandle>,
    node: Arc<nexus_core_rs::Node>,
    tx: ResultEventSender,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut seen: HashSet<String> = HashSet::new();
        let mut shutdown = shutdown;

        // Boot catch-up: a worker may have written a result while the
        // daemon was down (the result is durable on the doc). Forward
        // anything already present before the live stream starts.
        match doc.get_many_by_prefix(RESULT_PREFIX).await {
            Ok(entries) => {
                for e in &entries {
                    forward_result_entry(e, &node, &tx, &mut seen).await;
                }
            }
            Err(e) => warn!(error = %e, "result catch-up scan failed"),
        }

        let mut backoff = std::time::Duration::from_millis(500);
        let max_backoff = std::time::Duration::from_secs(30);

        loop {
            let stream =
                match tokio::time::timeout(std::time::Duration::from_secs(30), doc.subscribe())
                    .await
                {
                    Ok(Ok(s)) => s,
                    Ok(Err(e)) => {
                        warn!(error = %e, "result subscribe failed, retrying");
                        tokio::select! {
                            _ = tokio::time::sleep(backoff) => {}
                            _ = shutdown.changed() => { return; }
                        }
                        backoff = (backoff * 2).min(max_backoff);
                        continue;
                    }
                    Err(_) => {
                        warn!("result subscribe timed out (30s), retrying");
                        tokio::select! {
                            _ = tokio::time::sleep(backoff) => {}
                            _ = shutdown.changed() => { return; }
                        }
                        backoff = (backoff * 2).min(max_backoff);
                        continue;
                    }
                };

            info!("result subscribe active");
            backoff = std::time::Duration::from_millis(500);
            let mut stream = stream;

            loop {
                tokio::select! {
                    event = stream.next() => {
                        match event {
                            // A worker is always a separate node, so its
                            // `result:` writes arrive as InsertRemote. The
                            // coordinator itself only writes `task:`, never
                            // `result:`, so there is no local-write path to
                            // handle here.
                            Some(Ok(DocsLiveEvent::InsertRemote { entry, .. })) => {
                                forward_result_entry(&entry, &node, &tx, &mut seen).await;
                            }
                            Some(Ok(_)) => {}
                            Some(Err(e)) => {
                                warn!(error = %e, "result subscribe stream error, reconnecting");
                                break;
                            }
                            None => {
                                info!("result subscribe stream ended, reconnecting");
                                break;
                            }
                        }
                    }
                    _ = shutdown.changed() => {
                        info!("result subscribe shutting down");
                        return;
                    }
                }
            }

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
    use nexus_coordinator_rs::db::CoordinatorDb;
    use nexus_coordinator_rs::types::{TaskRecord, TaskStatus};
    use nexus_core_rs::crypto::KeyPair;
    use nexus_core_rs::task::{ResultEntry, ResultPayload, TASK_FORMAT_VERSION};
    use std::sync::Mutex;
    use std::time::Duration;

    fn pending_task_record(task_id: &str, project_id: &str) -> TaskRecord {
        TaskRecord {
            task_id: task_id.to_string(),
            status: TaskStatus::Pending,
            project_id: project_id.to_string(),
            model: "llama3".to_string(),
            created_at: 1_714_300_000,
            updated_at: 1_714_300_000,
            task_hash: "abc".to_string(),
            worker_node_id: None,
            result_hash: None,
            task_type: "inference".to_string(),
            redundancy_factor: 1,
        }
    }

    fn signed_result(task_id: &str, text: &str, keypair: &KeyPair) -> ResultEntry {
        let payload = ResultPayload {
            version: TASK_FORMAT_VERSION,
            task_id: task_id.to_string(),
            result_text: text.to_string(),
            tokens_generated: 7,
            generation_time_ms: 100,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 1_714_300_000,
            finished_at: 1_714_300_001,
            output_token_ids: vec![],
        };
        ResultEntry::sign(payload, keypair).expect("sign result")
    }

    /// Boot catch-up path: a `result:` entry already on the doc (a
    /// worker completed it while the daemon was down) must be forwarded
    /// through the real validator loop into the coordinator DB. No
    /// networking — proves the decode → forward → guardrail → persist
    /// chain on its own. The live `InsertRemote` path is covered by the
    /// cross-node test below.
    // multi_thread matches the prod runtime and the iroh-docs actor
    // requirement (P2-A-1, PATTERNS §P54).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn result_sync_boot_catchup_persists_into_coordinator_db() {
        let task_id = "task-rsync-catchup";
        let project_id = "proj-rsync-catchup";

        let node = Arc::new(nexus_core_rs::create_node().await.expect("boot node"));
        let docs = nexus_core_rs::docs::DocsClient::new(node.docs());
        let author = docs.author_default().await.expect("author");
        let doc = Arc::new(docs.create_doc().await.expect("create doc"));

        // Coordinator DB with the task pending.
        let db = CoordinatorDb::open_in_memory().expect("db");
        db.insert_task(&pending_task_record(task_id, project_id))
            .expect("insert task");
        let db = Arc::new(Mutex::new(db));

        // A worker writes its signed result onto the doc *before* the
        // sync loop starts, so only the catch-up scan can find it.
        let worker_kp = KeyPair::generate();
        let result = signed_result(task_id, "catch-up output", &worker_kp);
        doc.set(
            author,
            format!("result:{task_id}").into_bytes(),
            serde_json::to_vec(&result).expect("serialize result"),
        )
        .await
        .expect("write result entry");

        // Wire the real validator loop + the result-sync loop.
        let (tx, rx) = crate::validator_loop::create_result_channel();
        tokio::spawn(crate::validator_loop::run(Arc::clone(&db), rx));
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
        let handle = spawn_result_subscribe(Arc::clone(&doc), Arc::clone(&node), tx, stop_rx);

        // The result must reach the DB via the full bridge → validator
        // → set_task_result chain.
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                {
                    let g = db.lock().unwrap();
                    let detail = g.get_task_result(task_id).expect("get").expect("row");
                    if detail.result_text.as_deref() == Some("catch-up output") {
                        assert_eq!(
                            g.get_task(task_id).expect("get").expect("row").status,
                            TaskStatus::Completed
                        );
                        return;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .expect("result_text should reach the coordinator DB within 10s");

        let _ = stop_tx.send(true);
        handle.await.expect("result sync loop joins");
    }

    /// The real cross-process frontier the 2026-06-05 remediation
    /// demands: a worker on its **own iroh node** claims a task
    /// dispatched by the coordinator and writes a result that flows
    /// back — over genuine iroh-docs replication, not a shared handle —
    /// through the result-sync bridge and validator loop into the
    /// coordinator DB.
    ///
    /// Chain exercised end to end with zero mocks at the frontier
    /// (only the LLM is a deterministic `StubBackend`):
    /// `dispatch_loop` writes `task:` on node A → iroh-docs syncs A→B →
    /// worker engine claims + executes on node B → writes `result:` on
    /// node B → iroh-docs syncs B→A → `result_sync` forwards →
    /// `validator_loop` guardrails + persists → coordinator DB.
    // multi_thread is mandatory: two iroh nodes + the worker pump each
    // need the docs actor on a dedicated thread (P2-A-1, PATTERNS §P54).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn worker_result_syncs_into_coordinator_db_across_two_nodes() {
        use nexus_core_rs::task::{Task, TaskEntry};
        use nexus_worker_core::allowlist::{Allowlist, NewProject};
        use nexus_worker_core::config::{Engine as EngineCfg, WorkerConfig};
        use nexus_worker_core::consent::{ConsentConfig, ConsentLevel};
        use nexus_worker_core::engine::{Engine, EngineBoot};
        use nexus_worker_core::llm::StubBackend;
        use tokio::sync::oneshot;

        let task_id = "task-rsync-xnode";
        let project_id = "proj-rsync-xnode";

        // ---------- Node A: the coordinator ----------
        let node_a = Arc::new(nexus_core_rs::create_node().await.expect("boot node A"));
        let docs_a = nexus_core_rs::docs::DocsClient::new(node_a.docs());
        let author_a = docs_a.author_default().await.expect("author A");
        let doc_a = Arc::new(docs_a.create_doc().await.expect("create doc A"));
        let ticket = doc_a.share_write().await.expect("share write ticket");

        let db = CoordinatorDb::open_in_memory().expect("db");
        db.insert_task(&pending_task_record(task_id, project_id))
            .expect("insert task");
        let db = Arc::new(Mutex::new(db));

        // Validator loop + result-sync bridge on node A.
        let (result_tx, result_rx) = crate::validator_loop::create_result_channel();
        tokio::spawn(crate::validator_loop::run(Arc::clone(&db), result_rx));
        let (rs_stop_tx, rs_stop_rx) = tokio::sync::watch::channel(false);
        let rs_handle = spawn_result_subscribe(
            Arc::clone(&doc_a),
            Arc::clone(&node_a),
            result_tx,
            rs_stop_rx,
        );

        // Dispatch the task through the *real* dispatch loop.
        let (disp_tx, disp_rx) = crate::dispatch_loop::create_dispatch_channel();
        let (disp_stop_tx, disp_stop_rx) = oneshot::channel::<()>();
        let dispatch = tokio::spawn(crate::dispatch_loop::run(
            disp_rx,
            Arc::clone(&doc_a),
            author_a,
            disp_stop_rx,
        ));
        let task = Task {
            version: TASK_FORMAT_VERSION,
            task_id: task_id.into(),
            task_type: "inference".into(),
            prompt: "ping".into(),
            system_prompt: String::new(),
            model: "llama3".into(),
            priority: 5,
            created_at: 1_714_300_000,
            parent_task_id: String::new(),
            metadata: std::collections::BTreeMap::new(),
            is_open_source: true,
            estimated_watts: 0,
            estimated_vram_mb: 0,
            estimated_hours: 0.0,
            redundancy_factor: 1,
            verifiable: false,
            watermark_seed: Vec::new(),
            required_runtime: None,
        };
        let coord_kp = KeyPair::generate();
        let entry = TaskEntry::sign(task, &coord_kp).expect("sign task");
        disp_tx.send(entry).await.expect("dispatch send");
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if doc_a
                    .get_many_by_prefix(b"task:")
                    .await
                    .expect("tasks")
                    .len()
                    == 1
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .expect("dispatcher should write the task within 10s");
        drop(disp_tx);
        let _ = disp_stop_tx.send(());
        dispatch.await.expect("dispatch loop joins");

        // ---------- Node B: the worker (its own iroh node) ----------
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
                id: project_id.into(),
                name: "rsync xnode".into(),
                enabled: true,
                budget_joules: 0,
                // Cross-node sync: the worker joins node A's doc by its
                // shared write ticket — the real production join path.
                tasks_doc_ticket: Some(ticket.to_string()),
            })
            .expect("enroll");

        let sbfb_tmp = tempfile::tempdir().expect("tempdir");
        let mut consent = ConsentConfig::default_for("rsync-xnode-worker");
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
        let mut engine = Engine::new(boot).await.expect("worker engine boots");
        let w_stop = engine.take_shutdown_sender().expect("shutdown sender");
        let worker = tokio::spawn(async move { engine.run_until_shutdown().await });

        // ---------- Assert: the result reached node A's DB ----------
        tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                {
                    let g = db.lock().unwrap();
                    if let Some(detail) = g.get_task_result(task_id).expect("get") {
                        if detail.result_text.is_some() {
                            assert_eq!(
                                g.get_task(task_id).expect("get").expect("row").status,
                                TaskStatus::Completed,
                                "task must be completed once its result syncs back"
                            );
                            return;
                        }
                    }
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        })
        .await
        .expect("worker result should reach the coordinator DB within 30s");

        // Teardown.
        let _ = w_stop.send(());
        worker.await.expect("worker joins").expect("worker ok");
        let _ = rs_stop_tx.send(true);
        rs_handle.await.expect("result sync loop joins");
    }
}
