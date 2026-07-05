// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 81 Phase F — on-disk redb migration proofs.
//!
//! The S81 bump moved the workspace from iroh-docs 0.98 (redb 2.x) to
//! iroh-docs 0.101 (redb 4.1). Stores written by redb 2.x are on-disk
//! `FILE_FORMAT_VERSION3` already, but their variable-width TUPLE
//! tables carry the pre-redb-3.0 type tag, so redb 4 rejects them with
//! `TableTypeMismatch` and iroh-docs auto-migrates them at open (default
//! `redb-v2-migration` feature, temp-file + swap, one-way, keeps a
//! `.backup-redb-v2-tuples` sibling). iroh-blobs tables are all
//! non-tuple, so `blobs.db` opens under 0.103 with NO migration at all.
//!
//! Three proofs live here:
//! 1. hermetic: a synthetic legacy-tuple-tag docs store (forged with the
//!    `redb_v3` dev-dep — the SAME 3.1.3 already in the lock) migrates on
//!    `Store::persistent` and every row survives (upstream model:
//!    iroh-docs `fs.rs` `test_migration_redb_v2_tuples`).
//! 2. hermetic: a fresh iroh-blobs 0.103 fs store round-trips bytes
//!    across a reload with zero migration artefacts.
//! 3. env-gated EMPIRICAL gate on a FRESH copy of the real VPS store
//!    tarball (`data/vps-store-098/`, gitignored — carries node_key +
//!    NamespaceSecret, NEVER committed): blobs opens untouched, docs
//!    migrates with the backup sibling, both M8 namespaces survive,
//!    coordinator.db (M8/M18/M19) reads back, node_key is byte-identical
//!    (node_id unchanged for Phase H), anchors.json tickets re-parse.
//!    Skips green when the tarball (or `tar`) is absent — CI has neither.
//!    The copy is re-extracted PER RUN: the migration mutates it via the
//!    swap, so reusing an extracted tree would silently test a no-op.

use std::path::{Path, PathBuf};

use iroh_blobs::store::fs::FsStore;
use iroh_docs::store::Store as DocsStore;
use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

// ---------------------------------------------------------------------------
// Table definitions replicated from the PRIVATE upstream migration module
// (iroh-docs-0.101.0/src/store/fs/migrate_redb_v2_tuples.rs, `mod old` /
// `mod new`). They must stay byte-compatible with upstream: same table
// names, same key/value shapes. The `Legacy<T>` wrappers stamp the
// pre-redb-3.0 tuple type tag when forging the fixture.
// ---------------------------------------------------------------------------

type RecordsKey<'a> = (&'a [u8; 32], &'a [u8; 32], &'a [u8]);
type RecordsValue<'a> = (u64, &'a [u8; 64], &'a [u8; 64], u64, &'a [u8; 32]);
type LatestKey<'a> = (&'a [u8; 32], &'a [u8; 32]);
type LatestValue<'a> = (u64, &'a [u8]);
type RecordsByKeyKey<'a> = (&'a [u8; 32], &'a [u8], &'a [u8; 32]);
type NamespacesValue<'a> = (u8, &'a [u8; 32]);

const OLD_RECORDS: redb_v3::TableDefinition<redb_v3::Legacy<RecordsKey>, RecordsValue> =
    redb_v3::TableDefinition::new("records-1");
const OLD_LATEST: redb_v3::TableDefinition<LatestKey, redb_v3::Legacy<LatestValue>> =
    redb_v3::TableDefinition::new("latest-by-author-1");
const OLD_BY_KEY: redb_v3::TableDefinition<redb_v3::Legacy<RecordsByKeyKey>, ()> =
    redb_v3::TableDefinition::new("records-by-key-1");
// namespaces-2 components are fixed-width, so its type tag never changed —
// forged with the PLAIN redb_v3 definition, read back with the plain redb 4
// one. This is the table the boot self-heal ultimately depends on: a
// namespace surviving here is what keeps `open_doc` returning `Some`.
const OLD_NAMESPACES: redb_v3::TableDefinition<&[u8; 32], NamespacesValue> =
    redb_v3::TableDefinition::new("namespaces-2");

const NEW_RECORDS: redb::TableDefinition<RecordsKey, RecordsValue> =
    redb::TableDefinition::new("records-1");
const NEW_LATEST: redb::TableDefinition<LatestKey, LatestValue> =
    redb::TableDefinition::new("latest-by-author-1");
const NEW_BY_KEY: redb::TableDefinition<RecordsByKeyKey, ()> =
    redb::TableDefinition::new("records-by-key-1");
const NEW_NAMESPACES: redb::TableDefinition<&[u8; 32], NamespacesValue> =
    redb::TableDefinition::new("namespaces-2");

/// Suffix appended by the upstream migration to the original docs.redb
/// path (mirrors `runtime::docs_migration_backup_path` in the daemon).
const MIGRATION_BACKUP_SUFFIX: &str = ".backup-redb-v2-tuples";

fn backup_path(docs_redb: &Path) -> PathBuf {
    let mut p = docs_redb.to_owned().into_os_string();
    p.push(MIGRATION_BACKUP_SUFFIX);
    p.into()
}

/// Any `docs.db.migrate*` temp file left in `dir` means the migration's
/// temp-file + swap did not clean up after itself. Prefix replicated
/// from upstream `migrate_redb_v2_tuples.rs:99`
/// (`NamedTempFile::with_prefix_in("docs.db.migrate", dir)`).
fn orphan_migrate_temps(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with("docs.db.migrate"))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn docs_store_with_legacy_tuple_tags_migrates_on_open() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("docs.redb");

    let ns = [1u8; 32];
    let author = [2u8; 32];
    let key: &[u8] = b"kv:phase-f";
    let ns_sig = [3u8; 64];
    let auth_sig = [4u8; 64];
    let hash = [5u8; 32];
    let ns_secret = [9u8; 32];

    // Forge the legacy store exactly as redb 2.x left it on disk: file
    // format v3, tuple tables stamped with the OLD type tag.
    {
        let db = redb_v3::Database::create(&path).expect("forge legacy store");
        let tx = db.begin_write().expect("write tx");
        {
            let mut records = tx.open_table(OLD_RECORDS).expect("records-1 legacy");
            let mut latest = tx.open_table(OLD_LATEST).expect("latest legacy");
            let mut by_key = tx.open_table(OLD_BY_KEY).expect("by-key legacy");
            let mut namespaces = tx.open_table(OLD_NAMESPACES).expect("namespaces-2");
            records
                .insert(
                    (&ns, &author, key),
                    (42u64, &ns_sig, &auth_sig, 7u64, &hash),
                )
                .expect("insert record");
            latest
                .insert((&ns, &author), (42u64, key))
                .expect("insert latest");
            by_key
                .insert((&ns, key, &author), ())
                .expect("insert by-key");
            namespaces
                .insert(&ns, (1u8, &ns_secret))
                .expect("insert namespace");
        }
        tx.commit().expect("commit forge");
    }

    // Pre-migration control: redb 4 must REJECT the tuple tables with
    // TableTypeMismatch — this is the exact failure the migration exists
    // for; if this ever passes, the fixture no longer proves anything.
    {
        let db = redb::Database::create(&path).expect("redb4 opens the v3 file");
        let tx = db.begin_write().expect("write tx");
        let err = tx
            .open_table(NEW_RECORDS)
            .expect_err("legacy tag must mismatch");
        assert!(
            matches!(err, redb::TableError::TableTypeMismatch { .. }),
            "expected TableTypeMismatch, got {err:?}"
        );
        drop(tx);
    }

    // The open under iroh-docs 0.101 triggers the automatic migration.
    let store = DocsStore::persistent(&path).expect("Store::persistent migrates the legacy store");
    drop(store);

    let backup = backup_path(&path);
    assert!(
        backup.exists(),
        "migration must keep the {MIGRATION_BACKUP_SUFFIX} backup sibling"
    );
    assert!(
        orphan_migrate_temps(dir.path()).is_empty(),
        "migration must not leave docs.db.migrate* temp orphans"
    );

    // Post-migration: redb 4 reads every row back, byte-exact.
    {
        let db = redb::Database::create(&path).expect("redb4 opens the migrated store");
        let tx = db.begin_read().expect("read tx");

        let records = tx.open_table(NEW_RECORDS).expect("records-1 migrated");
        let rows: Vec<_> = records
            .iter()
            .expect("iter records")
            .collect::<Result<_, _>>()
            .expect("read records");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0.value(), (&ns, &author, key));
        assert_eq!(rows[0].1.value(), (42u64, &ns_sig, &auth_sig, 7u64, &hash));

        let latest = tx.open_table(NEW_LATEST).expect("latest migrated");
        let rows: Vec<_> = latest
            .iter()
            .expect("iter latest")
            .collect::<Result<_, _>>()
            .expect("read latest");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0.value(), (&ns, &author));
        assert_eq!(rows[0].1.value(), (42u64, key));

        let by_key = tx.open_table(NEW_BY_KEY).expect("by-key migrated");
        let rows: Vec<_> = by_key
            .iter()
            .expect("iter by-key")
            .collect::<Result<_, _>>()
            .expect("read by-key");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0.value(), (&ns, key, &author));

        let namespaces = tx
            .open_table(NEW_NAMESPACES)
            .expect("namespaces-2 migrated");
        let rows: Vec<_> = namespaces
            .iter()
            .expect("iter namespaces")
            .collect::<Result<_, _>>()
            .expect("read namespaces");
        assert_eq!(
            rows.len(),
            1,
            "the namespace row must survive the migration"
        );
        assert_eq!(rows[0].0.value(), &ns);
        assert_eq!(rows[0].1.value(), (1u8, &ns_secret));
    }

    // Idempotence: a second open is a plain open, not a second migration.
    let store = DocsStore::persistent(&path).expect("re-open after migration");
    drop(store);
    assert!(
        orphan_migrate_temps(dir.path()).is_empty(),
        "re-open must not re-run the migration"
    );
}

#[tokio::test]
async fn fresh_blobs_store_round_trips_across_reload() {
    let dir = tempfile::tempdir().expect("tempdir");
    let blobs_dir = dir.path().join("blobs");

    let payload = b"phase-f blobs round-trip".to_vec();
    let hash = {
        let store = FsStore::load(&blobs_dir).await.expect("create fs store");
        let tag = store
            .blobs()
            .add_bytes(payload.clone())
            .await
            .expect("add bytes");
        store.shutdown().await.expect("clean shutdown");
        tag.hash
    };

    // Reload: the entry persists and the bytes read back identical. This
    // pins the 0.103 schema round-trip the real-store gate relies on
    // (schema byte-identical 0.100 -> 0.103, non-tuple tables, so a
    // reload performs NO migration).
    let store = FsStore::load(&blobs_dir).await.expect("reload fs store");
    let data = store
        .blobs()
        .get_bytes(hash)
        .await
        .expect("blob survives the reload");
    assert_eq!(data.as_ref(), payload.as_slice());
    store.shutdown().await.expect("clean shutdown");

    // No migration artefact of any kind may appear for blobs.
    let stray: Vec<_> = std::fs::read_dir(&blobs_dir)
        .expect("read blobs dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("backup") || n.contains("migrate"))
        })
        .collect();
    assert!(
        stray.is_empty(),
        "blobs open must be migration-free: {stray:?}"
    );
}

// ---------------------------------------------------------------------------
// Env-gated empirical gate on the real VPS store copy.
// ---------------------------------------------------------------------------

/// Root of the workspace, resolved from this crate's manifest dir.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("workspace root resolves")
}

/// Listing of (file name, size) under a dir, non-recursive.
fn shallow_listing(dir: &Path) -> Vec<(String, u64)> {
    let mut v: Vec<(String, u64)> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let meta = e.metadata().ok()?;
                    if meta.is_file() {
                        Some((e.file_name().to_string_lossy().into_owned(), meta.len()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[tokio::test]
async fn real_vps_store_copy_migrates_and_survives() {
    // Env gate: the tarball is gitignored (secrets: node_key,
    // NamespaceSecret write-caps, worker.key) so CI never has it; the
    // test then early-returns green, mirroring the relay-gated
    // multi_daemon convention. The EMPIRICAL result only exists where
    // the copy exists (dev machines + local Docker mount).
    let tarball = workspace_root()
        .join("data")
        .join("vps-store-098")
        .join("nexus-grid-store-098-pre-s81a3.tar.gz");
    if !tarball.exists() {
        eprintln!(
            "real_vps_store_copy_migrates_and_survives: tarball absent — skipping (env-gated)"
        );
        return;
    }

    // FRESH extraction per run (load-bearing: the docs migration mutates
    // the copy via rename+persist, a reused tree would test a no-op).
    let scratch = tempfile::tempdir().expect("scratch dir");
    let status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&tarball)
        .arg("-C")
        .arg(scratch.path())
        .status();
    let Ok(status) = status else {
        eprintln!(
            "real_vps_store_copy_migrates_and_survives: no tar binary — skipping (env-gated)"
        );
        return;
    };
    assert!(status.success(), "tar extraction failed");
    let root = scratch.path().join("nexus-grid");
    assert!(root.exists(), "tarball must extract a nexus-grid/ root");

    // Empirical layout of the real store (NEXUS_GRID_ROOT): iroh/ lives
    // UNDER shell-daemon/, .sbfb/ at the nexus-grid root.
    let sd = root.join("shell-daemon");
    let iroh_dir = sd.join("iroh");
    let blobs_dir = iroh_dir.join("blobs");
    let docs_redb = iroh_dir.join("docs.redb");
    let node_key_path = sd.join("node_key");

    let node_key_before = std::fs::read(&node_key_path).expect("node_key present in the copy");
    let blobs_data_before = shallow_listing(&blobs_dir.join("data"));
    assert!(
        !blobs_data_before.is_empty(),
        "the real store carries blob payload files"
    );

    // --- blobs: opens under 0.103 with NO migration (the store is
    // FILE_FORMAT_VERSION3 and every iroh-blobs table is non-tuple).
    // The copy is DIRTY (recovery_required — taken from a live daemon),
    // so this ALSO exercises the redb repair path on real data.
    {
        let store = FsStore::load(&blobs_dir)
            .await
            .expect("blobs.db (real, dirty) opens under iroh-blobs 0.103 without migration");
        store.shutdown().await.expect("clean shutdown");
    }
    let stray: Vec<_> = std::fs::read_dir(&blobs_dir)
        .expect("read blobs dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("backup") || n.contains("migrate"))
        })
        .collect();
    assert!(
        stray.is_empty(),
        "blobs open must create no migration artefact: {stray:?}"
    );
    assert_eq!(
        shallow_listing(&blobs_dir.join("data")),
        blobs_data_before,
        "blob payload files must be untouched by the open"
    );

    // --- docs: the SAME open the daemon boot performs triggers the
    // one-way tuple migration (temp+swap), keeps the backup sibling and
    // leaves no temp orphan.
    {
        let store = DocsStore::persistent(&docs_redb)
            .expect("docs.redb (real, dirty) migrates under iroh-docs 0.101");
        drop(store);
    }
    assert!(
        backup_path(&docs_redb).exists(),
        "docs migration must keep the backup sibling"
    );
    assert!(
        orphan_migrate_temps(&iroh_dir).is_empty(),
        "docs migration must not leave temp orphans"
    );
    // Second open = plain open (no second migration).
    {
        let store = DocsStore::persistent(&docs_redb).expect("re-open after migration");
        drop(store);
    }

    // --- M8 namespaces survive: every namespace_id row in coordinator.db
    // (sbfb-ideas + sbfb-feed) must still exist in the migrated
    // namespaces-2 table — this is exactly what keeps the boot self-heal
    // recreate arm NOT entered (open_doc finds the replica).
    let m8: Vec<(String, Vec<u8>)> = {
        let conn = rusqlite::Connection::open(sd.join("coordinator.db"))
            .expect("coordinator.db opens (WAL replay)");
        let mut stmt = conn
            .prepare("SELECT app_name, namespace_id FROM storage_namespaces ORDER BY app_name")
            .expect("M8 table readable");
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
            })
            .expect("M8 query")
            .collect::<Result<Vec<_>, _>>()
            .expect("M8 rows");
        // M18 keep_online + M19 invites/seed_invite stay readable after the
        // WAL replay (the -wal file carries most of the live state).
        for table in ["keep_online", "invites", "seed_invite"] {
            let count: i64 = conn
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
                .unwrap_or_else(|e| panic!("{table} must be readable post-replay: {e}"));
            eprintln!("real-store gate: {table} rows = {count}");
            // Anti-vacuity pin on THIS versioned tarball (pre-s81a3): the
            // live VPS had one keep_online pin — a zero here would mean the
            // WAL replay silently dropped M18 state, not an empty table.
            if table == "keep_online" {
                assert!(count >= 1, "keep_online must survive the WAL replay");
            }
        }
        rows
    };
    let m8_names: Vec<&str> = m8.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        m8_names.contains(&"sbfb-ideas") && m8_names.contains(&"sbfb-feed"),
        "expected the two M8 keys in the real store, got {m8_names:?}"
    );
    {
        let db = redb::Database::create(&docs_redb).expect("redb4 opens the migrated docs store");
        let tx = db.begin_read().expect("read tx");
        let namespaces = tx
            .open_table(NEW_NAMESPACES)
            .expect("namespaces-2 readable");
        for (app, ns_id) in &m8 {
            let ns: [u8; 32] = ns_id
                .as_slice()
                .try_into()
                .expect("M8 namespace_id is 32 bytes");
            let hit = namespaces.get(&ns).expect("namespaces-2 lookup").is_some();
            assert!(
                hit,
                "namespace for M8 key {app} must survive the migration (self-heal recreate \
                 arm must never be entered on a migrated store)"
            );
        }
    }

    // --- identity + persisted contracts survive by construction (all
    // outside the two redb files) — assert it anyway, it is cheap and it
    // is the Phase H "node_id unchanged" prerequisite.
    assert_eq!(
        std::fs::read(&node_key_path).expect("node_key still present"),
        node_key_before,
        "node_key must be byte-identical (node_id unchanged for Phase H)"
    );
    for f in [
        iroh_dir.join("default-author"),
        root.join(".sbfb").join("directory_revision.json"),
        sd.join("subscriptions.json"),
    ] {
        assert!(
            f.exists(),
            "expected survivor file missing: {}",
            f.display()
        );
    }

    // anchors.json: every persisted locator ticket must still parse as a
    // BlobTicket string under the current lock (S75 boot re-pull path).
    let anchors_raw =
        std::fs::read_to_string(sd.join("anchors.json")).expect("anchors.json present");
    let anchors: serde_json::Value =
        serde_json::from_str(&anchors_raw).expect("anchors.json parses");
    let list = anchors
        .get("anchors")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    // Anti-vacuity pin on THIS versioned tarball: the live VPS had one
    // subscribed anchor locator — an empty list would make the parse
    // loop below prove nothing.
    assert!(
        !list.is_empty(),
        "the real store carries at least one anchor locator"
    );
    for entry in &list {
        let ticket = entry
            .get("ticket")
            .and_then(|t| t.as_str())
            .expect("locator carries a ticket string");
        ticket
            .parse::<iroh_blobs::ticket::BlobTicket>()
            .expect("persisted anchor BlobTicket parses under the current lock");
    }
    eprintln!(
        "real-store gate: PASS (m8_keys={m8_names:?}, anchors={}, blobs_data_files={})",
        list.len(),
        blobs_data_before.len()
    );
}
