// SPDX-License-Identifier: AGPL-3.0-or-later
//! Invite minting, tracking, and revocation ledger
//! (Sprint 41 Phase B, port of invite.py S22).

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteRecord {
    pub id: String,
    pub wire: String,
    pub scope: String,
    pub project_id: String,
    pub project_name: String,
    pub expires_at: i64,
    pub max_uses: Option<i64>,
    pub uses_count: i64,
    pub revoked_at: Option<i64>,
    pub note: Option<String>,
    pub created_at: i64,
}

pub struct MintRequest<'b> {
    pub id: &'b str,
    pub wire: &'b str,
    pub scope: &'b str,
    pub project_id: &'b str,
    pub project_name: &'b str,
    pub expires_at: i64,
    pub max_uses: Option<i64>,
    pub note: Option<&'b str>,
}

pub struct InviteLedger<'a> {
    db: &'a CoordinatorDb,
}

impl<'a> InviteLedger<'a> {
    pub fn new(db: &'a CoordinatorDb) -> Self {
        Self { db }
    }

    pub fn mint(&self, req: &MintRequest<'_>) -> Result<InviteRecord, CoordinatorError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.db.conn().execute(
            "INSERT INTO invites \
             (id, wire, scope, project_id, project_name, expires_at, \
              max_uses, uses_count, revoked_at, note, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, NULL, ?8, ?9)",
            rusqlite::params![
                req.id,
                req.wire,
                req.scope,
                req.project_id,
                req.project_name,
                req.expires_at,
                req.max_uses,
                req.note,
                now,
            ],
        )?;
        Ok(InviteRecord {
            id: req.id.to_string(),
            wire: req.wire.to_string(),
            scope: req.scope.to_string(),
            project_id: req.project_id.to_string(),
            project_name: req.project_name.to_string(),
            expires_at: req.expires_at,
            max_uses: req.max_uses,
            uses_count: 0,
            revoked_at: None,
            note: req.note.map(String::from),
            created_at: now,
        })
    }

    pub fn revoke(&self, invite_id: &str) -> Result<bool, CoordinatorError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let changed = self.db.conn().execute(
            "UPDATE invites SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
            rusqlite::params![now, invite_id],
        )?;
        Ok(changed > 0)
    }

    pub fn get(&self, invite_id: &str) -> Result<Option<InviteRecord>, CoordinatorError> {
        let result = self
            .db
            .conn()
            .query_row(
                "SELECT id, wire, scope, project_id, project_name, expires_at, \
                 max_uses, uses_count, revoked_at, note, created_at \
                 FROM invites WHERE id = ?1",
                rusqlite::params![invite_id],
                row_to_invite,
            )
            .ok();
        Ok(result)
    }

    pub fn list(&self, limit: usize) -> Result<Vec<InviteRecord>, CoordinatorError> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, wire, scope, project_id, project_name, expires_at, \
             max_uses, uses_count, revoked_at, note, created_at \
             FROM invites ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit as i64], row_to_invite)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

fn row_to_invite(row: &rusqlite::Row<'_>) -> rusqlite::Result<InviteRecord> {
    Ok(InviteRecord {
        id: row.get(0)?,
        wire: row.get(1)?,
        scope: row.get(2)?,
        project_id: row.get(3)?,
        project_name: row.get(4)?,
        expires_at: row.get(5)?,
        max_uses: row.get(6)?,
        uses_count: row.get(7)?,
        revoked_at: row.get(8)?,
        note: row.get(9)?,
        created_at: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> CoordinatorDb {
        CoordinatorDb::open_in_memory().unwrap()
    }

    fn mk_req<'b>(id: &'b str, wire: &'b str, scope: &'b str) -> MintRequest<'b> {
        MintRequest {
            id,
            wire,
            scope,
            project_id: "proj1",
            project_name: "TestProject",
            expires_at: 9999999999,
            max_uses: None,
            note: None,
        }
    }

    #[test]
    fn mint_and_get() {
        let db = make_db();
        let ledger = InviteLedger::new(&db);
        let mut req = mk_req("inv-001", "nx1abc", "worker");
        req.max_uses = Some(10);
        req.note = Some("test invite");
        let rec = ledger.mint(&req).unwrap();
        assert_eq!(rec.id, "inv-001");
        assert_eq!(rec.scope, "worker");
        assert_eq!(rec.uses_count, 0);
        assert!(rec.revoked_at.is_none());
        let fetched = ledger.get("inv-001").unwrap().unwrap();
        assert_eq!(fetched.wire, "nx1abc");
    }

    #[test]
    fn revoke_invite() {
        let db = make_db();
        let ledger = InviteLedger::new(&db);
        ledger.mint(&mk_req("inv-r1", "wire1", "observer")).unwrap();
        assert!(ledger.revoke("inv-r1").unwrap());
        let rec = ledger.get("inv-r1").unwrap().unwrap();
        assert!(rec.revoked_at.is_some());
        assert!(!ledger.revoke("inv-r1").unwrap());
    }

    #[test]
    fn list_invites() {
        let db = make_db();
        let ledger = InviteLedger::new(&db);
        ledger.mint(&mk_req("inv-a", "w1", "worker")).unwrap();
        ledger.mint(&mk_req("inv-b", "w2", "observer")).unwrap();
        let list = ledger.list(10).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn get_nonexistent() {
        let db = make_db();
        let ledger = InviteLedger::new(&db);
        assert!(ledger.get("nope").unwrap().is_none());
    }
}
