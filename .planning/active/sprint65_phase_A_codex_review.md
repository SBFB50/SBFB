Audit effectué sur l’état local du worktree `master`. Note objective : `COMMONS.md` et `docs/trust/` existent mais sont non suivis par git (`??`), donc confirmés dans le worktree, pas dans `HEAD`.

### Livrable 1 : Auth tier `feed_insert`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/feed_sync.rs:438`, `crates/nexus-shell-daemon/src/http.rs:346`
- Evidence :
```rust
443: let internal = headers
444:     .get("x-sbfb-feed-internal")
446:     == Some("1");
447: if !internal {
449:     StatusCode::FORBIDDEN,
```

### Livrable 2 : Version guard `verify_entry`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/public_feed.rs:445`
- Evidence :
```rust
445: pub fn verify_entry(entry: &FeedEntry) -> Result<(), String> {
446:     if entry.version != FEED_FORMAT_VERSION {
447:         return Err(format!(
448:             "unsupported feed version {}, expected {}",
449:             entry.version, FEED_FORMAT_VERSION
```

### Livrable 3 : Migration raw-op `serde_json::Value`
- Statut : CONFIRME
- Fichier(s) : `public_feed.rs:76`, `public_feed.rs:100`, `public_feed.rs:110`, `public_feed.rs:224`, `feed_materializer.rs:38`, `feed_sync.rs:433`
- Evidence :
```rust
76: pub struct FeedEntry {
79:     pub op: Value,
100: pub struct FeedEntryCanonical {
102:     pub op: Value,
110: pub fn try_parse_op(op: &Value) -> Option<PublicFeedOperation> {
```
Complément vérifié : `validate_feed_operation(op: &Value)` accepte les ops inconnues avec size-check seul (`public_feed.rs:222-235`), `insert_feed_operation(op: Value)` (`public_feed.rs:284-290`), `replay_all()` parse en `Value` (`public_feed.rs:422-427`), le materializer ignore les ops inconnues (`feed_materializer.rs:38-40`), `feed_sync` reçoit `serde_json::Value` (`feed_sync.rs:433-435`).

### Livrable 4 : `PUBLIC_FEED_SPEC.md` §9.1 raw-op
- Statut : CONFIRME
- Fichier(s) : `docs/protocol/PUBLIC_FEED_SPEC.md:307`
- Evidence :
```md
307: ### 9.1 Forward compatibility (raw-op)
312: Since Sprint 65, `FeedEntry.op` is stored as a raw
313: `serde_json::Value` instead of a typed enum.
316: - Nodes **MUST** store and propagate unknown `op_type` values
323: - Nodes **MUST NOT** interpret or act on unknown `op_type` values.
```

### Livrable 5 : `TRUST_TAXONOMY.md`
- Statut : CONFIRME
- Fichier(s) : `docs/trust/TRUST_TAXONOMY.md:9`, `:83`, `:106`
- Evidence :
```md
9: ### N0 — Upload direct
18: ### N1 — Source lisible
28: ### N2 — Provenance auto-attestee
83: ## Dimensions transversales
106: ## Why not OpenSSF Scorecard
```
Les six niveaux N0-N5 sont présents (`:9`, `:18`, `:28`, `:44`, `:57`, `:69`) et les trois dimensions transversales sont présentes (`:87`, `:93`, `:99`).

### Livrable 6 : `COMMONS.md`
- Statut : CONFIRME
- Fichier(s) : `COMMONS.md:7`, `:14`, `:16`, `:23`
- Evidence :
```md
7: Le code du protocole SBFB est sous **AGPL-3.0-or-later**
14: - **Pas de CLA** (Contributor License Agreement).
16: - **Pas de fondation**. Le projet est maintenu par un mainteneur
17:   solo, pattern OpenBSD.
23: Les apps deployees sur le reseau SBFB sont a **source verifiable**
```

### Livrable 7 : Deploy -> feed wiring
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/deploy.rs:72`, `:237`, `:252`, `:276`
- Evidence :
```rust
237: publish_announcement(
250: ).await;
252: // Wire deploy→feed: auto-insert ReleasePublished into the public feed.
254: let release_op = serde_json::to_value(
276: nexus_coordinator_rs::public_feed::insert_feed_operation(
```
Le rejet HTTP est corrigé par `starts_with("https://")` à `deploy.rs:72`.

### Livrable 8 : Tests +7 Rust
- Statut : CONFIRME
- Fichier(s) : `public_feed.rs:1636`, `:1667`, `:1691`, `deploy.rs:803`, `:813`, `:820`, `:828`
- Evidence :
```rust
1661: let result = verify_entry(&entry);
1662: assert!(result.is_err());
1685: assert_eq!(entries.len(), 1);
1687: assert!(verify_chain(&entries).is_ok());
846: assert!(nexus_coordinator_rs::public_feed::validate_feed_operation(&op).is_ok());
```
Tests exécutés : les 3 tests `public_feed` ciblés passent individuellement, et `cargo test -p nexus-shell-daemon deploy_ --locked` passe avec les 4 tests demandés inclus.

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0
- Estimation totale LOC fixes manquants : 0

