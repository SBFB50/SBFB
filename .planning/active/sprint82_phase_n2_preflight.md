# Sprint 82 Phase N2 — Preflight G8 (extraction harness de test partagé → `test_support.rs`, PO-10)

## Contexte + méthode (Workflow multi-agents, 2026-07-16)

- HEAD au preflight : `9ea7c05` (Phase N `2e87eef` + amendement PO-10). Phases A→N DONE (14/24).
- Orchestration : Workflow ultracode `wf_5a5ba997-889` — **51 agents** (5 scans G8 + cartographie
  gather-move, réfuteur adversarial par claim, critic final). 0 agent en erreur.
- Mission (plan §Phase N2, amendement PO-10) : GATHER-move du harness de test éparpillé dans
  `http.rs::tests` (12663 l, mod tests :4211-12663) vers
  `crates/nexus-shell-daemon/src/test_support.rs` (`#[cfg(test)] mod test_support;`,
  items pub(crate)) + migration du test duress shard-session (D-N3). Ce preflight devait
  trancher l'emplacement de la famille golden — TRANCHÉ : elle MIGRE (voir Décision).

## Dérives plan à consigner (vérifiées disque, HEAD `9ea7c05`)

1. **Move-set du plan INCOMPLET — 3 items omis** (dépendances transitives) :
   `AUTH_HEADER_NAME` (:4299, 3 occurrences, consommé par `build_test_router_ext`
   :4343/:4344), `enum TestHeaders` (:4307, paramètre de `_ext` + 3 wrappers),
   `GOLDEN_REDACTED` (:12212, consommé par `golden_redact` :4222). Tous trois DOIVENT
   physiquement migrer avec le harness.
2. **Narratif de supersession à corriger** : PO-10 supersede EXPLICITEMENT seulement **D-N3**
   (migration duress, « le verrou », plan :366-368). **D-N2 n'est PAS supersedé par PO-10** —
   le plan (:363-365) DÉLÈGUE la relocalisation golden à ce preflight et RÉAFFIRME l'invariant
   anti-fragmentation. La décision MOVE ci-dessous est donc une décision de design du
   preflight (jugement argumenté), pas un fait accompli PO-10.
3. **Slot main.rs corrigé** : `#[cfg(test)] mod test_support;` va entre `mod tasks_api;` (:60)
   et `#[cfg(unix)] mod uds_server;` (:61-62) — ordre lexical storage < tasks < test < uds
   (le scout disait entre storage_api et tasks_api : FAUX).
4. Coords §Récap du plan (12460 l / région tests 4546-12460) STALES — mod tests réel
   :4211-12663 au HEAD.
5. `to_bytes` = `axum::body::to_bytes` (pas http_body_util).

## Décision de design — la famille golden MIGRE (bloc atomique)

**MOVE** : bannière (:12195-12203) + infra (`GOLDEN_VOLATILE_FIELDS` :12205-12210,
`GOLDEN_REDACTED` :12212, `golden_redact` :12214-12235, `GoldenBody` :12237-12242,
`GoldenCase` :12244-12257, `golden_check` :12259-12302, `golden_run` :12304-12311) + les
**9 tests `golden_http_*`** (:12313-12662) partent EN UN BLOC vers `test_support.rs`.
Leviers (jugement argumenté, consigné honnêtement comme non-forcé par l'évidence — les deux
options satisfont l'invariant anti-fragmentation et l'observateur reste stable puisque
`build_router` reste dans http.rs) :
1. **Isolation du filet** : le golden net est LE filet des phases O→S4 qui vont lourdement
   éditer http.rs::tests — un filet ne doit pas vivre dans le fichier qu'il protège.
2. **Cible PO-10 ≤2500 l TOTAL** : le bloc golden = 468 l caractérisant des domaines qui
   auront QUITTÉ http.rs après O→S4 — le garder consommerait ~1/5 du budget final.
3. **Couplage harness maximal + transversalité** : golden est le seul bloc consommant les
   3 constructeurs (build_test_router :12307, cors :12594, web_root :12635) et couvre 8
   domaines — aucun `_api.rs` de domaine ne peut l'accueillir ; son seul foyer atomique
   hors http.rs est test_support.rs.
L'infra golden peut rester PRIVÉE dans test_support (seul le harness routeur/état a besoin
de pub(crate)) ; les 9 tests gardent `#[tokio::test]`.

## Move-set COMPLET (bornes exactes au HEAD `9ea7c05`)

Vers `test_support.rs` (dédenté 4 espaces, mod tests → top-level) :
- Cluster 1 : **:4289-4463** — TEST_TOKEN (4289-4294 doc incluse), AUTH_HEADER_NAME
  (4296-4300), TestHeaders (4302-4310), build_test_router_ext (4312-4354 ; conserver le
  `use` fn-local HeaderValue/HOST/ORIGIN :4330-4331 + `axum::extract::Request` inline :4341
  + le commentaire-garde :4321-4323 « Nothing else may be added here » [le test
  feed_insert_rejects_without_internal_header :7506 exige que le harness n'ajoute PAS
  x-sbfb-feed-internal]), build_test_router (4356-4362), mk_state (4364-4373),
  mk_state_with_sbfb_home (4375-4379), mk_state_with_mode (4381-4383), mk_state_with_mode_tx
  (4385-4463, littéral DaemonHttpState 36 champs inclus).
- build_cors_test_router : **:7224-7228**.
- build_test_router_with_web_root : **:8128-8143** (la bannière SPA :8124-8126 RESTE avec
  les tests SPA qui restent).
- Famille golden : **:12195-12662** (bannière + infra + 9 tests).

Vers `shard_session_http_api.rs::tests` :
- `shard_session_routes_noop_in_duress` : **:6250-6341** VERBATIM + imports additionnels
  (`crate::test_support::{build_test_router, mk_state_with_mode}` ou glob,
  `axum::body::to_bytes`, `axum::http::{Method, Request}`, `tower::ServiceExt` pour
  `.oneshot` ; Arc + StatusCode déjà via `super::*`).

RESTENT dans http.rs::tests (helpers test-local NON-harness) : `own_browse_entry` (:4465),
le builder inline de `default_curators_returns_configured_list` (:7395-7419 — SEUL autre
site de construction BlobServeCache/BrowseAggregator/CuratorRuntime : c'est LUI qui garde
vivants les imports :4217-4219, NE PAS le co-migrer), `BrowseListResponse` (:748 cfg(test)).

## Câblage + hygiène d'imports (les vrais risques, clippy -D warnings)

- main.rs : `#[cfg(test)] mod test_support;` entre :60 et :61. Précédent prouvé dans CE
  binaire : `#[cfg(test)] mod handler_tests` (main.rs:1193-1194, résout `super::handle_init`
  :1206) + résolution frère out-of-line prouvée par les 32 `mod X;`.
- http.rs::tests : ajouter `use crate::test_support::*;` juste après `use super::*;`
  (:4213) — le glob garde les ~180+ call-sites bare-name INTACTS (0 édit par call-site).
  Le bloc use existant (:4214-4220) reste INCHANGÉ (0 unused_imports grâce à
  default_curators qui reste).
- Bloc use de test_support.rs (minimal clippy-clean) : Arc ; SystemTime ; axum::Router ;
  axum::middleware ; axum::http::{Method, Request, StatusCode} [StatusCode requis par les
  goldens cors :12607 + spa :12646 — l'omettre = E0433] ; axum::body::to_bytes ;
  tower::ServiceExt ; tokio::sync::RwLock ; nexus_core_rs::{KeyPair, create_node,
  PowSolveCache} ; nexus_shell_daemon_core::{auth::AuthState, blob_serve::BlobServeCache,
  browse::BrowseAggregator, iroh_runtime::CuratorRuntime} ; crate::http::{DaemonHttpState,
  build_router} (+ compléments dictés par le compilateur, en liste explicite).
  NE PAS importer module-scope : axum::body::Body (usage fully-qualified :12264/:12266 →
  unused), axum::extract::Request (collision E0252 avec http::Request ; usage inline :4341),
  HeaderValue/HOST/ORIGIN (restent fn-local :4330-4331).
- Visibilité : 0 escalade côté PRODUCTION (DaemonHttpState pub + 36 champs pub, build_router
  pub :246, GossipCmdTx pub runtime.rs:1670, panic::* pub, shard_session::* pub). Items
  harness → pub(crate) ; NB si `build_test_router_ext` est pub(crate), `TestHeaders` DOIT
  l'être aussi (type dans l'interface). Infra golden privée.

## S1b/S2/S3/S4 — synthèse

- **S1b deps** : 0 dep, 0 feature, 0 delta Cargo.toml/lock (les deux options golden sont
  dependency-neutres). 0 unused_imports côté http.rs::tests (default_curators garde tout).
- **S2 décisions** : body M `29a9255` = promesses TestHeaders Inject/Raw ENSEMBLE + garde
  feed-internal à préserver verbatim ; précédent S81-C « Voie 2 » = pattern test_support
  connu/différé, aucun refus historique ; N2 AVANT O ⇒ move-une-fois = observateur stable
  pour tout O→S4.
- **S3 threat model** : sécurité-neutre par construction. TEST_TOKEN jamais compilé hors
  cfg(test). Découverte clé : les VRAIS tests d'auth négative (401/403 token/Host/Origin)
  vivent dans nexus-shell-daemon-core/src/auth.rs::tests (:886/:898/:939/:952/:966),
  INSENSIBLES au move ; dans http.rs seuls feed_insert 403 (:7506) et cors-reject (:7250)
  dépendent de la POSTURE (jamais du lieu). Le duress migré reste sous authed_routes
  (auth_required câblé :538).
- **S4 wire/docs** : 0 surface wire ; **0 docs-contrat à re-pointer** (grep complet : seuls
  SPRINT_LOG:67 [historique S46 immuable] et llms.txt:37 [symboles PRODUCTION non-mobiles] ;
  check-sharding-docs/frontier ne grep-ent aucun symbole du harness). Homonymes hors-crate
  (registry.rs mk_state, operator_server.rs TEST_TOKEN) sans lien.

## Vérification adversariale — synthèse

51 agents ; 5 claims REFUTED corrigés à la source dans ce document : (1) supersession
D-N2 : seule D-N3 est supersedée ; (2) « évidence tranche le MOVE » → jugement argumenté
non-forcé (consigné) ; (3) bloc use minimal : StatusCode manquait dans une version
(contradiction S1b vs S5 résolue pour S1b) ; (4) golden-placement dependency-neutre ;
(5) slot main.rs corrigé. + raffinement : le précédent handler_tests est un module INLINE —
le mécanisme fichier-out-of-line est prouvé par la COMBINAISON (32 mod out-of-line + 1
cfg(test) crate-root), inference saine.

## Oracle T1 précis

- `cargo fmt --all --check` 0 ; `cargo clippy --workspace --all-targets --locked --
  -D warnings` 0 (piège dominant = hygiène d'imports des DEUX côtés).
- `cargo nextest run -p nexus-shell-daemon -E 'test(golden_http_)'` : **9/9**.
- Duress : 1 test listé sous `shard_session_http_api::tests::` (plus sous http::tests).
- `cargo nextest run --workspace --locked` : **== 2108 Win EXACT** (gather-move
  count-neutre : 0 test ajouté/supprimé, duress change seulement de path).
- Docker sbfb-ci **mount `/workspace`** : == 2112 EXACT.
- Doctests + gates docs (sharding/frontier) inchangés verts ; 0 delta Cargo.lock.
- T2 = N-A.

## Verdict: PLAN-ADAPT

Le geste est EXECUTE-shaped (test-only, 0 wire/route/DTO/dep, count-neutre, 0 escalade de
visibilité prod, décision golden explicitement déléguée au preflight — donc pas une
déviation) mais PLAN-ADAPT est requis : le move-set du plan est INCOMPLET (3 items omis :
AUTH_HEADER_NAME, TestHeaders, GOLDEN_REDACTED), 2 faits de narratif à préciser (D-N2 non
supersedé — décision prise ICI ; bloc use exact avec StatusCode), slot main.rs corrigé.
Décision design : famille golden MIGRE en bloc atomique avec le harness. 0 DESIGN-CONFLICT
(aucune décision Day-0 violée), 0 SCOPE-CUT. Le code suit l'approche corrigée ci-dessus.
