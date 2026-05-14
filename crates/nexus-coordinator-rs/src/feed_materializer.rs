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
///
/// This is a pure projection (fold) over feed entries. It does NOT
/// verify the hash-chain or signatures — call [`verify_chain`] first
/// if the feed source is untrusted. For local feeds written by
/// [`insert_feed_operation`], the chain is maintained at write time.
pub fn materialize_full(db: &CoordinatorDb) -> Result<PublicRegistryView, String> {
    let entries = crate::public_feed::replay_all(db)?;
    let mut view = PublicRegistryView::new();
    for entry in &entries {
        view.apply(entry);
    }
    Ok(view)
}

/// Materialize after verifying the hash-chain and Ed25519 signatures.
///
/// Returns an error if the chain is corrupt or any signature is invalid.
pub fn materialize_verified(db: &CoordinatorDb) -> Result<PublicRegistryView, String> {
    let entries = crate::public_feed::replay_all(db)?;
    crate::public_feed::verify_chain(&entries)?;
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

                let mut view = match existing_view {
                    Some(v) => v,
                    None => materialize_up_to(db, last_seq)?,
                };

                for entry in &new_entries {
                    crate::public_feed::verify_entry(entry)?;
                    view.apply(entry);
                }

                if let Some(last) = new_entries.last() {
                    db.save_feed_cursor(last.seq, &last.entry_hash)
                        .map_err(|e| format!("cursor save: {e}"))?;
                }

                Ok(view)
            } else {
                // Hash mismatch — feed was truncated or replaced.
                // Full re-materialization with verify_chain for safety.
                let entries = crate::public_feed::replay_all(db)?;
                crate::public_feed::verify_chain(&entries)?;
                let mut view = PublicRegistryView::new();
                for entry in &entries {
                    view.apply(entry);
                }
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

    fn hex_project(byte: u8) -> String {
        format!("{byte:02x}").repeat(32)
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
        let pa = hex_project(0xaa);
        insert_feed_operation(&db, sample_release(&pa), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view = materialize_full(&db).unwrap();
        assert_eq!(view.projects.len(), 1);
        let status = &view.projects[&pa];
        assert!(status.published);
        assert!(!status.source_stale);
        let expected_hash = "b".repeat(64);
        assert_eq!(
            status.latest_release_hash.as_deref(),
            Some(expected_hash.as_str())
        );
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
        let pb = hex_project(0xbb);
        insert_feed_operation(&db, sample_release(&pb), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_stale(&pb), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view = materialize_full(&db).unwrap();
        let status = &view.projects[&pb];
        assert!(status.published);
        assert!(status.source_stale);
    }

    #[test]
    fn test_cursor_persist_resume() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        let pc = hex_project(0xcc);
        let pd = hex_project(0xdd);
        let pe = hex_project(0xee);

        insert_feed_operation(&db, sample_release(&pc), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_stale(&pc), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_release(&pd), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view1 = materialize_incremental(&db, None).unwrap();
        assert_eq!(view1.projects.len(), 2);

        let cursor = db.load_feed_cursor().unwrap().expect("cursor saved");
        assert_eq!(cursor.0, 3);

        insert_feed_operation(&db, sample_stale(&pd), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_release(&pe), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view2 = materialize_incremental(&db, Some(view1)).unwrap();
        assert_eq!(view2.projects.len(), 3);
        assert!(view2.projects[&pd].source_stale);
        assert!(view2.projects[&pe].published);

        let full = materialize_full(&db).unwrap();
        assert_eq!(view2, full);
    }

    #[test]
    fn test_cursor_hash_mismatch_triggers_full_rebuild() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        let pf = hex_project(0xff);

        insert_feed_operation(&db, sample_release(&pf), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_stale(&pf), &pk, |d| kp.sign(d).to_vec()).unwrap();

        // Save a cursor with a wrong hash to simulate DB replacement
        db.save_feed_cursor(2, "badhash").unwrap();

        // Incremental should detect mismatch, verify chain, and rebuild
        let view = materialize_incremental(&db, None).unwrap();
        assert_eq!(view.projects.len(), 1);
        assert!(view.projects[&pf].source_stale);

        // Cursor should be updated to the real last entry
        let cursor = db.load_feed_cursor().unwrap().expect("cursor updated");
        assert_eq!(cursor.0, 2);
        assert_ne!(cursor.1, "badhash");
    }

    #[test]
    fn test_cursor_persist_reopen_file_db() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        let pg = hex_project(0x11);

        {
            let db = CoordinatorDb::open(&db_path).unwrap();
            insert_feed_operation(&db, sample_release(&pg), &pk, |d| kp.sign(d).to_vec()).unwrap();
            let _ = materialize_incremental(&db, None).unwrap();
        }

        {
            let db = CoordinatorDb::open(&db_path).unwrap();
            insert_feed_operation(&db, sample_stale(&pg), &pk, |d| kp.sign(d).to_vec()).unwrap();
            let view = materialize_incremental(&db, None).unwrap();
            assert_eq!(view.projects.len(), 1);
            assert!(view.projects[&pg].source_stale);

            let full = materialize_full(&db).unwrap();
            assert_eq!(view, full);
        }
    }

    #[test]
    fn test_source_stale_without_release() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        let p_orphan = hex_project(0x22);
        insert_feed_operation(&db, sample_stale(&p_orphan), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view = materialize_full(&db).unwrap();
        assert_eq!(view.projects.len(), 1);
        let status = &view.projects[&p_orphan];
        assert!(!status.published);
        assert!(status.source_stale);
        assert!(status.latest_release_hash.is_none());
    }

    #[test]
    fn test_cursor_restart_consistency() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        let pr1 = hex_project(0x33);
        let pr2 = hex_project(0x44);

        insert_feed_operation(&db, sample_release(&pr1), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_stale(&pr1), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_release(&pr2), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let view_inc = materialize_incremental(&db, None).unwrap();
        let view_full = materialize_full(&db).unwrap();
        assert_eq!(view_inc, view_full);

        let view_restart = materialize_full(&db).unwrap();
        assert_eq!(view_inc, view_restart);
    }

    #[test]
    fn test_incremental_no_existing_view_rebuilds_prefix() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        let ph = hex_project(0x55);

        insert_feed_operation(&db, sample_release(&ph), &pk, |d| kp.sign(d).to_vec()).unwrap();
        let _ = materialize_incremental(&db, None).unwrap();

        insert_feed_operation(&db, sample_stale(&ph), &pk, |d| kp.sign(d).to_vec()).unwrap();

        // Call without existing_view — should rebuild from DB up to cursor, then apply new
        let view = materialize_incremental(&db, None).unwrap();
        assert!(view.projects[&ph].source_stale);

        let full = materialize_full(&db).unwrap();
        assert_eq!(view, full);
    }

    #[test]
    fn test_incremental_verify_per_entry() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        let pi = hex_project(0x66);

        insert_feed_operation(&db, sample_release(&pi), &pk, |d| kp.sign(d).to_vec()).unwrap();
        let _ = materialize_incremental(&db, None).unwrap();

        insert_feed_operation(&db, sample_stale(&pi), &pk, |d| kp.sign(d).to_vec()).unwrap();

        // Corrupt the second entry's signature in the DB
        db.execute_batch_raw(&format!(
            "UPDATE public_feed SET signature = '{}' WHERE seq = 2",
            "00".repeat(64)
        ))
        .unwrap();

        // Incremental should reject the corrupted entry
        let result = materialize_incremental(&db, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("signature verification failed")
        );
    }
}
