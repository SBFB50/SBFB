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

    let entry = KudosEntry {
        entry_id,
        worker_node_id: worker_node_id.to_string(),
        task_id: task_id.to_string(),
        project_id: project_id.to_string(),
        amount: tokens_generated,
        created_at: now,
        prev_hash: String::new(),
        entry_hash: String::new(),
    };

    db.insert_kudos(&entry)?;

    tracing::info!(
        project_id,
        worker = &worker_node_id[..worker_node_id.len().min(16)],
        tokens = tokens_generated,
        "kudos credited"
    );

    Ok(())
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
}
