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
        amount: tokens_generated,
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

pub fn get_project_kudos(
    db: &CoordinatorDb,
    project_id: &str,
) -> Result<ProjectKudos, CoordinatorError> {
    let total = db.get_project_kudos_total(project_id)?;
    let contributors = db
        .get_project_contributors(project_id)?
        .into_iter()
        .map(|(worker_node_id, total)| ContributorKudos {
            worker_node_id,
            total,
        })
        .collect();

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
        credit(&db, "proj-1", "worker-a", "task-2", 20).expect("credit 2");
        assert_eq!(db.get_project_kudos_total("proj-1").expect("total"), 30);
    }

    #[test]
    fn get_project_kudos_empty() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kudos = get_project_kudos(&db, "nonexistent").expect("get");
        assert_eq!(kudos.total, 0);
        assert!(kudos.contributors.is_empty());
    }

    #[test]
    fn get_project_kudos_with_contributors() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        credit(&db, "proj-1", "worker-a", "t1", 50).expect("c1");
        credit(&db, "proj-1", "worker-b", "t2", 30).expect("c2");
        credit(&db, "proj-1", "worker-a", "t3", 20).expect("c3");

        let kudos = get_project_kudos(&db, "proj-1").expect("get");
        assert_eq!(kudos.total, 100);
        assert_eq!(kudos.contributors.len(), 2);
        assert_eq!(kudos.contributors[0].worker_node_id, "worker-a");
        assert_eq!(kudos.contributors[0].total, 70);
        assert_eq!(kudos.contributors[1].worker_node_id, "worker-b");
        assert_eq!(kudos.contributors[1].total, 30);
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
}
