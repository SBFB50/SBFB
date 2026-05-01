# Phase Review — Sprint 50 Phase A

## Verdict : PASS (1 P2, 1 P3)

Rigor signal : 2 findings P2+ documentes (>=1 requis pour PASS).

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — Phase A fixe 2 carries P2 (JoinHandle lifecycle + handler DB tests). Structural. Respecte.

## Staging check (Step 1bis)
- Phase fichiers : 2 modifies (runtime.rs, main.rs) + 1 preflight
- Untracked accidentels : 0

## Suites (Step 2)
- cargo fmt : 0 diff ✅
- cargo clippy : 0 warnings ✅
- cargo nextest workspace : 1199 passed, 0 failed ✅
- cargo doctests : ok ✅
- release build : ok ✅
- ruff format + check : ok ✅
- pytest SDK : 195 ✅
- pytest coord : 264+17f+6s ✅
- pytest gov : 46 ✅
- tsc : 0 error ✅
- npm lint : 0 error ✅
- Vitest : 267 ✅
- npm build : ok ✅
- size-limit : 5/5 ✅

## Modified-file branch coverage (Step 2bis, G9)
- runtime.rs : `dispatch_handle.take()` + join at shutdown — exercee par `start_then_shutdown_roundtrip_cleans_up_running_json` ✅
- main.rs : 4 async handler tests (NEW) = le code ajoute EST le test coverage ✅

## Delta tests
| Suite | Entree S50 | Phase A | Delta |
|---|---|---|---|
| Rust nextest | 1195 | 1199 | +4 (handler tests) |
| Tous autres | inchanges | inchanges | +0 |

## Commit body validation
- Format titre : `feat(sprint50): Sprint 50 Phase A — dette pair dispatch JoinHandle + CLI integration tests` ✅
- Delta tests coherent : +4 Rust ✅
- Scope cuts honoured : 8/8 ✅
- Co-Authored-By : present ✅

## Scope cuts verification
- Events SSE daemon-native : 0 fichiers ✅
- MCP server Rust : 0 fichiers ✅
- app-gov recreation : 0 fichiers ✅
- CI/CD + binaires : 0 fichiers ✅
- VPS deployment : 0 fichiers ✅
- Kudos debit/stake : 0 fichiers ✅
- Pagination SQL : 0 fichiers ✅
- Test infra mk_state() : 0 fichiers ✅

## Horizon long-terme + documentation amont
- Design doc : N/A (dette pair, pas de nouveau module) ✅
- D1..D4 avec alternatives + rationale : ✅ (kickoff §4)
- Solution la plus poussee : ✅ (JoinHandle = pattern tokio standard)
- Aucune LOC estimee au plan : ✅ (corrige pre-commit hook)

## Research grounding (Step 4bis)
- S1a OSS prior art : preflight G8 EXECUTE, APPROACH-ALIGNED (tokio standard) ✅
- S1b deps : 0 nouvelle dep ✅
- context7 : N/A (pas de lib externe) ✅

## Findings

**P2-REVIEW-A-1-S50 — dispatch_handle join order assumption**

Le dispatch loop channel se ferme quand le dernier clone du sender
(dans DaemonHttpState via Arc) est drop. Le shutdown() drop le HTTP
state en joignant le http_handle (L741-743), ce qui drop l'Arc.
MAIS si le gossip task ou le peer task garde une ref indirecte au
state, le sender pourrait survivre apres le HTTP join. En pratique
le DaemonHttpState n'est PAS clone dans gossip/peer (verifie dans
runtime.rs — seul le router/http_handle recoit l'Arc), donc le
channel close effectivement au HTTP join. Le join dispatch qui suit
(L745-749) est correct. Finding P2 "assumption correcte mais fragile
en cas de refactoring futur" — carry S51 1/3.

**P3-REVIEW-A-2-S50 — handler tests ne verifient pas stdout**

Les 4 handler tests verifient que les handlers ne paniquent pas
mais ne capturent pas stdout pour verifier le contenu affiche (ex :
"Project initialized.", "Created invite: inv-..."). Acceptable car
le but est de tester le chemin DB, pas le format d'affichage.

## Recommendation
- Ready to commit : oui
- Carry-overs S51 : P2-REVIEW-A-1-S50 dispatch_handle join order 1/3
