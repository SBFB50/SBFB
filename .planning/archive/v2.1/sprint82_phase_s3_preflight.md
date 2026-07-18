# Sprint 82 — Phase S3 — Préflight G8 (synthèse)

- **Phase** : S3 — split domaines `feed/provenance` + `search` + `preview/proof-card` → `crates/nexus-shell-daemon/src/{feed_api.rs, search_api.rs, preview_api.rs}`
- **Date** : 2026-07-17
- **HEAD** : `b9b892a` — `http.rs` = 5635 l (lecture seule)
- **Cas** : B (pre-code)
- **Verdict** : **EXECUTE** (3ᵉ EXECUTE de la série après P puis S2 — le plan §S3 nomme le move-set par NOM et il est EXACT sur disque, 0 borne stale ; la seule question déférée [module de `preview_load`+`get_proof_card`] est TRANCHÉE sur le défaut du plan `preview_api.rs` ; les 3 mécaniques de co-migration [promotions test_support, dé-link F2, retrait orphelin `Bytes`] sont le pattern DISCIPLINE standing N2/O/Q/S2, PAS des corrections de plan ; 0 bump SHARED, 0 décision Day-0/PO touchée)

Préflight = dossier 6 scans factuels [inventaire-prod, tests-cluster, consommateurs-imports, decisions-historiques, docs-threat-model, wire-routes-gates] + 6 critics adversariaux. Les critics PRIMENT sur les scans. Le synthétiseur a ré-ancré lui-même sur disque HEAD `b9b892a` les faits load-bearing (move-set 1411-1853, F2:891, routes 490-521, main.rs 31-72, daemon.ts:617, les 3 helpers partagés + leurs call-sites, la liste des tests de la région 4440-5640).

---

## 1. Contexte

Phase S3 = 8ᵉ split de la série N..S3 (pattern PROUVÉ 7×). Le plan (`sprint82_plan.md` §S3, l.431-438, amendement PO-10 « S82 = une fin ») nomme 8 symboles racine + destinations. Contrairement à O/R/S le plan cite des numéros de ligne (1415/1484/1535/1539/1623/1637/1715/1738) — **exacts sur disque ce coup-ci**, mais tous re-dérivés PAR NOM (discipline D3/D4 : jamais faire confiance à un numéro de plan).

Spécificité S3 vs S2 : le move-set PRODUCTION est un **slab CONTIGU** (1411–1853, 443 l, 0 fn/type étrangère intercalée) coupé en **3 sous-tranches** par module cible ; mais les **TESTS sont NON-contigus** — la population search vient de DEUX régions séparées (haut : 4769-5004, bas : 5328-5474), avec des tests STAY (browse/deploy/fork) intercalés au milieu, et **3 helpers de test partagés** straddling moving↔staying qui FORCENT une promotion vers `test_support`. C'est le crux structurel de S3. **Extraction par NOM, jamais par plage, ni pour le prod ni pour les tests.**

Frontière AMONT : la fn qui précède (`canary_freshness`/l'`impl` qui ferme à :1409) STAY. Frontière AVAL : bannière `// Tests` :1855-1857 + `#[cfg(test)] mod tests` :1859 STAY (routeur core http.rs).

---

## 2. Décision module `preview_load` + `get_proof_card` : **preview_api.rs (les DEUX ensemble)** — TRANCHÉ

Défaut du plan (§S3 l.435 « preview_api.rs par défaut ») **RATIFIÉ pour les deux handlers**. Motivation (unanime inventaire-prod + tests-cluster + decisions-historiques) :

1. **Adjacence + lignée Sprint 68** : `preview_load` (S68 Phase B) et `get_proof_card` (S68 Phase A) sont physiquement contigus (:1706-1853), petits (18 l + 116 l), 0 symbole étranger entre eux — même pattern que S2 (browse+nodes, 2 petits domaines adjacents fusionnés).
2. **Aucun foyer existant convaincant** : `get_proof_card` est un AGRÉGATEUR cross-domaine (browse_aggregator + curator_runtime + provenance DB + proof_card compute). Le seul chevauchement de source de données est `browse_api.rs`, MAIS son doc-header (S2, browse_api.rs:2-23) borne EXPLICITEMENT sa charte à browse+nodes et renvoie le cross-domaine à http.rs — y coller proof-card élargirait sa charte écrite. `preview_load` est totalement isolé (`state.preview_store` seul).
3. **0 couplage interne** entre les deux (pas de DTO/helper/champ d'état partagé) — cohabitation « deux singletons résiduels S68 dans un fichier », pattern assumé.

Alternative `get_proof_card → browse_api.rs` **REJETÉE** (élargirait une charte déclinée par écrit + scinderait la paire S68).

---

## 3. MOVE-SET FINAL PRODUCTION (dérivé par NOM, lignes disque HEAD b9b892a)

Slab contigu 1411–1853, coupé en 3 tranches. Bannières de section appartiennent au symbole qui SUIT (co-migrent).

### 3.1 → `feed_api.rs` (tranche 1411–1608)

| # | Symbole | Kind | Lignes | Vis. actuelle → cible | Bump |
|---|---|---|---|---|---|
| — | Bannière « Sprint 63 Phase B — Provenance endpoint » | — | 1411–1413 | co-migre | — |
| 1 | `get_provenance` | async fn handler | 1415–1478 | privée → **pub(crate)** | ROUTINE |
| — | Bannière « Sprint 63 Phase C — Feed cursor endpoint » | — | 1480–1482 | co-migre | — |
| 2 | `get_feed_cursor` | async fn handler | 1484–1521 | privée → **pub(crate)** | ROUTINE |
| 3 | `FeedEntriesQuery` | struct DTO (`#[derive(Debug, serde::Deserialize)]`) | 1523–1533 | privée → **pub(crate)** [DÉVIATION compile, §4.5] | ROUTINE |
| 4 | `default_feed_limit` | fn helper (serde `default=`) | 1535–1537 | privée → **reste privée** | — |
| 5 | `get_feed_entries` | async fn handler | 1539–1608 | privée → **pub(crate)** | ROUTINE |

### 3.2 → `search_api.rs` (tranche 1610–1704)

| # | Symbole | Kind | Lignes | Vis. actuelle → cible | Bump |
|---|---|---|---|---|---|
| — | Bannière « Sprint 67 Phase B: FTS5 search endpoint » | — | 1610–1612 | co-migre | — |
| 6 | `SearchQuery` | struct DTO | 1614–1621 | privée → **pub(crate)** [DÉVIATION compile, §4.5] | ROUTINE |
| 7 | `default_search_limit` | fn helper | 1623–1625 | privée → **reste privée** | — |
| 8 | doc CARRY-5 + `MAX_SEARCH_OFFSET` + `MAX_SEARCH_QUERY_BYTES` | consts | 1627–1635 (doc 1627–1633) | privées → **restent privées** | — |
| 9 | `search_handler` | async fn handler | 1637–1704 | privée → **pub(crate)** | ROUTINE |

`search_handler` appelle `truncate_on_char_boundary` (:1655) qui **RESTE** dans http.rs (pub(crate) depuis S) → import entrant (§5).

### 3.3 → `preview_api.rs` (tranche 1706–1853)

| # | Symbole | Kind | Lignes | Vis. actuelle → cible | Bump |
|---|---|---|---|---|---|
| — | Bannière « Sprint 68 Phase B — Ephemeral preview load endpoint » | — | 1706–1708 | co-migre | — |
| 10 | `PreviewLoadResponse` | struct DTO (`pub`) | 1710–1713 | **`pub`** → **reste `pub`** (verbatim) | — |
| 11 | `preview_load` | async fn handler | 1715–1732 | privée → **pub(crate)** | ROUTINE |
| — | Bannière « Sprint 68 Phase A — ProofCard evidence score endpoint » | — | 1734–1736 | co-migre | — |
| 12 | `get_proof_card` | async fn handler | 1738–1853 | privée → **pub(crate)** | ROUTINE |

**≈ 443 l prod** (feed 198 + search 95 + preview 148).

### 3.4 Symboles PARTAGÉS restant dans http.rs (STAY) — 0 bump

| Symbole | Ligne | Vis. | Consommateur move-set | Action |
|---|---|---|---|---|
| `DaemonHttpState` | 75 | `pub struct` | les 6 handlers (State extractor) | import `use crate::http::DaemonHttpState;` (3 modules) |
| `truncate_on_char_boundary` | 892 | `pub(crate)` (S) | `search_handler` :1655 SEUL | import `use crate::http::{…, truncate_on_char_boundary};` (search_api) |
| `ErrorResponse` / `runtime_error_to_response` / `mint_blob_ticket` / `BrowseListResponse` / `spa_fallback` / cluster pull-resolution | — | — | **0 caller move-set** (vérifié : 1415-1853 émet ses erreurs via `Json(json!({"error":…}))` inline) | **NE PAS toucher** (cluster pull STAY→S4) |

### 3.5 Bump ledger CONSOLIDÉ = **8 bumps ROUTINE, 0 SHARED** [AMENDÉ in-phase, déviation compiler-forcée]

`get_provenance` / `get_feed_cursor` / `get_feed_entries` / `search_handler` / `preview_load` / `get_proof_card` privés → `pub(crate)` (routes les re-pointent full-path ; chacun référencé UNIQUEMENT par `build_router` + son propre test co-migrant → jamais besoin de `pub`). `PreviewLoadResponse` reste `pub` verbatim ; `default_feed_limit`/`default_search_limit`/`MAX_SEARCH_*`/`insert_test_feed_entry` restent privés (confirmé compile).

**DÉVIATION COMPILER-FORCÉE (§4.5)** : la prédiction « DTOs Query restent privés — le lint ne se déclenche pas » est **RÉFUTÉE par la compile** (4 erreurs `type FeedEntriesQuery/SearchQuery is private` aux call-sites `build_router` http.rs:495/:497 + 2 warnings `private_interfaces`). Cause exacte = classe Phase R « reachability du type au call-site » : le handler `pub(crate)` re-pointé cross-module expose `Query<T>` dans sa signature, atteignable depuis http.rs, alors que `T` reste privé au nouveau module — intra-`http.rs` (avant S3) le call-site et le type partageaient le même module, le cas ne se posait pas. Fix minimal : `FeedEntriesQuery` + `SearchQuery` → **pub(crate)** (champs PRIVÉS conservés, Deserialize-only, construction serde uniquement) → total **8 ROUTINE / 0 SHARED**. Même pattern que la déviation E0425 compiler-forcée Phase O (`ingest_remote_directory`), amendée au préflight avant commit.

---

## 4. Arbitrage scan-vs-critic (corrections matérielles)

Le dossier converge. Corrections critic re-vérifiées sur disque par le synthétiseur :

1. **decisions-historiques (ADJUSTED)** : `feed_insert_rejects_without_internal_header` est à **http.rs:2421** (pas 4420-4421 comme écrit dans le scan). SANS impact S3 : ce test exerce `crate::feed_sync::feed_insert` (write-side, PAS dans le move-set read `get_feed_*`) → STAY avec feed_sync. Décision inchangée.
2. **wire-routes-gates (ADJUSTED)** : le scan avait déclaré « Angle mort F2 = 0 risque » — **RÉFUTÉ par son critic ET re-confirmé par moi sur disque**. Le hazard F2:891 est RÉEL (§6). Les 5 autres scans + critics l'avaient déjà attrapé ; consensus = F2 réel, à traiter.
3. **inventaire-prod (CONFIRMED, 2 nits immatériels)** : (a) placement alpha de `mod search_api;` = entre `runtime`(58)/`seed_api`(59), PAS `result_sync`/`runtime` (corrigé §5) ; (b) use-list feed_api omet `hex` — mais `hex::` est full-path dans les corps (get_provenance:1433, get_proof_card:1750/1816) → 0 `use` requis (§5).
4. Nits off-by-one (ancre `#[tokio::test]` vs ligne `async fn`, labels de bannière paraphrasés) : immatériels, tout re-dérivé par NOM.
5. **[AMENDEMENT IN-PHASE 2026-07-17] Déviation compiler-forcée** : les DTOs `FeedEntriesQuery` (feed_api) et `SearchQuery` (search_api) prédits « restent privés » par les scans consommateurs-imports + inventaire-prod sont passés **pub(crate)** — 4 erreurs compile `private type` aux call-sites routes (détail §3.5). Champs privés conservés. 0 impact wire (Deserialize-only). Bump ledger 6 → **8 ROUTINE**, toujours 0 SHARED. Checklist item 9 (« STOP si bump SHARED ou edit d'un autre module ») NON déclenchée : c'est un bump de symboles DU move-set.

Aucune correction ne déplace un symbole, ne rend une borne stale, ni ne force un bump SHARED. **Le plan tient.**

---

## 5. Re-points CODE + blocs `use` prédits

### 5.1 Routes dans `build_router` (fn @246) — 6 re-points full-path, paths BYTE-IDENTIQUES

| Ligne | Actuel | Après |
|---|---|---|
| 490 | `get(get_feed_cursor)` | `get(crate::feed_api::get_feed_cursor)` |
| 491 | `get(get_feed_entries)` | `get(crate::feed_api::get_feed_entries)` |
| 492 | `get(search_handler)` | `get(crate::search_api::search_handler)` |
| 493 | `get(get_proof_card)` | `get(crate::preview_api::get_proof_card)` |
| 494 | `post(preview_load)` | `post(crate::preview_api::preview_load)` |
| **521** (route multi-ligne 519–522) | `get(get_provenance)` | `get(crate::feed_api::get_provenance)` |

⚠️ Route #6 (`/api/v1/project/{project_id}/provenance`) est **multi-ligne** (path :520, `get(get_provenance)` :521) — faux-négatif grep mono-ligne (classe Phase P). Repérée. **89 routes invariantes** (dernier `.route(` avant `mod tests` :1859).

### 5.2 `main.rs` — 3 `mod` additifs (ordre alpha, modules déclarés dans main.rs, pas de lib.rs)

- `mod feed_api;` entre `mod dispatch_loop;` (41) et `mod feed_sync;` (42)
- `mod preview_api;` entre `mod panic;` (54) et `mod publish_api;` (55)
- `mod search_api;` entre `mod runtime;` (58) et `mod seed_api;` (59) — **correction critic** (`search_api` < `seed_api`)

### 5.3 Orphelin `use` dans http.rs = **1** (`body::Bytes`)

`axum::body::Bytes` (http.rs:41, dans le bloc `use axum::{…}` :39-47) devient orphelin — `\bBytes\b` = **exactement 2 hits** (import :41 + `preview_load` :1715, vérifié). `preview_load` part → **retirer `body::Bytes,`** du bloc. SEUL orphelin. Non-orphelins confirmés (autres consommateurs restent) : `Serialize`/`Deserialize` (@56), `debug!`/`warn!` (@60), `Path`/`State`/`StatusCode`/`Json`/`IntoResponse` (pervasifs). **Lister À LA COMPILE (leçon R) — cette prédiction est grep-vérifiée mais confirmer sous `-D warnings`.**

### 5.4 Blocs `use` prédits (prod) — chaque import à re-vérifier À LA COMPILE

**feed_api.rs**
```rust
use std::sync::Arc;
use axum::extract::{Path, State};                 // Path: get_provenance ; State: les 3
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use crate::http::DaemonHttpState;
```
`hex`, `serde_json`, `tracing::error!`, `axum::extract::Query` (:1541), `nexus_coordinator_rs::*`, `serde::Deserialize` (derive full-path @1523) restent full-path → 0 `use`.

**search_api.rs**
```rust
use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use crate::http::{DaemonHttpState, truncate_on_char_boundary};
```
`axum::extract::Query` (:1639), `std::time::Instant`, `nexus_coordinator_rs::search::search`, `serde_json`, `tracing::error!` full-path.

**preview_api.rs**
```rust
use std::sync::Arc;
use axum::body::Bytes;                             // preview_load
use axum::extract::{Path, State};                  // Path: get_proof_card ; State: les 2
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};               // derive PreviewLoadResponse BARE @1710
use tracing::debug;                                // preview_load: debug! BARE @1716
use crate::http::DaemonHttpState;
```
`std::collections::HashSet` (:1748), `hex::encode`, `nexus_coordinator_rs::{provenance, proof_card}`, `nexus_shell_daemon_core::preview::PreviewError` full-path.

### 5.4bis Blocs `use` prédits (`mod tests` co-migrés) — hazard classe N2/R

Template canonique S2 (browse_api.rs:286-294) : `use super::*;` + `use axum::body::to_bytes;` + `use axum::http::{Method, Request};` + `use tower::ServiceExt;` + `use crate::test_support::*;`. N'importer QUE l'utilisé (clippy `-D unused`).

- **feed_api tests** : + `use nexus_core_rs::KeyPair;` (`KeyPair::generate()` BARE @4521/4559/4567, provenance cross-node). `create_node` NON observé (`mk_state` encapsule) — NE PAS importer, **confirmer à la compile**.
- **search_api tests** : + `use axum::Router;` (nested `do_publish`/`do_search` typent `router: Router` BARE @4820/4840).
- **preview_api tests** : + `use nexus_shell_daemon_core::browse::BrowseEntry;` (`test_proof_card_endpoint_http` construit `BrowseEntry{…}` BARE @5487 ; le `use …browse::{BrowseSource, BrowseStatus};` local @5482 reste DANS la fn verbatim).

---

## 6. HAZARD F2 (angle mort S2) — **1 lien intra-doc à traiter** (les DEUX directions balayées)

- **http.rs:891** — doc-comment `///` de `truncate_on_char_boundary` (qui **RESTE**, pub(crate) :892) contient le lien intra-doc `` [`MAX_SEARCH_QUERY_BYTES`] `` (CARRY-5). `MAX_SEARCH_QUERY_BYTES` (:1635) **PART** vers search_api.rs → lien cassé.
- **Différence vs miss S2 (F2:755)** : ici le symbole documenté est un item PRODUCTION `pub(crate)` (PAS `#[cfg(test)]`) → `cargo doc` LE RENDRAIT. **Pas de `#![deny(rustdoc::broken_intra_doc_links)]`** dans main.rs/http.rs (vérifié) → **warning**, pas fail dur — mais à traiter (hygiène + classe exacte du miss S2).
- **Résolution recommandée = dé-link** : `` [`MAX_SEARCH_QUERY_BYTES`] `` → `` `MAX_SEARCH_QUERY_BYTES` `` (backticks seuls, 0 bump). L'alternative full-path `` [`crate::search_api::MAX_SEARCH_QUERY_BYTES`] `` warnerait aussi (item privé d'un autre module) → **dé-link préféré**.
- **Balayage bidirectionnel exhaustif** : c'est le **SEUL** lien `[Symbole]` de tout http.rs pointant un symbole du move-set (inventaire complet des `///` à crochets : 69/71/73/94/… tous intra-STAY ; 891 = unique). Sens inverse propre : les corps du move-set n'ont qu'un `///` (doc CARRY-5 :1627-1633) qui référence `MAX_SEARCH_OFFSET`/`MAX_SEARCH_QUERY_BYTES` en backticks (0 crochet) et co-migre AVEC eux.

---

## 7. Tests co-migrés = **23** + **3 promotions test_support** + **2 STAY-helpers**

La région 4440-5640 (inventoriée fn par fn) confirme l'entrelacement : feed (bloc contigu 1-10) ; search (2 régions non-contiguës, 11-20 haut + 32-33 bas) séparées par des tests STAY (21-31 browse/deploy/fork) ; preview (bloc contigu 34-39). **Chaque test/helper migre par NOM.**

### 7.1 → `feed_api.rs` (9 tests + 1 helper local)
`provenance_endpoint_absent_status` (fn 4451), `provenance_endpoint_found_and_verified` (4473), `provenance_cross_node_verified` (4518), `provenance_cross_node_tampered` (4556), `provenance_endpoint_returns_app_version` (4595), `feed_cursor_empty_returns_zero` (4634), `feed_cursor_returns_saved_position` (4655), `test_feed_entries_endpoint_paginated` (4704), `test_feed_entries_endpoint_filters_by_project_id` (4734). **Helper `insert_test_feed_entry` (4681)** = feed-local (usages 4709-4742 = les 2 feed-entries tests SEULS) → co-migre PRIVÉ, 0 promotion.

### 7.2 → `search_api.rs` (8 tests)
`test_search_endpoint_http` (4769), `publish_makes_app_searchable_by_name` (4817, **+ nested `do_publish` 4820 / `do_search` 4840** co-migrent verbatim, 0 promotion), `published_app_searchable_by_category` (4932), `_single_letter` (4947), `_description_word` (4968), `_multi_word_query` (4987), `search_handler_json_includes_triplet` (5328), **`search_clamps_offset_and_query` (5389, MANDATORY** — code ACTIF `MAX_SEARCH_QUERY_BYTES` @5440 ; résout via `use super::*`, const reste privé, **impossible de rester en http.rs sans bump** → co-move obligatoire, visibilité minimale).

### 7.3 → `preview_api.rs` (6 tests)
`test_proof_card_endpoint_http` (5481), `test_proof_card_endpoint_not_found` (5528), `test_preview_load_returns_hash` (5552), `test_preview_blob_serve_accessible` (5577), `test_preview_eviction_after_ttl` (5605), `test_preview_max_size_rejected` (5616).

### 7.4 Promotions → `test_support.rs` en `pub(crate)` = **3** (crux ; pattern N2/O/Q)
Trois helpers straddling moving↔staying (ne peuvent co-migrer, duplication interdite) → hoister :

| Helper | Def http.rs | Consommateurs MOVING | Consommateurs STAY |
|---|---|---|---|
| `publish_app` | 4887–4910 | published_app_searchable_by_* (4935/4952/4971/4990) | multiple_apps_get_distinct_browse_cards (5031/5035), published_app_browse_id_is_blake3 (5058) |
| `search_total` | 4913–4929 | published_app_searchable_by_* (4939/4943/4956/4961/4975/4980/4995/5000) | multiple_apps (5050/5051), fork_redeploy_loop_e2e_single_node (5148) |
| `make_test_zip` | 3917–3928 | test_preview_load_returns_hash (5555), test_preview_blob_serve_accessible (5579) | deploy_private_valid_zip_returns_200 (3934) |

Transitivement PROPRE (vérifié corps) : `publish_app`/`search_total` n'appellent que `build_test_router`(:103)/`mk_state`(:114) déjà pub(crate) + items déjà importés dans test_support (`Method`/`Request`/`to_bytes`/`StatusCode`/`ServiceExt`/`Arc`, `serde_json`/`axum::body::Body` full-path) ; `make_test_zip` = `zip::` + std (test_support utilise déjà `zip` pour `make_zip`). **Count-neutral** (pas `#[test]`). `make_test_zip` **PAS interchangeable** avec `test_support::make_zip` (:740) : make_zip = `CompressionMethod::Stored` + liste de fichiers ; make_test_zip = Deflate + fichier unique fixe → **promote verbatim, NE PAS réécrire**. 0 collision de nom dans test_support (seule mention = prose backtick `publish_app` @755, qui devient exacte après promotion — `deploy_workspace_app` « mirrors `publish_app` » sont alors co-résidents).

### 7.5 STAY-helpers (NE PAS balayer)
`browse_entries` (5007) + `post_workspace` (5179) : consommés UNIQUEMENT par des tests STAY → restent http.rs, 0 move.

### 7.6 Tests « nom dit domaine mais STAY » (anti faux-orphelin)
- `fork_redeploy_resigns_provenance_as_local_node` (5082) : nom « provenance » mais exerce `/deploy-workspace` + lit `db.get_provenance_by_project` DIRECT (5097) — **n'appelle PAS `get_provenance`**. Domaine deploy. STAY.
- `browse_index_rejects_open_source_without_provenance` (1874, 1er test du module) : exerce `index_browse_entry` (:1038 STAY) + `search::search` direct — browse-index-integrity, PAS un handler du move-set. STAY.
- `finalize_deploy_open_source_arm_propagates_version_and_flag` (5281) : `db.get_provenance_by_project` direct, domaine deploy. STAY.
- `feed_insert_rejects_without_internal_header` (**2421**, correction critic) : `crate::feed_sync::feed_insert` write-side. STAY.

---

## 8. Re-points DOCS = **1 genuine** + preuves négatives

| # | Fichier:ligne | Contenu disque | Action | Type |
|---|---|---|---|---|
| 1 | **web/src/api/daemon.ts:617** | « The Rust `search_handler` (`nexus-shell-daemon/src/http.rs`) serialises… » | swap chemin → `nexus-shell-daemon/src/search_api.rs` | clean, INCONDITIONNEL (précédent R : daemon.ts:102 → curators_api.rs) |
| 2 | **http.rs:891** | lien F2 (§6) | dé-link | F2 |

**AUCUN autre re-point (preuves négatives vérifiées)** :
- **daemon.ts:646** « Mirrors the `search_handler` envelope » = nom de SYMBOLE seul, sans chemin → reste EXACT, **NO-ACTION** (signaler aux reviewers pour éviter un faux-positif).
- **THREAT_MODEL.md** : §13 Preview (728-767) nomme uniquement `nexus_shell_daemon_core::preview::*` (PreviewStore/MAX_PREVIEW_BYTES/PreviewError), JAMAIS le handler `preview_load` ni une ligne http.rs. 0 des 8 symboles. CLEAN.
- **LOOPBACK_ENDPOINTS_TRUST_TIERS.md** : `/api/daemon/feed/insert` (l.78, hors move-set) + `/api/daemon/search` (l.88) cités par STRING de route → routes byte-identiques → CLEAN.
- **PATTERNS.md (rust+shell)** : 0 match des 8 handlers/routes. CLEAN.
- **docs/factory/** : seul hit `http.rs` = FACTORY_GATES.md:190 (mention générique blob-serve CSP self-check). CLEAN.
- **3 gates** (`check-frontier-contracts.sh` / `check-factory-docs.sh` / `check-sharding-docs.sh`) : anti-promise scan traverse crates/ mais matche PROMISE_RE (le move verbatim préserve le texte → verdict-invariant) ; `« in S74 »` de search_handler:1682-1685 ne matche PAS PROMISE_RE. `frontier-tag`/anchors http.rs = aucun des 8. CLEAN.
- Coverage addenda critic (non-re-points, tous CLEAN vérifiés) : POST_CHATONS.md:479-482 (route-strings), TOOLING.md:291 (ancre `http.rs:483-494`, sous 494 → non décalée par l'extraction 1415-1853).
- **Goldens test_support** : les 9 `golden_http_*` observent /health+blob-serve/shard/seed/frost/coordinator/curators/publish/CORS/SPA — **AUCUN ne couvre /feed /search /provenance /preview /proof-card** → filet reste vert SANS edit. Le vrai filet = tests domaine co-migrés + count nextest invariant. Gap golden pré-existant (cohérent P2 Codex S2 « 0 golden browse/nodes »), à consigner au commit body.
- **HORS-PÉRIMÈTRE (jamais re-pointé)** : SPRINT_LOG.md (narration figée), archives .planning/v2.1/sprint66-68 (records figés à d'anciens HEAD), sprint82_plan.md + artefacts phase (re-dérivés par NOM).

---

## 9. Invariants VERBATIM à préserver au move

| Invariant | Site disque | Origine |
|---|---|---|
| **0 wire bump** — search_index LOCAL, colonnes M17 UNINDEXED/never-matchable, `FEED_FORMAT_VERSION` reste 1 | doc `search_handler` :1682-1690 (verbatim, dans le corps) | S73-D `0f86e5a` |
| **CARRY-5 clamps** — `MAX_SEARCH_OFFSET=10_000` + `MAX_SEARCH_QUERY_BYTES=1024` + troncature UTF-8-safe (offset non-borné = DoS ; `usize::MAX as i64` flip négatif = fuite de lignes) | doc consts :1627-1633 + code :1652-1655 + test `search_clamps_offset_and_query` :5389 | S75-G `8b53c38` |
| **serde(default) = runtime-tolerance** (FeedEntriesQuery/SearchQuery), PAS des zombies legacy — pre-launch policy | 1523-1533 / 1614-1621 | pre-launch |
| **preview size ceiling** via `PreviewError::TooLarge` → 413 | preview_load :1719 | S68-B |
| Bannières de lignée S63/S67/S68 co-migrent verbatim avec leurs handlers | 1411/1480/1610/1706/1734 | — |

**Absence explicitement vérifiée** : 0 duress / 0 guardrail / 0 internal-header / 0 consent dans 1411-1853 (grep). Les 8 handlers sont read/store purs. Le guardrail-before-persist D5 (S73-A) vit sur le WRITE `result_text` (validator_loop), PAS ici — rien à préserver de ce côté.

---

## 10. Synthèses S1a/S1b/S2/S3/S4 (une ligne)

- **S1a SOTA** : N/A (refacto pur). Pattern extraction-par-NOM outillé standing.
- **S1b deps** : **0 dep nouvelle** (axum/serde/tracing/hex/zip/core-crates/tower déjà consommés).
- **S2 histoire** : domaines nés S63 (provenance/feed) + S67-B FTS5 + S68 (preview/proof-card) ; défense injection FTS5 vit dans `nexus-coordinator-rs::search` (hors move-set) ; SearchManifest = DEFER standing (scope-cut search, feed-local-replicate suffit en pilote fermé). Aucune décision « rejected/deviation » move-set-spécifique au-delà.
- **S3 threat** : surface INCHANGÉE (0 route, 0 tier, 0 gate). 1 re-point daemon.ts:617. LOOPBACK/THREAT_MODEL ancrés par PATH/crate-core.
- **S4 wire+frontier** : **0 wire bump structurellement** (loopback JSON pur ; réponses = littéraux `json!` inline ordre préservé + 1 DTO mono-champ `pub PreviewLoadResponse` + 2 DTO Deserialize-only ordre non-wire ; 0 `*_VERSION`/`DOMAIN_`/canonical dans 1411-1853). 6 routes re-pointées full-path paths byte-identiques. `frontier_closure` N/A (couplage front par chemin+shape). 0 tag FRONTIER.

---

## 11. Comptabilité attendue

- **http.rs** : 5635 → **≈ 4300–4350 l** (retrait ≈443 l prod + ≈810 l tests + ≈54 l helpers promus − 1 l orphelin `Bytes` ; + re-points full-path négligeables). Estimation — les tests étant scattered, chiffre exact À L'IMPLÉMENTATION.
- **feed_api.rs** : ≈ **520–550 l** (prod 198 + tests ≈314 + header/use ≈20).
- **search_api.rs** : ≈ **440–470 l** (prod 95 + tests ≈340 [2 régions] + header/use ≈20).
- **preview_api.rs** : ≈ **310–340 l** (prod 148 + tests ≈155 + header/use ≈20).
- **test_support.rs** : **+ ≈54 l** (3 helpers promus).
- **Net-invariants** : 0 wire bump, 0 dep, **nextest count EXACT Win 2108 / Docker 2112** (promotions count-neutral, tous les `#[test]` co-migrent dans le même crate), goldens 9/9 sans edit, Vitest web 412 / operator 201 inchangés, 89 routes.

---

## 12. Verdict final : **EXECUTE**

Move pur discipliné. Le plan §S3 nomme le move-set par NOM et il est EXACT sur disque (0 borne stale) ; la question déférée (module preview) tranche sur le défaut du plan ; les 3 mécaniques (promotions test_support, dé-link F2, retrait `Bytes`) sont le pattern DISCIPLINE standing (N2/O/Q/S2), pas des corrections de plan ; 6 bumps ROUTINE / 0 SHARED ; 0 décision Day-0/PO touchée.

### Checklist compile-hazard (AVANT 1ᵉʳ build)
1. 6 bumps ROUTINE : les 6 handlers → `pub(crate)`.
2. 6 routes → full-path (dont #6 multi-ligne :519-522) ; 3 `mod` main.rs (positions alpha §5.2, `search_api` entre 58/59).
3. Retirer `body::Bytes,` du bloc `use axum::{…}` http.rs:41 (SEUL orphelin — confirmer sous `-D warnings`).
4. Dé-link F2 http.rs:891.
5. Promouvoir `publish_app`/`search_total`/`make_test_zip` → `test_support.rs` pub(crate) (verbatim, PAS de réécriture).
6. `mod tests` co-migrés : `use crate::http::truncate_on_char_boundary` (search prod) ; imports N2/R : `KeyPair` (feed), `axum::Router` (search), `nexus_shell_daemon_core::browse::BrowseEntry` (preview) — n'importer que l'utilisé.
7. STAY : `browse_entries`/`post_workspace`/cluster pull-resolution/`fork_redeploy_resigns`/`browse_index_rejects`/`feed_insert_rejects`(2421) — NE PAS balayer.
8. daemon.ts:617 → `search_api.rs` (édit commentaire seul) ; daemon.ts:646 NO-ACTION.
9. Si la compile réclame un edit `seed_api.rs`/`publish_api.rs`/`browse_api.rs` ou un bump SHARED : STOP, l'arbitrage a été mal appliqué — re-vérifier.

### Gates D4 à rejouer (§7.4)
`cargo fmt --all --check` ; `cargo clippy --workspace --all-targets --locked -D warnings` ; `cargo nextest run --workspace --locked` (== 2108 Win) ; `cargo test --workspace --locked --doc` ; `cargo build -p nexus-shell-daemon --release` ; pipeline web (daemon.ts édité — lint/tsc/test:unit/build/size + scan-en-strings) ; Docker sbfb-ci dual-platform (== 2112) ; 3 gates docs-contrat ; review Workflow + Codex Sol.

### Pièges standing (rappel)
Docker sbfb-ci mount `/workspace` + `bash -c` (JAMAIS `bash -lc`) + `MSYS_NO_PATHCONV=1` chemin hôte explicite ; `set -o pipefail` ; gros cargo → `run_in_background` ; codex `--sandbox read-only` ; `SBFB_TEST_HTTP_TIMEOUT_SECS=120` sous Docker-on-Windows ; preuve token-level NON-CIRCULAIRE ; chaîne `&&` web avale un post-FAIL ; flake sigint sous charge → re-run solo avant de conclure.
