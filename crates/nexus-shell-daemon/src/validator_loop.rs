// SPDX-License-Identifier: AGPL-3.0-or-later
//! Event-driven result validation loop (Sprint 38 Phase A, MANDATORY 3/3).
//!
//! Subscribes to a broadcast channel of [`ResultEvent`]s and validates
//! each result via [`nexus_coordinator_rs::validator::validate_result`],
//! crediting kudos on acceptance. Runs as a background tokio task in the
//! daemon runtime alongside the HTTP server and gossip subscribe loop.
//!
//! The HTTP `POST /api/v1/results/submit` handler remains as a synchronous
//! fallback path. The broadcast channel is the primary event-driven path
//! for gossip-originated results (wired in future sprints).

use std::sync::{Arc, Mutex};

use nexus_coordinator_rs::db::CoordinatorDb;
use nexus_coordinator_rs::kudos_ledger;
use nexus_coordinator_rs::validator::{self, ValidationOutcome};
use nexus_core_rs::task::ResultEntry;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ResultEvent {
    NewResult(ResultEntry),
}

pub type ResultEventSender = broadcast::Sender<ResultEvent>;

const CHANNEL_CAPACITY: usize = 64;

pub fn create_result_channel() -> (ResultEventSender, broadcast::Receiver<ResultEvent>) {
    broadcast::channel(CHANNEL_CAPACITY)
}

pub async fn run(db: Arc<Mutex<CoordinatorDb>>, mut rx: broadcast::Receiver<ResultEvent>) {
    tracing::info!("validator_loop started");
    loop {
        match rx.recv().await {
            Ok(ResultEvent::NewResult(entry)) => {
                process_result(&db, &entry);
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!(n, "validator_loop lagged, dropped events");
            }
            Err(broadcast::error::RecvError::Closed) => {
                tracing::info!("validator_loop channel closed, exiting");
                break;
            }
        }
    }
}

fn process_result(db: &Arc<Mutex<CoordinatorDb>>, entry: &ResultEntry) {
    let guard = match db.lock() {
        Ok(g) => g,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned in validator_loop");
            return;
        }
    };

    match validator::validate_result(&guard, entry) {
        Ok((ValidationOutcome::Accepted, Some(task_record))) => {
            let worker_id = hex::encode(entry.worker_pubkey);
            if let Err(e) = kudos_ledger::credit(
                &guard,
                &task_record.project_id,
                &worker_id,
                &entry.payload.task_id,
                entry.payload.tokens_generated,
            ) {
                tracing::warn!(
                    task_id = %entry.payload.task_id,
                    "kudos credit failed in validator_loop (non-fatal): {e}"
                );
            }
            tracing::info!(
                task_id = %entry.payload.task_id,
                "result accepted via validator_loop"
            );
        }
        Ok((outcome, _)) => {
            tracing::debug!(
                task_id = %entry.payload.task_id,
                outcome = ?outcome,
                "result rejected in validator_loop"
            );
        }
        Err(e) => {
            tracing::error!(
                task_id = %entry.payload.task_id,
                "validation failed in validator_loop: {e}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_coordinator_rs::types::{TaskRecord, TaskStatus};
    use nexus_core_rs::crypto::KeyPair;
    use nexus_core_rs::task::{ResultPayload, TASK_FORMAT_VERSION};

    fn setup_db_with_task(task_id: &str) -> CoordinatorDb {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let record = TaskRecord {
            task_id: task_id.to_string(),
            status: TaskStatus::Pending,
            project_id: "proj-1".to_string(),
            model: "llama3".to_string(),
            created_at: 1714300000,
            updated_at: 1714300000,
            task_hash: "abc".to_string(),
            worker_node_id: None,
            result_hash: None,
            task_type: "inference".to_string(),
            redundancy_factor: 1,
        };
        db.insert_task(&record).expect("insert");
        db
    }

    fn make_result(task_id: &str, keypair: &KeyPair) -> ResultEntry {
        let payload = ResultPayload {
            version: TASK_FORMAT_VERSION,
            task_id: task_id.to_string(),
            result_text: "test output".to_string(),
            tokens_generated: 42,
            generation_time_ms: 500,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 1714300000,
            finished_at: 1714300001,
            output_token_ids: vec![],
        };
        ResultEntry::sign(payload, keypair).expect("sign")
    }

    #[tokio::test]
    async fn validator_loop_processes_result() {
        let db = setup_db_with_task("task-vl-1");
        let db = Arc::new(Mutex::new(db));
        let (tx, rx) = create_result_channel();

        let worker_kp = KeyPair::generate();
        let entry = make_result("task-vl-1", &worker_kp);

        let db_clone = Arc::clone(&db);
        let handle = tokio::spawn(run(db_clone, rx));

        tx.send(ResultEvent::NewResult(entry)).expect("send");
        drop(tx);
        handle.await.expect("validator loop joins");

        let guard = db.lock().unwrap();
        let task = guard.get_task("task-vl-1").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Completed);
        let kudos = guard.get_project_kudos_total("proj-1").expect("k");
        assert!(kudos > 0, "kudos must be credited after accepted result");
    }

    #[tokio::test]
    async fn validator_loop_idempotent_double_submit() {
        let db = setup_db_with_task("task-vl-2");
        let db = Arc::new(Mutex::new(db));
        let (tx, rx) = create_result_channel();

        let worker_kp = KeyPair::generate();
        let entry = make_result("task-vl-2", &worker_kp);

        let db_clone = Arc::clone(&db);
        let handle = tokio::spawn(run(db_clone, rx));

        tx.send(ResultEvent::NewResult(entry.clone()))
            .expect("send 1");
        tx.send(ResultEvent::NewResult(entry)).expect("send 2");
        drop(tx);
        handle.await.expect("validator loop joins");

        let guard = db.lock().unwrap();
        let single_credit = nexus_coordinator_rs::kudos_ledger::log_utility(42);
        assert_eq!(
            guard.get_project_kudos_total("proj-1").expect("k"),
            single_credit,
            "double submit must credit only once"
        );
    }

    #[tokio::test]
    async fn validator_loop_rejects_bad_signature() {
        let db = setup_db_with_task("task-vl-3");
        let db = Arc::new(Mutex::new(db));
        let (tx, rx) = create_result_channel();

        let worker_kp = KeyPair::generate();
        let mut entry = make_result("task-vl-3", &worker_kp);
        entry.signature[0] ^= 0xff;

        let db_clone = Arc::clone(&db);
        let handle = tokio::spawn(run(db_clone, rx));

        tx.send(ResultEvent::NewResult(entry)).expect("send");
        drop(tx);
        handle.await.expect("validator loop joins");

        let guard = db.lock().unwrap();
        let task = guard.get_task("task-vl-3").expect("get").expect("found");
        assert_eq!(
            task.status,
            TaskStatus::Pending,
            "bad sig must not transition"
        );
        assert_eq!(guard.get_project_kudos_total("proj-1").expect("k"), 0);
    }
}
