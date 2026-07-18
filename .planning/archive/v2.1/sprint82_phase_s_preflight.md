# Sprint 82 — Phase S — Préflight G8 (synthèse)

- **Phase** : S — split domaine `publish` → `crates/nexus-shell-daemon/src/publish_api.rs`
- **Date** : 2026-07-16
- **HEAD** : `f7d42bc` — `http.rs` = 7554 l (lecture seule)
- **Cas** : B (pre-code)
- **Verdict** : **PLAN-ADAPT** (bornes plan 1159-1727 PRE-N doublement stales → move-set re-dérivé par NOM ; 4 arbitrages IN/STAY tranchés sur précédents R/O + PO-10 ; **2 bumps de symboles PARTAGÉS** dont 1 NOUVEAU vs Phase R ; aucune décision Day-0/PO contredite → PAS DESIGN-CONFLICT)

Préflight = 6 scans factuels (inv-prod, inv-tests, s2-history, s3-threat, s4-wire-frontier, s1-deps-imports) + 3 critics adversariaux (completeness, shared-symbols, tests-docs-repoints). Le synthétiseur a re-greppé lui-même chaque claim contesté ou load-bearing (voir §Vérifications directes). Les critics PRIMENT sur les scans quand leur preuve est plus récente/précise ; sinon re-grep par le synthétiseur.

---

## 1. Contexte + bornes stales re-dérivées

Phase S = 6ᵉ split de la série N..R (pattern PROUVÉ 5×). Le plan S4 nomme `publish_api.rs` comme destination et donne les bornes `http.rs:1159-1727` — **PRE-N (http.rs = 13130 l à l'époque), doublement stales** (les 3 phases N/O/P/Q/R ont chacune constaté des bornes fausses : R `884-1102` balayait le mauvais domaine ; O tombait dans le corps de `seed_count`). Les bornes 1159-1727 englobent aujourd'hui la **zone frontière S2** (pull-resolution + NodesResponse + index-helpers). **DISCARD** : move-set re-dérivé exclusivement par NOM sur le disque actuel.

Zone `publish` physiquement NON-contiguë : les 3 DTOs (757-801), `publish_project` (1077-1145), le bloc directory (1147-1517) et `publish_blob` (1808-1848) sont séparés par ~4 domaines étrangers (browse/diagnostic entre les DTOs ; pull-resolution 1519-1629 + nodes 1631-1728 + index-chokepoint 1730-1806 entre les helpers révision et `publish_blob`). **Extraction par NOM, jamais par plage.**

### Vérifications directes du synthétiseur (re-grep HEAD f7d42bc)

| Claim | Résultat re-grep | Source contestée |
|---|---|---|
| `publish_project` appelle `wrap_payload_with_pow` | **FAUX** — 1077-1145 route UNIQUEMENT via `crate::deploy::publish_announcement` (:1128) ; 0 appel `wrap_payload_with_pow` dans le corps | PATTERNS.md:1481 déjà stale AVANT S (critic completeness ADJUST-1 + tests-docs A2 confirmés) |
| Callers `wrap_payload_with_pow` | deploy.rs:696 + http.rs:1362 (dans `build_sign_announce_directory`, MOVE) + runtime.rs `_static` (fn distincte) | — |
| `ErrorResponse` struct/champ | 814 `pub(crate) struct`, 815 `error: String` **PRIVÉ (pas de `pub`)** | bump champ requis |
| `truncate_on_char_boundary` | 1443 `fn` PRIVÉE ; doc :1438-1442 dit explicitement « catalog fields … AND … search `q` param [`MAX_SEARCH_QUERY_BYTES`] (CARRY-5) » → dual-domaine | STAY + bump |
| `runtime.rs` caller de `reannounce_directory_at_boot` | 1516 `crate::http::reannounce_directory_at_boot(&boot_driver_state).await` | re-point unique |
| Routes publish | 370 `post(publish_project)`, 371 `post(publish_blob)`, 372 `post(publish_directory)` | re-point full-path |
| `mod` slot | main.rs:53 `mod panic;` / :54 `mod quarantine_api;` → insérer `mod publish_api;` en :54 | — |
| `BrowseListResponse` | 750 `#[cfg(test)]` / 753 `pub struct` (browse-domaine, STAY) | re-import N2 unique |
| Docs 3435 / 1039 | `build_sign_announce_directory` in/dans `http.rs` (`own_entries` reste `browse.rs`) | re-point clean |
| `test_support.rs:573-576` | doc-comment `golden_http_publish_domain` « `publish_blob` … sits ~2000 lines away … scattered domain » | doc-honnêteté stale au move (ADD-1) |
| Lignes tests | vps_authoring **3488**, publish_accepts_...full_provenance **3823**, publish_blob_stores **4155**, daemon_boot_rejects_task_dispatch **4635**, publish_and_gossip **7056** | drifts A3 corrigés |

---

## 2. Synthèses factuelles S1a/S1b/S2/S3/S4

- **S1a SOTA** : N/A (refacto pur, 0 delta lib/API). Pattern extraction-par-NOM outillé standing depuis N.
- **S1b deps/imports** : **0 dépendance nouvelle** (axum/serde/tracing/nexus-core-rs/nexus-shell-daemon-core/tokio + dev-deps tower/tempfile déjà consommés par les `*_api.rs` O/Q/R). Bloc `use` prédit §MOVE-SET. **2 compile-hazards** (ErrorResponse.error champ, truncate_on_char_boundary) — voir §Bumps.
- **S2 histoire** : lignée publish `52d4004` (S12-B pipeline) → `3b7ef54` (Remediation #8, chemin canonique unique) → `479a87c` (S75-A PoW re-mint own-only) → `f6637d3` (S75-B `publish_directory` + revision monotone + anti-spoof) → `1486fc9` (S75-E `build_sign_announce_directory` core + `reannounce_directory_at_boot` state-driven). Gate S16 D-1 (audit finding inline :1095) = invariant audit-driven. Invariant cardinal domaine : « héberger != publier, seeder != auteur » (Phase O `542254b`).
- **S3 threat** : surface **INCHANGÉE** (0 route, 0 trust tier, 0 gate change). `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` ancre par PATH (drift-proof, 0 re-point). Gates duress = early-returns intrinsèques au handler (voyagent verbatim). **1 seul re-point sécurité** : THREAT_MODEL.md:1039.
- **S4 wire+frontier** : **0 wire bump** (DTOs publish = loopback HTTP JSON, PAS gossip canonical ; le type gossip `ProjectAnnouncement`/`PROJECT_ANNOUNCEMENT_VERSION` vit dans `nexus-shell-daemon-core/src/publish.rs`, hors move-set — construit par `deploy::publish_announcement` qui reste dans deploy.rs). **Frontier front N/A** (web ne consomme AUCUNE des 3 routes publish ; nav « Publier » → `/api/v1/deploy-from-repo`, domaine distinct). **0 gate-script cassé** (`check-frontier-contracts.sh`/`check-sharding-docs.sh`/`check-factory-docs.sh` : 0 ancre symbole publish). Census DOMAIN (25, gelé) inaffecté.

---

## 3. MOVE-SET FINAL (dérivé par NOM, lignes disque actuel)

### 3.1 Symboles PRODUCTION → `publish_api.rs` (13)

| # | Symbole | Kind | Lignes (doc→fin) | Vis. actuelle | Vis. post-move | Bump |
|---|---|---|---|---|---|---|
| 1 | `PublishRequest` | pub struct DTO | 757–788 | `pub` | `pub` | — |
| 2 | `PublishResponse` | pub struct DTO | 790–794 | `pub` | `pub` | — |
| 3 | `PublishBlobResponse` | pub struct DTO | 796–801 | `pub` | `pub` | — |
| 4 | `publish_project` | async fn handler | 1067–1145 (fn 1077) | **privée** | **pub(crate)** | ROUTINE |
| 5 | `PublishDirectoryResponse` | struct DTO | 1147–1158 | privée | privée | — |
| 6 | `publish_directory` | async fn handler | 1160–1202 (fn 1173) | **privée** | **pub(crate)** | ROUTINE |
| 7 | `DirectoryPublishOutcome` | enum | 1204–1216 | `pub(crate)` | `pub(crate)` | — |
| 8 | `build_sign_announce_directory` | async fn | 1218–1384 (fn 1228) | `pub(crate)` | `pub(crate)` | — |
| 9 | `reannounce_directory_at_boot` | async fn | 1386–1426 (fn 1403) | `pub(crate)` | `pub(crate)` | — (re-point externe) |
| 10 | `DirectoryRevisionFile` | struct on-disk | 1428–1436 | privée | privée | — |
| 11 | `read_directory_revision` | fn | 1454–1473 (fn 1459) | `pub(crate)` | `pub(crate)` | — |
| 12 | `next_directory_revision` | fn (+ `static REVISION_LOCK`) | 1475–1517 (fn 1494) | privée | privée | — |
| 13 | `publish_blob` | async fn handler | 1808–1848 (fn 1814) | **privée** | **pub(crate)** | ROUTINE |

**≈ 511 l prod** + bloc test co-migré.

**Tranches / contiguïté** :
- **Bloc A 1147–1517** (items 5→12) contigu **AVEC îlot-STAY 1438–1452** (`truncate_on_char_boundary`) — extraction par NOM ⇒ îlot **sauté proprement**, laissé en place.
- **Singletons non-contigus** : DTOs 757–801 (items 1-3), `publish_project` 1067–1145 (item 4), `publish_blob` 1808–1848 (item 13).

### 3.2 Symboles PARTAGÉS restant dans http.rs (STAY) — 2 bumps + 4 zéro-bump

| Symbole | Ligne | Vis. actuelle | Décision | Coût | Consommateurs (preuve STAY) |
|---|---|---|---|---|---|
| **`ErrorResponse.error` (champ)** | 815 | **champ PRIVÉ** (struct 814 `pub(crate)`) | **STAY + bump champ → `pub(crate)`** | **1 bump** | Construit à 4 sites déplacés (1106/1198/1825/1841) → E0451 depuis publish_api sans bump. 2 sites restants (840 `runtime_error_to_response`, 1877 `panic_wipe`). Multi-domaine → STAY |
| **`truncate_on_char_boundary`** | 1443 | `fn` PRIVÉE | **STAY + bump → `pub(crate)`** | **1 bump** | Appelée par item 8 (1297/1302/1306/1310, MOVE) ET handler search :2412 (`MAX_SEARCH_QUERY_BYTES`, CARRY-5, S3, STAY). publish_api appelle `crate::http::truncate_on_char_boundary` |
| `index_browse_entry` | 1753 | `pub(crate)` | **STAY** | 0 | deploy.rs:740 + runtime.rs:2345 + tests 2660/2672. **0 caller move-set** (`publish_project` l'atteint transitivement via `deploy::publish_announcement`) |
| `trustworthy_open_source` | 1745 | `pub(crate)` | **STAY** | 0 | runtime.rs:2311 + interne à `index_browse_entry`. 0 caller move-set |
| `mint_blob_ticket` | 2052 | `pub(crate)` | **STAY** | 0 | item 8 (1356) + deploy.rs:682 + seed_api.rs:956. Tri-domaine. publish_api appelle `crate::http::mint_blob_ticket` |
| `wrap_payload_with_pow` | 1046 | `pub(crate)` | **STAY** | 0 | item 8 (1362) + deploy.rs:696. publish_api appelle `crate::http::wrap_payload_with_pow` |

### 3.3 Bump ledger CONSOLIDÉ = **5 bumps** (3 ROUTINE + 2 SHARED-SYMBOL)

- **ROUTINE** (transfo standard per-phase) : `publish_project`, `publish_directory`, `publish_blob` privés → `pub(crate)` (routes les re-pointent full-path). 3 handlers, **0 consommateur code externe** (seulement routes 370-372 + tests + commentaires seed_api.rs:907 / shard_session_http_api.rs:95 / browse.rs:604 ; le `publish_project` de `nexus-test-harness` lib.rs:182 est une **méthode distincte** du client, pas le handler).
- **SHARED-SYMBOL** (classe Phase-R, noteworthy) :
  1. **`ErrorResponse.error` champ privé → `pub(crate)`** — NOUVEAU vs Phase R. R n'a bumpé que le *struct* + le *constructeur* `runtime_error_to_response` (curators_api ne fait que RECEVOIR l'erreur) ; S est différent : les handlers publish **CONSTRUISENT** le littéral `ErrorResponse { error: … }` → le CHAMP doit atteindre `pub(crate)`. **Préférer le bump champ** au constructeur `::new()` (le constructeur réécrirait les 4 corps déplacés → casse le move verbatim ; critic shared-symbols ADJ-2). Lint-clean `private_interfaces` : struct `pub(crate)` (pas `pub`) + champ `pub(crate)` + type `String` public → 0 fuite.
  2. **`truncate_on_char_boundary` privé → `pub(crate)`** (STAY). Coût 1 bump quel que soit STAY/MOVE ; STAY = arête neutre (évite un forward-edge `search → publish_api` + un 2ᵉ move en S3 ; + `rustdoc` : sa doc lie `[`MAX_SEARCH_QUERY_BYTES`]` qui reste http.rs — MOVE casserait le lien).

**Compile-test mental `-D warnings` + `private_interfaces`** : les 5 bumps sont sains (aucune fuite d'interface privée). `DaemonHttpState` (75) = `pub struct` + tous champs `pub` → 0 bump accès `state.*`. `crate::deploy::{publish_announcement, AnnouncementParams}` déjà `pub(crate)` → 0 bump.

### 3.4 Re-points CODE hors module (4)

| Site | Actuel | Après |
|---|---|---|
| routes http.rs:370/371/372 | `post(publish_project|publish_blob|publish_directory)` | `post(crate::publish_api::…)` full-path (paths byte-identiques, restent dans `build_router`) |
| **runtime.rs:1516** | `crate::http::reannounce_directory_at_boot(&boot_driver_state)` | `crate::publish_api::reannounce_directory_at_boot(…)` (déjà `pub(crate)`, 0 bump ; jouxte `crate::seed_api::run_boot_seed_driver` :1523 migré Phase O — même pattern) |
| main.rs | — | insérer `mod publish_api;` en **:54** (entre `mod panic;` :53 et `mod quarantine_api;` :54) |
| seed_api.rs:22-24 | `use crate::http::{DIRECTORY_PULL_TIMEOUT_SECS, DaemonHttpState, directory_pull_providers, find_directory_app_by_project, mint_blob_ticket}` | **INCHANGÉ** (aucun symbole importé ne bouge en S) |

### 3.5 Bloc `use` prédit `publish_api.rs` (prod)

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Publish loopback HTTP domain — extrait verbatim de `http.rs`
//! (Sprint 82 Phase S, discipline PO-10 : tests co-migrés via le harness
//! partagé `crate::test_support`). Les routes restent enregistrées dans
//! `crate::http::build_router` et re-pointent ici en full-path ; paths,
//! shapes JSON et status codes inchangés. Invariants : duress noop_identity,
//! Remediation #8 chemin canonique deploy::publish_announcement, verrou 1,
//! floor anti-rollback revision, caps UTF-8-boundary.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use nexus_core_rs::BlobsClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::http::{
    DaemonHttpState, ErrorResponse, mint_blob_ticket, truncate_on_char_boundary,
    wrap_payload_with_pow,
};
```
`crate::deploy::{publish_announcement, AnnouncementParams}` + `crate::noop_identity::*` + `nexus_core_rs::*` + `nexus_shell_daemon_core::*` + `hex`/`serde_json` restent **fully-qualified** dans les corps (mirroir seed_api.rs). `Json` depuis `axum::response`. Pas de `SystemTime` (seul consommateur `list_nodes` ne bouge PAS), pas de `RwLock`/`std::path::Path` bare.

### 3.6 Bloc `use` prédit `mod tests` (co-migré)

```rust
#[cfg(test)]
mod tests {
    use super::*;                        // move-set + bloc use prod (StatusCode, Arc, BlobsClient, ErrorResponse)
    use crate::test_support::*;          // mk_state{,_with_sbfb_home,_with_mode,_with_mode_tx}, own_browse_entry, build_test_router, make_zip
    use crate::http::BrowseListResponse; // N2 : #[cfg(test)] pub struct http.rs:750-753 (browse, STAY) — consommé par 4 tests déplacés
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;
```
`KeyPair`/`create_node`/`SystemTime`/`RwLock`/`PowSolveCache` **PAS nécessaires** (aucun test publish déplacé ne les consomme bare — contraste curators_api). `tempfile`/`serde_json`/`hex`/`nexus_core_rs::*`/`crate::deploy::*`/`crate::runtime::GossipCmd`/`nexus_shell_daemon_core::publish::*`/`std::env`/`tokio::*` = fully-qualified dans les corps.

### 3.7 Tests co-migrés = **16 obligatoires** (+ **2 optionnels** recommandés)

> **Conflit résolu en faveur du critic tests-docs (A1)** : SCAN s1-deps-imports §6 mé-énumérait ses « 16 » (omettait `vps_authoring` 3488 + `daemon_boot_..._rejects_task_dispatch` 4635, et repliait les 2 optionnels dedans — contradiction interne, s1 §5 flaggait pourtant vps « mobile »). L'ensemble correct = INV-TESTS §2. `vps_authoring` est LE test le plus critique (seul à direct-call `reannounce_directory_at_boot`/`read_directory_revision`/`build_sign_announce_directory`/`DirectoryPublishOutcome::Published` en `super::`) → le laisser casserait la compile.

**16 OBLIGATOIRES** (drivent un symbole/route du move-set) :

| Cluster | Test | Ligne fn | Sujet move-set |
|---|---|---|---|
| directory (direct-call) | `publish_directory_route_signs_and_announces` | 2810 | `publish_directory` |
| | `publish_directory_noop_in_duress` | 2883 | `publish_directory` (bannière S20-B) |
| | `publish_directory_revision_is_monotone_across_publishes` | 2897 | `publish_directory` |
| | `publish_directory_revision_survives_logical_restart` | 2918 | `publish_directory` |
| | `publish_directory_revision_falls_back_to_sbfb_home_env` | 2946 | `publish_directory` |
| | `publish_directory_truncates_oversized_fields` | 2974 | `publish_directory` |
| | `publish_directory_excludes_spoofed_unheld_blob` | 3014 | `publish_directory` |
| | `publish_directory_concurrent_revisions_are_distinct` | 3063 | `publish_directory` (`multi_thread` worker_threads=4) |
| | `vps_authoring_signs_own_directory` | **3488** | `reannounce_directory_at_boot`/`read_directory_revision`/`build_sign_announce_directory`/`DirectoryPublishOutcome` (direct) |
| project (router) | `publish_returns_200_and_adds_direct_entry` | 3667 | `PublishRequest`/`PublishResponse` (+ `BrowseListResponse`) |
| | `publish_rejects_is_open_source_without_provenance_chain` | 3730 | `PublishRequest` (gate D-1) |
| | `publish_accepts_is_open_source_with_full_provenance_chain` | **3823** | `PublishRequest` (+ `BrowseListResponse`) |
| | `publish_with_archive_hash_populates_browse_entry` | 4373 | `PublishRequest` (+ `BrowseListResponse`) |
| | `daemon_boot_in_duress_mode_publishes_fake_curator_empty` | 4575 | POST /publish sous duress → `publish_project` (+ `BrowseListResponse`) |
| publish-blob (router) | `publish_blob_stores_and_returns_hash` | 4155 | `PublishBlobResponse` |
| | `daemon_boot_in_duress_mode_rejects_task_dispatch` | **4635** | `/publish-blob` 503 duress (nom trompeur ; doc :4631 « #B-rt-3 The /publish-blob handler in Duress mode returns 503 » — c'est un test publish-blob) |

**2 OPTIONNELS** (drivent `crate::deploy::publish_announcement` seul, **0 symbole move-set** → compilent des deux côtés ; co-migration = **jugement Phase-S**, PAS forcée compile) :
- `publish_announcement_persists_to_outbox_for_replay` (3087) — Remediation #8 real-frontier §P57
- `publish_and_gossip_use_per_app_project_id` (7056) — per-app project_id, jumeau de 3087

> Scans divergent : inv-prod/inv-tests/s1/s2 → co-migrer (cohésion thématique, gardes de régression du seam `publish_project`→outbox, anti-orphelins PO-10). s4 → STAY (« caractérisent deploy/outbox, pas le handler publish »). **Les deux sont compile-valides + count-invariants.** **Recommandation synthétiseur : co-migrer** (cohésion + PO-10) — arbitrage implémenteur/PO confirmable au moment du move.

**Fixtures : 0 promotion** — `build_test_router`:103, `mk_state`:114, `mk_state_with_sbfb_home`:118, `mk_state_with_mode`:124, `mk_state_with_mode_tx`:132, `own_browse_entry`:704, `make_zip`:739, `deploy_workspace_app`:755 TOUS déjà `pub(crate)` dans test_support.rs. Aucun helper local `mod tests` http.rs consommé par les tests mobiles.

**Re-import N2 : 1 seul** — `use crate::http::BrowseListResponse;` (`#[cfg(test)] pub struct` http.rs:750-753, browse-domaine STAY, consommé par tests 3667/3823/4373/4575 déplacés ET par tests browse restants 3662/4754 → PAS orphelin dans http.rs).

### 3.8 Tests qui RESTENT (frontière, NE PAS balayer)

- **Cluster SEARCH → S3** : `publish_makes_app_searchable_by_name` 6695, `published_app_searchable_by_*` 6810/6825/6846/6865, `published_app_browse_id_is_blake3_not_node_id` 6933 + helpers locaux `do_publish` 6698 / `publish_app` 6765 / `search_total` 6791 / `browse_entries` 6885 / `post_workspace` 7098. **Contrainte compile DURE** : ces helpers sont des `fn` non-`pub` DANS `mod tests` de http.rs → inaccessibles cross-module (E0425). Les co-migrer casserait la compile → partent en bloc avec search (S3). 0 référence de type move-set (inline `serde_json::json!`).
- **Frontière S2/seed** : `directory_resolvers_match_hash_and_project` 3162, `fetch_provider_ordering` 3254, `nodes_response_pins_envelope_and_grouping` 3305, `reachable_via_seeder_status` 3378, `browse_index_rejects_open_source_without_provenance` 2630 (→ `index_browse_entry`/`trustworthy_open_source` STAY).

### 3.9 `use` orphelins http.rs candidats après retrait

**Probablement 0** (contraste Phase R = 3). `axum::body::Bytes` encore consommé par `preview_load` :2472. `serde::{Deserialize,Serialize}`/`BlobsClient` : nombreux consommateurs restants. `mod tests` http.rs : `to_bytes`/`Method`/`Request`/`ServiceExt`/`KeyPair`/`create_node`/`BrowseListResponse` restent consommés par search/browse/blob-serve/curator. **À LISTER À LA COMPILE, ne pas présumer** (leçon R).

---

## 4. Arbitrages (chaque Q avec justification précédent + PO-10)

| Q | Décision | Justification |
|---|---|---|
| **`publish_blob` IN vs OUT** (plan S4) | **IN** | Plan S4 balayait `publish_blob` MAIS S4 nomme DÉJÀ `publish_api.rs` comme destination. Précédents **Phase R `default_curators` IN** (`f7d42bc`) + **Phase O keep-online IN** : quand le plan route un symbole vers une phase ultérieure dont la destination EST déjà le même fichier → tirer IN maintenant (OUT = double-churn `build_router` route 371 + tests orphelins = viol PO-10). |
| **Boot re-announce `reannounce_directory_at_boot` IN** | **IN** | Côté producteur du directory-publish (S75-E `1486fc9`). Consommateur externe unique = runtime.rs:1516 → re-point (déjà `pub(crate)`, 0 bump). Répond au scout : le boot driver appelant EST runtime.rs:1516, PAS seed_api.rs (seed_api.rs:147 = commentaire). |
| **Revision helpers IN** (`read_directory_revision`, `next_directory_revision`, `DirectoryRevisionFile`) | **IN** | Anti-rollback du directory-publish. `next_directory_revision` seul caller = `build_sign_announce_directory` (item 8, MOVE), reste privé. `read_directory_revision` caller = item 9 (MOVE). Cohésion domaine. |
| **`index_browse_entry` + `trustworthy_open_source` STAY/MOVE** | **STAY** | Précédent **Phase R `ErrorResponse` STAY** : symbole partagé multi-domaine reste au hub. Chokepoint browse-index consommé par deploy.rs:740 + runtime.rs:2311/2345 + tests, **0 caller move-set** (`publish_project` l'atteint transitivement via `deploy::publish_announcement`). MOVE forgerait `deploy→publish_api` + `runtime→publish_api` pour 0 relief. Déjà `pub(crate)` → 0 bump si STAY. |
| **`truncate_on_char_boundary` STAY vs MOVE** | **STAY + bump** | Util string générique dual-domaine (publish + search @2412). Coût 1 bump des 2 côtés ; STAY = arête neutre (évite forward-edge `search→publish_api` + 2ᵉ move S3 + casse `rustdoc` lien `MAX_SEARCH_QUERY_BYTES`). Mirror `ErrorResponse` STAY. |
| **2 tests optionnels 3087/7056 co-migrer** | **Co-migrer (recommandé)** | Cohésion thématique + anti-orphelins PO-10 ; 0 dépendance compile (FQ paths), count-invariant. Arbitrage confirmable. |
| **seed_api.rs changes** | **0 change** | Imports :22-24 = uniquement symboles STAY (mint_blob_ticket, find_directory_app_by_project, directory_pull_providers, DIRECTORY_PULL_TIMEOUT_SECS, DaemonHttpState). |

**Ne PAS déborder (frontière S2, réservé O/S2/S4)** : pull-resolution 1519-1629 (`PULL_PROVIDER_CAP` 1528, `DIRECTORY_PULL_TIMEOUT_SECS` 1534, `find_directory_app_by_hash` 1540, `find_directory_app_by_project` 1565, `directory_pull_providers` 1601 — importés par seed_api.rs:24) + nodes 1631-1728 (`NodesResponse`, `ObservedNodeView`, `NodeSummary`, `nodes_response`, `list_nodes`) + index-chokepoint 1730-1806.

---

## 5. Re-points docs EXHAUSTIFS (fichier:ligne)

> **Conflit résolu en faveur des critics completeness (ADJUST-1) + tests-docs (A2)** : SCAN s3-threat déclarait PATTERNS.md:1481 « drift-proof, pas de re-point ». **RÉFUTÉ** : ligne 1480 porte le full-path `crates/nexus-shell-daemon/src/http.rs :: publish_project`. Re-grep synthétiseur confirme.

### Ledger CODE-SÉCURITÉ + PROSE = **3 edits** (2 clean + 1 REWRITE)

| # | Fichier:ligne | Contenu actuel | Action | Type |
|---|---|---|---|---|
| 1 | **THREAT_MODEL.md:1039** | ``build_sign_announce_directory` dans `http.rs`` | swap `http.rs` → `publish_api.rs` (ex-http.rs S82 Phase S) ; **NE PAS toucher** `own_entries` même phrase (reste `browse.rs`) | clean (sécurité) |
| 2 | **PATTERNS.md:3435** | ``build_sign_announce_directory` in daemon `http.rs`` (mirror de 1039) | swap `http.rs` → `publish_api.rs` ; `own_entries` reste `browse.rs` | clean |
| 3 | **PATTERNS.md:1480-1482** | ``…/http.rs :: publish_project` now calls `wrap_payload_with_pow(&state, &payload)` (same file)` | **REWRITE, PAS token-swap** | rewrite (P3 doc-comptabilité) |

**Détail #3 (triplement stale, re-grep confirmé)** :
1. token `http.rs` → `publish_api.rs` (publish_project MOVE) ;
2. `(same file)` devient **FAUX** (`wrap_payload_with_pow` STAY http.rs, `build_sign_announce_directory` MOVE) ;
3. « publish_project **now calls** `wrap_payload_with_pow` » est **DÉJÀ FAUX sur le disque actuel** — `publish_project` (1077-1145) route UNIQUEMENT via `crate::deploy::publish_announcement` (:1128), **0 appel** `wrap_payload_with_pow`. Le vrai caller in-http.rs est `build_sign_announce_directory` (:1362, MOVE).
→ **Réancrer l'exemple sur `build_sign_announce_directory`** (qui appelle bien le helper) dans `publish_api.rs`, et corriger « (same file) » puisque le helper reste http.rs. Profondeur (correction honnête vs simple swap) = jugement P3 implémenteur/PO.

### Doc-honnêteté additionnelle (ADD-1, manqué par les 6 scans)

- **`test_support.rs:573-576`** — doc-comment de `golden_http_publish_domain` (observateur externe, STAY) : « `publish_blob` … sits ~2000 lines away … scattered domain ». Post-S les 3 handlers sont co-localisés (~450 l) → **rationale matériellement fausse**. Le test reste valide (route-driven, byte-identique) mais la justification est stale — même classe P3 doc-honnêteté. Vit à côté du golden-net (témoin de stabilité Phase S) donc **sera lu**. **Corriger au move.**

### AUCUN re-point (vérifié — exclusions correctes)

- `LOOPBACK_ENDPOINTS_TRUST_TIERS.md:77/86` : ancres par PATH (routes inchangées).
- `PATTERNS.md:4155` : `http.rs:directory_pull_providers` → STAY http.rs. Exact.
- `PATTERNS.md:699-702`/`3366-3372`/`1481` (revision) + `shell/PATTERNS.md:1159` (route-ownership `POST /publish`) : ancrés `directory_revision.json`/`anchors.json`/path → drift-proof.
- `SPRINT_LOG.md:56` : `DaemonHandle` test-harness historique S57, jamais re-pointé (convention).
- `directory_revision.json` (LIVE_FLIP_RUNBOOK.md:109/137, STORE_MIGRATION_OPS.md:35/38) : filename on-disk, inchangé. `DirectoryRevisionFile`/`truncate_on_char_boundary` n'apparaissent dans AUCUN doc.
- Gate-scripts + web/src (`daemon.ts`) : 0 match symbole/route publish → `frontier_closure` N/A.

---

## 6. Invariants VERBATIM à préserver au move

| Invariant | Site (disque actuel) | SHA |
|---|---|---|
| **Duress noop early-return** (1er statement, AVANT gossip sender) | `publish_project`:1089-1092 (early-return `PublishResponse{published:false}`) ; `build_sign_announce_directory`:1233 (`DirectoryPublishOutcome::DuressNoop`) ; `publish_blob`:1820 (`503 maintenance`) | S20-B `7ff22a0` |
| **Remediation #8** — chemin canonique unique `crate::deploy::publish_announcement` | `publish_project`:1124-1142 (déjà full-path :1128 → survit inchangé) | `3b7ef54` |
| **Gate S16 audit D-1** — refus `is_open_source=true` sans `provenance_hash` ET `repo_url` | `publish_project`:1095-1114 | S16 D-1 |
| **verrou 1 host-only** (`publish_directory` = projection read-side, jamais sélecteur write-side) + verrou 4 (own-apps + blob-held local) + lock-3 (no peer node id) | `publish_directory` doc :1167-1172 ; `build_sign_announce_directory` :1223-1227 | S75-B/E |
| **Floor anti-rollback / revision monotone** (`0`=jamais publié = clé gate state-driven ; `static REVISION_LOCK` process-wide strictly-increasing ; fallback `sbfb_home` ; persist best-effort) | `DirectoryRevisionFile`:1433, `read_directory_revision`:1459, `next_directory_revision`:1494 + REVISION_LOCK 1494-1516 | `f6637d3`+`1486fc9` |
| **Cap UTF-8 boundary** (clamp catalog fields avant signature) | appels `truncate_on_char_boundary` dans `build_sign_announce_directory` :1297-1314 | S75-B |
| **Directory announce LIVE-only, jamais persisté outbox** (durabilité producteur = boot re-announce seul) | `build_sign_announce_directory` :1340-1355 | S74-D `4c1acc5` |

Bannières de provenance (doc-comments) co-migrent verbatim avec leurs owners : `publish_project` 1067-1076, `publish_directory` 1160-1172, `build_sign_announce_directory` 1218-1227 + 1340-1355, `reannounce_directory_at_boot` 1386-1402 (doc dit **Sprint 75 Phase E** — scout mislabel « Phase F », non-matériel), revision 1428-1493. Le `//!` route-inventory (lignes 16-18, `POST /publish`/`publish-blob`/`directory/publish`) **RESTE http.rs** (décrit le router qui reste).

---

## 7. Adaptations vs plan (justifie PLAN-ADAPT)

1. **Bornes plan `http.rs:1159-1727` PRE-N doublement stales → DISCARD** ; move-set re-dérivé par NOM (tranches réelles §3.1 : 757-801 / 1067-1145 / 1147-1517 [îlot-STAY 1438-1452 sauté] / 1808-1848).
2. **`publish_blob` IN** (plan S4 le balayait ; S4 nomme déjà `publish_api.rs` → PO-10 anti-double-churn, précédents R/O).
3. **Boot re-announce + revision helpers IN** (cohésion producteur directory-publish).
4. **2 bumps SHARED-SYMBOL** (`ErrorResponse.error` champ NOUVEAU vs R + `truncate_on_char_boundary`) — extension stricte de la leçon Phase R (les handlers publish CONSTRUISENT l'erreur, pas seulement la reçoivent).
5. **3 re-points docs** dont 1 REWRITE (PATTERNS.md:1480-1482 déjà stale) + 1 doc-honnêteté test_support.rs:573-576 (ADD-1).
6. **s1 §6 sous-comptait les tests** (16 corrects = INV-TESTS §2, pas s1 §6). **s3 mé-classait PATTERNS.md:1481** (drift-proof réfuté).

Aucune décision Day-0/PO contredite (0 wire bump, 0 dep, iroh pinné inchangé, Factory hors daemon inaffecté, routes byte-identiques, T0 `authed_routes`/`build_router` protégés WIRING_SPEC:144/165). → **PAS DESIGN-CONFLICT.**

---

## 8. Verdict final : **PLAN-ADAPT**

Move pur discipliné (pattern N..R 6ᵉ application). Attentes net-invariant : **0 wire bump, 0 dep, nextest count EXACT Win 2108 / Docker 2112, golden net 9/9** (observateur externe `golden_http_publish_domain` byte-identique — routes re-pointées full-path). http.rs rétrécit ≈511 l prod + bloc test co-migré ; îlot `truncate_on_char_boundary` (15 l) + index-chokepoint (~77 l) + pull-resolution + nodes restent.

### Checklist compile-hazard (AVANT 1er build)
1. **`ErrorResponse.error` privé → `pub(crate)`** (4 sites construction déplacés 1106/1198/1825/1841) — sinon E0451.
2. **`truncate_on_char_boundary` → `pub(crate)`** (STAY http.rs, référencé cross-module).
3. `publish_project`/`publish_directory`/`publish_blob` privés → `pub(crate)` (routes re-pointent full-path).
4. Re-point runtime.rs:1516 + routes 370-372 + `mod publish_api;` main.rs:54.
5. `use crate::http::BrowseListResponse;` dans `mod tests` déplacé (N2, unique).
6. Cluster search 6695-6933 + helpers locaux **RESTENT** (E0425 dur).
7. `use` orphelins http.rs : **lister À LA COMPILE** (candidat réaliste : aucun).

### Pièges standing (rappel)
Docker sbfb-ci mount `/workspace` OBLIGATOIRE + `bash -c` (JAMAIS `bash -lc`) ; `set -o pipefail` ; gros cargo → `run_in_background` ; codex `--sandbox read-only` (elevated cassé « missing field sandboxPolicy ») ; `SBFB_TEST_HTTP_TIMEOUT_SECS=120` sous Docker-on-Windows ; préflight Workflow d'abord (attendre PLAN-ADAPT).
