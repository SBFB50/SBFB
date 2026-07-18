# Sprint 82 Phase R — Review (Workflow)

## Contexte + méthode

- Diff reviewé : working tree vs HEAD `7faa632` (Phase Q `coordinator_api.rs` DONE) — split du
  domaine `curators` de `http.rs` vers un NOUVEAU module
  `crates/nexus-shell-daemon/src/curators_api.rs` (move PUR verbatim, discipline étendue PO-10 ;
  preflight PLAN-ADAPT — **1ᵉʳ bump de visibilité d'un symbole PARTAGÉ** de la série de splits S82).
- Date : 2026-07-16.
- Orchestration : Workflow ultracode — 8 dimensions + 2 vérificateurs adversariaux (critics) +
  synthèse ; chaque claim load-bearing **re-vérifié sur disque** ici (oracle indépendant, PAS le
  script fourni). 0 agent en erreur.
- Périmètre code + doc (5 fichiers trackés modifiés + 2 untracked de phase) :
  - `http.rs` (**8111 → 7554 l, net −557** ; numstat `+20/−577`),
  - `curators_api.rs` (NEUF, **611 l** — SPDX l.1 + `//!` anglais + `use` block + 4 `pub struct`
    DTOs + 4 `pub(crate) async fn` + 10 `#[tokio::test]`),
  - `main.rs` (**+1** `mod curators_api;` au slot alpha `:37`),
  - `web/src/api/daemon.ts` (1/1 re-point commentaire file-ancré), `docs/rust/PATTERNS.md`
    (§G-3, 3 DTOs re-attribués + numéros inline dropés + test re-pointé), `docs/shell/PATTERNS.md`
    (P19, 1/1).
  - untracked : `curators_api.rs` + `sprint82_phase_r_preflight.md`.
- Preflight : PLAN-ADAPT (`sprint82_phase_r_preflight.md`) — approche du plan confirmée + **6
  adaptations matérielles prouvées disque** (2 bumps `pub(crate)` `ErrorResponse` + helper STAY ;
  Q1 `default_curators` = IN ; correction `SubscribeCuratorResponse`→`SubscriptionsResponse` ;
  bornes re-dérivées par NOM ; 3 re-points docs FORM-couplés ; slot `mod` main.rs `:36↔:37`)
  + **1 edit compile-force non prévu** (retrait de 3 `use` du mod tests de `http.rs`).

## Verdict: PASS

8 dimensions **PASS**. **0 P0, 0 P1, 0 P2 — 1 P3 confirmé** après vérification adversariale : la
décomposition narrative « −568 slices / +11 fmt / −3 use » du décompte de lignes ne boucle pas à
net −557 (somme à −560) ; le vrai contributeur fmt&bump est **+14** (cf. Table des findings). Le
top-line **8111→7554 est EXACT sur disque** (`wc -l` + `numstat +20/−577`) et la preuve
**TOKEN_IDENTICAL ×2** est le gate de fidélité contraignant — le P3 est un nit de comptabilité pour
le **commit body pas encore écrit**, 0 défaut de code, non-bloquant.

Les 2 critics rendent **CONFIRMED** : critic-moveset confirme la fidélité du move (4 DTOs + 4
handlers + 10 tests, 0 résiduel, dispositions STAY correctes) et la sévérité P3 exacte ;
critic-proof confirme la preuve token **non-circulaire** (chaque slice source est un substring
verbatim de `http_before.rs` 8111 l — 9/9 `slice in before` == True) et **bornée** (normalisations
= 4 préfixes handler `pub(crate)` + 1 fmt-rewrap asserté unique). **0 finding réfuté, 0 missed**
matériel des deux côtés.

Verdict initialement rendu **PASS-PENDING** (review OK, Codex pas encore joué) — promu **PASS**
après réconciliation Codex (cf. section « Codex reconciliation » en fin de document).

## Dimension 1 — Fidélité verbatim (PASS)

Move BYTE-PARFAIT, re-prouvé indépendamment :

- **12 hunks `http.rs`** tracent 1:1 au preflight §2/§3 ou à une adaptation documentée : 4 route
  re-points (§5), 4 tranches DTO/handler retirées (items 1-3 + Q1 item 4 ; items 5-7 + Q1 item 8),
  1 hunk bump `pub(crate)` (`ErrorResponse` + `runtime_error_to_response`, STAY §1.4), 3 `use` test
  orphelins retirés, 4 hunks de suppression de tests (10 tests). **0 hunk inexpliqué.**
- **`curators_api.rs`** = exactement les 8 défs (4 `pub struct` :38/49/56/73 + 4 `pub(crate) async
  fn` :80/92/148/168) + 10 `#[tokio::test]` (`mod tests` :181, glob `use crate::test_support::*`
  :193). SPDX l.1, `//!` header ANGLAIS (:2-15) conforme aux siblings N/O/P/Q — le brouillon FR du
  preflight §4 a été correctement anglicisé.
- **Sur-suppression = 0** : les 5 DTOs interleaving STAY (`BrowseListResponse`/`PublishRequest`/
  `PublishResponse`/`PublishBlobResponse`/`NeighborhoodResponse`) = 1 def chacun encore dans
  `http.rs` ; les 4 tests curator-adjacents STAY (`info_reflects_live_curator_runtime_counts`,
  `browse_returns_empty_list…`, `daemon_boot…publishes_fake_curator_empty` #B-rt-1,
  `spa_fallback_serves_curators_as_html_document`) tous présents.
- **Sous-suppression = 0** : grep des 8 symboles migrés dans `http.rs` = 0 def résiduelle ET 0
  référence ; les 10 noms de tests migrés = 0 dans `http.rs` (le seul hit `CuratorRuntime` restant
  à `http.rs:1347` est un doc-comment `CuratorRuntime::repull_directories`, pas un type).
- **`main.rs`** : 1 ligne `+mod curators_api;` au slot alphabétique correct (`:35`
  `contributor_api` → `:36` `coordinator_api` → **`:37` `curators_api`** → `:38` `deploy`), mod
  normal (pas `cfg(test)`).

## Dimension 2 — Suites §7.4 (PASS)

`cargo fmt --all --check` re-exécuté → exit 0. Tests migrés ciblés (`-E
'test(/^curators_api::tests::/)'`) → **10 run / 10 PASS / 456 skipped** ; goldens (`-E
'test(/golden_http/)'`) → **9 run / 9 PASS / 457 skipped** avec `golden_http_curators_domain`
présent et VERT (`test_support.rs:544`, famille atomique). Crate count = **466** confirmé par 2 runs
concordants (10+456 == 9+457). `git diff -- '*Cargo.toml' '*Cargo.lock'` = VIDE (0 delta dep,
adaptation preflight S1b tenue). Réconciliation lignes exacte : `http.rs` disque = 7554, numstat
+20/−577 = net −557, 8111 − 557 = 7554 EXACT ; `curators_api.rs` = 611. Les 2 flakes env sont
correctement qualifiés : Docker `sigint_triggers_graceful_shutdown_and_removes_running_json` (classe
documentée Phase Q, re-run solo PASS), vitest coverage sous charge parallèle (memory
`vitest_env_variance`, re-run solo PASS).

## Dimension 3 — Branch coverage sémantique (PASS)

Les 10 tests migrent BYTE-IDENTIQUES en substance (diff strict source scratchpad
tests_1..4.rs+banner vs `curators_api.rs::tests` = 396 == 396 lignes non-blanches, seul delta = +4
blanks aux 4 bornes de tranche, cosmétique rustfmt). **Aucun test affaibli** : chaque fn apparaît
exactement 1× (dest==src==1) ; agrégat asserts 26==26, `.uri(` 12==12, `Method::` 12==12 — mêmes
routes, mêmes asserts, mêmes status codes. Tests router-driven via `build_test_router` + `.oneshot`
URI-string (`/api/daemon/curators`, `/curators/subscribe`, `/default-curators`), 0 appel direct de
handler. Invariants duress + hot-join intacts dans les corps migrés : `subscribe_curator_in_duress…`
et `…invalid_hex…` assertent le canal gossip `Empty` ; `…pushes_hot_join…` assert 1 seul
`GossipCmd::JoinPeers` puis `Empty` ; `daemon_boot_in_duress…rejects_curator_subscribe_real` assert
réponse ET attention-set vides. **0 test curator orphelin** dans `http.rs::tests` (0 hit
`/api/daemon/curators`|`default-curators`, 0 des 10 fn).

## Dimension 4 — Scope + déviations (PASS)

**0 code hors du split.** git diff = exactement 5 fichiers phase trackés + 2 untracked de phase.
Chaque hunk `http.rs` est soit un move, soit une adaptation documentée (2 bumps `pub(crate)`
`ErrorResponse` :814 + `runtime_error_to_response` :818, tous deux STAY, champ `error` reste privé
:815), soit une conséquence compile-forcée (fmt-rewrap de la signature helper forcé par
`pub(crate)` ; retrait de 3 `use` tests). **0 renommage** (`SubscribeCuratorRequest`/
`SubscriptionsResponse`/`CuratorsListResponse` verbatim), **0 refactor opportuniste**. Le helper
hors-phase `mk_state_with_default_curators` (preflight §3 « amélioration facultative ») **n'a PAS
été ajouté** — `test_support.rs` INTOUCHÉ, `default_curators_returns_configured_list` migre verbatim
en construisant `DaemonHttpState` inline (conforme à la discipline). Q1 = IN appliqué INTÉGRALEMENT
(handler + DTO + 2 tests + route :375 + doc P19). Retrait des 3 `use` EXACTEMENT minimal
(`BlobServeCache`/`BrowseAggregator`/`CuratorRuntime`, ni plus ni moins), orphelinage réel vérifié
(post-migration, 0 ref bare aux 3 types dans `http.rs`), `clippy -D warnings` VERT = preuve
mécanique. Fichiers hors-phase PO NON touchés (`verification_blueprint.md` + 2 `workflow_*.md` =
état pré-existant, recherche parallèle, 0 rapport avec le split).

## Dimension 5 — Sécurité deep (PASS)

Tous les blocs load-bearing sécurité migrent INTACTS (prouvé par la byte-identité) :

- **Porte duress `subscribe_curator`** (`curators_api.rs:96-111`) : `curator_subscribe_in_duress(…)
  == SubscribeOutcome::Noop` est le 1ᵉʳ statement exécutable après `debug!`, early-return 200
  `SubscriptionsResponse{subscribed_curators: Vec::new()}` STRICTEMENT AVANT `subscribe` (:112) et
  AVANT le push JoinPeers (:128) — la clé leurre n'atteint jamais subscribe ni le gossip.
- **Hot-join** (:113 bras Ok, push :128) : `GossipCmd::JoinPeers` best-effort APRÈS la mutation.
  Grep crate-wide → `curators_api.rs:130` = SEUL producteur ; `runtime.rs:2064` = seul consommateur
  (STAY). Bras Err (:142) + early-return duress gardent le push inatteignable pour clés
  invalides/leurres.
- **Absence de porte duress sur `unsubscribe_curator`** (INTENTIONNEL S20-B) préservée verbatim —
  aucune porte ajoutée.
- **Validation pubkey = PASSTHROUGH** ; le mapping erreur→status
  (`runtime_error_to_response`, `BadPubkeyHex→400`, autres→422, `Persistence→500`) reste verbatim
  dans `http.rs:818` via le helper STAY ; `nexus-shell-daemon-core` intouchée.
- Les 4 routes restent sous `authed_routes` (bearer X-SBFB-Token + Host loopback + Origin), paths
  byte-identiques → **tier T0 inchangé**, move auth-neutre. Les bumps `pub(crate)` n'élargissent pas
  la surface hors crate (module `mod curators_api;` PRIVÉ, `ErrorResponse` champ privé, DTOs
  effectivement crate-internes). Golden DELETE 400 résout car la chaîne
  `unsubscribe → BadPubkeyHex → runtime_error_to_response → ErrorResponse` (STAY intra-`http.rs`)
  est préservée.

## Dimension 6 — Docs-contrat frontière (test-acteur, §6.12) (PASS)

Les 3 re-points docs appliqués et EXACTS : `daemon.ts:102` `http.rs`→`curators_api.rs` (commentaire
seul, schéma Zod `.strict()` intouché) ; `docs/rust/PATTERNS.md` §G-3 re-attribue les 3 DTOs à
`curators_api.rs` « since the Sprint 82 Phase R split », laisse `BrowseListResponse` à `http.rs`,
**drop les numéros inline** (162/173/180/201), re-pointe le test →
`curators_api::tests::subscribe_rejects_extra_fields` ; `docs/shell/PATTERNS.md:1177` P19
`http.rs`→`curators_api.rs` (Q1=IN cohérent). **0 ref stale résiduelle** (grep docs/ des symboles
migrés = uniquement les lignes corrigées + `SPRINT_LOG.md` narration historique « NE JAMAIS
toucher »). **`frontier_closure` Phase-T = N/A CORRECT** : move wire-byte-identique, 4 DTOs migrent
verbatim avec `#[serde(deny_unknown_fields)]` (curators_api.rs:37/48/55/72), le couplage Zod reste
satisfait (`DaemonCuratorsResponseSchema`/`SubscriptionsResponseSchema` miroirs), 0 nouvelle
frontière. Test-acteur : aucun acteur nouveau ; web lit par ROUTE+FORME inchangées ; seul le
commentaire file-ancré (`daemon.ts:102`) était à corriger. Les 3 gates docs passent
(`check-frontier-contracts.sh`, `check-sharding-docs.sh`, `check-factory-docs.sh` EXIT=0) + SPDX
(`check-spdx.sh` EXIT=0, `curators_api.rs` l.1 conforme).

## Dimension 7 — Livrables + comptabilité (PASS, 1 P3 → F1)

Move-set du preflight §2-§3 LIVRÉ exact : `curators_api.rs` (611 l) + `build_router` pointe
`crate::curators_api::<handler>` (4 re-points :287/291/295/375). `curators_api.rs` conforme au
pattern sibling (SPDX + `//!` anglais + `use` groupé + `mod tests` glob `test_support`). Rien dans
le diff ne contredit `refactor(daemon): … (0 wire bump)` : Cargo delta VIDE, 4 paths byte-identiques,
0 constante wire touchée, 4 DTOs verbatim avec `#[serde(deny_unknown_fields)]` préservé. Golden
atomique préservé (`test_support.rs` git-clean, famille 9/9). **Un SEUL nit P3 (F1)** : la
décomposition narrative du décompte de lignes « −568 slices / +11 fmt / −3 use » ne boucle pas à net
−557 (somme à −560) — cf. Table des findings. Le top-line 8111→7554 et `curators_api.rs`=611 sont
exacts disque.

## Dimension 8 — Patterns + conventions (PASS)

Gabarit `*_api.rs` fidèle : SPDX l.1 (`// SPDX-License-Identifier: AGPL-3.0-or-later`) ; `//!` header
ANGLAIS (:2-15) de même forme que seed/frost/coordinator ; bloc `use` non-test = layout canonique
std/external/crate alphabétique ; `mod tests { use super::*; … use crate::test_support::*; }` groupé
identique aux siblings. **LANGUE** : grep non-ASCII sur `curators_api.rs` = uniquement tirets cadratins
(—) et 1 flèche (→) dans des phrases ANGLAISES — 0 mot français, ponctuation typographique verbatim
du source http.rs. **0 EMOJI**. **NO FUTURE-PROVENANCE** (anti STALE-PHASE-K) : grep
TODO/FIXME/will/coming/later/Phase S-T = néant, toute réf Sprint pointe en arrière ou vers la
provenance courante (82). **0 magic number introduit** (move verbatim, littéraux pré-existants du
test-code). §P70+ frontier discipline honorée (numéros inline dropés, noms ne pourrissent pas).
ENCODING UTF-8 sans BOM, LF-only. **Déviation POSITIVE vs preflight** (bon choix) : le preflight §4
esquissait le `//!` en FRANÇAIS ; l'implémenteur l'a écrit en ANGLAIS pour matcher les siblings et
satisfaire dim-8(b) — l'esquisse preflight était incohérente (« mirror seed_api » + prose FR), le
code a fait le bon geste.

## Table des findings (après vérification adversariale)

| # | Sév. | Dimension | Titre | Action |
|---|---|---|---|---|
| F1 | **P3** | 7 (Livrables) | Décomposition lignes « +11 fmt » ne boucle pas à net −557 (`http.rs:814`) — vrai contributeur fmt&bump = **+14** | **CONFIRMÉ par les 2 critics, non-bloquant** — routage vers le commit body (pas encore écrit) |

**0 P0 / 0 P1 / 0 P2 / 1 P3.**

Détail F1 : `git diff --numstat http.rs` = **+20 / −577** → net **−557** (top-line 8111→7554 EXACT
disque, `curators_api.rs`=611). La décomposition narrative annoncée « −568 slices / +11 fmt / −3
use » somme à −560 (écart 3). Décompte réel disque : deletions non-slice = 3 `use` + 4 routes
mono-lignes + 1 `struct ErrorResponse` (→pub(crate)) + 1 signature `runtime_error_to_response`
(→3 l) = 9 ; slice deletions = 577 − 9 = **568 ✓** ; insertions = 20 = 16 route-rewrap + 1
`ErrorResponse pub(crate)` + 3 helper-rewrap, toutes non-slice ; fmt&bump NET = 20 − 6 (anciennes
lignes route/signature) = **+14**, pas +11. **Boucle correcte : −568 slices / +14 fmt&bump / −3 use
= −557.** Le commit body doit énoncer cette décomposition qui boucle ; ne pas reporter « +11 fmt ».
La preuve **TOKEN_IDENTICAL ×2** reste le gate de fidélité contraignant — aucun défaut de code,
sévérité P3 non sur/sous-notée.

**Findings RÉFUTÉS : néant.** Les 2 critics ne réfutent aucun finding du bundle et n'ajoutent aucun
missed matériel. Le seul défaut surfacé (F1) est self-reporté par le bundle et confirmé par les deux
critics.

## Preuve verbatim (TOKEN_IDENTICAL ×2)

`prove_tokens.py` re-exécuté indépendamment → **PROD src=675 dst=675 equal=True ; TESTS src=1133
dst=1133 equal=True ; VERDICT TOKEN_IDENTICAL ×2**. Audit du script (critic-proof) :

- **Non-circularité** : chaque slice source scratchpad (`prod_A/B/D/E.rs`, `tests_1..4.rs`,
  `banner_default_curators.txt`) est un substring VERBATIM du `http_before.rs` sauvegardé (8111 l) —
  9/9 `slice in before` == True. Le côté source est une vraie extraction de l'ORIGINAL, pas une copie
  du dest fabriquée pour matcher.
- **Normalisation honnête + bornée** : (1) `pub(crate) async fn `→`async fn ` = replace global mais
  inoffensif (compare la SÉQUENCE COMPLÈTE de tokens, pas des counts ; un strip parasite surfacerait
  en inégalité) ; (2) le fmt-rewrap est gardé par `assert n_wraps == 1` et cible l'unique signature
  multi-ligne `default_curators` (FMT_WRAPPED→FMT_ORIGINAL). Aucune autre transformation masquée. La
  tokenisation whitespace couvre les doc-comments + la bannière Sprint 11 (`banner_default_curators.txt`
  splicée à la bonne position).

## Vérification §7.4 (suites, résultats ACTED audités)

- **Compile parfaite** (imports prédits par le preflight exacts).
- **Win** : `fmt --all --check` 0 (re-exécuté) ; `clippy --workspace --all-targets -D warnings` VERT
  (arbitre du bump `ErrorResponse`) ; crate `nexus-shell-daemon` **466/466** ; **nextest workspace
  2108/2108 0 skipped, delta ±0 EXACT** vs baseline 2108 ; doctests verts ; build release daemon OK.
- **Docker canonique `sbfb-ci`** (mount `/workspace`, `bash -c`) : `fmt` OK ; **nextest 2112** =
  2111 PASS + 1 flake env `sigint_triggers_graceful_shutdown_and_removes_running_json` re-run solo
  PASS (classe documentée Phase Q, orthogonale au move ; 2112 = 2108 Win + 4 `#[cfg(unix)]`).
- **Web** : lint 0 errors (5 warnings baseline), tsc OK, unit **412/412**, coverage FAIL 2 tests
  sous charge parallèle → PASS solo (87.27/79.01/86.02/88.59 ≥ seuils, memory `vitest_env_variance`),
  build + size verts, `scan-en-strings` clean.
- **Périmètre golden** : famille 9/9 PASS (dont `golden_http_curators_domain`, byte-identique) —
  observateur externe 0-drift JSON, chaîne handler+helper+`ErrorResponse` STAY verbatim.
- **Gates docs** : `check-frontier-contracts` OK (25 DOMAIN figés), `check-sharding-docs` OK,
  `check-factory-docs` OK, `check-spdx` OK (`curators_api.rs` l.1 conforme).
- T2 = N/A (move pur).

## Conformité aux 6 adaptations preflight + edit compile-force

1. **2 bumps `pub(crate)` IN `http.rs`** — `struct ErrorResponse` :814 `pub(crate)` champ `error`
   RESTE privé :815, `fn runtime_error_to_response` :818 `pub(crate)`, les deux RESTENT dans
   `http.rs` (5 consommateurs non-curators intacts). ✓
2. **Q1 `default_curators` = IN** — DTO :73 + handler :168 + route :375 + 2 tests co-migrés. ✓
3. **`SubscriptionsResponse`** (pas `SubscribeCuratorResponse` inexistant), :49. ✓
4. **4 tranches par NOM** — 0 leftover curator DTO/handler/route bare dans `http.rs` (grep NONE). ✓
5. **3 re-points docs** — `daemon.ts:102`, `PATTERNS.md §G-3` (numéros inline dropés + test
   re-pointé), `shell/PATTERNS.md P19` ; config-domain (`CuratorConfig`) + `SPRINT_LOG` INTACTS. ✓
6. **Slot `mod` main.rs** — `mod curators_api;` :37 entre `coordinator_api` :36 et `deploy` :38. ✓
7. **[compile-force, non prévu preflight]** — retrait de 3 `use` du mod tests de `http.rs`
   (`BlobServeCache`/`BrowseAggregator`/`CuratorRuntime`), 0 usage restant après migration de
   `default_curators_returns_configured_list`, sinon `unused_imports` sous `-D warnings` ; `clippy`
   VERT = preuve que rien de requis ne manque et rien d'inutilisé ne reste. ✓

## Note de staging / hygiène commit (routage, pas un finding)

Working tree porte 3 fichiers de recherche PO **hors-phase** (état pré-existant, intacts, à NE PAS
committer avec Phase R) : ` M .planning/research/sprint82_workflow_engine/verification_blueprint.md`
+ `?? …/workflow_agents_app_conception_ultradeep_2026-07-15.md` +
`?? …/workflow_hub_product_conception_2026-07-15.md`. Consigne au committer (discipline standing) :
**stager EXPLICITEMENT** les 7 fichiers de phase — `curators_api.rs` + `http.rs` + `main.rs` +
`web/src/api/daemon.ts` + `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md` +
`sprint82_phase_r_preflight.md` (+ ce `sprint82_phase_r_review.md`) — **jamais `git add -A`/`-a`** ;
vérifier `git diff --cached --name-only`. Chiffres à porter au body : `http.rs` 8111→**7554** (net
−557), `curators_api.rs` **611** (neuf), delta tests **±0** (Win 2108 / Docker 2112), décomposition
qui boucle **−568 slices / +14 fmt&bump / −3 use = −557** (F1).

## Prochaine étape

Codex (`gpt-5.6-sol`, `model_reasoning_effort=max`) sur les livrables de phase, puis réconciliation
→ promotion PASS-PENDING → PASS si CLEAN (ou boucle si GAP). Le fix F1 est purement commit-body
(body pas encore écrit) — pas de code à retoucher avant Codex.

## Codex reconciliation

Rapport brut : `sprint82_phase_r_codex_review.md` (output `codex exec` NON réécrit). **Audit
effectif au 3ᵉ lancement** — les 2 premiers lancements se sont terminés SANS verdict (aucun
CONFIRME/GAP rendu) : le mode sandbox `elevated` du CLI codex 0.144.1 était cassé côté machine
(« missing field sandboxPolicy », terminal muet même sur `Write-Output ok`) ; diagnostic main-thread
→ relance avec `--sandbox read-only` (politique STRICTEMENT adaptée à un audit lecture-seule ;
`~/.codex/config.toml` NON modifié — réparation du mode `elevated` à statuer côté PO). Les 2
sorties avortées ont été remplacées par le rapport effectif (même prompt, `-o` même artefact).

Résultat effectif : **7/9 CONFIRMÉ + 0 GAP + 2 PARTIEL** — les 2 PARTIEL sont des réserves
d'ENVIRONNEMENT/process, pas des défauts de code :
- **L8 (golden vert)** : Codex confirme TOUT le statique (blob `test_support.rs` == HEAD
  `80a456b4…`, 9 goldens présents, delta attributs de test `http.rs` 166→156 + `curators_api.rs`
  10 = net 0, binaire test post-build liste 9 goldens + 10 migrés, fmt + `git diff --check` verts) ;
  sa réserve = impossibilité de RE-JOUER les goldens dans SON sandbox read-only
  (`tempfile::tempdir()` → PermissionDenied, `.cargo-lock`). L'exécution verte est portée par nos
  suites (goldens 9/9 + crate 466/466 + workspace Win 2108/2108 + Docker 2112, section §7.4 supra).
- **L9 (périmètre)** : les 5 contraintes explicites CONFIRMÉES (blobs Cargo.toml/lock == HEAD,
  `web/` limité au commentaire `daemon.ts:102`, `build_router`/`authed_routes` non renommés,
  `DaemonHttpState` identique à HEAD, corps migrés identiques) ; sa réserve = la provenance des 3
  fichiers PO-research hors-phase, non établissable par un `working tree vs HEAD` seul — c'est le
  statu quo documenté (recherche PO parallèle, hors staging, cf. Note de staging supra).

**0 correction code requise → pas de boucle** (critère memory : « CLEAN ou P2/P3 documentés » —
équivalent CLEAN-code avec 2 PARTIEL-process documentés ici et dans le commit body). Le P3 F1
(décomposition −568 slices / +14 fmt&bump / −3 use = −557) est porté au commit body. Suites non
relancées : aucun fichier touché depuis leur exécution.
