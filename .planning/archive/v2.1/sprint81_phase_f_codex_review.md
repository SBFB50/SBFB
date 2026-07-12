### Livrable 1 : helper + garde recreate interrompu
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2557`, `:2574`, `:2685`, `:2813`
- Evidence :
```rust
pub(crate) fn docs_migration_backup_path(iroh_data_dir: &std::path::Path) -> std::path::PathBuf {
    iroh_data_dir.join("docs.redb.backup-redb-v2-tuples")
}
```
```rust
None => {
    refuse_recreate_on_interrupted_migration(
        iroh_data_dir,
        &format!("storage namespace for app {app_name} (ns {ns_id})"),
    )?;
```
```rust
None => {
    refuse_recreate_on_interrupted_migration(
        iroh_data_dir,
        &format!("feed namespace (ns {ns_id})"),
    )?;
```
Le chemin première création est explicitement non gardé (`runtime.rs:2707-2711`, `:2832-2835`).

### Livrable 2 : `iroh_data_dir: Option<&Path>` + call-sites
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2618`, `:2757`, `:696`, `:744`; `crates/nexus-shell-daemon/src/dispatch_loop.rs:796`, `:809`
- Evidence :
```rust
pub(crate) async fn boot_storage_namespace(
    ...
    identity_mode: nexus_core_rs::IdentityMode,
    iroh_data_dir: Option<&std::path::Path>,
) -> Result<crate::storage_api::StorageNamespaceState> {
```
```rust
match boot_storage_namespace(
    ...
    identity_mode,
    Some(iroh_data_dir.as_path()),
)
```
```rust
let st = crate::runtime::boot_storage_namespace(
    ...
    nexus_core_rs::IdentityMode::Normal,
    None,
)
```

### Livrable 3 : test storage refuse recreate
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:4485`
- Evidence :
```rust
std::fs::write(docs_migration_backup_path(iroh_dir.path()), b"backup").unwrap();

let err = boot_storage_namespace(
    ...
    Some(iroh_dir.path()),
)
```
```rust
assert!(
    msg.contains("interrupted redb migration"),
    "diagnosable interrupted-migration marker expected, got: {msg}"
);
assert_eq!(row.namespace_id, stale.to_vec());
```

### Livrable 4 : test feed refuse recreate
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:4534`
- Evidence :
```rust
std::fs::write(docs_migration_backup_path(iroh_dir.path()), b"backup").unwrap();

let err = boot_feed_namespace(
    ...
    Some(iroh_dir.path()),
)
```
```rust
assert!(
    msg.contains("interrupted redb migration"),
    "diagnosable interrupted-migration marker expected, got: {msg}"
);
assert_eq!(row.namespace_id, stale.to_vec());
```

### Livrable 5 : test docs legacy tuple tags
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/tests/store_migration.rs:52`, `:103`, `:144`, `:160`, `:174`, `:225`
- Evidence :
```rust
const OLD_RECORDS: redb_v3::TableDefinition<redb_v3::Legacy<RecordsKey>, RecordsValue> =
    redb_v3::TableDefinition::new("records-1");
const OLD_LATEST: redb_v3::TableDefinition<LatestKey, redb_v3::Legacy<LatestValue>> =
    redb_v3::TableDefinition::new("latest-by-author-1");
```
```rust
assert!(
    matches!(err, redb::TableError::TableTypeMismatch { .. }),
    "expected TableTypeMismatch, got {err:?}"
);
```
```rust
assert!(backup.exists());
assert!(orphan_migrate_temps(dir.path()).is_empty());
assert_eq!(rows[0].1.value(), (1u8, &ns_secret));
```

### Livrable 6 : test fresh blobs reload
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/tests/store_migration.rs:235`
- Evidence :
```rust
let store = FsStore::load(&blobs_dir).await.expect("reload fs store");
let data = store.blobs().get_bytes(hash).await.expect("blob survives the reload");
assert_eq!(data.as_ref(), payload.as_slice());
```
```rust
assert!(
    stray.is_empty(),
    "blobs open must be migration-free: {stray:?}"
);
```

### Livrable 7 : test env-gate copie VPS réelle
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/tests/store_migration.rs:314`, `:324`, `:331`, `:370`, `:398`, `:420`, `:449`, `:477`, `:508`
- Evidence :
```rust
if !tarball.exists() {
    eprintln!("real_vps_store_copy_migrates_and_survives: tarball absent — skipping (env-gated)");
    return;
}
```
```rust
let status = std::process::Command::new("tar")
    .arg("-xzf")
    .arg(&tarball)
    .arg("-C")
```
```rust
assert!(
    m8_names.contains(&"sbfb-ideas") && m8_names.contains(&"sbfb-feed"),
    "expected the two M8 keys in the real store, got {m8_names:?}"
);
```
```rust
assert_eq!(std::fs::read(&node_key_path).expect("node_key still present"), node_key_before);
assert!(!list.is_empty(), "the real store carries at least one anchor locator");
```

### Livrable 8 : dev-deps redb/redb_v3/rusqlite + lock
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/Cargo.toml:167`, `Cargo.lock:5199`, `Cargo.lock:7029`
- Evidence :
```toml
redb = "4.1"
redb_v3 = { package = "redb", version = "3.1" }
rusqlite = { workspace = true }
```
```toml
"redb 3.1.3",
"redb 4.1.0",
"rusqlite",
```
```toml
[[package]]
name = "redb"
version = "3.1.3"
...
version = "4.1.0"
```
Le diff `Cargo.lock` ajoute uniquement ces 3 arêtes à `nexus-core-rs`; pas de package `redb 2.6.3`.

### Livrable 9 : artefact T2
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint81_t2_f_store_migration.json:9`
- Evidence :
```json
"real_store_copy_migration": {
  "verdict": "PASS",
  "criterion": "on a FRESH per-run extraction...",
  "observed": "run 2026-07-06 ...",
  "rollback_note": "the docs migration is ONE-WAY..."
}
```
```json
"hermetic_anchors": {
  "docs_migration": "store_migration::docs_store_with_legacy_tuple_tags_migrates_on_open ...",
  "blobs_round_trip": "store_migration::fresh_blobs_store_round_trips_across_reload ...",
  "recreate_guard": "runtime::tests::boot_storage_namespace_refuses_recreate_on_interrupted_migration + boot_feed_namespace_refuses_recreate_on_interrupted_migration ..."
}
```
Le cinquième test `real_vps_store_copy_migrates_and_survives` est nommé dans `observed` ligne 12; les deux tests runtime sont groupés sous `recreate_guard`.

### Livrable 10 : invariant 0 bump wire
- Statut : CONFIRME
- Fichier(s) : diff limité à `Cargo.lock`, `crates/nexus-core-rs/Cargo.toml`, `crates/nexus-shell-daemon/src/dispatch_loop.rs`, `crates/nexus-shell-daemon/src/runtime.rs`; `crates/nexus-core-rs/tests/store_migration.rs:4`
- Evidence :
```rust
//! The S81 bump moved the workspace from iroh-docs 0.98 (redb 2.x) to
//! iroh-docs 0.101 (redb 4.1). Stores written by redb 2.x are on-disk
//! `FILE_FORMAT_VERSION3` already...
```
Commande ciblée exécutée : `git diff -G '(_VERSION|DOMAIN_.*_V1|FEED_FORMAT_VERSION|TASK_FORMAT_VERSION|ANNOUNCEMENT_VERSION|BLOB_VERSION|CANARY_VERSION)' ...` → `0` ligne de diff. Aucun fichier wire/protocole n’est modifié dans `git diff --name-only`.

### Vérifications ciblées exécutées
- `cargo test -p nexus-core-rs --test store_migration docs_store_with_legacy_tuple_tags_migrates_on_open -- --exact --nocapture` : OK
- `cargo test -p nexus-shell-daemon refuses_recreate_on_interrupted_migration -- --nocapture` : OK, 2 tests
- `cargo test -p nexus-core-rs --test store_migration real_vps_store_copy_migrates_and_survives -- --exact --nocapture` : OK
- `cargo test -p nexus-core-rs --test store_migration fresh_blobs_store_round_trips_across_reload -- --exact --nocapture` : OK

## Résumé final
- Total livrables : 10
- Confirmés : 10
- Gaps : 0
- Partiels : 0