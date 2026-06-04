Audit fait sur le worktree courant de `master`, sans historique de session.

### Livrable 1 : Migration M17
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:228`, `:244-259`
- Evidence :
```rust
228:    M::up("ALTER TABLE tasks ADD COLUMN result_text TEXT;"),
244:    M::up(
245:        "DROP TABLE IF EXISTS search_index;
246:    CREATE VIRTUAL TABLE search_index USING fts5(
253:        repo_url UNINDEXED,
254:        commit_sha UNINDEXED,
255:        archive_hash UNINDEXED,
256:        provenance_hash UNINDEXED,
257:        is_open_source UNINDEXED,
```
- `rg "M::up"` compte bien M17 comme 17e entrée, après M16 `result_text`.

### Livrable 2 : `SearchResult` enrichi
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/search.rs:26-33`
- Evidence :
```rust
26:    pub repo_url: Option<String>,
27:    pub commit_sha: Option<String>,
31:    pub archive_hash: Option<String>,
32:    pub provenance_hash: Option<String>,
33:    pub is_open_source: bool,
```

### Livrable 3 : Bridge `artifact_hash` -> `archive_hash`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/public_feed.rs:32-39`, `crates/nexus-coordinator-rs/src/search.rs:213-220`, `:649-653`
- Evidence :
```rust
32:pub struct ReleasePublishedPayload {
34:    pub repo_url: String,
35:    pub commit_sha: String,
36:    pub artifact_hash: String,
```
```rust
213:        // NAME BRIDGE ... the feed payload field is
219:        archive_hash: opt_field("artifact_hash"),
220:        provenance_hash: opt_field("provenance_hash"),
```
```rust
649:        // The load-bearing name bridge...
653:        assert_eq!(str_col("archive_hash"), Some("a".repeat(64)));
```

### Livrable 4 : Ecriture/lecture du triplet
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/search.rs:78-93`, `:253-268`, `:116-142`
- Evidence :
```rust
78:        "INSERT INTO search_index
80:             repo_url, commit_sha, archive_hash, provenance_hash, is_open_source)
89:            provenance.repo_url,
93:            provenance.is_open_source,
```
```rust
253:        "INSERT OR REPLACE INTO search_index
255:             repo_url, commit_sha, archive_hash, provenance_hash, is_open_source)
264:            fields.repo_url,
268:            fields.is_open_source,
```
```rust
116:        "SELECT project_id, project_name, category, description, op_type, source_type,
117:                repo_url, commit_sha, archive_hash, provenance_hash, is_open_source,
137:                repo_url: row.get(6)?,
141:                is_open_source: row.get::<_, Option<bool>>(10)?.unwrap_or(false),
142:                score: row.get(11)?,
```

### Livrable 5 : JSON `search_handler`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:2018-2026`
- Evidence :
```rust
2018:                // Provenance triplet ...
2022:                "repo_url": r.repo_url,
2023:                "commit_sha": r.commit_sha,
2024:                "archive_hash": r.archive_hash,
2025:                "provenance_hash": r.provenance_hash,
2026:                "is_open_source": r.is_open_source,
```

### Livrable 6 : Design note SearchManifest D3
- Statut : CONFIRME
- Fichier(s) : `.planning/research/s73_searchmanifest_index_node_design.md:1-6`, `:100-114`, `:178-191`
- Evidence :
```md
1:# SearchManifest — design de la forme correcte (noeud-index opt-in)
3:**Statut** : **DEFERRED** ... Aucune ligne de code wire
4:n'est livree en S73.
```
```md
107:- **Noeud-index (opt-in explicite)** ...
108:  `index-node` (flag de config, **default OFF**).
114:### §4.2 Forme wire (esquisse — NON codee en S73)
```
- Contrôle code : `rg SearchManifest... crates` ne retourne que `public_feed.rs:78`, un commentaire forward-compat, aucun type/constante/variant wire livré.

### Livrable 7 : PATTERNS §P56
- Statut : CONFIRME
- Fichier(s) : `docs/rust/PATTERNS.md:3005-3080`
- Evidence :
```md
3005:## §P56 — Sprint 73 Phase C/D : FTS5 hot reindex + UNINDEXED provenance triplet (D1/D2)
3011:**(D1) Hot incremental reindex, keyed by feed `seq` as the FTS5 rowid.**
3036:**(D2) Enrich `SearchResult` with the provenance triplet — UNINDEXED
3055:- **Name bridge `artifact_hash` → `archive_hash`.**
3058:  name it `archive_hash`. `extract_index_fields` reads the **source** key
```

### Livrable 8 : 5 nouveaux tests
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/search.rs:620`, `:672`, `:718`, `:740`, `crates/nexus-shell-daemon/src/http.rs:6455`
- Evidence :
```rust
620:    fn search_result_carries_provenance_triplet() {
653:        assert_eq!(str_col("archive_hash"), Some("a".repeat(64)));
672:    fn migration_m17_recreates_index_unindexed() {
709:        assert_eq!(total, 0, "archive_hash is UNINDEXED — not full-text matchable");
```
```rust
718:    fn search_result_null_triplet_for_non_release_op() {
730:        assert!(results[0].repo_url.is_none());
734:        assert!(!results[0].is_open_source);
740:    fn enriched_fields_unindexed_not_matchable() {
764:        assert_eq!(by_hash, 0, "archive_hash is UNINDEXED");
```
```rust
6455:    async fn search_handler_json_includes_triplet() {
6499:        assert_eq!(hit["repo_url"], "https://github.com/test/forkable");
6505:            hit["archive_hash"],
6512:        assert_eq!(hit["is_open_source"], true);
```
- Les assertions sont utiles : bridge, null triplet, UNINDEXED, JSON.

### Invariants transverses
- Pas de bump wire : `FEED_FORMAT_VERSION` reste `1` dans `crates/nexus-coordinator-rs/src/public_feed.rs:20`; `git diff` ne montre aucune définition `*_VERSION` modifiée.
- Reconstructibilité : boot appelle `rebuild_from_feed` dans `crates/nexus-shell-daemon/src/runtime.rs:773-780`.
- Tests lancés : `cargo test -p nexus-coordinator-rs search::tests:: --locked` → 16 passed ; `cargo test -p nexus-shell-daemon search_handler_json_includes_triplet --locked` → 1 passed.

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0