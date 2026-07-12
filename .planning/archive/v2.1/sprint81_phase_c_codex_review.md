Audit statique du diff local `master` au-dessus de `c899d54`. Je n’ai pas exécuté les tests.

### Livrable 1 : P2-SIBLING-SYNC-SET
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2560`, `2581`, `2583`, `2638`, `2642`, `2690`, `2701`, `2703`, `2755`, `2758`
- Evidence :
```rust
2638: // S81 Phase C chokepoint: single sync-set entry for ALL arms
2640: match crate::noop_identity::sync_set_entry_in_duress(identity_mode) {
2642:     doc.start_sync(Vec::new()).await.with_context(|| {
```
`boot_storage_namespace` et `boot_feed_namespace` ont le chokepoint après le `match existing`, avec fail-fast. Les discriminateurs `Replica not found` restent intacts aux lignes `2583` et `2703`.

### Livrable 2 : Gate duress unique des 3 docs
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/noop_identity.rs:96`, `136`; `crates/nexus-shell-daemon/src/runtime.rs:516`, `648`, `696`, `744`, `2082`, `2640`, `2756`
- Evidence :
```rust
136: pub fn sync_set_entry_in_duress(mode: IdentityMode) -> SyncSetOutcome {
137:     match mode {
138:         IdentityMode::Normal => SyncSetOutcome::Enter,
139:         IdentityMode::Duress => SyncSetOutcome::Skip,
```
Les trois chemins prod utilisent `identity_mode` issu de `opts.identity_mode` (`runtime.rs:516`) et les branches `Skip` ne loggent pas de marqueur `duress`.

### Livrable 3 : Signatures et tests existants mis à jour
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2543`, `2548`, `2674`, `2678`, `4318`, `4358`, `4397`, `4441`; `crates/nexus-shell-daemon/src/dispatch_loop.rs:694`
- Evidence :
```rust
2543: pub(crate) async fn boot_storage_namespace(
2548:     identity_mode: nexus_core_rs::IdentityMode,
2674: pub(crate) async fn boot_feed_namespace(
2678:     identity_mode: nexus_core_rs::IdentityMode,
```
Les 4 tests A2 passent `IdentityMode::Normal` et le test convergence #5 appelle `open_project_doc_for_dispatch(&docs_a2, IdentityMode::Normal)`.

### Livrable 4 : 6 tests sibling 2-noeuds
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/dispatch_loop.rs:769`, `894`, `932`, `956`, `969`, `982`, `993`, `1003`, `1014`
- Evidence :
```rust
769: async fn run_sibling_reopen_scenario(kind: SiblingKind, reopen: SiblingReopen) -> bool {
894: assert_eq!(doc_a2.id(), ns_id,
932: let converged = loop {
934:     .start_sync(vec![a2_addr.clone()])
```
Les 6 tests couvrent storage/feed en CONTROL, GREEN et duress, avec assertions de convergence/non-convergence.

### Livrable 5 : Test duress project doc
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/dispatch_loop.rs:1031`, `1074`, `1080`, `1096`, `1114`
- Evidence :
```rust
1074: let doc_a2 = crate::runtime::open_project_doc_for_dispatch(
1076:     nexus_core_rs::IdentityMode::Duress,
1080: assert_eq!(doc_a2.id(), doc_id,
1114: assert!(!converged,
```
Le test rouvre le même doc, redial via `start_sync`, puis vérifie que l’écriture post-restart ne converge pas.

### Livrable 6 : Tests DocTicket
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/docs.rs:506`, `511`, `513`, `514`, `519`, `533`
- Evidence :
```rust
511: let ticket = doc.share_write().await.expect("mint write ticket");
513: let parsed: DocsTicket = s.parse().expect("persisted ticket string re-parses");
514: assert_eq!(parsed.capability.id(), doc.id(),
519: assert_eq!(parsed.to_string(), s,
```
Le test hostile vérifie trois chaînes en `Err` aux lignes `534-539`.

### Livrable 7 : Unit test helper
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/noop_identity.rs:161`, `168`
- Evidence :
```rust
161: assert_eq!(
162:     sync_set_entry_in_duress(IdentityMode::Normal),
163:     SyncSetOutcome::Enter
168: fn duress_mode_skips_sync_set_entry() {
```

### Livrable 8 : Groupe nextest two-node-convergence
- Statut : CONFIRME
- Fichier(s) : `.config/nextest.toml:17`, `25`, `26`, `36`, `37`, `44`
- Evidence :
```toml
25: [test-groups]
26: two-node-convergence = { max-threads = 2 }
37: filter = 'package(nexus-shell-daemon) & test(/(convergence_|without_start_sync|reenters_sync_set|duress_skips_sync_set)/)'
44: slow-timeout = { period = "60s", terminate-after = 3 }
```

### Livrable 9 : Recalibrations doc-only 0.98 -> 0.101
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/doc_sync.rs:32`, `crates/nexus-shell-daemon/src/runtime.rs:2030`, `crates/nexus-core-rs/src/docs.rs:390`, `crates/nexus-core-rs/Cargo.toml:42`, `crates/nexus-shell-daemon/src/http.rs:3213`
- Evidence :
```rust
32: //! Recalibrated against iroh-docs 0.101 at the S81 Phase B/C bump —
33: //! mechanism unchanged: broadcast gate `is_syncing` (`live.rs:713`),
34: //! sync-set insert only via `start_sync` (`live.rs:408-414`),
35: //! incoming-sync reject `AbortReason::NotFound` (`state.rs:96-97`).
```
Le texte `remote_info_iter` est cohérent avec la source locale `iroh-1.0.1`: `endpoint.rs:1620` expose `remote_info`, et `rg remote_info_iter` ne trouve rien.

### Livrable 10 : Invariants transverses
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2583`, `2703`; `crates/nexus-core-rs/Cargo.toml:42`, `49`; `crates/nexus-shell-daemon/src/dispatch_loop.rs:560`
- Evidence :
```rust
2583: Err(e) if e.to_string().contains("Replica not found") => None,
2703: Err(e) if e.to_string().contains("Replica not found") => None,
```
`git diff -G'DOMAIN_|_FORMAT_VERSION'` ne retourne aucun hunk. Le diff Cargo ne change qu’un commentaire autour de `url = { workspace = true }`; aucune dépendance ajoutée ni pin Phase B modifié. `git diff -U0` montre uniquement le doc-comment du CONTROL A4 avant `dispatch_loop.rs:560`, pas le corps du test.

## Resume final
- Total livrables : 10
- Confirmes : 10
- Gaps : 0
- Partiels : 0