# Sprint 47 — Audit plan (Sprint 46 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S46).
**Tip d'entree** : `812f3ba` (S46 Phase C, dernier feat commit).
**Documents source** : `sprint46_kickoff.md` (D1..D4) +
`sprint46_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint46_verification.md` (28/28 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint46_plan.md,
(3) sprint46_kickoff.md §D1..D4. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint46_audit_findings.md`.

## Track A — Integration tests MANDATORY (Phase A)

- [ ] A-1 : mk_state() enrichissement — verifier canary_input
  passe de None a Some(Arc<CanaryInputManager>). 2 sites dans
  http.rs (mk_state_with_mode + default_curators test).
- [ ] A-2 : 19 tests Router::oneshot() — verifier couverture des
  12 routes MANDATORY. Chaque route a au minimum 1 test.
  consent (6), files (5), canary_api (4), contributor_api (4).
- [ ] A-3 : consent tests — verifier GET returns default (level=1),
  POST error paths (level 0, level 5, invalid hex, missing id).
  Happy paths filesystem non testes (carry P2-REVIEW-A-1-S46).
- [ ] A-4 : files tests — verifier error paths (invalid sha 400,
  not found 404, too large 413). Upload happy path non teste
  (carry P2-REVIEW-A-2-S46).
- [ ] A-5 : canary tests — verifier freshness 200, inject-rate
  update, divergence empty. canary_input=Some requis.
- [ ] A-6 : contributor tests — verifier project empty list,
  invalid hex 400, envelope not found 404, invalid hex 400.

## Track B — Dette pair S44 (Phase B)

- [ ] B-1 : P2-REVIEW-A-1-S44 as_str/serde — verifier grep
  `as_str.*match` = 0 dans crate daemon et types.rs. Resolution
  par verification, pas code change.
- [ ] B-2 : P2-REVIEW-B-1-S44 kudos pagination — verifier
  KudosListQuery a limit (default 100, cap 500) + offset.
  Handler applique skip(offset).take(capped_limit). Count APRES
  skip/take (pas avant).
- [ ] B-3 : P3-REVIEW-B-2-S44 shell discover self-only — verifier
  test shell_discover_returns_self valide count=1 + own node_id.
- [ ] B-4 : P3-AUDIT-A-1-S44 pagination tests — verifier
  kudos_entries_with_limit_offset + tasks_list_with_limit
  exercent le comportement limit/offset au niveau handler.
- [ ] B-5 : P3-AUDIT-B-1-S44 diagnostic silent fallback — verifier
  0 unwrap_or_default() dans diagnostic_api.rs sur appels DB.
  active_workers_since erreur → 500.

## Track C — Integration tests routes recentes (Phase B)

- [ ] C-1 : 17 tests Router::oneshot() — verifier couverture des
  14 routes recentes. invite (3), quarantine (3), tasks (2),
  kudos (2), health (1), shell (1), diagnostic (1),
  worker_state (1) + 3 tests dette.
- [ ] C-2 : invite_create_success — verifier status 201 + id
  present dans response.
- [ ] C-3 : quarantine tests — verifier flush_not_found + drop_
  not_found retournent 404.
- [ ] C-4 : diagnostic error propagation — verifier test
  diagnostic_fairness_returns_500_on_poisoned_mutex fonctionne
  avec le nouveau code.

## Track D — Frontend direct-daemon (Phase C)

- [ ] D-1 : coordinator.ts — verifier paths mis a jour /api/v1/*.
  Routes gardees : /app/*, /project. Routes migrees : /tasks,
  /kudos, /invite, /health, /shell/discover, /worker-state.
- [ ] D-2 : error classes — verifier ApiProtocolError et
  ApiHttpError existent. Aliases retro-compat CoordinatorProtocol
  Error et CoordinatorHttpError gardes (deprecated).
- [ ] D-3 : daemon.ts — verifier callProxy() supprime, callDaemon()
  utilise. Paths : /info (pas /daemon/info), /curators (pas
  /daemon/curators), etc.
- [ ] D-4 : proxy envelope — verifier ProxyDataEnvelopeRaw,
  ProxyUnavailableEnvelope, ProxyErrorEnvelope supprimes.
  DaemonResult<T> conserve avec implementation directe.
- [ ] D-5 : schemas Zod — verifier HealthSchema, TaskRowSchema,
  KudosEntrySchema, ShellDiscoverResponseSchema mis a jour pour
  matcher les reponses daemon Rust.
- [ ] D-6 : callers — verifier KudosTab.tsx, TasksTab.tsx,
  ProjectDetail.tsx adaptes aux nouveaux champs.
- [ ] D-7 : tests — verifier daemon.test.ts et BrowsedProject.
  test.tsx mis a jour (mocks sans proxy envelope).
- [ ] D-8 : Vitest delta -1 — verifier test supprime etait le
  test proxy envelope "503 body unreadable" (chemin de code
  retire, 503 toujours couvert par autre test).

## Track E — Process / meta

- [ ] E-1 : G8 preflights 3/3 — verifier coherence
  (A + B + C tous EXECUTE, 0 DESIGN-CONFLICT).
- [ ] E-2 : scope cuts 13/13 — verifier aucun viole (diff --stat).
- [ ] E-3 : 6 carries resolus — verifier dans le diff.
- [ ] E-4 : G1 design review present — verifier scoring D1-D4.
- [ ] E-5 : sprint pair = phase dette obligatoire Phase B presente.
- [ ] E-6 : Phase reviews 3/3 presents (A + B + C).
- [ ] E-7 : Phase preflights 3/3 presents (A + B + C).

## Track F — Doc coherence

- [ ] F-1 : CLAUDE.md etat actuel — verifier S46 + compteurs.
- [ ] F-2 : SPRINT_LOG.md row S46 — verifier presente.
- [ ] F-3 : memory nexus_grid_pivot.md — verifier tip mis a jour.

---

## Carries S47

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 10+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S45 diagnostic Err path non teste | 2/3 | |
| P2-REVIEW-A-2-S45 invite ID collision multi-daemon | 2/3 | |
| P2-REVIEW-B-1-S45 modules Python suppression differee | 2/3 | |
| P3-AUDIT-B-4-S45 TOCTOU canary reload | 1/3 | |
| P2-INT-1-S46 integration tests deploy.rs + apps.rs | 1/3 | NEW |
| P2-INT-2-S46 integration test auth/token | 1/3 | NEW |
| P2-REVIEW-A-1-S46 consent happy path | 1/3 | NEW |
| P2-REVIEW-A-2-S46 files upload happy path | 1/3 | NEW |
| P2-REVIEW-B-1-S46 kudos SQL pagination | 1/3 | NEW |
| P2-REVIEW-C-1-S46 app-specific schema drift | 1/3 | NEW |
| P2-REVIEW-C-2-S46 deprecated error class aliases | 1/3 | NEW |

**Note S47 impair** : S47 est impair → pas de phase dette
obligatoire (§6.2.1 Regle 1). Mais 3 items S45 atteignent 2/3
→ deviennent MANDATORY S48 si non resolus S47.

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S47 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint46): ... avant S47 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D4 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S46 (decision sprint, pas audit)
- Routes app-specific /app/* (scope cut coordinator Python S47+)
