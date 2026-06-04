// SPDX-License-Identifier: AGPL-3.0-or-later
//! FTS5 full-text search over browse entries and feed operations.

use crate::db::CoordinatorDb;
use crate::error::CoordinatorError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub project_id: String,
    pub project_name: String,
    pub category: String,
    pub description: String,
    pub op_type: String,
    pub source_type: String,
    pub score: f64,
}

pub fn sanitize_query(input: &str) -> Option<String> {
    let cleaned: String = input.chars().filter(|c| *c != '\0').collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return None;
    }
    let tokens: Vec<String> = cleaned
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect();
    if tokens.is_empty() {
        return None;
    }
    Some(tokens.join(" "))
}

pub fn index_entry(
    db: &CoordinatorDb,
    project_id: &str,
    project_name: &str,
    category: &str,
    description: &str,
    op_type: &str,
    source_type: &str,
) -> Result<(), CoordinatorError> {
    db.conn().execute(
        "INSERT INTO search_index (project_id, project_name, category, description, op_type, source_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![project_id, project_name, category, description, op_type, source_type],
    )?;
    Ok(())
}

pub fn search(
    db: &CoordinatorDb,
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<(Vec<SearchResult>, u64), CoordinatorError> {
    let sanitized = match sanitize_query(query) {
        Some(q) => q,
        None => return Ok((Vec::new(), 0)),
    };

    let total: u64 =
        db.conn()
            .prepare_cached("SELECT COUNT(*) FROM search_index WHERE search_index MATCH ?1")?
            .query_row(rusqlite::params![sanitized], |row| row.get::<_, i64>(0))? as u64;

    let mut stmt = db.conn().prepare_cached(
        "SELECT project_id, project_name, category, description, op_type, source_type, bm25(search_index)
         FROM search_index
         WHERE search_index MATCH ?1
         ORDER BY bm25(search_index)
         LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![sanitized, limit as i64, offset as i64],
        |row| {
            Ok(SearchResult {
                project_id: row.get(0)?,
                project_name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                op_type: row.get(4)?,
                source_type: row.get(5)?,
                score: row.get(6)?,
            })
        },
    )?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok((results, total))
}

/// The indexable fields a feed operation contributes to the FTS5 index.
///
/// Feed operations carry no `project_name`/`category` today (see
/// `public_feed::*Payload`): only the curator/stale `reason` (or legacy
/// `comment`) becomes matchable `description` text. The provenance triplet
/// enrichment lands in Phase D (migration M17), as UNINDEXED columns.
struct IndexFields {
    project_id: String,
    project_name: String,
    category: String,
    description: String,
}

/// Extract the FTS5 index fields from a parsed feed operation payload.
///
/// Shared by the hot path ([`upsert_feed_entry`]) and the repair path
/// ([`rebuild_from_feed`]) so the two cannot drift apart. Mirrors the
/// historical boot-rebuild extraction: `project_name`/`category` are empty
/// for current feed ops, `description` comes from `reason` (or `comment`).
fn extract_index_fields(op: &serde_json::Value) -> IndexFields {
    let field = |key: &str| {
        op.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    IndexFields {
        project_id: field("project_id"),
        project_name: field("project_name"),
        // No feed op carries a category today; the boot rebuild always
        // indexed an empty category for feed entries — mirror that here.
        category: String::new(),
        description: op
            .get("reason")
            .or_else(|| op.get("comment"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    }
}

/// Hot incremental FTS5 reindex of a single feed entry (Sprint 73 Phase C).
///
/// Keyed by the feed `seq` as the FTS5 rowid: a re-arrived entry rewrites the
/// same row rather than appending a duplicate (idempotent — a second line of
/// defence behind the `entry_hash` dedup in `feed_sync`). `INSERT OR REPLACE`
/// is the canonical upsert for a standalone (non external-content) FTS5 table.
///
/// Called inside the `feed_sync` DB lock scope right after a successful
/// [`CoordinatorDb::insert_feed_entry`], so a gossiped project becomes
/// searchable the instant it is ingested instead of only at the next boot
/// rebuild. The statement is a single short write, so it shares the existing
/// critical section without an extra lock round-trip.
///
/// Note on the rowid space: feed rows own `[1, max feed seq]`. Browse-sourced
/// rows ([`index_entry`], auto rowid) are currently test-only; wiring browse
/// indexing in production (S74) must partition the rowid space so a feed
/// upsert cannot clobber a browse row.
pub fn upsert_feed_entry(
    db: &CoordinatorDb,
    seq: u64,
    op: &serde_json::Value,
    op_type: &str,
) -> Result<(), CoordinatorError> {
    let fields = extract_index_fields(op);
    db.conn().execute(
        "INSERT OR REPLACE INTO search_index
            (rowid, project_id, project_name, category, description, op_type, source_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'feed')",
        rusqlite::params![
            seq as i64,
            fields.project_id,
            fields.project_name,
            fields.category,
            fields.description,
            op_type,
        ],
    )?;
    Ok(())
}

/// Repair path: rebuild the feed slice of the index from the durable feed.
///
/// No longer on the hot path (Phase C makes ingest reindex incrementally).
/// Kept for boot recovery and migrations (e.g. M17 in Phase D): the index is
/// fully reconstructible from `public_feed`. Re-uses [`upsert_feed_entry`] so
/// rebuilt rows are byte-for-byte identical to hot-path rows (same rowid=seq,
/// same shared extractor).
pub fn rebuild_from_feed(db: &CoordinatorDb) -> Result<usize, CoordinatorError> {
    db.conn()
        .execute("DELETE FROM search_index WHERE source_type = 'feed'", [])?;

    let entries = db.get_feed_entries()?;
    let mut indexed = 0usize;
    for entry in &entries {
        let op: serde_json::Value = serde_json::from_str(&entry.payload).unwrap_or_default();
        upsert_feed_entry(db, entry.seq, &op, &entry.op_type)?;
        indexed += 1;
    }
    Ok(indexed)
}

pub fn clear_all(db: &CoordinatorDb) -> Result<(), CoordinatorError> {
    db.conn().execute("DELETE FROM search_index", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> CoordinatorDb {
        CoordinatorDb::open_in_memory().expect("open in-memory")
    }

    #[test]
    fn test_search_index_browse_entry() {
        let db = setup_db();
        index_entry(
            &db,
            "proj1",
            "My Project",
            "gov",
            "A governance tool",
            "",
            "browse",
        )
        .expect("index");
        let (results, total) = search(&db, "governance", 20, 0).expect("search");
        assert_eq!(total, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].project_id, "proj1");
        assert_eq!(results[0].project_name, "My Project");
        assert_eq!(results[0].source_type, "browse");
    }

    #[test]
    fn test_search_index_feed_entry() {
        let db = setup_db();
        let row = crate::db::FeedEntryRow {
            seq: 1,
            op_type: "ReleasePublished".to_string(),
            payload: serde_json::json!({
                "project_id": "proj-abc",
                "repo_url": "https://github.com/test/repo",
                "commit_sha": "abc123",
                "artifact_hash": "deadbeef",
                "is_open_source": true
            })
            .to_string(),
            author: "aa".repeat(32),
            signature: "sig".to_string(),
            entry_hash: "hash1".to_string(),
            prev_hash: "0".repeat(64),
            created_at: 1700000000,
        };
        db.insert_feed_entry(&row).expect("insert feed");
        rebuild_from_feed(&db).expect("rebuild");

        let (_, total) = search(&db, "ReleasePublished", 20, 0).expect("search");
        assert_eq!(
            total, 0,
            "op_type is UNINDEXED, should not match FTS5 query"
        );

        let (results2, _) = search(&db, "proj-abc", 20, 0).expect("search");
        assert!(results2.is_empty(), "project_id is UNINDEXED");
    }

    #[test]
    fn test_search_query_returns_score() {
        let db = setup_db();
        index_entry(
            &db,
            "p1",
            "Alpha Tool",
            "dev",
            "developer utilities",
            "",
            "browse",
        )
        .expect("index");
        index_entry(
            &db,
            "p2",
            "Beta Tool",
            "dev",
            "developer framework",
            "",
            "browse",
        )
        .expect("index");
        let (results, _) = search(&db, "developer", 20, 0).expect("search");
        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(r.score != 0.0, "bm25 score must be non-zero");
        }
    }

    #[test]
    fn test_search_query_pagination() {
        let db = setup_db();
        for i in 0..5 {
            index_entry(
                &db,
                &format!("p{i}"),
                &format!("Project {i}"),
                "cat",
                "searchable description",
                "",
                "browse",
            )
            .expect("index");
        }
        let (results, total) = search(&db, "searchable", 2, 0).expect("search");
        assert_eq!(total, 5);
        assert_eq!(results.len(), 2);

        let (results2, _) = search(&db, "searchable", 2, 2).expect("search offset");
        assert_eq!(results2.len(), 2);
        assert_ne!(results[0].project_id, results2[0].project_id);
    }

    #[test]
    fn test_search_query_empty_returns_empty() {
        let db = setup_db();
        index_entry(&db, "p1", "Something", "cat", "desc", "", "browse").expect("index");
        let (results, total) = search(&db, "nonexistent", 20, 0).expect("search");
        assert_eq!(total, 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_sanitizer_rejects_nul_bytes() {
        let result = sanitize_query("hello\0world");
        assert_eq!(result, Some("\"helloworld\"".to_string()));

        let result_spaces = sanitize_query("hello\0 world");
        assert_eq!(result_spaces, Some("\"hello\" \"world\"".to_string()));

        let result_empty = sanitize_query("\0\0");
        assert_eq!(result_empty, None);
    }

    #[test]
    fn test_sanitize_escapes_fts5_syntax() {
        let result = sanitize_query("OR AND \"test\"");
        assert_eq!(result, Some("\"OR\" \"AND\" \"\"\"test\"\"\"".to_string()));
    }

    // -- Sprint 73 Phase C: hot incremental FTS5 reindex (D1) --

    fn feed_row(seq: u64, op: serde_json::Value, hash: &str) -> crate::db::FeedEntryRow {
        let op_type = op
            .get("op_type")
            .and_then(|v| v.as_str())
            .unwrap_or("CuratorVouched")
            .to_string();
        crate::db::FeedEntryRow {
            seq,
            op_type,
            payload: op.to_string(),
            author: "aa".repeat(32),
            signature: "sig".to_string(),
            entry_hash: hash.to_string(),
            prev_hash: "0".repeat(64),
            created_at: 1_700_000_000,
        }
    }

    #[test]
    fn feed_ingest_indexes_entry_hot() {
        let db = setup_db();
        // Mirror the feed_sync hot path: persist a feed entry, then upsert it
        // into the index in the same step — no boot rebuild in between.
        let op = serde_json::json!({
            "project_id": "proj-hot",
            "curator_pubkey": "bb".repeat(32),
            "reason": "endorses the quantum compiler project"
        });
        let row = feed_row(0, op.clone(), "hothash");
        let seq = db.insert_feed_entry(&row).expect("insert feed");
        upsert_feed_entry(&db, seq, &op, "CuratorVouched").expect("hot upsert");

        // Searchable at once, without rebuild_from_feed / a reboot.
        let (results, total) = search(&db, "quantum", 20, 0).expect("search");
        assert_eq!(
            total, 1,
            "freshly ingested entry must be searchable at once"
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_type, "feed");
    }

    #[test]
    fn reindex_hot_is_idempotent() {
        let db = setup_db();
        let op = serde_json::json!({
            "project_id": "proj-idem",
            "reason": "stale duplicate guard entry"
        });
        // Same seq upserted twice → INSERT OR REPLACE rewrites the same rowid.
        upsert_feed_entry(&db, 42, &op, "SourceBecameStale").expect("upsert 1");
        upsert_feed_entry(&db, 42, &op, "SourceBecameStale").expect("upsert 2");

        let (results, total) = search(&db, "duplicate", 20, 0).expect("search");
        assert_eq!(
            total, 1,
            "re-upserting the same seq must not create a duplicate row"
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn extract_index_fields_shared_with_rebuild() {
        let db = setup_db();
        let op = serde_json::json!({
            "project_id": "proj-shared",
            "project_name": "Shared Fields",
            "reason": "anti drift check"
        });
        let row = feed_row(0, op.clone(), "sharedhash");
        let seq = db.insert_feed_entry(&row).expect("insert");

        // Path A: full rebuild (repair path).
        rebuild_from_feed(&db).expect("rebuild");
        let (rebuilt, _) = search(&db, "drift", 20, 0).expect("search rebuilt");
        assert_eq!(rebuilt.len(), 1);

        // Path B: hot upsert of the same entry after clearing the index.
        clear_all(&db).expect("clear");
        upsert_feed_entry(&db, seq, &op, &row.op_type).expect("upsert");
        let (hot, _) = search(&db, "drift", 20, 0).expect("search hot");
        assert_eq!(hot.len(), 1);

        // Both paths index identical fields — no drift between hot and rebuild.
        assert_eq!(rebuilt[0].project_id, hot[0].project_id);
        assert_eq!(rebuilt[0].project_name, hot[0].project_name);
        assert_eq!(rebuilt[0].category, hot[0].category);
        assert_eq!(rebuilt[0].description, hot[0].description);
        assert_eq!(rebuilt[0].op_type, hot[0].op_type);
        assert_eq!(rebuilt[0].source_type, hot[0].source_type);
    }

    #[test]
    fn hot_reindex_keeps_search_results_consistent() {
        // The coordinator DB is a single Connection behind one Mutex, so an
        // upsert and a search serialize at the Rust lock (not via WAL reader
        // isolation). This asserts correctness under interleave: a search after
        // each hot upsert observes a consistent, monotonically growing index —
        // no torn or duplicated rows.
        let db = setup_db();
        for seq in 1..=5u64 {
            let op = serde_json::json!({
                "project_id": format!("proj-{seq}"),
                "reason": format!("interleaved indexing entry {seq}")
            });
            upsert_feed_entry(&db, seq, &op, "SourceBecameStale").expect("upsert");
            let (results, total) = search(&db, "interleaved", 50, 0).expect("search");
            assert_eq!(
                total, seq,
                "search must observe each hot upsert immediately"
            );
            assert_eq!(results.len() as u64, seq);
        }
    }

    #[test]
    fn rebuild_from_feed_still_repairs() {
        let db = setup_db();
        let op = serde_json::json!({
            "project_id": "proj-repair",
            "reason": "repairable index entry"
        });
        let row = feed_row(0, op, "repairhash");
        db.insert_feed_entry(&row).expect("insert");

        // Simulate a corrupted/empty index (e.g. a node that ingested the feed
        // before the index existed, or a Phase D M17 migration reset).
        clear_all(&db).expect("clear");
        let (_, before) = search(&db, "repairable", 20, 0).expect("search before");
        assert_eq!(before, 0, "index empty before repair");

        // The repair path repopulates from the durable feed.
        let n = rebuild_from_feed(&db).expect("rebuild");
        assert_eq!(n, 1);
        let (results, after) = search(&db, "repairable", 20, 0).expect("search after");
        assert_eq!(after, 1, "rebuild_from_feed must repopulate the index");
        assert_eq!(results[0].source_type, "feed");
    }
}
