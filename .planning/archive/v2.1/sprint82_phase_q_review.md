# Sprint 82 Phase Q — Review (Workflow)

## Contexte + méthode

- Diff reviewé : working tree vs HEAD `1aa7a0f` (Phase P `frost_api.rs` DONE) — split du
  domaine `coordinator` de `http.rs` vers un NOUVEAU module
  `crates/nexus-shell-daemon/src/coordinator_api.rs` (move PUR verbatim, discipline étendue
  PO-10 ; preflight PLAN-ADAPT — 2ᵉ split de la classe O, une fixture promue).
- Date : 2026-07-16.
- Orchestration : Workflow ultracode — 8 dimensions + vérificateurs adversariaux + synthèse ;
  chaque claim load-bearing **re-vérifié sur disque** ici (oracle indépendant, PAS le script
  fourni). 0 agent en erreur.
- Périmètre code + doc (7 fichiers trackés + 1 neuf) :
  - `http.rs` (**8951 → 8111 l, net −840** ; numstat `+13/−853`),
  - `coordinator_api.rs` (NEUF, **864 l** — SPDX + `//!` + 5 `use` + 4 `pub(crate) async fn`
    + 13 `#[tokio::test]` + 2 helpers co-migrés, **0 DTO local**),
  - `test_support.rs` (**817 → 840 l, +23** add-only : `make_test_submission` promue
    `pub(crate)` + doc-comment 3 l),
  - `main.rs` (**+1** `mod coordinator_api;` au slot alpha),
  - `validator.rs` (3/3 re-points doc), `PATTERNS.md` (1/1), `THREAT_MODEL.md` (1/1).
- Preflight : PLAN-ADAPT (`sprint82_phase_q_preflight.md`) — approche confirmée + 3 adaptations
  matérielles : (1) promotion `make_test_submission` → `test_support.rs pub(crate)`
  (compiler-forcée E0425 par 3 tests `tasks_api` STAY), (2) ré-honnêteté in-phase de 5 refs docs
  file-ancrées, (3) arbitrage C1 (`authed_routes` vs `build_router`) dans le doc-comment.

## Verdict: PASS

8 dimensions **PASS**. **0 P0, 0 P1, 0 P2 — 1 P3 confirmé** après vérification adversariale :
F1 (Dimension 8) le doc-comment `//!` **double-attribuait** le nudge local-worker (label inline
« S76-A » vs parenthèse « (Sprint 35 Phase B) » qui, par la convention du reste de l'en-tête,
se rattache au nom qui la précède) — **cosmétique, sans impact compile/wire/route ni violation
P70/P72** — **CORRIGÉ IN-PHASE** avant Codex (`coordinator_api.rs:7-8` : « Task submit
(Sprint 35 Phase B) runs the input guardrail chain BEFORE dispatch plus the S76-A local-worker
nudge », `cargo fmt --all --check` re-vérifié 0) ; Codex a audité l'état post-fix.

Les **4 findings de staging** remontés par les dimensions 1/4/Livrables (P3+P2+P2+P3) sont tous
**RÉFUTÉS à NOT-A-FINDING** par la vérification adversariale : ils décrivent un état de working
tree PRÉ-EXISTANT hors-scope (les 3 fichiers de recherche PO `.planning/research/*`) ou une
hygiène de commit HYPOTHÉTIQUE (aucun commit joué), PAS un défaut du diff de Phase Q — déjà
documentés et arbitrés (mémoire « Hors-phase PO intacts »). Retenus uniquement comme **note de
routage** pour le committer (cf. §Note de staging). Le finding « figer 8111 vs ~8102 » est
également NOT-A-FINDING (le `~8102` vient du brief de tâche éphémère, n'existe dans AUCUN
artefact staged ; le disque = 8111, chiffre à porter au body).

Promotion PASS-PENDING → **PASS** effectuée après réconciliation Codex (cf. §Codex
reconciliation : **CLEAN round 1, 9/9 CONFIRMÉ, 0 GAP**).

## Dimension 1 — Fidélité verbatim (PASS)

Move BYTE-PARFAIT, re-prouvé indépendamment sur disque (`git show HEAD` + `diff`) :

- **Bloc PROD** : HEAD `http.rs:2217-2516` (300 l) == `coordinator_api.rs:26-325` après l'unique
  transformation inverse `pub(crate) async fn coordinator_` → `async fn coordinator_`
  (exactement 4×) — **`diff` vide, 300/300 lignes, 0 édition parasite**.
- **Bloc TESTS** : HEAD `http.rs:5627-5630` (bannière) + `5651-6173` == `coordinator_api.rs:337-863`
  — **`diff` vide, 527/527 lignes**. 13 `#[tokio::test]` + `make_result_entry`/`_with_text`
  co-migrés verbatim.
- **`make_test_submission`** : HEAD `http.rs:5631-5649` == `test_support.rs:822-840` modulo
  dedent-4 + préfixe `pub(crate)` ; le seul ajout est le doc-comment 3 lignes. Ordre/liste des
  15 champs préservé (`project_id` premier … `required_runtime`).
- **`http.rs` = exactement 3 hunks** (route re-point + suppression prod + suppression cluster
  tests), 0 hunk parasite ; **13 insertions** = les 4 routes re-pointées éclatées multi-lignes
  par rustfmt (le chemin complet `crate::coordinator_api::` dépasse la largeur), **paths
  byte-identiques**. `main.rs` = 1 ligne ; `validator.rs`/`PATTERNS.md`/`THREAT_MODEL.md` =
  re-points token-seuls.

Doc-comment anglais conforme aux splits frères N/O/P (pas une déviation malgré le brouillon
français du preflight §4). SPDX ligne 1, EOF newline présents.

## Dimension 2 — Suites §7.4 (PASS)

Vérifications rapides re-exécutées et confirmées : `cargo fmt --all --check` **exit 0** ; 13
`#[tokio::test]` dans `coordinator_api.rs` (1:1 avec la liste preflight §3), 0 `#[test]` sync ;
0 def résiduelle de handler dans `http.rs` (seul survivant `coordinator_health_ok` @6107 =
domaine health_api, nom trompeur, STAY légitime). Conservation exacte des attributs HEAD→working
(total `#[tokio::test]` 168→168, `#[test]` 18→18) — **exactement 13 migrés, 0 perdu/ajouté** :
substantie le delta ±0 au grain fichier. `golden_http_coordinator_domain` (`test_support.rs:502`)
RESTE atomique et vert (diff `test_support` = add-only +23, borne 815+, golden intouché). Les 2
flakes env sont correctement qualifiés (sigint Docker `tests/e2e.rs` = process+signal+fs orthogonal
au move, famille `running_json` documentée J/P, solo PASS ; vitest web sur suite intouchée).

## Dimension 3 — Branch coverage sémantique (PASS)

Les 13 tests router-driven co-migrés exercent toutes les branches principales des 4 handlers
(submit_task : input-guardrail/PII 400 + poisoned 500 ; submit_result : accepted-persist +
bad_signature + task_not_found + task_not_pending + guardrail-reject-rien-persisté [CARRY-2
terminal] + persist-après-guardrail + kudos-credit + poisoned 500 ; get_kudos : json + poisoned
500 ; verify_chain : valid). ZÉRO test coordinator orphelin dans `http.rs::tests` (scan par URI :
`/api/v1/tasks/submit`, `/api/v1/results/submit`, `/api/v1/kudos/{project_id}[/verify]` n'apparaissent
qu'aux défs de route ; les hits kudos résiduels = `/entries` + `/leaderboard` → `kudos_api`, STAY).
Les lacunes de couverture (submit_task 200-success via oneshot, AwaitingQuorum, QuorumRejected,
chemins Err→500) sont **PRÉ-EXISTANTES au HEAD** — un move pur ne peut ni les créer ni les combler,
hors périmètre Q.

## Dimension 4 — Scope + staging (PASS)

Move intra-crate scope-propre : **0 delta Cargo.toml/Cargo.lock/feature** (vérifié), 0 changement
de route-path, 0 rename `build_router`/`authed_routes`, 0 édit de `DaemonHttpState`, 0 wire bump
réel, 5 re-points docs exactement comme déclarés. Les 4 routes restent chaînées dans
`authed_routes` (tier T0). Aucun fichier `web/`, `sbfb-factory`, `docs/factory`, `WIRING_SPEC.md`
ni scratchpad `.py` dans le diff. Le `_VERSION` du diff = 2 déplacements purs de
`version: nexus_core_rs::task::TASK_FORMAT_VERSION` (helpers test), équilibrés à l'identique — pas
un bump. **Seul point : 3 fichiers de recherche PO hors-phase dans le working tree** (voir §Note de
staging) — état pré-existant, à exclure du commit, PAS un défaut du diff.

## Dimension 5 — Research grounding vs preflight (PASS)

Les 3 adaptations matérielles du preflight PLAN-ADAPT sont appliquées à l'identique sur disque :
(1) `make_test_submission` promue `test_support.rs:822 pub(crate)`, copie locale supprimée, 3 tests
`tasks_api` STAY (`http.rs:5957/6276/6278`) la résolvent via le glob `use crate::test_support::*`
pré-existant — E0425 pré-empté, confirmé empiriquement par le delta ±0 ; (2) les 5 refs file-ancrées
re-pointées `http.rs`→`coordinator_api.rs`, les 4 symbole-seules (`validator.rs:39/178`,
`PATTERNS.md:3390`, `THREAT_MODEL.md:938`) + `SPRINT_LOG.md:26` laissées INTACTES à raison (le move
préserve le NOM du symbole) ; (3) l'arbitrage C1 tranché dans le `//!` avec la formulation POSITIVE
S1a « inside `authed_routes` », jamais la négation INV-3 réfutée. Plan d'imports honoré (5 `use`
prod ; bloc test `KeyPair` SEUL, pas `create_node` → `unused_imports` pré-empté).

## Dimension 6 — Sécurité deep (PASS)

Tous les blocs load-bearing sécurité migrent INTACTS (prouvé par la byte-identité) : guardrail
input-avant-dispatch (`default_input_chain().run` avant `dispatcher::submit_task`) ; guardrail
output-AVANT-persist S73-A/D5 (`validate_result_pre_guardrail` → `default_output_chain().run` AVANT
`validate_result_post_guardrail`) + tripwire TERMINAL CARRY-2 ; bridge feed
`result_event_tx.send(NewResult)` placé après persist/credit ; credit sanity-bound forwardé au
chokepoint `kudos_ledger::credit`. Le dedup S76-D `(worker_pubkey,task_id)` vit dans `result_sync.rs`
(HORS diff, intouché) — correctement en aval du `send` migré. Les 4 routes restent sous
`auth_required` (`from_fn_with_state`) + `cors_layer` → **tier T0 inchangé** ; 0 nouvelle surface,
0 changement de status-code ni de forme JSON (golden concordant).

## Dimension 6bis — Docs-contrat frontière (test-acteur) (PASS)

`frontier_closure` = **N/A** confirmé : 0 signature DTO touchée (types requête `TaskSubmission`/
`ResultEntry` dans des crates INTOUCHÉES, 0 delta Cargo). `git diff -- web/` VIDE + les 4 paths
byte-identiques ⇒ les consommateurs `coordinator.ts` (submitTask/submitComputeTask/verifyKudos,
path+Zod-couplés à des formes produites hors `http.rs`) inaffectés → **0 index Phase T**. Les
commentaires de provenance in-code pointent uniquement le passé immuable (grep
`phase-t|will|future|todo|fixme` = 0). Les 3 gates docs re-passés verts : `check-spdx` (357 fichiers,
exit 0), `check-sharding-docs` (exit 0), `check-frontier-contracts` (25 DOMAIN figés, exit 0).

## Dimension 7 — Livrables + comptabilité (PASS)

Move-set du preflight §2-§3 LIVRÉ exact (voir §Comptabilité). 4 handlers `pub(crate) async fn` +
13 tests + 2 helpers co-migrés + 1 fixture promue ; 4 re-points route (`http.rs:407/411/415/419`)
+ 5 refs docs ; 0 helper prod, 0 bump `pub(crate)` de symbole `http.rs`, 0 re-point `runtime.rs`,
0 DTO local (livrable plan « + DTO » satisfait vacuously — correction factuelle preflight §1.5).
La borne plan `http.rs:3722-4023` était STALE (pré-N), correctement re-dérivée par NOM à
`2217-2516` — divergence attendue et documentée, pas un défaut.

## Dimension 8 — Patterns + conventions (PASS, 1 P3 → F1)

Gabarit `*_api.rs` fidèle (SPDX + `//!` + ordre `use` std→axum→crate + `mod tests` avec glob
`test_support`) ; import test `KeyPair` seul (piège `create_node` pré-empté) ; anglais intégral,
0 emoji ; `pub(crate)` minimal (4 handlers, 0 DTO exposé, `make_result_entry*` restés privés) ;
promotion fixture conforme au pattern Phase O (doc-comment bi-consommateur) ; slot alphabétique
correct ; provenance P70-conforme (passé immuable uniquement). **Un SEUL nit P3 (F1)** :
`coordinator_api.rs:7-8` « the S76-A local-worker nudge (Sprint 35 Phase B) » — la parenthèse, par
la convention du reste de l'en-tête (`… invariant (Sprint 73 Phase A, D5)`, `kudos read (Sprint 36
Phase C)`), se rattache au nom immédiatement précédent (le nudge), ce qui contredit le label inline
« S76-A » ; les corps confirment la séparation réelle (nudge = S76-A `:82`, handler task-submit =
S35-B bannière `:27`). En-tête NOUVELLEMENT écrit cette phase → artefact de phase, non hérité,
non pré-arbitré (le preflight §4 proposait une formulation plus propre). Cosmétique, non-bloquant.

## Table des findings (après vérification adversariale)

| # | Sév. | Dimension | Titre | Action |
|---|---|---|---|---|
| F1 | **P3** | 8 (Patterns) | Doc-comment `//!` double-attribue le nudge local-worker (« S76-A » inline vs « (Sprint 35 Phase B) » en parenthèse — la parenthèse se rattache au nudge par la convention de l'en-tête) | **CONFIRMÉ, non-bloquant** — à corriger in-phase de préférence : réordonner pour que « (Sprint 35 Phase B) » se rattache à « Task submit » et que « S76-A » reste l'unique ancre du nudge |

**0 P0 / 0 P1 / 0 P2 / 1 P3.**

**Findings RÉFUTÉS à NOT-A-FINDING** (conservés en note de routage, pas comptabilisés) :
- Staging des 3 fichiers de recherche PO hors-phase (remonté 3× : Dim 1 P3, Dim 4 P2, Livrables P2)
  → état working-tree PRÉ-EXISTANT hors-scope + hygiène de commit HYPOTHÉTIQUE (aucun commit joué),
  déjà documenté/arbitré → cf. §Note de staging.
- « Figer 8111 vs ~8102 » (Livrables P3) → le `~8102` vient du brief de tâche éphémère, absent de
  tout artefact staged ; le chiffre disque = **8111 (net −840)**, à porter au body — routage, pas
  défaut.

## Comptabilité réconciliée (vérifiée disque)

- **`http.rs` 8951 → 8111, net −840** — réconcilié 2 façons concordantes : `wc -l` disque
  (8951→8111) ; `git diff --numstat` (`+13/−853` = −840). Les 13 insertions = rustfmt éclate les 4
  routes re-pointées en multi-lignes (chemin complet plus long) ; les −853 = slice prod
  (`2217-2516`, 300 l) + cluster tests (`5627-6173`, dont `make_test_submission` déplacée) +
  churn route, collapses de blank inclus.
- **`coordinator_api.rs` = 864 l (neuf)** — SPDX l.1, `//!` `:2-16`, 5 `use` `:18-24`, 4 handlers
  `pub(crate) async fn` (`:30/120/253/299`), 13 `#[tokio::test]`, 2 helpers (`make_result_entry`
  `:341`, `make_result_entry_with_text` `:345`), **0 struct/enum local**.
- **`test_support.rs` 817 → 840, +23** add-only (1 blank + 3 doc + 19 fn) ; golden atomique
  (`:502`) intouché.
- **`main.rs` +1** (`mod coordinator_api;` @36, slot alpha `contributor`<`coordinator`<`deploy`).
- **Docs** : `validator.rs` 3/3 (`:404/421/448`), `PATTERNS.md` 1/1 (`:3578` §P61.2),
  `THREAT_MODEL.md` 1/1 (`:1117` kudos sanity-bound) — tous `http.rs`→`coordinator_api.rs`.
- **Tests : delta ±0** (13 relocalisés intra-crate, même binaire). Attributs conservés 168→168
  `#[tokio::test]`. Baseline **Win 2108 / Docker 2112** préservée.
- **Preuve token-level** : `verify_phase_q_proof.py` (scratchpad) — PROD 795 tok / TESTS 1140 tok /
  `make_test_submission` 40 tok TOKEN_IDENTICAL ; corroboré ici par `diff` VIDE sur les 3 blocs.

## Vérification §7.4 (suites, résultats main thread audités)

- Compile **parfaite 1ᵉʳ coup** (imports prédits par le preflight exacts, y compris `KeyPair`
  seul).
- **Win** : `fmt --all --check` **0** (re-exécuté ici, exit 0) ; `clippy --workspace --all-targets
  -D warnings` **0** ; **nextest 2108/2108 EXACT** (delta ±0) ; doctests OK ; build release daemon
  OK.
- **Docker canonique `sbfb-ci`** (mount `/workspace`, `bash -c`) : `fmt` 0 ; `clippy` 0 ;
  **nextest 2112/2112 no-fail-fast 0 fail** (run déchargé) ; 1 flake e2e
  `sigint_triggers_graceful_shutdown_and_removes_running_json` au 1ᵉʳ run sous charge 4-blocs
  (`tests/e2e.rs`, famille `running_json` documentée J/P, orthogonale au move, solo PASS).
- **Web** : lint 0, tsc 0, Vitest 412/412 (1 flake env sous charge, re-run 412 PASS), coverage
  87.27/79.01/86.02/88.59 (≥ seuils), build, size 6/6, scan-en clean.
- **Operator** : lint 0, Vitest 201/201, build, size, 6 gates clean.
- **Gates docs** : `check-sharding-docs` OK, `check-frontier-contracts` OK, `check-spdx` OK
  (re-vérifiés).
- **Périmètre golden** : famille 9/9 PASS (dont `golden_http_coordinator_domain`, byte-identique).
- T2 = N/A (move pur, plan).

## Note de staging / hygiène commit (routage, pas un finding)

Le working tree porte 3 fichiers de recherche PO **hors-phase** (état pré-existant, intacts, à NE
PAS committer avec Phase Q) :
- ` M .planning/research/sprint82_workflow_engine/verification_blueprint.md` (bannière
  « ARCHITECTURE SUPERSEDED », 0 contenu coordinator),
- `?? .planning/research/workflow_agents_app_conception_ultradeep_2026-07-15.md`,
- `?? .planning/research/workflow_hub_product_conception_2026-07-15.md`.

Consigne au committer (discipline standing, déjà appliquée aux 5 phases précédentes) : **stager
EXPLICITEMENT** les 8 fichiers de phase — `coordinator_api.rs` + `http.rs` + `test_support.rs` +
`main.rs` + `validator.rs` + `PATTERNS.md` + `THREAT_MODEL.md` + l'artefact
`sprint82_phase_q_preflight.md` (+ ce `sprint82_phase_q_review.md`) — **jamais `git add -A`/`-a`** ;
vérifier `git diff --cached --name-only` : le blueprint + les 2 `workflow_*.md` ne doivent PAS y
figurer. Chiffres à porter au body : `http.rs` 8951→**8111** (net −840), `coordinator_api.rs` 864
(neuf), `test_support.rs` 817→840 (+23), delta tests **±0** (Win 2108 / Docker 2112).

## Codex reconciliation

- Joué le 2026-07-16 : `Get-Content .git/CODEX_SPRINT82_PHASE_Q.txt -Raw | codex exec -m
  gpt-5.6-sol -c model_reasoning_effort=max -o .planning/active/sprint82_phase_q_codex_review.md`
  (output BRUT conservé tel quel, 9 livrables audités).
- **Verdict Codex : CLEAN round 1 — 9/9 CONFIRMÉ, 0 GAP, 0 PARTIEL** (8ᵉ round-1-clean S82).
  Audit indépendant : re-run `cargo nextest run --workspace --locked` **2108/2108 PASS delta 0
  exact** + `cargo fmt --all --check` PASS ; comparaison des blocs migrés contre
  `git show HEAD` avec **SHA-256 identiques** (PROD 300/300 l `58e61722…`, TESTS 527/527 l
  `8a3d3b9f…`) après la seule normalisation `pub(crate)` autorisée ; contrôles de résidus 0/0/0 ;
  `DaemonHttpState` 128/128 l identiques ; Cargo.toml/Cargo.lock/web/ zéro chemin modifié.
- Séquencement : le fix P3 F1 (doc-comment, cosmétique) a été appliqué AVANT le run Codex ;
  Codex a donc audité l'état final committable. 0 GAP à corriger → aucune boucle requise.
- Précision Codex (livrable 4, non-GAP) : les tests STAY nommés sont 2 fonctions
  (`task_result_route_404_then_text_on_completed`, `tasks_list_with_limit`) pour 3 appels à la
  fixture (`http.rs:5957/6276/6278`) — cohérent avec l'inventaire préflight (usages 6797/7116/7118
  pré-move, renumérotés post-move).
