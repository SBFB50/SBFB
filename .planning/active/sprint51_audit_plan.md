# Sprint 51 — Audit plan (Sprint 50 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S50).
**Tip d'entree** : `7358bd4` (S50 Phase B, dernier feat commit).
**Documents source** : `sprint50_kickoff.md` (D1..D4) +
`sprint50_plan.md` (§Phase A, §Phase B) +
`sprint50_verification.md` (22/22 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint50_plan.md,
(3) sprint50_kickoff.md §D1..D4. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint50_audit_findings.md`.

## Track A — Dispatch JoinHandle (Phase A)

- [ ] A-1 : verifier que `DaemonRuntime` a un champ
  `dispatch_handle: Option<JoinHandle<()>>`.
- [ ] A-2 : verifier que `start()` stocke le handle du
  `tokio::spawn(dispatch_loop::run(...))` dans ce champ.
- [ ] A-3 : verifier que `shutdown()` join le handle apres le
  HTTP serve join (channel close → loop exit → join).

## Track B — CLI handler integration tests (Phase A)

- [ ] B-1 : verifier 4 tests dans `handler_tests` module de
  main.rs : init_creates_db, invite_create_list_revoke_cycle,
  quarantine_list_empty, capability_enable_disable_cycle.
- [ ] B-2 : verifier que les tests utilisent tempdir + DB reelle.
- [ ] B-3 : verifier les 4 tests passent dans nextest.

## Track C — Python deletion (Phase B)

- [ ] C-1 : verifier que `packages/nexus-coordinator/`,
  `packages/nexus-sdk/`, `packages/nexus-app-gov/`,
  `crates/nexus-core-py/` n'existent plus.
- [ ] C-2 : verifier que `Cargo.toml` n'a plus `nexus-core-py`
  dans members ni `pyo3` dans workspace.dependencies.
- [ ] C-3 : verifier que `pyproject.toml` n'a plus `packages/*`
  ni `crates/nexus-core-py` dans workspace members.
- [ ] C-4 : verifier `cargo build --workspace --locked` compile
  sans reference a pyo3.

## Track D — Frontend cleanup (Phase B)

- [ ] D-1 : verifier que `useAppEvents.ts`, `AppTabPage.tsx`,
  `cross_lang.test.ts`, `schema_v2_cross_lang.test.ts` n'existent
  plus dans web/src/.
- [ ] D-2 : verifier que App.tsx n'a plus la route
  `/app/:appName/tabs/:tabName`.
- [ ] D-3 : verifier que `.size-limit.json` n'a plus l'entry
  TabViewRenderer.
- [ ] D-4 : verifier tsc + Vitest + build + size passent.

## Track E — Process / meta

- [ ] E-1 : G8 preflights 2/2 presents (A + B tous EXECUTE).
- [ ] E-2 : scope cuts 8/8 respectes (diff --stat).
- [ ] E-3 : Phase reviews 2/2 presents (A + B).
- [ ] E-4 : Delta tests cumule coherent : Rust +4, Vitest -17,
  Python -528.
- [ ] E-5 : Sprint pair — phase dette obligatoire Phase A (confirme).

## Track F — Doc coherence

- [ ] F-1 : CLAUDE.md — sections Python supprimees, compteurs
  mis a jour, structure crates sans nexus-core-py.
- [ ] F-2 : SPRINT_LOG.md row S50 presente.
- [ ] F-3 : memory nexus_grid_pivot.md — tip mis a jour.

---

## Carries S51

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-A-1-S48 canary reload size cap | 2/3 | S48 review |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 2/3 | S48 review |
| P2-AUDIT-A-1-S48 carry doc accuracy | 2/3 | S48 audit |
| P2-REVIEW-A-1-S50 dispatch join order | 1/3 | NEW S50 |
| P2-REVIEW-B-1-S50 nexus/ legacy monolith | 1/3 | NEW S50 |

**Note S51 impair** : pas de phase dette obligatoire. 3 items a
2/3 — si non adresses S51, ils passent a 3/3 et deviennent
MANDATORY pour S52 (§6.2.1 Regle 2).

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S51 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint50): ... avant S51 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D4 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S50 (decision sprint, pas audit)
- nexus/ legacy monolith (carry P2-REVIEW-B-1-S50, pas audit)
