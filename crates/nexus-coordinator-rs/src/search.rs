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
    // -- Sprint 73 Phase D: provenance triplet (D2) --
    //
    // Carried as UNINDEXED columns (returned, never full-text matchable)
    // so a search hit can drive a fork in S74 (`repo_url@commit_sha` from
    // the forge, or `archive_hash` as the blob fallback) without a second
    // round-trip. `None` for non-release ops (CuratorVouched etc.) and for
    // any pre-M17 index row. These are an output-only DTO: an `Option`
    // already serialises to JSON `null`, which is the runtime tolerance the
    // pre-launch policy asks for — there is no historical wire compat to
    // honour (search_index is local, reconstructible from the feed).
    pub repo_url: Option<String>,
    pub commit_sha: Option<String>,
    /// Mirrors `BrowseEntry.archive_hash` and the S74 fork consumer
    /// (`ProofCardInput.archive_hash`). Sourced from the feed payload's
    /// `artifact_hash` field — see the name bridge in [`extract_index_fields`].
    pub archive_hash: Option<String>,
    pub provenance_hash: Option<String>,
    pub is_open_source: bool,
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

/// The provenance triplet a browse/release row contributes to the index.
///
/// Mirrors the four `BrowseEntry` provenance fields plus `commit_sha` (which
/// only the feed payload carries). All optional: a private/legacy entry has
/// no archive or provenance. Sprint 73 Phase D.
#[derive(Debug, Default, Clone)]
pub struct Provenance<'a> {
    pub repo_url: Option<&'a str>,
    pub commit_sha: Option<&'a str>,
    pub archive_hash: Option<&'a str>,
    pub provenance_hash: Option<&'a str>,
    pub is_open_source: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn index_entry(
    db: &CoordinatorDb,
    project_id: &str,
    project_name: &str,
    category: &str,
    description: &str,
    op_type: &str,
    source_type: &str,
    provenance: &Provenance<'_>,
) -> Result<(), CoordinatorError> {
    db.conn().execute(
        "INSERT INTO search_index
            (project_id, project_name, category, description, op_type, source_type,
             repo_url, commit_sha, archive_hash, provenance_hash, is_open_source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            project_id,
            project_name,
            category,
            description,
            op_type,
            source_type,
            provenance.repo_url,
            provenance.commit_sha,
            provenance.archive_hash,
            provenance.provenance_hash,
            provenance.is_open_source,
        ],
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
        "SELECT project_id, project_name, category, description, op_type, source_type,
                repo_url, commit_sha, archive_hash, provenance_hash, is_open_source,
                bm25(search_index)
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
                // Provenance triplet (Phase D): UNINDEXED columns, `None`
                // for non-release ops / pre-M17 rows. `is_open_source` is
                // read tolerantly (an absent column → `false`).
                repo_url: row.get(6)?,
                commit_sha: row.get(7)?,
                archive_hash: row.get(8)?,
                provenance_hash: row.get(9)?,
                is_open_source: row.get::<_, Option<bool>>(10)?.unwrap_or(false),
                score: row.get(11)?,
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
/// `comment`) becomes matchable `description` text. A `ReleasePublished`
/// op additionally carries the provenance triplet (Sprint 73 Phase D),
/// stored UNINDEXED so a hit can drive a fork (S74).
struct IndexFields {
    project_id: String,
    project_name: String,
    category: String,
    description: String,
    repo_url: Option<String>,
    commit_sha: Option<String>,
    archive_hash: Option<String>,
    provenance_hash: Option<String>,
    is_open_source: bool,
}

/// Extract the FTS5 index fields from a parsed feed operation payload.
///
/// Shared by the hot path ([`upsert_feed_entry`]) and the repair path
/// ([`rebuild_from_feed`]) so the two cannot drift apart. Mirrors the
/// historical boot-rebuild extraction: `project_name`/`category` are empty
/// for current feed ops, `description` comes from `reason` (or `comment`).
///
/// Provenance triplet (Phase D): pulled from a `ReleasePublishedPayload`
/// (`public_feed.rs`); `None` for every other op type. Each value is an
/// optional non-empty string — an absent or empty JSON field yields `None`
/// rather than an empty match.
fn extract_index_fields(op: &serde_json::Value) -> IndexFields {
    let field = |key: &str| {
        op.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    // An optional, non-empty string field. Used for the provenance triplet
    // so a missing field (non-release op) or an empty string both map to
    // `None` — never an empty UNINDEXED column.
    let opt_field = |key: &str| {
        op.get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
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
        repo_url: opt_field("repo_url"),
        commit_sha: opt_field("commit_sha"),
        // NAME BRIDGE (Phase D preflight S4): the feed payload field is
        // `artifact_hash` (`ReleasePublishedPayload.artifact_hash`), while
        // the returned column / S74 fork consumer / `BrowseEntry` all name
        // it `archive_hash`. Read the *source* key here and store it under
        // the *consumer* name. Reading `archive_hash` would silently yield
        // `None` for every real release.
        archive_hash: opt_field("artifact_hash"),
        provenance_hash: opt_field("provenance_hash"),
        is_open_source: op
            .get("is_open_source")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
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
            (rowid, project_id, project_name, category, description, op_type, source_type,
             repo_url, commit_sha, archive_hash, provenance_hash, is_open_source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'feed', ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            seq as i64,
            fields.project_id,
            fields.project_name,
            fields.category,
            fields.description,
            op_type,
            fields.repo_url,
            fields.commit_sha,
            fields.archive_hash,
            fields.provenance_hash,
            fields.is_open_source,
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
        let archive = "dd".repeat(32);
        let prov = "ee".repeat(32);
        index_entry(
            &db,
            "proj1",
            "My Project",
            "gov",
            "A governance tool",
            "",
            "browse",
            &Provenance {
                repo_url: Some("https://github.com/test/proj1"),
                commit_sha: Some("abc1230000000000000000000000000000000000"),
                archive_hash: Some(&archive),
                provenance_hash: Some(&prov),
                is_open_source: true,
            },
        )
        .expect("index");
        let (results, total) = search(&db, "governance", 20, 0).expect("search");
        assert_eq!(total, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].project_id, "proj1");
        assert_eq!(results[0].project_name, "My Project");
        assert_eq!(results[0].source_type, "browse");
        // The browse path carries the provenance triplet too (S74 fork).
        assert_eq!(
            results[0].repo_url.as_deref(),
            Some("https://github.com/test/proj1")
        );
        assert!(results[0].is_open_source);
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
            &Provenance::default(),
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
            &Provenance::default(),
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
                &Provenance::default(),
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
        index_entry(
            &db,
            "p1",
            "Something",
            "cat",
            "desc",
            "",
            "browse",
            &Provenance::default(),
        )
        .expect("index");
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

    // -- Sprint 73 Phase D: provenance triplet enrichment (D2) --

    /// A real `ReleasePublishedPayload` (serialised exactly as the stored
    /// feed op) lands its provenance triplet in the index, applying the
    /// `artifact_hash` → `archive_hash` name bridge. A ReleasePublished op
    /// carries no matchable text (no name/reason), so the row is read back
    /// directly by rowid rather than via a full-text query.
    #[test]
    fn search_result_carries_provenance_triplet() {
        let db = setup_db();
        let payload = crate::public_feed::ReleasePublishedPayload {
            project_id: "proj-rel".to_string(),
            repo_url: "https://github.com/test/rel".to_string(),
            commit_sha: "c".repeat(40),
            artifact_hash: "a".repeat(64),
            provenance_hash: Some("b".repeat(64)),
            is_open_source: true,
        };
        let op = serde_json::to_value(&payload).expect("serialize payload");
        upsert_feed_entry(&db, 7, &op, "ReleasePublished").expect("upsert");

        // A ReleasePublished op carries no matchable text (no name/reason),
        // so read the stored columns back by rowid rather than via search().
        let str_col = |name: &str| -> Option<String> {
            db.conn()
                .query_row(
                    &format!("SELECT {name} FROM search_index WHERE rowid = 7"),
                    [],
                    |row| row.get(0),
                )
                .expect("select column")
        };
        assert_eq!(
            str_col("repo_url").as_deref(),
            Some("https://github.com/test/rel")
        );
        assert_eq!(str_col("commit_sha"), Some("c".repeat(40)));
        // The load-bearing name bridge: the `archive_hash` column is sourced
        // from the payload's `artifact_hash` field, NOT an `archive_hash` key
        // (which the payload does not have). Reading the wrong key would
        // silently yield None for every real release.
        assert_eq!(str_col("archive_hash"), Some("a".repeat(64)));
        assert_eq!(str_col("provenance_hash"), Some("b".repeat(64)));

        let oss: Option<bool> = db
            .conn()
            .query_row(
                "SELECT is_open_source FROM search_index WHERE rowid = 7",
                [],
                |row| row.get(0),
            )
            .expect("select is_open_source");
        assert_eq!(oss, Some(true));
    }

    /// Migration M17 (DROP + recreate with the 4 UNINDEXED provenance
    /// columns) loses no data: a feed entry rebuilt from the durable feed
    /// repopulates WITH the triplet. The columns are UNINDEXED — a MATCH on
    /// the hash value returns nothing.
    #[test]
    fn migration_m17_recreates_index_unindexed() {
        // `open_in_memory` applies every migration including M17; a SELECT of
        // the new columns below would error if M17 had not recreated them.
        let db = setup_db();
        let payload = crate::public_feed::ReleasePublishedPayload {
            project_id: "proj-m17".to_string(),
            repo_url: "https://github.com/test/m17".to_string(),
            commit_sha: "1".repeat(40),
            artifact_hash: "2".repeat(64),
            provenance_hash: None,
            is_open_source: false,
        };
        let op = serde_json::to_value(&payload).expect("serialize");
        let row = feed_row(0, op, "m17hash");
        let seq = db.insert_feed_entry(&row).expect("insert feed");

        // Simulate the post-migration repopulate (DROP/recreate leaves the
        // index empty; the boot rebuild refills it from the durable feed).
        clear_all(&db).expect("clear");
        let n = rebuild_from_feed(&db).expect("rebuild");
        assert_eq!(n, 1);

        let archive: Option<String> = db
            .conn()
            .query_row(
                "SELECT archive_hash FROM search_index WHERE rowid = ?1",
                [seq as i64],
                |r| r.get(0),
            )
            .expect("select archive_hash");
        assert_eq!(
            archive,
            Some("2".repeat(64)),
            "rebuild must repopulate the triplet through M17 (no data loss)"
        );

        let (_, total) = search(&db, &"2".repeat(64), 20, 0).expect("search hash");
        assert_eq!(
            total, 0,
            "archive_hash is UNINDEXED — not full-text matchable"
        );
    }

    /// A non-release op (CuratorVouched, matchable via its `reason`) has no
    /// provenance: the triplet is `None`/`false`, not a crash or empty string.
    #[test]
    fn search_result_null_triplet_for_non_release_op() {
        let db = setup_db();
        let op = serde_json::json!({
            "project_id": "proj-cur",
            "curator_pubkey": "bb".repeat(32),
            "reason": "endorses the photon mapper project"
        });
        upsert_feed_entry(&db, 3, &op, "CuratorVouched").expect("upsert");

        let (results, total) = search(&db, "photon", 20, 0).expect("search");
        assert_eq!(total, 1);
        assert_eq!(results.len(), 1);
        assert!(results[0].repo_url.is_none());
        assert!(results[0].commit_sha.is_none());
        assert!(results[0].archive_hash.is_none());
        assert!(results[0].provenance_hash.is_none());
        assert!(!results[0].is_open_source);
    }

    /// The provenance columns are UNINDEXED: a row carrying a hash is found
    /// by its indexed name/description, never by MATCHing the hash or URL.
    #[test]
    fn enriched_fields_unindexed_not_matchable() {
        let db = setup_db();
        let archive = "f".repeat(64);
        index_entry(
            &db,
            "proj-u",
            "Unindexed Probe",
            "cat",
            "matchable description",
            "",
            "browse",
            &Provenance {
                repo_url: Some("https://example.test/unindexed"),
                archive_hash: Some(&archive),
                ..Default::default()
            },
        )
        .expect("index");

        // Found by its indexed name...
        let (_, by_name) = search(&db, "probe", 20, 0).expect("search name");
        assert_eq!(by_name, 1);
        // ...but never by the hash or the URL (both UNINDEXED).
        let (_, by_hash) = search(&db, &archive, 20, 0).expect("search hash");
        assert_eq!(by_hash, 0, "archive_hash is UNINDEXED");
        let (_, by_repo) =
            search(&db, "https://example.test/unindexed", 20, 0).expect("search repo");
        assert_eq!(by_repo, 0, "repo_url is UNINDEXED");
    }
}
