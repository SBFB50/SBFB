// SPDX-License-Identifier: AGPL-3.0-or-later
//! SQLite LRU cache for forge contribution records (Couche 3 trust-web).
//!
//! Caches the output of [`nexus_core_rs::attestations::forge_parser::parse_git_log`]
//! per repository URL with a configurable TTL (default 7 days). WAL mode
//! for concurrent reads during gossip verification.

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

use nexus_core_rs::attestations::forge_parser::{ForgeContribution, SigType};
use nexus_core_rs::error::{NexusError, Result};

const DEFAULT_TTL_SECS: u64 = 7 * 24 * 3600; // 7 days

static MIGRATIONS: &[M<'static>] = &[M::up(
    "CREATE TABLE IF NOT EXISTS forge_contributions (
        repo_url TEXT NOT NULL,
        fingerprint TEXT NOT NULL,
        commit_count INTEGER NOT NULL,
        first_seen INTEGER NOT NULL,
        last_seen INTEGER NOT NULL,
        forge_url TEXT NOT NULL,
        sig_type TEXT NOT NULL,
        cached_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
        PRIMARY KEY (repo_url, fingerprint)
    );",
)];

pub struct TrustCache {
    db: Connection,
    ttl: Duration,
}

impl TrustCache {
    pub fn open(db_path: &Path) -> Result<Self> {
        Self::open_with_ttl(db_path, Duration::from_secs(DEFAULT_TTL_SECS))
    }

    pub fn open_with_ttl(db_path: &Path, ttl: Duration) -> Result<Self> {
        let mut db = Connection::open(db_path)
            .map_err(|e| NexusError::Other(format!("trust_cache: open failed: {e}")))?;
        db.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| NexusError::Other(format!("trust_cache: WAL mode failed: {e}")))?;
        db.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| NexusError::Other(format!("trust_cache: synchronous failed: {e}")))?;

        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations
            .to_latest(&mut db)
            .map_err(|e| NexusError::Other(format!("trust_cache: migration failed: {e}")))?;

        Ok(Self { db, ttl })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        Self::open_in_memory_with_ttl(Duration::from_secs(DEFAULT_TTL_SECS))
    }

    #[cfg(test)]
    pub fn open_in_memory_with_ttl(ttl: Duration) -> Result<Self> {
        let mut db = Connection::open_in_memory()
            .map_err(|e| NexusError::Other(format!("trust_cache: open_in_memory failed: {e}")))?;
        db.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| NexusError::Other(format!("trust_cache: WAL mode failed: {e}")))?;

        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations
            .to_latest(&mut db)
            .map_err(|e| NexusError::Other(format!("trust_cache: migration failed: {e}")))?;

        Ok(Self { db, ttl })
    }

    /// Retrieve cached contributions for a repo, or parse and cache if stale/missing.
    ///
    /// `parse_fn` is called when the cache is cold or expired. Typically
    /// wraps `forge_parser::parse_git_log`.
    pub fn get_or_parse<F>(&self, repo_url: &str, parse_fn: F) -> Result<Vec<ForgeContribution>>
    where
        F: FnOnce() -> Result<Vec<ForgeContribution>>,
    {
        if let Some(cached) = self.get_if_fresh(repo_url)? {
            return Ok(cached);
        }
        let contributions = parse_fn()?;
        self.store(repo_url, &contributions)?;
        Ok(contributions)
    }

    pub fn invalidate(&self, repo_url: &str) -> Result<()> {
        self.db
            .execute(
                "DELETE FROM forge_contributions WHERE repo_url = ?1",
                rusqlite::params![repo_url],
            )
            .map_err(|e| NexusError::Other(format!("trust_cache: invalidate failed: {e}")))?;
        Ok(())
    }

    fn get_if_fresh(&self, repo_url: &str) -> Result<Option<Vec<ForgeContribution>>> {
        let ttl_secs = self.ttl.as_secs() as i64;
        let mut stmt = self
            .db
            .prepare(
                "SELECT fingerprint, commit_count, first_seen, last_seen, forge_url, sig_type
                 FROM forge_contributions
                 WHERE repo_url = ?1
                   AND (julianday('now') - julianday(cached_at)) * 86400 < ?2",
            )
            .map_err(|e| NexusError::Other(format!("trust_cache: prepare failed: {e}")))?;

        let rows = stmt
            .query_map(rusqlite::params![repo_url, ttl_secs], |row| {
                Ok(ForgeContribution {
                    fingerprint: row.get(0)?,
                    commit_count: row.get(1)?,
                    first_seen: row.get(2)?,
                    last_seen: row.get(3)?,
                    forge_url: row.get(4)?,
                    sig_type: match row.get::<_, String>(5)?.as_str() {
                        "Ssh" => SigType::Ssh,
                        _ => SigType::Gpg,
                    },
                })
            })
            .map_err(|e| NexusError::Other(format!("trust_cache: query failed: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            results
                .push(row.map_err(|e| NexusError::Other(format!("trust_cache: row failed: {e}")))?);
        }

        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results))
        }
    }

    fn store(&self, repo_url: &str, contributions: &[ForgeContribution]) -> Result<()> {
        self.db
            .execute(
                "DELETE FROM forge_contributions WHERE repo_url = ?1",
                rusqlite::params![repo_url],
            )
            .map_err(|e| NexusError::Other(format!("trust_cache: delete failed: {e}")))?;

        let mut stmt = self
            .db
            .prepare(
                "INSERT INTO forge_contributions
                 (repo_url, fingerprint, commit_count, first_seen, last_seen, forge_url, sig_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .map_err(|e| NexusError::Other(format!("trust_cache: prepare insert failed: {e}")))?;

        for c in contributions {
            let sig_str = match c.sig_type {
                SigType::Gpg => "Gpg",
                SigType::Ssh => "Ssh",
            };
            stmt.execute(rusqlite::params![
                repo_url,
                c.fingerprint,
                c.commit_count,
                c.first_seen,
                c.last_seen,
                c.forge_url,
                sig_str,
            ])
            .map_err(|e| NexusError::Other(format!("trust_cache: insert failed: {e}")))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample_contributions() -> Vec<ForgeContribution> {
        vec![
            ForgeContribution {
                fingerprint: "abcd1234".to_string(),
                commit_count: 10,
                first_seen: 1_700_000_000,
                last_seen: 1_710_000_000,
                forge_url: "https://github.com/test/repo".to_string(),
                sig_type: SigType::Gpg,
            },
            ForgeContribution {
                fingerprint: "ef567890".to_string(),
                commit_count: 5,
                first_seen: 1_705_000_000,
                last_seen: 1_710_000_000,
                forge_url: "https://github.com/test/repo".to_string(),
                sig_type: SigType::Ssh,
            },
        ]
    }

    #[test]
    fn test_trust_cache_store_and_retrieve() {
        let cache = TrustCache::open_in_memory().unwrap();
        let repo_url = "https://github.com/test/repo";

        let result = cache
            .get_or_parse(repo_url, || Ok(sample_contributions()))
            .unwrap();
        assert_eq!(result.len(), 2);

        let mut parse_called = false;
        let cached = cache
            .get_or_parse(repo_url, || {
                parse_called = true;
                Ok(vec![])
            })
            .unwrap();
        assert!(!parse_called, "should use cache, not re-parse");
        assert_eq!(cached.len(), 2);
    }

    #[test]
    fn test_trust_cache_ttl_expiry() {
        let cache = TrustCache::open_in_memory_with_ttl(Duration::from_secs(0)).unwrap();
        let repo_url = "https://github.com/test/repo";

        cache
            .get_or_parse(repo_url, || Ok(sample_contributions()))
            .unwrap();

        let mut parse_called = false;
        let fresh = cache
            .get_or_parse(repo_url, || {
                parse_called = true;
                Ok(vec![ForgeContribution {
                    fingerprint: "new_key".to_string(),
                    commit_count: 1,
                    first_seen: 1_720_000_000,
                    last_seen: 1_720_000_000,
                    forge_url: repo_url.to_string(),
                    sig_type: SigType::Gpg,
                }])
            })
            .unwrap();
        assert!(parse_called, "TTL=0 should force re-parse");
        assert_eq!(fresh.len(), 1);
        assert_eq!(fresh[0].fingerprint, "new_key");
    }

    #[test]
    fn test_trust_cache_invalidate() {
        let cache = TrustCache::open_in_memory().unwrap();
        let repo_url = "https://github.com/test/repo";

        cache
            .get_or_parse(repo_url, || Ok(sample_contributions()))
            .unwrap();

        cache.invalidate(repo_url).unwrap();

        let mut parse_called = false;
        cache
            .get_or_parse(repo_url, || {
                parse_called = true;
                Ok(vec![])
            })
            .unwrap();
        assert!(parse_called, "invalidation should force re-parse");
    }
}
