Verdict global : le split est fonctionnellement cohérent et les corps de production sont intacts, mais la phase ne satisfait pas totalement la discipline stricte annoncée. Trois réserves empêchent un `PASS` sans nuance : deux lignes rustdoc non autorisées, un fichier source supplémentaire modifié, et l’absence d’une réexécution indépendante des tests dans cet environnement en lecture seule.

Référence auditée : `master`, `HEAD f7d42bce388c`. `cargo fmt --all --check` et `git diff --check` passent.

### Livrable 1 : nouveau module `publish_api.rs` et move production

- Statut : PARTIEL
- Fichier(s) : `crates/nexus-shell-daemon/src/publish_api.rs:1-565` ; `HEAD:crates/nexus-shell-daemon/src/http.rs:757-801,1067-1517,1808-1848`
- Evidence :
  - SPDX présent ligne 1, bandeau anglais lignes 2-21, tier T0 lignes 16-18 et symboles SHARED restant dans `http.rs` lignes 19-21.
  - Imports demandés présents exactement lignes 23-36.
  - Les 13 symboles existent une seule fois dans le crate, tous dans `publish_api.rs`, aux lignes 40, 73, 79, 96, 168, 192, 224, 247, 422, 452, 462, 497 et 528.
  - Comparaison à `git show HEAD:http.rs`, après neutralisation des trois `pub(crate)` et du re-wrap autorisé :
    - DTOs `HEAD:757-801` → courant `38-82` : exact.
    - Directory `HEAD:1147-1436` → courant `166-455` : exact.
    - Révision `HEAD:1454-1517` → courant `457-520` : exact.
    - Blob `HEAD:1808-1848` → courant `522-565` : exact.
    - `publish_project` : seul écart ci-dessous.

```rust
92: /// adds the resulting [`BrowseEntry`] to the aggregator so it
93: /// appears in the local `/browse` immediately.
94: ///
95: /// [`BrowseEntry`]: nexus_shell_daemon_core::browse::BrowseEntry
96: pub(crate) async fn publish_project(
```

- Écart : les lignes 94-95 n’existent pas dans `HEAD:http.rs:1067-1145`. Elles réparent un lien rustdoc, mais ne font pas partie des seules transformations autorisées. Les corps exécutables restent byte-identiques après normalisation.

### Livrable 2 : invariants de sécurité et anti-rollback

- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/publish_api.rs:96-565`
- Evidence :
  - `publish_project` : après `debug!` ligne 100, la première opération est la porte duress, avant tout appel au chemin gossip.

```rust
108: if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
109:     == crate::noop_identity::PublishOutcome::Noop
110: {
111:     return (StatusCode::OK, Json(PublishResponse { published: false })).into_response();
```

  - Gate D-1 présent lignes 122-132 ; chemin canonique exclusif lignes 143-161. Aucun appel à `wrap_payload_with_pow` dans `publish_project`.

```rust
122: if req.is_open_source && (req.provenance_hash.is_none() || req.repo_url.is_none()) {
123:     return (
124:         StatusCode::BAD_REQUEST,
125:         Json(ErrorResponse {
```

```rust
147: crate::deploy::publish_announcement(
148:     &state,
149:     crate::deploy::AnnouncementParams {
150:         project_id: &project_id,
```

  - Directory : duress avant signature lignes 247-256 ; catalogue `own_entries` lignes 264-265 ; blob-held local ligne 307 ; quatre truncations UTF-8 lignes 316, 321, 325 et 329 ; signature locale ligne 337.
  - L’annonce directory est envoyée directement par `sender.broadcast` lignes 375-385. Aucun `GossipCmd::Outbox` n’existe dans le code production du module.
  - `publish_blob` retourne bien le 503 générique :

```rust
537: if crate::noop_identity::task_dispatch_in_duress(state.identity_mode)
538:     == crate::noop_identity::DispatchOutcome::Reject503
539: {
540:     return (
541:         StatusCode::SERVICE_UNAVAILABLE,
```

  - Anti-rollback : absence de home → `0` lignes 462-475 ; fallback `auth::sbfb_home` lignes 463-468 et 501-506 ; verrou process-wide lignes 497-499 ; incrément et écriture tmp+rename lignes 508-516 ; boot re-announce state-driven lignes 422-439.

### Livrable 3 : 18 tests co-migrés verbatim

- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/publish_api.rs:567-1399` ; blocs sources dans `HEAD:http.rs`
- Evidence : les sept comparaisons sont exactes ligne pour ligne :
  - `HEAD:2805-3138` → `577-910`
  - `HEAD:3487-3550` → `912-975`
  - `HEAD:3666-3872` → `977-1183`
  - `HEAD:4154-4175` → `1185-1206`
  - `HEAD:4372-4432` → `1208-1268`
  - `HEAD:4564-4651` → `1270-1357`
  - `HEAD:7055-7094` → `1359-1398`

```rust
569: use super::*;
570: use axum::body::to_bytes;
571: use axum::http::{Method, Request};
572: use tower::ServiceExt;
573:
574: use crate::http::BrowseListResponse;
575: use crate::test_support::*;
```

- Les 18 tests contiennent chacun au moins une assertion utile, pour un total statique de 62 assertions. Exemple concurrent :

```rust
851: assert_eq!(
852:     revs,
853:     [1, 2],
854:     "concurrent publishes must produce distinct monotone revisions"
855: );
```

- Le compte d’attributs de test est conservé : `HEAD http.rs = 156`, courant `http.rs = 138`, `publish_api.rs = 18`.

### Livrable 4 : deux bumps de visibilité dans `http.rs`

- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:774-782,1033-1047,1900`
- Evidence : le struct reste `pub(crate)` et son seul champ devient `pub(crate)` avec le commentaire demandé.

```rust
777: pub(crate) struct ErrorResponse {
778:     /// `pub(crate)`: the publish handlers (Sprint 82 Phase S,
779:     /// `publish_api.rs`) CONSTRUCT the literal cross-module — the
780:     /// struct alone is not enough, the field must reach them too.
781:     pub(crate) error: String,
```

```rust
1038: pub(crate) fn truncate_on_char_boundary(s: &str, max_bytes: usize) -> String {
1039:     if s.len() <= max_bytes {
1040:         return s.to_string();
1041:     }
```

- La doc et le corps de `truncate_on_char_boundary` sont identiques à `HEAD:http.rs:1438-1452` après retrait du seul `pub(crate)`. L’appel search bare reste ligne 1900. Aucune autre visibilité n’a été augmentée dans le code restant de `http.rs`.

### Livrable 5 : amputation propre de `http.rs`

- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:39-60,370-381,774-782,1033-1047,2974-2977,5402-5764`
- Evidence :
  - Taille exacte : `HEAD = 7554` lignes, courant = `6220`.
  - Numstat : `+21/-1355`, soit `-1334` net.
  - Les 13 définitions déplacées et les 18 tests nommés sont absents de `http.rs`.
  - Les seuls tests utilisant encore publish sont les six tests search et leurs helpers locaux, lignes 5402-5764.
  - `Response` et `info` ont été retirés des imports, lignes 45 et 60.
  - Routes re-pointées sans changement des paths :

```rust
370: .route(
371:     "/api/daemon/publish",
372:     post(crate::publish_api::publish_project),
373: )
374: .route(
```

- Les deux autres routes sont identiques aux lignes 374-381. La bannière blob-serve est réécrite honnêtement lignes 2975-2976.

### Livrable 6 : déclaration du module dans `main.rs`

- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/main.rs:51-55`
- Evidence :

```rust
51: mod named_pipe_server;
52: mod noop_identity;
53: mod panic;
54: mod publish_api;
55: mod quarantine_api;
```

- Déclaration normale, sans `cfg(test)`, au slot alphabétique demandé.

### Livrable 7 : re-point du boot driver

- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:1512-1524`
- Evidence :

```rust
1514: // the discovery window is the very point of the re-emit.
1515: if crate::publish_api::reannounce_directory_at_boot(&boot_driver_state).await {
1516:     info!("producer node directory re-announced at boot");
1517: }
1518: // Sprint 82 Phase A: hold the shared lock across the driver so a
```

- Le contexte voisin, dont `crate::seed_api::run_boot_seed_driver`, est inchangé.

### Livrable 8 : ré-honnêteté des cinq sites documentaires

- Statut : CONFIRME
- Fichier(s) :
  - `docs/security/THREAT_MODEL.md:1038-1040`
  - `docs/rust/PATTERNS.md:1480-1486,3438-3440`
  - `crates/nexus-shell-daemon/src/test_support.rs:573-580`
  - `crates/nexus-shell-daemon/src/http.rs:2974-2977`
- Evidence :
  - THREAT_MODEL réattribue `build_sign_announce_directory` à `publish_api.rs` tout en gardant `own_entries` dans `browse.rs`.
  - Le miroir P59.8 fait la même attribution ligne 3439.
  - La section PoW désigne les deux vrais callers, vérifiés dans le code à `deploy.rs:696` et `publish_api.rs:381`.

```text
1480: - Publish — every outbound announce goes through
1481:   wrap_payload_with_pow(&state, &payload) in
1482:   crates/nexus-shell-daemon/src/http.rs; its callers are
1483:   deploy.rs :: publish_announcement ...
1485:   publish_api.rs :: build_sign_announce_directory ...
```

- Le doc-comment golden indique maintenant la co-localisation lignes 573-577 et la bannière `http.rs` ne revendique plus `publish-blob`.
- Un scan hors `.planning/research` ne trouve plus les références stales ciblées.

### Livrable 9 : golden family intacte et verte

- Statut : PARTIEL
- Fichier(s) : `crates/nexus-shell-daemon/src/test_support.rs:294-345,351-623,628-699`
- Evidence :
  - Les neuf fonctions `golden_http_*` existent aux lignes 351, 394, 427, 469, 502, 544, 582, 628 et 669.
  - Le corps de `golden_http_publish_domain` lignes 581-623 est exactement identique à HEAD. Le diff de `test_support.rs` ne modifie que son doc-comment.
  - Les trois cas attendus sont présents : `publish_empty` ligne 585, `publish_blob_empty` ligne 597 et `directory_publish_empty` ligne 608.
  - Le harness effectue des assertions réelles sur statut, en-têtes et corps :

```rust
308: assert_eq!(
309:     status, case.want_status,
310:     "[golden:{}] status drifted (body: {text})",
311:     case.name
312: );
```

- Écart de vérification : l’environnement est strictement en lecture seule ; une exécution Cargo/nextest écrirait dans `target/` et les tests golden écrivent dans des répertoires temporaires. Je confirme donc l’intégrité et la substance des tests, mais pas leur état « vert » courant ni les totaux Windows `2108` / Docker `2112`.

### Livrable 10 : périmètre strict

- Statut : PARTIEL
- Fichier(s) : `crates/nexus-shell-daemon/src/http.rs:75-137,246,282,762,1012,1038,1131,1275,1283,1540` ; `crates/nexus-shell-daemon-core/src/browse.rs:596-605`
- Evidence :
  - Aucun `Cargo.toml`, `Cargo.lock` ou fichier sous `web/` n’est modifié.
  - `DaemonHttpState` est exactement identique à HEAD.
  - `build_router` et `authed_routes` gardent leurs noms.
  - `BrowseListResponse`, `wrap_payload_with_pow`, pull-resolution, nodes, `trustworthy_open_source`, `index_browse_entry` et `mint_blob_ticket` restent dans `http.rs`.
  - Les corps déplacés n’ont aucun delta logique ; l’écart du livrable 1 est exclusivement rustdoc.
- Écart de périmètre : hors `.planning/research`, un fichier source supplémentaire non prévu dans le ledger des cinq sites est modifié :

```rust
602: /// that the node actually HOLDS the entry's archive blob locally
603: /// (content-addressing = the ownership truth, verrou 4) and caps the catalog
604: /// before signing. See `publish_api::publish_directory`.
605: pub fn own_entries(&self, my_node_id: &str) -> Vec<BrowseEntry> {
```

`HEAD` disait `http::publish_directory`. La correction est documentaire et cohérente, sans changement de logique, mais elle étend le périmètre annoncé comme strict.

## Résumé final

- Total livrables : 10
- Confirmés : 7
- Gaps : 0
- Partiels : 3

Les trois réserves sont : rustdoc non verbatim dans `publish_project`, modification documentaire supplémentaire de `browse.rs`, et absence de réexécution indépendante des suites dans l’environnement read-only.