# Sprint 82 — Phase S3 — Review (Workflow ultracode)

- **Phase** : S3 — split domaines `feed/provenance` + `search` + `preview/proof-card`
  → `crates/nexus-shell-daemon/src/{feed_api.rs, search_api.rs, preview_api.rs}`
- **Date** : 2026-07-17
- **Base** : HEAD `b9b892a` + working tree Phase S3 (non committée)
- **Méthode** : Workflow — 8 dimensions parallèles opus-4-8[1m] + passe verify
  adversariale sur chaque finding ; diff lu en entier ; oracle token re-dérivé
  INDÉPENDAMMENT depuis `git show b9b892a:…/http.rs` (non-circulaire vis-à-vis du
  manifest de session) ; faits load-bearing ré-ancrés sur disque par le
  synthétiseur (line counts, 89 routes, 6 re-points, 23 tests, F2, DTO deviation).

## Verdict: PASS

> Review Workflow : **8/8 dimensions PASS**, **0 P0 / 0 P1 / 0 P2**. Un seul
> finding **P3 CONFIRMÉ** (comptabilité : bornes de plages en prose du commit
> body légèrement imprécises vs hunks git `-U0` — net inchangé, correction
> cosmétique du wording, appliquée au commit body). Promu de PASS-PENDING à
> **PASS** après le gate Codex Sol (round 1 : 9/10 CONFIRMED + 1 PARTIAL
> process, GLOBAL PASS, 0 P0/P1 — cf. §Codex reconciliation).

## Dimensions (8/8 PASS)

| # | Dimension | Verdict | Findings |
|---|---|---|---|
| 1 | verbatim-oracle (token indépendant + reconstruction) | PASS | 0 |
| 2 | routes-wire (89==89 / 0 wire / 0 dep / goldens) | PASS | 0 |
| 3 | tests-graphe (co-migration + helpers) | PASS | 0 |
| 4 | scope-stay (frontières + STAY sémantique) | PASS | 0 |
| 5 | securite-invariants | PASS | 0 |
| 6 | docs-contrat (test-acteur §6.12 + angle mort F2) | PASS | 0 |
| 7 | comptabilite (livrables chiffrés) | PASS | **1 P3 confirmé** |
| 8 | patterns-hygiene | PASS | 0 |

### 1. verbatim-oracle — TOKEN_IDENTICAL prouvé non-circulairement
Chaque tranche re-dérivée PAR NOM depuis `b9b892a:http.rs` et comparée token à
token (whitespace-insensible, autorisant SEULEMENT les 8 préfixes `pub(crate)`,
la dé-indentation des 3 helpers, et la ponctuation de wrap rustfmt) contre
`feed_api.rs` / `search_api.rs` / `preview_api.rs` + la queue de
`test_support.rs` : les **11 tranches déplacées** MATCH (3 blocs prod handlers,
5 groupes de tests dont la réassemblée non-contiguë search en 3 régions, 3
helpers promus). Reconstruction complète (`http_old.rs` moins les 4 plages
migrées, diffé au vrai `http.rs`) : les suppressions n'ont RIEN retiré au-delà
du contenu déplacé — seuls résidus = retrait du `use body::Bytes` orphelin, les
6 re-points de routes full-path (noms de handler byte-identiques), le dé-link F2,
et un unique collapse de ligne blanche imposé par fmt à la couture du `mod tests`
(whitespace-only, `fmt --check` EXIT 0). Champs des DTO restés privés
(feed_api.rs:146-153, search_api.rs:34-38) ; `PreviewLoadResponse`/`pub hash`
verbatim (preview_api.rs:38-40).

### 2. routes-wire — 89==89, 0 wire, 0 dep, goldens intacts
(a) Set-diff canonique de `build_router` = **89 == 89**, tous paths + méthodes
byte-identiques ; SEUL delta = 6 handler-refs re-pointés `crate::feed_api::` /
`crate::search_api::` / `crate::preview_api::` (single→multi-ligne = pur wrap
rustfmt du path full-qualifié). (b) 0 wire : les 11 symboles prod token-identiques
modulo visibilité + re-wrap fmt de 2 signatures (`get_feed_cursor`,
`preview_load` — corps byte-identiques) ; toutes les shapes JSON préservées
(`PreviewLoadResponse{hash}`, 413 `TooLarge{error}`, `Json(card)`,
`{results,total}`, `{entries}`, `{verified}`) ; `FEED_FORMAT_VERSION` /
`*_ANNOUNCEMENT_VERSION` / `DOMAIN_*` intouchés (seul un commentaire déplacé les
mentionne). (c) 0 dep : Cargo.toml/lock absents du diff. (d) 9 `golden_http_*`
intouchés (test_support = append pur +64/-0 en queue). Le pair suspect
`/provenance→/browse` est un artefact d'alignement de lignes git (tests browse
STAY remontent combler le trou des tests provenance migrés) — confirmé par
placement exact-une-fois/zéro.

### 3. tests-graphe — 23 co-migrés exactement une fois, graphe helpers correct
`http.rs` 133→110 (−23) ; feed_api +9, search_api +8, preview_api +6 = 23 ;
set-diff {retirés de http.rs} vs {ajoutés aux 3 modules} VIDE dans les deux sens,
0 doublon intra-union, total crate conservé (mesuré 466). Les 3 helpers promus
single-définis `pub(crate)` dans test_support (make_test_zip:849, publish_app:863,
search_total:889) ; les tests STAY les résolvent via le glob PRÉ-EXISTANT
`use crate::test_support::*` (http.rs:1429) — glob unique, 0 ambiguïté.
`insert_test_feed_entry` co-migré PRIVÉ dans feed_api (seul consommateur = tests
feed). Imports des mod tests neufs (KeyPair/Router/BrowseEntry + glob) tous
consommés. 0 test orphelin : aucun test STAY de http.rs ne référence un symbole
parti (les moved symbols n'apparaissent que dans le routing prod + 1 doc :902).

### 4. scope-stay — périmètre exact, STAY byte-identiques
`git diff HEAD --name-only` = exactement les 8 fichiers de phase + 3 hors-phase
PO (blueprint M + 2 untracked workflow_*). Cluster Directory-only pull-resolution
(`find_directory_app_by_project`, `directory_pull_providers`,
`DIRECTORY_PULL_TIMEOUT_SECS`, `PULL_PROVIDER_CAP`) byte-identique (dans le gap
entre hunks → cible S4 intacte). `browse_entries`/`post_workspace` + les 2 tests
browse relocalisés DANS http.rs (pas déplacés vers browse_api.rs, pas perdus).
`truncate_on_char_boundary` reste pub(crate) (http.rs:903), importé par
search_api, doc altérée SEULEMENT par le dé-link F2. Les 4 tests « nom dit
domaine mais STAY » présents (browse_index_rejects :1441, feed_insert_rejects
:1988, fork_redeploy_resigns :4080, finalize_deploy_open_source :4279). AUCUN
edit d'un sibling `*_api.rs` (seed/publish/browse/curators/coordinator/frost/
quarantine/diagnostic absents du set). Bannière orpheline pré-existante
`// -- Sprint 74 Phase D: keep-online local pin --` STAY (http.rs:4321).

### 5. securite-invariants — auth + invariants VERBATIM préservés
Les 6 routes re-pointées restent DANS `authed_routes` (bearer+Host+Origin,
`auth_required` @http.rs:606) — aucune ne glisse vers public_routes/token_route.
CARRY-5 clamps verbatim + ordre clamp/troncature AVANT search (search_api.rs:71/73
puis :76). S73-D triplet UNINDEXED + `serde(default)` runtime-tolerance migrés
verbatim. `preview_load` : `TooLarge`→413. `get_provenance`/`get_proof_card` :
verify_provenance Ed25519 (verified/failed/absent) + tests cross-node
verified/tampered co-migrés. 0 duress / 0 guardrail / 0 internal-header dans le
move-set (vérifié contre les originaux retirés — aucune perte silencieuse ; les
write-side gated `feed_insert`/`feed_status` restent routés `crate::feed_sync`).

### 6. docs-contrat — F2 traité, angle mort balayé, gates verts
daemon.ts:617 re-pointe `http.rs`→`search_api.rs` (Zod inchangé) ; daemon.ts:646
= nom-seul sans chemin → NO-ACTION correct. Dé-link F2 fait à http.rs:902
(backticks, 0 crochet) ; grep négatif ciblé = c'est le SEUL lien intra-doc
http.rs vers un symbole déplacé, et les 3 modules neufs portent 0 lien à crochets
→ aucun `[SymboleReste]` cassé. Grep des 13 symboles déplacés dans docs/ ne
touche que SPRINT_LOG.md (narration figée). Les 3 gates docs-contrat re-joués
EXIT 0 (frontier / factory / sharding). `frontier_closure` N/A justifié (89==89,
route strings byte-identiques, 3 DTO déplacés verbatim, shapes daemon.ts
inchangées). Consigné (pré-existant, NON causé par S3) : TOOLING.md:291 ancre
http.rs:483-494 déjà imprécise à HEAD b9b892a — candidate cleanup futur.

### 7. comptabilite — chiffres réconciliés indépendamment (1 P3)
`http.rs` 5635→4322 réconcilie à la ligne : numstat 53/1366 = net −1313 ;
5635−1313=4322 == wc -l. Bumps = **8 ROUTINE / 0 SHARED** (6 handlers + 2 DTO
Query privés@HEAD → pub(crate) ; `PreviewLoadResponse` pub verbatim non compté ;
+3 promotions test_support). Tests 9/8/6=23, crate 466 (nextest list),
count-neutre. wc -l + 3 mod main.rs alpha + daemon.ts comment-only conformes.
Workspace ±0 (2108) structurellement garanti (moves intra-crate, 0 test add/del).
**1 finding P3 CONFIRMÉ** (voir §Findings).

### 8. patterns-hygiene — headers factuellement exacts, style conforme
Les 3 headers de module vérifiés contre code + histoire : counts 9/8/6,
attributions routes/sprint (S63-B/C, S67-B, S68-A/B), cross-claims
(`feed_insert/feed_status vivent dans feed_sync` confirmé feed_sync.rs:524/591 ;
`truncate_on_char_boundary dual-domaine` confirmé) — 0 sur-attribution, 0
promesse-du-futur (anti STALE-PHASE-K respecté). Style aligné sur les 8 splits
`*_api.rs` antérieurs (SPDX l.1, ordre std/external/crate, tests `super::*` +
`test_support::*`). fmt --all --check + clippy -p nexus-shell-daemon
--all-targets -D warnings re-joués EXIT 0. main.rs mods alpha
(feed_api:42 / preview_api:56 / search_api:61).

## Findings

### P3 CONFIRMÉ — Bornes de plages supprimées imprécises vs hunks git `-U0`
- **Dimension** : comptabilite. **Sévérité** : P3 (plancher). **is_real=true**,
  non réfutable sur disque, dans le scope de phase (le commit body EST un
  livrable de phase).
- **Fait** : le descriptif de phase cite « tests 4450-5005 » et « tests
  5325-5634 ». Les frontières git mécaniques diffèrent : (1) le hunk région-2 est
  `@@ -5324,311 @@` — la suppression démarre à old:5324 (ligne vide), 311 lignes,
  donc « 5325-5634 » est off-by-one au début et sous-compte (310 vs 311) ;
  (2) HEAD:http.rs:5323 = bannière keep-online (STAY, survit à new:4321), 5324 =
  blanc, 5325 = bannière Sprint 73 (début du contenu supprimé) ; (3) le cluster-1
  dernier hunk `@@ -4520,540 @@` s'étend jusqu'à old:5059 et est ENTRELACÉ de
  tests STAY (browse_entries/multiple_apps/published_app survivant à new
  4005/4023/4053) — « 4450-5005 » est un label de région contiguë, pas l'étendue
  mécanique.
- **Impact** : NUL sur le net (−1313 exact, HEAD 5635 → worktree 4322), sur les
  compteurs (2108→2108), sur la correction, sur tout gate. Les bornes citées sont
  des labels approximatifs défendables (frontières SÉMANTIQUES : bannière Sprint
  73 à 5325 ; dernier test avant le helper STAY browse_entries) et non des claims
  faux. La « correction » proposée par le reviewer (relabel 4450-5059) est
  elle-même discutable car elle taggerait une région contenant des tests STAY.
- **Action** : correction cosmétique du wording du commit body (citer 5324-5634 ;
  cluster 4450-5059 entrelacé) — NON bloquante.

### Aucun finding réfuté ce round
Contrairement à S2 (2 P3 de trace réfutés), les 7 autres dimensions n'émettent
que des OK-NOTE ; aucune n'a produit de finding à réfuter. Les 2 constats
pré-existants consignés (0 golden couvrant feed/search/preview ; TOOLING.md:291
ancre imprécise depuis avril 2026) sont des dettes ANTÉRIEURES à S3, inchangées
par un move pur, non-bloquantes.

## Déviation compiler-forcée (consignée) + fix in-phase

- **Déviation DTO Query privés → pub(crate)** : la prédiction préflight « les DTO
  Query restent privés — le lint ne se déclenche pas » est RÉFUTÉE par la compile
  (4 erreurs `type FeedEntriesQuery/SearchQuery is private` aux call-sites
  `build_router`). Classe Phase R « reachability du type au call-site » : le
  handler `pub(crate)` re-pointé cross-module expose `Query<T>` dans sa signature
  atteignable depuis http.rs, alors que `T` reste privé au module neuf. Fix
  minimal : `FeedEntriesQuery` (feed_api.rs:145) + `SearchQuery` (search_api.rs:33)
  → **pub(crate)**, **champs conservés PRIVÉS** (feed_api.rs:146-153,
  search_api.rs:34-38), Deserialize-only → **0 impact wire**. Bump ledger 6 → **8
  ROUTINE / 0 SHARED**. Amendé au préflight §3.5/§4.5 AVANT commit (même pattern
  que la déviation E0425 compiler-forcée Phase O).
- **Fix doc-wrap** : clippy `doc_lazy_continuation` sur le header NEUF de
  feed_api.rs (ligne doc commençant par « + Host ») → re-wrap 3 lignes, **0 code**.

## Suites (§7.4) — vérifiées, re-ancrées sur disque

- **Structure** (re-jouée par le synthétiseur, lecture seule) : http.rs
  **5635→4322** (numstat 53/1366, net −1313 EXACT) ; feed_api 556 / search_api
  479 / preview_api 354 / test_support 905 (+64/−0) ; `.route(` = **89 == 89** ;
  6 re-points full-path crate::{feed_api,search_api,preview_api}:: (http.rs
  491/495/497/500/504/532) ; F2 dé-link http.rs:902 (backticks) ; consts
  MAX_SEARCH_* localisées search_api.rs:52-53 UNIQUEMENT ; 0 stray
  `crate::http::<movedsym>` ; `body::Bytes` retiré de http.rs ; daemon.ts:617
  re-pointé ; main.rs 3 mods alpha ; 8 tests STAY présents ; 0 nom de test
  déplacé restant dans http.rs ; git status = 8 fichiers de phase + 3 hors-phase
  PO (staging sélectif au commit, JAMAIS `git add -A`).
- **Gates rapportés côté phase** (à re-confirmer au commit, non re-lancés ici —
  lecture seule) : fmt --check clean Win ; clippy workspace -D warnings PASS Win ;
  nextest Win workspace **2108/2108** 0-skip (delta ±0 EXACT) ; crate **466/466** ;
  doctests 6 ; web 7/7 (Vitest 412) ; operator Vitest 201 ; 3 gates docs EXIT 0 ;
  Docker sbfb-ci nextest **2112/2112 CONFIRMÉ VERT** (0 skip, 0 flake,
  FMT-DOCKER-CLEAN, mount `/workspace`) — terminé APRÈS l'écriture initiale de
  cette review, statut rafraîchi à la réconciliation Codex (P3 Codex #3) ;
  s'ajoutent depuis : doctests 6 pass + release build daemon OK (6m47s) =
  BLOC RUST WIN 5/5 TOUT VERT.

## À porter au commit body

8 bumps ROUTINE / 0 SHARED (6 handlers privés → pub(crate) + déviation
compiler-forcée 2 DTO Query privés → pub(crate) champs privés conservés) ;
`PreviewLoadResponse` pub verbatim ; 23 tests co-migrés (9 feed / 8 search / 6
preview) ; 3 promotions test_support (make_test_zip/publish_app/search_total,
count-neutral) ; dé-link F2 http.rs:902 ; re-point daemon.ts:617 (:646 NO-ACTION) ;
fix doc-wrap feed_api header. **Corriger le wording des bornes de plages** (P3 :
région-2 = 5324-5634 ; cluster-1 = 4450-5059 entrelacé de tests STAY).
**GAP pré-existant consigné** (non-bloquant, candidat S4) : aucun des 9
`golden_http_*` n'observe /feed /search /provenance /preview /proof-card
(observateur externe = tests domaine co-migrés) ; TOOLING.md:291 ancre imprécise
pré-existante. STAY vers S4 : cluster Directory-only pull-resolution + bannière
orpheline Sprint 74. HORS-PHASE PO à EXCLURE du commit : verification_blueprint.md
(M) + 2 untracked workflow_*_2026-07-15.md. Comptabilité : http.rs 5635→4322 /
feed_api 556 / search_api 479 / preview_api 354 / test_support +64 ; 0 wire, 0 dep,
0 route path change, 89 routes.

## Codex reconciliation

- **Rapport** : `.planning/active/sprint82_phase_s3_codex_review.md` — output
  BRUT `codex exec -m gpt-5.6-sol -c model_reasoning_effort=max --sandbox
  read-only` (workaround standing `elevated` cassé, cf. Phase R), non réécrit.
- **Round 1** : **9/10 CONFIRMED + 1 PARTIAL** (livrable 10 « green gates » —
  replay runtime sandbox-limité côté Codex, 0 défaut code ; équivalent
  round-1-clean code, 10ᵉ de la série S82). **GLOBAL VERDICT: PASS, 0 P0/P1 GAP.**
- **Notes P2/P3 Codex, toutes traitées sans boucle** : (P2 pré-existant) 0 golden
  feed/search/provenance/preview — DÉJÀ consigné ici + commit body ; (P3 #1)
  wording des bornes = hunk mécanique 5324-5634 + cluster-1 entrelacé — appliqué
  au commit body (converge avec le P3 de cette review) ; (P3 #2) exclure du
  commit le blueprint PO + les 2 research untracked — staging sélectif appliqué ;
  (P3 #3) statut Docker périmé dans cette review — rafraîchi ci-dessus (2112/2112
  CONFIRMÉ VERT). 0 GAP code → 0 correction code → pas de boucle re-suites/
  re-review/re-Codex requise (critère d'arrêt : CLEAN ou P2/P3 documentés).
