# Sprint 48 — Audit plan (Sprint 47 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S47).
**Tip d'entree** : `3641871` (S47 Phase C, dernier feat commit).
**Documents source** : `sprint47_kickoff.md` (D1..D4) +
`sprint47_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint47_verification.md` (28/28 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint47_plan.md,
(3) sprint47_kickoff.md §D1..D4. Ne PAS lire le code source
avant d'avoir parcouru les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint47_audit_findings.md`.

## Track A — S45 carries resolution (Phase A)

- [ ] A-1 : diagnostic Err path — verifier que le test
  diagnostic_fairness_returns_500_on_corrupted_db existe dans
  http.rs et qu'il drop la table kudos puis verifie 500.
- [ ] A-2 : invite ID collision fix — verifier que le format ID
  dans invite_api.rs est `inv-{node_id_8}-{ts}-{seq}`. Verifier
  que state.node_id est accessible dans le handler.
- [ ] A-3 : Python modules suppression — verifier que les 7
  modules supprimes (fairness, forge, honeypot, pow_counter,
  provenance, redundancy, watermark_detector) n'ont plus aucun
  import dans le codebase. Verifier que les 7 fichiers tests
  associes sont aussi supprimes.
- [ ] A-4 : execute_batch_raw — verifier que cette methode est
  #[doc(hidden)] dans db.rs. Evaluer le risque d'exposition SQL.

## Track B — Integration tests 5 routes (Phase B)

- [ ] B-1 : deploy_private happy path — verifier le test avec
  fixture zip (make_test_zip) + BlobsClient reel + 200 + hash.
- [ ] B-2 : deploy error paths — verifier invalid zip → 400,
  non-HTTP URL → 400, invalid SHA → 400.
- [ ] B-3 : apps integration — verifier list empty, list with
  entries (BrowseAggregator.add_direct_entry), get by id, get
  unknown → 404.
- [ ] B-4 : auth_token — verifier GET /auth/token retourne 200
  avec TEST_TOKEN.

## Track C — Happy path tests + aliases cleanup (Phase C)

- [ ] C-1 : consent happy path — verifier 4 tests (set level,
  get persisted, whitelist add, whitelist remove). Verifier
  utilisation SBFB_HOME env var pour isolation.
- [ ] C-2 : files happy path — verifier 3 tests (upload +
  manifest + stream). Verifier SHA-256 correcte et contenu stream.
- [ ] C-3 : deprecated aliases — verifier suppression des 3
  exports (CoordinatorProtocolError, CoordinatorHttpError,
  normalizeCoordinatorUrl). Grep codebase pour 0 ref restante.
- [ ] C-4 : callers migres — verifier AddCoordinatorDialog.tsx,
  projectStore.ts, tests → ApiProtocolError, ApiHttpError,
  normalizeApiUrl.

## Track D — Process / meta

- [ ] D-1 : G8 preflights 3/3 presents (A + B + C tous EXECUTE).
- [ ] D-2 : scope cuts 11/11 respectes (diff --stat).
- [ ] D-3 : 8 carries resolus verifies dans le diff.
- [ ] D-4 : Phase reviews 3/3 presents (A + B + C).
- [ ] D-5 : Delta tests cumule coherent : Rust +17, Python -65.

## Track E — Doc coherence

- [ ] E-1 : CLAUDE.md etat actuel — verifier S47 + compteurs.
- [ ] E-2 : SPRINT_LOG.md row S47 — verifier presente.
- [ ] E-3 : memory nexus_grid_pivot.md — verifier tip mis a jour.

---

## Carries S48

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 11+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P3-AUDIT-B-4-S45 TOCTOU canary reload | 2/3 | |
| P2-REVIEW-B-1-S46 kudos SQL pagination | 2/3 | |
| P2-REVIEW-C-1-S46 app-specific schema drift | 2/3 | |
| P2-REVIEW-A-1-S47 execute_batch_raw pub | 1/3 | NEW |
| P2-REVIEW-A-2-S47 invite format test | 1/3 | NEW |
| P2-REVIEW-B-1-S47 deploy BlobsClient fragility | 1/3 | NEW |
| P2-REVIEW-C-1-S47 set_var process-wide | 1/3 | NEW |

**Note S48 pair** : S48 est pair → phase dette obligatoire
(§6.2.1 Regle 1). 2 items a 2/3 (TOCTOU canary, kudos SQL)
deviennent MANDATORY S49 si non resolus. 1 item (app-specific
schema) a 2/3 aussi.

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S48 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint47): ... avant S48 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D4 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S47 (decision sprint, pas audit)
- App runtime migration Rust (scope cut S48+)
