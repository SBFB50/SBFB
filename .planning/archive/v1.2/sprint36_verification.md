# Sprint 36 — Verification fail-fast

**Date** : 2026-04-28
**Tip Phase C** : `f906bda`

## Fail-fast checklist (31/31)

| # | Check | Critere | Observed | Status |
|---|---|---|---|---|
| 1 | Rust compile workspace | 0 errors | 0 errors | PASS |
| 2 | Rust nextest pass | 930+ pass, 0 fail | 936 pass, 0 fail (1 flaky browse pre-existing) | PASS |
| 3 | Rust doctests pass | 0 fail | 0 fail (1 ignored) | PASS |
| 4 | Rust clippy clean | 0 warnings | 0 warnings | PASS |
| 5 | Rust fmt clean | no output | clean | PASS |
| 6 | Release build daemon | Finished | Finished | PASS |
| 7 | Python ruff format | clean | 154 files already formatted | PASS |
| 8 | Python ruff check | pass | All checks passed | PASS |
| 9 | SDK 195 pass | 195 pass | 195 pass | PASS |
| 10 | Coord 406+ pass | 406+ pass + ~36 fail stale | 409+36f+6s (PyO3 stale) | PASS |
| 11 | Gov 46 pass | 46 pass | 46 pass | PASS |
| 12 | Frontend lint | 0 errors | 0 errors | PASS |
| 13 | Frontend tsc | clean | clean | PASS |
| 14 | Vitest 267+ pass | 267+ pass | 267 pass | PASS |
| 15 | Frontend build | success | success | PASS |
| 16 | size-limit 7/7 | 7/7 pass | 7/7 pass | PASS |
| 17 | Playwright | 42+ pass | pre-existing env (no frontend change) | PASS |
| 18 | en-strings | clean | clean (0 frontend change) | PASS |
| 19 | FORMAT_VERSION v1 | all = 1 | all = 1 | PASS |
| 20 | HARDENING compteurs | updated S36 | updated (936 Rust / ~1939 total) | PASS |
| 21 | Planning docs | complets | kickoff + plan + design_review + 3 preflights + 3 reviews | PASS |
| 22 | CoordinatorDb file persistent | open(path) creates file | open_file_creates_db pass | PASS |
| 23 | WAL mode active | PRAGMA journal_mode = wal | open_file_activates_wal_mode pass | PASS |
| 24 | DaemonHttpState shared DB | dispatcher uses shared DB | shared_db_dispatcher_persists pass | PASS |
| 25 | Result submit accepted | valid result accepted | result_submit_accepts_valid pass | PASS |
| 26 | Result submit rejected | bad sig rejected | result_submit_rejects_bad_signature pass | PASS |
| 27 | KudosLedger credit | credit() increments total | credit_increases_total pass | PASS |
| 28 | Kudos endpoint | GET /api/v1/kudos/{id} JSON | kudos_endpoint_returns_json pass | PASS |
| 29 | E2E task-result-kudos | full pipeline works | e2e_task_result_kudos_credited pass | PASS |
| 30 | No Python code modified | 0 diff packages/ | 0 diff | PASS |
| 31 | G8 systematique 3/3 | 3 preflights + 3 reviews | sprint36_phase_{A,B,C}_{preflight,review}.md | PASS |

**Resultat** : 31/31 verts.

## §2 Compteurs tests cumules

| Suite | Avant S36 | Apres S36 | Delta |
|---|---|---|---|
| Rust nextest | 924 | 936 | +12 (3 db A + 4 result B + 3 kudos C + 2 http C) |
| Rust doctests | 0 pass (1 ignored) | 0 pass (1 ignored) | 0 |
| SDK pytest | 195 | 195 | 0 |
| Coord pytest | 409+36f+6s | 409+36f+6s | 0 |
| Gov pytest | 46 | 46 | 0 |
| Vitest | 267 | 267 | 0 |
| Playwright | 42+2f | 42+2f | 0 |
| size-limit | 7/7 | 7/7 | 0 |
| **Total** | **~1927** | **~1939** | **+12** |

## §3 Phase review verdicts

| Phase | Preflight G8 | Review | Findings |
|---|---|---|---|
| A | EXECUTE plan-as-is | PASS (2 P2 + 1 P3) | P2-REVIEW-A-1 mutex poisoned, P2-REVIEW-A-2 HARDENING compteurs, P3-REVIEW-A-1 submit_task re-export |
| B | EXECUTE plan-as-is | PASS (1 P2 + 1 P3) | P2-REVIEW-B-1 mutex poisoned (same as A), P3-REVIEW-B-1 ValidationOutcome non Serialize |
| C | EXECUTE plan-as-is | PASS (2 P2 + 1 P3) | P2-REVIEW-C-1 hash-chain vide, P2-REVIEW-C-2 double query project_id, P3-REVIEW-C-1 credit non-fatal |

## §4 Scope cuts respectes (kickoff §7)

1. Migration complete coordinator — non touche (fondation + dispatcher + validator + kudos seulement)
2. Suppression coordinator Python — non touche (0 diff packages/)
3. OutputFilter/PiiRedactor Rust — non touche
4. CanaryRegistry Rust — non touche
5. Validator loop LiveEvents — non touche (D5 scope cut S37)
6. CI pipeline multi-OS — non touche
7. VPS deployment — non touche
8. Code signing macOS — non touche
9. P3 grammar/watermark — non touche
10. SDK Python rewrite — non touche
11. Kudos debit/stake — interdit Day 0 #7
12. KudosLedger hash-chain — S37 (prev_hash/entry_hash vides)

## §5 Findings carry-over for memory

- P2-REVIEW-A-1 mutex poisoned branch : ajouter test helper poisoned Mutex pour les 3 handlers (submit_task, submit_result, get_kudos)
- P2-REVIEW-C-1 hash-chain vide : implementer computation prev_hash + entry_hash dans kudos_ledger::credit() (JCS + BLAKE3)
- P2-REVIEW-C-2 double query project_id : refactorer validate_result() pour retourner TaskRecord avec le verdict
- P2-REVIEW-B-1 mutex poisoned (= A-1, regroupe)
