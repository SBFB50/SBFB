Branche vérifiée : `master`. `git diff --stat` montre bien 4 fichiers modifiés, `97 insertions(+)`, `0 deletion(-)`. Tests ciblés exécutés : `cargo test --locked -p sbfb-factory --test operator_server authoring_knowledge`, résultat `2 passed`.

### Livrable 1 : constante animejs seule
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:361`
- Evidence :
```rust
361:/// authoritative. Single source for the manifest list (one edit point per
362:/// pack). animejs pack only at this revision.
363:const AUTHORING_KNOWLEDGE_MANIFESTS: &[&str] = &["docs/factory/knowledge/animejs/MANIFEST.json"];
```

### Livrable 2 : helper `authoring_knowledge`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:368`
- Evidence :
```rust
368:fn authoring_knowledge(root: &std::path::Path) -> Vec<serde_json::Value> {
369:    AUTHORING_KNOWLEDGE_MANIFESTS
370:        .iter()
371:        .map(|rel| file_hash(root, rel))
372:        .collect()
```

### Livrable 3 : champ dans `handle_context_pack`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:424`
- Evidence :
```rust
424:        "process_docs": [
428:            file_hash(root, "CLAUDE.md"),
429:        ],
430:        "authoring_knowledge": authoring_knowledge(root),
431:        "active_artifacts": active_artifacts,
```

### Livrable 4 : dual-write dans `handle_chat_session`
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:678`
- Evidence :
```rust
678:    let context_pack = serde_json::json!({
684:        // handle_chat_session rebuilds its own context_pack literal rather than
686:        // authoring_knowledge() helper at both sites.
687:        "authoring_knowledge": authoring_knowledge(root),
```

### Livrable 5 : invariant D6 et metadonnees seules
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:345`, `crates/sbfb-factory/src/operator_server.rs:437`, `crates/sbfb-factory/src/operator_server.rs:696`
- Evidence :
```rust
346:                "path": rel,
347:                "hash": &hash.to_hex()[..8],
348:                "exists": true,
352:    serde_json::json!({"path": rel, "exists": false})
437:        "chat_history_authoritative": false,
696:        "chat_history_authoritative": false,
```

### Livrable 6 : securite, chemin hardcode uniquement
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:363`, `crates/sbfb-factory/src/operator_server.rs:369`
- Evidence :
```rust
363:const AUTHORING_KNOWLEDGE_MANIFESTS: &[&str] = &["docs/factory/knowledge/animejs/MANIFEST.json"];
369:    AUTHORING_KNOWLEDGE_MANIFESTS
370:        .iter()
371:        .map(|rel| file_hash(root, rel))
```
Le chemin `authoring_knowledge` vient de la constante, pas de `req`.

### Livrable 7 : commentaire provenance Sprint 79 Phase D
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/src/operator_server.rs:358`, `crates/sbfb-factory/src/operator_server.rs:682`
- Evidence :
```rust
358:/// Sprint 79 Phase D — decision D1: the packs live under
359:/// `docs/factory/knowledge/<pack>/` (hashed by provenance, outside any app
360:/// workspace); decision D6: they are consumed/displayed and never
682:        // Sprint 79 Phase D — decision D6: dual-write so chat sessions carry
```
Aucune promesse forward de type `will/adds/lands in Phase` trouvée dans les fichiers concernés.

### Livrable 8 : ligne routing dans les deux skills
- Statut : CONFIRME
- Fichier(s) : `.claude/skills/nexus-phase-preflight/SKILL.md:122`, `.claude/skills/nexus-phase-review/SKILL.md:56`
- Evidence :
```md
122:| UI / animation / design app SBFB | `prompts/agent/app-authoring.md` + `docs/factory/knowledge/` | CSP iframe connect-src 'none', vendorisation UMD same-origin, knowledge consommee non-autoritaire |
56:| UI / animation / design app SBFB | `prompts/agent/app-authoring.md` + `docs/factory/knowledge/` | CSP iframe connect-src 'none', vendorisation UMD same-origin, knowledge consommee non-autoritaire |
```
Comparaison UTF-8 des deux lignes : byte-identique.

### Livrable 9 : deux tests utiles
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/tests/operator_server.rs:729`, `crates/sbfb-factory/tests/operator_server.rs:772`
- Evidence :
```rust
729:fn operator_context_pack_includes_authoring_knowledge() {
748:    assert_eq!(animejs["exists"], true, "animejs MANIFEST should exist");
758:    let bytes = std::fs::read(repo_root.join("docs/factory/knowledge/animejs/MANIFEST.json"))
760:    let expected = blake3::hash(&bytes).to_hex()[..8].to_string();
761:    assert_eq!(
```
```rust
772:fn operator_chat_session_includes_authoring_knowledge() {
780:    let cp = &body["context_pack"];
781:    let ak = cp["authoring_knowledge"].as_array().expect(
784:    assert!(
791:    assert_eq!(cp["chat_history_authoritative"], false);
```

### Livrable 10 : aucune nouvelle dependance
- Statut : CONFIRME
- Fichier(s) : `crates/sbfb-factory/Cargo.toml:14`, `crates/sbfb-factory/Cargo.toml:22`, `Cargo.toml:48`, `Cargo.toml:59`
- Evidence :
```toml
crates/sbfb-factory/Cargo.toml:14:blake3 = { workspace = true }
crates/sbfb-factory/Cargo.toml:22:serde_json = { workspace = true }
Cargo.toml:48:serde_json = "1.0"
Cargo.toml:59:blake3 = "1.5"
```
`git diff --name-only -- Cargo.toml crates/sbfb-factory/Cargo.toml` ne retourne aucun fichier.

## Resume final
- Total livrables : 10
- Confirmes : 10
- Gaps : 0
- Partiels : 0