# Sprint 71 — Intake / Briefing (pre-kickoff)

**Ecrit** : 2026-05-30 (session de cadrage PO).
**Pour** : l'agent `nexus-sprint-kickoff` (produit kickoff + plan +
design_review a partir de ce doc) et toute session fraiche.
**Roadmap CANON** : `.planning/roadmap_v5_factory_complete_vision.md`.
**Theme S71** : Assainissement compute + securite + reconciliation du bloc
off-sprint. Sprint de **consolidation d'ouverture d'arc** (Arc 3.5 Factory
Complete Vision) — phases elargies, zero feature speculative, chaque item
reference un bug/carry/test-manquant reel.

---

## 1. Situation de process NON-STANDARD (a lire en premier)

- **Tip** : `d5ddb95`, **11 commits ahead d'origin/master** (rien pousse).
- **S70 clos** a `201b24d`, MAIS **~14 commits feat/fix(factory) +
  docs(community) ont lande APRES, hors cycle sprint** (`e26d9f2..d5ddb95`,
  `git diff --stat 201b24d..HEAD` = 33 fichiers, +5574/-682). Zero
  preflight/review/Codex/audit/body-9-sections.
- **`sprint70_audit_findings.md` n'existe pas** — l'audit gate de cloture
  S70 n'a jamais tourne, et le tip a depuis diverge.
- `.planning/active/` contient encore tous les docs S70 + `sprint71_audit_plan.md`
  (a migrer vers `archive/v2.1/` a l'ouverture).
- **WIP terminal** : `crates/sbfb-factory/src/terminal.rs` (refactor
  `.cast`→`.log` incomplet qui cassait le build) est dans **`git
  stash@{0}`** « WIP terminal plaintext-logging refactor (incomplete) --
  S71 Factory ». Tree propre, `cargo check -p sbfb-factory` OK au HEAD.

**Strategie de reconciliation (validee PO — PO-3 reconciliation complete)** :
ne PAS rejouer l'audit gate S70 sur un tip pollue. A la place :

- **Phase 0 = audit-absorb** : la session fraiche S71 ingere le diff
  `201b24d..HEAD`, le traite comme dette d'entree, ecrit
  `sprint70_audit_findings.md` (audit retroactif du bloc off-sprint +
  S70).
- Les phases S71 produisent la **reconciliation** : retro-review (11
  dimensions) + retro-Codex (exec brut) + retro-audit du code off-sprint,
  + la couverture de tests manquante, documentes dans `.planning/active/`.
- L'**audit gate S71 de sortie** valide a la fois la reconciliation du
  bloc off-sprint ET les nouvelles phases.

---

## 2. Inventaire des gaps pertinents S71 (cartographie 2026-05-30)

### Compute (bloquants — fondent tout l'arc)
- **B-1 / P0** : cle dispatch `tasks/{id}` (`dispatch_loop.rs:35`) vs
  worker `task:` (`runtime.rs:833,845`) → aucune tache ne route en prod.
- **B-2 / P0** : quorum compare le hash exact de `result_text`
  (`validator.rs:115`) → workers honnetes en sampling tous rejetes.
  Resoudre par greedy seed-fixe (PO-11) ; logprobs/watermark inerte
  (`task.rs:339-364`, `logprobs_hash=32 zeros`).
- **B-3 / P1** : aucun E2E cross-process compute (worker e2e = CLI only,
  multi_daemon = feed/blob only).
- **Dette** : `RedundancyDispatcher` module mort (`redundancy.rs`) ;
  `execute_build` jamais appele (`build_executor.rs:126`) ; double notion
  « provider » (string `process.rs:24` vs runtime `LlmBackend`).

### Securite Factory (contrat §4 — gating)
- **G2 / P0** : `bypassPermissions` (`llm_bridge.rs:80`) + terminal PTY
  spawnent un agent autonome non-gate ; le SSE `handle_chat_stream`
  (`operator_server.rs:735-796`) court-circuite le filtre SENSITIVE_ACTIONS.
- **G9 / P1** : modele hardcode `sonnet` (`operator_server.rs:776`) —
  viole la regle modele (`opus-4-8`). `handle_chat_message` = stub
  « integration pending » ; `handle_chat_send` log only.
- **G7 / P1** : CORS `Any` + zero auth sur serveur qui ecrit des fichiers
  et spawn des process (`operator_server.rs:88`). Pattern correct non
  applique : `daemon_client.rs:64-65` (X-SBFB-Token + Host guard).
- **G12 / P1** : `spawn_claude_stream` sans timeout ; `claude.cmd`/`claude`
  resolu via PATH sans verification ni diagnostic.

### Reconciliation / tests (bloc off-sprint)
- **G5 / P1** : ~14 commits off-sprint sans cycle (terminal, llm_bridge,
  sprint_history, endpoints chat/sprint-history/diff, expansions UI).
- **G6 / P1** : surfaces off-sprint quasi non-testees (`terminal.rs` 0,
  `sprint_history.rs` 0/1047 lignes, `operator_server` unit 0,
  `process.rs` 0, spawn LLM/PTY 0).
- **G13 / P2** : 3 deps workspace off-sprint (`portable-pty`, `async-stream`,
  `futures`) non passees au preflight G8/S1b CVE.

### A trancher en S71 phase A
- **G1 / P0** : WIP terminal `stash@{0}` — finir le cablage
  `PlainTextWriter` (+ aligner extension lue/ecrite : `list_sessions`
  filtre `.cast`, serve endpoint sert `{name}.cast`, label UI `.cast`)
  OU jeter le stash et garder l'asciicast. Ne jamais laisser flotter.

### Hors S71 (vont S72+ — ne PAS traiter ici)
- G3/G4/G8/G10/G17/G18/G19/G23 (atelier, Viewer, contrats UI, socle
  readonly, UX intentions) → S72-S74.
- G14 (secret_scanner), G15 (canonical bytes T-NN+3), G16 (E2E publish),
  G20/G21/G22 → repartis S71 (dette pair) / S72+.

---

## 3. Decisions PO actees

Voir `roadmap_v5 §1` (PO-1 a PO-14). Pour S71, les pertinentes :
PO-2 (gater le pilotage agent), PO-3 (reconciliation complete), PO-11
(greedy seed-fixe), PO-12 (kudos non-monetaire), PO-14 (Claude pilote).

---

## 4. Outline de phases propose pour S71 (a formaliser par le kickoff)

Indicatif — le kickoff/plan affine. Consolidation = phases elargies.

- **Phase 0** — audit-absorb du bloc off-sprint + `sprint70_audit_findings.md`.
- **Phase A** — Fix B-1 (cle dispatch) + 1er E2E cross-process
  coordinator→worker→Ollama→validation. Decision WIP terminal (G1).
- **Phase B** — B-2 validation stochastique (greedy seed-fixe) +
  reconcilier notion provider + retirer modules morts
  (RedundancyDispatcher, execute_build clarifie). Deps off-sprint au
  preflight (G13).
- **Phase C** — Securite Factory : gater bypassPermissions / SSE
  SENSITIVE_ACTIONS (G2), modele opus-4-8 (G9), CORS+token (G7), timeout +
  diagnostic claude (G12).
- **Phase D** — Reconciliation process du bloc off-sprint : retro-review +
  retro-Codex + retro-audit + couverture de tests des surfaces off-sprint
  (G5, G6).
- **Phase E** — wrap-up : verification + `sprint71_audit_plan.md` (pour
  S72) + PATTERNS + memory.

Note : si la charge B-1/B-3 (compute E2E cross-machine) s'avere trop
lourde pour cohabiter avec la reconciliation Factory, le kickoff peut
scinder en deux sprints (assainissement compute / reconciliation Factory)
— a arbitrer par l'agent kickoff avec le PO.

---

## 5. Contraintes process rappel

- Amender le **contrat Operator** (`docs/agent/RRV_FACTORY_CONTRACT.md §4`)
  pour autoriser explicitement le pilotage agent local privilegie **gate**
  (PO-2) — sinon les phases securite contredisent le contrat.
- Pre-launch : rien pousse (11 ahead) → reconciliation locale libre, pas
  de bump version wire.
- Migrer les docs S70 `active/` → `archive/v2.1/` a l'ouverture (chore).
