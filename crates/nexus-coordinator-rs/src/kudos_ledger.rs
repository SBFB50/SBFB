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
const KUDOS_EMA_ALPHA: f64 = 0.97;

pub fn log_utility(tokens: u64) -> u64 {
    (KUDOS_LOG_SCALE * (1.0 + tokens as f64).log2()).max(1.0) as u64
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
) -> Result<(), CoordinatorError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let entry_id = format!("{task_id}-{now}");
    let prev_hash = db
        .get_last_entry_hash(project_id)?
        .unwrap_or_else(|| "genesis".to_string());

    let mut entry = KudosEntry {
        entry_id,
        worker_node_id: worker_node_id.to_string(),
        task_id: task_id.to_string(),
        project_id: project_id.to_string(),
        amount: log_utility(tokens_generated),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_increases_total() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "task-1", 10).expect("credit 1");
        let after_one = db.get_project_kudos_total("proj-1").expect("total");
        assert!(after_one > 0, "first credit must produce positive amount");
        credit(&db, "proj-1", "worker-a", "task-2", 20).expect("credit 2");
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
        credit(&db, "proj-1", "worker-a", "t1", 50).expect("c1");
        credit(&db, "proj-1", "worker-b", "t2", 30).expect("c2");
        credit(&db, "proj-1", "worker-a", "t3", 20).expect("c3");

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
        credit(&db, "proj-1", "worker-a", "task-1", 10).expect("credit");
        let entries = db.get_project_entries("proj-1").expect("entries");
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].entry_hash.is_empty(), "entry_hash must be set");
        assert_eq!(entries[0].entry_hash.len(), 64, "BLAKE3 hex = 64 chars");
    }

    #[test]
    fn credit_genesis_hash() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "task-1", 10).expect("credit");
        let entries = db.get_project_entries("proj-1").expect("entries");
        assert_eq!(entries[0].prev_hash, "genesis");
    }

    #[test]
    fn credit_chains_prev_hash() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "task-1", 10).expect("c1");
        credit(&db, "proj-1", "worker-a", "task-2", 20).expect("c2");
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
        credit(&db, "proj-1", "worker-a", "t1", 10).expect("c1");
        credit(&db, "proj-1", "worker-a", "t2", 20).expect("c2");
        credit(&db, "proj-1", "worker-a", "t3", 30).expect("c3");
        assert!(verify_chain(&db, "proj-1").expect("verify"));
    }

    #[test]
    fn verify_chain_tampered() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "t1", 10).expect("c1");
        credit(&db, "proj-1", "worker-a", "t2", 20).expect("c2");
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
        credit(&db, "proj-a", "worker-a", "t1", 10).expect("c1");
        credit(&db, "proj-b", "worker-a", "t2", 20).expect("c2");

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
        credit(&db, "proj-1", "worker-a", "t1", 100).expect("c1");

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
        credit(&db, "proj-1", "worker-a", "t1", 50).expect("c1");
        credit(&db, "proj-1", "worker-a", "t2", 100).expect("c2");
        credit(&db, "proj-1", "worker-a", "t3", 200).expect("c3");
        assert!(
            verify_chain(&db, "proj-1").expect("verify"),
            "hash chain must be valid after log-utility credits"
        );
    }

    #[test]
    fn effective_score_empty() {
        assert_eq!(effective_score(&[], 1_000_000), 0);
    }
}
