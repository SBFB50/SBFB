# Phase Review — Sprint 35 Phase B

## Verdict : PASS (1 P2 + 1 P3)

Rigor signal : 2 findings documentes (>=1 P2+ requis pour PASS rigoureux).

## Memory consultation
- feedback_approach.md : pick deepest — appel direct nexus-core-rs, pas de PyO3, conforme D4
- Violations memory : 0

## Staging check (Step 1bis)
- Phase fichiers : dispatcher.rs (NEW), lib.rs (+1 mod), Cargo.toml (+rand dep), http.rs (+route +handler), daemon Cargo.toml (+dep coordinator-rs), preflight
- Planning/docs split : preflight dans staging, review sera chore(planning) separe
- Untracked accidentels : 0

## Suites
- Rust nextest : 913 -> 919 (+6 dispatcher) ✅ (1 flaky browse pre-existing)
- Rust clippy : 0 warnings ✅
- Rust fmt : clean ✅
- Release build : OK ✅
- Python SDK : 195 pass (1 flaky file-lock) ✅
- Python gov : 46 pass ✅
- Frontend : 267 Vitest + build + size OK ✅

## Commit body validation
- Format titre : ✅
- Delta tests coherent : ✅ (913→919 = +6 dispatcher)
- Scope cuts honoured : ✅
- Co-Authored-By present : ✅

## Modified-file branch coverage (Step 2bis, G9)
- `http.rs` : `coordinator_submit_task()` handler NEW — tested indirectement via dispatcher unit tests (le handler est thin routing, toute la logique est dans dispatcher.submit()) ✅
- `lib.rs` : +1 `pub mod dispatcher;` — N/A (declaration)
- `Cargo.toml` : +1 dep — N/A

## Scope cuts verification
- "Migration complete coordinator" §7.1 : dispatcher only, pas complet ✅
- "Suppression coordinator Python" §7.2 : 0 suppression ✅
- "KudosLedger Rust" §7.3 : non touche ✅

## Horizon long-terme + documentation amont
- D1..D5 avec alternatives : ✅ (kickoff complet)
- Solution la plus poussee : ✅ (appel direct Rust sans PyO3)
- Aucune LOC estimee au plan : ✅

## Research grounding (Step 4bis)
- S1a OSS prior art : ✅ (APPROACH-ALIGNED dans preflight)
- S1b deps context7 : N/A (rand existant workspace)

## Findings

### P2-REVIEW-B-1 : coordinator_submit_task ouvre DB in-memory a chaque requete

`http.rs:1269` — le handler ouvre `CoordinatorDb::open_in_memory()`
a chaque appel. Les taches ne persistent pas entre les requetes.
En production, le dispatcher doit vivre dans `DaemonHttpState`
avec une DB persistante. Acceptable pour Phase B (proof of concept
endpoint), le wire-up complet est Phase C/S36.
**Action** : carry S36, integrer dispatcher dans DaemonHttpState
avec DB fichier.

### P3-REVIEW-B-1 : pas de test d'integration HTTP pour le endpoint

Le handler est couvert indirectement par les tests dispatcher
unitaires. Un test d'integration via `tower::ServiceExt::oneshot`
(pattern existant dans http.rs tests) serait plus robuste.
Acceptable car le handler est thin (8 lignes de routing).

## Recommendation
- Ready to commit : **oui**
- Carry-overs S36 : P2-REVIEW-B-1 dispatcher persistant dans DaemonHttpState
