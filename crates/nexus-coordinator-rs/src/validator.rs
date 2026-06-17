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

/// A result that has cleared every check EXCEPT the output guardrail,
/// ready to be persisted once the caller runs
/// [`crate::guardrails::default_output_chain`] on `result_text` and it
/// passes.
///
/// Sprint 73 Phase A (D5): persistence ([`CoordinatorDb::set_task_result`],
/// which flips the task to `completed` AND writes the retrievable
/// `result_text` in one atomic UPDATE) must happen STRICTLY AFTER the
/// output guardrail clears the text. Otherwise `GET
/// /api/v1/tasks/{id}/result` briefly serves content the guardrail later
/// rejects, and kudos get credited for it. The
/// [`validate_result_pre_guardrail`] / [`validate_result_post_guardrail`]
/// split makes guardrail-before-persist the only reachable order on the
/// two network ingress points (HTTP `coordinator_submit_result`, gossip
/// `validator_loop`).
#[derive(Debug, Clone)]
pub struct PendingResultPersist {
    pub task_id: String,
    pub worker_id: String,
    /// Provenance hash: single path = signature hex; quorum path = the
    /// agreed value (`best_hash`).
    pub result_hash: String,
    /// Human-readable text to guardrail then persist: single path =
    /// worker `payload.result_text`; quorum path = the agreed text
    /// (`best_hash`, which on that path IS the text — PATTERNS §P53).
    pub result_text: String,
    pub now: u64,
}

/// Validate a result up to — but NOT including — persistence.
///
/// Runs the 3-layer verification (signature + task existence + status
/// guard) and, for redundant tasks, the quorum accumulation. It NEVER
/// calls [`CoordinatorDb::set_task_result`]: when the result is
/// acceptable it returns a [`PendingResultPersist`] describing what to
/// persist, leaving the caller to run the output guardrail first (Sprint
/// 73 Phase A, D5).
///
/// Returns `(outcome, task_record, pending)`. `pending` is `Some` only
/// when `outcome == Accepted` (a single result, or a quorum just
/// reached); the task row is still un-completed at that point —
/// [`validate_result_post_guardrail`] completes it. `pending` is `None`
/// for every non-accepting outcome and for `AwaitingQuorum`.
pub fn validate_result_pre_guardrail(
    db: &CoordinatorDb,
    entry: &ResultEntry,
) -> Result<
    (
        ValidationOutcome,
        Option<TaskRecord>,
        Option<PendingResultPersist>,
    ),
    CoordinatorError,
> {
    if entry.verify_signature().is_err() {
        tracing::warn!(
            task_id = %entry.payload.task_id,
            "result signature verification failed"
        );
        return Ok((ValidationOutcome::RejectedBadSignature, None, None));
    }

    let task = match db.get_task(&entry.payload.task_id)? {
        Some(t) => t,
        None => {
            tracing::warn!(
                task_id = %entry.payload.task_id,
                "result references unknown task"
            );
            return Ok((ValidationOutcome::RejectedTaskNotFound, None, None));
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
        return Ok((ValidationOutcome::RejectedTaskNotPending, None, None));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let worker_id = hex::encode(entry.worker_pubkey);

    if task.redundancy_factor > 1 {
        return validate_quorum_pre_guardrail(
            db,
            &task,
            &worker_id,
            &entry.payload.result_text,
            now,
        );
    }

    // Single-result path: defer the human-readable text AND the
    // provenance hash for persistence behind the output guardrail.
    let pending = PendingResultPersist {
        task_id: entry.payload.task_id.clone(),
        worker_id,
        result_hash: hex::encode(entry.signature),
        result_text: entry.payload.result_text.clone(),
        now,
    };

    tracing::debug!(
        task_id = %entry.payload.task_id,
        tokens = entry.payload.tokens_generated,
        "result validated, pending output guardrail"
    );

    Ok((ValidationOutcome::Accepted, Some(task), Some(pending)))
}

/// Persist a result that has cleared the output guardrail (Sprint 73
/// Phase A, D5).
///
/// This is the ONLY function that writes the retrievable `result_text`
/// and flips the task to `completed`. Callers MUST run
/// [`crate::guardrails::default_output_chain`] on `pending.result_text`
/// and confirm it passed before calling this — see
/// [`PendingResultPersist`].
pub fn validate_result_post_guardrail(
    db: &CoordinatorDb,
    pending: &PendingResultPersist,
) -> Result<(), CoordinatorError> {
    db.set_task_result(
        &pending.task_id,
        &pending.worker_id,
        &pending.result_hash,
        &pending.result_text,
        pending.now,
    )?;

    tracing::info!(
        task_id = %pending.task_id,
        "result persisted after output guardrail"
    );

    Ok(())
}

/// Terminally reject a task whose validated result tripped the output
/// guardrail (Sprint 75 Phase G, CARRY-2 / S74 audit).
///
/// Both network ingress points (HTTP `coordinator_submit_result` and the
/// gossip `validator_loop`) run the output guardrail between
/// [`validate_result_pre_guardrail`] and
/// [`validate_result_post_guardrail`]. Before this helper a tripwire was
/// only logged and the task silently kept its prior non-terminal state —
/// a zombie: the validated submission was already consumed (single path)
/// or the quorum already reached (redundant path), so no future event
/// could ever move the task again. Marking it `Rejected` makes the trip
/// terminal and observable, and the status guard in
/// [`validate_result_pre_guardrail`] then refuses any late submission.
/// The quorum `task_results` rows are kept deliberately: they are the
/// audit trail of what was submitted and become inert once the task is
/// terminal.
pub fn reject_result_on_guardrail_trip(
    db: &CoordinatorDb,
    pending: &PendingResultPersist,
) -> Result<(), CoordinatorError> {
    db.update_task_status(&pending.task_id, TaskStatus::Rejected, pending.now)?;
    tracing::warn!(
        task_id = %pending.task_id,
        "task terminally rejected after output-guardrail tripwire"
    );
    Ok(())
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
fn validate_quorum_pre_guardrail(
    db: &CoordinatorDb,
    task: &TaskRecord,
    worker_id: &str,
    sha256: &str,
    now: u64,
) -> Result<
    (
        ValidationOutcome,
        Option<TaskRecord>,
        Option<PendingResultPersist>,
    ),
    CoordinatorError,
> {
    let inserted = db.insert_task_result(&task.task_id, worker_id, sha256, now)?;
    if !inserted {
        return Ok((ValidationOutcome::AwaitingQuorum, Some(task.clone()), None));
    }

    let results = db.get_task_results(&task.task_id)?;
    let count = results.len() as u8;

    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for r in &results {
        *counts.entry(r.sha256.as_str()).or_insert(0) += 1;
    }

    let majority_threshold = (task.redundancy_factor as usize) / 2;

    if count < task.redundancy_factor {
        // B.2 (carry S73): do not zombie-wait when quorum is already
        // mathematically impossible. Acceptance needs some hash with
        // `best_count > majority_threshold`; the most any hash can still reach
        // is its current count plus every result yet to arrive. If even that
        // ceiling cannot clear the threshold, no future vote can rescue the
        // task — reject it terminally now instead of leaving it AwaitingQuorum
        // forever (the redundancy>1 zombie). Only fires for redundancy >= 4,
        // where enough distinct hashes can arrive before the full count.
        let best_now = counts.values().copied().max().unwrap_or(0);
        let remaining = (task.redundancy_factor - count) as usize;
        if best_now + remaining <= majority_threshold {
            db.update_task_status(&task.task_id, TaskStatus::Rejected, now)?;
            tracing::warn!(
                task_id = %task.task_id,
                results = count,
                required = task.redundancy_factor,
                distinct_hashes = counts.len(),
                "build quorum impossible (no hash can reach majority) — rejected early"
            );
            return Ok((ValidationOutcome::QuorumRejected, Some(task.clone()), None));
        }

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
        return Ok((ValidationOutcome::AwaitingQuorum, Some(task.clone()), None));
    }

    let (best_hash, best_count) = counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(&h, &c)| (h, c))
        .unwrap_or(("", 0));

    if best_count > majority_threshold {
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
            "build quorum reached — pending output guardrail"
        );

        // On the quorum path `best_hash` IS the agreed `result_text`
        // (the `sha256` column holds raw text here, PATTERNS §P53), so
        // it doubles as both the provenance hash and the retrievable
        // text. The output guardrail (Sprint 73 Phase A, D5) runs on this
        // agreed text BEFORE it is persisted by
        // `validate_result_post_guardrail`.
        let pending = PendingResultPersist {
            task_id: task.task_id.clone(),
            worker_id: worker_id.to_string(),
            result_hash: best_hash.to_string(),
            result_text: best_hash.to_string(),
            now,
        };

        Ok((
            ValidationOutcome::Accepted,
            Some(task.clone()),
            Some(pending),
        ))
    } else {
        db.update_task_status(&task.task_id, TaskStatus::Rejected, now)?;
        tracing::warn!(
            task_id = %task.task_id,
            distinct_hashes = counts.len(),
            "build quorum divergence — rejected"
        );
        Ok((ValidationOutcome::QuorumRejected, Some(task.clone()), None))
    }
}

/// Test-only in-process validator harness.
///
/// Gated `#[cfg(test)]` so the guardrail-less `pre + post` composition in
/// [`ResultValidator::validate`] is **not reachable from any production
/// crate** — the invariant "no path persists `result_text` without the
/// output guardrail" is closed at the API level (Sprint 73 Phase A, D5).
/// The two network ingress points run the split explicitly with the
/// guardrail in between (`http.rs` `coordinator_submit_result`,
/// `validator_loop`). Used only by this module's unit tests of the
/// verification + quorum logic.
#[cfg(test)]
pub struct ResultValidator {
    db: CoordinatorDb,
}

#[cfg(test)]
impl ResultValidator {
    pub fn new(db: CoordinatorDb) -> Self {
        Self { db }
    }

    /// In-process validation **and** persistence WITHOUT the output
    /// guardrail — test-only (see [`ResultValidator`]). Exercises the
    /// verification + quorum logic; the guardrail is a daemon-layer
    /// concern covered in `http.rs` / `validator_loop.rs` tests.
    pub fn validate(
        &self,
        entry: &ResultEntry,
    ) -> Result<(ValidationOutcome, Option<TaskRecord>), CoordinatorError> {
        let (outcome, record, pending) = validate_result_pre_guardrail(&self.db, entry)?;
        if let Some(pending) = pending {
            validate_result_post_guardrail(&self.db, &pending)?;
        }
        Ok((outcome, record))
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

    /// In-process pre+post composition used by the verification/quorum
    /// unit tests below. The output guardrail is a daemon-layer concern
    /// exercised in `http.rs` / `validator_loop.rs`; these tests focus on
    /// signature/status/quorum + persistence, so they mirror the
    /// pre-Sprint-73 `validate_result` ergonomics.
    fn validate_result(
        db: &CoordinatorDb,
        entry: &ResultEntry,
    ) -> Result<(ValidationOutcome, Option<TaskRecord>), CoordinatorError> {
        let (outcome, record, pending) = validate_result_pre_guardrail(db, entry)?;
        if let Some(pending) = pending {
            validate_result_post_guardrail(db, &pending)?;
        }
        Ok((outcome, record))
    }

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

        // Sprint 72 Phase D: the accepted single-path result persists the
        // worker's human-readable text retrievably (what the Operator
        // network arm reads back), not just the signature hash.
        let detail = validator
            .db()
            .get_task_result("task-100")
            .expect("get")
            .expect("found");
        assert_eq!(detail.result_text.as_deref(), Some("test output"));
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

        db.set_task_result("task-102", "w1", "r1", "first text", 100)
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
    fn quorum_impossible_before_full_count_rejects_early() {
        // B.2 (carry S73): redundancy=4 needs a hash with count > 2 (i.e. >= 3)
        // agreeing. Three DISTINCT hashes arrive (1 each) with one slot left: the
        // best any hash can still reach is 1 + 1 = 2, which can never exceed the
        // majority threshold (2). Quorum is already impossible, so the task is
        // rejected at the 3rd result — NOT left zombie in AwaitingQuorum.
        let db = setup_build_task("build-imposs", 4);
        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let w3 = KeyPair::generate();

        let (o1, _) =
            validate_result(&db, &make_build_result("build-imposs", &w1, "hash_a")).expect("v1");
        assert_eq!(o1, ValidationOutcome::AwaitingQuorum);
        let (o2, _) =
            validate_result(&db, &make_build_result("build-imposs", &w2, "hash_b")).expect("v2");
        assert_eq!(o2, ValidationOutcome::AwaitingQuorum, "2/4 still possible");

        let (o3, _) =
            validate_result(&db, &make_build_result("build-imposs", &w3, "hash_c")).expect("v3");
        assert_eq!(
            o3,
            ValidationOutcome::QuorumRejected,
            "3 distinct hashes + 1 slot left = quorum impossible"
        );
        let task = db.get_task("build-imposs").expect("get").expect("found");
        assert_eq!(task.status, TaskStatus::Rejected, "terminal, not zombie");
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

    // -----------------------------------------------------------
    // Sprint 76 Phase D — quorum trust contract locked (validator INCHANGE)
    // -----------------------------------------------------------

    /// Verrou: the quorum's trust contract is UNCHANGED by Phase D —
    /// `validate_quorum_pre_guardrail` counts ONE vote per worker and
    /// accepts only on a strict majority of byte-identical `result_text`.
    /// This is the invariant the Phase D result-sync dedup fix MIRRORS at
    /// the bridge: the bridge now forwards one entry per distinct worker
    /// (so a redundancy>1 quorum can form), and the validator's own
    /// per-`(worker, task)` dedup guarantees a single worker still cannot
    /// manufacture a quorum by submitting twice. The function body itself
    /// stays diff-empty this phase (verified by
    /// `git diff --stat validator.rs`).
    #[test]
    fn validator_quorum_unchanged() {
        // A single worker cannot self-inflate a redundancy=2 quorum by
        // resubmitting the same result: the (worker, task) dedup keeps it
        // at one vote → AwaitingQuorum, never Accepted.
        let db = setup_build_task("quorum-self-inflate", 2);
        let solo = KeyPair::generate();
        let r = make_build_result("quorum-self-inflate", &solo, "lonely greedy output");

        let (o1, _) = validate_result(&db, &r).expect("v1");
        assert_eq!(o1, ValidationOutcome::AwaitingQuorum);
        let (o2, _) = validate_result(&db, &r).expect("v2 (same worker, same result)");
        assert_eq!(
            o2,
            ValidationOutcome::AwaitingQuorum,
            "one worker resubmitting must NOT manufacture a quorum"
        );
        let solo_task = db
            .get_task("quorum-self-inflate")
            .expect("get")
            .expect("found");
        assert_eq!(solo_task.status, TaskStatus::AwaitingQuorum);
        assert_eq!(
            db.get_task_results("quorum-self-inflate")
                .expect("results")
                .len(),
            1,
            "a duplicate (worker, task) result is deduped to a single vote"
        );

        // Two DISTINCT workers agreeing reach the strict majority →
        // Accepted, with the agreed text as the canonical result. The
        // exact-match boundary is intact.
        let db2 = setup_build_task("quorum-two-distinct", 2);
        let agreed = "stable greedy answer";
        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let (a1, _) = validate_result(&db2, &make_build_result("quorum-two-distinct", &w1, agreed))
            .expect("a1");
        assert_eq!(a1, ValidationOutcome::AwaitingQuorum);
        let (a2, _) = validate_result(&db2, &make_build_result("quorum-two-distinct", &w2, agreed))
            .expect("a2");
        assert_eq!(a2, ValidationOutcome::Accepted);
        let done = db2
            .get_task("quorum-two-distinct")
            .expect("get")
            .expect("found");
        assert_eq!(done.status, TaskStatus::Completed);
        assert_eq!(done.result_hash.as_deref(), Some(agreed));
    }

    // -----------------------------------------------------------
    // Sprint 73 Phase A — guardrail-before-persist (D5)
    // -----------------------------------------------------------

    #[test]
    fn quorum_guardrail_runs_on_agreed_text() {
        // redundancy 2: two honest workers agree on a text that carries an
        // invisible character. The pre phase reaches quorum and returns the
        // AGREED text as the pending persist candidate; the output guardrail
        // trips on it, so the caller must skip persistence. Proves the
        // guardrail gates the quorum path on the agreed text, not on a raw
        // single submission (Sprint 73 Phase A, D5).
        let db = setup_build_task("quorum-gr", 2);
        let agreed = "agreed\u{200B}text";
        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();

        let (o1, _, p1) =
            validate_result_pre_guardrail(&db, &make_build_result("quorum-gr", &w1, agreed))
                .expect("v1");
        assert_eq!(o1, ValidationOutcome::AwaitingQuorum);
        assert!(p1.is_none(), "awaiting quorum yields no persist candidate");

        let (o2, _, p2) =
            validate_result_pre_guardrail(&db, &make_build_result("quorum-gr", &w2, agreed))
                .expect("v2");
        assert_eq!(o2, ValidationOutcome::Accepted);
        let pending = p2.expect("quorum reached returns a pending persist");
        assert_eq!(
            pending.result_text, agreed,
            "the guardrail must run on the AGREED text"
        );

        // Same guardrail the daemon runs: the agreed text trips it.
        let ctx = crate::guardrails::GuardrailContext {
            system_prompt: "",
            user_prompt: "",
            model_output: &pending.result_text,
        };
        let gr = crate::guardrails::default_output_chain().run(&ctx);
        assert!(
            !gr.passed,
            "agreed text with an invisible char must trip the guardrail"
        );

        // The caller skips `validate_result_post_guardrail` on a tripwire,
        // so nothing is persisted and the task is not completed.
        let task = db.get_task("quorum-gr").expect("get").expect("found");
        assert_ne!(task.status, TaskStatus::Completed);
        assert!(
            db.get_task_result("quorum-gr")
                .expect("get")
                .expect("found")
                .result_text
                .is_none(),
            "no retrievable text before the guardrail clears it"
        );
    }
}
