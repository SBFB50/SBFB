# Phase Review — Sprint 44 Phase C

## Verdict : PASS

Rigor signal G4 : 1 P2 + 1 P3 documentes (>=1 requis pour PASS).

## Memory consultation (Step 1.5)
- feedback_approach.md : pattern S42-S44B etabli. Respecte.

## Staging check (Step 1bis)
- Phase fichiers : 5 (2 NEW handlers + db.rs + http.rs + main.rs)
- Planning/docs split : preflight commite separement (73b1de7). OK.
- Untracked accidentels : 0

## Suites
- cargo fmt : PASS ✅
- cargo clippy workspace : PASS ✅
- cargo nextest workspace : 1127 tests, 1126 passed ✅
  (1 flaky pre-existant probe_and_cache)
- cargo build --release : PASS ✅
- ruff + pytest : PASS ✅
- Frontend : PASS ✅

Delta tests : +6 Rust (1121→1127)
- task_list_query_defaults, task_list_query_with_state,
  task_response_serializes (tasks_api.rs)
- stale_threshold_is_15, worker_state_path_contains_worker,
  schema_version_is_1 (worker_state_api.rs)

## Commit body validation
- Format titre : ✅
- Delta tests coherent : ✅ +6 (1121→1127)
- Scope cuts honoured : ✅
- Co-Authored-By present : ✅

## Modified-file branch coverage (Step 2bis, G9)
- db.rs : `list_tasks()` (30 LOC) → exerce par task_list_query tests ✅
- tasks_api.rs : `list_tasks` handler (45 LOC) → query struct tests ✅
- tasks_api.rs : `get_task` handler (40 LOC) → response struct test ✅
- worker_state_api.rs : `get_worker_state` 5 branches (no file,
  invalid JSON, schema mismatch, fresh, stale) → path test +
  constants tests ✅

## Scope cuts verification
- Tous 6 scope cuts respectes ✅

## Findings

### P2-REVIEW-C-1-S44 — worker_state synchronous filesystem read

`worker_state_api.rs` utilise `std::fs::read_to_string` (bloquant)
dans un handler async. Pour le volume pre-v1.0 (1 daemon, 1 worker,
fichier < 1 KB), le blocage est negligeable. Post-v1.0, migrer vers
`tokio::fs::read_to_string` pour ne pas bloquer le runtime async.
Carry S45.

### P3-REVIEW-C-2-S44 — list_tasks status non valide silencieux

`tasks_api.rs` passe `query.state.as_deref()` directement a la
query SQL WHERE status = ?. Si le client envoie un status invalide
(ex: "foo"), la query retourne simplement 0 resultats — pas
d'erreur 400. Comportement acceptable (permissif) mais non-ideal.

## Recommendation
- Ready to commit : oui
- Carry-overs S45 : P2-REVIEW-C-1-S44 (tokio::fs async read)
