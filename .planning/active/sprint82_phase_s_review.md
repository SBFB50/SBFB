# Sprint 82 Phase S — Review (Workflow)

## Contexte + méthode

- Diff reviewé : working tree vs HEAD `f7d42bc` (Phase R `curators_api.rs` DONE) — 6ᵉ split de
  `http.rs`, domaine `publish` extrait VERBATIM vers un NOUVEAU module
  `crates/nexus-shell-daemon/src/publish_api.rs` (move PUR, discipline étendue PO-10 ; tests
  co-migrés via `crate::test_support`). Preflight PLAN-ADAPT.
- Date : 2026-07-16.
- Orchestration : Workflow ultracode — **8 dimensions + 3 vérificateurs adversariaux (critics)** +
  cette synthèse ; chaque claim load-bearing **re-vérifié sur disque** (grep/`git diff`/`git show`
  indépendants, PAS une relecture des artefacts du main thread). 0 agent en erreur.
- Périmètre code + doc (6 fichiers trackés modifiés + 1 module untracked + 1 preflight untracked) :
  - `http.rs` (**7554 → 6220 l, net −1334** ; numstat `+21 / −1355`),
  - `publish_api.rs` (NEUF, **1397 l** — SPDX l.1 + `//!` anglais + `use` block + 5 tranches prod +
    `mod tests` avec **18 tests** [16 obligatoires + 2 optionnels co-migrés]),
  - `main.rs` (**+1** `mod publish_api;` au slot alpha `:54`, entre `panic` `:53` et
    `quarantine_api` `:55`),
  - `runtime.rs` (1/1 re-point `crate::publish_api::reannounce_directory_at_boot` `:1516`),
  - `test_support.rs` (doc-honnêteté `golden_http_publish_domain` `:573-577`, corps golden
    byte-identique),
  - `docs/rust/PATTERNS.md` (2 sites : swap `:3439` zone P59.8 + REWRITE zone `:1478-1490`
    `wrap_payload_with_pow`),
  - `docs/security/THREAT_MODEL.md` (swap `:1039` zone).
  - untracked : `publish_api.rs` + `sprint82_phase_s_preflight.md`.
- Preflight : PLAN-ADAPT (`sprint82_phase_s_preflight.md`) — move-set **13 symboles prod** + **16
  tests obligatoires + 2 optionnels** co-migrés (recommandation synthétiseur suivie) ; arbitrages
  `publish_blob` IN / boot-reannounce IN / revision helpers IN / `truncate_on_char_boundary`
  STAY+bump / `index_browse_entry` STAY ; **5 bumps** (3 handlers `pub(crate)` ROUTINE +
  `ErrorResponse.error` champ + `truncate`) ; re-points docs.

## Verdict: PASS

8 dimensions **PASS**. **0 P0, 0 P1, 0 P2 — 3 P3 confirmés** après vérification adversariale, tous
non-bloquants et de classe comptabilité / re-point-doc (cf. Table des findings). Le move est
byte-parfait : la preuve **TOKEN_IDENTICAL ×2** (PROD + TESTS) est le gate de fidélité contraignant,
re-joué indépendamment ; le top-line **7554→6220** est EXACT sur disque (`git diff --numstat` =
`21/1355`), et `publish_api.rs`=1397 l. Aucun défaut de code, aucune régression sécurité / frontière
/ tests.

Les 3 critics rendent **CONFIRMED** :
- **refute-findings** : 1 seul finding réel (P3 comptabilité), arithmétiquement exact
  (net −1334, `use`+`truncate` = net-0 in-place), **0 finding réfuté** ; corrobore mécaniquement les
  13 symboles définis exactement 1× crate-wide, tous dans `publish_api.rs` (dont
  `DirectoryPublishOutcome` = `enum`, pas struct).
- **hunt-missed** : **2 suppléments P3** ratés par les 8 dimensions — un lien intra-doc
  `[\`BrowseEntry\`]` **neuf-cassé** par le move (`publish_api.rs:92`, **prouvé empiriquement par
  `cargo doc`**) + une réf rustdoc **stale** dans `browse.rs:604` (`http::publish_directory` →
  symbole déplacé). Verdict global hunt-missed = PASS (2 P3 non-bloquants). Note honnête : 2 liens
  cassés PRÉ-EXISTANTS (`ProjectAnnouncement`, `auth::sbfb_home`) portés verbatim depuis HEAD = **PAS
  des régressions**.
- **suites-recheck** : Win 2108/2108 vert, `fmt` exit 0, comptabilité −1334 confirmée
  indépendamment ; le seul rouge Docker = flake env famille `running_json`
  (`e2e.rs:282 sigint_triggers_graceful_shutdown_and_removes_running_json`), **hors-domaine publish**,
  isolé par un run `--no-fail-fast` (2111 passed / 1 flake).

**PASS-PENDING** (jamais committable en l'état) : Codex (`gpt-5.6-sol`, effort max) pas encore joué,
et le re-run Docker solo-PASS frais du flake `sigint` reste à capturer côté main thread. Les 3 P3
sont routés au commit body (F1 comptabilité) et à un fix trivial optionnel in-class (F2/F3), sans
bloquer le PASS.

## Dimension 1 — Fidélité verbatim non-circulaire (PASS)

Move BYTE-PARFAIT, re-prouvé INDÉPENDAMMENT (slices re-extraites de `git show HEAD:http.rs`, tokenizer
lexer maison, comparaison contre le `publish_api.rs` disque — aucune lecture des `.tok` du main
thread) :

- **Non-circularité** : les 12 slices (`p1..p5` PROD + `t1..t7` TESTS) ré-extraites du HEAD sont
  `diff` byte-à-byte IDENTIQUES aux artefacts scratchpad — extraits EXACTS de l'ORIGINAL, ne dérivent
  pas du nouveau fichier.
- **PROD** : old 4991 tok → new 5004 tok, delta **+13**, `0 delete / 0 replace`, **4 inserts
  seulement** = 3 × `pub(crate)` (`publish_project`, `publish_directory`, `publish_blob`) + 1 `,`
  (rewrap signature `publish_blob`). `3×4 + 1 = 13` EXACT. Zéro token supprimé/altéré/réordonné.
  Corrobore l'assertion main-thread TOKEN_IDENTICAL 2576 (le décompte diffère uniquement par la
  granularité du lexer, l'ENSEMBLE des divergences matche à 100 %).
- **TESTS** : old 8454 tok = new 8454 tok, delta **0** → **TOKEN_IDENTICAL** (préambule `use super::*`
  frais correctement exclu de la région comparée). Corrobore l'assertion TOKEN_IDENTICAL 2803.
- **Bornage confirmé** : `publish_blob` HEAD = signature 1 ligne sans `pub(crate)` ni virgule
  trailing → `publish_api.rs` `pub(crate)` ⇒ ligne >100 chars ⇒ rustfmt éclate les params + virgule
  trailing. fmt mécanique légitime, unique. Les 4 autres `pub(crate)` du bloc directory
  (`build_sign_announce_directory`, `reannounce_directory_at_boot`, `read_directory_revision`,
  `DirectoryPublishOutcome`) N'apparaissent PAS comme inserts ⇒ déjà `pub(crate)` à HEAD ⇒ seuls les
  3 handlers ont un bump de visibilité, cohérent avec « 5 bumps [3 handlers + `ErrorResponse.error` +
  `truncate`] ».
- STAY non re-définis dans `publish_api.rs` (`truncate_on_char_boundary`, `index_browse_entry`,
  `mint_blob_ticket`, `wrap_payload_with_pow`, `ErrorResponse`) → tous importés via `use crate::http`.

## Dimension 2 — Intégrité du diff (PASS, 1 P3 → F1)

Chaque hunk classé dans exactement une catégorie, **0 hunk inexpliqué** :

- **16 hunks `http.rs`** : 12 retraits de slices (p1..p5 + t1..t7), 3 re-points de routes en full-path
  `crate::publish_api::{publish_project,publish_blob,publish_directory}` (`:372/:376/:380`, +12/−3),
  1 bump `ErrorResponse.error → pub(crate)` + doc 3 l (`:778-781`, +4/−1), 1 bump
  `truncate_on_char_boundary → pub(crate)` (net 0), 1 bannière blob-serve re-honnêtée (1→2 l), 2
  retraits de `use` orphelins (`Response` de axum, `info` de tracing — **0 usage restant vérifié des
  deux**).
- **Joints propres** : chaque frontière = `}` / blank unique / (test suivant | séparateur) ; **0
  double-blank dans tout `http.rs`** (awk = 0). Séparateurs de sections intacts
  (`// === Directory-only pull resolution ===`, CORS helper, Sprint 23 Phase E). **Îlot `truncate`
  conservé** (15 l, `:1033-1047`, corps verbatim), 1 blank propre avant/après. Bannières D-1 et
  `====` S20-B retirées AVEC leurs tests (0 orphelin de bannière).
- **0 double-définition** : les 13 symboles prod + tests échantillonnés → 1× crate-wide, tous dans
  `publish_api.rs` ; `ErrorResponse` → 1× `http.rs:777`. Seuls refs résiduels des handlers dans
  `http.rs` = les 3 route full-paths.
- **Autres fichiers** : `main.rs` mod-insert slot alpha ; `runtime.rs` re-point (résout : def
  `publish_api.rs:420`) ; `test_support.rs` doc-only (corps golden inchangé).
- **git status conforme** : uniquement fichiers de phase + 3 hors-phase PO ; 0 fichier parasite.

**Comptabilité (F1, P3)** : `http.rs` net −1334 (`--numstat 21/1355`). La décomposition narrative du
contexte « slices −1347 / routes +9 / doc champ +3 / bannière +1 / **use −2** » somme à **−1336** :
les 2 réécritures de `use` sont **net-0 in-place** (modif, pas suppression) et le bump `truncate` est
aussi net-0. Décomposition qui boucle : `−1347 slices + 9 routes + 3 ErrorResponse + 1 bannière + 0
use + 0 truncate = −1334`. **Impact code nul** (git byte-consistent, tous hunks attribués) ; à
corriger uniquement dans le commit body (pas encore écrit), pattern identique au P3 comptabilité de
Phase R.

## Dimension 3 — Sémantique des tests (PASS)

18/18 tests co-migrés vérifiés nom par nom (reconstruction indépendante par brace-matching depuis
`git show HEAD:http.rs`, non circulaire) :

- **(1) Absents de `http.rs`** : grep des 18 noms → EXIT=1 (0 match). Corroboré par le compte
  d'attributs de test : HEAD `http.rs` 156 → `http.rs` courant 138 (delta **18 exact**) ;
  `publish_api.rs` = 18 → **156 = 138 + 18** (move count-neutre).
- **(2) Présents verbatim** : les 18 fns → **ALL_IDENTICAL** octet pour octet, incluant bannières
  `///`, attributs `#[tokio::test]` et le `multi_thread worker_threads=4` préservé. Les 2 optionnels
  (`publish_announcement_persists_to_outbox_for_replay:857`,
  `publish_and_gossip_use_per_app_project_id:1358`) intacts ; les 2 duress-publish (#B-rt-1 fake
  curator empty, #B-rt-3 rejects task dispatch) présents.
- **(3) Imports résolvent + consommés** : `mod tests` `:567-573` = `use super::*` +
  `use crate::test_support::*` + `use crate::http::BrowseListResponse` + `to_bytes` /
  `{Method,Request}` / `tower::ServiceExt`. Usages comptés dans la zone (to_bytes 19, Method/Request
  13, `.oneshot` 13 ⇒ `ServiceExt` trait consommé, PublishRequest/Response/BlobResponse 7/2/1,
  fixtures `pub(crate)` de test_support sans promotion). `crate::http::BrowseListResponse` (STAY
  `http.rs:762` cfg-test) reste consommé côté http.rs (`:2750/:3461`) → non-orphelin des deux côtés.
- **(4) 0 test publish oublié dans `http.rs`** : seuls restent les 6 tests du cluster SEARCH,
  publiant via helpers locaux non-`pub` (`do_publish:5405`, `publish_app:5472`, `post_workspace:5764`)
  et référençant **0 symbole move-set** (grep région 5390-5680 → EXIT=1) ⇒ STAY légitime E0425.
- **Nextest workspace Win 2108/2108 0 skipped** (log `bog2bkmh0.output` Summary [75.280s]) = baseline
  EXACT ±0. 0 test dupliqué, 0 fixture cassée, 0 orphelin.

## Dimension 4 — Invariants de sécurité (PASS)

Tous les blocs load-bearing sécurité migrent INTACTS (prouvé par la byte-identité, seule exception =
les normalisations assertées) :

- **Duress early-return AVANT tout effet sensible** : `publish_project` `:106-110` (gate → `Noop`,
  `published:false` AVANT gossip et `deploy::publish_announcement`) ; `build_sign_announce_directory`
  `:250-254` (gate = 1er statement, AVANT signature) ; `publish_blob` `:535-545` (gate → 503 AVANT
  `BlobsClient::new`/`add_bytes`). Seul un `debug!` bénin précède (identique HEAD, non un finding).
- **Remediation #8 chemin canonique unique** : `publish_project` `:145-159` route UNIQUEMENT via
  `crate::deploy::publish_announcement` ; **0 appel** `wrap_payload_with_pow` dans le corps.
- **Gate S16 D-1** : `:120-131` `is_open_source && (provenance_hash.is_none() || repo_url.is_none())`
  → 400, message byte-identique.
- **verrou 1 / verrou 4 / lock-3** (`build_sign_announce_directory`) : own-only `:263`
  `own_entries(&my_node_id)` ; blob-held gate `:299-307`
  `if !matches!(blobs.has(hash_arr).await, Ok(true)) { continue; }` (content-addressing = vérité de
  propriété) ; clé locale seule `:267/:335` ; cap défensif `:276` `NODE_DIRECTORY_MAX_ENTRIES`.
- **Anti-rollback** (`next_directory_revision`) : `static REVISION_LOCK` process-wide `:496` +
  monotone `saturating_add(1)` `:507` + persist atomique tmp→rename `:512-515` + fallback
  `sbfb_home`/`return 1` ; `read_directory_revision` retourne 0 = jamais publié = clé du gate
  `reannounce_directory_at_boot:421`.
- **Caps UTF-8** : exactement **4** `truncate_on_char_boundary` (`:314/:319/:323/:327`) avant `sign()`
  ; helper reste `http.rs:1038` (`pub(crate)`, importé).
- **Bump `ErrorResponse.error` = 0 fuite** : `http.rs:777` `pub(crate) struct` (PAS `pub`) + `:781`
  `pub(crate) error: String` + doc 3 l, `private_interfaces` clean, dérive inchangée → wire JSON
  `{"error":...}` byte-identique. Module `mod publish_api;` non-`pub` + handlers `pub(crate)` →
  **0 surface externe élargie**.
- **0 changement de surface** : 3 routes byte-identiques (`/api/daemon/publish`,
  `/api/daemon/publish-blob`, `/api/daemon/directory/publish`), toujours dans `authed_routes`
  (tier T0 loopback bearer + Host + Origin), seul re-point = full-path handler. 0 nouvel endpoint, 0
  route déplacée entre tiers. Doc-sécurité THREAT_MODEL/PATTERNS = swap `http.rs→publish_api.rs`,
  `own_entries` reste `browse.rs`, PATTERNS:1478-1490 corrige un claim DÉJÀ FAUX au disque
  (« publish_project now calls wrap_payload_with_pow »).

## Dimension 5 — Frontières / scope (PASS)

Le move ne déborde sur aucun domaine adjacent (8 checks re-greppés) :

1. **Pull-resolution STAY `http.rs`** : `PULL_PROVIDER_CAP:1058`, `DIRECTORY_PULL_TIMEOUT_SECS:1064`,
   `find_directory_app_by_hash:1070`, `..._by_project:1095`, `directory_pull_providers:1131` — 0 dans
   `publish_api.rs`. `seed_api.rs` **non modifié** ; ses imports `use crate::http::{...}` intacts.
2. **Nodes STAY** : `NodesResponse/ObservedNodeView/NodeSummary/nodes_response/list_nodes` + test
   `nodes_response_pins_envelope_and_grouping` — 0 dans `publish_api.rs`.
3. **Index-chokepoint STAY** : `trustworthy_open_source:1275`, `index_browse_entry:1283` — 0 réf
   (ni def ni appel) dans `publish_api.rs`.
4. **blob_serve / mint_blob_ticket / panic_wipe STAY** ; `publish_api` importe `mint_blob_ticket` et
   l'appelle bare (`publish_blob` déplacé IN).
5. **Tests cluster/search STAY** (`publish_makes_app_searchable_*`, `do_publish`, `post_workspace`).
6. **`BrowseListResponse` STAY `http.rs:762`** et sert les deux côtés (http.rs tests + publish_api
   tests via `use crate::http::BrowseListResponse`, 0 redéfinition).
7. **`truncate_on_char_boundary` STAY + bump seul** (diff ne touche QUE la ligne de signature,
   doc-comment intact) ; publish_api y accède via `use crate::http` + 4 appels bare verbatim.
8. **0 wire** : aucun `Cargo*` touché (0 delta), `nexus-core-rs` UNTOUCHED, `publish_api.rs` = 0
   constante `_VERSION`/`ANNOUNCEMENT_VERSION`/`FEED_FORMAT_VERSION`/`DOMAIN_`, 0 canonical.

Hygiène de frontière (bonus, 0 finding) : retraits `use` orphelins sûrs ; 1 seul re-point cross-module
(`runtime.rs:1516`) ; 9 symboles prod déplacés + tests échantillonnés → 1× crate-wide ; 2 tests
duress « optionnels » co-migrés à bon droit (ils exercent la surface publish malgré leurs noms
boot/curator — pas un débordement du domaine curator Phase R).

## Dimension 6 — Conformité préflight (PASS)

L'implémentation suit le PLAN-ADAPT point par point, 0 déviation non documentée :

- **Move-set 13 symboles** présents avec visibilités conformes §3.1 : 3 DTOs `pub` (`:40/73/79`) ;
  `publish_project/directory/blob` **`pub(crate)`** (`:94/190/526`) ; `PublishDirectoryResponse` +
  `DirectoryRevisionFile` privés (`:166/450`) ; `DirectoryPublishOutcome`/`build_sign_announce_directory`/
  `reannounce_directory_at_boot`/`read_directory_revision` `pub(crate)` (`:222/245/420/460`) ;
  `next_directory_revision` privé + `static REVISION_LOCK:496`. Symboles MOVED absents de `http.rs`
  (grep NONE-FOUND-GOOD), 0 duplication.
- **Arbitrages §4 appliqués** : `publish_blob` IN ✓ ; boot-reannounce IN + re-point `runtime.rs:1516`
  (seul caller externe) ✓ ; revision helpers IN ✓ ; `index_browse_entry` + `trustworthy_open_source`
  STAY ✓ ; `truncate` STAY + bump (doc dual-domaine intacte) ✓ ; 2 tests optionnels co-migrés
  (bandeau annonce **18 tests**) ✓ ; `seed_api.rs` 0 change ✓.
- **Checklist compile-hazard §8 (7 pts)** conforme : `ErrorResponse.error → pub(crate)`
  (`private_interfaces` clean) ; `truncate → pub(crate)` ; 3 handlers `pub(crate)` ; re-points
  runtime + 3 routes + `mod publish_api;` slot alpha ; `use crate::http::BrowseListResponse` déplacé ;
  cluster search + helpers `do_publish`/`publish_app` STAY (E0425) ; `use` orphelins listés À LA
  COMPILE.
- **Écart réel « 2 use orphelins `http.rs` »** = classe/discipline §3.9 (« À LISTER À LA COMPILE, ne
  pas présumer — leçon R ») : `Response` (type de retour des handlers déplacés, désormais consommé
  publish_api) + `info` (seul consommateur `reannounce_directory_at_boot`, migré). Retraits prouvés
  sains par `clippy -D warnings` + nextest 466/466. **Lecture main-thread confirmée**, pas une
  déviation.
- Invariants VERBATIM §6 préservés (duress early-return, Remediation #8, gate D-1, verrous 1/4/lock-3,
  floor anti-rollback + REVISION_LOCK, cap UTF-8, announce LIVE-only/jamais-outbox, route-inventory
  `//!` STAY http.rs:16-18, bannière S20-B + hot-join verbatim).

## Dimension 7 — Docs re-points (PASS)

Les 5 edits docs vérifiés au disque, exacts factuellement ; re-grep exhaustif = **0 re-point manqué**
côté docs/web/scripts/tools/examples ; les 3 gates docs EXIT 0 :

1. **THREAT_MODEL.md:1039** swap `build_sign_announce_directory` `http.rs → publish_api.rs`,
   `own_entries` reste `browse.rs`. ✓
2. **PATTERNS.md:3439** (zone P59.8, miroir) même swap, `own_entries` reste `browse.rs`. ✓
3. **PATTERNS.md:1478-1490** REWRITE : `wrap_payload_with_pow` DEF reste `http.rs:1012`
   (`pub(crate) fn`) ; 2 vrais callers ré-ancrés = `deploy.rs:696` (dans `publish_announcement`,
   Remediation #8) + `publish_api.rs:379` (dans `build_sign_announce_directory`). L'ancienne prose
   « publish_project **now calls** … (same file) » était doublement stale → REWRITE justifié. ✓
4. **test_support.rs:573-577** doc-honnêteté : rationale « ~2000 lines away / scattered » → immuable-
   passé « they were scattered across `http.rs` when this net was laid » + « co-located in
   `publish_api.rs` (S82 Phase S) » ; corps golden byte-identique (5 ins / 4 del = doc seul). ✓
5. **Bannière `http.rs::tests` blob-serve** : `// Sprint 12 Phase A: blob-serve + publish-blob` →
   mention `publish_api.rs`, 1→2 l re-honnêtée. ✓

- Gates : `check-frontier-contracts.sh` / `check-sharding-docs.sh` / `check-factory-docs.sh` = clean.
- `SPRINT_LOG.md:56` (`DaemonHandle::publish_project` = méthode client test-harness, SHA-loggée)
  correctement EXCLU (convention historique jamais re-pointée). Ancres route PATH
  (`shell/PATTERNS.md`, `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`) inchangées, drift-proof.

## Dimension 8 — Livrables + patterns (PASS)

Gabarit `*_api.rs` fidèle aux 5 modules précédents (N/O/P/Q/R) :

- **Bandeau** : SPDX l.1 identique ; provenance `//!` « extracted verbatim from `http.rs` (Sprint 82
  Phase S, PO-10 extended discipline: the domain's 18 router-driven and direct-call tests
  co-migrated below via `crate::test_support`) » (formule « and direct-call » = plus précise, le
  domaine a `vps_authoring` direct-call, pas une déviation) ; bloc T0 tier + bloc SHARED-stays
  (`ErrorResponse`/`truncate_on_char_boundary`/`mint_blob_ticket`/`wrap_payload_with_pow` STAY
  `http.rs` — les 4 confirmés au disque `:777/1038/1540/1012`). Compteur bandeau exact = **18
  marqueurs `#[test]/#[tokio::test]`**.
- **Slot `mod publish_api;`** `main.rs:54` alpha correct (`pan` < `pub` < `qua`), +1 exact.
- **D4 gate refacto** : `fmt --all --check` OK (Win + Linux) ; clippy `-D warnings` VERT (3
  `doc_lazy_continuation` réglés au disque) ; 2 `use` orphelins fixés + prouvés non-consommés
  (`info!` = 0, `Response` nu = 0) ; nextest Win 2108/2108 0-skip EXACT.
- **Doc-comment `ErrorResponse.error`** techniquement exact : consommateur cross-module réel =
  `publish_api.rs` construit `ErrorResponse { … }` à **4 sites** (`:123/215/540/556`) → E0451 sans
  bump champ (contraste correct vs Phase R où R ne bumpait que struct + constructeur).
- **Anti STALE-PHASE-K** : scan `lines away|sits ~|now calls|currently|line NNN` sur http.rs +
  test_support + PATTERNS + THREAT_MODEL + publish_api → **0 match**. Provenance immuable-passé
  uniquement (« since S82 Phase S », « ex-`http.rs` S82 Phase S », « when this net was laid »).
- **frontier_closure N/A** : 3 route paths byte-identiques, `grep web/src` des routes/handlers publish
  → 0 match (web ne consomme aucune des 3) → frontier front N/A confirmé. Encoding UTF-8 sans BOM,
  LF-only ; 0 emoji ; 0 magic number introduit (move verbatim).

## Table des findings (après vérification adversariale)

| # | Sév. | Source | Titre | Action |
|---|---|---|---|---|
| F1 | **P3** | Dim intégrité-diff + critics refute / suites-recheck (**CONFIRMÉ ×3**) | Décomposition lignes « use −2 » ne boucle pas au net −1334 (`http.rs`) — les 2 `use` + `truncate` sont **net-0 in-place** | **CONFIRMÉ, non-bloquant** — routage vers le commit body (pas encore écrit) ; porter `−1334 = 21 ins/1355 del`, décrire `use`+`truncate` en net-0 |
| F2 | **P3** | Critic hunt-missed (raté par les 8 dim, **prouvé `cargo doc`**) | Lien intra-doc `[\`BrowseEntry\`]` **neuf-cassé** par le move (`publish_api.rs:92`) — résolvait à HEAD (`http.rs` importait `BrowseEntry` bare), `publish_api` ne l'importe pas | **Non-bloquant** — fix trivial in-class : `use nexus_shell_daemon_core::browse::BrowseEntry;` (recommandé, cohérent avec la discipline re-point) OU body-note |
| F3 | **P3** | Critic hunt-missed (raté par les 8 dim) | Réf rustdoc **stale** `browse.rs:604` : `See \`http::publish_directory\`` nomme le mauvais module post-move (symbole migré) | **Non-bloquant** — fix trivial : `\`publish_api::publish_directory\`` (même classe que les re-points THREAT_MODEL:1039 / PATTERNS:3439 déjà traités) OU body-note |

**0 P0 / 0 P1 / 0 P2 / 3 P3.**

Détail F2/F3 (angle mort de la preuve token) : `cargo doc -p nexus-shell-daemon --no-deps -W
rustdoc::broken_intra_doc_links` (exit 0, 8 warnings) surface un `unresolved link to BrowseEntry`
`publish_api.rs:92`. TOKEN_IDENTICAL ne le capte PAS car la **résolution de lien dépend du scope
d'import environnant** (http.rs riche → publish_api minimal), pas des tokens ; `cargo doc` est absent
du suite ET de la CI (`git grep 'cargo doc' .woodpecker/.github` → NONE), 0 `deny(broken_intra_doc_links)`
→ invisible aux gates verts. **Impact réel = nul** (runtime/compile/test = 0 ; seul le lien hypertexte
de la rustdoc générée dégrade en texte brut). F3 est cross-crate (core→daemon, backticks simples, non
un lien résoluble) donc 0 warning cargo doc, mais **factuellement stale re: un symbole déplacé par la
phase** — exactement la classe que la discipline re-point a traitée pour les 3 sites docs/. Les
docs-repoints ont scoppé leur grep à `docs/`+`web/src`+`scripts`+`tools`+`examples` (jamais
`crates/**/*.rs`), d'où le miss.

**NOT-A-FINDING (documentés)** :
- 2 liens intra-doc PRÉ-EXISTANTS `[\`ProjectAnnouncement\`]` (`publish_api.rs:90`) + `[\`auth::sbfb_home\`]`
  (`publish_api.rs:479`) → **DÉJÀ cassés à HEAD** dans `http.rs` (importés seulement fully-qualified),
  portés **verbatim** ⇒ conformes à la discipline de move, **PAS des régressions** (hunt-missed).
- OK-note P3 « section-header blob-serve » (préflight-conformance) = **le même changement physique**
  que la bannière déjà comptée par intégrité-diff (édit doc-honnêteté classe ADD-1, 0 comportement) —
  sa propre dimension le déclare « ne compte pas comme déviation ». Non-finding (refute-findings).

**Findings RÉFUTÉS : néant.** Le critic refute-findings ne réfute aucun finding et confirme l'exactitude
arithmétique de F1 ; hunt-missed ajoute F2/F3 (survivent) ; suites-recheck corrobore F1 et 0 défaut code.

## Preuve verbatim (TOKEN_IDENTICAL ×2)

Re-joué indépendamment (slices ré-extraites de `git show HEAD`, lexer maison, `difflib`) :

- **PROD** : old 4991 tok → new 5004 tok, delta **+13** = 3 `pub(crate)` + 1 `,`, `0 delete / 0
  replace`. Corrobore l'assertion main-thread **TOKEN_IDENTICAL 2576** (granularité lexer différente,
  ensemble des divergences identique à 100 %).
- **TESTS** : old 8454 = new 8454, delta **0** → **TOKEN_IDENTICAL**. Corrobore l'assertion **2803**.
- **Non-circularité prouvée** : les 12 slices ré-extraites du HEAD = byte-à-byte identiques aux
  artefacts scratchpad ⇒ le côté source est une extraction de l'ORIGINAL, pas une copie du dest.
- **Bornage** : les 4 seules divergences PROD = les 3 préfixes handler `pub(crate)` assertés + le
  fmt-rewrap unique de `publish_blob` (gardé par la contrainte « ligne >100 chars »). Aucune autre
  transformation.

## Vérification §7.4 (suites, résultats audités)

- **Compile parfaite 1er coup** (imports prédits par le préflight exacts).
- **Win** : `fmt --all --check` 0 (re-exécuté indépendamment, EXIT=0) ; `clippy --workspace
  --all-targets -D warnings` VERT ; crate `nexus-shell-daemon` **466/466 0-skip** ; **nextest
  workspace 2108/2108 0 skipped, delta ±0 EXACT** vs baseline 2108 (log `bog2bkmh0.output` Summary
  [75.280s] + CLIPPY-OK + DOCTESTS-OK + RELEASE-OK + RUST-WIN-ALL-GREEN) ; build release daemon OK.
- **Docker canonique `sbfb-ci`** (mount `/workspace`, `bash -c`) : `fmt` OK (FMT-LINUX-OK). Baseline
  **2112** (2108 Win + 4 `#[cfg(unix)]`). 1er run fail-fast : FAIL à 951/2112 sur
  `e2e sigint_triggers_graceful_shutdown_and_removes_running_json` (`e2e.rs:282`) = **FLAKE ENV connu
  famille `running_json`** (phases Q/R, re-run solo PASS), **hors-domaine publish** (binaire
  `tests/e2e.rs` absent du diff). Run `--no-fail-fast` (`bc2amh64q.output`) : **2112 run, 2111 passed
  (16 slow), 1 failed** = UNIQUE `sigint` (**tous les tests publish verts**) ; solo PASS
  (`bfkht8f7d.output`) ; run plein-vert existant `b12g81hyf.output` = 2112/2112. **Docker = PENDING**
  pour un solo-PASS frais du cycle, mais identité du flake établie.
- **Web** : lint / tsc / unit **412/412** / coverage / build / size / `scan-en-strings` = ALL-GREEN.
- **Operator** : lint / build / unit **201** = ALL-GREEN.
- **Golden** : famille **9/9 PASS** dont `golden_http_publish_domain` (route-driven, byte-identique) —
  observateur externe 0-drift JSON.
- **Gates docs** : `check-frontier-contracts` / `check-sharding-docs` / `check-factory-docs` = EXIT 0.
- T2 = N/A (move pur).

## Comptabilité (chiffres à porter au body)

`git diff --stat` = **6 files changed, 37 insertions(+), 1365 deletions(-)** (fichiers trackés ;
`publish_api.rs` untracked hors stat) :

| Fichier | ins | del | net |
|---|---|---|---|
| `http.rs` | 21 | 1355 | **−1334** |
| `main.rs` | 1 | 0 | +1 |
| `runtime.rs` | 1 | 1 | 0 |
| `test_support.rs` | 5 | 4 | +1 |
| `PATTERNS.md` | 8 | 4 | +4 |
| `THREAT_MODEL.md` | 1 | 1 | 0 |

`publish_api.rs` = **1397 l** (neuf, untracked). Décomposition `http.rs` qui **boucle** (F1) :
`−1347 slices + 9 routes + 3 ErrorResponse doc + 1 bannière + 0 use (net-0 in-place) + 0 truncate
(net-0) = −1334` (`+21 / −1355`). Session cumulée `http.rs` : 13130 → **6220** sur la série de splits.
Delta tests **±0** (Win 2108 / Docker 2112). **Ne jamais reporter « use −2 »** (F1).

## Conformité aux arbitrages préflight + edits compile-force

1. **Move-set 13 symboles IN `publish_api.rs`**, visibilités §3.1 exactes. ✓
2. **`publish_blob` IN** (`:526`), **boot-reannounce IN** + re-point `runtime.rs:1516` (seul caller
   externe), **revision helpers IN** (`read/next_directory_revision` + `DirectoryRevisionFile` +
   `REVISION_LOCK`). ✓
3. **`truncate_on_char_boundary` STAY + bump `pub(crate)`** (`http.rs:1038`, doc dual-domaine intacte)
   ; **`index_browse_entry` STAY** (`http.rs:1283`, 0 bump). ✓
4. **5 bumps** = 3 handlers `pub(crate)` + `ErrorResponse.error` `pub(crate)` + `truncate`
   `pub(crate)` ; `ErrorResponse` struct RESTE `pub(crate)` (champ non `pub`). ✓
5. **2 tests optionnels co-migrés** (`publish_announcement_persists_to_outbox_for_replay:857`,
   `publish_and_gossip_use_per_app_project_id:1358`) — recommandation synthétiseur suivie. ✓
6. **Re-points docs** : THREAT_MODEL:1039, PATTERNS:3439, PATTERNS:1478-1490 REWRITE,
   test_support:573-577, bannière blob-serve ; `seed_api.rs` 0 change, `SPRINT_LOG` INTACT. ✓
7. **[compile-force, classe §3.9]** retrait de 2 `use` orphelins `http.rs` (`Response` axum, `info`
   tracing) — orphelinage réel prouvé (0 usage restant), `clippy -D warnings` VERT. ✓

## Note de staging / hygiène commit (routage, pas un finding)

Working tree porte 3 fichiers de recherche PO **hors-phase** (état pré-existant, intacts, à NE PAS
committer avec Phase S) : ` M .planning/research/sprint82_workflow_engine/verification_blueprint.md`
+ `?? …/workflow_agents_app_conception_ultradeep_2026-07-15.md` +
`?? …/workflow_hub_product_conception_2026-07-15.md`. Consigne au committer (discipline standing) :
**stager EXPLICITEMENT** les fichiers de phase — `publish_api.rs` + `http.rs` + `main.rs` +
`runtime.rs` + `test_support.rs` + `docs/rust/PATTERNS.md` + `docs/security/THREAT_MODEL.md` +
`sprint82_phase_s_preflight.md` (+ ce `sprint82_phase_s_review.md`) — **jamais `git add -A`/`-a`** ;
vérifier `git diff --cached --name-only`. Chiffres à porter au body : `http.rs` 7554→**6220** (net
−1334, 21 ins/1355 del), `publish_api.rs` **1397** (neuf), delta tests **±0** (Win 2108 / Docker
2112), décomposition qui boucle **−1347 slices / +9 routes / +3 ErrorResponse / +1 bannière / use &
truncate net-0 = −1334** (F1) ; mentionner F2 (BrowseEntry) + F3 (`browse.rs:604`) — fix trivial
in-class recommandé OU body-note.

## Prochaine étape

Codex (`gpt-5.6-sol`, `model_reasoning_effort=max`, `--sandbox read-only` tant que `elevated` cassé)
sur les livrables de phase, puis réconciliation → promotion PASS-PENDING → PASS si CLEAN (ou boucle si
GAP). En parallèle : re-run Docker solo-PASS frais du flake `sigint` famille `running_json` pour
sceller la baseline 2112. Les 3 P3 (F1 body-only ; F2/F3 fixes triviaux in-class optionnels) n'exigent
aucune retouche de code avant Codex.

## Codex reconciliation

Rapport brut : `sprint82_phase_s_codex_review.md` (`codex exec -m gpt-5.6-sol -c
model_reasoning_effort=max --sandbox read-only`, round 1, artefact NON réécrit). Résumé Codex :
**10 livrables — 7 CONFIRMÉ, 0 GAP, 3 PARTIEL**. Tri des 3 PARTIEL (aucun défaut code) :

1. **Livrable 1 PARTIEL — 2 lignes rustdoc « non autorisées » dans `publish_project`**
   (`publish_api.rs:94-95`, reference-link `[BrowseEntry]`). C'est le **fix F2 de cette review**
   (critic hunt-missed, lien intra-doc neuf-cassé prouvé `cargo doc`), appliqué in-class APRÈS la
   rédaction du prompt Codex — le prompt listait la normalisation pré-review (3 `pub(crate)` +
   fmt-rewrap) sans le doc-link. Codex confirme lui-même : « Les corps exécutables restent
   byte-identiques après normalisation ». Réparation re-prouvée : `cargo doc -p nexus-shell-daemon`
   ne signale plus `unresolved link to BrowseEntry` (7 warnings restants = pré-existants portés
   verbatim, hors-phase). **PARTIEL-process documenté au commit body, pas un GAP.**
2. **Livrable 10 PARTIEL — `browse.rs:604` modifié hors ledger des 5 sites docs.** C'est le **fix F3
   de cette review** (réf rustdoc stale `http::publish_directory` → `publish_api::publish_directory`),
   même séquencement post-prompt. Édition documentaire 1 ligne, 0 logique — Codex la juge
   « documentaire et cohérente ». **PARTIEL-process documenté, pas un GAP.**
3. **Livrable 9 PARTIEL — réserve env read-only** (Codex ne peut pas ré-exécuter cargo/nextest sous
   `--sandbox read-only` : `target/` + tempdirs en écriture). Substance et intégrité des goldens
   confirmées statiquement par Codex ; l'état vert est porté par les runs main-thread : **Win
   2108/2108 0-skip, Docker `--no-fail-fast` 2112 run / 2111 passed + flake env `sigint` re-run solo
   PASS (1.639s), crate 466/466, golden 9/9**. Même classe que les réserves env de la Phase R.

Corroborations Codex indépendantes : verbatim exact par `git show HEAD` sur les 5 tranches prod + 7
blocs tests ; compte d'attributs conservé **156 = 138 (http.rs) + 18 (publish_api.rs)** ; 62
assertions statiques dans les tests migrés ; numstat `+21/−1355` ; routes paths byte-identiques ;
`DaemonHttpState` identique à HEAD ; 0 delta Cargo.toml/lock/web.

Critère d'arrêt boucle Codex (« CLEAN ou P2/P3 documentés ») : **atteint round 1** — 0 GAP, 0 défaut
code ; les 3 PARTIEL sont des réserves process/env documentées ci-dessus et au commit body. Aucune
correction de code exigée → pas de re-boucle suites/review/Codex. Verdict promu **PASS**.
