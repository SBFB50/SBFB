// SPDX-License-Identifier: AGPL-3.0-or-later
//! Quarantine queue for borderline gossip messages
//! (Sprint 41 Phase C, port of quarantine_queue.py S21).
//!
//! Background sweep loop deferred to Tier 5 wire-up (D3).

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineEntry {
    pub id: i64,
    pub topic: String,
    pub sender_pubkey_hex: String,
    pub payload_json: String,
    pub received_at: i64,
    pub rate_strikes: i32,
    pub pow_status: String,
    pub flush_status: String,
}

pub struct QuarantineQueue<'a> {
    db: &'a CoordinatorDb,
    ttl_secs: i64,
}

impl<'a> QuarantineQueue<'a> {
    pub fn new(db: &'a CoordinatorDb, ttl_secs: i64) -> Self {
        Self {
            db,
            ttl_secs: ttl_secs.max(1),
        }
    }

    pub fn add(
        &self,
        topic: &str,
        sender_pubkey_hex: &str,
        payload_json: &str,
        rate_strikes: i32,
        pow_status: &str,
    ) -> Result<i64, CoordinatorError> {
        let now = now_epoch();
        self.db.conn().execute(
            "INSERT INTO quarantine_messages \
             (topic, sender_pubkey_hex, payload_json, received_at, \
              rate_strikes, pow_status, flush_status) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')",
            rusqlite::params![
                topic,
                sender_pubkey_hex,
                payload_json,
                now,
                rate_strikes,
                pow_status
            ],
        )?;
        let id = self.db.conn().last_insert_rowid();
        Ok(id)
    }

    pub fn list_pending(&self) -> Result<Vec<QuarantineEntry>, CoordinatorError> {
        self.list_by_status("pending")
    }

    pub fn list_by_status(&self, status: &str) -> Result<Vec<QuarantineEntry>, CoordinatorError> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, topic, sender_pubkey_hex, payload_json, received_at, \
             rate_strikes, pow_status, flush_status \
             FROM quarantine_messages WHERE flush_status = ?1 \
             ORDER BY received_at ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![status], row_to_entry)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn flush(&self, row_id: i64) -> Result<bool, CoordinatorError> {
        self.set_status(row_id, "flushed")
    }

    pub fn drop_entry(&self, row_id: i64) -> Result<bool, CoordinatorError> {
        self.set_status(row_id, "dropped")
    }

    pub fn flush_expired(&self) -> Result<usize, CoordinatorError> {
        let cutoff = now_epoch() - self.ttl_secs;
        let deleted = self.db.conn().execute(
            "DELETE FROM quarantine_messages \
             WHERE received_at < ?1 AND flush_status = 'pending'",
            rusqlite::params![cutoff],
        )?;
        Ok(deleted)
    }

    pub fn pending_count(&self) -> Result<usize, CoordinatorError> {
        let count: i64 = self.db.conn().query_row(
            "SELECT COUNT(*) FROM quarantine_messages WHERE flush_status = 'pending'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    fn set_status(&self, row_id: i64, new_status: &str) -> Result<bool, CoordinatorError> {
        let changed = self.db.conn().execute(
            "UPDATE quarantine_messages SET flush_status = ?1 \
             WHERE id = ?2 AND flush_status = 'pending'",
            rusqlite::params![new_status, row_id],
        )?;
        Ok(changed > 0)
    }
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<QuarantineEntry> {
    Ok(QuarantineEntry {
        id: row.get(0)?,
        topic: row.get(1)?,
        sender_pubkey_hex: row.get(2)?,
        payload_json: row.get(3)?,
        received_at: row.get(4)?,
        rate_strikes: row.get(5)?,
        pow_status: row.get(6)?,
        flush_status: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> CoordinatorDb {
        CoordinatorDb::open_in_memory().unwrap()
    }

    #[test]
    fn add_and_list_pending() {
        let db = make_db();
        let q = QuarantineQueue::new(&db, 900);
        let id = q
            .add("gossip", "aabb", r#"{"msg":"hi"}"#, 2, "valid")
            .unwrap();
        assert!(id > 0);
        let pending = q.list_pending().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].topic, "gossip");
    }

    #[test]
    fn flush_entry() {
        let db = make_db();
        let q = QuarantineQueue::new(&db, 900);
        let id = q.add("t", "pk", "{}", 0, "valid").unwrap();
        assert!(q.flush(id).unwrap());
        assert_eq!(q.pending_count().unwrap(), 0);
        assert!(!q.flush(id).unwrap());
    }

    #[test]
    fn drop_entry() {
        let db = make_db();
        let q = QuarantineQueue::new(&db, 900);
        let id = q.add("t", "pk", "{}", 0, "missing").unwrap();
        assert!(q.drop_entry(id).unwrap());
        assert_eq!(q.pending_count().unwrap(), 0);
    }

    #[test]
    fn flush_expired_removes_old() {
        let db = make_db();
        let q = QuarantineQueue::new(&db, 900);
        db.conn()
            .execute(
                "INSERT INTO quarantine_messages \
                 (topic, sender_pubkey_hex, payload_json, received_at, \
                  rate_strikes, pow_status, flush_status) \
                 VALUES ('t', 'pk', '{}', 1000, 0, 'valid', 'pending')",
                [],
            )
            .unwrap();
        let deleted = q.flush_expired().unwrap();
        assert_eq!(deleted, 1);
    }

    #[test]
    fn fresh_entry_not_expired() {
        let db = make_db();
        let q = QuarantineQueue::new(&db, 900);
        q.add("t", "pk", "{}", 0, "valid").unwrap();
        let deleted = q.flush_expired().unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(q.pending_count().unwrap(), 1);
    }
}
