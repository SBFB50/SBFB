# Phase Review — Sprint 36 Phase A

## Verdict : PASS (2 P2 + 1 P3)

Rigor signal : 3 findings (2 P2 + 1 P3) documentes / >=1 requis pour PASS.

## Memory consultation (Step 1.5)
- feedback_approach.md : "no band-aid, pick deepest" — singleton DB = solution deep. Respecte.
- feedback_background_checks.md : checks lances en background. Respecte.

## Staging check (Step 1bis)
- Phase fichiers : 6 (db.rs, dispatcher.rs, http.rs, runtime.rs, PATTERNS.md, HARDENING_ROADMAP.md)
- Planning split : sprint36_phase_A_preflight.md untracked → chore(planning) AVANT phase commit
- Untracked accidentels : 0

## Suites (all green)
- Rust nextest : 924 → 927 (+3 : open_file, wal_mode, shared_db) ✅
- Rust clippy : clean ✅
- Rust fmt : clean ✅
- Rust doctests : pass ✅
- Release build : pass ✅
- Python ruff : clean ✅
- SDK pytest : 195 pass ✅
- Coord pytest : 409+36f+6s ✅
- Gov pytest : 46 pass ✅
- Frontend lint+tsc+vitest+build+size : all pass ✅

## Delta tests
- Rust nextest : 924 → 927 (+3 Phase A)
- Total : ~1927 → ~1930 (+3)

## Modified-file branch coverage (Step 2bis, G9)
- db.rs : `Debug impl` (3 LOC) → trivial, N/A ✅
- db.rs : `open()` fix double-open → tested by `open_file_creates_db_and_returns_schema_v1` + `open_file_activates_wal_mode` ✅
- dispatcher.rs : `submit_task()` free fn (extracted from `TaskDispatcher::submit()`) → tested by 6 existing dispatcher tests via delegation ✅
- http.rs : `coordinator_db.lock()` branch in handler → tested by existing `coordinator_submit_task` tests (mk_state now uses shared DB) ✅
- http.rs : poisoned mutex branch → defensive, returns 500 — CONCERN (no test for poisoned case, but trivial error path) ✅
- runtime.rs : `CoordinatorDb::open()` at boot → integration-level (daemon boot), tested indirectly via test harness ✅

## Scope cuts verification
- Migration complete coordinator (§7.1) : 0 fichier OutputFilter/PiiRedactor/CanaryRegistry → ✅
- Suppression coordinator Python (§7.2) : 0 diff packages/ → ✅
- Validator loop LiveEvents (§7.5) : pas de tokio subscription → ✅
- KudosLedger Rust (§7.3 = Phase C, pas A) : 0 nouveau module kudos → ✅

## Horizon long-terme + documentation amont
- Design doc : D1-D5 dans kickoff, pas de nouveau module structurant Phase A (refactoring) ✅
- D1-D5 alternatives citees : oui (3 alternatives par decision dans kickoff §4) ✅
- Solution la plus poussee : singleton Arc<Mutex<>> = standard axum, pas de shortcut ✅
- LOC estimees au plan : 0 (nettoyees avant commit planning) ✅

## Research grounding (Step 4bis)
- S1a preflight : APPROACH-ALIGNED (pattern standard axum, 35 sprints precedent) ✅
- S1b deps : 0 nouvelle dep ✅
- Plan §Research : technologies deja dans workspace (§6 plan.md) ✅

## Findings

### P2-REVIEW-A-1 : Mutex poisoned branch non testee
http.rs handler `coordinator_submit_task` : la branche `Err(_poisoned)`
(L1272) n'est pas exercee par un test. Le path retourne 500 internal.
C'est une branche defensive triviale (3 LOC), mais elle pourrait
masquer un bug si le message d'erreur change. Carry S37 acceptable.

### P2-REVIEW-A-2 : HARDENING_ROADMAP compteurs approximatifs
Le last_validated mis a jour mentionne "~927 Rust" mais le nextest
reporte 927 exactement. Les compteurs coord "409+36f+6s" ne sont pas
reverifies ce sprint (0 change Python). Les totaux sont des
approximations (~1930). Le verification.md Phase D les consolidera.

### P3-REVIEW-A-1 : submit_task pub fn expose sans re-export lib.rs
La fonction `submit_task` est publique dans `dispatcher.rs` mais pas
re-exportee dans `lib.rs`. Le daemon l'appelle via le chemin complet
`nexus_coordinator_rs::dispatcher::submit_task`. Pas de probleme
fonctionnel, juste une asymetrie d'API. Nit.

## Recommendation
- Ready to commit : **oui** (apres chore(planning) preflight)
- Carry-overs S37 : P2-REVIEW-A-1 (mutex test), P2-REVIEW-A-2 (compteurs)
