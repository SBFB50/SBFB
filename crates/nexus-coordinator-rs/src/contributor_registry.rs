// SPDX-License-Identifier: AGPL-3.0-or-later
//! Contributor attestation registry — Couche 2 Sybil gate
//! (Sprint 41 Phase B, port of contributor_registry.py S22).

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorRecord {
    pub project_id: String,
    pub contributor_node_id: String,
    pub first_deploy_ts: i64,
    pub commit_sha: String,
    pub repo_url: String,
    pub coord_sig_hex: String,
    pub attestation_json: String,
}

pub struct ContributorRegistry<'a> {
    db: &'a CoordinatorDb,
}

impl<'a> ContributorRegistry<'a> {
    pub fn new(db: &'a CoordinatorDb) -> Self {
        Self { db }
    }

    pub fn record(
        &self,
        project_id: &str,
        contributor_node_id: &str,
        commit_sha: &str,
        repo_url: &str,
        coord_sig_hex: &str,
        attestation_json: &str,
    ) -> Result<ContributorRecord, CoordinatorError> {
        if let Some(existing) = self.get(project_id, contributor_node_id)? {
            return Ok(existing);
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.db.conn().execute(
            "INSERT OR IGNORE INTO contributor_attestations \
             (project_id, contributor_node_id, first_deploy_ts, commit_sha, \
              repo_url, coord_sig_hex, attestation_json, recorded_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                project_id,
                contributor_node_id,
                now,
                commit_sha,
                repo_url,
                coord_sig_hex,
                attestation_json,
                now,
            ],
        )?;
        self.get(project_id, contributor_node_id)?
            .ok_or_else(|| CoordinatorError::Validation("record insert failed".into()))
    }

    pub fn get(
        &self,
        project_id: &str,
        contributor_node_id: &str,
    ) -> Result<Option<ContributorRecord>, CoordinatorError> {
        let result = self
            .db
            .conn()
            .query_row(
                "SELECT project_id, contributor_node_id, first_deploy_ts, \
                 commit_sha, repo_url, coord_sig_hex, attestation_json \
                 FROM contributor_attestations \
                 WHERE project_id = ?1 AND contributor_node_id = ?2",
                rusqlite::params![project_id, contributor_node_id],
                |row| {
                    Ok(ContributorRecord {
                        project_id: row.get(0)?,
                        contributor_node_id: row.get(1)?,
                        first_deploy_ts: row.get(2)?,
                        commit_sha: row.get(3)?,
                        repo_url: row.get(4)?,
                        coord_sig_hex: row.get(5)?,
                        attestation_json: row.get(6)?,
                    })
                },
            )
            .ok();
        Ok(result)
    }

    pub fn is_verified_contributor(
        &self,
        project_id: &str,
        contributor_node_id: &str,
    ) -> Result<bool, CoordinatorError> {
        Ok(self.get(project_id, contributor_node_id)?.is_some())
    }

    pub fn list_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ContributorRecord>, CoordinatorError> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT project_id, contributor_node_id, first_deploy_ts, \
             commit_sha, repo_url, coord_sig_hex, attestation_json \
             FROM contributor_attestations \
             WHERE project_id = ?1 \
             ORDER BY first_deploy_ts ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id], |row| {
            Ok(ContributorRecord {
                project_id: row.get(0)?,
                contributor_node_id: row.get(1)?,
                first_deploy_ts: row.get(2)?,
                commit_sha: row.get(3)?,
                repo_url: row.get(4)?,
                coord_sig_hex: row.get(5)?,
                attestation_json: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> CoordinatorDb {
        CoordinatorDb::open_in_memory().unwrap()
    }

    #[test]
    fn record_and_get() {
        let db = make_db();
        let reg = ContributorRegistry::new(&db);
        let rec = reg
            .record(
                "proj1",
                "node1",
                "abc123",
                "https://example.com",
                "sig1",
                "{}",
            )
            .unwrap();
        assert_eq!(rec.project_id, "proj1");
        assert_eq!(rec.contributor_node_id, "node1");
        let fetched = reg.get("proj1", "node1").unwrap().unwrap();
        assert_eq!(fetched.commit_sha, "abc123");
    }

    #[test]
    fn record_idempotent() {
        let db = make_db();
        let reg = ContributorRegistry::new(&db);
        let r1 = reg
            .record("proj1", "node1", "sha1", "url1", "sig1", "{}")
            .unwrap();
        let r2 = reg
            .record("proj1", "node1", "sha2", "url2", "sig2", "{}")
            .unwrap();
        assert_eq!(r1.first_deploy_ts, r2.first_deploy_ts);
        assert_eq!(r1.commit_sha, r2.commit_sha);
    }

    #[test]
    fn is_verified_contributor() {
        let db = make_db();
        let reg = ContributorRegistry::new(&db);
        assert!(!reg.is_verified_contributor("proj1", "node1").unwrap());
        reg.record("proj1", "node1", "sha", "url", "sig", "{}")
            .unwrap();
        assert!(reg.is_verified_contributor("proj1", "node1").unwrap());
    }

    #[test]
    fn list_for_project() {
        let db = make_db();
        let reg = ContributorRegistry::new(&db);
        reg.record("proj1", "node1", "sha1", "url1", "sig1", "{}")
            .unwrap();
        reg.record("proj1", "node2", "sha2", "url2", "sig2", "{}")
            .unwrap();
        reg.record("proj2", "node3", "sha3", "url3", "sig3", "{}")
            .unwrap();
        let list = reg.list_for_project("proj1").unwrap();
        assert_eq!(list.len(), 2);
    }
}
