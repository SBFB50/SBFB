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

pub fn rebuild_from_feed(db: &CoordinatorDb) -> Result<usize, CoordinatorError> {
    db.conn()
        .execute("DELETE FROM search_index WHERE source_type = 'feed'", [])?;

    let entries = db.get_feed_entries()?;
    let mut indexed = 0usize;
    for entry in &entries {
        let payload: serde_json::Value = serde_json::from_str(&entry.payload).unwrap_or_default();
        let project_id = payload
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let project_name = payload
            .get("project_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let description = payload
            .get("reason")
            .or_else(|| payload.get("comment"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        index_entry(
            db,
            project_id,
            project_name,
            "",
            description,
            &entry.op_type,
            "feed",
        )?;
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
}
