// SPDX-License-Identifier: AGPL-3.0-or-later
//! Feed materializer — projects a `PublicRegistryView` from the
//! append-only feed log.
//!
//! The materializer reads feed entries (via [`FeedStore`][super::public_feed])
//! and folds them into a `HashMap<project_id, ProjectFeedStatus>`.
//! A persistent cursor (`last_seq`, `last_entry_hash`) enables
//! incremental re-materialization after restart.

use std::collections::HashMap;

use crate::db::CoordinatorDb;
use crate::public_feed::{FEED_FORMAT_VERSION, FeedEntry, PublicFeedOperation};

/// Per-project status derived from the feed.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectFeedStatus {
    pub published: bool,
    pub source_stale: bool,
    pub latest_release_hash: Option<String>,
    pub repo_url: Option<String>,
    pub last_updated: u64,
}

/// Materialized view over the public feed — one entry per project.
#[derive(Debug, Clone, PartialEq)]
pub struct PublicRegistryView {
    pub projects: HashMap<String, ProjectFeedStatus>,
}

impl PublicRegistryView {
    fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }

    fn apply(&mut self, entry: &FeedEntry) {
        match &entry.op {
            PublicFeedOperation::ReleasePublished(p) => {
                let status =
                    self.projects
                        .entry(p.project_id.clone())
                        .or_insert(ProjectFeedStatus {
                            published: false,
                            source_stale: false,
                            latest_release_hash: None,
                            repo_url: None,
                            last_updated: 0,
                        });
                status.published = true;
                status.source_stale = false;
                status.latest_release_hash = Some(p.artifact_hash.clone());
                status.repo_url = Some(p.repo_url.clone());
                status.last_updated = entry.timestamp;
            }
            PublicFeedOperation::SourceBecameStale(p) => {
                let status =
                    self.projects
                        .entry(p.project_id.clone())
                        .or_insert(ProjectFeedStatus {
                            published: false,
                            source_stale: false,
                            latest_release_hash: None,
                            repo_url: None,
                            last_updated: 0,
                        });
                status.source_stale = true;
                status.last_updated = entry.timestamp;
            }
        }
    }
}

/// Materialize the full `PublicRegistryView` from genesis (seq=0).
pub fn materialize_full(db: &CoordinatorDb) -> Result<PublicRegistryView, String> {
    let entries = crate::public_feed::replay_all(db)?;
    let mut view = PublicRegistryView::new();
    for entry in &entries {
        view.apply(entry);
    }
    Ok(view)
}

/// Materialize incrementally from a saved cursor.
///
/// Loads the cursor from the database. If the cursor's
/// `last_entry_hash` matches the feed entry at `last_seq`,
/// only entries after `last_seq` are processed. Otherwise,
/// falls back to full materialization from genesis (safety).
///
/// Returns the updated view and saves the new cursor.
pub fn materialize_incremental(
    db: &CoordinatorDb,
    existing_view: Option<PublicRegistryView>,
) -> Result<PublicRegistryView, String> {
    let cursor = db
        .load_feed_cursor()
        .map_err(|e| format!("cursor load: {e}"))?;

    match cursor {
        Some((last_seq, last_hash)) => {
            let stored_hash = entry_hash_at_seq(db, last_seq)?;
            if stored_hash.as_deref() == Some(last_hash.as_str()) {
                let rows = db
                    .get_feed_entries_after_seq(last_seq)
                    .map_err(|e| format!("db error: {e}"))?;
                let new_entries = rows_to_entries(rows)?;

                let mut view = existing_view.unwrap_or_else(|| {
                    materialize_up_to(db, last_seq).unwrap_or_else(|_| PublicRegistryView::new())
                });

                for entry in &new_entries {
                    view.apply(entry);
                }

                if let Some(last) = new_entries.last() {
                    db.save_feed_cursor(last.seq, &last.entry_hash)
                        .map_err(|e| format!("cursor save: {e}"))?;
                }

                Ok(view)
            } else {
                let view = materialize_full(db)?;
                save_cursor_from_db(db)?;
                Ok(view)
            }
        }
        None => {
            let view = materialize_full(db)?;
            save_cursor_from_db(db)?;
            Ok(view)
        }
    }
}

fn entry_hash_at_seq(db: &CoordinatorDb, seq: u64) -> Result<Option<String>, String> {
    let rows = db
        .get_feed_entries()
        .map_err(|e| format!("db error: {e}"))?;
    for row in &rows {
        if row.seq == seq {
            return Ok(Some(row.entry_hash.clone()));
        }
    }
    Ok(None)
}

fn materialize_up_to(db: &CoordinatorDb, up_to_seq: u64) -> Result<PublicRegistryView, String> {
    let entries = crate::public_feed::replay_all(db)?;
    let mut view = PublicRegistryView::new();
    for entry in &entries {
        if entry.seq > up_to_seq {
            break;
        }
        view.apply(entry);
    }
    Ok(view)
}

fn rows_to_entries(rows: Vec<crate::db::FeedEntryRow>) -> Result<Vec<FeedEntry>, String> {
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let op: PublicFeedOperation =
            serde_json::from_str(&row.payload).map_err(|e| format!("payload parse: {e}"))?;
        entries.push(FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: row.seq,
            op,
            author_pubkey: row.author,
            timestamp: row.created_at,
            entry_hash: row.entry_hash,
            prev_hash: row.prev_hash,
            signature: row.signature,
        });
    }
    Ok(entries)
}

fn save_cursor_from_db(db: &CoordinatorDb) -> Result<(), String> {
    let rows = db
        .get_feed_entries()
        .map_err(|e| format!("db error: {e}"))?;
    if let Some(last) = rows.last() {
        db.save_feed_cursor(last.seq, &last.entry_hash)
            .map_err(|e| format!("cursor save: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::public_feed::{
        PublicFeedOperation, ReleasePublishedPayload, SourceBecameStalePayload,
        insert_feed_operation,
    };

    fn test_keypair() -> nexus_core_rs::KeyPair {
        nexus_core_rs::KeyPair::from_secret_bytes(&[42u8; 32])
    }

    fn pubkey_hex(kp: &nexus_core_rs::KeyPair) -> String {
        hex::encode(kp.public_bytes())
    }

    fn sample_release(project_id: &str) -> PublicFeedOperation {
        PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: project_id.to_string(),
            repo_url: "https://github.com/org/app".to_string(),
            commit_sha: "a".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
        })
    }

    fn sample_stale(project_id: &str) -> PublicFeedOperation {
        PublicFeedOperation::SourceBecameStale(SourceBecameStalePayload {
            project_id: project_id.to_string(),
            reason: "repo_unreachable".to_string(),
        })
    }

    #[test]
    fn test_materialize_release_published() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        insert_feed_operation(&db, sample_release("proj-a"), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view = materialize_full(&db).unwrap();
        assert_eq!(view.projects.len(), 1);
        let status = &view.projects["proj-a"];
        assert!(status.published);
        assert!(!status.source_stale);
        let expected_hash = "b".repeat(64);
        assert_eq!(status.latest_release_hash.as_deref(), Some(expected_hash.as_str()));
        assert_eq!(
            status.repo_url.as_deref(),
            Some("https://github.com/org/app")
        );
    }

    #[test]
    fn test_materialize_source_stale() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        insert_feed_operation(&db, sample_release("proj-b"), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_stale("proj-b"), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view = materialize_full(&db).unwrap();
        let status = &view.projects["proj-b"];
        assert!(status.published);
        assert!(status.source_stale);
    }

    #[test]
    fn test_cursor_persist_resume() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        insert_feed_operation(&db, sample_release("proj-c"), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_stale("proj-c"), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_release("proj-d"), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view1 = materialize_incremental(&db, None).unwrap();
        assert_eq!(view1.projects.len(), 2);

        let cursor = db.load_feed_cursor().unwrap().expect("cursor saved");
        assert_eq!(cursor.0, 3);

        insert_feed_operation(&db, sample_stale("proj-d"), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_release("proj-e"), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view2 = materialize_incremental(&db, Some(view1)).unwrap();
        assert_eq!(view2.projects.len(), 3);
        assert!(view2.projects["proj-d"].source_stale);
        assert!(view2.projects["proj-e"].published);

        let full = materialize_full(&db).unwrap();
        assert_eq!(view2, full);
    }
}
