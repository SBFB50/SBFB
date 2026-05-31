# Sprint 70 — Audit Findings (AUDIT-ABSORB, Phase 0 S71)

**Auditeur** : session de cadrage S71 (2026-05-30). Audit **independant** :
le bloc off-sprint a ete ecrit par le PO (FlowUP) hors session agent — la
cartographie multi-agents 2026-05-30 EST l'audit independant (l'auditeur
n'a pas ecrit ce code).
**Type** : **audit-absorb** — deviation documentee du pattern §3.1
(kickoff S71 §3, PO-3 2026-05-30). Couvre S70 (livraisons normales,
deja vertes a la cloture `201b24d`) ET le **bloc off-sprint**
(`201b24d..d5ddb95`, ~14 commits, +5574/-682, 33 fichiers, zero cycle).
**Tip audite** : `d5ddb95` (+ commits `chore(planning)` S71 `d4bcceb`,
`e92e7d8`, `1190d18`).
**Source intake** : cartographie surface Factory + couche compute/RRV
(2 workflows multi-agents, voir `sprint71_intake.md §2`).

---

## 1. Verdict global

**CONDITIONAL PASS — reconciliation in-sprint (deviation).**

Particularite de l'audit-absorb : les P0/P1 ne sont PAS fixes par des
commits `fix(sprint70)` prealables a la Phase A (pattern §3.1 standard).
Ils sont **le scope meme des phases A-D de S71** : auditer puis re-corriger
en deux passes dupliquerait le travail sur les memes fichiers
(`operator_server.rs`, `llm_bridge.rs`, `validator.rs`, `dispatch_loop.rs`).
On **absorbe** la dette en entree, on la corrige dans les phases, l'audit
gate S71 de sortie valide la fermeture.

- **5 P0** : B-1, B-2, G1, G2, G8 → phases A/B/C.
- **8 P1** : B-3, G4, G5, G6, G7, G9, G10, G12 → phases A/C/D (+ S72-S74 pour
  G4/G10 produit).
- **P2/P3** : tech debt loggee, repartie S71 dette / S72+.

**S70 lui-meme** (commits jusqu'a `201b24d`) : livraisons normales,
verifications vertes a la cloture, 7 phases A-G avec cycle complet. Aucun
finding bloquant **propre a S70** — la dette est concentree dans le bloc
**off-sprint** posterieur.

---

## 2. Findings par track

### Track COMPUTE (fonde tout l'arc — phases A/B)

- **B-1 / P0** — Cle de dispatch disjointe. `dispatch_loop.rs:35` ecrit
  `tasks/{id}` ; le worker lit `get_many_by_prefix("task:")`
  (`runtime.rs:833,845`). **Aucune tache dispatchee n'est vue par un worker
  reel** — le flux compute n'a jamais tourne qu'en test in-process par
  injection directe. → **Phase A** (D1, fix + test round-trip de cle).
- **B-2 / P0** — Quorum hash-exact sur sortie stochastique.
  `validate_quorum` compte `r.sha256` = hash de `result_text`
  (`validator.rs:115`) ; deux workers honnetes en sampling divergent → tous
  rejetes. → **Phase B** (D2, greedy seed-fixe, PO-11).
- **B-3 / P1** — Zero E2E cross-process compute. `worker` e2e = CLI only ;
  `multi_daemon` = feed/blob only. Le chemin coordinator→worker→Ollama→
  validation n'a aucune couverture cross-process. → **Phase A** (1er E2E).
- **Dette compute / P2** — `RedundancyDispatcher` (`redundancy.rs`) module
  mort ; `execute_build` (`build_executor.rs:126`) jamais appele ; double
  notion « provider » (string `process.rs:24` vs runtime `LlmBackend`). →
  **Phase B** (D8 : retirer/cabler/documenter).

### Track SECURITE FACTORY (off-sprint — phase C)

- **G2 / P0** — Pilotage agent non-gate. `llm_bridge.rs:80`
  `--permission-mode bypassPermissions` ; le SSE `handle_chat_stream`
  (`operator_server.rs:735-796`) court-circuite le filtre SENSITIVE_ACTIONS
  applique par `handle_chat_message`/`handle_chat_send`/`handle_action_run`.
  Un agent autonome bypassPermissions = session shell/commit/push non gatee,
  contredit le contrat Operator §4. → **Phase C** (D3 + amendement contrat).
- **G9 / P1** — Modele hardcode `"sonnet"` (`operator_server.rs:776`),
  viole la regle modele (`claude-opus-4-8[1m]`). `handle_chat_message` stub
  « integration pending » ; `handle_chat_send` log only. → **Phase C** (D4).
- **G7 / P1** — CORS `Any` + zero auth (`operator_server.rs:87-90`) sur un
  serveur qui ecrit des fichiers et spawn des process. Pattern correct
  (`daemon_client.rs:64-65` X-SBFB-Token + Host guard) non applique. CSRF/
  DNS-rebinding local. → **Phase C** (D5).
- **G12 / P1** — `spawn_claude_stream` sans timeout ; `claude.cmd`/`claude`
  resolu via PATH sans verification ni diagnostic (`llm_bridge.rs:74,107`).
  → **Phase C** (D6).

### Track BUILD / WIP (phase A)

- **G1 / P0** — WIP terminal `.cast`→`.log` incomplet dans `stash@{0}`
  (cassait le build : `write_asciicast_*` supprimes mais appeles,
  `PlainTextWriter` non cable, extension lue `.cast` vs ecrite `.log`).
  Neutralise par stash (HEAD compile), mais flotte. → **Phase A** (D7,
  trancher : jeter le stash et garder l'asciicast par defaut).

### Track PROCESS / RECONCILIATION (phase D)

- **G5 / P1** — Bloc de ~14 commits off-sprint sans cycle
  (preflight/review/Codex/audit/body 9 sections). Surfaces : `terminal.rs`,
  `llm_bridge.rs`, `sprint_history.rs`, endpoints chat/sprint-history/diff,
  expansions `operator_server`/`AgentChat`/`SprintHistory`. → **Phase D**
  (retro-review + retro-Codex + retro-audit).
- **G6 / P1** — Surfaces off-sprint quasi non-testees : `terminal.rs` 0,
  `sprint_history.rs` 0/1047 lignes (parsing git+markdown fragile),
  `operator_server` unit 0, `process.rs` 0, spawn LLM/PTY 0. → **Phase D**.
- **G13 / P2** — 3 deps workspace off-sprint (`portable-pty`,
  `async-stream`, `futures`) non passees au preflight G8/S1b CVE.
  `portable-pty` = spawn de process, surface securite a revalider. →
  **Phase B** (preflight).

### Track UI / VIEWER / PRODUIT (differe S72-S74)

- **G4 / P1** — Viewer casse (double mismatch JSON `entries`/`projects` +
  ProofCard nested/plat). → **S74** (atelier).
- **G8 / P0** — 3 mismatches contrat JSON UI (`json.prompt` vs `content`,
  `json.pack`) → PhaseAssistant/AgentTransfer/ContextPackBuilder cassees a
  l'usage. → **S72** (contrats UI) — note : P0 d'usage UI mais hors socle
  compute, ne bloque pas S71.
- **G10 / P1** — Socle `factory-ui/readonly` orphelin (importe par personne)
  ; `operator/api-client.ts` mort. → **S72-S74**.
- **G3/G17 / P1-P2** — Boucle produit non cablee, Operator opere le repo
  nexus pas un projet cible. → **S74** (atelier).
- **G18/G19/G23 / P2** — SprintOverview `test_counts` inexistant ; i18n
  incomplet pages off-sprint ; jargon UX en CTA. → **S72-S74**.

### Track CLI / GATES (differe)

- **G14 / P2** — secret_scanner minimal (3 patterns). → **S72+**.
- **G15 / P2** — canonical_bytes duplique (T-NN+3, seuil 3-reports). →
  **S72+** (extraction nexus-core-rs).
- **G16 / P1** — pas d'E2E publish avec daemon reel. → **S72+**.
- **G20/G21/G22 / P3** — git diff sans `--`, unwrap/lock poison, FG4 toujours
  passant. → **S71 dette / S72+**.

---

## 3. Findings list par severite

| ID | Sev | Titre | Phase de fermeture |
|----|-----|-------|--------------------|
| B-1 | P0 | Cle dispatch disjointe | S71 A |
| B-2 | P0 | Quorum hash-exact stochastique | S71 B |
| G1 | P0 | WIP terminal casse (stash) | S71 A |
| G2 | P0 | Pilotage agent non-gate (SSE) | S71 C |
| G8 | P0 | Mismatches contrat JSON UI | S72 |
| B-3 | P1 | Zero E2E cross-process compute | S71 A |
| G4 | P1 | Viewer casse | S74 |
| G5 | P1 | Bloc off-sprint non reconcilie | S71 D |
| G6 | P1 | Surfaces off-sprint non testees | S71 D |
| G7 | P1 | CORS Any + zero auth | S71 C |
| G9 | P1 | Modele hardcode sonnet | S71 C |
| G10 | P1 | Socle readonly orphelin | S72-S74 |
| G12 | P1 | Spawn sans timeout/diagnostic | S71 C |
| G16 | P1 | Pas d'E2E publish | S72+ |
| G3/G13/G14/G15/G17/G18/G19/G23 | P2 | (voir §2) | reparti |
| G20/G21/G22 | P3 | (voir §2) | dette |

---

## 4. Commits fix attendus

**Aucun `fix(sprint70)` prealable.** Deviation audit-absorb (§1) : les P0/P1
sont reconcilies **dans les phases S71 A-D**, pas avant la Phase A. Les
P0/P1 produit (G4/G8/G10/G3) sont differes S72-S74 (hors socle compute, ne
bloquent pas l'assainissement S71).

---

## 5. P2 a logger en tech debt

G13 (deps off-sprint preflight, Phase B), G14 (secret_scanner), G15
(canonical bytes T-NN+3), G17 (Operator repo cible), G18/G19/G23 (UI/UX) —
loggees dans PATTERNS.md au fil des phases / au wrap-up S71 (§Phase E).

## 6. P3 laisses sans action immediate

G20 (git diff `--`), G21 (unwrap/lock poison), G22 (FG4 informatif) —
repris en dette S71 si une phase touche le fichier, sinon S72+.

## 7. Notes on audit completeness

- Couvert : le diff complet `201b24d..d5ddb95` (33 fichiers), la couche
  compute (worker/coordinator/dispatch/validator), la securite Factory, le
  Viewer/socle, le parcours E2E produit, le contrat gele.
- Non couvert (assume) : la qualite des docs `community` (CHATONS, README
  Codeberg) du bloc off-sprint — contenu editorial, hors scope technique.
  Les 426 lignes modifiees de `nexus-sprint-kickoff.md` (agent) off-sprint :
  notees, a re-verifier si le process agent en depend (non bloquant S71).
- Limite : les compteurs de tests exacts d'entree n'ont pas ete re-mesures
  (le crate sbfb-factory etait casse par le WIP avant stash) — re-mesure au
  demarrage Phase A (`plan §1`).
