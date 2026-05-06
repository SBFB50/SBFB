// SPDX-License-Identifier: AGPL-3.0-or-later
//! Delayed upload queue with anti-correlation jitter
//! (Sprint 41 Phase C, port of upload_queue.py S19).
//!
//! Background flush loop deferred to Tier 5 wire-up (D3).

use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;

pub const DEFAULT_MEAN_JITTER_S: f64 = 90.0;
pub const DEFAULT_MAX_JITTER_S: f64 = 300.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadEntry {
    pub upload_id: String,
    pub deliver_at: f64,
    pub task_payload_json: String,
    pub enqueued_at: f64,
    pub status: String,
}

pub struct UploadQueue<'a> {
    db: &'a CoordinatorDb,
    mean_jitter: f64,
    max_jitter: f64,
}

impl<'a> UploadQueue<'a> {
    pub fn new(db: &'a CoordinatorDb, mean_jitter: f64, max_jitter: f64) -> Self {
        Self {
            db,
            mean_jitter: mean_jitter.max(0.001),
            max_jitter: max_jitter.max(0.001),
        }
    }

    pub fn schedule(
        &self,
        upload_id: &str,
        task_payload_json: &str,
    ) -> Result<f64, CoordinatorError> {
        let now = now_f64();
        let delay = self.compute_jitter();
        let deliver_at = now + delay;
        self.db.conn().execute(
            "INSERT INTO delayed_uploads \
             (upload_id, deliver_at, task_payload_json, enqueued_at, status) \
             VALUES (?1, ?2, ?3, ?4, 'pending')",
            rusqlite::params![upload_id, deliver_at, task_payload_json, now],
        )?;
        Ok(delay)
    }

    pub fn ready_uploads(&self) -> Result<Vec<UploadEntry>, CoordinatorError> {
        let now = now_f64();
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT upload_id, deliver_at, task_payload_json, enqueued_at, status \
             FROM delayed_uploads \
             WHERE deliver_at <= ?1 AND status = 'pending' \
             ORDER BY deliver_at ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![now], row_to_entry)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn mark_done(&self, upload_id: &str) -> Result<bool, CoordinatorError> {
        let changed = self.db.conn().execute(
            "UPDATE delayed_uploads SET status = 'done' \
             WHERE upload_id = ?1 AND status = 'pending'",
            rusqlite::params![upload_id],
        )?;
        Ok(changed > 0)
    }

    pub fn mark_failed(&self, upload_id: &str, reason: &str) -> Result<bool, CoordinatorError> {
        let changed = self.db.conn().execute(
            "UPDATE delayed_uploads SET status = 'failed' \
             WHERE upload_id = ?1 AND status = 'pending'",
            rusqlite::params![upload_id],
        )?;
        if changed > 0 {
            let _ = reason;
        }
        Ok(changed > 0)
    }

    pub fn pending_count(&self) -> Result<usize, CoordinatorError> {
        let count: i64 = self.db.conn().query_row(
            "SELECT COUNT(*) FROM delayed_uploads WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn compute_jitter(&self) -> f64 {
        let u: f64 = rand::thread_rng().r#gen();
        let u_safe = if u <= 0.0 { 1e-18 } else { u };
        let raw = -self.mean_jitter * u_safe.ln();
        raw.min(self.max_jitter)
    }
}

fn now_f64() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<UploadEntry> {
    Ok(UploadEntry {
        upload_id: row.get(0)?,
        deliver_at: row.get(1)?,
        task_payload_json: row.get(2)?,
        enqueued_at: row.get(3)?,
        status: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> CoordinatorDb {
        CoordinatorDb::open_in_memory().unwrap()
    }

    #[test]
    fn schedule_and_pending_count() {
        let db = make_db();
        let q = UploadQueue::new(&db, 0.001, 0.01);
        q.schedule("u1", r#"{"task_id":"t1"}"#).unwrap();
        assert_eq!(q.pending_count().unwrap(), 1);
    }

    #[test]
    fn ready_uploads_after_delay() {
        let db = make_db();
        let q = UploadQueue::new(&db, 0.001, 0.001);
        q.schedule("u1", "{}").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ready = q.ready_uploads().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].upload_id, "u1");
    }

    #[test]
    fn mark_done() {
        let db = make_db();
        let q = UploadQueue::new(&db, 0.001, 0.001);
        q.schedule("u1", "{}").unwrap();
        assert!(q.mark_done("u1").unwrap());
        assert_eq!(q.pending_count().unwrap(), 0);
        assert!(!q.mark_done("u1").unwrap());
    }

    #[test]
    fn mark_failed() {
        let db = make_db();
        let q = UploadQueue::new(&db, 0.001, 0.001);
        q.schedule("u1", "{}").unwrap();
        assert!(q.mark_failed("u1", "timeout").unwrap());
        assert_eq!(q.pending_count().unwrap(), 0);
    }

    #[test]
    fn jitter_in_range() {
        let db = make_db();
        let q = UploadQueue::new(&db, 90.0, 300.0);
        for _ in 0..20 {
            let j = q.compute_jitter();
            assert!(j >= 0.0);
            assert!(j <= 300.0);
        }
    }

    #[test]
    fn done_not_in_ready() {
        let db = make_db();
        let q = UploadQueue::new(&db, 0.001, 0.001);
        q.schedule("u1", "{}").unwrap();
        q.mark_done("u1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ready = q.ready_uploads().unwrap();
        assert!(ready.is_empty());
    }
}
