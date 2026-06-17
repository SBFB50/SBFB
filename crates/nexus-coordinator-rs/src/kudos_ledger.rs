// SPDX-License-Identifier: AGPL-3.0-or-later
//! Kudos ledger — credits reputation tokens after result validation.
//!
//! Kudos are non-monetary, non-transferable reputation scores tied to
//! Ed25519 worker identities (Day 0 decision #7). This module exposes
//! `credit()` (called by the result handler after `Accepted`) and
//! read-only queries for the HTTP API.

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;
use crate::types::KudosEntry;

// log2 chosen over ln for informatique intuition (doublement = +1000 kudos).
// Constant factor vs ln is absorbed by KUDOS_LOG_SCALE.
const KUDOS_LOG_SCALE: f64 = 1000.0;
// Half-life ~23 days at 1 entry/day. Pre-launch frequency is low;
// alpha=0.95 (S21 research) decays too fast for occasional contributors.
pub const KUDOS_EMA_ALPHA: f64 = 0.97;

pub fn log_utility(tokens: u64) -> u64 {
    (KUDOS_LOG_SCALE * (1.0 + tokens as f64).log2()).max(1.0) as u64
}

// Anti-gaming plausibility ceiling on self-declared token counts
// (Sprint 76 Phase E, D4-Q; cf. THREAT_MODEL §15.3). `tokens_generated`
// and `generation_time_ms` both live in the SAME signed result payload
// and sit OUTSIDE the quorum (the validator compares only `result_text`),
// so a solo worker could otherwise farm kudos by declaring an absurd token
// count. This bound is a plausibility check (BOINC `wu.rsc_fpops_bound`
// analogue), NOT an anti-Sybil attestation — an adversary who controls the
// payload can satisfy it by forging both fields consistently. The ceiling
// is deliberately far above any real decode throughput (real rates are at
// most a few hundred tokens/sec, i.e. << 1 token/ms); it bounds absurdity
// ("1e9 tokens in 5 ms"), not a real rate. `log_utility` already compresses
// the marginal incentive to <10x but does NOT cap the absolute value — this
// ceiling closes that residual leak before `log_utility` is applied.
pub const TOKENS_PER_MS_CEILING: u64 = 1_000;

/// Clamp a self-declared `tokens_generated` to a plausible maximum given
/// the self-declared `generation_time_ms`. The time is floored to 1 ms so a
/// sub-millisecond (rounded-to-zero) reply still credits up to the per-ms
/// ceiling rather than collapsing to zero kudos. `saturating_mul` guards
/// against overflow on an absurd time claim.
pub fn sanity_bounded_tokens(tokens_generated: u64, generation_time_ms: u64) -> u64 {
    let ceiling = TOKENS_PER_MS_CEILING.saturating_mul(generation_time_ms.max(1));
    tokens_generated.min(ceiling)
}

#[derive(serde::Serialize)]
struct HashableKudosEntry<'a> {
    entry_id: &'a str,
    worker_node_id: &'a str,
    task_id: &'a str,
    project_id: &'a str,
    amount: u64,
    created_at: u64,
    prev_hash: &'a str,
}

fn compute_entry_hash(entry: &KudosEntry, prev_hash: &str) -> String {
    let hashable = HashableKudosEntry {
        entry_id: &entry.entry_id,
        worker_node_id: &entry.worker_node_id,
        task_id: &entry.task_id,
        project_id: &entry.project_id,
        amount: entry.amount,
        created_at: entry.created_at,
        prev_hash,
    };
    let canonical = nexus_core_rs::canonical_bytes(&hashable, nexus_core_rs::DOMAIN_KUDOS_V1)
        .expect("KudosEntry serialization cannot fail");
    let hash = blake3::hash(&canonical);
    hex::encode(hash.as_bytes())
}

pub fn credit(
    db: &CoordinatorDb,
    project_id: &str,
    worker_node_id: &str,
    task_id: &str,
    tokens_generated: u64,
    generation_time_ms: u64,
) -> Result<(), CoordinatorError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let entry_id = format!("{task_id}-{now}");
    let prev_hash = db
        .get_last_entry_hash(project_id)?
        .unwrap_or_else(|| "genesis".to_string());

    // D4-Q (Sprint 76 Phase E): clamp the self-declared token count to a
    // plausible maximum BEFORE `log_utility`, closing the absolute-value
    // leak that `log_utility` alone leaves open.
    let bounded_tokens = sanity_bounded_tokens(tokens_generated, generation_time_ms);

    let mut entry = KudosEntry {
        entry_id,
        worker_node_id: worker_node_id.to_string(),
        task_id: task_id.to_string(),
        project_id: project_id.to_string(),
        amount: log_utility(bounded_tokens),
        created_at: now,
        prev_hash: prev_hash.clone(),
        entry_hash: String::new(),
    };

    entry.entry_hash = compute_entry_hash(&entry, &prev_hash);

    db.insert_kudos(&entry)?;

    tracing::info!(
        project_id,
        worker = &worker_node_id[..worker_node_id.len().min(16)],
        tokens = tokens_generated,
        bounded_tokens,
        generation_time_ms,
        "kudos credited"
    );

    Ok(())
}

pub fn verify_chain(db: &CoordinatorDb, project_id: &str) -> Result<bool, CoordinatorError> {
    let entries = db.get_project_entries(project_id)?;
    let mut expected_prev = "genesis".to_string();

    for entry in &entries {
        if entry.prev_hash != expected_prev {
            return Ok(false);
        }
        let recomputed = compute_entry_hash(entry, &entry.prev_hash);
        if entry.entry_hash != recomputed {
            return Ok(false);
        }
        expected_prev = entry.entry_hash.clone();
    }

    Ok(true)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectKudos {
    pub project_id: String,
    pub total: u64,
    pub contributors: Vec<ContributorKudos>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContributorKudos {
    pub worker_node_id: String,
    pub total: u64,
}

pub fn effective_score(entries: &[KudosEntry], now_secs: u64) -> u64 {
    entries
        .iter()
        .map(|e| {
            let age_days = now_secs.saturating_sub(e.created_at) / 86400;
            (e.amount as f64 * KUDOS_EMA_ALPHA.powi(age_days as i32)) as u64
        })
        .sum()
}

pub fn get_project_kudos(
    db: &CoordinatorDb,
    project_id: &str,
    now_secs: u64,
) -> Result<ProjectKudos, CoordinatorError> {
    let entries = db.get_project_entries(project_id)?;
    let mut worker_map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

    for entry in &entries {
        let age_days = now_secs.saturating_sub(entry.created_at) / 86400;
        let eff = (entry.amount as f64 * KUDOS_EMA_ALPHA.powi(age_days as i32)) as u64;
        *worker_map.entry(entry.worker_node_id.clone()).or_default() += eff;
    }

    let total: u64 = worker_map.values().sum();
    let mut contributors: Vec<ContributorKudos> = worker_map
        .into_iter()
        .map(|(worker_node_id, total)| ContributorKudos {
            worker_node_id,
            total,
        })
        .collect();
    contributors.sort_by_key(|c| std::cmp::Reverse(c.total));

    Ok(ProjectKudos {
        project_id: project_id.to_string(),
        total,
        contributors,
    })
}

/// A contributor's standing on a single project (Sprint 76 Phase E, D4).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContributorProject {
    pub project_id: String,
    /// EMA-decayed kudos for this node on this project.
    pub effective_total: u64,
    /// Number of quorum-validated kudos lines on this project.
    pub tasks_served: u64,
}

/// A single node's contribution standing across ALL projects (Sprint 76
/// Phase E, D4). This is a SECOND aggregation view over the existing kudos
/// ledger — keyed on `worker_node_id` instead of `project_id` — and reuses
/// [`effective_score`] verbatim (same EMA `alpha = 0.97`). It is a self-view
/// per node, NOT a network-wide ranking (the EigenTrust global-ranking model
/// is rejected; cf. design_review D4).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContributorSummary {
    pub worker_node_id: String,
    /// EMA-decayed kudos summed across every project this node served.
    pub effective_total: u64,
    /// Total quorum-validated kudos lines credited to this node (= tasks
    /// served). One line per accepted result; uniqueness of `(task_id,
    /// worker)` is enforced UPSTREAM at result acceptance (status-guard +
    /// `UNIQUE(task_id, worker_id)` on `task_results`), so a node cannot
    /// inflate this count by re-crediting the same task.
    pub tasks_served: u64,
    /// Per-project breakdown, sorted by `effective_total` descending.
    pub per_project: Vec<ContributorProject>,
}

/// Aggregate a single node's kudos across all projects (Sprint 76 Phase E).
///
/// Mirror of [`get_project_kudos`] keyed on `worker_node_id`. Reads the
/// node's entries through the `idx_kudos_worker` index (pre-existing,
/// db.rs migration M0) and applies [`effective_score`] EXACTLY — no new
/// scoring formula. `tasks_served` counts the credited (quorum-validated)
/// ledger lines.
pub fn get_contributor_summary(
    db: &CoordinatorDb,
    worker_node_id: &str,
    now_secs: u64,
) -> Result<ContributorSummary, CoordinatorError> {
    let entries = db.get_worker_entries(worker_node_id)?;

    let mut by_project: std::collections::HashMap<String, Vec<KudosEntry>> =
        std::collections::HashMap::new();
    for entry in entries {
        by_project
            .entry(entry.project_id.clone())
            .or_default()
            .push(entry);
    }

    let mut per_project: Vec<ContributorProject> = Vec::with_capacity(by_project.len());
    let mut effective_total = 0u64;
    let mut tasks_served = 0u64;

    for (project_id, project_entries) in by_project {
        // `effective_score` is order-independent (per-entry decay summed),
        // so the per-project sum equals the all-projects score.
        let eff = effective_score(&project_entries, now_secs);
        let count = project_entries.len() as u64;
        effective_total += eff;
        tasks_served += count;
        per_project.push(ContributorProject {
            project_id,
            effective_total: eff,
            tasks_served: count,
        });
    }

    // Deterministic order: effective score desc, project_id asc as tiebreak.
    per_project.sort_by(|a, b| {
        b.effective_total
            .cmp(&a.effective_total)
            .then_with(|| a.project_id.cmp(&b.project_id))
    });

    Ok(ContributorSummary {
        worker_node_id: worker_node_id.to_string(),
        effective_total,
        tasks_served,
        per_project,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_increases_total() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "task-1", 10, 1_000).expect("credit 1");
        let after_one = db.get_project_kudos_total("proj-1").expect("total");
        assert!(after_one > 0, "first credit must produce positive amount");
        credit(&db, "proj-1", "worker-a", "task-2", 20, 1_000).expect("credit 2");
        let after_two = db.get_project_kudos_total("proj-1").expect("total");
        assert!(after_two > after_one, "second credit must increase total");
    }

    #[test]
    fn get_project_kudos_empty() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let kudos = get_project_kudos(&db, "nonexistent", now).expect("get");
        assert_eq!(kudos.total, 0);
        assert!(kudos.contributors.is_empty());
    }

    #[test]
    fn get_project_kudos_with_contributors() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "t1", 50, 1_000).expect("c1");
        credit(&db, "proj-1", "worker-b", "t2", 30, 1_000).expect("c2");
        credit(&db, "proj-1", "worker-a", "t3", 20, 1_000).expect("c3");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let kudos = get_project_kudos(&db, "proj-1", now).expect("get");
        assert!(kudos.total > 0);
        assert_eq!(kudos.contributors.len(), 2);
        let worker_a = kudos
            .contributors
            .iter()
            .find(|c| c.worker_node_id == "worker-a")
            .unwrap();
        let worker_b = kudos
            .contributors
            .iter()
            .find(|c| c.worker_node_id == "worker-b")
            .unwrap();
        assert!(
            worker_a.total > worker_b.total,
            "worker-a (70 tokens) > worker-b (30 tokens)"
        );
    }

    #[test]
    fn credit_sets_entry_hash() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "task-1", 10, 1_000).expect("credit");
        let entries = db.get_project_entries("proj-1").expect("entries");
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].entry_hash.is_empty(), "entry_hash must be set");
        assert_eq!(entries[0].entry_hash.len(), 64, "BLAKE3 hex = 64 chars");
    }

    #[test]
    fn credit_genesis_hash() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "task-1", 10, 1_000).expect("credit");
        let entries = db.get_project_entries("proj-1").expect("entries");
        assert_eq!(entries[0].prev_hash, "genesis");
    }

    #[test]
    fn credit_chains_prev_hash() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "task-1", 10, 1_000).expect("c1");
        credit(&db, "proj-1", "worker-a", "task-2", 20, 1_000).expect("c2");
        let entries = db.get_project_entries("proj-1").expect("entries");
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[1].prev_hash, entries[0].entry_hash,
            "second entry prev_hash must equal first entry_hash"
        );
    }

    #[test]
    fn verify_chain_valid() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "t1", 10, 1_000).expect("c1");
        credit(&db, "proj-1", "worker-a", "t2", 20, 1_000).expect("c2");
        credit(&db, "proj-1", "worker-a", "t3", 30, 1_000).expect("c3");
        assert!(verify_chain(&db, "proj-1").expect("verify"));
    }

    #[test]
    fn verify_chain_tampered() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "t1", 10, 1_000).expect("c1");
        credit(&db, "proj-1", "worker-a", "t2", 20, 1_000).expect("c2");
        db.conn()
            .execute(
                "UPDATE kudos SET entry_hash = 'tampered' WHERE task_id = 't1'",
                [],
            )
            .expect("tamper");
        assert!(!verify_chain(&db, "proj-1").expect("verify"));
    }

    #[test]
    fn cross_project_chains_independent() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-a", "worker-a", "t1", 10, 1_000).expect("c1");
        credit(&db, "proj-b", "worker-a", "t2", 20, 1_000).expect("c2");

        let entries_a = db.get_project_entries("proj-a").expect("a");
        let entries_b = db.get_project_entries("proj-b").expect("b");
        assert_eq!(entries_a[0].prev_hash, "genesis");
        assert_eq!(entries_b[0].prev_hash, "genesis");
        assert_ne!(entries_a[0].entry_hash, entries_b[0].entry_hash);
    }

    #[test]
    fn log_utility_compression() {
        let low = log_utility(1);
        let high = log_utility(100);
        assert!(low > 0, "log_utility(1) must be positive");
        assert!(high > low, "more tokens = more kudos");
        let ratio = high as f64 / low as f64;
        assert!(
            ratio < 10.0,
            "100x tokens must compress to < 10x kudos (got {ratio:.1}x)"
        );
    }

    #[test]
    fn log_utility_minimum() {
        assert!(
            log_utility(0) >= 1,
            "tokens=0 must produce at least 1 kudos"
        );
    }

    #[test]
    fn effective_score_decays_with_age() {
        let now = 10_000_000u64;
        let recent = KudosEntry {
            entry_id: "e1".into(),
            worker_node_id: "w".into(),
            task_id: "t1".into(),
            project_id: "p".into(),
            amount: 1000,
            created_at: now - 86400,
            prev_hash: "genesis".into(),
            entry_hash: "h1".into(),
        };
        let old = KudosEntry {
            entry_id: "e2".into(),
            worker_node_id: "w".into(),
            task_id: "t2".into(),
            project_id: "p".into(),
            amount: 1000,
            created_at: now - 86400 * 90,
            prev_hash: "h1".into(),
            entry_hash: "h2".into(),
        };
        let score_recent = effective_score(&[recent], now);
        let score_old = effective_score(&[old], now);
        assert!(
            score_recent > score_old,
            "recent entry ({score_recent}) must score higher than 90-day old ({score_old})"
        );
    }

    #[test]
    fn effective_score_no_decay_fresh() {
        let now = 1_000_000u64;
        let entry = KudosEntry {
            entry_id: "e1".into(),
            worker_node_id: "w".into(),
            task_id: "t1".into(),
            project_id: "p".into(),
            amount: 5000,
            created_at: now,
            prev_hash: "genesis".into(),
            entry_hash: "h1".into(),
        };
        let score = effective_score(&[entry], now);
        assert_eq!(
            score, 5000,
            "fresh entry must have full score (alpha^0 = 1)"
        );
    }

    #[test]
    fn get_project_kudos_uses_ema() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "t1", 100, 1_000).expect("c1");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let kudos_now = get_project_kudos(&db, "proj-1", now).expect("now");
        let kudos_future = get_project_kudos(&db, "proj-1", now + 86400 * 30).expect("future");
        assert!(
            kudos_now.total > kudos_future.total,
            "score must decrease over 30 days ({} vs {})",
            kudos_now.total,
            kudos_future.total
        );
    }

    #[test]
    fn log_utility_preserves_chain() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "t1", 50, 1_000).expect("c1");
        credit(&db, "proj-1", "worker-a", "t2", 100, 1_000).expect("c2");
        credit(&db, "proj-1", "worker-a", "t3", 200, 1_000).expect("c3");
        assert!(
            verify_chain(&db, "proj-1").expect("verify"),
            "hash chain must be valid after log-utility credits"
        );
    }

    #[test]
    fn effective_score_empty() {
        assert_eq!(effective_score(&[], 1_000_000), 0);
    }

    // ---- Sprint 76 Phase E (D4-Q) — anti-gaming sanity bound ----

    #[test]
    fn sanity_bound_clamps_implausible_token_claims() {
        // Plausible: 500 tokens in 5 s (~100 tok/s) — far below the ceiling,
        // so the bound leaves an honest result untouched.
        assert_eq!(sanity_bounded_tokens(500, 5_000), 500);
        // Absurd: 1e9 tokens claimed in 5 ms — clamped to 5 * CEILING.
        let bounded = sanity_bounded_tokens(1_000_000_000, 5);
        assert_eq!(bounded, 5 * TOKENS_PER_MS_CEILING);
        assert!(bounded < 1_000_000_000, "the absurd claim must be reduced");
        // The clamp meaningfully reduces the credited kudos (closes the
        // absolute-value leak that log_utility alone leaves open).
        assert!(
            log_utility(bounded) < log_utility(1_000_000_000),
            "clamped kudos must be strictly lower than the unbounded claim"
        );
        // A rounded-to-zero generation time floors to the 1 ms ceiling
        // rather than collapsing the credit to zero.
        assert_eq!(sanity_bounded_tokens(10_000, 0), TOKENS_PER_MS_CEILING);
    }

    #[test]
    fn sanity_bound_preserves_honest_large_credit() {
        // Regression guard (the worker now stamps a REAL generation_time_ms):
        // a large honest credit with a plausible duration must pass through
        // UNCLAMPED. 5000 tokens in 30 s (~167 tok/s) is well under the
        // ceiling (1000 tok/ms * 30_000 ms), so the bound is a no-op.
        assert_eq!(sanity_bounded_tokens(5_000, 30_000), 5_000);
        // Even a brisk 2000 tokens in 2 s is preserved.
        assert_eq!(sanity_bounded_tokens(2_000, 2_000), 2_000);
        // The credited kudos reflect the FULL token count, not a flat ceiling.
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "p", "w", "t-big", 5_000, 30_000).expect("big");
        let entries = db.get_project_entries("p").expect("entries");
        assert_eq!(entries[0].amount, log_utility(5_000));
        assert!(
            entries[0].amount > log_utility(TOKENS_PER_MS_CEILING),
            "an honest large credit must out-score the flat clamp value"
        );
    }

    #[test]
    fn credit_applies_sanity_bound_to_amount() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        // Honest credit: 100 tokens in 1 s.
        credit(&db, "p", "w", "t-honest", 100, 1_000).expect("honest");
        // Gamed credit: 1e9 tokens claimed in 5 ms — must be clamped so it
        // does NOT dwarf the honest entry's kudos.
        credit(&db, "p", "w", "t-gamed", 1_000_000_000, 5).expect("gamed");
        let entries = db.get_project_entries("p").expect("entries");
        let honest = entries.iter().find(|e| e.task_id == "t-honest").unwrap();
        let gamed = entries.iter().find(|e| e.task_id == "t-gamed").unwrap();
        // Unbounded, the gamed amount would be log_utility(1e9) >> honest;
        // bounded, it can only reach log_utility(5 * CEILING).
        assert_eq!(gamed.amount, log_utility(5 * TOKENS_PER_MS_CEILING));
        assert!(
            gamed.amount < log_utility(1_000_000_000),
            "gamed entry must be clamped below its unbounded value"
        );
        assert!(honest.amount > 0);
    }

    // ---- Sprint 76 Phase E (D4) — contributor summary view ----

    #[test]
    fn get_contributor_summary_aggregates_ema() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        // worker-a serves two projects; worker-b serves one.
        credit(&db, "proj-1", "worker-a", "t1", 100, 1_000).expect("c1");
        credit(&db, "proj-2", "worker-a", "t2", 50, 1_000).expect("c2");
        credit(&db, "proj-1", "worker-b", "t3", 80, 1_000).expect("c3");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let summary = get_contributor_summary(&db, "worker-a", now).expect("summary");
        assert_eq!(summary.worker_node_id, "worker-a");
        // The contributor view reuses effective_score; the cross-project
        // total equals the sum of the two per-project effective scores.
        let per_sum: u64 = summary.per_project.iter().map(|p| p.effective_total).sum();
        assert_eq!(summary.effective_total, per_sum);
        assert_eq!(summary.per_project.len(), 2, "worker-a served 2 projects");
        // Same EMA as get_project_kudos: the score decays over time.
        let future = get_contributor_summary(&db, "worker-a", now + 86400 * 30).expect("future");
        assert!(
            summary.effective_total > future.effective_total,
            "contributor score must decay over 30 days like the project view"
        );
    }

    #[test]
    fn contributor_summary_counts_tasks_served() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "t1", 10, 1_000).expect("c1");
        credit(&db, "proj-1", "worker-a", "t2", 10, 1_000).expect("c2");
        credit(&db, "proj-2", "worker-a", "t3", 10, 1_000).expect("c3");
        credit(&db, "proj-1", "worker-b", "t4", 10, 1_000).expect("c4");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let summary = get_contributor_summary(&db, "worker-a", now).expect("summary");
        // tasks_served = number of credited (quorum-validated) ledger lines.
        assert_eq!(summary.tasks_served, 3);
        let p1 = summary
            .per_project
            .iter()
            .find(|p| p.project_id == "proj-1")
            .unwrap();
        assert_eq!(p1.tasks_served, 2, "2 of worker-a's tasks were on proj-1");
        // worker-b's line is not attributed to worker-a.
        let other = get_contributor_summary(&db, "worker-b", now).expect("summary b");
        assert_eq!(other.tasks_served, 1);
    }

    #[test]
    fn get_contributor_summary_empty() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let summary = get_contributor_summary(&db, "nobody", 1_000_000).expect("summary");
        assert_eq!(summary.effective_total, 0);
        assert_eq!(summary.tasks_served, 0);
        assert!(summary.per_project.is_empty());
    }
}
