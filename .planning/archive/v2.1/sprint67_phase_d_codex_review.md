Audit effectué sur les fichiers présents en `master`, sans historique de session. Test ciblé exécuté : `cargo test -p sbfb-factory --locked` → 16 tests passés.

### Livrable 1 : `provenance.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/provenance.rs:7`, `:10-16`, `:18-45`, `:49-65`
- Evidence :
```rust
10: pub struct Provenance {
11:     pub schema_version: u32,
12:     pub template_hash: String,
13:     pub variables_hash: String,
14:     pub output_hash: String,
15:     pub generated_at: String,
```
`generate()` calcule `variables_hash` et `output_hash` aux lignes 18-29, `to_json()` utilise `serde_json::to_string_pretty` aux lignes 44-45, et `EXCLUDED_FILES` exclut bien `factory.template.lock` / `factory.provenance.json` aux lignes 7 et 64-65.

### Livrable 2 : Wiring dans `create()`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:105-113`
- Evidence :
```rust
105:     let lock = TemplateLock::generate("static", "1.0.0", &template_files, name, version);
106:     fs::write(out.join("factory.template.lock"), lock.to_json()?)?;
112:     let prov = Provenance::generate(out, &lock.template_hash, &variables)?;
113:     fs::write(out.join("factory.provenance.json"), prov.to_json()?)?;
```

### Livrable 3 : `mod provenance` déclaré
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/main.rs:6-9`
- Evidence :
```rust
6: mod provenance;
7: mod secret_scanner;
8: mod template_engine;
9: mod template_lock;
```

### Livrable 4 : Pattern P52 BlobStore
- Statut : CONFIRME
- Fichier(s) : `docs/rust/PATTERNS.md:2609-2642`, `crates/nexus-core-rs/src/node.rs:111-126`
- Evidence :
```rust
2611: `BlobStore` in `nexus-core-rs/src/node.rs` wraps `MemStore` and
2612: `FsStore` behind a two-variant enum with a manual `Deref` to the
2613: common trait object (`Store`). Callers receive `&Store` from
2614: `Node::blobs_store()` regardless of backing implementation.
```
Les sections “When to use” et “Limitation” sont présentes aux lignes 2634-2640.

### Livrable 5 : Note P2-66-1 feed republish limitation
- Statut : CONFIRME
- Fichier(s) : `docs/rust/PATTERNS.md:2646-2656`, `crates/nexus-shell-daemon/src/runtime.rs:1961`, `:1988-1991`
- Evidence :
```text
2648: `test_feed_republish_at_boot` (runtime.rs l.1961) verifies that the
2649: daemon boots without panic and that `feed_handle.is_some()` after
2650: restart, but does NOT assert that feed entries are actually present
2651: in iroh-docs after republish.
```

### Livrable 6 : Tests unitaires provenance
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/provenance.rs:99-159`
- Evidence :
```rust
100:     fn test_provenance_hash_deterministic() {
114:     fn test_provenance_template_hash_matches_lock() {
126:     fn test_provenance_json_parsable() {
142:     fn test_provenance_excludes_lock_and_provenance_files() {
```
Les tests ont des assertions utiles : égalité des hashes lignes 108-110, hash template ligne 122, JSON/schema/hash length lignes 134-138, exclusion lock/provenance ligne 158.

### Livrable 7 : Test wiring `test_create_generates_provenance`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/template_engine.rs:237-256`
- Evidence :
```rust
243:         let prov_path = out.join("factory.provenance.json");
244:         assert!(prov_path.exists());
247:         assert_eq!(prov["schema_version"], 1);
248:         assert!(prov["output_hash"].as_str().unwrap().len() == 64);
255:         assert_eq!(prov["template_hash"], lock["template_hash"]);
```

## Résumé final
- Total livrables : 7
- Confirmés : 7
- Gaps : 0
- Partiels : 0