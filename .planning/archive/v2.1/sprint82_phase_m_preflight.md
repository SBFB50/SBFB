# Sprint 82 Phase M — Preflight G8 (golden de caractérisation HTTP + dédup harness `build_test_router*`)

## Contexte + méthode (Workflow multi-agents, 2026-07-15)

Phase M est la **FONDATION du split `http.rs`** (D3) : (1) établir un test de
**caractérisation/golden** qui verrouille l'identité pre/post des réponses HTTP sur les surfaces
à extraire, AVANT tout déplacement de domaine ; (2) consolider les constructeurs de routeur de
test dupliqués (`build_test_router*`) en un seul constructeur paramétré. Cas B (pre-code).
Couvre `REFACTO-HTTP-TEST-HARNESS-DEDUP`, `S82-TEST-REFACTO-NONREG`. **Refacto test-only** :
0 route path, 0 wire bump, 0 dep runtime, `build_router` (prod) byte-identique. Le golden est le
filet des splits **N→S** (6 domaines : N shard-session, O seed, P frost, Q coordinator,
R curators, S publish). Baseline vivante post-Phase L : **Win 2099 / Docker 2103 / Vitest 412 /
operator 201**.

**Six scans factuels** (S1a-facts état-code, S1a-OSS prior-art/lib, S1b-deps/CVE, S2 décisions
historiques, S3 threat-model/postures d'auth, S4 wire) ont été produits puis **vérifiés
adversarialement** (lentilles FACTS/DRIFT, SECURITY, TESTABILITÉ). Le WRITER a **re-vérifié à la
source disque** au HEAD `013b611` chaque coordonnée porteuse : les 4 constructeurs, la signature
`build_router`, la couche d'injection vs raw, le retour `(Router, TempDir)`, la taille fichier,
le count de call-sites, les champs volatiles de `mk_state`, et un échantillon de handlers de
domaine (frost/coordinator).

**Bilan** : le plan §Phase M est **réalisable sans toucher aucune décision Day-0 ni aucun
invariant de sécurité** (auth reste dans le crate CORE, aucune posture d'auth altérée, 0 dep
runtime, golden 100 % `#[cfg(test)]`). Mais **5 faits du plan sont corrigés** (coordonnées
stales, count de constructeurs, axes de paramétrisation, oracle de count, contiguité des
domaines) qui changent l'approche. → **PLAN-ADAPT**.

**Réconciliation adversariale** : 0 claim porteur REFUTED sur les 6 paires (les vérificateurs ont
confirmé 100 % des coordonnées re-dérivées par NOM). Deux caveats UNVERIFIABLE (counts nextest
non re-mesurables en lecture seule ; URLs OSS externes non-fetchées) sont non-porteurs. **Une
erreur de VÉRIFICATEUR renversée** : la note S2-VERIFY qualifiait `coordinator_verify_chain` de
« hallucination de scan » (« pas de route verify_chain ») — FAUX : le handler existe
(`http.rs:3973`) et est câblé `GET /api/v1/kudos/{project_id}/verify` (`http.rs:376-377`) ; le
scan S1a/S2 (« coordinator = 4 handlers ») avait raison, la note du vérificateur est l'erreur.

---

## Dérives plan à consigner (vérifiées disque, HEAD `013b611`)

1. **Constructeurs** : plan §M l.308 dit `4+ build_test_router* (4645/4649/7534/8380)`. Réel =
   **3 fns littéralement `build_test_router*`** (`build_test_router` :4627, `build_test_router_with_cors`
   :4631, `build_test_router_with_web_root` :8563) **+ un 4ᵉ constructeur au préfixe DIFFÉRENT**
   (`build_cors_test_router` :7661) qu'un grep `build_test_router*` **MANQUE**. Les 4 coordonnées
   du plan sont fausses (dérive −18/−18 pour le 1ᵉʳ couple ; `7534`/`8380` ne pointent aucun
   constructeur : `7534`=commentaire, `8380`=`.method(Method::GET)` dans un test).
2. **Taille fichier** : plan (Phase M kickoff) dit `~12460 l` ; réel = **12643 l** (`wc -l`).
   Croissance +183 l depuis le kickoff (phases A/B/D/G ont touché `http.rs`, PAS L). Corollaire :
   toute coordonnée `http.rs:NNN` du plan est à re-dériver par NOM.
3. **Axes de paramétrisation** : plan l.308 dit `cors/web_root en options` (2 axes). Réel = **3ᵉ
   axe porteur** = la **couche d'injection de headers** (inject `x-sbfb-token` + `Host 127.0.0.1:0`,
   puis `h.remove(ORIGIN)`) PRÉSENTE sur `build_test_router`/`_with_cors`/`_with_web_root`, ABSENTE
   sur `build_cors_test_router` (raw). De plus `build_test_router_with_web_root` retourne
   `(Router, TempDir)`, pas `Router` nu. Un constructeur à 2 params ne peut exprimer ni la posture
   raw (tests CORS) ni la possession du `TempDir` (fallback SPA).
4. **Oracle de count** : plan l.312 dit `count nextest >= baseline` + baseline stale `2095/2099`.
   Réel = baseline vivante **Win 2099 / Docker 2103**, et Phase M **AJOUTE** des tests golden →
   l'oracle doit être **`== baseline + N_golden` EXACT** (leçon Phase L : `>=` masque une perte de
   couverture nette). `N_golden` explicité dans le commit body.
5. **Contiguité des domaines N→S** : plan l.318 « 1 domaine = 1 phase » implique des blocs
   contigus proprement extractibles. Réel = **publish (S) est SCATTERED** : `publish_project`
   (:1159) + `publish_directory` (:1255) forment une paire, mais `publish_blob` (:3261) est
   **~2000 l plus loin**, coincé entre `default_curators` (:3245) et `panic_wipe` (:3305). R
   (curators) et O (seed) ont des frontières de helpers partagés à trancher. Non-bloquant pour M
   (M est test-only) mais dimensionne le filet golden (doit couvrir `publish_blob` isolé).

---

## S1a-facts — État réel du code (surfaces à verrouiller)

- **`http.rs` = 12643 l** (`wc -l`). Module de test unique `#[cfg(test)] mod tests` ouvert
  **:4528-4529 → EOF** (~8115 l de test). Un 2ᵉ `#[cfg(test)]` :739 garde une **struct**
  (`BrowseListResponse`, DTO de décode test), PAS un module. `evidence: http.rs:739, :4528-4529, EOF :12643`.
- **4 constructeurs de routeur de test** convergent tous vers l'unique fn prod
  `build_router(state: Arc<DaemonHttpState>, auth: AuthState, cors_origins: &[String], web_root: Option<&FsPath>) -> Router`
  (**:246-251**) :
  - `build_test_router(state)` **:4627** — wrapper zéro-arg délègue à `build_test_router_with_cors(state, &[])` (:4628).
  - `build_test_router_with_cors(state, cors)` **:4631** — `build_router(...).layer(from_fn)` qui
    injecte `x-sbfb-token` + `Host 127.0.0.1:0` et **retire `Origin`** (:4634-4648, `h.remove(ORIGIN)` :4644).
  - `build_cors_test_router(state, cors)` **:7661** — `build_router(...)` **BRUT, sans `.layer`**
    (:7662) : les tests CORS pilotent `Origin` eux-mêmes. **5 call-sites** (7667/7686/7709/7729/7752),
    3 avec origins non-vides. **Seul variant** qui exerce `cors` non-vide ET la posture raw.
  - `build_test_router_with_web_root(state)` **:8563** — `build_router(..., Some(tmp.path()))` +
    **même couche d'injection** (:8578-8590, `h.remove(ORIGIN)` :8587), retourne
    **`(Router, tempfile::TempDir)`** (:8591). **4 call-sites** (8597/8623/8649/8669).
- **`build_test_router(` = 179 tokens** (`grep -c`) = 1 def + **178 call-sites** → toute modif de
  la signature de ce wrapper toucherait 178 sites. `evidence: grep -c build_test_router( = 179`.
- **Paramètre `cors` de `build_test_router_with_cors` = MORT** : unique call-site (:4628) passe
  `&[]`. Toute la vraie couverture CORS non-vide passe par `build_cors_test_router` (raw).
- **Mécanique de test** : `tower::ServiceExt::oneshot` (`use` :4537, idiome unique, ~196-197
  usages) + `Request::builder().method().uri().body(Body::from(serde_json::to_vec))` +
  `axum::body::to_bytes(resp.into_body(), 1<<20)` (`use` :4531) + `serde_json::from_slice`.
  **Aucun hyper client, aucun `axum-test`.** Le golden réutilise ce chemin EXACT, 0 dep.
- **Auth de test** : `const TEST_TOKEN` :4611 (64 hex `deadbeef…`, fixe pour déterminisme),
  `const AUTH_HEADER_NAME` :4616 (`from_static("x-sbfb-token")`). Le vrai `auth_required` +
  `AuthState` sont importés du crate **CORE** (`nexus_shell_daemon_core::auth` :51), câblés :529
  via `from_fn_with_state`. **Un split de domaine ne déplace donc PAS l'auth.**
- **Non-déterminisme** : `mk_state` fixe `daemon_version:"0.1.0-test"` (:4696) et `api_port:12345`
  (:4699), MAIS `node_id: node.node_id()` (:4695, node frais par test via `create_node`),
  `boot_time: SystemTime::now()` (:4697), `pow_keypair: KeyPair::generate()` (:4711) sont
  **volatiles run-à-run**. Tout corps golden échoant ces champs FLAPPERA. `evidence: http.rs:4695-4711`.
- **Domaines contigus (extractibles proprement)** : **N shard-session** 2158-2490 (8 fns,
  routes :315-336) ; **P frost** 3552-3703 (structs locales 3541/3571/3659, routes :365-371) ;
  **Q coordinator** 3704-4004 (`coordinator_submit_task` :3704, `coordinator_submit_result`
  :3794, `coordinator_get_kudos` :3927, **`coordinator_verify_chain` :3973** [câblé
  `/api/v1/kudos/{project_id}/verify` :376-377], routes :372-378). **R curators** 884-1024
  (`list_curators` :884, `subscribe_curator` :896, `unsubscribe_curator` :952) mais suivi de
  helpers browse partagés (`subscribed_catalog_index` :1025, `browse_views` :1049) — frontière.
- **Domaines à frontière** : **O seed** 2491-3244 est bordé par des helpers directory/keep-online
  distincts (`set_keep_online` :1613, `find_directory_app_by_hash` :1702, `directory_pull_providers`
  :1763, `nodes_response` :2080) — concern séparé. **S publish** SCATTERED (cf. dérive #5).

**Aucune infra snapshot/golden** dans le workspace : `insta`/`expect-test`/`assert_snapshot`/`.snap`
= 0 hit sur tout le repo (`*.rs` + `*.toml` + `Cargo.lock`). Le golden est à **hand-roll**.

---

## S1a-OSS — Prior art / bibliothèques (0 dep nécessaire)

- **Stack pinnée** : `axum` 0.8.9 (Cargo.lock:520-521 ; workspace pin `axum = "0.8"` feature `ws`,
  Cargo.toml:170), `tower` 0.5.3 (Cargo.lock:9824-9825 ; pin `tower = "0.5"` sans feature explicite,
  default features ON, Cargo.toml:171), `tower-http` 0.6 (features `cors`+`fs`, Cargo.toml:172).
- **`ServiceExt::oneshot` DÉJÀ disponible et compilant** (feature-unifiée via axum) — l'idiome
  in-tree exact (`use tower::ServiceExt` :4537, ~196-197 usages). Un golden bâti dessus **ne
  requiert AUCUNE dep nouvelle**. Un `axum::Router` implémente `tower::Service` → `oneshot` est la
  méthode axum-maintenue de caractériser un routeur sans serveur live (state appliqué via
  `with_state`, ce que `build_router` fait déjà).
- **`insta` ABSENT** de tout `Cargo.toml` ET de `Cargo.lock` (`^name = "insta"` = 0 hit ; seul un
  substring `install` @Cargo.toml:284). L'adopter serait une **dev-dep NOUVELLE** (+ arbre
  transitif `similar`/`console`). **Non requis** : le golden hand-rollé (canonicalisation
  `serde_json::Value` + masquage) offre la même capacité sans grossir la surface auditée pendant un
  sprint dont Phase K vient de clore 4 RUSTSEC et dont Phase G fait tourner un gate cargo-deny.
- **Canon OSS de stabilisation des snapshots non-déterministes** = redactions (mapping
  sélecteur→placeholder, `dynamic_redaction`) — reproductible à la main via un normaliseur
  `serde_json::Value` (tri récursif des clés + redaction d'une allowlist fixe).
- **Précédent dev-dep** : Sprint 73 Phase B (`a4e1542`) a ajouté `serial_test = "3.4"` comme dev-dep
  workspace NOUVELLE (body : « serial_test = nouvelle dev-dep »). **La politique sprint est
  « 0 dep RUNTIME ajoutée »** — les dev-deps sont une catégorie distincte, permise quand justifiée.
  Donc `insta` **ne violerait pas** la politique 0-dep-runtime ; le hand-roll reste néanmoins le
  choix conservateur (0 surprise arbre + honore le gate supply-chain actif). **Nuance de
  formulation, pas de contrainte** : ne pas dire « interdit par 0-dep » — c'est un arbitrage
  confort/arbre, escaladable PO si l'ergonomie du hand-roll devient pénible sur 6 domaines.

---

## S1b-deps — Deps / CVE (0 dep, hand-roll certifié)

- **`serde_json` = dep RUNTIME DIRECTE** du crate daemon (`serde_json = { workspace = true }`
  Cargo.toml:59 ; 417 usages `serde_json::` dans `http.rs`) → **disponible dans `#[cfg(test)]`**
  pour sérialiser/comparer les corps golden. **0 ajout requis.**
- **`axum::body::to_bytes` + `axum::http::{Method, Request}`** déjà importés dans le module test
  (:4531/:4532 ; 131 call-sites `to_bytes(`). Le golden réutilise l'extraction existante.
- **[dev-dependencies] daemon actuelles** : `nexus-coordinator-rs` (feature `test-support`),
  `serial_test` (workspace :149), `zip` (workspace :151), `tempfile` 3.12 (:155), `libc` 0.2
  (`cfg(unix)` :161). **Aucune lib golden/snapshot.** `evidence: Cargo.toml:144-161`.
- **`insta`/`axum-test`/`pretty_assertions` = ABSENTS** partout (Cargo.toml + Cargo.lock).
- **Conclusion** : NEW dev-dep = **NON**. Hand-roll via `serde_json` + `tower::ServiceExt::oneshot`
  + `axum::body::to_bytes` (tous existants) → satisfait D4 (fmt + clippy + nextest count) avec
  **ZÉRO delta supply-chain**, aligné D3/D4 et la politique 0-dep-runtime.

**Piège de consolidation (compile-backstopped)** : `build_test_router_with_cors` importe
`HeaderValue` localement (:4632) alors que `build_test_router_with_web_root` ne l'importe pas
(seul `use axum::http::header::{HOST, ORIGIN}` :8564). En fusionnant, **hoister l'import
`HeaderValue`** — sinon échec compile (fail-loud).

---

## S2 — Décisions historiques + coordonnées (drift systémique)

- **`http.rs` NON touché par Phase L** ; tip du fichier = `d2705b7` (Phase G). Depuis le kickoff
  S82 (`6fc263b`) : 12460 → 12643 l (**+183 l**), via A/B/D/G. **Toute coordonnée `http.rs:NNN`
  du plan/design-review est mécaniquement stale.** `evidence: git log 6fc263b..HEAD -- http.rs = {A,B,D,G}, pas L`.
- **Leçon Phase L (`013b611`)** : pour un refacto PUR l'oracle a été **resserré à `count ==
  baseline` EXACT** (pas `>=`), car `>=` masque une perte nette. Delta L = **±0 EXACT** (2099/2099,
  2103/2103, 412/412). Phase M **AJOUTE** N_golden → oracle = `== baseline + N_golden` EXACT.
- **Précédent d'extraction** : les modules `*_api.rs` (`tasks_api` S44, `storage_api` S56,
  `kudos_api` S44) sont **NÉS standalone**, aucun extrait de `http.rs`. Les splits N→S seront les
  **premières extractions de domaine hors `http.rs`** — d'où le golden comme filet novateur.
- **Aucune décision Day-0 ne bloque Phase M.** D3 (split incrémental borné, golden AVANT), D4
  (gate d'invariance), politique 0-dep-runtime = toutes honorées par un refacto test-only.
- **Piège procédure (leçon L, memory)** : `| tail` masque l'exit code cargo (2 faux verts en L) →
  **`set -o pipefail`** sur tous les pipelines de gate ; gros jobs cargo en background killed →
  **avant-plan séquentiel**. fmt mass-reformat → gate dual-platform (Win + Docker rust:1.94).

---

## S3 — Threat model : postures d'auth à préserver (EXECUTE-with-guardrails)

- **Modèle loopback UNIFORME T0** : bearer `x-sbfb-token` + Host allowlist + Origin + peer creds
  (UDS SO_PEERCRED / Named Pipe SDDL). Les tiers T1/T2 (CONFIRM_PROMPT/BIOMETRIC) sont
  **design-only** ; les 6 domaines sont T0 en code. `evidence: LOOPBACK_ENDPOINTS_TRUST_TIERS.md:23-27,53`.
- **La consolidation NE DOIT PAS altérer les postures d'auth** :
  - `build_test_router`/`_with_cors`/`_with_web_root` injectent le token+Host et **suppriment
    `Origin`** → chaque requête PASSE `auth_required` ; ces 3 variants **ne peuvent PAS observer**
    401/403 transport. Les tests 401/403/Host/Origin faisant autorité vivent dans le crate CORE
    (`nexus-shell-daemon-core/src/auth.rs:886-981` : missing/wrong token, rebound host,
    cross-origin, https-loopback-origin, ipv6-loopback) — **invariants sous Phase M**.
  - `build_cors_test_router` (:7661) OMET délibérément la couche → `Origin` réel atteint le gate
    CORS. **5 tests CORS en dépendent** (`cors_loopback_*`, `cors_custom_origin_*`,
    `cors_rejects_*` :7666/7685/7707/7727/7750). **Un merge qui injecte toujours la couche efface
    `Origin` avant `cors_layer` et casse ces 5 tests.**
- **La couche d'injection est byte-identique** entre `_with_cors` (4634-4648) et `_with_web_root`
  (8578-8590) — vraie cible de dédup, MAIS elle fait DEUX choses (inject token+Host ET
  `h.remove(ORIGIN)`) : le toggle de consolidation doit **gouverner les DEUX ensemble**.
- **Piège de régression** : `feed_insert_rejects_without_internal_header` (:7941) dépend du harness
  N'AJOUTANT PAS `x-sbfb-feed-internal` (assert 403 sans / 503 avec, :7959-7987). Garder la couche
  d'injection **byte-identique** (token+Host seulement, remove Origin ; **rien d'autre**).
- **Pas de secret dans le golden** : `TEST_TOKEN` est une const hex fixe, aucune route ne l'échoie.
- **Pas d'`EventWriter` à câbler** : `http.rs` n'émet AUCUN `SecurityEvent` (grep = 0 ; seul
  `panic.rs` utilise `nexus_events`, service jamais invoqué en test unitaire). Ajouter un event
  wiring = scope creep sans assertion couvrante.
- **`THREAT_MODEL.md`** (§15.3.1 Sprint 82) ne contraint PAS le harness de test unitaire `http.rs`
  (ses réfs « harness » = scripts d'acceptance). Aucune contrainte liante sur M au-delà de
  préserver les gates.

---

## S4 — Wire format (0-wire PROUVÉ, frontier N/A)

- **13 constantes de version wire dans `nexus-core-rs/src`**, toutes = 1, **AUCUNE atteignable**
  par un refacto `#[cfg(test)]` : `ACTIVATION_COMMIT`/`COMPUTE_GROUP`/`CURATOR_LIST`/`KEY_ROTATION`/
  `NODE_DIRECTORY`/`POW`/`SEED`/`SHARD_PLAN`/`RUN_PROOF`/`TASK`_FORMAT_VERSION, `TASK_RESPONSE`,
  `PIN_FILE`, `BLOB_VERSION`. `FEED_FORMAT_VERSION` vit hors core-rs (`nexus-coordinator-rs/public_feed.rs:20`).
- **Périmètre du diff = `#[cfg(test)]` PUR** : les 4 constructeurs sont dans `mod tests`
  (4529→EOF). **Seule construction prod** = `pub fn build_router` (:246), 5 `Router::new` (255/260/
  270/282/531), consommée en prod uniquement par `runtime.rs:1395` (import :64). La consolidation
  ne peut PAS altérer le binaire prod.
- **Table de routes verrouillée par `build_router` (:256-543)** — 3 tiers : PUBLIC (no auth :
  `/health`, nest `/blob-serve/{hash}/{*path}` + `blob_serve_csp_middleware` :257/578) ; TOKEN
  (Host+Origin seulement : `/auth/token` :270-272) ; **AUTHED** (~86 `.route(` sous `auth_required`
  :282-529, dont multi-méthode `/app/{name}/state/{key}` :394-397 GET/POST/DELETE et body-limité
  `/api/v1/deploy` :435-437) ; FALLBACK SPA (`ServeDir`+`ServeFile`) **uniquement si `web_root` Some**
  (:538-543). `evidence: http.rs:256-543`.
- **`frontier_closure = N/A`** : le fixture golden est lu UNIQUEMENT par les tests Rust de ce même
  crate via `oneshot` in-process (aucun runtime distinct — pas de shell React, pas de worker, pas
  de client externe). **PAS un item docs-contrat Track T.** Aucun DTO serde, aucune signature de
  handler, aucune ligne de routage modifiée → **0 route path, 0 wire bump** (D4, politique
  pre-launch intacte).

**Caveat déterminisme publish (S)** : `publish_blob` retourne un `archive_hash` content-addressed ;
sa stabilité golden dépend de `make_zip` (:7920-7932, `SimpleFileOptions::default()`) dont le
last-modified peut varier selon la version du crate `zip`. **Ne pas présumer « content-addressed ⇒
déterministe »** : confirmer que `make_zip` sort des octets stables, sinon masquer `archive_hash`.

---

## Vérification adversariale — table des claims

| # | Scan | Claim | Verdict | Correction retenue (disque fait foi) |
|---|---|---|---|---|
| 1 | tous | 4 constructeurs @4627/4631/**7661**/8563 ; coords plan 4645/4649/7534/8380 stales | **CONFIRMED** | Re-vérifié par NOM. `build_cors_test_router` (7661) au préfixe différent = manqué par le glob. |
| 2 | s1a/s3/s4 | `build_cors_test_router` = SEUL raw (sans couche), 5 call-sites CORS | **CONFIRMED** | :7662 `build_router(...)` sans `.layer` ; couche inject byte-id 4634-4648 vs 8578-8590. |
| 3 | s1a/s3 | couche inject fait inject token+Host **ET** `h.remove(ORIGIN)` :4644/:8587 | **CONFIRMED** | Le toggle de dédup doit gouverner les DEUX ensemble (sinon CORS-origin cassé). |
| 4 | s1a | `build_test_router(` = 179 = 1 def + 178 call-sites | **CONFIRMED** | `grep -c` = 179. Garder le wrapper zéro-arg intact = seul chemin behavior-preserving. |
| 5 | s1a | `_with_web_root` retourne `(Router, TempDir)` | **CONFIRMED** | :8591 `(router, tmp)`. Ne peut PAS se replier dans un constructeur `-> Router` nu. |
| 6 | s1a/s1b/s1a-OSS | 0 infra snapshot (insta/expect-test/.snap absents) ; hand-roll | **CONFIRMED** | 0 hit repo-wide. `serde_json` (dep directe :59) + `oneshot` (:4537) suffisent. |
| 7 | s3 | auth 401/403 = crate CORE `auth.rs:886-981`, invariant sous M | **CONFIRMED** | Split de domaine ne déplace pas l'auth (`use …core::auth` :51). |
| 8 | s3 | `feed_insert_rejects_without_internal_header` :7941 couplé au harness sans header extra | **CONFIRMED** | Assert 403 sans / 503 avec (:7959-7987). Couche inject à garder byte-id. |
| 9 | s1a/s3 | volatils `node_id`:4695 / `boot_time`:4697 / `pow_keypair`:4711 (node frais/test) | **CONFIRMED** | Golden DOIT masquer/normaliser ces champs sous peine de flake. |
| 10 | s1b | piège import `HeaderValue` local (:4632) absent de `_with_web_root` | **CONFIRMED** | Hoister l'import à la consolidation (compile-backstop). |
| 11 | tous | baseline vivante Win 2099 / Docker 2103, oracle `== baseline + N_golden` EXACT | **UNVERIFIABLE** | Counts non re-mesurables en lecture seule → **re-dériver LIVE** (`cargo nextest`) au code time ; raisonnement leçon-L sain. |
| 12 | s2-VERIFY note | `coordinator_verify_chain` = « hallucination », pas de route | **RENVERSÉ** | **FAUX** : handler :3973 câblé `GET /api/v1/kudos/{project_id}/verify` :376-377. Coordinator = 4 handlers (s1a/s2 corrects). |
| 13 | s1a/s2 | domaines contigus N/P/Q ; **S publish SCATTERED** (`publish_blob` :3261 ~2000 l isolé) | **CONFIRMED** | Le golden doit couvrir `publish_blob` isolé pour netter le déplacement dispersé de S. |

**UNVERIFIABLE non-porteurs** (aucun ne bloque le verdict) : (a) les counts nextest 2099/2103
exigent un run réel → re-mesurer au code time ; (b) URLs OSS externes (insta.rs, rust-analyzer
extract_module) non-fetchées, illustratives, technique prouvée in-tree (196 oneshot).

---

## Approche d'implémentation

### Design du golden (filet des splits N→S)

- **0 dep, hand-rollé** : piloter chaque surface via l'idiome existant `app.oneshot(Request::builder()
  .method().uri().body(...))` (:4537) → capturer `(status, headers curées, corps)` → comparer via
  un petit **normaliseur `serde_json::Value`** qui (a) **trie récursivement les clés** et
  (b) **redacte une allowlist FIXE de champs volatiles** vers des placeholders stables :
  `node_id`, `*_ticket`/`ticket`, `signature`/`sig`, `*_at`/`boot_time`/timestamps, ports
  éphémères, et tout hash dérivé d'un node frais (`archive_hash` si `make_zip` non-stable).
  Assertions = **statut + un set d'en-têtes significatifs explicites** (CSP blob-serve, COOP/COEP,
  `content-type`, `access-control-allow-origin`) **+ corps normalisé**.
- **Surfaces couvertes (≥ 1 par domaine + invariants transverses)** : N shard-session, O seed,
  P frost, Q coordinator (dont `verify_chain` via `/api/v1/kudos/{project_id}/verify`), R curators,
  S **publish + `publish_blob` isolé** (:3261) ; **+ 1 CORS-preflight via routeur RAW**
  (`build_cors_test_router`, seul capable d'exercer `Origin`) ; **+ 1 surface d'en-têtes CSP/blob-serve**
  ; **+ le fallback SPA** (`web_root` Some sert `index.html` en `text/html`). Privilégier les
  chemins **déterministes** (4xx/validation, strings statiques) ; pour les 200, asserter le key-set
  + valeurs masquées.
- **Fixtures** : **inline dans le module test `http.rs`** (table `(method, uri, [body]) → (status,
  headers, corps canonique)`). **Aucun fichier `.snap`, aucun `insta`.** Le golden doit être
  **vert DEUX fois sur HEAD inchangé** (prouver la complétude de l'allowlist de redaction) AVANT
  d'être fié ; ensuite le gate D4 certifie chaque split N→S.
- **Attention coût** : chaque test HTTP boote un node iroh frais (`create_node` :4680) →
  privilégier les surfaces non-networked/déterministes ; classe env Docker-on-Windows
  (`operator_server`/`convergence_*` >30 s) → `SBFB_TEST_HTTP_TIMEOUT_SECS=120` + re-run solo
  avant de conclure BLOCK.

### Design de la consolidation `build_test_router*`

- **UN constructeur paramétré à 3 axes** (test-only, 0 runtime dep, 0 route path) :
  `fn build_test_router_ext(state, cors: &[String], web_root: Option<&Path>, headers: TestHeaders) -> Router`
  où `TestHeaders::{Inject, Raw}` (ou `inject: bool`) **gouverne SIMULTANÉMENT** l'injection
  token+Host **ET** `h.remove(ORIGIN)`. **Hoister l'import `HeaderValue`.**
- **Préservation des postures d'auth (wrappers minces conservés)** :
  - `build_test_router(state)` = wrapper zéro-arg (Inject, `&[]`, `None`) → **178 call-sites
    intacts**.
  - `build_cors_test_router(state, cors)` = wrapper **Raw** → **5 call-sites CORS** conservent la
    posture raw (`Origin` observable).
  - `build_test_router_with_web_root(state)` = wrapper **Inject + `Some(web_root)`** retournant
    toujours **`(Router, TempDir)`** (le TempDir ne peut PAS se replier dans un `-> Router` nu :
    GC prématuré ⇒ fallback SPA sans `index.html`). **4 call-sites** intacts.
  - `build_test_router_with_cors` (interne, cors mort) = plié dans le wrapper zéro-arg.
- **Migration** : seuls les ~9 call-sites de variantes (5 raw + 4 web_root) sont touchés
  indirectement (mêmes noms de wrappers ⇒ **0 churn de signature**). `build_router` (prod) +
  la table de routes restent **byte-identiques** (diff :246-543 = vide après phase).
- **Séquencement** : golden **vert sur HEAD inchangé D'ABORD** (caractérisation), PUIS dédup
  prouvée byte-identique contre lui. L'ordre est load-bearing (filet avant dédup).

---

## Oracle T1 précis

Phase M **AJOUTE** des tests golden ; la dédup ne supprime que des **fns constructeur** (pas de
`#[test]`, donc 0 delta count depuis la dédup). L'oracle est donc **`== baseline_live + N_golden`
EXACT** (leçon Phase L adaptée à une phase additive — `>=` masquerait une suppression de test
compensée par les golden) :

- **`nextest Win == 2099 + N_golden`** ; **`Docker == 2103 + N_golden`** ; `N_golden` = nombre
  EXACT de `#[test]`/`#[tokio::test]` golden ajoutés, **énuméré explicitement dans le commit body**.
- **`Vitest web == 412` invariant** ; **`operator == 201` invariant** (Phase M ne touche pas le front).
- **`N_golden` identique Win vs Docker** : ne PAS `#[cfg(unix)]`-gater les tests golden (sinon N
  diverge — l'écart Win/Docker de +4 est dû aux `#[cfg(unix)]` préexistants ; ne pas en ajouter).
- **RE-DÉRIVER la baseline LIVE au code time** (`cargo nextest run --workspace --locked`) — ne PAS
  graver `2099/2103` sans re-mesure ; le plan (`2095/2099`) est stale.

**Gate D4 complet** : `cargo fmt --all --check` (**dual-platform Win + Docker rust:1.94**) +
`cargo clippy --workspace --all-targets --locked -- -D warnings` + `cargo nextest run --workspace
--locked` **== oracle exact** + `cargo test --workspace --locked --doc` + `cargo build -p
nexus-shell-daemon --release` (`cargo clean` si LNK1140) + **Vitest web 412 + operator 201** +
**0 route path** + **0 bump wire** + **golden vert sur l'état ACTUEL avant tout déplacement**.
`set -o pipefail` sur tous les pipelines (piège L : `| tail` masque l'exit code). **T2 = N/A**
(golden = filet interne, pas d'acceptance cross-machine). **frontier_closure = N/A** (prouvé S4).

---

## Verdict: PLAN-ADAPT

Le plan §Phase M (golden de caractérisation HTTP verrouillant l'identité pre/post + consolidation
des `build_test_router*` en un constructeur paramétré) est **réalisable sans violer aucune décision
Day-0 ni aucun invariant de sécurité** : auth intacte (crate CORE), aucune posture d'auth altérée,
0 dep runtime, golden 100 % `#[cfg(test)]`, `build_router` prod byte-identique (0 route path,
0 wire bump, S4 prouvé). Mais **5 faits du plan sont corrigés** (evidence disque pour chacun,
HEAD `013b611`) qui changent l'approche :

1. **Constructeurs = 3 `build_test_router*` (@4627/4631/8563) + un 4ᵉ au NOM DIFFÉRENT**
   (`build_cors_test_router` @7661, manqué par le glob) — pas « 4+ `build_test_router*` ». Les 4
   coordonnées plan (4645/4649/7534/8380) sont TOUTES stales. Re-dériver par NOM **en élargissant
   au-delà du préfixe** (greper les call-sites de `build_router`).
2. **Fichier = 12643 l** (pas ~12460) ; +183 l depuis kickoff (A/B/D/G, pas L) → toute coordonnée
   `http.rs:NNN` à re-dériver par NOM.
3. **3ᵉ axe de paramétrisation IMPOSÉ** : au-delà de `cors`/`web_root`, la **couche d'injection**
   (inject token+Host **ET** `h.remove(ORIGIN)`) est présente sur 3 variants, ABSENTE sur le raw
   (tests CORS) ; **+ `_with_web_root` retourne `(Router, TempDir)`**. Un merge 2-params casse les
   5 tests CORS ou fuit le TempDir. Toggle `TestHeaders::{Inject,Raw}` requis + wrappers minces
   conservés (178 + 5 + 4 call-sites intacts).
4. **Oracle resserré à `== baseline_live + N_golden` EXACT** (Win **2099** / Docker **2103** +
   N_golden), pas `>=` ni la baseline stale `2095/2099` ; `N_golden` énuméré, identique Win/Docker,
   baseline re-mesurée LIVE au code time.
5. **Domaines N→S non uniformément contigus** : N/P/Q contigus, R/O à frontières de helpers
   partagés, **S publish SCATTERED** (`publish_blob` @3261 isolé ~2000 l de `publish_project/directory`)
   → le golden DOIT couvrir `publish_blob` isolé pour netter le déplacement dispersé de S (M
   reste test-only, non-bloquant, mais dimensionnant).

**Angles morts explicites (à couvrir par relecture/vérification, pas par foi)** : (a) déterminisme
de `make_zip` pour `archive_hash` (masquer si non byte-stable) ; (b) complétude de l'allowlist de
redaction (prouver le golden vert DEUX fois sur HEAD inchangé) ; (c) préservation byte-identique de
la couche d'injection (piège `feed_insert_rejects_without_internal_header` :7941). Aucune
recommandation ne contredit D3 (split incrémental borné, golden AVANT), D4 (gate d'invariance), ni
la politique 0-dep-runtime. `insta` reste une escalade dev-dep OPTIONNELLE (non requise, arbitrable PO).
