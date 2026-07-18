# Sprint 82 — Phase S2 — Review (Workflow ultracode)

- **Phase** : S2 — split domaine `browse + nodes` → `browse_api.rs`
- **Date** : 2026-07-17
- **Base** : HEAD `be7e2be` + working tree Phase S2 (non committée)
- **Méthode** : Workflow `wf_11151767-626` — 10 agents opus-4-8[1m] (8 dimensions
  parallèles + passe verify adversariale sur chaque finding), diff lu en entier,
  oracle token re-dérivé INDÉPENDAMMENT depuis `git show be7e2be` (non-circulaire
  vis-à-vis du manifest de session).

## Verdict: PASS

> Promu après réconciliation Codex (round 1 CLEAN, voir `## Codex
> reconciliation`). Review Workflow : 8/8 dimensions PASS,
> **0 P0 / 0 P1 / 0 P2** ; 2 findings P3 rapportés par la dimension
> preflight-conformité, **tous deux RÉFUTÉS** par la passe adversariale
> (is_real=false : trace prédiction-vs-impl, pas des défauts).

## Dimensions (8/8 PASS)

| Dimension | Verdict | Findings |
|---|---|---|
| verbatim-token (oracle indépendant) | PASS | 0 |
| diff-complet | PASS | 0 |
| tests-semantique | PASS | 0 |
| scope-frontieres | PASS | 0 |
| securite-invariants | PASS | 0 |
| docs-contrat (test-acteur §6.12) | PASS | 0 |
| patterns-comptabilite | PASS | 0 |
| preflight-conformite | PASS | 2 P3 → réfutés |

### 1. verbatim-token — TOKEN_IDENTICAL prouvé non-circulairement
Les 6 tranches HEAD (853-1000, 1161-1258, 2180-2291, 2457-2638, 2730-2752,
3445-3463) re-extraites depuis `git show be7e2be:…/http.rs` == corps de
`browse_api.rs` (après retrait bandeau/`use`/entête tests/`}` final, et
re-normalisation des 3 `pub(crate) async fn` → `async fn`) : **2513 == 2513
tokens**. `http.rs` courant == HEAD moins les 6 tranches avec pour SEULS autres
deltas : 3 routes re-pointées full-path (browse/pull re-wrappée 4 lignes par
rustfmt, whitespace-only, path byte-identique) + 1 ligne doc F2.

### 2. diff-complet
9 hunks http.rs, tous dans le périmètre annoncé ; `git status` = le set de
phase (browse_api.rs neuf + 4 M + preflight.md) PLUS les 3 fichiers research
hors-phase PO (blueprint M + 2 untracked workflow_*) qui restent intacts et
HORS staging — commit en staging sélectif, jamais `git add -A` (note Codex).
Bloc `use` prod MINIMAL (chaque import consommé, `Deserialize` correctement
absent) ; entête mod tests MINIMAL ; 0 use orphelin dans http.rs (SystemTime →
boot_time/blob_serve, BrowseEntry → BrowseListResponse/index_browse_entry, etc.).
Bandeau `//!` provenance-passé-immuable uniquement, claims vérifiés exacts.

### 3. tests-semantique
browse_api.rs porte EXACTEMENT 5 tests (2 MANDATORY direct-call + 3
router-driven) ; corps byte-identiques à HEAD (diff = IDENTICAL ×5) ; http.rs
les a perdus exactement (0 hit) ; les 4 stayers présents (browse_index_rejects
:1874, directory_resolvers :1957, fetch_provider_ordering :2049, spa_fallback
:2829) ; 0 duplication (chaque nom 1× dans le crate) ; helpers résolvent
(test_support pub(crate) ×6, BrowseListResponse via crate::http) ; count crate
invariant **466/466** ; cluster search S3 INTACT.

### 4. scope-frontieres
Cluster pull-resolution ENTIER intact dans http.rs (5 symboles + 2 tests
HARD-BOUND, 0 changement de visibilité) ; `seed_api.rs` / `publish_api.rs` :
**0 edit** (git diff vide — invariant Phase S préservé) ; blob_serve /
mint_blob_ticket / index-chokepoint / wrap_payload_with_pow / truncate intacts ;
paths de routes byte-identiques, même compte de `.route(` ; 0 wire (0 hit
`_VERSION|DOMAIN_|canonical` dans le diff) ; Cargo.toml/lock intacts ;
`//!` route-inventory http.rs intact.

### 5. securite-invariants
Duress early-return de browse_pull verbatim (1er statement, seul handler duress
du move-set — list_browse/list_nodes read-only sans garde, intentionnel) ;
logique CATALOG-BACKED de browse_views inchangée + doc SEC-UXARR-1 verbatim ;
skip empty-hash placeholders verbatim ; bannières verrou-4 / /browse
byte-identique / KEEP-ONLINE-READ-PATH préservées ; THREAT_MODEL.md:1024
re-pointé (style « ex-http.rs S82 Phase S2 » cohérent avec la row Phase O) ;
shapes JSON identiques ({entries}, {requested}, envelope {nodes, observed}).

### 6. docs-contrat
3 re-points faits (THREAT_MODEL:1024, F2 http.rs:758 dé-croché, test_support
:700-703 ADD-1 honnête) ; grep des 10 symboles dans docs/ + scripts/ + CI :
0 re-point restant (hits résiduels = name-only drift-proof ou historique) ;
liens rustdoc internes de browse_api ([`NodesResponse`] ×2) résolvent
(intra-module) ; frontier_closure N/A confirmé (web = route-path + shapes
miroir, refs name-only) ; 3 gate-scripts sans ancre vers le move-set.

### 7. patterns-comptabilite
http.rs **6220 → 5635** (−585 net : −588 tranches+blanks, +3 re-wrap fmt
browse/pull) ; browse_api.rs **635 l** (prédiction 620-660) ; style conforme aux
*_api.rs ; `mod browse_api;` alpha (main.rs:32) ; exactement 3 `pub(crate)` dans
browse_api.rs ; PATTERNS:939 / :3354-3355 / :4159 + shell:2087 restent vrais
(STAY appliqués) ; verrue doc « GET /browse attachée à BrowseEntryView »
préservée telle quelle (verbatim d'abord) ; 0 TODO/magic/emoji introduit.

### 8. preflight-conformite
Checklist compile-hazard 1-8 : TOUTE conforme. Les 4 arbitrages appliqués
(cluster STAY→S4, BrowseListResponse STAY, spa_fallback STAY, reachable
CO-MIGRE). Bloc use prod = match EXACT à la prédiction §4.1. 2 P3 de trace
rapportés puis **réfutés** par la passe verify :
1. « F2 dé-croché sans préfixe crate:: » — réfuté : le dé-crochet est une des
   deux options sanctionnées par le preflight ; un span backtick n'est pas un
   lien intra-doc (0 résolution rustdoc), et la référence module-qualifiée nomme
   fidèlement le nouvel emplacement.
2. « Ordre des imports mod tests ≠ prédiction §4.2 » — réfuté : le bloc réel est
   le SEUL ordonnancement fmt-conforme (gate `cargo fmt --check` dur) ; la
   prédiction était non-normative (« la liste est une prédiction ») ; les 7
   symboles prédits tous présents.

## Suites (§7.4) au moment de la review

- `cargo fmt --all --check` : **CLEAN** (après application du re-wrap fmt).
- `cargo nextest run -p nexus-shell-daemon --locked` : **466/466 PASS**
  (compile parfaite du 1ᵉʳ coup, count crate invariant).
- Bloc web complet : **ALL_WEB_GREEN** (lint, tsc, **412/412** unit, coverage,
  build, size, scan-en-strings — marqueur final atteint, aucune étape avalée).
- Bloc Rust Windows complet : **ALL_RUST_WIN_GREEN** — clippy
  `--all-targets -D warnings` clean, nextest workspace **2108/2108 PASS**
  (delta **±0 EXACT** vs baseline Win 2108), doctests OK, release build OK.
- Docker sbfb-ci (canonique Linux) : **DOCKER_GREEN** — fmt 0, nextest
  `--no-fail-fast` **2112/2112 PASS, 0 flake** (delta ±0 EXACT).
- Bloc operator **ALL-GREEN** (lint, tsc, unit **201/201**, 35 fichiers) ;
  3 gates docs frontier/sharding/factory **exit 0** ; `cargo doc` : 7 warnings
  TOUS pré-existants HEAD, 0 introduit, 0 lien BrowseEntryView.

## À porter au commit body

3 bumps ROUTINE (handlers → pub(crate)) ; 5 tests co-migrés (2 MANDATORY + 3
router-driven) ; arbitrages cluster pull STAY→S4 (routage explicite au préflight
S4 : « le cluster suit-il blob_serve ? ») + BrowseListResponse STAY +
spa_fallback STAY + reachable co-migré (correction critic vs inv-prod) ; fix F2
lien invisible-à-cargo-doc ; re-points THREAT_MODEL:1024 + test_support ADD-1 ;
**GAP pré-existant consigné** : aucun des 9 golden_http_* n'observe browse/nodes
(observateur externe = les tests route-level co-migrés) et browse_pull n'a aucun
test direct — gaps pré-existants inchangés, non-bloquants ; 0 wire bump, 0 dep,
0 route path change ; comptabilité http.rs 6220→5635 / browse_api.rs 635.

## Codex reconciliation

Codex GPT-5.6 Sol round 1 (`codex exec -m gpt-5.6-sol -c
model_reasoning_effort=max --sandbox read-only` — elevated toujours cassé,
piège standing) — artefact brut : `sprint82_phase_s2_codex_review.md`.
**GLOBAL VERDICT : CLEAN / PASS WITH NOTES — 0 P0, 0 P1, 10/10 livrables
CONFIRMED.** Corroborations indépendantes : comparaison mécanique UTF-8 des 6
tranches vs `git show be7e2be` (3 seuls préfixes pub(crate)) ;
`FULL_HTTP_RECONSTRUCTION_EXACT` (http.rs courant reconstruit depuis HEAD) ;
multiset des paths de routes 89==89 ; `git diff --quiet be7e2be` exit 0 sur
seed_api.rs ET publish_api.rs ; grep des 10 symboles dans docs/scripts/CI
propre ; duress/CATALOG-BACKED/verrou-4/shapes vérifiés ligne à ligne ; les 2
tests MANDATORY exécutés à frais PASS (les 3 router-tests bloqués UNIQUEMENT
par le sandbox read-only — tempdir refusé test_support.rs:137 — couverts par
les runs main-thread Win 2108/2108 + Docker 2112/2112). Notes réconciliées :
2 P2 résiduels PRÉ-EXISTANTS (0 golden browse/nodes, browse_pull sans test
direct) consignés review + commit body ; 1 note staging (pas de `git add -A`)
appliquée au commit sélectif ; 1 P3 formulation review.md §2 — corrigé
in-class AVANT commit. Critère d'arrêt « CLEAN ou P2/P3 documentés » atteint
round 1 — 0 boucle. Verdict promu PASS ; suites non relancées (0 changement
de code post-Codex).
