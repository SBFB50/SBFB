// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-(consumer, model) PoW task counter with daily UTC reset
//! (Sprint 41 Phase A, port of pow_counter.py S23).

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;

fn today_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

pub struct PowCounter<'a> {
    db: &'a CoordinatorDb,
}

impl<'a> PowCounter<'a> {
    pub fn new(db: &'a CoordinatorDb) -> Self {
        Self { db }
    }

    pub fn increment(&self, consumer_id: &str, model_id: &str) -> Result<u32, CoordinatorError> {
        let today = today_utc();
        let conn = self.db.conn();

        let existing: Option<(u32, String)> = conn
            .query_row(
                "SELECT count, last_reset_utc FROM pow_task_counts \
                 WHERE consumer_id = ?1 AND model_id = ?2",
                rusqlite::params![consumer_id, model_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        let new_count = if let Some((count, reset_date)) = existing {
            if reset_date == today {
                let nc = count + 1;
                conn.execute(
                    "UPDATE pow_task_counts SET count = ?1 \
                     WHERE consumer_id = ?2 AND model_id = ?3",
                    rusqlite::params![nc, consumer_id, model_id],
                )?;
                nc
            } else {
                conn.execute(
                    "INSERT OR REPLACE INTO pow_task_counts \
                     (consumer_id, model_id, count, last_reset_utc) \
                     VALUES (?1, ?2, 1, ?3)",
                    rusqlite::params![consumer_id, model_id, today],
                )?;
                1
            }
        } else {
            conn.execute(
                "INSERT INTO pow_task_counts \
                 (consumer_id, model_id, count, last_reset_utc) \
                 VALUES (?1, ?2, 1, ?3)",
                rusqlite::params![consumer_id, model_id, today],
            )?;
            1
        };

        Ok(new_count)
    }

    pub fn get_count(&self, consumer_id: &str, model_id: &str) -> Result<u32, CoordinatorError> {
        let today = today_utc();
        let conn = self.db.conn();

        let result: Option<(u32, String)> = conn
            .query_row(
                "SELECT count, last_reset_utc FROM pow_task_counts \
                 WHERE consumer_id = ?1 AND model_id = ?2",
                rusqlite::params![consumer_id, model_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        match result {
            Some((count, reset_date)) if reset_date == today => Ok(count),
            _ => Ok(0),
        }
    }

    pub fn reset_expired(&self) -> Result<usize, CoordinatorError> {
        let today = today_utc();
        let deleted = self.db.conn().execute(
            "DELETE FROM pow_task_counts WHERE last_reset_utc < ?1",
            rusqlite::params![today],
        )?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> CoordinatorDb {
        CoordinatorDb::open_in_memory().unwrap()
    }

    #[test]
    fn increment_and_get() {
        let db = make_db();
        let pc = PowCounter::new(&db);
        let c1 = pc.increment("user1", "llama3").unwrap();
        assert_eq!(c1, 1);
        let c2 = pc.increment("user1", "llama3").unwrap();
        assert_eq!(c2, 2);
        assert_eq!(pc.get_count("user1", "llama3").unwrap(), 2);
    }

    #[test]
    fn get_count_absent() {
        let db = make_db();
        let pc = PowCounter::new(&db);
        assert_eq!(pc.get_count("nobody", "nomodel").unwrap(), 0);
    }

    #[test]
    fn separate_consumer_model_pairs() {
        let db = make_db();
        let pc = PowCounter::new(&db);
        pc.increment("user1", "llama3").unwrap();
        pc.increment("user1", "llama3").unwrap();
        pc.increment("user1", "mistral").unwrap();
        pc.increment("user2", "llama3").unwrap();
        assert_eq!(pc.get_count("user1", "llama3").unwrap(), 2);
        assert_eq!(pc.get_count("user1", "mistral").unwrap(), 1);
        assert_eq!(pc.get_count("user2", "llama3").unwrap(), 1);
    }

    #[test]
    fn reset_expired_removes_old_rows() {
        let db = make_db();
        let conn = db.conn();
        conn.execute(
            "INSERT INTO pow_task_counts (consumer_id, model_id, count, last_reset_utc) \
             VALUES ('old', 'model', 5, '2020-01-01')",
            [],
        )
        .unwrap();
        let pc = PowCounter::new(&db);
        let deleted = pc.reset_expired().unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(pc.get_count("old", "model").unwrap(), 0);
    }
}
