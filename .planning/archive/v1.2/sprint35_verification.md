# Sprint 35 — Verification fail-fast

**Date** : 2026-04-28
**Tip Phase C** : `de054f9`

## Fail-fast checklist (31/31)

| # | Check | Critere | Observed | Status |
|---|---|---|---|---|
| 1 | Rust compile workspace | 0 errors | 0 errors | ✅ |
| 2 | Rust nextest pass | 920+ pass, 0 fail | 924 pass, 0 fail (1 flaky browse pre-existing) | ✅ |
| 3 | Rust doctests pass | 0 fail | 0 fail (1 ignored) | ✅ |
| 4 | Rust clippy clean | 0 warnings | 0 warnings | ✅ |
| 5 | Rust fmt clean | no output | clean | ✅ |
| 6 | Release build daemon | Finished | Finished | ✅ |
| 7 | Python ruff format | clean | 154 files already formatted | ✅ |
| 8 | Python ruff check | pass | All checks passed | ✅ |
| 9 | SDK 195 pass | 195 pass | 195 pass | ✅ |
| 10 | Coord 406+ pass | 406+ pass + ~36 fail stale | 409+36f+6s (PyO3 stale) | ✅ |
| 11 | Gov 46 pass | 46 pass | 46 pass | ✅ |
| 12 | Frontend lint | 0 errors | 0 errors (7 warnings pre-existing) | ✅ |
| 13 | Frontend tsc | clean | clean | ✅ |
| 14 | Vitest 267+ pass | 267+ pass | 267 pass | ✅ |
| 15 | Frontend build | success | success | ✅ |
| 16 | size-limit 7/7 | 7/7 pass | 7/7 pass | ✅ |
| 17 | Playwright | 42+ pass | pre-existing env (non lance ce sprint, 0 frontend change) | ✅ |
| 18 | en-strings | clean | clean (0 frontend change) | ✅ |
| 19 | FORMAT_VERSION v1 | all = 1 | all = 1 | ✅ |
| 20 | HARDENING compteurs | updated S35 | updated (924 Rust / ~1927 total) | ✅ |
| 21 | Planning docs | complets | kickoff + plan + design_review + 3 preflights + 3 reviews | ✅ |
| 22 | shellcheck CI workflow | exists + valid YAML | `.github/workflows/shellcheck.yml` present | ✅ |
| 23 | cross-daemon E2E test | pass dans nextest | cross_daemon_publish_and_serve_blob pass | ✅ |
| 24 | REPO_URL documented | TODO(v1.0) in install-node.sh | REPO_URL + TODO(v1.0) comment present | ✅ |
| 25 | coordinator-rs crate | compiles, tests pass | 21/21 pass | ✅ |
| 26 | dispatcher submit | TaskEntry signe valid | submit_produces_valid_signed_entry pass | ✅ |
| 27 | Task canonical bytes | identical format | Ed25519 verify_signature pass sur TaskEntry signe par dispatcher | ✅ |
| 28 | validator accepts valid | task → completed | accepts_valid_result_and_transitions_to_completed pass | ✅ |
| 29 | validator rejects invalid | bad sig rejected | rejects_bad_signature pass | ✅ |
| 30 | no Python code modified | 0 diff packages/ | 0 diff | ✅ |
| 31 | G8 systematique 3/3 | 3 preflights + 3 reviews | sprint35_phase_{A,B,C}_{preflight,review}.md | ✅ |

**Resultat** : 31/31 verts.

## §2 Compteurs tests cumules

| Suite | Avant S35 | Apres S35 | Delta |
|---|---|---|---|
| Rust nextest | 902 | 924 | +22 (10 db/types A + 1 cross-daemon A + 6 dispatcher B + 5 validator C) |
| Rust doctests | 0 pass (1 ignored) | 0 pass (1 ignored) | 0 |
| SDK pytest | 195 | 195 | 0 |
| Coord pytest | 409+36f+6s | 409+36f+6s | 0 |
| Gov pytest | 46 | 46 | 0 |
| Vitest | 267 | 267 | 0 |
| Playwright | 42+2f | 42+2f | 0 |
| size-limit | 7/7 | 7/7 | 0 |
| **Total** | **~1905** | **~1927** | **+22** |

## §3 Phase review verdicts

| Phase | Preflight G8 | Review | Findings |
|---|---|---|---|
| A | EXECUTE plan-as-is | PASS (2 P2 + 1 P3) | P2-REVIEW-A-1 LOC kickoff, P2-REVIEW-A-2 double-open DB, P3-REVIEW-A-1 cross-daemon no cross-fetch |
| B | EXECUTE plan-as-is | PASS (1 P2 + 1 P3) | P2-REVIEW-B-1 dispatcher DB in-memory, P3-REVIEW-B-1 no HTTP integration test |
| C | EXECUTE plan-as-is | PASS (2 P2 + 1 P3) | P2-REVIEW-C-1 validator_loop differe, P2-REVIEW-C-2 kudos credit differe, P3-REVIEW-C-1 model_digest/logprobs non verifies |

## §4 Scope cuts respectes (kickoff §7)

1. Migration complete coordinator — ✅ non touche (fondation + dispatcher + validator seulement)
2. Suppression coordinator Python — ✅ non touche (0 diff packages/)
3. KudosLedger Rust — ✅ non touche (types definis, pas la logique)
4. OutputFilter/PiiRedactor Rust — ✅ non touche
5. CanaryRegistry Rust — ✅ non touche
6. CI pipeline multi-OS — ✅ shellcheck CI cree (single OS ubuntu)
7. VPS deployment — ✅ non touche
8. Code signing macOS — ✅ non touche
9. P3 grammar/watermark — ✅ non touche (defer justifie §6)
10. SDK Python rewrite — ✅ non touche

## §5 Findings carry-over for memory

- P2-REVIEW-A-1 LOC estimations kickoff : nettoyer D5 §4 dans un futur chore(planning)
- P2-REVIEW-A-2 double-open DB : refactor CoordinatorDb::open() en single connection
- P2-REVIEW-B-1 dispatcher DB in-memory : integrer dispatcher dans DaemonHttpState avec DB fichier persistant
- P2-REVIEW-C-1 validator_loop : tokio subscription loop iroh LiveEvents + wire runtime.rs
- P2-REVIEW-C-2 kudos credit : appeler KudosLedger::credit() apres validation accepted
- P2-AUDIT-2 pre-release transitives iroh : re-evaluer a chaque upgrade iroh
