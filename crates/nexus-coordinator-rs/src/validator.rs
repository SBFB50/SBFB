// SPDX-License-Identifier: AGPL-3.0-or-later
//! Result validator — verifies worker results and updates task state.
//!
//! The validator receives a [`ResultEntry`] from the P2P network,
//! runs the 3-layer verification (signature + task existence +
//! status guard), and transitions the task to completed or rejected
//! in the local database.

use nexus_core_rs::task::ResultEntry;

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;
use crate::types::{TaskRecord, TaskStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationOutcome {
    Accepted,
    RejectedBadSignature,
    RejectedTaskNotFound,
    RejectedTaskNotPending,
}

pub fn validate_result(
    db: &CoordinatorDb,
    entry: &ResultEntry,
) -> Result<(ValidationOutcome, Option<TaskRecord>), CoordinatorError> {
    if entry.verify_signature().is_err() {
        tracing::warn!(
            task_id = %entry.payload.task_id,
            "result signature verification failed"
        );
        return Ok((ValidationOutcome::RejectedBadSignature, None));
    }

    let task = match db.get_task(&entry.payload.task_id)? {
        Some(t) => t,
        None => {
            tracing::warn!(
                task_id = %entry.payload.task_id,
                "result references unknown task"
            );
            return Ok((ValidationOutcome::RejectedTaskNotFound, None));
        }
    };

    if task.status != TaskStatus::Pending && task.status != TaskStatus::Dispatched {
        tracing::debug!(
            task_id = %entry.payload.task_id,
            status = %task.status.as_str(),
            "result for task not in pending/dispatched state"
        );
        return Ok((ValidationOutcome::RejectedTaskNotPending, None));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let worker_id = hex::encode(entry.worker_pubkey);
    let result_hash = hex::encode(entry.signature);

    db.set_task_result(&entry.payload.task_id, &worker_id, &result_hash, now)?;

    tracing::info!(
        task_id = %entry.payload.task_id,
        worker = %worker_id[..16],
        tokens = entry.payload.tokens_generated,
        "result accepted"
    );

    Ok((ValidationOutcome::Accepted, Some(task)))
}

pub struct ResultValidator {
    db: CoordinatorDb,
}

impl ResultValidator {
    pub fn new(db: CoordinatorDb) -> Self {
        Self { db }
    }

    pub fn validate(
        &self,
        entry: &ResultEntry,
    ) -> Result<(ValidationOutcome, Option<TaskRecord>), CoordinatorError> {
        validate_result(&self.db, entry)
    }

    pub fn db(&self) -> &CoordinatorDb {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use nexus_core_rs::crypto::KeyPair;
    use nexus_core_rs::task::{ResultPayload, TASK_FORMAT_VERSION};

    use super::*;
    use crate::types::TaskRecord;

    fn setup_db_with_task(task_id: &str) -> (CoordinatorDb, KeyPair) {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
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
        };
        db.insert_task(&record).expect("insert");
        (db, kp)
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

    #[test]
    fn accepts_valid_result_and_transitions_to_completed() {
        let (db, _coord_kp) = setup_db_with_task("task-100");
        let worker_kp = KeyPair::generate();
        let entry = make_result("task-100", &worker_kp);

        let validator = ResultValidator::new(db);
        let (outcome, record) = validator.validate(&entry).expect("validate");
        assert_eq!(outcome, ValidationOutcome::Accepted);
        let record = record.expect("accepted must return TaskRecord");
        assert_eq!(record.project_id, "proj-1");

        let task = validator
            .db()
            .get_task("task-100")
            .expect("get")
            .expect("found");
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.worker_node_id.is_some());
        assert!(task.result_hash.is_some());
    }

    #[test]
    fn rejects_bad_signature() {
        let (db, _coord_kp) = setup_db_with_task("task-101");
        let worker_kp = KeyPair::generate();
        let mut entry = make_result("task-101", &worker_kp);
        entry.signature[0] ^= 0xff;

        let validator = ResultValidator::new(db);
        let (outcome, record) = validator.validate(&entry).expect("validate");
        assert_eq!(outcome, ValidationOutcome::RejectedBadSignature);
        assert!(record.is_none());

        let task = validator
            .db()
            .get_task("task-101")
            .expect("get")
            .expect("found");
        assert_eq!(
            task.status,
            TaskStatus::Pending,
            "task must stay pending on bad sig"
        );
    }

    #[test]
    fn rejects_unknown_task() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let worker_kp = KeyPair::generate();
        let entry = make_result("nonexistent", &worker_kp);

        let validator = ResultValidator::new(db);
        let (outcome, record) = validator.validate(&entry).expect("validate");
        assert_eq!(outcome, ValidationOutcome::RejectedTaskNotFound);
        assert!(record.is_none());
    }

    #[test]
    fn rejects_already_completed_task() {
        let (db, _coord_kp) = setup_db_with_task("task-102");
        let worker_kp = KeyPair::generate();

        db.set_task_result("task-102", "w1", "r1", 100)
            .expect("first complete");

        let entry = make_result("task-102", &worker_kp);
        let validator = ResultValidator::new(db);
        let (outcome, record) = validator.validate(&entry).expect("validate");
        assert_eq!(outcome, ValidationOutcome::RejectedTaskNotPending);
        assert!(record.is_none());
    }

    #[test]
    fn accepts_dispatched_task() {
        let (db, _coord_kp) = setup_db_with_task("task-103");
        db.update_task_status("task-103", TaskStatus::Dispatched, 100)
            .expect("dispatch");

        let worker_kp = KeyPair::generate();
        let entry = make_result("task-103", &worker_kp);

        let validator = ResultValidator::new(db);
        let (outcome, record) = validator.validate(&entry).expect("validate");
        assert_eq!(outcome, ValidationOutcome::Accepted);
        let record = record.expect("accepted must return TaskRecord");
        assert_eq!(record.project_id, "proj-1");
    }
}
