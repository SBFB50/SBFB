# Sprint 82 Phase R — Preflight G8 (split domaine curators → `curators_api.rs`, discipline étendue PO-10)

## En-tête

- **Sprint** : 82 (dette docs-contrat + refacto, 24 phases, C9).
- **Phase** : R — split du domaine `curators` de `http.rs` vers un NOUVEAU module
  `crates/nexus-shell-daemon/src/curators_api.rs`.
- **Date** : 2026-07-16.
- **HEAD au preflight** : `7faa632` (Phase Q `coordinator_api.rs` DONE). Phases A→Q DONE (18/24).
  `http.rs` = **8111 lignes** sur disque.
- **Verdict** : **PLAN-ADAPT** (justifié §13 — approche du plan confirmée, split pur verbatim
  vers `curators_api.rs` ; mais 6 adaptations matérielles requises, dont deux compiler-forcées
  et une contredisant le séquencement du plan : (1) **bump `pub(crate)` de la DTO PARTAGÉE
  `ErrorResponse` + du helper `runtime_error_to_response`** — 1ᵉʳ bump de visibilité d'un symbole
  `http.rs` PARTAGÉ de la série de splits S82, et la « 1-bump / ErrorResponse untouched » du
  bundle est REFUTÉE par le critic compile [compile-testé] : le chemin recommandé coûte **2** bumps,
  pas 1 ; (2) **arbitrage Q1 `default_curators` = IN** alors que le plan le route en Phase S4 ;
  (3) correction du symbole du plan `SubscribeCuratorResponse` **inexistant** → `SubscriptionsResponse` ;
  (4) bornes re-dérivées par NOM [plan `:884-1102` doublement stale, 5 tranches non-contiguës] ;
  (5) 3 re-points docs sur une frontière **FORM-couplée** [Zod `.strict()`, pas path-only comme Q] ;
  (6) slot `mod` corrigé main.rs `:36↔:37`).

## Contexte + méthode (Workflow multi-agents, 2026-07-16)

- Mission : split du domaine curators de `http.rs` (8111 l post-Q) vers `curators_api.rs`,
  **move PUR verbatim token-identical** (0 route path, 0 bump wire, 0 dep, 0 logique), sous
  discipline étendue PO-10 (les tests router-driven du domaine co-migrent via le harness partagé
  `test_support.rs` ; la famille golden 9/9 RESTE atomique dans `test_support.rs` et ne migre PAS).
- Dossier reconcilié : 6 inventaires (INV prod-moveset, routes, tests, coupling, docs-frontier,
  imports) + 1 INV sécurité + 1 arbitrage Q1 + 5 scans (S1a OSS/axum, S1b deps/CVE, S2 historique,
  S3 threat-model, S4 wire-format) + **3 critics adversariaux** (moveset, compile, process).
  **Les critics PRIMENT sur les inventaires quand ils réfutent avec preuve disque** — 4 réfutations
  matérielles intégrées ci-dessous (§1). Tous les faits load-bearing re-vérifiés sur disque avec
  `fichier:ligne`.
- Modèles du pattern (à imiter) : `shard_session_http_api.rs` (Phase N), `seed_api.rs` (Phase O,
  1ᵉʳ helper carry + fixture promue), `frost_api.rs` (Phase P, 1ᵉʳ EXECUTE pur), `coordinator_api.rs`
  (Phase Q, 1ᵉʳ 0-DTO-local + promotion `make_test_submission`), `test_support.rs` (Phase N2,
  harness + golden 9/9). **Phase R = 1ᵉʳ bump de visibilité d'un symbole PARTAGÉ** (`ErrorResponse`),
  ce qu'aucun split N→Q n'a exigé (tous importaient des symboles DÉJÀ `pub` de `crate::http`).

## 1. Faits corrigés (vérifiés disque) — réconciliation bundle × critics

1. **Bornes du plan `http.rs:884-1102` DOUBLEMENT STALES (pré-Phase-N).** `884-885` = la bannière
   générique `// Handlers` ; la fenêtre `884-1102` balaie des STAY (`health` `887-895`, `info`
   `897-902`, `project_info` `904-922`, `browse` `1011+`) et **rate `default_curators` @1935**. Le
   domaine réel est **NON-CONTIGU en 4 tranches** (§2), re-dérivées par NOM (règle standing depuis N).

2. **Correction de symbole (bundle unanime + critics) : `SubscribeCuratorResponse` (seed `~:748`)
   N'EXISTE PAS** (grep crate-wide = 0 hit). La DTO de réponse partagée par subscribe + unsubscribe
   s'appelle **`SubscriptionsResponse` (`http.rs:750`)**. Extraire par NOM, jamais par le nom du seed.

3. **`ErrorResponse` est une DTO PARTAGÉE — RESTE dans `http.rs`, bumpée `pub(crate)`.**
   `struct ErrorResponse` privée `http.rs:853` (champ privé `error` `:854`), construite au site
   curators `:877` (via `runtime_error_to_response`) **ET par 5 handlers NON-curators qui restent** :
   `:1230`, `:1322`, `:1962`, `:1978`, `:2014` (browse/publish/directory/panic). Grep confirmé.
   Elle NE PEUT PAS migrer.

4. **CRITIC:COMPILE — RÉFUTATION LOAD-BEARING (compile-testée) : la « 1-bump / ErrorResponse
   untouched » du bundle NE COMPILE PAS.** Les têtes de `inv:imports` / `inv:coupling` /
   `scan:s1a-oss` prétendent qu'on peut garder le helper dans `http.rs` bumpé `pub(crate)` en
   laissant `ErrorResponse` **privée** avec **1 seul** bump. FAUX (rustc) :
   - un `pub(crate) fn runtime_error_to_response(...) -> (StatusCode, Json<ErrorResponse>)` avec
     `ErrorResponse` module-privée déclenche `error: type ErrorResponse is more private than the
     item` [lint `private_interfaces`, **erreur sous le gate CI exact** `clippy … -- -D warnings`] ;
   - le site d'appel `crate::http::runtime_error_to_response(e).into_response()` depuis
     `curators_api.rs` déclenche **`error: type ErrorResponse is private`** — l'effacement par
     `.into_response()` NE dispense PAS (reachability du type du receveur, casse même un `cargo build`
     nu). Le claim `scan:s1a-oss` « never needs to NAME ErrorResponse » est explicitement réfuté.
   - **Coût réel des deux chemins = 2 bumps `pub(crate)`** (pas 1). Le chemin RECOMMANDÉ
     (helper-STAYS) = `fn runtime_error_to_response` (`:857`) **+** `struct ErrorResponse` (`:853`),
     champ `error` **reste privé** (la construction reste intra-`http.rs`). Le chemin helper-MOVES
     = `struct ErrorResponse` **+** champ `error` (E0451 littéral cross-module) + retrait de
     `CuratorRuntimeError` de l'import `http.rs:54`. Les deux coûtent 2 bumps ; helper-STAYS gagne
     car il garde le champ privé (exposition minimale). **Disposition retenue : helper-STAYS.**

5. **CRITIC:MOVESET + CRITIC:COMPILE — slot `mod` corrigé.** `inv:prod-moveset` disait « entre
   `mod contributor_api;`(:35) et `mod coordinator_api;`(:36) » : FAUX alphabétiquement
   (`con` < `coo` < `cur`). Disque main.rs : `mod contributor_api;` `:35`, `mod coordinator_api;`
   `:36`, `mod deploy;` `:37`. Slot correct = **entre `:36` et `:37`** (`coordinator` < `curators`
   < `deploy`). Cosmétique (l'ordre `mod` est compile-indifférent) mais le fait du bundle était faux.

6. **ARBITRAGE Q1 = IN (déviation vs plan).** Le `sprint82_plan.md` route `default_curators` en
   Phase S4. `inv:q1-default-curators` prouve **0 divergence de destination** (S4 nomme DÉJÀ
   `curators_api.rs` comme foyer) : IN = pur réordonnancement. OUT re-pointe `build_router` DEUX
   fois pour un seul module et **orpheline 2 tests curators** (`default_curators_returns_empty`
   `:4471`, `_configured_list` `:4524`) dans `http.rs::tests` entre R et S4 → viole le critère
   PO-10 « domaine complet, jamais de test orphelin ». Précédent Phase O : Q1 keep-online absorbé IN.
   **Retenu : IN.** (Si le PO tranche OUT au review : move-set prod = 6 items / 124 l, route `:364`
   reste locale, `DefaultCuratorsResponse` + ses 2 tests restent dans `http.rs`.)

7. **Frontière FORM-couplée (≠ Phase Q path-only).** `web/src/api/daemon.ts` reflète les formes des
   DTOs via Zod `.strict()` (`DaemonCuratorsResponseSchema` `:104-109`, `SubscriptionsResponseSchema`
   `:117-121`) et `daemon.ts:102` ancre le NOM DE FICHIER (`« CuratorsListResponse in
   nexus-shell-daemon/src/http.rs »`). Le move est **wire-byte-identique** (0 route/champ/status) →
   **0 contrat à re-fermer, `frontier_closure` Phase-T = N/A** (0 nouvelle frontière) — MAIS le move
   **n'est PAS doc-touch-free** : 3 refs file-ancrées deviennent stales (§9). Le claim `inv:tests`
   « path-based CLEAR, 0 web index » est réfuté par `critic:process` sur ce point précis (localisé à
   `inv:tests` ; l'agrégat récupère la bonne lecture via `inv:coupling` + `scan:s4-wire`).

8. **`default_curators` HANDLER vs `DaemonHttpState.default_curators` CHAMP — collision bénigne.**
   Le handler `default_curators` (`:1935`) vit dans le namespace fn ; le champ
   `state.default_curators` (`:96` `pub`, lu au `:1940`) reste dans `DaemonHttpState` (STAY). Le
   re-point route utilise le chemin complet `crate::curators_api::default_curators` (0 `use`) → 0
   ombre. `test_support.rs:161 default_curators: vec![]` = initialiseur du CHAMP (golden family), INTACT.

## 2. Move-set FINAL — Production → `curators_api.rs` (helper-STAYS)

**4 tranches NON-CONTIGUËS. 4 handlers privés `async fn` → `pub(crate) async fn` (routes re-pointées
full-path). 4 DTOs `pub struct` migrent verbatim (attributs `#[serde(deny_unknown_fields)]` inclus,
NE PAS reflow). `runtime_error_to_response` + `ErrorResponse` NE MIGRENT PAS (§1.3/§1.4 — restent
dans `http.rs`, bumpés `pub(crate)`).**

| # | Symbole | Kind | Bornes | Route (path inchangé) | Tranche |
|---|---|---|---|---|---|
| 1 | `SubscribeCuratorRequest` | DTO | `728-742` | body POST subscribe | A |
| 2 | `SubscriptionsResponse` | DTO | `744-752` | réponse subscribe + unsubscribe | A |
| 3 | `CuratorsListResponse` | DTO | `754-769` | réponse GET curators | A |
| 4 | `DefaultCuratorsResponse` (**Q1**) | DTO | `827-833` | réponse GET default-curators | B |
| 5 | `list_curators` | handler | `924-933` | `GET /api/daemon/curators` (`:285`) | D |
| 6 | `subscribe_curator` | handler | `935-990` | `POST /api/daemon/curators/subscribe` (`:286`) | D |
| 7 | `unsubscribe_curator` | handler | `992-1009` | `DELETE /api/daemon/curators/{pubkey}` (`:287`) | D |
| 8 | `default_curators` (**Q1**) | handler | `1932-1943` | `GET /api/daemon/default-curators` (`:364`) | E |

- **Total move-set = 8 items / 143 lignes prod.** Core non-Q1 (items 1-3, 5-7) = 124 l ; items Q1
  (4, 8) = 19 l. Bornes byte-confirmées par `critic:moveset` (aucune borne ne draine un STAY).
- Tranche A `728-769` : item 1 inclut son doc-comment G-3 `728-736` (rationale audit Sprint 8, load-
  bearing, verbatim) + derive `737` + serde `738` + struct `739-742`. Items 2/3 idem.
- **Interleaving STAY à ne PAS balayer** (entre les tranches) : `BrowseListResponse` `771-786`
  (`#[cfg(test)]`), `PublishRequest` `788-819`, `PublishResponse` `821-825`, `PublishBlobResponse`
  `835-840`, `NeighborhoodResponse` `842-848`, **`ErrorResponse` `850-855`** (PARTAGÉE, STAY bumpée),
  `health` `887-895`, `info` `897-902`, `project_info` `904-922`, tous les handlers browse/publish/
  directory `1011-1930`. Extraire chaque tranche INDÉPENDAMMENT (une copie-bloc `:728-1009` ou
  `:884-1102` traînerait des STAY).
- **Piège de délimitation nommée** : le handler `info` (`897-902`), dont le doc dit « for the
  shell's Browse / Curators page header », est domaine `info`/snapshot — **EXCLU**. La route SPA
  `/curators` (ServeDir fallback, `:581`) est un chemin DIFFÉRENT — hors move-set.
- **`ReachabilityBucket`** : PAS un symbole de `http.rs` (0 grep) ; le seed `:774` est le mot
  « reachability bucket » dans le doc-comment de `CuratorsListResponse`. N/A.

## 3. Move-set FINAL — Tests → `curators_api.rs::tests` (discipline PO-10)

**10 `#[tokio::test]` router-driven co-migrent** (Q1=IN inclus). Tous pilotés par le harness partagé
`test_support` (`build_test_router`/`mk_state`/`mk_state_with_mode`/`mk_state_with_mode_tx`) via
`.oneshot` + URI-string ; **ZÉRO appel direct de handler**.

1. `list_curators_returns_empty_when_nothing_cached` `3756-3774`
2. `subscribe_then_list_then_delete_happy_path` `3776-3840`
3. `subscribe_rejects_extra_fields` `3842-3879` (assert 422 `deny_unknown_fields`)
4. `subscribe_rejects_bad_pubkey_hex_as_400` `3881-3900`
5. `subscribe_curator_pushes_hot_join_for_subscribed_peer` `3902-3948` (S81-E3, `mk_state_with_mode_tx(Normal,tx)`, assert 1 `GossipCmd::JoinPeers`)
6. `subscribe_curator_in_duress_pushes_no_hot_join` `3950-3986` (**lock duress-empty-channel**, `mk_state_with_mode_tx(Duress,tx)`)
7. `subscribe_curator_invalid_hex_pushes_no_hot_join` `3988-4021`
8. `default_curators_returns_empty_when_unconfigured` `4471-4488` (**Q1**)
9. `default_curators_returns_configured_list` `4524-4608` (**Q1**, construit `DaemonHttpState` inline)
10. `daemon_boot_in_duress_mode_rejects_curator_subscribe_real` `5147-5186` (#B-rt-2, S20-B, POST subscribe sous Duress → 200 ACK, attention-set vide)

**E0425 promotion class = VIDE.** Aucune fixture locale `http.rs::tests` n'est partagée entre un test
migrant et un test STAY (contraste Phase Q `make_test_submission`). Les 10 tests dépendent uniquement
des helpers `pub(crate)` de `test_support`, de crates externes (`KeyPair`, `hex`, `create_node`,
`CuratorRuntime`, `BrowseAggregator`, `BlobServeCache`, `PowSolveCache`) et des 4 DTOs qui migrent
avec les handlers. **Rien à promouvoir vers `test_support.rs`.**

**Fixture inline lourde (optionnel, non-bloquant)** : `default_curators_returns_configured_list`
(`4524-4608`) construit `DaemonHttpState{…}` champ-par-champ (`4537-4592`, `mk_state` ne pouvant pas
fixer `default_curators`), tirant ~10-30 imports de type dans le scope test de `curators_api.rs`
(`create_node`, `CuratorRuntime`, `BrowseAggregator`, `BlobServeCache`, `PowSolveCache`, `KeyPair`,
`RwLock`, `SystemTime`, …). **Précédent** : `seed_api.rs::tests` / `coordinator_api.rs::tests` tirent
déjà `create_node`. Migrer verbatim (défaut discipline). Amélioration facultative : un helper
`test_support::mk_state_with_default_curators(Vec<String>)` réduirait la surface d'imports — **NON
requis pour la phase** (le migrer verbatim est conforme).

**Golden — RESTE, atomique :** `golden_http_curators_domain` (`test_support.rs:543-571`, `#[tokio::test]`).
2 cas : `GET /api/daemon/curators` → 200 `{"entries":[],"subscribed_curators":[]}` ; `DELETE
/api/daemon/curators/deadbeef` → 400 `{"error":"invalid curator pubkey hex (expected 64 lowercase
chars): deadbeef"}`. **NE MIGRE PAS** (bloc atomique 9-golden PO-10, observateur externe). Reste VERT
car paths byte-identiques + chaîne `unsubscribe → CuratorRuntime → BadPubkeyHex → runtime_error_to_response
→ ErrorResponse` préservée (handler + helper + struct restent verbatim, helper/struct STAY).

**Tests STAY (pièges à NE PAS balayer)** — touchent `curator_runtime` mais tapent un AUTRE domaine :
- `info_reflects_live_curator_runtime_counts` `4023-4047` — domaine `info` (`GET /api/daemon/info`,
  `state.curator_runtime.subscribe(..)` en setup, assert du snapshot). STAY.
- `browse_returns_empty_list_when_no_curators_cached` `4049-4071` — domaine `browse`. STAY.
- `daemon_boot_in_duress_mode_publishes_fake_curator_empty` `5091-5145` (#B-rt-1) — domaine
  publish/boot (`POST /publish` + `GET /browse` ; `PublishRequest`/`PublishResponse`/`BrowseListResponse`).
  **Piège d'adjacence** : sibling du #B-rt-2 migrant, séparés par la bannière S20-B `:5080` avec ce
  test publish AU MILIEU — trancher par ROUTE pilotée, pas par proximité textuelle. STAY.
- `spa_fallback_serves_curators_as_html_document` `5269-5293` — domaine SPA ServeDir (`GET /curators`
  = document HTML, pas l'API). STAY.

**Comptes :** exactement **10 tests relocalisés** (8 si Q1=OUT), tous `#[tokio::test]`. **Delta ±0
EXACT** (pure relocation, 0 ajout/suppression). Golden count inchangé. Baseline **Win 2108 / Docker 2112**.

## 4. Plan d'imports

**`curators_api.rs` — en-tête (mirror `seed_api.rs`) :**
```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Curators loopback HTTP domain — extrait verbatim de `http.rs`
//! (Sprint 82 Phase R, discipline étendue PO-10 : les tests router-driven
//! du domaine co-migrent ci-dessous via le harness partagé `crate::test_support`).
//! Liste des listes curator signées cachées + attention set (GET /curators),
//! subscribe avec la porte duress S20-B + le hot-join gossip S81-E3, unsubscribe,
//! et default-curators (config `[curator]`, S11-B). Les routes restent
//! enregistrées dans `crate::http::build_router` À L'INTÉRIEUR de `authed_routes`
//! et re-pointent ici par chemin complet ; paths, formes JSON et status codes
//! INCHANGÉS. Le helper `runtime_error_to_response` et la DTO PARTAGÉE
//! `ErrorResponse` restent dans `http.rs` (consommés par 5 handlers non-curators).

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::http::{DaemonHttpState, runtime_error_to_response};
```
- **Justification par-import** (vérifiée sur les 8 corps) : `Arc` (`State<Arc<DaemonHttpState>>`
  partout) ; `Path` (`unsubscribe_curator` `:996 Path(pubkey): Path<String>`) ; `State` (les 4
  handlers) ; `StatusCode` (`OK`/`BAD_REQUEST`) ; `IntoResponse` (`impl IntoResponse` + `.into_response()`) ;
  `Json` (bodies) ; `Serialize, Deserialize` (les 4 DTOs dérivent `#[derive(Debug,Clone,Serialize,Deserialize)]`) ;
  `debug` (les 4 handlers appellent `debug!` et RIEN d'autre — 0 `info!`/`warn!`) ; `DaemonHttpState`
  (type State, `pub struct` `http.rs:75`) ; `runtime_error_to_response` (`pub(crate)` bumpé,
  appelé `:988`/`:1007` des handlers migrants).
- **NON importés** (full-path/verbatim, piège N2 over-import) : `nexus_core_rs::CuratorListEntry`
  reste full-path dans `CuratorsListResponse.entries` (`:763`) — PAS de `use` ; `crate::noop_identity::{curator_subscribe_in_duress,SubscribeOutcome}`
  full-path (`:947-948`) ; `crate::runtime::GossipCmd` full-path (`:976`). PAS de `serde_json`
  (aucun `json!` dans le slice), PAS d'`axum::response::Response` (aucun type nommé), PAS de
  `CuratorRuntimeError` (le helper qui l'utilise RESTE dans `http.rs`), PAS d'`ErrorResponse`
  (RESTE dans `http.rs`, jamais nommée par `curators_api.rs` — l'appel passe par
  `runtime_error_to_response(e).into_response()`).

**`curators_api.rs::tests` — bloc imports :**
```rust
use super::*;
use axum::body::to_bytes;
use axum::http::{Method, Request};
use nexus_core_rs::{KeyPair, create_node};
use tower::ServiceExt;
use crate::test_support::*;
```
- `create_node` **REQUIS ici** (contraste Phase Q `KeyPair`-seul) : `default_curators_returns_configured_list`
  construit `DaemonHttpState` inline avec `create_node` (`~:4543`). `KeyPair` requis (génération de
  clés dans les round-trips). Vérifier au build : si un import test s'avère inutilisé sous
  `-D warnings`, retirer (le gate l'attrape ; pré-empté ici mais dépend du corps exact des 10 tests).

**`http.rs` côté départ :**
- **BUMPS `pub(crate)` (2, §1.4)** : `struct ErrorResponse` `:853` privé → `pub(crate)` (champ `error`
  `:854` **reste privé**) ; `fn runtime_error_to_response` `:857` privé → `pub(crate)`. Les deux
  RESTENT dans `http.rs`. Après le départ des 2 appelants curators, le helper n'a plus que des
  appelants cross-module → `pub(crate)` + `use` externe supprime le `dead_code` (pas de warning).
- Retirer les 4 tranches DTO/handler (§2, chacune indépendamment). `cargo fmt` collapse les blanks.
- **AUCUN import `http.rs` ne devient orphelin** (C1 vérifié) : `delete` (routing `:46`) encore
  utilisé `:557` après le départ de la route `:287` ; `Path` encore utilisé `:2030/:2311/:2634` ;
  `CuratorRuntimeError` (`:54`) encore utilisé par le helper RESTANT `:857/:864-873` ;
  `Serialize/Deserialize` par d'autres DTOs ; `debug/info/warn`, `get/post/delete` largement.
- Retirer le cluster tests migrés (`3756-4021` + `4471-4608` + `5147-5186`) → `curators_api.rs::tests`.

**`main.rs` :** ajouter `mod curators_api;` au slot **entre `mod coordinator_api;` (`main.rs:36`) et
`mod deploy;` (`main.rs:37`)** (§1.5) — mod NORMAL (pas `cfg(test)`). Pas de `lib.rs` (main.rs = racine).

## 5. Re-points de routes (4 sites, chemin complet `crate::curators_api::`, paths byte-identiques)

- `http.rs:285` `get(list_curators)` → `get(crate::curators_api::list_curators)`
- `http.rs:286` `post(subscribe_curator)` → `post(crate::curators_api::subscribe_curator)`
- `http.rs:287` `delete(unsubscribe_curator)` → `delete(crate::curators_api::unsubscribe_curator)`
- `http.rs:364` `get(default_curators)` → `get(crate::curators_api::default_curators)` (**Q1=IN**)

Les 4 routes sont mono-ligne (0 piège rustfmt multi-ligne, contraste Phase Q `verify_chain`). Toutes
dans `authed_routes` (`http.rs:282`), fusionné `.merge(authed_routes)` `:576` + `.with_state(state)`
`:577`. **Aucun autre re-point de code** : les 4 handlers sont référencés UNIQUEMENT à ces sites route
+ leurs défs (grep crate-wide : 0 ref hors `http.rs` ; les tests sont router-driven par URI-string,
jamais par nom de fn ; `nexus-test-harness/src/lib.rs:167` a SON PROPRE helper HTTP-client par route,
pas un import de symbole).

## 6. Couplages d'état + visibilités (bumps minimaux justifiés)

**Champs `DaemonHttpState` accédés (tous déjà `pub`, 0 édit du struct) :**
- `curator_runtime` — les 4 handlers curators (`list_snapshot`/`subscribed_pubkeys_hex`/`subscribe`/`unsubscribe`).
- `identity_mode` (`:106`, `pub`) — porte duress de `subscribe_curator` (`:947`).
- `gossip_cmd_tx` (`:93`, `pub`, `crate::runtime::GossipCmdTx`) — hot-join S81-E3 (`:974-979`).
- `default_curators` (`:96`, `pub`, `Vec<String>`) — `default_curators` handler (`:1940`). **Reste dans le struct.**

Tous accédés via l'extracteur `State<Arc<DaemonHttpState>>`. `crate::noop_identity::{curator_subscribe_in_duress,SubscribeOutcome}`
(`pub` `noop_identity.rs:117/76`) et `crate::runtime::GossipCmd::JoinPeers` (`pub enum` `runtime.rs`)
sont des items `crate::` full-path → résolvent inchangés depuis `curators_api.rs`.

**Bumps de visibilité `http.rs` (2 — au-delà de la règle « handlers → pub(crate) ») :**
| Symbole | `http.rs` | Avant | Après | Raison | Disposition |
|---|---|---|---|---|---|
| `struct ErrorResponse` | `:853` | privé | **`pub(crate)`** | PARTAGÉE (5 non-curators) + signature du helper `pub(crate)` (`private_interfaces`) + reachability call-site | STAY http.rs |
| champ `error` | `:854` | privé | **privé (inchangé)** | construction intra-`http.rs` (helper reste) | STAY http.rs |
| `fn runtime_error_to_response` | `:857` | privé | **`pub(crate)`** | appelé cross-module par `curators_api.rs` (`:988`/`:1007` migrants) | STAY http.rs |
| 4 handlers | `:926/:938/:994/:1935` | privé | **`pub(crate)`** | re-pointés full-path dans `build_router` | MOVE |
| 4 DTOs | `:739/:750/:757/:830` | `pub` | **`pub` (inchangé)** | déjà `pub` (harmless dans `mod` privé) | MOVE |

**Total transformations `http.rs` = 4 handlers `pub(crate)` + `ErrorResponse` `pub(crate)` +
`runtime_error_to_response` `pub(crate)`.** Phase R n'est PAS un « handlers-only → pub(crate) » pur :
elle porte exactement 1 bump de symbole PARTAGÉ (`ErrorResponse`), le 1ᵉʳ de la série de splits S82.

## 7. Invariants sécurité VERBATIM (à préserver byte-identique — fidélité, pas design)

- **Porte duress `subscribe_curator` (S20-B, `http.rs:943-957`)** : 1ᵉʳ statement exécutable après
  `debug!(:942)`. En `IdentityMode::Duress`, `curator_subscribe_in_duress(state.identity_mode) ==
  SubscribeOutcome::Noop` → **return 200 `SubscriptionsResponse{subscribed_curators: Vec::new()}`**,
  NE mute PAS l'attention-set, N'ATTEINT NI `curator_runtime.subscribe` NI le push JoinPeers. Ordre à
  préserver : `debug!` → **duress early-return** → `subscribe` → **JoinPeers push** → 200. La duress
  early-return DOIT rester AVANT subscribe ET le push (rend le push inatteignable sous la clé leurre).
- **Hot-join `GossipCmd::JoinPeers` (S81-E3, `http.rs:958-987`)** : dans le bras `Ok`,
  `let _ = state.gossip_cmd_tx.send(GossipCmd::JoinPeers(vec![req.curator_pubkey_hex.clone()])).await`
  (best-effort, résultat jeté). `subscribe_curator` est **le SEUL producteur de `GossipCmd::JoinPeers`**
  (assert `subscribe_curator_in_duress_pushes_no_hot_join` `:3950` ; consommateur `runtime.rs` STAY).
  Le push est DANS le bras Ok APRÈS la mutation → hex-invalide (bras Err) ne pousse jamais (`:3988`),
  duress ne pousse jamais (`:3950`). Migre verbatim.
- **Absence de porte duress sur `unsubscribe_curator` (INTENTIONNEL, S20-B)** : retirer un abonnement
  leurre est inoffensif ; sous duress l'ensemble n'a jamais grossi. **NE PAS ajouter de porte** —
  l'absence est un invariant, l'ajouter serait un changement de comportement (casse le token-identical).
- **Validation pubkey = PASSTHROUGH** : `unsubscribe_curator`/`subscribe_curator` passent le hex brut
  à `curator_runtime.{unsubscribe,subscribe}` ; la validation vit dans `nexus-shell-daemon-core`
  (`parse_pubkey_hex` → `CuratorRuntimeError::BadPubkeyHex` si ≠ 64-char lowercase). Le mapping
  erreur→status (`runtime_error_to_response`, `BadPubkeyHex→400`, autres→422, `Persistence→500`)
  RESTE dans `http.rs` (helper STAY) — inchangé, la crate core est intouchée.
- **Auth par composition de routeur (meta-invariant)** : les 4 routes sont dans `authed_routes` ;
  l'auth (bearer X-SBFB-Token + Host loopback + Origin) est appliquée UNE fois au niveau groupe
  (`.layer(from_fn_with_state(auth, auth_required))` `http.rs:571`), pas dans les handlers. Déplacer
  les corps est **auth-neutre** ; `build_router` (STAY) garde les routes dans `authed_routes` avec
  paths byte-identiques.

## 8. S1a/S1b/S2/S3/S4 — synthèse des scans

- **S1a (OSS / axum)** : move intra-crate pur, **0 SOTA/redesign**. Le pattern (module handler +
  re-point full-path + harness `test_support`) est prouvé 4× (N/O/P/Q). Ordre extracteurs (State
  `FromRequestParts` d'abord, `Json`/`Path` dernier) satisfait verbatim. `Path<String>` prouvé dans
  seed_api/shard_session. Seul piège écosystème = erreur opaque `Handler` si un extracteur ne résout
  pas identiquement → neutralisé par le plan d'imports §4 (mirror seed_api, signatures verbatim ;
  diagnostic `#[debug_handler]` si build rouge, PAS redesign). Multi-verb/per-route middleware =
  structurellement inapplicables.
- **S1b (deps/CVE)** : **0 delta `Cargo.toml`, 0 delta `Cargo.lock`, 0 feature**. Toutes les crates
  consommées déjà directes (`axum` `:42`, `serde` `:58`, `serde_json` `:59`, `tracing` `:46`,
  `nexus-core-rs` `:29`, `nexus-shell-daemon-core` `:21`). `nexus_core_rs::CuratorListEntry` +
  `nexus_shell_daemon_core::iroh_runtime::CuratorRuntimeError` déjà directs. Supply-chain : hickory
  bump déjà atterri Phase K ; veilles trigger-driven ; zones rouges (iroh 0.98 pin / wasmtime / libcrux)
  inchangées et sans rapport. **Rien à rouvrir en R.**
- **S2 (historique)** : NO DESIGN-CONFLICT. Domaine fondé S7-C (818429d : 3 routes curator +
  `runtime_error_to_response`), config-list S11-B (e5cc165 : `GET /default-curators` +
  `DaemonHttpState.default_curators`), duress S20-B (c32ecb3), hot-join S81-E3 (e05338f). Crypto
  curator (`nexus-core-rs/curator.rs`, `CURATOR_LIST_FORMAT_VERSION=1`, `DOMAIN_CURATOR_LIST_V1`)
  explicitement INTOUCHÉE par le move. **Aucun commit du domaine ne contient « must remain in
  http.rs » / co-location / DEVIATION / split rejeté.** Précédent N/O/P/Q = pattern explicite.
- **S3 (threat-model)** : surface INCHANGÉE, move order-neutral ET surface-neutral. Les 4 routes
  restent **T0** (`LOOPBACK_ENDPOINTS_TRUST_TIERS.md:67` liste `GET /api/daemon/curators` T0/T0 par
  ROUTE-PATH byte-identique ; les 3 autres implicitement T0). **0 clause doc ne pin un handler
  curator à une localisation FICHIER** (contraste Phase O qui a dû re-pointer `L1019` → `seed_api.rs`).
  Les refs `THREAT_MODEL.md` curators sont concept-level (`L44/76/201/453/592/623/1802`, « curator
  lists »/`CuratorVouched`/`T-CURATOR-VOUCH`) ; le seul `http.rs:symbole` proche (`L1039
  build_sign_announce_directory`) est domaine node-directory (STAY, drift FUTUR pour un split
  nodes, pas R). **0 édit threat-doc requis par R.**
- **S4 (wire-format)** : **0 wire bump PROUVÉ**. Types wire curator (`CuratorList`/`CuratorListEntry`/
  `CURATOR_LIST_FORMAT_VERSION`/`DOMAIN_CURATOR_LIST_V1`) dans `nexus-core-rs`, intouchés
  (`CuratorsListResponse.entries` référence `CuratorListEntry` par import, pas redéfinition). Les 4
  DTOs loopback migrent verbatim (`#[serde(deny_unknown_fields)]` load-bearing, copier l'attribut).
  Golden `golden_http_curators_domain` = preuve mécanique (cas-2 400 dépend de la chaîne
  handler+helper+`ErrorResponse` — tous verbatim/STAY). Frontière **FORM-couplée** (§1.7) mais
  wire-byte-identique → 0 contrat à re-fermer. Pré-launch policy : 0 bump requis, 0 permis.

## 9. Docs-contrat, frontière web et gates CI

**Frontière web (path + FORM) — verdict `frontier_closure` Phase-T = N/A** (0 nouvelle frontière, wire
byte-identique) MAIS **3 re-points file-ancrés REQUIS** (élevés de « optionnel » à REQUIS par
`critic:process` — laisser stale = mensonge d'attribution fichier, finding review/Codex, pattern O/Q ;
non-gatés, CI reste verte quoi qu'il arrive) :
1. **`web/src/api/daemon.ts:102`** — `« CuratorsListResponse in nexus-shell-daemon/src/http.rs »` →
   `curators_api.rs` (commentaire TS, non-gaté).
2. **`docs/rust/PATTERNS.md:935-938`** (§G-3 closure) — attribue `SubscribeCuratorRequest`(line 162)/
   `SubscriptionsResponse`(173)/`CuratorsListResponse`(180) à `http.rs` avec numéros DÉJÀ stales.
   Re-attribuer les 3 DTOs migrant à `curators_api.rs` + **laisser `BrowseListResponse` (line 201)
   noté `http.rs`** (domaine browse, STAY) + **droper les numéros inline** (les noms ne pourrissent
   pas, les numéros oui — pattern O/Q).
3. **`docs/shell/PATTERNS.md:1177`** (P19) — `« http.rs (GET /default-curators) »` → `curators_api.rs`
   (**uniquement si Q1=IN** ; sinon reste `http.rs` jusqu'à S4). Bare filename, non-gaté.

**INTACT (config-domain, ≠ DTO migrant)** : `docs/shell/PATTERNS.md:1166/1176/1309/1311`
(`CuratorConfig.default_curators` dans `config.rs`, sans rapport avec `DefaultCuratorsResponse`).
**NE JAMAIS toucher** : `docs/claude/SPRINT_LOG.md` (narration historique S56/S57/S59).

**Gates CI — AUCUN NE CASSE (vérifié disque, `critic:process` PROUVÉ pas asserté) :** les 3 scripts
(`check-frontier-contracts.sh`, `check-sharding-docs.sh`, `check-factory-docs.sh`) ont **0 coupling
`http.rs:<symbole-curator>`** (grep curator/subscribe/CuratorsList/Subscriptions/default-curators sur
`scripts/check-*.sh` = No matches). Le seul `http.rs` source-ref dans un gate =
`check-sharding-docs.sh:90 anchor_present '…http.rs' 'shard-session'` (ancre SHARD, intouchée). Aucun
ref numérique `http.rs:NNNN` gaté. **Obligations de construction** (à satisfaire, pas des casses) :
(a) SPDX ligne 1 de `curators_api.rs` (`check-spdx.sh`, triple surface) ; (b) `phase-review-cross-check.yml`
exige `sprint82_phase_r_review.md` (satisfait par le workflow normal de phase).

## 10. Hazards compile et plan d'exécution en tranches

**Hazards compile (tous pré-empbés) :**
1. **`private_interfaces` / `type ErrorResponse is private`** si `ErrorResponse` reste privée après
   bump du helper → **bumper AUSSI `struct ErrorResponse` `:853` en `pub(crate)`** (champ privé), §1.4/§6.
2. **E0451 / ErrorResponse field private** — N/A sous helper-STAYS (construction reste intra-`http.rs`).
   (Le chemin helper-MOVES l'exigerait — non retenu.)
3. **`unused_imports`** si import test inutilisé sous `-D warnings` → vérifier `create_node`/`KeyPair`
   au build ; retirer ce qui n'est pas consommé par les 10 corps.
4. **Sur-capture de tests cross-domaine** (`info`/`browse`/`boot-publish`/`SPA`) → trancher par ROUTE
   pilotée, pas par proximité textuelle (§3, piège d'adjacence #B-rt-1/#B-rt-2).
5. **Extraction par le mauvais nom** (`SubscribeCuratorResponse` inexistant) → extraire `SubscriptionsResponse`.
6. **Ordre interne de `subscribe_curator`** — déplacer le corps en UN bloc contigu ; NE PAS réordonner
   (duress AVANT subscribe/push).

**Plan d'exécution (ordre suggéré) :**
1. Préflight Workflow `nexus-phase-preflight-deep` (déjà fait — ce document).
2. Créer `curators_api.rs` : SPDX + `//!` + use block (§4) ; coller les 4 handlers verbatim
   (tranches D/E), `async fn` → `pub(crate) async fn` ; coller les 4 DTOs verbatim (tranches A/B,
   attributs serde inclus).
3. Dans `http.rs` : bumper `struct ErrorResponse` `:853` + `fn runtime_error_to_response` `:857` en
   `pub(crate)` (helper + struct RESTENT) ; retirer les 4 tranches DTO/handler.
4. Ajouter `#[cfg(test)] mod tests` à `curators_api.rs` : coller les 10 tests verbatim + bloc imports
   test (§4) ; retirer le cluster tests migrés de `http.rs`.
5. Re-pointer les 4 routes (§5) ; ajouter `mod curators_api;` à main.rs slot `:36↔:37`.
6. Ré-honnêter in-phase les 3 refs docs (§9) ; laisser INTACT config/SPRINT_LOG.
7. Preuve (§11).

## 11. Protocole de preuve (pattern outillé standing N→Q)

- **(a) Extraction par NOM + manifest dry-run** du move-set (8 items prod + 10 tests) avant tout Edit.
- **(b) `cmp` token-level** sur les DEUX slices : PROD (4 handlers + 4 DTOs) ET TEST (10 tests), avant/après
  = byte-identique modulo `async fn`→`pub(crate) async fn` (handlers) et localisation module.
- **(c) Goldens 9/9 VERTS** — `golden_http_curators_domain` (observateur externe) prouve 0 drift JSON.
- **(d) Delta nextest ±0 EXACT** sur les DEUX plateformes : **Win 2108** + **Docker 2112** (image
  `sbfb-ci`, mount `/workspace` OBLIGATOIRE + `bash -c` [jamais `bash -lc`], `SBFB_TEST_HTTP_TIMEOUT_SECS=120`,
  gros cargo → `run_in_background`). Tout écart Win≠2108 / Docker≠2112 = test droppé/dupliqué → STOP.
- **(e) `cargo fmt --all --check` + `clippy --workspace --all-targets --locked -- -D warnings`** verts
  (le gate `-D warnings` est l'arbitre du bump `ErrorResponse` : rouge si le bump manque).
- **(f) `git show -- '*Cargo.toml' '*Cargo.lock'`** VIDE (0 delta dep, pattern N/N2/O/P/Q).

## 12. Risques résiduels

- **`ErrorResponse` bump = 1ᵉʳ symbole PARTAGÉ exposé de la série** — expose un struct interne
  crate-wide (via `pub(crate)`). Fidèle et minimal (champ privé), mais un review/Codex le notera
  comme la seule édition non-mécanique de R ; consigné §6 comme adaptation portée.
- **Frontière FORM-couplée** — si une signature DTO était touchée (elle ne l'est pas), le verdict
  `frontier_closure` basculerait en obligation Phase-T. Vigilance : garder les 4 DTOs byte-identiques.
- **Test lourd inline** `default_curators_returns_configured_list` (imports ~10-30 types) — précédent
  seed_api/coordinator_api, mais gonfle le scope test de `curators_api.rs`. Migrer verbatim ; helper
  `mk_state_with_default_curators` = amélioration facultative hors-phase.
- **Q1 OUT (si le PO tranche autrement)** — bascule move-set à 6 items / 124 l, route `:364` locale,
  2 tests `default_curators` restent dans `http.rs` (orphelins jusqu'à S4). Non retenu (§1.6).
- **Drift FUTUR non-R** : `THREAT_MODEL.md:1039 build_sign_announce_directory` deviendra stale à un
  split nodes/directory ultérieur (S2/S3), pas R.

## Verdict: PLAN-ADAPT

Approche du plan **CONFIRMÉE** — split du domaine curators vers `curators_api.rs`, 4 routes inchangées
(T0 authed, byte-identiques), golden-gardé, **0 wire bump / 0 dep / 0 Cargo delta / 0 route path change**.
**0 DESIGN-CONFLICT** : S1a/S1b/S2/S3/S4 unanimes no-conflict ; non-monétaire Day-0 intact, iroh 0.98
pin intouché, crypto curator (`nexus-core-rs`) intouchée, invariants sécurité (duress + JoinPeers
seul-producteur + passthrough validation + auth-par-routeur) préservés par move verbatim, pré-launch
policy respectée.

**Adaptations matérielles requises (chacune prouvée disque) — ce qui distingue R des splits N→Q :**
1. **Bump `pub(crate)` de la DTO PARTAGÉE `ErrorResponse` (`http.rs:853`, champ privé) + du helper
   `runtime_error_to_response` (`:857`)** — 1ᵉʳ bump d'un symbole PARTAGÉ de la série S82. La tête
   « 1-bump / ErrorResponse untouched » du bundle est **REFUTÉE par le critic compile (compile-testé)** :
   les deux chemins coûtent 2 bumps ; helper-STAYS retenu (garde le champ privé). ErrorResponse RESTE
   dans `http.rs` (5 consommateurs non-curators `:1230/:1322/:1962/:1978/:2014`).
2. **Arbitrage Q1 `default_curators` = IN** (route `:364`, DTO `DefaultCuratorsResponse` `827-833`,
   handler `1932-1943`, tests `4471-4488`+`4524-4608`) alors que le plan le route en S4 — justifié : 0
   divergence de destination (S4 nomme DÉJÀ `curators_api.rs`), OUT double-churn + orpheline 2 tests
   (viole PO-10), précédent O.
3. **Correction du symbole du plan** : `SubscribeCuratorResponse` (seed `~:748`) **n'existe pas** →
   `SubscriptionsResponse` (`http.rs:750`), partagée subscribe+unsubscribe.
4. **Bornes re-dérivées par NOM** — plan `:884-1102` doublement stale (balaie health/info/project_info/
   browse, rate `default_curators`), domaine réel en 4 tranches non-contiguës (§2).
5. **3 re-points docs file-ancrés REQUIS** (frontière FORM-couplée, pas path-only comme Q) :
   `web/src/api/daemon.ts:102`, `docs/rust/PATTERNS.md:935-938`, `docs/shell/PATTERNS.md:1177`.
   `frontier_closure` Phase-T reste N/A (0 nouvelle frontière, wire byte-identique).
6. **Slot `mod` corrigé** : `main.rs` entre `:36` (`coordinator_api`) et `:37` (`deploy`), PAS `:35↔:36`
   (erreur `inv:prod-moveset`).

Tout le reste est P-class (slice extrait par nom, tests router-driven co-migrent, E0425 promotion
class VIDE, golden atomique STAY, 0 route path change, delta ±0 EXACT). Le code suit l'approche corrigée
ci-dessus (§2-§11).
