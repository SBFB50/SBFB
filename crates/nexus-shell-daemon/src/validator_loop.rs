// SPDX-License-Identifier: AGPL-3.0-or-later
//! Event-driven result validation loop (Sprint 38 Phase A, MANDATORY 3/3).
//!
//! Subscribes to a broadcast channel of [`ResultEvent`]s and validates
//! each result via [`validate_result_pre_guardrail`] →
//! [`default_output_chain`] → [`validate_result_post_guardrail`]
//! (Sprint 73 Phase A, D5: the output guardrail runs BEFORE the result
//! text is persisted), crediting kudos only on acceptance. Runs as a
//! background tokio task in the daemon runtime alongside the HTTP server
//! and gossip subscribe loop.
//!
//! [`validate_result_pre_guardrail`]: nexus_coordinator_rs::validator::validate_result_pre_guardrail
//! [`validate_result_post_guardrail`]: nexus_coordinator_rs::validator::validate_result_post_guardrail
//!
//! The HTTP `POST /api/v1/results/submit` handler remains as a synchronous
//! fallback path. The broadcast channel is the primary event-driven path
//! for gossip-originated results (wired in future sprints).

use std::sync::{Arc, Mutex};

use nexus_coordinator_rs::db::CoordinatorDb;
use nexus_coordinator_rs::guardrails::{GuardrailContext, default_output_chain};
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

    match validator::validate_result_pre_guardrail(&guard, entry) {
        Ok((ValidationOutcome::Accepted, Some(task_record), Some(pending))) => {
            // Sprint 73 Phase A (D5): gossip-sourced results are the most
            // sensitive ingress — they arrive from untrusted network peers
            // (Sprint 38) and historically reached the DB with NO output
            // guardrail at all. Run the guardrail BEFORE persisting; a
            // tripwire skips persistence entirely and credits no kudos.
            let guardrail_ctx = GuardrailContext {
                system_prompt: "",
                user_prompt: "",
                model_output: &pending.result_text,
            };
            let gr = default_output_chain().run(&guardrail_ctx);
            if !gr.passed {
                let reason = gr.tripwire.unwrap_or_else(|| "guardrail_rejected".into());
                tracing::warn!(
                    task_id = %entry.payload.task_id,
                    %reason,
                    "gossip result rejected by output guardrail — not persisted, no kudos credited"
                );
                return;
            }
            if let Err(e) = validator::validate_result_post_guardrail(&guard, &pending) {
                tracing::error!(
                    task_id = %entry.payload.task_id,
                    "result persist failed in validator_loop: {e}"
                );
                return;
            }
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
        Ok((outcome, _, _)) => {
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
        make_result_with_text(task_id, keypair, "test output")
    }

    fn make_result_with_text(task_id: &str, keypair: &KeyPair, text: &str) -> ResultEntry {
        let payload = ResultPayload {
            version: TASK_FORMAT_VERSION,
            task_id: task_id.to_string(),
            result_text: text.to_string(),
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

    // ---------------------------------------------------------------
    // Sprint 73 Phase A — guardrail-before-persist on the gossip path (D5)
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn validator_loop_rejected_result_not_persisted() {
        // A gossip-sourced result whose text trips the output guardrail
        // (invisible char) must NOT be persisted and must NOT credit kudos.
        // Before D5 the validator_loop had no guardrail at all and would
        // have completed the task with the rejected text.
        let db = setup_db_with_task("task-vl-gr");
        let db = Arc::new(Mutex::new(db));
        let (tx, rx) = create_result_channel();

        let worker_kp = KeyPair::generate();
        let entry = make_result_with_text("task-vl-gr", &worker_kp, "leaked\u{200B}secret");

        let db_clone = Arc::clone(&db);
        let handle = tokio::spawn(run(db_clone, rx));

        tx.send(ResultEvent::NewResult(entry)).expect("send");
        drop(tx);
        handle.await.expect("validator loop joins");

        let guard = db.lock().unwrap();
        let task = guard.get_task("task-vl-gr").expect("get").expect("found");
        assert_eq!(
            task.status,
            TaskStatus::Pending,
            "guardrail tripwire must not transition the task to completed"
        );
        assert!(
            guard
                .get_task_result("task-vl-gr")
                .expect("get")
                .expect("found")
                .result_text
                .is_none(),
            "no retrievable text after a guardrail rejection"
        );
        assert_eq!(
            guard.get_project_kudos_total("proj-1").expect("k"),
            0,
            "no kudos for guardrail-rejected output"
        );
    }

    #[tokio::test]
    async fn validator_loop_accepted_result_persisted() {
        // A clean gossip-sourced result clears the guardrail and IS
        // persisted retrievably, with kudos credited — proving persistence
        // happens on the post-guardrail path, not before it.
        let db = setup_db_with_task("task-vl-ok");
        let db = Arc::new(Mutex::new(db));
        let (tx, rx) = create_result_channel();

        let worker_kp = KeyPair::generate();
        let entry = make_result_with_text("task-vl-ok", &worker_kp, "clean output");

        let db_clone = Arc::clone(&db);
        let handle = tokio::spawn(run(db_clone, rx));

        tx.send(ResultEvent::NewResult(entry)).expect("send");
        drop(tx);
        handle.await.expect("validator loop joins");

        let guard = db.lock().unwrap();
        let task = guard.get_task("task-vl-ok").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(
            guard
                .get_task_result("task-vl-ok")
                .expect("get")
                .expect("found")
                .result_text
                .as_deref(),
            Some("clean output"),
            "clean text persisted after the guardrail cleared it"
        );
        assert!(
            guard.get_project_kudos_total("proj-1").expect("k") > 0,
            "kudos credited for guardrail-cleared output"
        );
    }
}
