# Sprint 71 — Plan d'execution detaille

**Ecrit** : 2026-05-30 (apres `sprint71_kickoff.md`, avant 1er commit feat).
**Theme** : Assainissement compute + securite Factory + reconciliation
bloc off-sprint. Consolidation Arc 3.5.
**Source decisions** : `sprint71_kickoff.md §5` (D1..D8).
**Source scope** : `roadmap_v5_factory_complete_vision.md §3 (S71)`.

Dependances inter-phases (graphe) :

```
Phase 0 (audit-absorb) ──┬─> Phase A (B-1 + E2E + WIP terminal)
                         │       └─> Phase B (B-2 greedy + nettoyage)  [B reutilise l'E2E de A]
                         ├─> Phase C (securite Factory)  [independante de A/B]
                         └─> Phase D (reconciliation off-sprint)  [absorbe A/B/C dans la retro]
                                 └─> Phase E (wrap-up)  [verification de tout]
```

Phase A → Phase B : B etend l'E2E cross-process de A en ajoutant
`redundancy>1` + sorties deterministes (le harness E2E de A est
reutilise). Phase C est independante (fichiers Factory disjoints du
compute). Phase D retro-audite le code des A/B/C **et** le bloc
off-sprint. Phase E verifie l'ensemble.

---

## 1. Etat verifie a l'entree

A re-mesurer au demarrage de l'execution (apres migration chore des
docs S70). Cellules a remplir sur le SHA reel :

| Suite | Valeur entree (a mesurer) | Commande |
|-------|---------------------------|----------|
| Rust workspace (nextest) | ~1486 (baseline S70) | `cargo nextest run --workspace --locked` |
| Rust doctests | a mesurer | `cargo test --workspace --locked --doc` |
| Vitest unit | ~279 (baseline S70) | `npm run test:unit` |
| size-limit | 6/6 | `npm run size` |
| clippy warnings | a mesurer (doit etre 0) | `cargo clippy --workspace --all-targets --locked -- -D warnings` |

- **Tip entree** : `d5ddb95` (+ `d4bcceb` chore roadmap), 12 ahead origin.
- **Stash** : `stash@{0}` WIP terminal plaintext (a trancher Phase A).
- **Surfaces off-sprint a 0 test** (G6) : `terminal.rs`,
  `sprint_history.rs` (1047 lignes), `operator_server` unit,
  `process.rs`, spawn LLM/PTY.

La colonne « Observed » du `verification.md` final derive de ces
mesures d'entree.

---

## 2. Decisions Day 0 (gelees — rappel synthetique)

| D | Decision | Implication code |
|---|----------|------------------|
| D1 | Cle dispatch unique `task:` (B-1) | `dispatch_loop.rs:35` |
| D2 | Quorum greedy seed-fixe (B-2, PO-11) | chemin soumission worker→backend + flag tache verifiable |
| D3 | Gater le SSE, garder bypassPermissions (G2, PO-2) | `operator_server.rs:735-796` + contrat §4 |
| D4 | Modele `claude-opus-4-8[1m]` (G9) | `operator_server.rs:776,665` |
| D5 | Token + Host guard + CORS restreint (G7) | `operator_server.rs:80-107` |
| D6 | Timeout subprocess + diagnostic claude (G12) | `llm_bridge.rs:64-118` |
| D7 | WIP terminal tranche Phase A (G1) | `terminal.rs:27,30,133` + stash |
| D8 | Modules morts retires/clarifies (dette) | `redundancy.rs`, `build_executor.rs:126`, `process.rs:24` |

Detail + alternatives rejetees : `sprint71_kickoff.md §5`.

---

## 3. Research consulte

Sprint de consolidation — recherche interne factuelle (pas de SOTA
externe a sourcer ; aucune primitive crypto/spec nouvelle, §kickoff §1).

- Lecture-de-code des fichiers cibles (file:line dans chaque phase).
- `git log/diff 201b24d..HEAD` pour la cartographie off-sprint.
- Pattern auth correct : `daemon_client.rs:64-65` (X-SBFB-Token+Host).
- Pattern gating correct deja en place : `handle_action_run`
  `ACTION_ALLOWLIST` (`operator_server.rs:339`), `handle_chat_message`
  /`handle_chat_send` SENSITIVE_ACTIONS (`operator_server.rs:606,687`)
  — a propager au SSE (le seul chemin non garde).
- Deps off-sprint a passer au G8/S1b CVE (Phase B) : `portable-pty`,
  `async-stream`, `futures`.
- Determinisme inference greedy seed-fixe : a verifier au preflight
  Phase A/B contre la doc Ollama/llama.cpp (via context7 si dispo) —
  honneur du seed + temperature=0 + reproductibilite meme-backend.

---

## 4. Phase 0 — Audit-absorb (deviation §3 documentee)

### 0.1 Scope

Ce n'est PAS un commit feat. La session ingere `git diff 201b24d..HEAD`
(33 fichiers, +5574/-682), le traite comme dette d'entree, et produit
`sprint70_audit_findings.md` : un audit retroactif couvrant S70
(livraisons normales) ET le bloc off-sprint (~14 commits sans cycle).
En parallele (chore separe), migrer les docs S70 `active/` →
`archive/v2.1/`.

### 0.2 Fichiers touches

| Fichier | Role |
|---------|------|
| `.planning/active/sprint70_audit_findings.md` | Audit retroactif S70 + bloc off-sprint (NOUVEAU) |
| `.planning/active/sprint70_*.md` → `.planning/archive/v2.1/` | Migration chore (`git mv`) des docs S70 |

### 0.3 Tests plan

Aucun test code (phase audit/chore). Verification : le fichier
`sprint70_audit_findings.md` existe, liste les findings P0/P1/P2/P3 du
bloc off-sprint (au minimum G2/G7/G9/G12/B-1/B-2/G5/G6), avec verdict
**CONDITIONAL** (P0/P1 reconcilies in-sprint A-D, pas en fix prealable).

### 0.4 Critere acceptation

```
test -f .planning/active/sprint70_audit_findings.md
grep -c "^| P[01]" .planning/active/sprint70_audit_findings.md   # >= 6
ls .planning/archive/v2.1/sprint70_*.md   # docs S70 migres
```

### 0.5 Commit cible

Deux commits chore (pas feat) :
- `chore(planning): Sprint 71 Phase 0 — audit-absorb bloc off-sprint`
  (cree `sprint70_audit_findings.md`).
- `chore(planning): migrate S70 docs active → archive/v2.1`
  (git mv pur, ne touche que `.planning/`).

---

## 5. Phase A — Compute routing B-1 + 1er E2E + decision WIP terminal

### A.1 Scope

(1) Fixer **B-1** : la cle ecrite par le dispatch loop devient
`task:{task_id}` (alignee sur le scan worker). (2) Ecrire le **premier
E2E cross-process** coordinator→worker→Ollama→validation (B-3
inexistant) — une tache dispatchee est reellement vue et executee par
un worker reel, pas par injection in-process. (3) **Trancher le WIP
terminal** (D7) : preflight lit `stash@{0}`, puis soit drop (garde
asciicast `.cast`), soit termine le cablage plaintext + aligne les 3
sites d'extension. Etat coherent obligatoire, pas d'intermediaire.

### A.2 Fichiers touches

| Fichier | Role |
|---------|------|
| `crates/nexus-shell-daemon/src/dispatch_loop.rs` | Ligne 35 : `format!("tasks/{}", ...)` → `format!("task:{}", ...)` |
| `crates/nexus-worker-core/src/engine/runtime.rs` | Verifier que le scan `get_many_by_prefix(b"task:")` (l.833,845) reste inchange — c'est la reference d'alignement |
| `crates/nexus-test-harness/tests/compute_e2e.rs` (NOUVEAU) | E2E cross-process : daemon dispatch → worker claim → Ollama (ou backend mock deterministe) → result → validator |
| `crates/sbfb-factory/src/terminal.rs` | Decision G1 : soit inchange (stash drop), soit cablage plaintext complet + 3 sites extension (`session_log_path` l.27, `list_sessions` l.206, serve endpoint, label UI) |

### A.3 Tests plan (nommes)

1. `dispatch_loop::tests::dispatched_key_uses_task_prefix` — un
   `TaskEntry` ecrit par `run()` produit une cle commencant par `task:`.
2. `compute_e2e::dispatch_to_worker_roundtrip` — une tache soumise via
   le dispatcher est lue par le scan worker (prefixe aligne, B-1 ferme).
3. `compute_e2e::coordinator_worker_ollama_validation` — E2E complet :
   dispatch → claim → execution backend → result signe → validator
   accepte (redundancy=1). Gate sur disponibilite Ollama (skip propre
   sinon, cf. R2).
4. `terminal::tests::session_extension_consistent` (si plaintext
   retenu) — l'extension ecrite par `session_log_path` est l'extension
   filtree par `list_sessions` et servie par l'endpoint.

### A.4 Critere acceptation

```
cargo nextest run -p nexus-shell-daemon -p nexus-worker-core --locked
cargo nextest run -p nexus-test-harness --locked
cargo check -p sbfb-factory
git stash list   # stash@{0} resolu (drop ou applique+commit)
```

### A.5 Commit cible

```
fix(compute): Sprint 71 Phase A — align dispatch key + first cross-process E2E

## Contexte
B-1 : le dispatch loop ecrivait la cle `tasks/{id}` alors que le worker
scanne le prefixe `task:` — aucune tache dispatchee n'etait vue par un
worker reel. Phase A aligne la cle bout en bout et livre le premier E2E
cross-process coordinator→worker→Ollama→validation (B-3 inexistant).
Decision WIP terminal G1 tranchee (D7).
[+ Fichiers / Delta tests / Verification §7.4 / Scope cuts / G8 / Pre-launch / Codex / Carry closure]
```

(Body 9 sections complet au commit, template `commit_body_phase.txt`.)

---

## 6. Phase B — Quorum deterministe B-2 + nettoyage compute

### B.1 Scope

(1) **B-2 greedy seed-fixe** (D2, PO-11) : forcer l'inference greedy
(`temperature=0`, seed fixe) pour les taches verifiables, de sorte que
deux workers honnetes produisent le meme `result_text` → meme hash →
le quorum exact (`validator.rs:84-145`) accepte. (2) **Nettoyage
compute** (D8) : retirer/documenter `RedundancyDispatcher`
(`redundancy.rs`), trancher `execute_build` (`build_executor.rs:126`,
jamais appele), clarifier la double notion provider (`process.rs:24`
string vs runtime `LlmBackend`). (3) **G13** : passer les 3 deps
off-sprint au preflight G8/S1b CVE.

### B.2 Fichiers touches

| Fichier | Role |
|---------|------|
| `crates/nexus-worker-core/src/engine/runtime.rs` | Chemin soumission worker→backend : forcer greedy+seed pour taches `verifiable` |
| `crates/nexus-core-rs/src/task.rs` | Flag/champ `verifiable` (ou mode deterministe) sur `Task` — runtime tolerance `#[serde(default)]` |
| `crates/nexus-coordinator-rs/src/validator.rs` | Inchange logiquement (le compte exact `r.sha256` devient correct une fois les sorties deterministes) — ajouter doc/assert |
| `crates/nexus-coordinator-rs/src/redundancy.rs` | Retrait (mort) ou DEPRECATED documente si appelant S75 nommable |
| `crates/nexus-worker-core/src/build_executor.rs` | `execute_build` (l.126) : cabler ou retirer selon preflight |
| `crates/sbfb-factory/src/process.rs` | Ligne 24 `PROVIDERS` : doc de la distinction provider-prompt vs backend-execution, ou unification si redondant |
| `docs/rust/PATTERNS.md` | Documenter la decision provider/backend + deps off-sprint validees |

### B.3 Tests plan (nommes)

1. `runtime::tests::verifiable_task_uses_greedy_seed` — une tache
   `verifiable` configure le backend en greedy + seed fixe.
2. `compute_e2e::two_honest_workers_same_hash` — deux workers honnetes
   sur la meme tache deterministe produisent le meme `result_text`/hash.
3. `compute_e2e::quorum_accepts_deterministic_redundancy` — quorum
   `redundancy_factor=2` atteint et accepte sur sortie deterministe
   (etend l'E2E de Phase A).
4. `validator::tests::quorum_rejects_nondeterministic_divergence` —
   sorties divergentes (non-deterministes) → quorum n'accepte pas
   (preserve la propriete de rejet des outliers).
5. (preflight G13) — pas un test runtime : trace dans le preflight.md
   que `portable-pty`/`async-stream`/`futures` sont passees au scan CVE.

### B.4 Critere acceptation

```
cargo nextest run -p nexus-worker-core -p nexus-coordinator-rs -p nexus-test-harness --locked
cargo clippy -p nexus-coordinator-rs -p nexus-worker-core --all-targets --locked -- -D warnings
# RedundancyDispatcher retire OU DEPRECATED documente :
grep -rn "RedundancyDispatcher" crates/   # 0 hit hors DEPRECATED/historique
```

### B.5 Commit cible

```
fix(compute): Sprint 71 Phase B — deterministic quorum (greedy seed) + dead module cleanup

## Contexte
B-2 : le quorum comparait le hash exact de result_text (validator.rs:115)
— deux workers honnetes en sampling etaient rejetes. PO-11 : greedy
seed-fixe pour taches verifiables rend le quorum exact utilisable.
Nettoyage : RedundancyDispatcher mort, execute_build jamais appele,
double notion provider clarifiee. Deps off-sprint passees au G8/S1b.
[+ 9 sections]
```

---

## 7. Phase C — Securite Factory G2/G7/G9/G12 + amendement contrat

### C.1 Scope

(1) **G2** (D3) : appliquer le filtre `SENSITIVE_ACTIONS` au SSE
`handle_chat_stream` (`operator_server.rs:735-796`) qui court-circuite
aujourd'hui le gating — un message contenant `shell`/`commit`/`push`/
`PASS` renvoie `requires_gate` au lieu de spawner un agent
`bypassPermissions` autonome. (2) **G9** (D4) : modele
`claude-opus-4-8[1m]` au lieu de `"sonnet"` (`operator_server.rs:776`),
cabler `ChatSendRequest.model` (l.665), clarifier les stubs. (3) **G7**
(D5) : CORS restreint + token `X-SBFB-Token` + Host guard
(`operator_server.rs:80-107`, pattern `daemon_client.rs:64-65`). (4)
**G12** (D6) : timeout subprocess + diagnostic resolution `claude`
(`llm_bridge.rs:64-118`). (5) **Amender** `RRV_FACTORY_CONTRACT.md §4`
(PO-2 : autoriser le pilotage agent local privilegie **gate**).

### C.2 Fichiers touches

| Fichier | Role |
|---------|------|
| `crates/sbfb-factory/src/operator_server.rs` | l.735-796 filtre SSE (G2) ; l.776+665 modele opus-4-8 (G9) ; l.80-107 CorsLayer restreint + middleware token + Host guard (G7) |
| `crates/sbfb-factory/src/llm_bridge.rs` | l.64-118 timeout subprocess + pre-spawn resolution check + diagnostic (G12) |
| `crates/sbfb-factory/src/daemon_client.rs` | Reference du pattern token+Host (l.64-65) — lecture, pas modif |
| `docs/agent/RRV_FACTORY_CONTRACT.md` | §4 amendement : pilotage agent local privilegie **gate** explicitement autorise (PO-2) |
| `web/` (front Operator, si present) | Bootstrap token serveur→front (transmission du token genere) |

### C.3 Tests plan (nommes)

1. `operator_server::tests::sse_gates_sensitive_action` — un dernier
   message user contenant `commit`/`push`/`shell`/`PASS` → le SSE
   renvoie `requires_gate`, ne spawn PAS d'agent.
2. `operator_server::tests::sse_allows_nonsensitive` — un message
   benin passe (happy-path PO-2 preserve).
3. `operator_server::tests::chat_stream_uses_opus_model` — le modele
   transmis au spawn est `claude-opus-4-8[1m]` (defaut) / `req.model`.
4. `operator_server::tests::server_rejects_missing_token` — requete
   sans `X-SBFB-Token` → 401/403.
5. `operator_server::tests::server_rejects_foreign_host` — Host non
   loopback → rejet.
6. `operator_server::tests::cors_restricts_origin` — origine etrangere
   refusee (plus de `Any`).
7. `llm_bridge::tests::spawn_times_out` — subprocess qui depasse le
   timeout est tue, le stream renvoie une erreur bornee.
8. `llm_bridge::tests::missing_claude_diagnostic` — `claude` absent du
   PATH → message diagnostic clair (pas `Failed to spawn` opaque).

### C.4 Critere acceptation

```
cargo nextest run -p sbfb-factory --locked
cargo clippy -p sbfb-factory --all-targets --locked -- -D warnings
grep -n '"sonnet"' crates/sbfb-factory/src/operator_server.rs   # 0 hit
grep -n "allow_origin(Any)" crates/sbfb-factory/src/operator_server.rs   # 0 hit
grep -n "Factory Operator" docs/agent/RRV_FACTORY_CONTRACT.md   # §4 amende
```

### C.5 Commit cible

```
fix(factory): Sprint 71 Phase C — gate SSE + opus-4-8 + token auth + spawn timeout

## Contexte
Bloc securite Factory (off-sprint). G2 : le SSE court-circuitait
SENSITIVE_ACTIONS et spawnait bypassPermissions sans gate. G9 : modele
hardcode sonnet (viole la regle modele). G7 : CORS Any + zero auth sur
un serveur qui ecrit/spawn. G12 : spawn sans timeout. Contrat Operator
§4 amende (PO-2 : pilotage agent local gate autorise).
[+ 9 sections]
```

---

## 8. Phase D — Reconciliation process du bloc off-sprint G5/G6

### D.1 Scope

(1) **G5** : produire les artefacts process manquants du bloc
off-sprint (~14 commits, +5500 lignes) — retro-review (dimensions
§4.5), retro-Codex (exec brut), retro-audit, documentes dans
`.planning/active/`. (2) **G6** : ecrire la couverture de tests des
surfaces off-sprint a 0 test (`terminal.rs`, `sprint_history.rs`
1047 lignes, `operator_server` unit residuels, `process.rs`, spawn
LLM/PTY non couverts par Phase C). C'est la phase la plus large et le
declencheur potentiel du scindage (kickoff §11, R8).

### D.2 Fichiers touches

| Fichier | Role |
|---------|------|
| `.planning/active/sprint71_offsprint_retro_review.md` | Retro-review 11 dimensions du bloc off-sprint (NOUVEAU) |
| `.planning/active/sprint71_offsprint_codex_review.md` | Sortie brute `codex exec -o` du retro-audit (NOUVEAU, non reecrit) |
| `crates/sbfb-factory/tests/terminal.rs` (NOUVEAU) | Tests `terminal.rs` (session log, PTY mock, extension) |
| `crates/sbfb-factory/tests/sprint_history.rs` (NOUVEAU) | Tests `sprint_history.rs` (parsing, endpoint) |
| `crates/sbfb-factory/tests/operator_server.rs` | Etendre : endpoints chat/sprint-history/diff non couverts Phase C |
| `crates/sbfb-factory/src/process.rs` | Tests unit `process.rs` (resolve_kind, repo_root, providers) |

### D.3 Tests plan (nommes)

1. `terminal::session_log_roundtrip` — un buffer PTY ecrit et relit
   correctement (asciicast ou plaintext selon D7).
2. `terminal::list_sessions_filters_correct_extension` — `list_sessions`
   ne renvoie que les fichiers de l'extension active.
3. `sprint_history::parses_active_and_archive` — l'endpoint
   sprint-history renvoie les sprints actifs + archives.
4. `sprint_history::diff_endpoint_returns_inline_code` — l'endpoint
   diff renvoie le contenu attendu.
5. `operator_server::chat_session_lifecycle` — create session → send →
   stream → log (sans spawn reel, mock).
6. `process::resolve_kind_aliases` — les alias (`review`→`phase-review`)
   resolvent ; `providers` liste correcte.
7. `process::repo_root_resolves` — `repo_root` renvoie la racine git.

### D.4 Critere acceptation

```
cargo nextest run -p sbfb-factory --locked
test -f .planning/active/sprint71_offsprint_retro_review.md
test -f .planning/active/sprint71_offsprint_codex_review.md
# Chaque surface off-sprint a >= 1 test :
cargo nextest list -p sbfb-factory | grep -E "terminal|sprint_history|process"
```

### D.5 Commit cible

```
test(factory): Sprint 71 Phase D — reconcile off-sprint block (retro-review + coverage)

## Contexte
G5/G6 : ~14 commits off-sprint (+5500 lignes) sans cycle ni tests.
Phase D produit la retro-review (11 dimensions), le retro-Codex (exec
brut), le retro-audit, et la couverture de tests des surfaces a 0 test
(terminal, sprint_history, operator_server unit, process, spawn).
[+ 9 sections]
```

**Note bascule (R8)** : si cette phase deborde (kickoff §11), elle se
ferme sur une reconciliation **partielle** (retro-review + retro-Codex
faits, tests prioritaires terminal+process) et la completion (tests
sprint_history exhaustifs + retro-audit complet) passe en S71-bis/S72
sur arbitrage PO. Decision au moment ou la phase deborde, pas avant.

---

## 9. Phase E — Wrap-up

### E.1 Scope

Produire les deux livrables obligatoires de cloture + PATTERNS +
memory.

### E.2 Fichiers touches

| Fichier | Role |
|---------|------|
| `.planning/active/sprint71_verification.md` | Self-report fail-fast rempli (colonne Observed) |
| `.planning/active/sprint71_audit_plan.md` | Feuille de route audit pour la session fraiche S72 |
| `docs/rust/PATTERNS.md` | Patterns compute (cle dispatch, greedy quorum) + tech debt residuel |
| `docs/shell/PATTERNS.md` | Patterns Factory securite (gate SSE, token loopback) |
| `nexus_grid_pivot.md` + `MEMORY.md` (memory) | Tip S71, compteurs tests, carries P2+ |

### E.3 Tests plan

Pas de nouveau test. La phase rejoue la **fail-fast checklist** (§10).

### E.4 Critere acceptation

```
# fail-fast checklist 100% verte (voir §10)
test -f .planning/active/sprint71_verification.md
test -f .planning/active/sprint71_audit_plan.md
```

### E.5 Commit cible

```
docs(sprint71): verification + audit plan for Sprint 72

## Contexte
Cloture S71. verification.md (fail-fast rempli) + audit_plan.md
(feuille de route S72) + PATTERNS (compute + Factory securite) + memory.
Reconciliation bloc off-sprint validee, arc S72-S76 debloque.
[+ 9 sections]
```

---

## 10. Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|-------|----------|---------|----------|
| 1 | fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | clippy workspace | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | |
| 3 | nextest workspace | `cargo nextest run --workspace --locked` | 0 fail | |
| 4 | doctests | `cargo test --workspace --locked --doc` | 0 fail | |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | OK | |
| 6 | B-1 cle alignee | `grep -n 'task:' crates/nexus-shell-daemon/src/dispatch_loop.rs` | `task:` present, `tasks/` absent | |
| 7 | B-1 round-trip | `cargo nextest run -p nexus-shell-daemon -E 'test(dispatched_key)'` | vert | |
| 8 | B-3 E2E cross-process | `cargo nextest run -p nexus-test-harness -E 'test(coordinator_worker)'` | vert (ou skip Ollama documente) | |
| 9 | B-2 greedy seed | `cargo nextest run -p nexus-worker-core -E 'test(verifiable_task_uses_greedy)'` | vert | |
| 10 | B-2 deux workers meme hash | `cargo nextest run -p nexus-test-harness -E 'test(two_honest_workers)'` | vert | |
| 11 | B-2 quorum accepte deterministe | `cargo nextest run -p nexus-test-harness -E 'test(quorum_accepts_deterministic)'` | vert | |
| 12 | quorum rejette divergence | `cargo nextest run -p nexus-coordinator-rs -E 'test(quorum_rejects)'` | vert | |
| 13 | RedundancyDispatcher resolu | `grep -rn 'RedundancyDispatcher' crates/` | 0 hit vivant | |
| 14 | execute_build tranche | `grep -n 'execute_build' crates/nexus-worker-core/src/build_executor.rs` | cable ou retire | |
| 15 | provider/backend documente | `grep -n 'provider' docs/rust/PATTERNS.md` | distinction documentee | |
| 16 | deps off-sprint au preflight | trace `sprint71_phase_B_preflight.md` | 3 deps scannees CVE | |
| 17 | G2 SSE gate sensible | `cargo nextest run -p sbfb-factory -E 'test(sse_gates_sensitive)'` | vert | |
| 18 | G2 SSE happy-path | `cargo nextest run -p sbfb-factory -E 'test(sse_allows_nonsensitive)'` | vert | |
| 19 | G9 modele opus | `grep -n '"sonnet"' crates/sbfb-factory/src/operator_server.rs` | 0 hit | |
| 20 | G9 modele cable | `cargo nextest run -p sbfb-factory -E 'test(chat_stream_uses_opus)'` | vert | |
| 21 | G7 token requis | `cargo nextest run -p sbfb-factory -E 'test(rejects_missing_token)'` | vert | |
| 22 | G7 Host guard | `cargo nextest run -p sbfb-factory -E 'test(rejects_foreign_host)'` | vert | |
| 23 | G7 CORS restreint | `grep -n 'allow_origin(Any)' crates/sbfb-factory/src/operator_server.rs` | 0 hit | |
| 24 | G12 timeout | `cargo nextest run -p sbfb-factory -E 'test(spawn_times_out)'` | vert | |
| 25 | G12 diagnostic | `cargo nextest run -p sbfb-factory -E 'test(missing_claude_diagnostic)'` | vert | |
| 26 | Contrat §4 amende | `grep -n 'gate' docs/agent/RRV_FACTORY_CONTRACT.md` | pilotage local gate explicite | |
| 27 | G6 terminal teste | `cargo nextest list -p sbfb-factory | grep terminal` | >= 1 test | |
| 28 | G6 sprint_history teste | `cargo nextest list -p sbfb-factory | grep sprint_history` | >= 1 test | |
| 29 | G6 process teste | `cargo nextest list -p sbfb-factory | grep process` | >= 1 test | |
| 30 | G5 retro-review present | `test -f .planning/active/sprint71_offsprint_retro_review.md` | existe | |
| 31 | G5 retro-Codex present | `test -f .planning/active/sprint71_offsprint_codex_review.md` | sortie brute codex | |
| 32 | Vitest front (si touche) | `npm run test:unit` | 0 fail | |
| 33 | size-limit (si touche) | `npm run size` | 6/6 | |
| 34 | scan-en-strings (si front touche) | `bash scripts/scan-en-strings.sh` | 0 string EN | |

(32-34 conditionnels : seulement si le front Operator est touche en
Phase C/D.)

---

## 11. Git plan (commits atomiques attendus)

| Ordre | Type | Titre |
|-------|------|-------|
| 1 | chore(planning) | Sprint 71 Phase 0 — audit-absorb bloc off-sprint |
| 2 | chore(planning) | migrate S70 docs active → archive/v2.1 |
| 3 | fix(compute) | Sprint 71 Phase A — align dispatch key + first cross-process E2E |
| 4 | fix(compute) | Sprint 71 Phase B — deterministic quorum (greedy seed) + dead module cleanup |
| 5 | fix(factory) | Sprint 71 Phase C — gate SSE + opus-4-8 + token auth + spawn timeout |
| 6 | test(factory) | Sprint 71 Phase D — reconcile off-sprint block (retro-review + coverage) |
| 7 | docs(sprint71) | verification + audit plan for Sprint 72 |

Chaque commit feat/fix/test passe la chaine preflight → review
PASS-PENDING → Codex → reconciliation → review PASS → commit body 9
sections (§4.3). Chore planning exempte de preflight/Codex (ne touche
que `.planning/`), mais Phase 0 audit-absorb produit un artefact
d'audit substantiel.

---

## 12. Scope cuts (copie kickoff §8)

1. ProviderRouter multi-LLM → S72.
2. Chat Factory cable routage reseau → S72.
3. Pont feed-distant → reindex FTS5 a chaud → S73.
4. Enrichissement `SearchResult` (triplet provenance) → S73.
5. Barre recherche shell cablee → S73.
6. Decision SearchManifest → S73.
7. Commandes `sbfb-factory search/open/fork` → S74.
8. Notion projet cible distinct nexus (G17) → S74.
9. Templates etendus (react, pyodide) → S74.
10. GPU partage volontaire cross-machine → S75.
11. Quorum redundancy>1 cross-MACHINE reel (B-3 leve) → S75.
12. Sharding pipeline gros modele → S76 STRETCH.
13. logprobs/watermark verification → V2 compute (post-S75).
14. Dashboard contributeur kudos per-task → S75.
15. @dev index tree-sitter → S71+ post-Gate 1.
16. Packaging produit Factory (PO-4) → S74.

L'auditeur grep chaque scope cut : aucune ligne de code S71 ne doit le
toucher (sinon P1).

---

## 13. Risks (R1..R8)

Reprise de `sprint71_kickoff.md §10`. Chaque mitigation est verifiable :

| # | Risque | Mitigation verifiable |
|---|--------|------------------------|
| R1 | Greedy non bit-exact cross-GPU | Test B-3 sur machine dev meme-backend ; limite documentee PATTERNS ; cross-GPU → S75 |
| R2 | E2E flaky (Ollama requis) | Gate disponibilite Ollama (skip propre), seed fixe ; prerequis runtime documente |
| R3 | Charge trop lourde 1 sprint | Phases elargies + point de bascule §11 kickoff |
| R4 | B-1 casse un test injectant `tasks/` | Grep tous sites cle avant fix ; aligner tests `task:` |
| R5 | Gating SSE casse discussion autonome | Gater seulement messages SENSITIVE_ACTIONS ; test happy-path |
| R6 | Token casse front off-sprint | Transmettre token au front meme commit ; test bootstrap |
| R7 | Retrait execute_build/RedundancyDispatcher | DEPRECATED+ROADMAP_COMMITMENTS si appelant S75, sinon retrait ; decision preflight B |
| R8 | Reconciliation D sous-estimee | Phase D la plus large ; declencheur scindage §11 ; reconciliation partielle fallback |

---

## 14. Checkpoint de cloture

S71 est ferme quand :

- [ ] Fail-fast checklist (§10) 100% verte (rows applicables).
- [ ] 7 commits atomiques landed (§11), bodies 9 sections.
- [ ] `sprint70_audit_findings.md` produit (Phase 0).
- [ ] Docs S70 migres `active/` → `archive/v2.1/`.
- [ ] `sprint71_verification.md` + `sprint71_audit_plan.md` ecrits.
- [ ] `sprint71_offsprint_retro_review.md` + `_codex_review.md` ecrits.
- [ ] `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md` a jour.
- [ ] `RRV_FACTORY_CONTRACT.md §4` amende (PO-2).
- [ ] Memory `nexus_grid_pivot.md` + `MEMORY.md` a jour.
- [ ] B-1 ferme (tache route reellement), B-2 quorum deterministe
      accepte, securite Factory gatee, bloc off-sprint reconcilie.
- [ ] Stash `stash@{0}` resolu (drop ou applique+commit).
- [ ] Decision arbitrage §11 actee (mono-sprint OU scindage documente).
