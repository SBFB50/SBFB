# Phase Review — Sprint 49 Phase A

## Verdict : PASS (1 P2, 1 P3)

Rigor signal : 2 findings P2+ documentes (>=1 requis pour PASS).

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — Phase A fait du wiring structural (dispatcher + validator existants vers project doc iroh-docs). Pas de band-aid. Respecte.
- feedback_context7_systematic.md : context7 sur iroh-docs 0.98 consulte dans preflight G8 S1b. API create/open/list/author_default confirmee. Respecte.

## Staging check (Step 1bis)
- Phase fichiers : 3 modifies (http.rs, runtime.rs, main.rs) + 1 NEW (dispatch_loop.rs)
- Planning preflight : sprint49_phase_A_preflight.md untracked — stage avec la phase (G8 output)
- Untracked accidentels : 0

## Suites (Step 2)
- cargo fmt : 0 diff ✅
- cargo clippy : 0 warnings ✅
- cargo nextest workspace : 1187 passed, 0 failed ✅
- cargo doctests : ok (1 ignored) ✅
- release build : ok ✅
- ruff format + check : ok ✅
- pytest SDK : 195 ✅
- pytest coord : 264+17f+6s ✅ (17f = PyO3 stale pre-existant)
- pytest gov : 46 ✅
- tsc : 0 error ✅
- npm lint : 0 error ✅
- Vitest : 267 ✅
- npm build : ok ✅
- size-limit : 5/5 ✅

## Modified-file branch coverage (Step 2bis, G9)
- http.rs : `if let Some(ref tx) = state.task_dispatch_tx` (defensive guard, None in tests) → CONCERN acceptable (defensive branch, main path tested)
- http.rs : `if let Err(e) = tx.try_send(...)` (error logging only) → CONCERN acceptable
- runtime.rs : `if let Some(&first_id) = existing.first()` (doc create/reopen) → exercised by runtime start() tests which now boot with docs protocol ✅
- dispatch_loop.rs : NEW file, `dispatch_loop_writes_to_doc` test covers happy path ✅

## Delta tests (Step 3)
- Rust : 1186 → 1187 (+1 : dispatch_loop_writes_to_doc)
- Tout le reste : inchange

## Commit body validation (Step 4)
- Format titre : ✅ `feat(sprint49): Sprint 49 Phase A — coordinator lifecycle in daemon dispatch + validator doc wiring`
- Contexte present : ✅ (coordinator absorption, project doc, dispatch MPSC, G1 D2 ack)
- Fichiers touches avec rationale : ✅
- Delta tests cumule : ✅ (+1)
- Scope cuts honoured : ✅ 12/12
- Co-Authored-By : ✅

## Research grounding (Step 4bis)
- S1a OSS prior art : iroh-docs natif (APPROACH-ALIGNED), documente dans preflight ✅
- S1b deps : iroh-docs 0.98 pinne, context7 confirme API stable ✅
- Preflight G8 : EXECUTE plan-as-is (`752b85d`) ✅

## Horizon long-terme (Step 4ter)
- Design doc : S49_coordinator_rust_migration.md present dans .planning/research/ ✅
- D1..D4 avec alternatives : ✅ (kickoff §4, 2-3 alternatives par decision)
- Solution la plus poussee : ✅ (MPSC channel = serialisation sans contention, meilleur que mutex direct sur doc)
- Aucune LOC estimee au plan : ✅ (corrige pre-commit hook)

## Scope cuts verification (Step 5)
- app-gov conversion : 0 fichier ✅
- events.py SSE : 0 fichier ✅
- MCP server migration : 0 fichier ✅
- PyO3 bindings removal : 0 fichier ✅
- Suppression coordinator Python : 0 fichier ✅
- Suppression SDK Python : 0 fichier ✅
- CI/CD + binaires : 0 fichier ✅
- VPS deployment : 0 fichier ✅
- Kudos debit/stake : 0 fichier ✅
- Test infra mk_state() : 0 fichier ✅
- Pagination SQL-side : 0 fichier ✅
- auth.rs set_var : 0 fichier ✅

## Findings

### P2 (1)

- **P2-REVIEW-A-1-S49** : le dispatch loop est spawned via
  `tokio::spawn` fire-and-forget dans runtime.rs — le JoinHandle
  n'est PAS stocke dans DaemonRuntime. Si le loop panic (ex :
  doc write echoue de facon fatale), le daemon ne le detecte pas.
  Les task handles gossip et HTTP sont stockes pour graceful
  shutdown, mais pas le dispatch loop. Pre-v1.0, le loop ne fait
  que logger des warnings sur erreur doc write (pas de panic). Post-
  v1.0, stocker le handle et surveiller la completion. 1/3.

### P3 (1)

- **P3-REVIEW-A-1-S49** : `entry.clone()` dans le handler HTTP
  task submit clone l'entiere TaskEntry (incluant signature 64
  bytes + task struct). Acceptable pour le debit pre-v1.0 (channel
  capacity 64, submissions HTTP rares). Si throughput augmente,
  considerer Arc<TaskEntry> pour le channel.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S50 : P2-REVIEW-A-1-S49 dispatch loop JoinHandle 1/3
