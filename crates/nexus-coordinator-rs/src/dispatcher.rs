// SPDX-License-Identifier: AGPL-3.0-or-later
//! Task dispatcher — signs and persists task submissions.
//!
//! The dispatcher receives a [`TaskSubmission`] (user-facing input),
//! constructs a signed [`TaskEntry`] using the coordinator's Ed25519
//! keypair, persists it in the local SQLite database, and returns
//! the entry for downstream broadcast to the P2P network.

use std::time::{SystemTime, UNIX_EPOCH};

use nexus_core_rs::crypto::KeyPair;
use nexus_core_rs::task::{TASK_FORMAT_VERSION, Task, TaskEntry};

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;
use crate::types::{TaskRecord, TaskStatus, TaskSubmission};

const BUILD_TASK_TYPE: &str = "build";
const REQUIRED_BUILD_METADATA: &[&str] = &["build.repo", "build.commit", "build.binary"];

fn validate_build_submission(submission: &TaskSubmission) -> Result<(), CoordinatorError> {
    for key in REQUIRED_BUILD_METADATA {
        match submission.metadata.get(*key) {
            Some(v) if !v.is_empty() => {}
            _ => {
                return Err(CoordinatorError::Validation(format!(
                    "build task requires non-empty metadata key '{key}'"
                )));
            }
        }
    }
    Ok(())
}

const BUILD_DEFAULT_REDUNDANCY: u8 = 3;

pub fn submit_task(
    db: &CoordinatorDb,
    keypair: &KeyPair,
    submission: TaskSubmission,
) -> Result<TaskEntry, CoordinatorError> {
    let is_build = submission.task_type == BUILD_TASK_TYPE;
    if is_build {
        validate_build_submission(&submission)?;
    } else {
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
    }

    let redundancy = if is_build {
        submission.redundancy_factor.max(BUILD_DEFAULT_REDUNDANCY)
    } else {
        submission.redundancy_factor
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let task_id = format!("{:016x}-{:016x}", now, rand::random::<u64>());
    let task_type = submission.task_type.clone();

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
        redundancy_factor: redundancy,
        verifiable: submission.verifiable,
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
        task_type,
        redundancy_factor: redundancy,
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
            verifiable: false,
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
    fn submit_propagates_verifiable_flag() {
        // The coordinator craft path carries `verifiable` into the
        // signed Task, so a caller can request deterministic compute
        // end to end — not just via direct Task construction
        // (Sprint 71 Phase B, B-2).
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let dispatcher = TaskDispatcher::new(db, kp);

        let mut sub = make_submission();
        sub.verifiable = true;
        let entry = dispatcher.submit(sub).expect("submit");
        assert!(entry.task.verifiable, "craft path must carry verifiable");
        // The flag lives inside the signed canonical bytes.
        entry.verify_signature().expect("signature must verify");

        // A default submission stays best-effort (verifiable = false).
        let plain = dispatcher.submit(make_submission()).expect("submit");
        assert!(!plain.task.verifiable);
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
    fn submit_build_task_accepts_empty_prompt() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let dispatcher = TaskDispatcher::new(db, kp);

        let mut sub = make_submission();
        sub.task_type = "build".into();
        sub.prompt = String::new();
        sub.model = String::new();
        sub.metadata.insert(
            "build.repo".into(),
            "https://github.com/example/repo".into(),
        );
        sub.metadata
            .insert("build.commit".into(), "abc123def456".into());
        sub.metadata
            .insert("build.binary".into(), "my-binary".into());

        let entry = dispatcher
            .submit(sub)
            .expect("build task should be accepted");
        assert_eq!(entry.task.task_type, "build");
        assert!(entry.task.prompt.is_empty());
        assert!(entry.task.model.is_empty());
    }

    #[test]
    fn submit_build_task_requires_metadata_keys() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = KeyPair::generate();
        let dispatcher = TaskDispatcher::new(db, kp);

        let mut sub = make_submission();
        sub.task_type = "build".into();
        sub.prompt = String::new();
        sub.model = String::new();

        let err = dispatcher
            .submit(sub)
            .expect_err("should reject missing metadata");
        assert!(matches!(err, CoordinatorError::Validation(_)));
        assert!(err.to_string().contains("build.repo"));
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
