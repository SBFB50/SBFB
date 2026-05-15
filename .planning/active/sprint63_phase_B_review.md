# Phase Review — Sprint 63 Phase B

## Verdict : PASS

(Rigor signal : 2 findings P2 documentes / >=1 requis pour PASS rigoureux)

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, OSS prior art obligatoire — respecte (preflight S1a documente)
- sprint14_keyoxide_decision.md : deploy from source, provenance SLSA L1 — respecte (insert au deploy_from_repo)
- feedback_context7_systematic.md : context7 avant code lib/API — N/A (pas de nouvelle dep)
- Violations memory : 0

## Staging check (Step 1bis)
- Phase fichiers : 3 (db.rs, deploy.rs, http.rs)
- Planning split : sprint63_phase_B_preflight.md untracked -> chore(planning) AVANT commit phase
- Untracked accidentels : 0

## Suites (Step 2)
- cargo fmt : clean
- cargo clippy workspace : 0 warnings
- cargo nextest workspace : 1299 -> 1303 (+4 Phase B)
- cargo doctests : clean
- release build : OK
- npm lint + tsc : clean
- Vitest : 258 -> 258 (+0, pas de frontend touche)
- npm build + size : 6/6
- Playwright : N/A (setup-only, pas de tests Phase B-specifiques)

## Commit body validation (Step 4)
- Format titre : `feat(feed): Sprint 63 Phase B — provenance endpoint HTTP + SQLite M12`
- Contexte : present
- Fichiers touches avec rationale : present
- Delta tests cumule : +4 Rust (1 DB roundtrip + 3 handlers)
- Scope cuts honoured : present
- Co-Authored-By : present

## Modified-file branch coverage (Step 2bis, G9)
- db.rs : `insert_provenance_record()` (17 LOC) -> tested by `provenance_insert_and_retrieve`
- db.rs : `get_provenance_by_project()` (18 LOC) -> tested by `provenance_insert_and_retrieve` (found + not-found paths)
- deploy.rs : `insert_provenance_record()` call (4 LOC, best-effort with debug log) -> tested indirectly by `provenance_endpoint_found_and_verified` (integration)
- http.rs : `get_provenance()` handler (30 LOC) -> tested by 3 tests (not_found, found_verified, found_wrong_key)
- http.rs : match branches Ok(Some), Ok(None), Err -> all 3 branches covered by tests

## Research grounding (Step 4bis)
- 4bis-A OSS prior art : PASS — preflight S1a documente, 2 recherches WebSearch (SLSA provenance storage, Sigstore Rekor API). APPROACH-ALIGNED (pattern single-node pragmatique vs transparency log ecosystem-wide).
- 4bis-B context7 deps : N/A — aucune nouvelle dep ajoutee. rusqlite et ed25519-dalek deja en place.

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc present : N/A (M12 est une extension du schema existant, pas un nouveau module structurant)
- D1..D5 avec alternatives + rationale : kickoff §4 D1 cite 3 alternatives rejetees (extraction zip a la volee, table separee par projet, hash only). PASS.
- Solution la plus poussee : SQLite M12 + verify_provenance live est la bonne approche pour le scope single-node. Transparency log type Rekor serait surdimensionne.
- LOC estimees au plan : §6 plan.md contient des estimations LOC → P2 pre-existant (plan author, pas Phase B code)

## Scope cuts verification (Step 5)
- CuratorVouched : 0 hits dans diff
- BuildQuorumReached : 0 hits dans diff
- quarantine feed : 0 hits
- age witness : 0 hits
- CLI verify-release : 0 hits
- Protocol Explorer verification : 0 hits (Phase D scope)

## Findings

- **P2-VERIFY-LOCAL-KEY-ONLY** : `get_provenance` handler verifie la provenance avec `state.pow_keypair.public_bytes()` (cle locale). Pour les records signes par d'autres coordinateurs (feed sync cross-node, S64+), la verification retourne `false` car la cle du signataire n'est pas resolue depuis `record.node_id`. Acceptable S63 (single-node, le daemon ne stocke que ses propres provenance). Carry S64 : resoudre la cle publique depuis pkarr/node_id pour verification cross-node.

- **P2-PLAN-LOC-ESTIMATE** : plan.md §6 contient des estimations LOC prospectives (~150, ~310, etc.). Contraire a feedback_approach.md §6.7 ("Pas d'estimation LOC"). Pre-existant dans le plan (pas introduit par Phase B code). Carry note pour futurs plans.

## Cross-review findings (GPT 5.5, 2026-05-15)

6 findings soumis, 2 confirmes + 2 faux positifs + 2 deja documentes.

### Confirmes — fixes appliques

- **P1-PROVENANCE-PERSIST-BEFORE-PUBLISH** : insert_provenance_record()
  etait a deploy.rs:172, AVANT blob store (221) et announcement (234).
  Si blob echoue, orphan provenance record en DB. **Fix** : deplace
  l'insert APRES blob store reussi (apres ligne 230).

- **P2-LATEST-NON-DETERMINISTIC** : ORDER BY created_at DESC LIMIT 1
  sans rowid tiebreaker dans get_provenance_by_project. Inconsistant
  avec le pattern kudos (db.rs:434). created_at = secondes, deux
  deploys rapides = non-deterministe. **Fix** : ajoute rowid DESC.

### Faux positifs (analyses par team 4 agents)

- **P1-VITEST-REPRO** : issue pre-existante documentee dans
  vitest_env_variance.md (2026-05-10). Phase B = 0 fichiers frontend.
  CI pin Node 20 = vert. Node 22+ experimental-webstorage = cause
  connue. NON attribuable a Phase B.

- **P2-PROCESS-FORMAT** : sections "Pre-launch protocol",
  "Verification 7.4", "Carry closure / Unblock", "Risk" NE SONT PAS
  des sections obligatoires du template §4.1. Le review skill valide
  6 items (titre, contexte, fichiers, delta tests, scope cuts,
  Co-Authored-By), tous presents.

### Deja documentes

- P2-VERIFY-LOCAL-KEY-ONLY : deja dans la review originale §Findings
- P2-COVERAGE-DEPLOY-E2E : trade-off accepte (deploy E2E = clone git
  + pipeline complet, hors scope unit test). Carry S64.

## Recommendation
- Ready to commit : **oui** (fixes appliques, re-verification en cours)
- Carry-overs S64 : P2-VERIFY-LOCAL-KEY-ONLY, P2-COVERAGE-DEPLOY-E2E
- Corrections applied : P1-PROVENANCE-PERSIST-BEFORE-PUBLISH, P2-LATEST-NON-DETERMINISTIC
