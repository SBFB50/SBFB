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
    AwaitingQuorum,
    QuorumRejected,
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

    if task.status != TaskStatus::Pending
        && task.status != TaskStatus::Dispatched
        && task.status != TaskStatus::AwaitingQuorum
    {
        tracing::debug!(
            task_id = %entry.payload.task_id,
            status = %task.status.as_str(),
            "result for task not in pending/dispatched/awaiting_quorum state"
        );
        return Ok((ValidationOutcome::RejectedTaskNotPending, None));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let worker_id = hex::encode(entry.worker_pubkey);

    if task.redundancy_factor > 1 {
        return validate_quorum(db, &task, &worker_id, &entry.payload.result_text, now);
    }

    let result_hash = hex::encode(entry.signature);
    db.set_task_result(&entry.payload.task_id, &worker_id, &result_hash, now)?;

    tracing::info!(
        task_id = %entry.payload.task_id,
        worker = %&worker_id[..16],
        tokens = entry.payload.tokens_generated,
        "result accepted"
    );

    Ok((ValidationOutcome::Accepted, Some(task)))
}

/// Quorum path for redundant tasks (`redundancy_factor > 1`).
///
/// Workers agree by **exact equality of `result_text`**. The
/// `sha256` parameter is the worker's raw `result_text`: the column
/// keeps the name `sha256` from its Sprint 55 build-task origin
/// (where it held a binary digest), but for inference results it
/// stores the text verbatim — no hashing happens on this path. A
/// strict majority of identical values is accepted; divergent values
/// are logged as outliers and the task is rejected.
///
/// This exact-match quorum is only *useful* when the workers ran
/// **deterministic** inference (`Task::verifiable` => greedy + fixed
/// seed, Sprint 71 Phase B / B-2): otherwise two honest workers
/// sample different text, never agree, and every redundant inference
/// task is wrongly rejected. Determinism is enforced worker-side at
/// submission (`build_generate_params`); the validator itself is
/// mode-agnostic and unchanged by B-2.
fn validate_quorum(
    db: &CoordinatorDb,
    task: &TaskRecord,
    worker_id: &str,
    sha256: &str,
    now: u64,
) -> Result<(ValidationOutcome, Option<TaskRecord>), CoordinatorError> {
    let inserted = db.insert_task_result(&task.task_id, worker_id, sha256, now)?;
    if !inserted {
        return Ok((ValidationOutcome::AwaitingQuorum, Some(task.clone())));
    }

    let results = db.get_task_results(&task.task_id)?;
    let count = results.len() as u8;

    if count < task.redundancy_factor {
        if task.status != TaskStatus::AwaitingQuorum {
            db.update_task_status(&task.task_id, TaskStatus::AwaitingQuorum, now)?;
        }
        tracing::info!(
            task_id = %task.task_id,
            worker = %&worker_id[..16.min(worker_id.len())],
            results = count,
            required = task.redundancy_factor,
            "build result stored, awaiting quorum"
        );
        return Ok((ValidationOutcome::AwaitingQuorum, Some(task.clone())));
    }

    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for r in &results {
        *counts.entry(r.sha256.as_str()).or_insert(0) += 1;
    }

    let majority_threshold = (task.redundancy_factor as usize) / 2;
    let (best_hash, best_count) = counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(&h, &c)| (h, c))
        .unwrap_or(("", 0));

    if best_count > majority_threshold {
        db.set_task_result(&task.task_id, worker_id, best_hash, now)?;

        for r in &results {
            if r.sha256 != best_hash {
                tracing::warn!(
                    task_id = %task.task_id,
                    outlier_worker = %r.worker_id,
                    outlier_sha256 = %r.sha256,
                    canonical_sha256 = %best_hash,
                    "quorum outlier detected"
                );
            }
        }

        tracing::info!(
            task_id = %task.task_id,
            sha256 = %best_hash,
            agreement = %format!("{best_count}/{count}"),
            "build quorum reached — accepted"
        );

        Ok((ValidationOutcome::Accepted, Some(task.clone())))
    } else {
        db.update_task_status(&task.task_id, TaskStatus::Rejected, now)?;
        tracing::warn!(
            task_id = %task.task_id,
            distinct_hashes = counts.len(),
            "build quorum divergence — rejected"
        );
        Ok((ValidationOutcome::QuorumRejected, Some(task.clone())))
    }
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
            task_type: "inference".to_string(),
            redundancy_factor: 1,
        };
        db.insert_task(&record).expect("insert");
        (db, kp)
    }

    fn setup_build_task(task_id: &str, redundancy_factor: u8) -> CoordinatorDb {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let record = TaskRecord {
            task_id: task_id.to_string(),
            status: TaskStatus::Pending,
            project_id: "proj-1".to_string(),
            model: String::new(),
            created_at: 1714300000,
            updated_at: 1714300000,
            task_hash: "abc".to_string(),
            worker_node_id: None,
            result_hash: None,
            task_type: "build".to_string(),
            redundancy_factor,
        };
        db.insert_task(&record).expect("insert");
        db
    }

    fn make_build_result(task_id: &str, keypair: &KeyPair, sha256: &str) -> ResultEntry {
        let payload = ResultPayload {
            version: TASK_FORMAT_VERSION,
            task_id: task_id.to_string(),
            result_text: sha256.to_string(),
            tokens_generated: 0,
            generation_time_ms: 0,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 1714300000,
            finished_at: 1714300001,
            output_token_ids: vec![],
        };
        ResultEntry::sign(payload, keypair).expect("sign")
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

    #[test]
    fn build_result_transitions_to_awaiting_quorum() {
        let db = setup_build_task("build-001", 3);
        let w1 = KeyPair::generate();
        let entry = make_build_result("build-001", &w1, "aabbccdd");

        let (outcome, _) = validate_result(&db, &entry).expect("validate");
        assert_eq!(outcome, ValidationOutcome::AwaitingQuorum);

        let task = db.get_task("build-001").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::AwaitingQuorum);

        let results = db.get_task_results("build-001").expect("results");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].sha256, "aabbccdd");
    }

    #[test]
    fn quorum_majority_sha256_accepts() {
        let db = setup_build_task("build-002", 3);
        let hash = "deadbeef1234567890abcdef";

        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let w3 = KeyPair::generate();

        let (o1, _) = validate_result(&db, &make_build_result("build-002", &w1, hash)).expect("v1");
        assert_eq!(o1, ValidationOutcome::AwaitingQuorum);

        let (o2, _) = validate_result(&db, &make_build_result("build-002", &w2, hash)).expect("v2");
        assert_eq!(o2, ValidationOutcome::AwaitingQuorum);

        let (o3, _) = validate_result(&db, &make_build_result("build-002", &w3, hash)).expect("v3");
        assert_eq!(o3, ValidationOutcome::Accepted);

        let task = db.get_task("build-002").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.result_hash.as_deref(), Some(hash));
    }

    #[test]
    fn quorum_divergence_rejects() {
        let db = setup_build_task("build-003", 3);

        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let w3 = KeyPair::generate();

        validate_result(&db, &make_build_result("build-003", &w1, "hash_a")).expect("v1");
        validate_result(&db, &make_build_result("build-003", &w2, "hash_b")).expect("v2");

        let (o3, _) =
            validate_result(&db, &make_build_result("build-003", &w3, "hash_c")).expect("v3");
        assert_eq!(o3, ValidationOutcome::QuorumRejected);

        let task = db.get_task("build-003").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Rejected);
    }

    #[test]
    fn quorum_single_outlier_detected() {
        let db = setup_build_task("build-004", 3);
        let canonical = "canonical_sha256_value";

        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let w3 = KeyPair::generate();

        validate_result(&db, &make_build_result("build-004", &w1, canonical)).expect("v1");
        validate_result(&db, &make_build_result("build-004", &w2, "outlier_hash")).expect("v2");

        let (o3, _) =
            validate_result(&db, &make_build_result("build-004", &w3, canonical)).expect("v3");
        assert_eq!(o3, ValidationOutcome::Accepted);

        let task = db.get_task("build-004").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.result_hash.as_deref(), Some(canonical));

        let results = db.get_task_results("build-004").expect("results");
        assert_eq!(results.len(), 3);
        let outliers: Vec<_> = results.iter().filter(|r| r.sha256 != canonical).collect();
        assert_eq!(outliers.len(), 1);
        assert_eq!(outliers[0].sha256, "outlier_hash");
    }

    #[test]
    fn inference_task_bypasses_quorum() {
        let (db, _coord_kp) = setup_db_with_task("inf-001");
        let worker_kp = KeyPair::generate();
        let entry = make_result("inf-001", &worker_kp);

        let (outcome, record) = validate_result(&db, &entry).expect("validate");
        assert_eq!(outcome, ValidationOutcome::Accepted);
        assert!(record.is_some());

        let task = db.get_task("inf-001").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.redundancy_factor, 1);
    }

    // -----------------------------------------------------------
    // Sprint 71 Phase B — deterministic-quorum (B-2) properties
    // -----------------------------------------------------------

    #[test]
    fn two_honest_workers_same_hash() {
        // Two independent workers that ran deterministic (greedy +
        // fixed-seed) inference produce the SAME `result_text`. Their
        // results are signed by different keypairs (distinct
        // signatures / worker ids) but carry the same quorum key, so
        // the validator counts them as agreeing — the property B-2
        // buys: honest workers converge instead of diverging.
        let db = setup_build_task("det-pair", 2);
        let agreed = "deterministic greedy output";

        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let r1 = make_build_result("det-pair", &w1, agreed);
        let r2 = make_build_result("det-pair", &w2, agreed);
        // Distinct workers, distinct signatures, identical quorum key.
        assert_ne!(r1.signature, r2.signature);
        assert_eq!(r1.payload.result_text, r2.payload.result_text);

        validate_result(&db, &r1).expect("v1");
        validate_result(&db, &r2).expect("v2");

        let results = db.get_task_results("det-pair").expect("results");
        assert_eq!(results.len(), 2);
        // Both honest results landed under one quorum key.
        let distinct: std::collections::HashSet<&str> =
            results.iter().map(|r| r.sha256.as_str()).collect();
        assert_eq!(distinct.len(), 1, "deterministic honest workers agree");
    }

    #[test]
    fn quorum_accepts_deterministic_redundancy() {
        // The B-2 acceptance path end to end: at redundancy_factor=2,
        // two honest workers with identical deterministic output reach
        // a strict majority and the task is Accepted.
        let db = setup_build_task("det-accept", 2);
        let agreed = "stable greedy answer";

        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();

        let (o1, _) =
            validate_result(&db, &make_build_result("det-accept", &w1, agreed)).expect("v1");
        assert_eq!(o1, ValidationOutcome::AwaitingQuorum);

        let (o2, _) =
            validate_result(&db, &make_build_result("det-accept", &w2, agreed)).expect("v2");
        assert_eq!(o2, ValidationOutcome::Accepted);

        let task = db.get_task("det-accept").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.result_hash.as_deref(), Some(agreed));
    }

    #[test]
    fn quorum_rejects_nondeterministic_divergence() {
        // The mirror property: had the workers sampled
        // non-deterministically, their `result_text` would diverge, no
        // value would reach a strict majority, and the task is
        // QuorumRejected. This is exactly the failure B-2 forces
        // worker-side determinism to avoid; outlier rejection stays
        // intact.
        let db = setup_build_task("nondet-reject", 2);

        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();

        validate_result(
            &db,
            &make_build_result("nondet-reject", &w1, "sampled text A"),
        )
        .expect("v1");
        let (o2, _) = validate_result(
            &db,
            &make_build_result("nondet-reject", &w2, "sampled text B"),
        )
        .expect("v2");
        assert_eq!(o2, ValidationOutcome::QuorumRejected);

        let task = db.get_task("nondet-reject").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Rejected);
    }
}
