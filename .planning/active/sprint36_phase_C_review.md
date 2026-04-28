# Phase Review — Sprint 36 Phase C

## Verdict : PASS (2 P2 + 1 P3)

Rigor signal : 3 findings (2 P2 + 1 P3) documentes / >=1 requis pour PASS.

## Memory consultation (Step 1.5)
- feedback_approach.md : "pick deepest" — port fidele Python ledger. Conforme.
- feedback_kudos_non_monetary.md : non-monetary, non-transferable. Phase C = credit() + query. Pas de debit/stake/burn. Conforme.
- fairness_vision.md : composition 3 couches post-v1.0. Phase C = credit simple. Conforme.

## Staging check (Step 1bis)
- Phase fichiers : 4 (kudos_ledger.rs NEW, lib.rs +pub mod, db.rs +2 queries, http.rs +kudos wire +GET endpoint +2 tests)
- Planning split : sprint36_phase_C_preflight.md untracked -> chore(planning) AVANT phase
- Untracked accidentels : 0

## Suites (targeted 127/127 pass)
- Rust nextest targeted : 127/127 (coordinator-rs 27 + shell-daemon 100) pass
- Clippy : clean
- Fmt : clean
- Full workspace + Python + Frontend + release build : background

## Delta tests
- Rust nextest coordinator-rs : 24 -> 27 (+3 kudos_ledger unit)
- Rust nextest shell-daemon : +2 integration (e2e_task_result_kudos + kudos_endpoint_json)
- Total : 931 -> 936 (+5 Phase C)

## Modified-file branch coverage (Step 2bis, G9)
- kudos_ledger.rs : `credit()` (25 LOC) -> tested by `credit_increases_total` + `e2e_task_result_kudos_credited` ✅
- kudos_ledger.rs : `get_project_kudos()` -> tested by `get_project_kudos_empty` + `get_project_kudos_with_contributors` + `kudos_endpoint_returns_json` ✅
- db.rs : `get_project_kudos_total()` -> tested indirectly via kudos_ledger tests ✅
- db.rs : `get_project_contributors()` -> tested via `get_project_kudos_with_contributors` ✅
- http.rs : `coordinator_get_kudos` handler -> tested by `kudos_endpoint_returns_json` ✅
- http.rs : kudos credit wire in result handler -> tested by `e2e_task_result_kudos_credited` ✅

## Scope cuts verification
- Hash-chain append-only (§7.12) : prev_hash + entry_hash = empty strings (placeholder). Pas de computation hash. -> ✅
- Validator loop LiveEvents (§7.5) : pas touche -> ✅
- Migration complete (§7.1) : pas OutputFilter/PiiRedactor -> ✅
- Kudos debit/stake (§7.11) : interdit Day 0 #7 -> ✅

## Findings

### P2-REVIEW-C-1 : prev_hash et entry_hash toujours vides
kudos_ledger.rs : `credit()` ecrit `prev_hash: String::new()` et
`entry_hash: String::new()`. Le hash-chain cryptographique est
scope-cut S37 (kickoff §7.12). Les champs existent dans la DB
pour la migration future mais ne sont pas computes. Carry S37.

### P2-REVIEW-C-2 : project_id lookup post-validation double query
http.rs L1330 : le handler fait `db.get_task()` une seconde fois
apres validation pour recuperer le project_id. La premiere query
est dans `validate_result()` mais ne retourne pas le project_id.
Refactoring : validate_result pourrait retourner le TaskRecord
avec le verdict. Carry S37 (optimisation, pas bug).

### P3-REVIEW-C-1 : kudos credit failure non-fatal
Le handler log un warn si credit() echoue mais retourne quand meme
200 Accepted. Correct pour ne pas bloquer la validation, mais le
worker ne sait pas que ses kudos n'ont pas ete credites. Acceptable
pour v1 loopback-only.

## Recommendation
- Ready to commit : **oui** (apres background checks + chore planning)
- Carry-overs S37 : P2-REVIEW-C-1 (hash-chain), P2-REVIEW-C-2 (double query)
