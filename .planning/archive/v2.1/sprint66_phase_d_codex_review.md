# Sprint 66 Phase D — Codex review

Sprint : 66
Phase : D — orphan recovery + RevocationCache SQLite
Branch : master
HEAD audite : `4986b55` (working tree avant commit `141f3ff`)

---

### Livrable 1 : Migration M14 — table key_rotations + index
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:197-210`
- Evidence :
  ```rust
  // M14: key rotation persistence (Sprint 66 Phase D)
  M::up(
      "CREATE TABLE IF NOT EXISTS key_rotations (
      id              INTEGER PRIMARY KEY AUTOINCREMENT,
      old_pubkey      TEXT NOT NULL,
      ...
  CREATE INDEX IF NOT EXISTS idx_keyrot_old ON key_rotations(old_pubkey);",
  ),
  ```
- 8 colonnes presentes (id, old_pubkey, new_pubkey, timestamp, transition_days, signature, reason, created_at) + index idx_keyrot_old.

### Livrable 2 : Methode insert_key_rotation()
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:924-951`
- Evidence :
  ```rust
  pub fn insert_key_rotation(
      &self,
      old_pubkey: &str,
      new_pubkey: &str,
      timestamp: u64,
      transition_days: u16,
      signature: &str,
      reason: &str,
  ) -> Result<(), CoordinatorError> {
  ```
- INSERT complet avec les 7 colonnes, created_at calcule via SystemTime::now().

### Livrable 3 : Methode load_key_rotations()
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:953-973`
- Evidence :
  ```rust
  pub fn load_key_rotations(&self) -> Result<Vec<KeyRotationRow>, CoordinatorError> {
      let mut stmt = self.conn.prepare(
          "SELECT old_pubkey, new_pubkey, timestamp, transition_days, signature, reason
           FROM key_rotations ORDER BY id ASC",
      )?;
  ```
- Retourne Vec<KeyRotationRow>, ordre par id ASC.

### Livrable 4 : Struct KeyRotationRow
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:992-999`
- Evidence :
  ```rust
  pub struct KeyRotationRow {
      pub old_pubkey: String,
      pub new_pubkey: String,
      pub timestamp: u64,
      pub transition_days: u16,
      pub signature: String,
      pub reason: String,
  }
  ```
- 6 champs avec types corrects.

### Livrable 5 : Fonction populate_cache()
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon-core/src/key_rotation_handler.rs:90-145`
- Evidence :
  ```rust
  pub fn populate_cache(
      cache: &Arc<RwLock<RevocationCache>>,
      rotations: &[(String, String, u64, u16, String)],
  ) -> usize {
      use nexus_core_rs::key_rotation::KeyRotationAnnouncement;
      let mut applied = 0usize;
      let mut guard = match cache.write() { ... };
  ```
- Decode hex → [u8;32], cree KeyRotationAnnouncement, apply_verified, skip invalides avec warn. Retourne count appliques.

### Livrable 6 : Boot daemon — restore RevocationCache depuis SQLite
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:526-562`
- Evidence :
  ```rust
  // 6a-2. Sprint 66 Phase D: restore the RevocationCache from
  //       persisted key rotations in SQLite.
  let revocation_cache =
      nexus_shell_daemon_core::key_rotation_handler::shared_revocation_cache();
  {
      let db = coordinator_db.lock()...;
      match db.load_key_rotations() {
          Ok(rows) if !rows.is_empty() => {
              let tuples: Vec<(String, String, u64, u16, String)> = rows...;
              let applied = ...::populate_cache(&revocation_cache, &tuples);
  ```
- Place apres coordinator_db open (l.519-524). load_key_rotations + populate_cache enchaines. Log info avec total+applied.

### Livrable 7 : Orphan detection dans runtime.rs
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:714-768`
- Evidence :
  ```rust
  // 6c-5b. Sprint 66 Phase D: orphan recovery
  if let Some(ref fs) = feed_sync_state {
      match fs.doc.get_many_by_prefix("feed/").await {
          Ok(doc_entries) => {
              let present_keys: std::collections::HashSet<String> = ...;
              ...
              if !is_genesis && !entry_hash_set.contains(entry.prev_hash.as_str()) {
                  warn!(..., "orphan recovery: skipping broken chain tail");
                  continue;
              }
  ```
- Compare SQLite vs iroh-docs, tail-safe skip (prev_hash invalide), republish orphelins. Log final orphans+recovered.

### Livrable 8 : format_feed_key rendu pub(crate)
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/feed_sync.rs:42`
- Evidence :
  ```rust
  pub(crate) fn format_feed_key(author_hex: &str, seq: u64) -> String {
  ```
- Utilise dans runtime.rs:735 via `crate::feed_sync::format_feed_key(...)`.

### Livrable 9 : Test migration_m14_creates_key_rotations_table
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:1388-1401`
- Evidence :
  ```rust
  fn migration_m14_creates_key_rotations_table() {
      ...
      let count: i64 = db.conn.query_row(
          "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='key_rotations'",
          [], |row| row.get(0),
      ).expect("query");
      assert_eq!(count, 1, "key_rotations table must exist after M14");
  }
  ```
- Assertion utile : verifie existence table dans sqlite_master.

### Livrable 10 : Test key_rotation_insert_and_load
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:1404-1424`
- Evidence :
  ```rust
  fn key_rotation_insert_and_load() {
      ...
      assert_eq!(rows.len(), 1);
      assert_eq!(rows[0].old_pubkey, "aabbcc");
      assert_eq!(rows[0].new_pubkey, "ddeeff");
      assert_eq!(rows[0].timestamp, 1_700_000_000);
      assert_eq!(rows[0].transition_days, 7);
      assert_eq!(rows[0].signature, "sig_hex");
      assert_eq!(rows[0].reason, "test rotation");
  }
  ```
- 7 assertions utiles couvrant le roundtrip complet.

### Livrable 11 : Test key_rotation_survives_reopen
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:1426-1438`
- Evidence :
  ```rust
  fn key_rotation_survives_reopen() {
      let dir = tempfile::tempdir()...;
      { let db = CoordinatorDb::open(&path)...; db.insert_key_rotation(...)...; }
      let db2 = CoordinatorDb::open(&path)...;
      let rows = db2.load_key_rotations()...;
      assert_eq!(rows.len(), 1);
      assert_eq!(rows[0].old_pubkey, "aa");
  }
  ```
- Close + reopen DB. 2 assertions verifient persistence.

### Livrable 12 : Test test_orphan_republish_recovery
- Statut : CONFIRME (apres fix GAP initial)
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2009-2060`
- Evidence :
  ```rust
  async fn test_orphan_republish_recovery() {
      ...
      assert_eq!(db.count_feed_entries().unwrap(), 1,
          "SQLite must have 1 entry before recovery boot");
      ...
      assert!(rt2.feed_handle.is_some(),
          "feed sync must be active after orphan recovery");
      ...
      assert_eq!(db.count_feed_entries().unwrap(), 1,
          "feed entry must survive recovery boot without data loss");
  }
  ```
- GAP initial : smoke test sans assertions. Corrige : 3 assertions (count avant boot, feed_handle active, count apres shutdown).

### Livrable 13 : Test test_key_rotation_persistence_survives_reboot
- Statut : CONFIRME (apres fix GAP initial)
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2064-2098`
- Evidence :
  ```rust
  async fn test_key_rotation_persistence_survives_reboot() {
      ...
      assert_eq!(rt1.revocation_cache().read().unwrap().len(), 0,
          "no rotations before insert");
      ...
      assert_eq!(rt2.revocation_cache().read().unwrap().len(), 1,
          "RevocationCache must contain the persisted rotation after reboot");
      assert!(rt2.revocation_cache().read().unwrap()
          .is_in_transition(&kp.public_bytes(), 1_700_000_000),
          "old key must be in transition after restore");
  }
  ```
- GAP initial : smoke test sans assertions. Corrige : 3 assertions (cache vide avant, len==1 apres reboot, is_in_transition confirme).

---

## Resume final

- Total livrables : 13
- Confirmes : 13
- Gaps initiaux : 2
- Gaps corriges : 2
- Gaps residuels : 0

## Verification post-fix
- 2/2 tests verts apres correction
- cargo clippy : 0 warnings
- cargo fmt : 0 diff
- cargo nextest run --workspace : 1347 passed
