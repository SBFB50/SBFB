// SPDX-License-Identifier: AGPL-3.0-or-later
//! Feed materializer — projects a `PublicRegistryView` from the
//! append-only feed log.
//!
//! The materializer reads feed entries (via [`FeedStore`][super::public_feed])
//! and folds them into a `HashMap<project_id, ProjectFeedStatus>`.
//! A persistent cursor (`last_seq`, `last_entry_hash`) enables
//! incremental re-materialization after restart.
//!
//! # Fold order (wf4, Sprint 81 Phase A)
//!
//! Local `seq` is an arrival-order artifact: remote entries are
//! inserted with the local AUTOINCREMENT (`feed_sync` ingest), so two
//! nodes holding the same entry set can store them under different
//! `seq`. Folding in `seq` order therefore diverges cross-node.
//!
//! The fold instead applies entries in a deterministic, content-derived
//! order over the per-author chain forest (see [`ordered_for_fold`]):
//! within one author the authoritative order is the `prev_hash` chain
//! walked from genesis (never the backdatable wall-clock timestamp);
//! across authors, concurrent chain heads are k-way merged with the
//! tie-break key `(timestamp, author_pubkey, entry_hash)` — every
//! component is part of the signed canonical form, so the resulting
//! order is identical on every node regardless of arrival order.
//! Last-write-wins *within that order* is the monotonic guarantee.

use std::collections::{HashMap, HashSet};

use crate::db::CoordinatorDb;
use crate::public_feed::{FEED_FORMAT_VERSION, FeedEntry, PublicFeedOperation, try_parse_op};

/// Per-project status derived from the feed.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectFeedStatus {
    pub published: bool,
    pub source_stale: bool,
    pub latest_release_hash: Option<String>,
    pub repo_url: Option<String>,
    pub last_updated: u64,
}

/// Content-derived tie-break key for the fold order (wf4).
///
/// All three components live in the signed `FeedEntryCanonical`, so
/// the key is byte-identical cross-node. `entry_hash` makes the key
/// unique (same author + same hash = deduplicated earlier), so the
/// k-way merge never faces an ambiguous tie.
type FoldKey = (u64, String, String);

fn fold_key(entry: &FeedEntry) -> FoldKey {
    (
        entry.timestamp,
        entry.author_pubkey.clone(),
        entry.entry_hash.clone(),
    )
}

fn fold_key_ref(entry: &FeedEntry) -> (u64, &str, &str) {
    (
        entry.timestamp,
        entry.author_pubkey.as_str(),
        entry.entry_hash.as_str(),
    )
}

/// Materialized view over the public feed — one entry per project.
#[derive(Debug, Clone, PartialEq)]
pub struct PublicRegistryView {
    pub projects: HashMap<String, ProjectFeedStatus>,
    /// Applied chain tip (`entry_hash`) per author. Supports the
    /// safe-append check in [`materialize_incremental`].
    applied_tips: HashMap<String, String>,
    /// Highest [`FoldKey`] applied so far — the monotonic guard key.
    /// An incoming entry sorting at or below this key cannot be
    /// appended (it would need to *insert* into the fold order), so
    /// the incremental path falls back to a full rebuild.
    max_applied_key: Option<FoldKey>,
    /// True when the last fold left entries unapplied (orphan suffix
    /// waiting for a gap to fill, or a fork-truncated chain).
    /// Appending on top of such a view is unsound — a full rebuild is
    /// forced instead.
    has_unapplied: bool,
}

impl PublicRegistryView {
    fn new() -> Self {
        Self {
            projects: HashMap::new(),
            applied_tips: HashMap::new(),
            max_applied_key: None,
            has_unapplied: false,
        }
    }

    /// Apply one entry and track the fold metadata (applied tip per
    /// author + monotonic guard key). All fold paths go through this
    /// method; `apply` alone never runs outside `ordered_for_fold`
    /// order.
    fn apply_in_order(&mut self, entry: &FeedEntry) {
        self.apply(entry);
        self.applied_tips
            .insert(entry.author_pubkey.clone(), entry.entry_hash.clone());
        let key = fold_key(entry);
        if self.max_applied_key.as_ref().is_none_or(|k| *k < key) {
            self.max_applied_key = Some(key);
        }
    }

    /// Fold one entry into the view. Last-write-wins is safe here
    /// because callers apply entries in `ordered_for_fold` order —
    /// the deterministic order IS the monotonic guarantee (both the
    /// `ReleasePublished` and `SourceBecameStale` arms overwrite, and
    /// both are covered by that order).
    fn apply(&mut self, entry: &FeedEntry) {
        let Some(typed) = try_parse_op(&entry.op) else {
            return;
        };
        match &typed {
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
            PublicFeedOperation::CuratorVouched(_) | PublicFeedOperation::CuratorDisendorsed(_) => {
                // Curator endorsement ops do not affect per-project
                // publish/stale status. They will feed the trust
                // overlay (S70+). For now, acknowledge and skip.
            }
            PublicFeedOperation::SeedAnnounced(_) => {
                // Sprint 74 Phase F: a seed announcement is an availability
                // (reachability) signal, NOT a release/stale event. It feeds
                // the in-memory multi-seed registry (daemon side), never the
                // per-project publish/stale projection. Acknowledge and skip.
            }
        }
    }
}

/// Deterministic fold order over the per-author chain forest (wf4).
///
/// `start_tips` maps an author to the chain tip already applied (used
/// by the incremental safe-append path); an absent author starts at
/// [`GENESIS_PREV_HASH`]. Returns the ordered longest reachable prefix
/// per author, k-way merged, plus a flag telling whether some entries
/// could NOT be ordered (orphan suffix behind a gap, intra-author fork,
/// or duplicate).
///
/// Availability rule (no all-or-nothing): a gap or a fork stops THAT
/// author's chain only — every other author keeps folding. An
/// intra-author fork (equivocation: two distinct entries sharing one
/// `prev_hash`) is rejected hard at the fork point, deterministically,
/// in per-author isolation. Orphan suffixes are applied later, once the
/// gap fills and a full rebuild re-runs the walk.
fn ordered_for_fold_from<'a>(
    entries: &'a [FeedEntry],
    start_tips: &HashMap<String, String>,
) -> (Vec<&'a FeedEntry>, bool) {
    // Dedup by entry_hash — the DB enforces logical uniqueness at
    // ingest (dedup-before-insert), this is defense against callers
    // passing duplicated slices. A dropped duplicate counts as
    // unapplied so the incremental path never trusts the result.
    let mut seen: HashSet<&str> = HashSet::new();
    let mut duplicates = false;
    let mut by_author: HashMap<&str, Vec<&FeedEntry>> = HashMap::new();
    for entry in entries {
        if seen.insert(entry.entry_hash.as_str()) {
            by_author
                .entry(entry.author_pubkey.as_str())
                .or_default()
                .push(entry);
        } else {
            duplicates = true;
        }
    }

    let mut has_unapplied = duplicates;
    let mut chains: Vec<Vec<&FeedEntry>> = Vec::with_capacity(by_author.len());
    for (author, author_entries) in &by_author {
        let mut by_prev: HashMap<&str, Vec<&FeedEntry>> = HashMap::new();
        for entry in author_entries {
            by_prev
                .entry(entry.prev_hash.as_str())
                .or_default()
                .push(entry);
        }

        let mut current: &str = start_tips
            .get(*author)
            .map(String::as_str)
            .unwrap_or(crate::public_feed::GENESIS_PREV_HASH);
        let mut chain: Vec<&FeedEntry> = Vec::new();
        loop {
            match by_prev.get(current) {
                Some(next) if next.len() == 1 => {
                    let entry = next[0];
                    chain.push(entry);
                    current = entry.entry_hash.as_str();
                }
                Some(_fork) => {
                    // Intra-author fork: deterministic hard stop at the
                    // fork point, isolated to this author.
                    has_unapplied = true;
                    break;
                }
                None => break, // chain end, or gap waiting to be filled
            }
        }
        if chain.len() != author_entries.len() {
            has_unapplied = true;
        }
        if !chain.is_empty() {
            chains.push(chain);
        }
    }

    // K-way merge: always emit the chain head with the smallest
    // content-derived key. Keys are unique (entry_hash dedup above),
    // so the merge is deterministic regardless of HashMap iteration
    // order. Linear head scan: K = number of authors, small.
    let total: usize = chains.iter().map(Vec::len).sum();
    let mut order: Vec<&FeedEntry> = Vec::with_capacity(total);
    let mut heads: Vec<usize> = vec![0; chains.len()];
    loop {
        let mut best: Option<usize> = None;
        for (i, chain) in chains.iter().enumerate() {
            if heads[i] >= chain.len() {
                continue;
            }
            best = match best {
                None => Some(i),
                Some(j) => {
                    if fold_key_ref(chain[heads[i]]) < fold_key_ref(chains[j][heads[j]]) {
                        Some(i)
                    } else {
                        Some(j)
                    }
                }
            };
        }
        match best {
            Some(i) => {
                order.push(chains[i][heads[i]]);
                heads[i] += 1;
            }
            None => break,
        }
    }
    (order, has_unapplied)
}

/// [`ordered_for_fold_from`] starting every author at genesis.
fn ordered_for_fold(entries: &[FeedEntry]) -> (Vec<&FeedEntry>, bool) {
    static EMPTY_TIPS: std::sync::OnceLock<HashMap<String, String>> = std::sync::OnceLock::new();
    ordered_for_fold_from(entries, EMPTY_TIPS.get_or_init(HashMap::new))
}

fn fold_all(entries: &[FeedEntry]) -> PublicRegistryView {
    let mut view = PublicRegistryView::new();
    let (ordered, has_unapplied) = ordered_for_fold(entries);
    for entry in ordered {
        view.apply_in_order(entry);
    }
    view.has_unapplied = has_unapplied;
    view
}

/// Materialize the full `PublicRegistryView` from genesis.
///
/// This is a pure projection (fold) over feed entries, applied in the
/// deterministic fold order (see module doc) so the result converges
/// cross-node whatever the local arrival order. It does NOT verify the
/// hash-chain or signatures — ordering and verification are orthogonal;
/// call [`verify_chain`] first if the feed source is untrusted. For
/// local feeds written by [`insert_feed_operation`], the chain is
/// maintained at write time.
pub fn materialize_full(db: &CoordinatorDb) -> Result<PublicRegistryView, String> {
    let entries = crate::public_feed::replay_all(db)?;
    Ok(fold_all(&entries))
}

/// Materialize after verifying the hash-chain and Ed25519 signatures.
///
/// Returns an error if the chain is corrupt or any signature is invalid
/// (`verify_chain` contract, all-or-nothing). The fold itself uses the
/// same deterministic order as [`materialize_full`], so both functions
/// converge on the same view for a valid feed.
pub fn materialize_verified(db: &CoordinatorDb) -> Result<PublicRegistryView, String> {
    let entries = crate::public_feed::replay_all(db)?;
    crate::public_feed::verify_chain(&entries)?;
    Ok(fold_all(&entries))
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
                if new_entries.is_empty() {
                    return match existing_view {
                        Some(v) => Ok(v),
                        None => materialize_up_to(db, last_seq),
                    };
                }

                // Fresh entries are always verified individually,
                // whatever fold path is taken below (incremental
                // verification of newly received entries).
                for entry in &new_entries {
                    crate::public_feed::verify_entry(entry)?;
                }

                let mut view = match existing_view {
                    Some(v) => v,
                    None => materialize_up_to(db, last_seq)?,
                };

                // Safe-append check (wf4): under a content-derived fold
                // order, a late arrival may sort BEFORE already-applied
                // entries — appending it would diverge from a full
                // rebuild. Appending is sound only when (a) the view has
                // no unapplied leftovers, (b) every new entry extends its
                // author's applied tip (no gap, fork, or duplicate), and
                // (c) every new entry sorts strictly after everything
                // applied (monotonic guard key). Otherwise the reordering
                // is resolved by a deterministic full rebuild.
                let (ordered_new, new_has_unapplied) =
                    ordered_for_fold_from(&new_entries, &view.applied_tips);
                let append_is_sound = !view.has_unapplied
                    && !new_has_unapplied
                    && ordered_new.len() == new_entries.len()
                    && ordered_new.iter().all(|entry| {
                        view.max_applied_key.as_ref().is_none_or(|max| {
                            (max.0, max.1.as_str(), max.2.as_str()) < fold_key_ref(entry)
                        })
                    });

                if append_is_sound {
                    for entry in ordered_new {
                        view.apply_in_order(entry);
                    }
                    if let Some(last) = new_entries.last() {
                        db.save_feed_cursor(last.seq, &last.entry_hash)
                            .map_err(|e| format!("cursor save: {e}"))?;
                    }
                    Ok(view)
                } else {
                    // Reordering detected — rebuild from genesis in the
                    // deterministic fold order.
                    let view = materialize_full(db)?;
                    save_cursor_from_db(db)?;
                    Ok(view)
                }
            } else {
                // Hash mismatch — feed was truncated or replaced.
                // Full re-materialization with verify_chain for safety.
                let entries = crate::public_feed::replay_all(db)?;
                crate::public_feed::verify_chain(&entries)?;
                save_cursor_from_db(db)?;
                Ok(fold_all(&entries))
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
    let prefix: Vec<FeedEntry> = entries
        .into_iter()
        .filter(|entry| entry.seq <= up_to_seq)
        .collect();
    Ok(fold_all(&prefix))
}

fn rows_to_entries(rows: Vec<crate::db::FeedEntryRow>) -> Result<Vec<FeedEntry>, String> {
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let op: serde_json::Value =
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
            pow_nonce: None,
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
        FeedEntryCanonical, GENESIS_PREV_HASH, PublicFeedOperation, ReleasePublishedPayload,
        SourceBecameStalePayload, compute_feed_canonical_bytes, compute_feed_entry_hash,
        insert_feed_operation, op_type,
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

    fn sample_release(project_id: &str) -> serde_json::Value {
        sample_release_artifact(project_id, &"b".repeat(64))
    }

    fn sample_release_artifact(project_id: &str, artifact_hash: &str) -> serde_json::Value {
        serde_json::to_value(PublicFeedOperation::ReleasePublished(
            ReleasePublishedPayload {
                project_id: project_id.to_string(),
                repo_url: "https://github.com/org/app".to_string(),
                commit_sha: "a".repeat(40),
                artifact_hash: artifact_hash.to_string(),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
                project_name: None,
                category: None,
            },
        ))
        .unwrap()
    }

    /// Build a signed entry with controlled timestamp and prev_hash —
    /// mirrors what a REMOTE author publishes on its own node.
    fn make_entry(
        kp: &nexus_core_rs::KeyPair,
        op: serde_json::Value,
        timestamp: u64,
        prev_hash: &str,
    ) -> FeedEntry {
        let author_pubkey = pubkey_hex(kp);
        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: op.clone(),
            author_pubkey: author_pubkey.clone(),
            timestamp,
            prev_hash: prev_hash.to_string(),
        };
        let bytes = compute_feed_canonical_bytes(&canonical).unwrap();
        let entry_hash = compute_feed_entry_hash(&canonical).unwrap();
        FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 0,
            op,
            author_pubkey,
            timestamp,
            entry_hash,
            prev_hash: prev_hash.to_string(),
            signature: hex::encode(kp.sign(&bytes)),
            pow_nonce: None,
        }
    }

    /// Ingest a remote entry the way `feed_sync` does: seq is assigned
    /// by the LOCAL arrival order (AUTOINCREMENT) — the out-of-order
    /// source the wf4 fix converges over.
    fn ingest_remote(db: &CoordinatorDb, entry: &FeedEntry) {
        let row = crate::db::FeedEntryRow {
            seq: 0,
            op_type: op_type(&entry.op).unwrap_or("Unknown").to_string(),
            payload: serde_json::to_string(&entry.op).unwrap(),
            author: entry.author_pubkey.clone(),
            signature: entry.signature.clone(),
            entry_hash: entry.entry_hash.clone(),
            prev_hash: entry.prev_hash.clone(),
            created_at: entry.timestamp,
        };
        db.insert_feed_entry(&row).unwrap();
    }

    fn sample_stale(project_id: &str) -> serde_json::Value {
        serde_json::to_value(PublicFeedOperation::SourceBecameStale(
            SourceBecameStalePayload {
                project_id: project_id.to_string(),
                reason: "repo_unreachable".to_string(),
            },
        ))
        .unwrap()
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

    // -- wf4 (Sprint 81 Phase A): cross-node fold convergence --

    #[test]
    fn test_out_of_order_ingest_converges_full() {
        let kp = test_keypair();
        let pa = hex_project(0xa1);
        let e1 = make_entry(
            &kp,
            sample_release_artifact(&pa, &"1".repeat(64)),
            1000,
            GENESIS_PREV_HASH,
        );
        let e2 = make_entry(&kp, sample_stale(&pa), 1001, &e1.entry_hash);
        let e3 = make_entry(
            &kp,
            sample_release_artifact(&pa, &"2".repeat(64)),
            1002,
            &e2.entry_hash,
        );

        let db_fwd = CoordinatorDb::open_in_memory().unwrap();
        for e in [&e1, &e2, &e3] {
            ingest_remote(&db_fwd, e);
        }
        let db_rev = CoordinatorDb::open_in_memory().unwrap();
        for e in [&e3, &e1, &e2] {
            ingest_remote(&db_rev, e);
        }

        let view_fwd = materialize_full(&db_fwd).unwrap();
        let view_rev = materialize_full(&db_rev).unwrap();
        // Entire view (fold metadata included), not just one field
        assert_eq!(view_fwd, view_rev);

        // Causal order wins: e3 (release) is last, so the project is
        // published and NOT stale, whatever the arrival order.
        let status = &view_fwd.projects[&pa];
        assert!(status.published);
        assert!(!status.source_stale);
        assert_eq!(
            status.latest_release_hash.as_deref(),
            Some(&*"2".repeat(64))
        );
        assert_eq!(status.last_updated, 1002);

        // materialize_verified converges on the same view (ordering is
        // shared; verification is orthogonal).
        assert_eq!(materialize_verified(&db_rev).unwrap(), view_fwd);
    }

    #[test]
    fn test_out_of_order_ingest_converges_incremental() {
        let kp = test_keypair();
        let pa = hex_project(0xa2);
        let e1 = make_entry(
            &kp,
            sample_release_artifact(&pa, &"1".repeat(64)),
            1000,
            GENESIS_PREV_HASH,
        );
        let e2 = make_entry(&kp, sample_stale(&pa), 1001, &e1.entry_hash);
        let e3 = make_entry(
            &kp,
            sample_release_artifact(&pa, &"2".repeat(64)),
            1002,
            &e2.entry_hash,
        );

        let db = CoordinatorDb::open_in_memory().unwrap();
        ingest_remote(&db, &e3); // arrives first, predecessors missing
        let view1 = materialize_incremental(&db, None).unwrap();
        assert!(
            view1.projects.is_empty(),
            "orphan suffix must not be applied before the gap fills"
        );

        ingest_remote(&db, &e1);
        ingest_remote(&db, &e2);
        // The late arrivals sort BEFORE the applied frontier —
        // safe-append must refuse and fall back to a full rebuild.
        let view2 = materialize_incremental(&db, Some(view1)).unwrap();
        let full = materialize_full(&db).unwrap();
        assert_eq!(view2, full);
        let status = &view2.projects[&pa];
        assert!(status.published);
        assert!(!status.source_stale);
        assert_eq!(
            status.latest_release_hash.as_deref(),
            Some(&*"2".repeat(64))
        );
    }

    #[test]
    fn test_incremental_key_reorder_on_clean_view_triggers_full_rebuild() {
        // Isolates conjunct (c) of the safe-append check: the view is
        // CLEAN (no unapplied leftovers) and the new entry extends its
        // author's tip (a fresh author from genesis), so the monotonic
        // guard key is the ONLY conjunct that can refuse the append.
        let kp_a = nexus_core_rs::KeyPair::from_secret_bytes(&[5u8; 32]);
        let kp_b = nexus_core_rs::KeyPair::from_secret_bytes(&[6u8; 32]);
        let p = hex_project(0xa6);
        let ea = make_entry(
            &kp_a,
            sample_release_artifact(&p, &"a".repeat(64)),
            100,
            GENESIS_PREV_HASH,
        );
        // Same project, earlier timestamp: in fold order eb sorts
        // BEFORE ea, so ea must win — a naive append of eb would make
        // eb win and diverge from the full rebuild.
        let eb = make_entry(
            &kp_b,
            sample_release_artifact(&p, &"e".repeat(64)),
            50,
            GENESIS_PREV_HASH,
        );

        let db = CoordinatorDb::open_in_memory().unwrap();
        ingest_remote(&db, &ea);
        let view1 = materialize_incremental(&db, None).unwrap();
        assert_eq!(
            view1.projects[&p].latest_release_hash.as_deref(),
            Some(&*"a".repeat(64))
        );

        ingest_remote(&db, &eb);
        let view2 = materialize_incremental(&db, Some(view1)).unwrap();
        let full = materialize_full(&db).unwrap();
        assert_eq!(view2, full);
        assert_eq!(
            view2.projects[&p].latest_release_hash.as_deref(),
            Some(&*"a".repeat(64)),
            "ea (ts=100) must win in fold order even though eb arrived last"
        );
    }

    #[test]
    fn test_cross_author_tie_break_deterministic() {
        let kp_a = nexus_core_rs::KeyPair::from_secret_bytes(&[1u8; 32]);
        let kp_b = nexus_core_rs::KeyPair::from_secret_bytes(&[2u8; 32]);
        let p = hex_project(0xcd);
        // Same project, same timestamp, two authors: the tie-break
        // (timestamp, author_pubkey, entry_hash) is fully content-
        // derived, so the winner is the same on every node.
        let ea = make_entry(
            &kp_a,
            sample_release_artifact(&p, &"a".repeat(64)),
            5000,
            GENESIS_PREV_HASH,
        );
        let eb = make_entry(
            &kp_b,
            sample_release_artifact(&p, &"e".repeat(64)),
            5000,
            GENESIS_PREV_HASH,
        );

        let db1 = CoordinatorDb::open_in_memory().unwrap();
        ingest_remote(&db1, &ea);
        ingest_remote(&db1, &eb);
        let db2 = CoordinatorDb::open_in_memory().unwrap();
        ingest_remote(&db2, &eb);
        ingest_remote(&db2, &ea);

        let v1 = materialize_full(&db1).unwrap();
        let v2 = materialize_full(&db2).unwrap();
        assert_eq!(v1, v2);

        // Last-write-wins in fold order: the LARGER key applies last.
        let winner = if (ea.timestamp, &ea.author_pubkey, &ea.entry_hash)
            > (eb.timestamp, &eb.author_pubkey, &eb.entry_hash)
        {
            "a".repeat(64)
        } else {
            "e".repeat(64)
        };
        assert_eq!(
            v1.projects[&p].latest_release_hash.as_deref(),
            Some(winner.as_str())
        );
    }

    #[test]
    fn test_intra_author_chain_order_beats_backdated_timestamp() {
        let kp = test_keypair();
        let pa = hex_project(0xa3);
        // e2 is causally AFTER e1 but claims an earlier wall-clock
        // timestamp (backdating). The chain rank must win: the final
        // view reflects e2, and an attacker cannot veto a legitimate
        // later update by backdating (no persistent hijack window).
        let e1 = make_entry(
            &kp,
            sample_release_artifact(&pa, &"1".repeat(64)),
            100,
            GENESIS_PREV_HASH,
        );
        let e2 = make_entry(
            &kp,
            sample_release_artifact(&pa, &"2".repeat(64)),
            50,
            &e1.entry_hash,
        );

        let db1 = CoordinatorDb::open_in_memory().unwrap();
        ingest_remote(&db1, &e1);
        ingest_remote(&db1, &e2);
        let db2 = CoordinatorDb::open_in_memory().unwrap();
        ingest_remote(&db2, &e2);
        ingest_remote(&db2, &e1);

        let v1 = materialize_full(&db1).unwrap();
        let v2 = materialize_full(&db2).unwrap();
        assert_eq!(v1, v2);
        assert_eq!(
            v1.projects[&pa].latest_release_hash.as_deref(),
            Some(&*"2".repeat(64))
        );
    }

    #[test]
    fn test_fork_isolation_no_global_error() {
        let kp_a = nexus_core_rs::KeyPair::from_secret_bytes(&[3u8; 32]);
        let kp_b = nexus_core_rs::KeyPair::from_secret_bytes(&[4u8; 32]);
        let pa = hex_project(0xa4);
        let pb = hex_project(0xb4);
        // Author A equivocates: two distinct entries share one
        // prev_hash. A's chain is hard-stopped at the fork point;
        // author B is unaffected (no all-or-nothing error).
        let a1 = make_entry(
            &kp_a,
            sample_release_artifact(&pa, &"1".repeat(64)),
            100,
            GENESIS_PREV_HASH,
        );
        let a2x = make_entry(
            &kp_a,
            sample_release_artifact(&pa, &"2".repeat(64)),
            200,
            &a1.entry_hash,
        );
        let a2y = make_entry(&kp_a, sample_stale(&pa), 200, &a1.entry_hash);
        let b1 = make_entry(
            &kp_b,
            sample_release_artifact(&pb, &"3".repeat(64)),
            300,
            GENESIS_PREV_HASH,
        );

        let db = CoordinatorDb::open_in_memory().unwrap();
        for e in [&a1, &a2x, &a2y, &b1] {
            ingest_remote(&db, e);
        }

        let view = materialize_full(&db).unwrap();
        assert_eq!(
            view.projects[&pa].latest_release_hash.as_deref(),
            Some(&*"1".repeat(64)),
            "author A must be applied only up to the fork point"
        );
        assert!(view.projects[&pb].published, "author B must be unaffected");

        // materialize_verified keeps its strict all-or-nothing
        // contract: the forked feed is an error there.
        assert!(materialize_verified(&db).is_err());
    }

    #[test]
    fn test_orphan_suffix_applied_when_gap_fills() {
        let kp = test_keypair();
        let pa = hex_project(0xa5);
        let e1 = make_entry(
            &kp,
            sample_release_artifact(&pa, &"1".repeat(64)),
            100,
            GENESIS_PREV_HASH,
        );
        let e2 = make_entry(&kp, sample_stale(&pa), 200, &e1.entry_hash);
        let e3 = make_entry(
            &kp,
            sample_release_artifact(&pa, &"2".repeat(64)),
            300,
            &e2.entry_hash,
        );

        let db = CoordinatorDb::open_in_memory().unwrap();
        ingest_remote(&db, &e1);
        ingest_remote(&db, &e3); // gap: e2 missing
        let view = materialize_full(&db).unwrap();
        assert_eq!(
            view.projects[&pa].latest_release_hash.as_deref(),
            Some(&*"1".repeat(64)),
            "suffix behind the gap must stay unapplied"
        );

        ingest_remote(&db, &e2); // gap fills
        let view = materialize_full(&db).unwrap();
        let status = &view.projects[&pa];
        assert_eq!(
            status.latest_release_hash.as_deref(),
            Some(&*"2".repeat(64))
        );
        assert!(!status.source_stale);
        assert_eq!(status.last_updated, 300);
    }
}
