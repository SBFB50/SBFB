# Phase Review — Sprint 64 Phase D

## Verdict : PASS (post-correction)

(Rigor signal : 4 findings P1 corrigees + 2 P2 documentes + 1 P3)

Initial review was FAIL — 4 P1 bloquants identifies par cross-review
utilisateur. Fix commit corrige les 4 P1.

## Memory consultation (Step 1.5)
- feedback_approach.md : tests deterministes (D1 respectee), pick deepest — respecte
- sprint14_keyoxide_decision.md : Ed25519 deploy from source — teste dans forgery test, respecte
- feedback_context7_systematic.md : N/A (pas de nouvelle lib/API)
- Violations memory : 0

## Staging check (Step 1bis)
- Phase fichiers : 5 (public_feed.rs + Cargo.toml + Cargo.lock + .gitignore + tests/multi_daemon.rs + feed_sync.rs)
- Planning/docs split : chore(planning) fait (preflight commit `9b6c282`)
- Untracked accidentels : 0

## Suites (post-fix)
- cargo fmt : 0 diff ✅
- cargo clippy : 0 warnings ✅ (workspace)
- cargo nextest coordinator-rs + shell-daemon : 487 PASS ✅
- cargo doctests : ok (1 ignored) ✅
- release build : ok ✅
- npm lint : 0 errors ✅
- tsc : 0 errors ✅
- Vitest : 265 → 265 (+0, no frontend change) ✅
- npm build : ok ✅
- size-limit : 6/6 ✅

## Commit body validation
- Format titre : ✅ `feat(feed): Sprint 64 Phase D — adversarial crypto + new node E2E`
- Delta tests coherent : ✅ (+5 = 4 crypto + 1 E2E)
- Scope cuts honoured : ✅ (12/12 non touches)
- Co-Authored-By present : ✅

## Modified-file branch coverage (Step 2bis, G9)
- `public_feed.rs` : `validate_feed_entry_timestamp()` (7 LOC) → tested by `test_adversarial_future_timestamp_rejected` ✅ (both OK 1h and rejected 31d paths)
- `feed_sync.rs` : `validate_feed_entry_timestamp()` call in ingestion path → exercises production wiring ✅
- `public_feed.rs` : BLAKE3 tamper test now constructs valid entry + tampers timestamp → verify_entry rejects with "entry_hash mismatch" ✅
- `tests/multi_daemon.rs` : hex-valid fixtures + correct JSON field names (`count`, `last_seq`) ✅

## Research grounding (Step 4bis)
- S1a OSS prior art : APPROACH-ALIGNED, crypto adversarial testing = standard practice ✅
- S1b deps : 4 dev-deps added (nexus-test-harness, tokio, reqwest, serde_json) — all existing workspace deps, 0 new external ✅
- context7 : N/A (no external API/lib touched) ✅

## Scope cuts verification
- 12/12 scope cuts (kickoff §7) : 0 fichiers diff les touchent ✅

## Horizon long-terme + documentation amont
- Design doc present : N/A (tests phase + 1 small production function)
- D1..D5 avec alternatives + rationale : frozen, not touched ✅
- Solution la plus poussee : deterministic tests per D1, production wiring of timestamp defense ✅
- Aucune LOC estimee au plan : ✅

## P1 corrigees (cross-review)

1. **P1 E2E fixtures non-hex** : `format!("{:0>64}", "project{i}")` contenait chars non-hex.
   Fix : `format!("{:0>64x}", i + 1)` — 100% hex valide.

2. **P1 mauvais champs JSON** : test lisait `entry_count` et `seq`, endpoints exposent `count` et `last_seq`.
   Fix : aligne sur les vrais noms de champs de feed_status et feed_cursor.

3. **P1 validate_feed_entry_timestamp non branche** : fonction existait mais jamais appelee en production.
   Fix : wired dans `feed_sync.rs` chemin d'ingestion distante, apres `verify_entry()`, avant `validate_feed_operation()`.

4. **P1 BLAKE3 test ne prouve pas reject via verify_entry** : testait seulement que hash change.
   Fix : construit une entry valide complete, tamper le timestamp, prouve que `verify_entry()` retourne "entry_hash mismatch".

## P2 documentes

- **P2** : `.gitignore` rule `tests/` bloquait silencieusement les integration tests Rust. Fixed. **Carry S65** : P2-GITIGNORE-SILENT-EXCLUDES.

- **P2** : Cargo.lock churn (215 ins / 228 del) cause par `cargo generate-lockfile` qui recalcule le graphe entier. Acceptable car les versions resolues sont identiques aux pins workspace existants (memes versions majeures/mineures). Le preflight G8 aurait du mentionner "4 dev-deps added" au lieu de "0 new dep". Corrige dans ce review.

## P3

- Nom `age_witness` renomme en `future_timestamp_rejected` pour eviter confusion avec le vrai `AgeWitness` de gossip.

## Recommendation
- Ready to commit : **oui** (post-correction)
- Carry-overs S65 : P2-GITIGNORE-SILENT-EXCLUDES, P2-URL-STRUCTURE-VALIDATION (Phase C)
- Corrections applied : 4 P1 resolus dans ce fix commit
