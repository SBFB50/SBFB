# Sprint 47 — Audit findings (Sprint 48 Phase 0)

**Auditeur** : session fraiche (pas la session qui a code S47).
**Tip d'entree** : `6de113c` (HEAD, Phase D wrap-up).
**Tip feat** : `3641871` (Phase C, dernier feat commit).
**Diff audite** : `d1ef20d..3641871` (3 phases : A carries S45,
B integration tests 5 routes, C happy path + aliases cleanup).
**Timebox** : 1h. **Methode** : 20 check items, 5 tracks,
4 agents paralleles, lecture code + grep evidence.

---

## Verdict : PASS

0 P0, 0 P1, 0 new P2, 1 P3. 4 carries P2 existants confirmes
valides. G4 satisfait (4 P2+ documentes). S48 Phase A peut
demarrer directement.

---

## Track A — S45 carries resolution (Phase A)

| Item | Verdict | Evidence |
|---|---|---|
| A-1 diagnostic Err test | ✅ | `http.rs:4518` `diagnostic_fairness_returns_500_on_corrupted_db` : mk_state(), DROP TABLE kudos, GET /diagnostic/fairness → 500 + "worker_contributions" |
| A-2 invite ID format | ✅ | `invite_api.rs:80-81` `inv-{node_prefix}-{now}-{seq}`, `state.node_id` pub field `http.rs:76` accessible via State extractor |
| A-3 Python modules supprimes | ✅ | 7 modules .py + 7 tests .py supprimes, 0 import restant dans packages/. Fichiers .pyc en __pycache__ non tracks (gitignore) |
| A-4 execute_batch_raw | ✅ | `db.rs:348` #[doc(hidden)] present. pub (pas pub(crate)) = carry confirme P2-REVIEW-A-1-S47 1/3 |

## Track B — Integration tests 5 routes (Phase B)

| Item | Verdict | Evidence |
|---|---|---|
| B-1 deploy_private happy | ✅ | `http.rs:4586` make_test_zip() (zip::ZipWriter + index.html) + mk_state() iroh Node reel → 200 + hash >= 32 chars |
| B-2 deploy error paths | ✅ | invalid zip → 400 (4608), non-HTTP URL → 400 (4624), invalid SHA → 400 (4647) |
| B-3 apps integration | ✅ | list empty → 200 count=0 (4673), list with entries via add_direct_entry → count=1 (4692), get by id → 200 (4730), get unknown → 404 (4769) |
| B-4 auth_token | ✅ | GET /auth/token → 200 + TEST_TOKEN (4787) |

9 tests total pour 5 routes — conforme verification.md.

## Track C — Happy path tests + aliases cleanup (Phase C)

| Item | Verdict | Evidence |
|---|---|---|
| C-1 consent happy path 4/4 | ✅ | set level 2 → 200 (4808), get persisted level 3 (4830), whitelist add (4867), whitelist remove (4896). SBFB_HOME tempdir isolation |
| C-2 files happy path 3/3 | ✅ | upload → 201 + sha256 64 hex (4943), manifest → sha match (4968), stream → content match (5008). sha2::Sha256 (files.rs:79) |
| C-3 aliases supprimes | ✅ | grep CoordinatorProtocolError + CoordinatorHttpError + normalizeCoordinatorUrl = 0 matches dans .ts/.tsx/.rs/.py |
| C-4 callers migres | ✅ | AddCoordinatorDialog.tsx (32-35), projectStore.ts (21), coordinator.ts (23,43,57), daemon.ts (16), tests → ApiProtocolError/ApiHttpError/normalizeApiUrl |

## Track D — Process / meta

| Item | Verdict | Evidence |
|---|---|---|
| D-1 G8 3/3 EXECUTE | ✅ | phase_A_preflight (dc891da), phase_B_preflight (5e10e80), phase_C_preflight (d86edd6) |
| D-2 scope cuts 11/11 | ✅ | diff --stat d1ef20d..3641871 : 44 fichiers, 0 hors-scope |
| D-3 8 carries resolus | ✅ | verification.md §3 : 3 S45 CLOSED + 5 S46 CLOSED |
| D-4 reviews 3/3 PASS | ✅ | A (2 P2, 1 P3), B (1 P2, 1 P3), C (1 P2, 1 P3) |
| D-5 delta tests | ✅ | Rust +17 (17 #[tokio::test] ajoutes), Python -65 (7 test files supprimes = -59p/-6f) |

## Track E — Doc coherence

| Item | Verdict | Evidence |
|---|---|---|
| E-1 CLAUDE.md | ✅ | "Sprints 0-47 CLOSED", 1185 Rust, ~1936 total, S47 delta +17/-65 |
| E-2 SPRINT_LOG.md | ✅ | Row S47 presente (tip 3641871, DONE, theme, 3 phases, carries) |
| E-3 memory tip | P3 | Tip `3641871` trails HEAD `6de113c` par 2 chore commits (Phase C review + Phase D wrap-up). Contenu factuel correct (S47 CLOSED). Corrige dans cette session |

---

## Findings

### P3 (1)

- **P3-AUDIT-E-1-S47** : memory nexus_grid_pivot.md tip `3641871`
  au lieu de `6de113c`. Delta = 2 commits chore(planning) sans
  code. Contenu factuel correct (S47 CLOSED). Corrige par
  l'auditeur dans la meme session.

### Carries confirmes valides (4 P2 existants, non new)

- **P2-REVIEW-A-1-S47** execute_batch_raw pub 1/3 : confirme
  `db.rs:348` #[doc(hidden)] + pub. Seul caller hors-crate = test
  http.rs:4522. Fix trivial pub→pub(crate) + #[cfg(test)] re-export.
- **P2-REVIEW-A-2-S47** invite format test 1/3 : confirme, pas de
  test unitaire verifiant le pattern `inv-{node8}-{ts}-{seq}`.
- **P2-REVIEW-B-1-S47** deploy BlobsClient fragility 1/3 : confirme,
  deploy_private test utilise un iroh Node reel (mk_state). Si Node
  boot echoue, test flaky. Risque pre-v1.0 accepte.
- **P2-REVIEW-C-1-S47** set_var process-wide 1/3 : confirme,
  consent/files tests utilisent std::env::set_var (thread-unsafe
  en Rust 1.66+). Pas de failure observee car tests sequentiels
  via nextest default. Risque si parallelisme futur.

---

## Carries S48

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 11+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 Day 0 |
| P3-AUDIT-B-4-S45 TOCTOU canary reload | 2/3 | microseconde, pre-v1.0 |
| P2-REVIEW-B-1-S46 kudos SQL pagination | 2/3 | |
| P2-REVIEW-C-1-S46 app-specific schema drift | 2/3 | |
| P2-REVIEW-A-1-S47 execute_batch_raw pub | 1/3 | |
| P2-REVIEW-A-2-S47 invite format test | 1/3 | |
| P2-REVIEW-B-1-S47 deploy BlobsClient fragility | 1/3 | |
| P2-REVIEW-C-1-S47 set_var process-wide | 1/3 | |

**Note S48 pair** : S48 est pair → phase dette obligatoire
(§6.2.1 Regle 1). 3 items a 2/3 (TOCTOU canary, kudos SQL,
app-specific schema) deviennent MANDATORY S49 si non resolus.

---

## Recommendation

Verdict PASS — 0 P0, 0 P1, 0 new P2, 1 P3 (cosmetic, corrige).
4 carries P2 existants confirmes valides et correctement
documentes dans le carry table S48. Aucun fix bloquant. S48
Phase A peut demarrer directement.
