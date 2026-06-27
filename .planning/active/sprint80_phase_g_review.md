# Review — Sprint 80 Phase G : backend Rust `GET /api/gates` (sbfb-factory)

**Date :** 2026-06-28
**Périmètre :** working tree NON COMMITÉ — 3 fichiers Rust (+~340/-15), 0 front, 0 route daemon.
**Orchestration :** Workflow ultracode `wf_1b699b3e-9db`, 8 agents Opus 4.8 1M (6 dimensions → adversarial → synthèse), 572k tokens.
**Fichiers revus :**
- `crates/sbfb-factory/src/gates.rs`
- `crates/sbfb-factory/src/operator_server.rs`
- `crates/sbfb-factory/tests/operator_server.rs`
- Grounding croisé (non modifiés) : `crates/sbfb-factory/src/process.rs` (`lint_planning_data:425`, `detect_sprint:122`, `LintDiagnostic:418`), `crates/sbfb-factory/src/pipeline.rs` (comparaisons publish substring intactes)
- Artefact preflight : `.planning/active/sprint80_phase_g_preflight.md`

---

## Verdict: PASS

Review sémantique + sécurité + patterns OK. Promu de PASS-PENDING à **PASS** après reconciliation du gate Codex (cf. §Codex reconciliation).

**Décompte findings (après filtrage adversarial) :** P0 = 0 · P1 = 0 · P2 = 0 · P3 = 3 (tous informationnels / durcissements optionnels, aucun bloquant). 2 des 3 P3 résolus in-phase (cf. §Résolutions in-phase).

Les 6 dimensions de review convergent à l'unanimité sur PASS-PENDING ; la passe adversariale confirme la conformité dans le code réel et rejette 2 faux-positifs P3.

---

## Résumé des 6 dimensions

| # | Dimension | Résultat |
|---|---|---|
| 1 | Correctness `gates_live_data` ligne-à-ligne (mapping errors/warnings/clean, closure `to_view`, exhaustivité registre, panics) | OK — 4 cas de mapping exacts, registre = exactement 6 `run_gate_*` couverts, 0 unwrap/panic |
| 2 | Conformité scope §6 (backend-pur, 0 daemon, Décision C, named constants minimal, 0 VERSION, 0 dérive) | OK — 3 `.rs`, route dans `authed` uniquement, `GateResult.issues` non refactoré |
| 3 | Sécurité deep (read-only/DoS, 0 input, auth, secret leak, fuite chemins) | OK — surface strictement plus restreinte que `/api/git/diff` (F), aucun nouveau threat |
| 4 | Tests-sémantique (corps lus, déterminisme, non-tautologie) | OK — fixtures tracées contre `lint_planning_data`+`detect_sprint`, split mustFix réellement exercé |
| 5 | Invariants cardinaux ligne-à-ligne + provenance doc-comments | OK — `GateResult` reste Debug-only, doc-comments descriptifs non-promissoires |
| 6 | Patterns/conventions + clippy ciblé | OK — `clippy -p sbfb-factory --all-targets -- -D warnings` CLEAN, miroir exact `handle_git_diff` |

---

## Conformité au preflight (verdict PLAN-ADAPT figé) — 5 décisions toutes tenues

| Décision figée | État | Évidence |
|---|---|---|
| 1. Registre à statut RESTITUÉ, lecture pure, 0 input, `workspace=state.root` en dur, AUCUN scan publish synchrone sur GET ; FG4/5/6/7/8→not_run, FG-CSP→not_applicable, lint-planning→passed/blocking/informational | CONFORME | `gates_live_data(&Path)` n'appelle aucun `run_gate_fg*` (FG hardcodés `NotRun`/`NotApplicable`) ; seul `lint_planning_data` tourne (read_dir non-récursif borné, early-return `ok` si absent, déjà GET-safe via `/api/lint`). `handle_gates(State(state))` passe `&state.root`, 0 param. |
| 2. Shape Décision C (NE PAS refactorer `GateResult.issues`) + `GateIssueView{message, file:Option<String>, line:Option<u32>}` peuplée depuis `LintDiagnostic` (`line` toujours None S80) ; status = enum 5 valeurs distinctes jamais aplaties ; JAMAIS `passed:bool`, JAMAIS champ racine `overall/all_passed/verdict/score` | CONFORME | `GateStatus` enum 5 variantes `#[serde(rename_all="snake_case")]` ; `GatesView{gates}` plat ; `GateResult` reste `#[derive(Debug)]` (jamais sérialisé) ; `to_view` mappe `d.file.clone()`, `line:None`. |
| 3. mustFix BLOQUANT : gate processus avec errors ET warnings → DEUX entrées (blocking=errors + informational=warnings), warnings jamais droppés | CONFORME | Deux `if !is_empty()` indépendants ; `Passed` seulement si les deux vides. Prouvé par test `..._splits_lint_errors_and_warnings...`. |
| 4. 6 noms FG + lint-planning en `pub const` aux definition points gates.rs (pas de réécriture des comparaisons pipeline.rs) | CONFORME | 7 `pub const GATE_*`, valeurs byte-identiques aux littéraux d'origine → `pipeline.rs .contains("FG4")` non cassé (substrings valides). |
| 5. V5/V6 au niveau FICHIER (`LintDiagnostic.file → GateIssueView.file`) | CONFORME | `to_view` restitue `file` ; `line` documenté carry S81. |

---

## Conformité scope / invariants cardinaux — tous tenus

- **0 verdict agrégé / 0 PASS calculé UI / l'Operator ne clôt aucun verdict** : `GatesView{gates}` est une liste plate, aucun champ racine sommant ; la route restitue, ne somme pas. Testé HTTP (`overall`/`all_passed`/`passed` absents au root) + unit.
- **États distincts jamais aplatis** : `GateStatus` 5 valeurs `snake_case`, jamais `passed:bool` ; test par entrée `g.get("passed").is_none()`.
- **Factory hors daemon** : route insérée dans le sous-routeur `authed` (sous `auth_required`), 0 route daemon ; `git diff` sur `nexus-shell-daemon*`/`coordinator`/`web` = VIDE.
- **Read-only idempotent** : `handle_gates` 0 input, `state.root` figé, aucun WalkDir+regex publish, aucun side-effect.
- **0 secret leak** : FG6 (qui embarque la valeur d'un secret dans ses issues plates côté publish) est `NotRun` à issues vides — sa fonction n'est jamais invoquée. Garde structurelle ET testée (tout gate ≠ `lint-planning` → `issues` vides). Seules les issues lint (codes/messages de discipline + nom de fichier seul) transitent.
- **Provenance doc-comments** : présents, immuables, descriptifs au présent ; aucun « Phase X will/adds/ships » (anti STALE-PHASE-K §6.12). Les mentions « carry S81 » décrivent une dette factuelle différée, pas une promesse sur ce code.

---

## Findings (P0:0 · P1:0 · P2:0 · P3:3)

| Sév. | Localisation | Description | Statut |
|---|---|---|---|
| P3 | `tests/operator_server.rs:operator_gates_endpoint` | Le contrat de sérialisation wire snake_case n'était épinglé par aucune assertion : le test HTTP vérifiait seulement que `status` est une string quelconque ; les unit tests comparent l'enum (`g.status == GateStatus::NotRun`) sans jamais sérialiser. Si `#[serde(rename_all="snake_case")]` était retiré (→ `"NotRun"`), aucun test ne le détecterait, alors que la shape spec et le front Phase H dépendent des strings exactes. | **CORRIGÉ in-phase** (cf. §Résolutions) |
| P3 | `src/gates.rs:gates_live_data` (lint-planning) | Quand le gate lint-planning a À LA FOIS errors et warnings, DEUX entrées portent le même `gate="lint-planning"` (Blocking + Informational). Comportement mustFix VOULU et figé (preflight §3.2), pas un défaut, mais un consommateur front qui indexerait par `gate` seul collisionnerait. | **DOCUMENTÉ in-phase** (cf. §Résolutions) |
| P3 | `src/gates.rs` (tests) | Le test d'idempotence byte-pour-byte (double appel `gates_live_data` → JSON égal), suggéré au preflight §2 comme EXPLICITEMENT optionnel non bloquant, n'a pas été ajouté. Fonction trivialement pure ; couvert transitivement par les 2 unit tests déterministes. | Aucune action requise S80 |

### Findings rejetés (passe adversariale)
- `Json(serde_json::json!(gates))` re-sérialise via un `Value` intermédiaire au lieu de `Json(gates)` direct : **NON-DÉFAUT** — miroir intentionnel et imposé du précédent `handle_git_diff` (Phase F) pour cohérence du routeur ; sur une route loopback authentifiée, cohérence > micro-perf. À conserver.
- Le 3e `if errors.is_empty() && warnings.is_empty()` recompute deux conditions au lieu d'un `else` : **NON-DÉFAUT** — garder deux `if` indépendants est CORRECT et VOULU pour le split errors/warnings (mustFix états distincts), 0 impact comportemental.

---

## Résolutions in-phase (post-review, avant Codex)

- **P3-1 CORRIGÉ** : `operator_gates_endpoint` asserte désormais que la réponse porte `status=="not_run"` ET `status=="not_applicable"` — deux statuts **déterministes** (FG hardcodés `not_run`, CSP toujours `not_applicable`, indépendants du repo live) → le contrat wire `#[serde(rename_all="snake_case")]` (consommé par le panneau Phase H) est épinglé. fmt + 25/25 gates tests verts après fix.
- **P3-2 DOCUMENTÉ** : doc-comment ajouté sur `GatesView` — un même `gate` peut apparaître en deux entrées (`blocking` + `informational`) ; les consommateurs (câblage Phase H) clavent par `(gate, status)`, jamais `gate` seul.
- **P3-3** : aucune action (optionnel, risque nul, couvert transitivement).

---

## Delta tests (+4 : 2 unit gates.rs + 2 HTTP)

Tous réels, déterministes là où requis, non tautologiques. Grounding : **219/219 sbfb-factory PASS** (215→219), **workspace nextest 2009→2013** ; fmt + clippy `--all-targets -D warnings` + doctest + release daemon = verts (Windows).

1. `gates_live_data_restitutes_distinct_statuses_on_a_clean_repo` (unit) — tempdir vide → `.planning/active` absent → lint `ok` → `Passed` ; asserte ≥1 `NotRun` + ≥1 `Passed` + CSP `NotApplicable` + FG carry 0 issue (secret-non-leak structurel) + len ≥ 6. Hermétique (ne dépend pas du repo live).
2. `gates_live_data_splits_lint_errors_and_warnings_into_distinct_entries` (unit) — NON tautologique, tracé contre `lint_planning_data`+`detect_sprint` : `current_sprint=10` ; `sprint5_kickoff.md` → `ORPHAN_FILE` warning ; `sprint10_phase_a_review.md`="PASS-PENDING" → `STALE_PASS_PENDING` error. Exactement 1 error + 1 warning → entrée `Blocking` + entrée `Informational` distinctes, `file` restitué, `line` None, pas de `Passed`. Insensible à l'ordre `read_dir`.
3. `operator_gates_endpoint` (HTTP) — shape-only + contrat snake_case épinglé (cf. P3-1).
4. `operator_gates_requires_auth` (HTTP) — `raw_get` sans token → 401, prouve l'enregistrement sous `authed`.

---

## Actions avant commit

1. **BLOQUANT — Gate Codex** (`codex exec`) : à lancer, joindre `codex_review.md` raw, boucler jusqu'à CLEAN ou P2/P3 documentés, puis promouvoir le verdict review→PASS.
2. **Discipline commit** : 1 commit `feat(factory-operator): Sprint 80 Phase G — <titre>`, body 9 sections, delta tests cumulé annoncé (+4), scope cuts respectés (front→Phase H, refactor `GateResult.issues`+line fine→carry S81).
3. **Vérifications lourdes** : main-thread (fmt + clippy --all-targets + nextest workspace + doctest + release Windows = FAITS verts ; dual-platform Docker AVANT push).

---

## Codex reconciliation

- **Gate Codex exécuté** : `codex exec --dangerously-bypass-approvals-and-sandbox` (model **gpt-5.5**, reasoning effort **xhigh**), output brut dans `sprint80_phase_g_codex_review.md` (NON réécrit).
- **Verdict Codex** : **6/6 livrables CONFIRME · 0 GAP · 0 PARTIEL · 0 finding P0/P1/P2/P3**. Codex a vérifié chaque livrable avec evidence `fichier:ligne` et a **relancé lui-même** `cargo test -p sbfb-factory gates_live_data` + `--test operator_server operator_gates` (tous verts).
- **Vérifications transverses Codex CONFIRMÉES** : aucun scan publish déclenché par GET (`handle_gates → gates_live_data → lint_planning_data`, 0 `run_gate_fg*`) ; aucun agrégat racine (`GatesView` ne porte que `gates`) ; aucune fuite de secret (FG6 `NotRun`, issues vides) ; doc-comments non-promissoires ; split errors/warnings non aplati ; tests déterministes/non-tautologiques.
- **GAPs corrigés** : aucun (Codex CLEAN dès le 1er tour ; boucle non nécessaire).
- **Suites relancées après correction** : sans objet (les 2 P3 résolus in-phase l'ont été AVANT Codex ; Codex a audité le code final). Grounding final : 219/219 sbfb-factory, workspace 2009→2013, fmt/clippy/doctest/release Windows verts.
- **Conclusion** : review promue à **PASS**, séquence review→Codex→commit respectée.
