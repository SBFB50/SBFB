// SPDX-License-Identifier: AGPL-3.0-or-later
//! Task dispatcher — signs and persists task submissions.
//!
//! The dispatcher receives a [`TaskSubmission`] (user-facing input),
//! constructs a signed [`TaskEntry`] using the coordinator's Ed25519
//! keypair, persists it in the local SQLite database, and returns
//! the entry for downstream broadcast to the P2P network.

use std::time::{SystemTime, UNIX_EPOCH};

use nexus_core_rs::crypto::KeyPair;
use nexus_core_rs::task::{Task, TaskEntry, TASK_FORMAT_VERSION};

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;
use crate::types::{TaskRecord, TaskStatus, TaskSubmission};

pub fn submit_task(
    db: &CoordinatorDb,
    keypair: &KeyPair,
    submission: TaskSubmission,
) -> Result<TaskEntry, CoordinatorError> {
    if submission.prompt.is_empty() {
        return Err(CoordinatorError::Validation(
            "prompt must not be empty".into(),
        ));
    }
    if submission.model.is_empty() {
        return Err(CoordinatorError::Validation(
            "model must not be empty".into(),
        ));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let task_id = format!("{:016x}-{:016x}", now, rand::random::<u64>());

    let task = Task {
        version: TASK_FORMAT_VERSION,
        task_id: task_id.clone(),
        task_type: submission.task_type,
        prompt: submission.prompt,
        system_prompt: submission.system_prompt,
        model: submission.model.clone(),
        priority: submission.priority,
        created_at: now,
        parent_task_id: submission.parent_task_id,
        metadata: submission.metadata,
        is_open_source: submission.is_open_source,
        estimated_watts: submission.estimated_watts,
        estimated_vram_mb: submission.estimated_vram_mb,
        estimated_hours: submission.estimated_hours,
        redundancy_factor: submission.redundancy_factor,
        watermark_seed: Vec::new(),
    };

    let entry =
        TaskEntry::sign(task, keypair).map_err(|e| CoordinatorError::Crypto(e.to_string()))?;

    let task_hash = hex::encode(entry.signature);

    let record = TaskRecord {
        task_id: entry.task.task_id.clone(),
        status: TaskStatus::Pending,
        project_id: submission.project_id,
        model: submission.model,
        created_at: now,
        updated_at: now,
        task_hash,
        worker_node_id: None,
        result_hash: None,
    };

    db.insert_task(&record)?;

    Ok(entry)
}

pub struct TaskDispatcher {
    db: CoordinatorDb,
    keypair: KeyPair,
}

impl TaskDispatcher {
    pub fn new(db: CoordinatorDb, keypair: KeyPair) -> Self {
        Self { db, keypair }
    }

    pub fn submit(&self, submission: TaskSubmission) -> Result<TaskEntry, CoordinatorError> {
        submit_task(&self.db, &self.keypair, submission)
    }

    pub fn db(&self) -> &CoordinatorDb {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn make_submission() -> TaskSubmission {
        TaskSubmission {
            project_id: "test-project".into(),
            task_type: "analysis".into(),
            prompt: "Analyze this text".into(),
            system_prompt: String::new(),
            model: "llama3".into(),
            priority: 5,
            parent_task_id: String::new(),
            metadata: BTreeMap::new(),
            is_open_source: false,
            estimated_watts: 0,
            estimated_vram_mb: 0,
            estimated_hours: 0.0,
            redundancy_factor: 1,
        }
    }

    #[test]
    fn submit_produces_valid_signed_entry() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let dispatcher = TaskDispatcher::new(db, kp);

        let entry = dispatcher.submit(make_submission()).expect("submit");

        assert_eq!(entry.task.version, TASK_FORMAT_VERSION);
        assert_eq!(entry.task.model, "llama3");
        assert!(!entry.task.task_id.is_empty());
        entry.verify_signature().expect("signature must be valid");
    }

    #[test]
    fn submit_persists_task_in_db() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let dispatcher = TaskDispatcher::new(db, kp);

        let entry = dispatcher.submit(make_submission()).expect("submit");

        let record = dispatcher
            .db()
            .get_task(&entry.task.task_id)
            .expect("get")
            .expect("task must exist in DB");
        assert_eq!(record.status, TaskStatus::Pending);
        assert_eq!(record.project_id, "test-project");
    }

    #[test]
    fn submit_rejects_empty_prompt() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let dispatcher = TaskDispatcher::new(db, kp);

        let mut sub = make_submission();
        sub.prompt = String::new();
        let err = dispatcher.submit(sub).expect_err("should reject");
        assert!(matches!(err, CoordinatorError::Validation(_)));
    }

    #[test]
    fn submit_rejects_empty_model() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let dispatcher = TaskDispatcher::new(db, kp);

        let mut sub = make_submission();
        sub.model = String::new();
        let err = dispatcher.submit(sub).expect_err("should reject");
        assert!(matches!(err, CoordinatorError::Validation(_)));
    }

    #[test]
    fn submit_generates_unique_task_ids() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let dispatcher = TaskDispatcher::new(db, kp);

        let e1 = dispatcher.submit(make_submission()).expect("submit 1");
        let e2 = dispatcher.submit(make_submission()).expect("submit 2");
        assert_ne!(e1.task.task_id, e2.task.task_id);
    }

    #[test]
    fn signed_entry_canonical_bytes_are_verifiable() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let pub_bytes = kp.public_bytes();
        let dispatcher = TaskDispatcher::new(db, kp);

        let entry = dispatcher.submit(make_submission()).expect("submit");

        assert_eq!(entry.author_pubkey, pub_bytes);
        entry
            .verify_signature()
            .expect("canonical bytes + Ed25519 verify must pass");
    }
}
