Audit fait sur le code courant du workspace `master`. Test exécuté : `cargo test -p nexus-coordinator-rs --lib --locked search::tests::` -> 12 tests passés, dont les 5 de Phase C.

### Livrable 1 : upsert incremental FTS5 par seq
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/search.rs:153`
- Evidence :
```rust
161:         "INSERT OR REPLACE INTO search_index
162:             (rowid, project_id, project_name, category, description, op_type, source_type)
163:          VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'feed')",
164:         rusqlite::params![
165:             seq as i64,
```
Idempotence réelle : `rowid = seq` + `INSERT OR REPLACE`, donc re-upsert même `seq` remplace la même ligne.

### Livrable 2 : extract_index_fields partagé
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/search.rs:114`, `crates/nexus-coordinator-rs/src/public_feed.rs:32`
- Evidence :
```rust
121:     IndexFields {
122:         project_id: field("project_id"),
123:         project_name: field("project_name"),
126:         category: String::new(),
127:         description: op
```
Description via `reason` puis `comment` : `search.rs:127-130`. Les payloads feed actuels (`ReleasePublished`, `SourceBecameStale`, `CuratorVouched`, `CuratorDisendorsed`) ne portent pas `project_name`/`category` dans `public_feed.rs:32-70`; le défaut vide est donc conservé.

### Livrable 3 : rebuild_from_feed réutilise le hot upsert
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/search.rs:183`
- Evidence :
```rust
187:     let entries = db.get_feed_entries()?;
188:     let mut indexed = 0usize;
189:     for entry in &entries {
190:         let op: serde_json::Value = serde_json::from_str(&entry.payload).unwrap_or_default();
191:         upsert_feed_entry(db, entry.seq, &op, &entry.op_type)?;
```
`rebuild_from_feed` supprime d’abord les lignes `source_type='feed'` (`search.rs:184-185`), puis repopule via `upsert_feed_entry`, donc même extracteur et même `rowid=seq`.

### Livrable 4 : appel hot dans feed_sync après insert, même lock
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/feed_sync.rs:232`, `crates/nexus-shell-daemon/src/feed_sync.rs:260`
- Evidence :
```rust
260:     match db.insert_feed_entry(&row) {
261:         Ok(seq) => {
268:             if let Err(e) = nexus_coordinator_rs::search::upsert_feed_entry(
269:                 &db,
270:                 seq,
```
Le `MutexGuard` `db` est acquis à `feed_sync.rs:232-238` et reste dans le même scope jusqu’à la fin de la fonction. L’échec d’upsert est best-effort : warning seulement à `feed_sync.rs:274-278`, après persistance du feed.

### Livrable 5 : busy_timeout 5s open et open_in_memory
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:259`, `crates/nexus-coordinator-rs/src/db.rs:276`
- Evidence :
```rust
264:         // Explicit 5s busy timeout: a hot feed reindex (Sprint 73 Phase C) may
268:         conn.busy_timeout(std::time::Duration::from_secs(5))?;
279:         // Keep the busy timeout in parity with the on-disk `open` path so test
281:         conn.busy_timeout(std::time::Duration::from_secs(5))?;
```

### Livrable 6 : THREAT_MODEL T-CURATOR-VOUCH mis à jour
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:565`
- Evidence :
```markdown
571: attributable. Le search index est reindexe a chaud a l'ingest
572: (Sprint 73 Phase C, apres les gates dedup + rate-limit) et reste
573: reconstructible au boot — les entries spam admises restent
574: visibles mais attribuables et bornees par le rate limiter.
```

### Livrable 7 : 5 tests search.rs avec assertions utiles
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/search.rs:365`
- Evidence :
```rust
380:         let (results, total) = search(&db, "quantum", 20, 0).expect("search");
381:         assert_eq!(
385:         assert_eq!(results.len(), 1);
386:         assert_eq!(results[0].source_type, "feed");
```
Les 5 tests existent et assertent un comportement réel : hot searchable (`365-387`), idempotence 1 résultat (`389-405`), égalité hot/rebuild sur 6 champs (`408-436`), croissance cohérente interleavée (`439-459`), réparation rebuild depuis feed durable (`462-483`).

### Contrôle scope : pas de M17/triplet en Phase C
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:211`, `crates/nexus-coordinator-rs/src/search.rs:7`
- Evidence :
```rust
211:     // M15: FTS5 search index (Sprint 67 Phase B)
223:     // M16: persist the accepted result text so the Operator's network
228:     M::up("ALTER TABLE tasks ADD COLUMN result_text TEXT;"),
229: ];
```
`SearchResult` reste à 7 champs (`search.rs:7-16`) et `search_index` reste le schéma M15 sans colonnes `repo_url`, `commit_sha`, `archive_hash`, `provenance_hash`.

## Resume final
- Total livrables : 7
- Confirmes : 7
- Gaps : 0
- Partiels : 0
