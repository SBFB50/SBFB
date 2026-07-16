# Sprint 82 Phase Q — Preflight G8 (split domaine coordinator → `coordinator_api.rs`, discipline étendue PO-10)

## En-tête

- **Sprint** : 82 (dette docs-contrat + refacto, 24 phases, C9).
- **Phase** : Q — split du domaine `coordinator` de `http.rs` vers un NOUVEAU module
  `crates/nexus-shell-daemon/src/coordinator_api.rs`.
- **Date** : 2026-07-16.
- **HEAD au preflight** : `1aa7a0f` (Phase P `frost_api.rs` DONE). Phases A→P DONE (17/24).
- **Verdict** : **PLAN-ADAPT** (justifié §11 — approche du plan confirmée, mais adaptations
  matérielles requises : promotion `make_test_submission` vers `test_support.rs` pub(crate)
  [compiler-forcée E0425], ré-honnêteté in-phase de 5 refs docs file-ancrées, arbitrage de
  contradiction C1 sur le doc-comment du module).

## Contexte + méthode (Workflow multi-agents, 2026-07-16)

- Mission : split du domaine coordinator de `http.rs` (8951 l post-P) vers `coordinator_api.rs`,
  **move PUR verbatim** (0 route path, 0 bump wire, 0 dep, 0 logique), sous discipline étendue
  PO-10 (les tests router-driven du domaine co-migrent via le harness partagé `test_support.rs` ;
  la famille golden 9/9 RESTE atomique dans `test_support.rs` et ne migre PAS).
- Dossier reconcilié : 6 inventaires + scans (INV-1 production, INV-2 tests, INV-3 frontière,
  S1a OSS/axum, S1b deps/CVE, S2 décisions historiques, S3 threat-model, S4 wire-format),
  7 claim-checks par handler/doc-ref, 3 critics adversariaux (C1 compile/coupling, C2 docs/gates,
  C3 tests). **Les corrections des claim-checks et des critics PRIMENT sur les inventaires
  initiaux.** Tous les faits load-bearing ci-dessous re-vérifiés sur disque avec `fichier:ligne`.
- Modèles du pattern (à imiter) : `shard_session_http_api.rs` (Phase N, `2e87eef`), `seed_api.rs`
  (Phase O, `542254b`), `frost_api.rs` (Phase P, `1aa7a0f`), `test_support.rs` (Phase N2,
  `c5be6e4`). Phase P = 1ᵉʳ EXECUTE des splits (aucune fixture partagée, 0 helper carry).

## 1. Faits corrigés (vérifiés disque)

1. **Bornes du plan `http.rs:3722-4023` STALES (pré-N).** Le domaine réel par NOM se trouve à
   `http.rs:2217-2516` (bannière S35-B `2217-2219` incluse → close `coordinator_verify_chain`
   `2516`). Re-dérivées par NOM (règle standing du sprint depuis N).
2. **4 handlers PRIVÉS** (`async fn`, aucun `pub`) — bornes exactes vérifiées :
   `coordinator_submit_task` `http.rs:2221-2305`, `coordinator_submit_result` `2311-2438`,
   `coordinator_get_kudos` `2444-2484`, `coordinator_verify_chain` `2490-2516`.
3. **Route `verify_chain` en registration MULTI-LIGNE** (`http.rs:408-411`, éclatée par rustfmt :
   `.route(` 408 / path 409 / `get(coordinator_verify_chain),` 410 / `)` 411) — le handler
   n'apparaît qu'à la ligne 410 ; un grep mono-ligne ne touche que 410 (piège Phase P confirmé
   réel). Les 3 autres routes sont mono-ligne (`405`, `406`, `407`).
4. **C1 REFUTÉ (compile/coupling)** — INV-3 affirmait « les 4 routes sont enregistrées dans
   `build_router`, PAS `authed_routes` » : **FAUX/imprécis**. Les 4 routes (`http.rs:405-411`)
   sont chaînées sur le sous-builder `authed_routes` (`http.rs:282 let authed_routes =
   Router::new()`), qui vit À L'INTÉRIEUR de `build_router` (`http.rs:246`). `public_routes`
   (`http.rs:260`) = seulement `/health` + `/blob-serve`. Le doc-comment du nouveau module doit
   dire « registered in `crate::http::build_router` inside `authed_routes` » (formulation S1a),
   PAS la négation d'INV-3.
5. **0 DTO local dans le slice.** Contrairement à frost_api (4 Request DTO locaux), le domaine
   coordinator ne déclare AUCUN struct/enum dans `2217-2516` : les corps de requête sont des
   types de crates EXTERNES (`nexus_coordinator_rs::types::TaskSubmission` `http.rs:2223`,
   `nexus_core_rs::task::ResultEntry` `http.rs:2313`) et TOUTES les réponses sont inline
   (`serde_json::json!(...)` / `serde_json::to_value(&...)` `http.rs:2279/2395/2465/2506`).
   Aucune contrainte field-order DANS `http.rs`.
6. **0 helper local `http.rs` consommé.** Chaque appel des 4 corps est vers une crate externe
   (`nexus_coordinator_rs::{guardrails,dispatcher,error,kudos_ledger}`, `nexus_core_rs::task`)
   ou un champ pub de `DaemonHttpState` ou std/serde_json/tracing/tokio/hex/axum. **Aucun bump
   `pub(crate)` de symbole `http.rs` interne requis** (contraste O/seed). Seuls les 4 handlers
   passent privé → `pub(crate)`.
7. **`make_test_submission` (`http.rs:5631`) est PARTAGÉE** — consommée par les tests coordinator
   migrants (5682/5726/5787/5831/5900/5950/6154) ET par des tests `tasks_api` qui RESTENT
   (`task_result_route_404_then_text_on_completed` @6797, `tasks_list_with_limit` @7116/7118).
   → **MUST promote to `test_support.rs` pub(crate)** + supprimer la copie locale, sinon E0425
   (pattern Phase O). `test_support.rs` n'a AUCUN `make_test_submission`/`make_result_entry`
   pré-existant (0 collision).

## 2. Move-set FINAL — Production → `coordinator_api.rs`

**Slice unique contiguë `http.rs:2217-2516` (bannières S35-B/S36-B/S36-C/S38-A incluses, migrent
verbatim). 4 handlers privés `async fn` → `pub(crate) async fn`. Signatures INCHANGÉES.**

| Handler | Bornes | Route (path inchangé) | Registration |
|---|---|---|---|
| `coordinator_submit_task` | `2221-2305` | `POST /api/v1/tasks/submit` | `http.rs:405` (mono-ligne) |
| `coordinator_submit_result` | `2311-2438` | `POST /api/v1/results/submit` | `http.rs:406` (mono-ligne) |
| `coordinator_get_kudos` | `2444-2484` | `GET /api/v1/kudos/{project_id}` | `http.rs:407` (mono-ligne) |
| `coordinator_verify_chain` | `2490-2516` | `GET /api/v1/kudos/{project_id}/verify` | `http.rs:408-411` (MULTI-LIGNE, handler @410) |

- **DTO** : AUCUN à migrer (types de requête externes, réponses inline — cf. §1.5). Le
  `TaskSubmission` externe (`nexus_coordinator_rs::types`) et `ResultEntry`
  (`nexus_core_rs::task`) ne sont PAS touchés par un move de handler.
- **Helpers** : AUCUN helper local `http.rs` (cf. §1.6). `crate::validator_loop::ResultEvent`
  (`pub enum`, `validator_loop.rs:30`, variante `NewResult` pub, `ResultEventSender` pub type
  `:34`) référencé full-path à `http.rs:2392` → résout depuis `coordinator_api.rs` inchangé,
  0 bump. `crate::http::DaemonHttpState` (`pub struct`, `http.rs:75`) importé dans le nouveau
  module (mirror seed_api/frost_api), reste défini dans `http.rs`.
- **Blocs VERBATIM load-bearing à préserver byte-identique** (fidélité, pas design) :
  - **Guardrail input-avant-dispatch** (`submit_task`) : `default_input_chain().run` `http.rs:2230`
    → `dispatcher::submit_task` `2259` ; 400 `input_rejected` si tripwire. + nudge local-worker
    S76-A avec `sbfb_home` `2271-2277`.
  - **Guardrail output-AVANT-persist D5 / S73-A** (`submit_result`) : `validate_result_pre_guardrail`
    → `default_output_chain().run` `~2341` AVANT `validate_result_post_guardrail` `~2370` ;
    tripwire → `reject_result_on_guardrail_trip` TERMINAL (CARRY-2 S74/S75-G) `~2353-2360`.
  - **Bridge feed** : `state.result_event_tx.send(crate::validator_loop::ResultEvent::NewResult(entry))`
    `http.rs:2390-2392`. **Le dedup S76-D `(worker_pubkey,task_id)` N'EST PAS dans le slice** —
    il vit downstream dans `result_sync.rs` (`forward_result_entry`) + `validator.rs`, INTOUCHÉS
    par un move pur. Le handler ne fait que le `send` ; celui-ci migre verbatim.
  - **Kudos credit sanity-bound** : `kudos_ledger::credit(...)` avec `entry.payload.tokens_generated`
    + `generation_time_ms` `http.rs:2380-2389` (site prod du plafond anti-gonflage — cf. §7 docs).
- **Non-monétaire Day-0** respecté par construction (read/verify/credit-réputation only ;
  0 cost/stake/burn). **Aucun DESIGN-CONFLICT.**

## 3. Move-set FINAL — Tests → `coordinator_api.rs::tests` (discipline PO-10)

**13 `#[tokio::test]` router-driven co-migrent** (cluster contigu `http.rs:5675-6173`, après le
commentaire « Sprint 36 Phase B » `~5627` ; tous pilotés par `build_test_router` + `.oneshot`,
ZÉRO appel direct de handler). Comptés = **13** (attributs vérifiés : slice lignes 49/93/128/154/
198/267/317/354/397/434/473/497/524 = `http.rs:5675/5719/5754/5780/5824/5893/5943/5980/6023/6060/
6099/6123/6150` ; prochain attribut @6175 = domaine canary, EXCLU) :

1. `result_submit_accepts_valid` `5676`
2. `result_submit_rejects_bad_signature` `5720`
3. `result_submit_rejects_unknown_task` `5755`
4. `result_submit_rejects_completed_task` `5781`
5. `submit_result_rejected_by_guardrail_persists_nothing` `5825` (D5 + CARRY-2)
6. `submit_result_accepted_persists_after_guardrail` `5894`
7. `e2e_task_result_kudos_credited` `5944`
8. `kudos_endpoint_returns_json` `5981`
9. `submit_task_returns_500_on_poisoned_mutex` `6024`
10. `submit_result_returns_500_on_poisoned_mutex` `6061`
11. `get_kudos_returns_500_on_poisoned_mutex` `6100`
12. `verify_chain_endpoint_returns_valid` `6124`
13. `submit_task_pii_rejected` `6151`

**Fixtures — 3 helpers, 2 traitements distincts :**
- **`make_test_submission` (`http.rs:5631`) → PROMUE `test_support.rs` pub(crate)** + copie locale
  supprimée (partagée avec les tests `tasks_api` STAY 6797/7116/7118, compiler-forcée E0425 sinon —
  pattern O). `http.rs::tests` fait déjà `use crate::test_support::*` (`http.rs:3048`) → le côté
  STAY résout automatiquement. `coordinator_api.rs::tests` fera `use crate::test_support::*;`.
- **`make_result_entry` (`http.rs:5651`) + `make_result_entry_with_text` (`http.rs:5655`) →
  co-migrent AVEC les tests** dans `coordinator_api.rs::tests` (usages 5687/5731/5758/5798/5837/
  5906/5955 = migrants SEULEMENT). NE PAS sur-promouvoir vers `test_support.rs` (sinon dead-code
  `unused` côté http.rs).
- **Déjà pub(crate) dans `test_support.rs`, réutilisées via le glob** : `mk_state` `:114`
  (fournit `coordinator_db` in-memory, `pow_keypair`, `result_event_tx` = `broadcast(8).0`, rx
  droppé → le `let _ = send` du handler avale le closed-channel, non-fatal), `build_test_router`
  `:103`, `TEST_TOKEN` `:36` (utilisé par `submit_task_pii_rejected` @6163).

**Golden — RESTE, atomique :** `golden_http_coordinator_domain` (`test_support.rs:501-540`,
`#[tokio::test]` @501 / fn @502). 3 cas : `kudos_get` → 200 `{project_id,total:0,contributors:[]}` ;
`kudos_verify_chain` → 200 `{valid:true}` ; `tasks_submit_empty` POST `{}` → 422 texte EXACT
`Failed to deserialize the JSON body into the target type: missing field \`project_id\` at line 1
column 2`. **NE MIGRE PAS** (bloc atomique 9-golden PO-10) ; reste VERT car les paths + le
field-order de `TaskSubmission` (crate externe `nexus_coordinator_rs::types`) sont INTOUCHÉS.

**Tests qui RESTENT (pièges à NE PAS balayer) :**
- `coordinator_health_ok` `http.rs:6947` — nom TROMPEUR ; tape `GET /api/v1/coordinator/health` →
  `crate::health_api::coordinator_health` (`http.rs:508`). Domaine health, PAS coordinator.
- `kudos_entries_empty` `6846` / `kudos_leaderboard_empty` `6866` / `kudos_entries_with_limit_offset`
  `7067` — `/api/v1/kudos/entries` + `/leaderboard` → `crate::kudos_api::{list_entries,leaderboard}`
  (`http.rs:512-515`). Domaine kudos_api. **Piège collision de préfixe** : seuls
  `/api/v1/kudos/{project_id}` et `/{project_id}/verify` sont coordinator.
- `task_result_route_404_then_text_on_completed` `6790` + `tasks_list_with_limit` `7111` — domaine
  `tasks_api` ; consomment `make_test_submission` (raison de sa promotion), STAY.

**Comptes :** exactement **13 tests relocalisés**, tous `#[tokio::test]` async, 0 `#[test]`. Les 3
helpers ne portent pas d'attribut. **Delta ±0** (pure relocation). Golden count inchangé.
Baseline **Win 2108 / Docker 2112**.

## 4. Plan d'imports

**`coordinator_api.rs` — en-tête (mirror `seed_api.rs`/`frost_api.rs`) :**
```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinator Rust-native task/result/kudos loopback HTTP domain — extrait
//! verbatim de `http.rs` (Sprint 82 Phase Q, discipline étendue PO-10 : les
//! tests router-driven du domaine co-migrent ci-dessous via le harness partagé
//! `crate::test_support`). Task submit (S35 B), result submit avec le guardrail
//! output-avant-persist S73-A/D5 + le feed bridge S76-D (S36 B), kudos read
//! (S36 C) et verify_chain (S38 A). Les routes restent enregistrées dans
//! `crate::http::build_router` À L'INTÉRIEUR de `authed_routes` et re-pointent
//! ici par chemin complet ; paths, formes JSON et status codes INCHANGÉS.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};

use crate::http::DaemonHttpState;
```
- **5 lignes `use`** exactement. Justification : les seuls idents NON-qualifiés dans les 4 corps
  sont `State` (extracteur), `StatusCode`, `IntoResponse`, `Json` (réponses inline), `Arc`
  (`State<Arc<DaemonHttpState>>` + `Arc::clone` `2272`), `DaemonHttpState`. **`Path` NON requis**
  (extracteurs `axum::extract::Path<String>` full-path `2446/2492`) ; **`SystemTime` NON requis**
  (`std::time::SystemTime` full-path `2459`) ; **serde derive NON requis** (0 DTO local). Tout le
  reste reste full-path dans les corps (`nexus_coordinator_rs::*`, `nexus_core_rs::task::ResultEntry`,
  `crate::validator_loop::ResultEvent`, `serde_json::*`, `tracing::*`, `tokio::spawn`, `hex::encode`).

**`coordinator_api.rs::tests` — bloc imports (ATTENTION piège c1/c3) :**
```rust
use super::*;
use axum::body::to_bytes;
use axum::http::{Method, Request};
use nexus_core_rs::KeyPair;
use tower::ServiceExt;
use crate::test_support::*;
```
- **`KeyPair` SEUL, PAS `create_node`** : copier verbatim le bloc de `seed_api.rs::tests`
  (`use nexus_core_rs::{KeyPair, create_node};`) tripperait `unused_imports` sous
  `clippy --all-targets -D warnings` (les 13 tests coordinator n'appellent JAMAIS `create_node` ;
  ils obtiennent le node via `mk_state`). `make_result_entry(worker_kp: &KeyPair)` force `KeyPair`
  en scope. (Non-bloquant — le gate `-D warnings` l'attrape — mais pré-empté pour éviter un
  aller-retour de compile.)

**`http.rs` côté départ :** retirer le slice `2217-2516` (blank `2216`/`2517` s'accolent →
collapse 1 blank, cargo fmt gère). AUCUN import `http.rs` ne devient orphelin : les idents des 4
handlers (`State`/`Arc`/`DaemonHttpState`/`IntoResponse`/`Json`/`StatusCode`) sont partagés
workspace-wide par d'autres handlers (C1 vérifié : `SystemTime`@37 + `FsPath`@35 restent utilisés
ailleurs). Retirer aussi le cluster tests `~5627-6173` (13 tests + `make_result_entry`/`_with_text`
→ `coordinator_api.rs::tests` ; `make_test_submission` → `test_support.rs`).

**`main.rs` :** ajouter `mod coordinator_api;` au slot alphabétique **entre `mod contributor_api;`
(`main.rs:35`) et `mod deploy;` (`main.rs:36`)** — mod NORMAL (pas `cfg(test)`, handlers de prod).
Vérifié : `contributor` < `coordinator` < `deploy`.

## 5. Re-points de routes (4 sites, chemin complet `crate::coordinator_api::`)

- `http.rs:405` `post(coordinator_submit_task)` → `post(crate::coordinator_api::coordinator_submit_task)`
- `http.rs:406` `post(coordinator_submit_result)` → `post(crate::coordinator_api::coordinator_submit_result)`
- `http.rs:407` `get(coordinator_get_kudos)` → `get(crate::coordinator_api::coordinator_get_kudos)`
- `http.rs:410` (dans le bloc multi-ligne 408-411) `get(coordinator_verify_chain)` →
  `get(crate::coordinator_api::coordinator_verify_chain)` ; path `409` + wrappers `408/411`
  INCHANGÉS.

**Aucun autre re-point de code.** Les 4 symboles sont référencés UNIQUEMENT à ces sites route +
leurs défs (grep crate-wide C1/INV-3 confirmé : 0 ref hors `http.rs`, les tests sont router-driven
par URI-string, jamais par nom de fn). `pub(crate)` suffit ; seuls `http.rs` + `main.rs`
(+ `test_support.rs` pour la fixture + `coordinator_api.rs` neuf) sont édités.

## 6. Couplages d'état (tous déjà `pub` sur `DaemonHttpState` — 0 édit du struct)

- `coordinator_db` (`http.rs:140`, `pub`, `Arc<Mutex<CoordinatorDb>>`) — les 4 handlers
  (2247/2315/2448/2494).
- `pow_keypair` (`:130`, `pub`) — `submit_task` (2258).
- `task_dispatch_tx` (`:160`, `pub`, `Option<...>`) — `submit_task` (2261).
- `project_doc` (`:156`, `pub`) — `submit_task` (2271, nudge local-worker).
- `local_worker` (`:165`, `pub`) — `submit_task` (2272).
- `sbfb_home` (`:149`, `pub`) — `submit_task` (2276, S76-A).
- `result_event_tx` (`:141`, `pub`, `ResultEventSender`) — `submit_result` (2390-2392).

Tous accédés via l'extracteur `State<Arc<DaemonHttpState>>` déplacé. **Aucune promotion de
visibilité de champ, aucun édit de `DaemonHttpState` (reste dans `http.rs`).**

## 7. S1a/S1b/S2/S3/S4 — synthèse des scans (tous EXECUTE / no-conflict)

- **S1a (OSS / axum)** : axum pinné **0.8.9** (`Cargo.toml:170` `features=["ws"]`, résolu
  `Cargo.lock:521` ; le commentaire `axum 0.7` de `crates/nexus-shell-daemon/Cargo.toml:40` est
  STALE — dette doc pré-existante, hors move, → lot doc Phase T). Pattern handler-par-module +
  re-point full-path + harness `test_support` = idiomatique axum 0.8, prouvé 3× (N/O/P). Seul
  piège écosystème documenté = l'erreur opaque `Handler`-non-satisfait si un extracteur/State ne
  résout pas identiquement dans le nouveau module (docs.rs/axum/handler ; tokio-rs/axum#3556 ;
  remède `#[debug_handler]`) — neutralisé par le plan d'imports §4 (mirror seed_api, signatures
  verbatim). **Multi-verb routing + per-route middleware/`from_fn_with_state` = STRUCTURELLEMENT
  INAPPLICABLES** (0 route multi-verbe ; les routes coordinator ne portent aucun `.layer` ; auth
  `from_fn_with_state` `http.rs:562` + cors `:569` enveloppent `authed_routes` et restent dans
  `http.rs`). Ordre des extracteurs (State FromRequestParts d'abord, Json FromRequest dernier)
  satisfait verbatim.
- **S1b (deps/CVE)** : move INTRA-CRATE → **0 delta `Cargo.toml`, 0 delta `Cargo.lock`, 0 delta
  feature**. Toutes les crates consommées déjà déclarées directes (`axum` `:42`, `serde_json`
  `:59`, `hex` `:66`, `tracing` `:46`, `tokio` `:34`, `nexus-coordinator-rs` `:30`,
  `nexus-core-rs` `:29`). `git show` sur N/N2/O/P `-- '*Cargo.toml' '*Cargo.lock'` = VIDE (pattern
  0-delta prouvé). **0 delta RUSTSEC** : `cargo deny check advisories bans` EXIT 0 ; 2 ignores
  quick-xml transitifs iroh (`deny.toml:84-85`) hors périmètre ; 4 ignores hickory déjà retirés
  Phase K. `multiple-versions='warn'` inchangé.
- **S2 (décisions historiques)** : NO DESIGN-CONFLICT. Chaque invariant du slice migre par
  construction — S73-A guardrail-avant-persist (D5, commit `6f5ff30`), CARRY-2 terminal tripwire,
  input-guardrail-avant-dispatch + nudge S76-A, feed `result_event_tx`. Le dedup S76-D (commit
  `d75ae77`) N'EST PAS dans le slice (`result_sync.rs`, intouché). PO-14 « un seul Done » +
  primitive result S72-D : le côté READ (`GET .../result` → `tasks_api::get_task_result`) est
  DÉJÀ splitté, hors move ; seul le côté persist migre. Kudos non-monétaire respecté. Frontières
  read déjà splittées (`tasks_api`/`kudos_api` `http.rs:512-533`) — NE PAS capturer/orpheliner.
- **S3 (threat-model)** : surface INCHANGÉE. Les 4 routes restent **T0** (loopback authed :
  bearer X-SBFB-Token + Host + Origin, `auth_required` `http.rs:562`) ; le move garde paths +
  registration dans `authed_routes`, ne change que la cible du handler → 0 tier, 0 surface.
  Invariants sécurité (guardrail output-avant-persist terminal + feed bridge + credit
  sanity-bound) migrent verbatim intacts. LOOPBACK §3 ne liste explicitement que
  `POST /api/v1/tasks/submit` (les 3 autres implicitement T0) — gap PRÉ-EXISTANT, hors scope Q.
- **S4 (wire-format)** : **0 wire bump PROUVÉ**. 0 DTO local migre (types requête externes,
  réponses inline) → aucun struct serde relocalisé, aucun field-order ne peut bouger ; le seul
  invariant golden (`TaskSubmission.project_id` premier, `test_support.rs:533-536` vs
  `nexus-coordinator-rs/src/types.rs:73`) préservé par construction. Aucun `*_VERSION`/`canonical`/
  `JCS` dans le slice. 4 paths byte-identiques (épinglés par le golden). Pré-launch policy : 0 bump
  requis et aucun permis.

## 8. Consommateurs et frontière (web/src) — verdict frontier_closure

- **web/src** : `web/src/api/coordinator.ts` — `submitTask()` L476 → `POST /api/v1/tasks/submit`
  (Zod `SubmitTaskResponseSchema`) ; `submitComputeTask()` L681-692 → même route (Zod
  `SubmitComputeTaskResponseSchema`, consommé `useBridge.ts:252`) ; `verifyKudos()` L490-493 →
  `GET /api/v1/kudos/{project_id}/verify` (Zod `KudosVerifySchema`, consommé `ProjectDetail.tsx:99`).
  **TOUS couplés par CHEMIN de route (inchangé) + schéma Zod reflétant des formes JSON produites
  HORS `http.rs`** (`TaskEntry` de `nexus_coordinator_rs`, `{valid:bool}` inline). **Faux ami** :
  le tableau kudos du front passe par `listKudos` → `/api/v1/kudos/entries` (`kudos_api::list_entries`),
  route VOISINE hors move-set.
- **VERDICT FRONTIÈRE** : **move pur ⇒ 0 signature DTO touchée ⇒ `frontier_closure` N/A + 0 index
  Phase T**. Aucune DTO locale ; consommateurs web path-based ; schémas Zod reflètent des formes
  produites hors `http.rs`. Rien côté web à indexer. (Toute signature touchée BASCULERAIT ce
  verdict en obligation Phase T — d'où la vigilance : garder les signatures byte-identiques.)

## 9. Docs-contrat et gates CI

**Refs docs file-ancrées à RÉ-HONNÊTER in-phase (PO-10, précédent Phase O ; NON-gating,
doc-honnêteté) :**
- **`crates/nexus-coordinator-rs/src/validator.rs:404`** — `guardrail in between (\`http.rs\`
  \`coordinator_submit_result\`,` → devient FAUX (symbole part vers `coordinator_api.rs`). **Le seul
  vrai ref file:symbole stale.** Re-pointer `\`http.rs\`` → `\`coordinator_api.rs\`` (laisser
  `\`validator_loop\`` intact, il ne migre pas).
- **`validator.rs:421`** — `concern covered in \`http.rs\` / \`validator_loop.rs\` tests.` et
  **`validator.rs:448`** — `exercised in \`http.rs\` / \`validator_loop.rs\`;` : les tests guardrail
  du domaine (`submit_result_rejected_by_guardrail_persists_nothing` @5825,
  `submit_result_accepted_persists_after_guardrail` @5894) co-migrent vers `coordinator_api.rs`,
  donc le token `\`http.rs\`` de test-localisation devient imprécis → re-pointer vers
  `\`coordinator_api.rs\``. LOW (file-level, pas symbole).
- **`docs/rust/PATTERNS.md:3578`** (§P61.2) — `The two prod sites (\`validator_loop.rs\`,
  \`http.rs\`) just forward` : le site credit `http.rs` vit dans `coordinator_submit_result`
  (`http.rs:2380-2389`) qui migre → `\`http.rs\`` devient imprécis. LOW-MED (file-level, non-gated).
- **`docs/security/THREAT_MODEL.md:1117`** — `2 sites prod \`validator_loop.rs\` + \`http.rs\`` (kudos
  sanity-bound) : même site credit migrant → `\`http.rs\`` imprécis. LOW-MED.

**Refs SYMBOLE-SEULES qui RESTENT VALIDES (0 re-point — le move préserve le NOM du symbole) :**
`validator.rs:39` + `validator.rs:178` (`HTTP \`coordinator_submit_result\``, sans chemin) ;
`docs/rust/PATTERNS.md:3390` (§P59.4, sans chemin) ; `docs/security/THREAT_MODEL.md:938` (sans
chemin).

**NE JAMAIS toucher :** `docs/claude/SPRINT_LOG.md:26` (narration historique immuable S73).

**Gates CI — AUCUN NE CASSE (C2 CLEAN, vérifié) :**
- Aucun `check-*-docs.sh` ne grep un symbole/route coordinator. `check-sharding-docs.sh`
  source-ref-check (scope `WIRING_SPEC.md` + `llms.txt`) : le seul token `http.rs` y est
  `http.rs:authed_routes` (`WIRING_SPEC.md:144/165`), symbole qui **RESTE** dans `http.rs` (le move
  ne renomme NI `build_router` NI `authed_routes`) → gate verte, mais un rename accidentel la
  casserait. `check-frontier-contracts.sh` (FRONTIER-tags + `DOMAIN_*_V1` + `BLOB_SERVE_CSP` +
  prompt-kind blake3) : le slice coordinator ne contient aucun de ces motifs. `check-factory-docs.sh`
  ne grep aucun symbole coordinator. Aucun ref numérique `http.rs:NNNN` gaté (la suppression de
  ~300 l ne peut pas tripper un « line out of range »). Scripts d'acceptance
  (`b3_live_pc_vps.sh:375`, `phase_h_compute_local.sh:5/35`) tapent le CHEMIN `/api/v1/tasks/submit`
  (inchangé), et ne sont pas dans la liste de steps CI.
- **Obligations de construction (à SATISFAIRE, pas des casses)** :
  (a) **SPDX** — `coordinator_api.rs` DOIT commencer ligne 1 par `// SPDX-License-Identifier:
  AGPL-3.0-or-later` (`check-spdx.sh`, triple surface : `.woodpecker/ci-linux.yml:82`,
  `.github/workflows/ci.yml:128`, `verify.sh:92`). (b) **phase-review-cross-check.yml** — le commit
  `refactor(daemon): Sprint 82 Phase Q ...` exige `sprint82_phase_q_review.md` (gate process
  orthogonale, satisfaite par le workflow normal de phase).

## 10. Hazards compile et plan d'exécution en tranches

**Hazards compile (tous pré-empbés, aucun bloquant) :**
1. **E0425 `make_test_submission`** si co-migrée au lieu de promue → **promouvoir vers
   `test_support.rs` pub(crate) + supprimer la copie locale** (§3). Compiler-forcé par les tests
   `tasks_api` STAY (6797/7116/7118).
2. **`unused_imports create_node`** si on copie le bloc test de seed_api → **`KeyPair` seul** (§4).
3. **Erreur opaque `Handler`** si les extracteurs ne résolvent pas identiquement → plan d'imports §4
   (mirror seed_api, signatures verbatim) ; diagnostic `#[debug_handler]` si build rouge, PAS
   redesign (S1a).
4. **Route multi-ligne `verify_chain`** → re-pointer le handler @410 uniquement (§5).
5. **Ne PAS renommer `build_router`/`authed_routes`** (sinon casse `check-sharding-docs.sh` via
   `WIRING_SPEC.md:144/165`).

**Plan d'exécution en tranches (ordre suggéré) :**
1. Créer `coordinator_api.rs` : en-tête SPDX + `//!` + 5 `use` (§4) ; coller les 4 handlers
   verbatim `http.rs:2221-2516` (bannières incluses), passer `async fn` → `pub(crate) async fn`.
2. Ajouter `#[cfg(test)] mod tests` : coller les 13 tests + `make_result_entry`/`_with_text`
   verbatim ; bloc imports test (§4, `KeyPair` seul).
3. Promouvoir `make_test_submission` → `test_support.rs` pub(crate) ; supprimer la copie locale
   `http.rs`.
4. Retirer de `http.rs` le slice prod `2217-2516` + le cluster tests migrés (`~5627-6173` sauf ce
   qui n'est pas coordinator) ; collapse blank.
5. Re-pointer les 4 routes (§5) ; ajouter `mod coordinator_api;` à `main.rs:36` (slot alpha).
6. Ré-honnêter in-phase les 5 refs docs file-ancrées (§9) ; laisser les symbole-seules + SPRINT_LOG.
7. Oracle T1 (§11).

## 11. Vérification adversariale — synthèse

- **7 claim-checks (handlers + doc-refs)** : les 4 handlers `claim_ok=true` (bornes/routes/absence
  de call-site externe/move-set complets, exacts). Les doc-refs `claim_ok=true` mais **audit
  INCOMPLET corrigé** : l'inventaire initial ne consignait que `validator.rs:404` ; les
  claim-checks ont récupéré `validator.rs:39/178` (symbole-seules, restent valides),
  `validator.rs:421/448` (file-level test-loc, migrent), `PATTERNS.md:3578` + `THREAT_MODEL.md:1117`
  (file-level credit-site, migrent) — tous intégrés §9.
- **C1 (compile/coupling) = CORRECTIONS** : (a) REFUTÉ le « registered in build_router, PAS
  authed_routes » d'INV-3 → routes sur `authed_routes` `http.rs:282` (doc-comment §4 corrigé) ;
  (b) MISSED → le bloc imports test doit être `KeyPair` seul (pas `{KeyPair, create_node}`).
  Production side jugée SOUND et exécutable ; slice contiguë propre, 5-line import set correct et
  complet, 7 champs state déjà pub, 0 helper local, symboles référencés uniquement aux routes.
- **C2 (docs/gates) = CLEAN** : « aucun gate ne casse » SURVIT au pass adversarial. INV-3
  `ci_gates_at_risk:[]` CORRECT. 2 obligations standing (SPDX triple-surface, phase-review
  cross-check) consignées §9.
- **C3 (tests) = CLEAN** : 0 orphelin (13 tests = 13 URIs de route, mapping 1:1, 0 URI dynamique) ;
  count ±0 ; promotion `make_test_submission` correcte + `make_result_entry`/`_with_text`
  co-migrants corrects ; golden unique STAY vert ; 0 test duress ; 0 test d'intégration sous
  `tests/` référençant coordinator. Seul finding = la même nuance import `KeyPair`-seul (LOW).

## Verdict: PLAN-ADAPT

Approche du plan **CONFIRMÉE** — split du domaine coordinator vers `coordinator_api.rs`, routes
inchangées (T0 authed), golden-gardé, 0 wire bump, 0 dep, 0 Cargo delta. **0 DESIGN-CONFLICT**
(S1a/S1b/S2/S3/S4 unanimes EXECUTE ; non-monétaire + guardrail D5 + PO-14 + iroh + pré-launch tous
préservés par construction ; C2/C3 CLEAN).

**Adaptations matérielles requises (au-delà de simples précisions factuelles) — ce qui distingue Q
de la Phase P (EXECUTE, aucune fixture partagée, 0 helper carry) et l'aligne sur la classe
Phase O (PLAN-ADAPT) :**
1. **Promotion `make_test_submission` → `test_support.rs` pub(crate)** + suppression de la copie
   locale (compiler-forcée E0425 par les tests `tasks_api` STAY — un implémenteur naïf suivant
   « co-migrate tests » heurte le mur ; adaptation porteuse de classe O, édite un 3ᵉ fichier
   `test_support.rs` que le plan ne spécifie pas).
2. **Ré-honnêteté in-phase de 5 refs docs file-ancrées** (`validator.rs:404/421/448`,
   `PATTERNS.md:3578`, `THREAT_MODEL.md:1117`) — livrable additionnel type docs-contrat (PO-10,
   précédent O), les symbole-seules restant intactes.
3. **Arbitrage de contradiction C1** (INV-3 `build_router` vs S1a `authed_routes`) tranché sur
   disque (`http.rs:282`) → formulation du doc-comment du module.

Tout le reste est P-class (bornes re-dérivées par NOM, slice contiguë propre, 0 DTO local,
0 helper carry, 0 bump `pub(crate)` de symbole `http.rs`, 0 re-point `runtime.rs`, 0 décision de
périmètre). Le code suit l'approche corrigée ci-dessus (§2-§10).
