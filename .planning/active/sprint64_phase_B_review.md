# Phase Review — Sprint 64 Phase B

## Verdict : PASS (post fix cross-review `490e491`)

(Rigor signal : 1 P1 fixed + 2 P2 carry + 1 P2 process + 1 P3)

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — respecte (orphan rollback = root-cause fix, pas band-aid)
- Aucune zone-specific memory en tension

## Staging check (Step 1bis)
- Phase fichiers : 5 (db.rs, public_feed.rs, feed_sync.rs, runtime.rs, README.md)
- Planning files : sprint64_phase_B_preflight.md (G8 output, part of phase process)
- Untracked hors-scope : 2 (.planning/research/ files) — non stages, user notifie
- Planning/docs split : preflight va avec phase commit (output G8)

## Suites (Step 2) — post fix `490e491`
- cargo fmt : 0 diff
- cargo clippy : 0 warnings
- cargo nextest : 1309 -> 1315 (+5 Phase B + 1 fix tail-safe)
- cargo doctests : ok (1 ignored)
- release build : ok
- npm lint + tsc : 0 errors
- Vitest : 265 (+0)
- npm build + size : 6/6

## Modified-file branch coverage (Step 2bis, G9)
- db.rs : `delete_feed_entry_if_tail()` (6 LOC, atomic SQL `NOT EXISTS`) -> tested by `test_feed_publish_orphan_rollback` + `test_feed_orphan_rollback_refuses_if_chained` PASS
- feed_sync.rs : rollback `match db.delete_feed_entry_if_tail` in `insert_and_publish_feed_operation` (5 LOC) -> DB path tested, 3 match arms logged PASS
- feed_sync.rs : rollback `match db.delete_feed_entry_if_tail` in `feed_insert` (5 LOC) -> same pattern PASS
- runtime.rs : no new production method (test only) N/A

## Delta tests (Step 3) — post fix
- Rust : 1309 -> 1315 (+5 Phase B + 1 fix = +6 total)
- Vitest : 265 -> 265 (+0)
- Cumule sprint : Rust 1305 -> 1315 (+10 = +4 Phase A + +6 Phase B+fix), Vitest 265

## Commit body validation (Step 4)
- Phase B : `feat(feed+docs): Sprint 64 Phase B — dette pair 5 items P2` (`c4f5098`)
- Fix : `fix(feed): tail-safe orphan rollback — refuse DELETE if entry is chained` (`490e491`)
- Delta tests coherent : +6 Rust total
- Scope cuts honoured : 12/12 non touches
- Co-Authored-By : present dans les 2 commits

## Research grounding (Step 4bis)
- 4bis-A : sprint64_phase_B_preflight.md S1a documente (3 patterns : Tokio JoinHandle, compensating transaction, pub-sub reconnect), APPROACH-ALIGNED PASS
- 4bis-B : phase dette, pas de nouvelle dep/API — N/A

## Horizon long-terme (Step 4ter)
- Design doc : N/A (dette phase, pas de nouveau module)
- D1..D5 alternatives : N/A (pas de nouvelle decision)
- Solution la plus poussee : tail-safe atomique SQL > unconditional DELETE PASS
- LOC estimates in plan : 0 (plan.md §5 clean)

## Scope cuts verification (Step 5)
- 12/12 scope cuts non touches dans le diff

## Findings

### P1 FIXED — rollback tail-safe (cross-review GPT 5.5)
- **Race identifiee** : `feed_insert` relache le mutex DB entre COMMIT et publish async. Requete concurrente peut chainer sur l'entree avant le rollback. DELETE unconditional cassait la chaine.
- **Fix `490e491`** : `delete_feed_entry_by_hash` -> `delete_feed_entry_if_tail` avec SQL atomique `DELETE ... AND NOT EXISTS (SELECT 1 FROM public_feed WHERE prev_hash = ?1)`. Si entry deja chainee, DELETE = no-op, chaine preservee.
- **Limite** : l'entree orpheline reste en DB (local-only, pas publiee dans iroh-docs). Pas de mecanisme de republish automatique. Classifie P2 carry : `P2-ORPHAN-REPUBLISH-RECOVERY` pour S65.

### P2 carry — `P2-FEED-JOIN-HANDLE-LEAK`
- **Probleme** : `feed_join` HTTP endpoint fait `tokio::spawn` fire-and-forget (feed_sync.rs:597). JoinHandle non stocke, pas de shutdown channel, pas de reconnect sur erreur stream.
- **Impact** : taches orphelines accumulent, shutdown non-graceful, stream error = task meurt silencieusement.
- **Owner** : S65 phase dette feed.
- **Trigger** : toute modification de `feed_join` ou `DaemonHttpState`.
- **Exit** : JoinHandle stocke dans `DaemonHttpState`, shutdown channel partage, reconnect loop (pattern `spawn_feed_subscribe`).

### P2 carry — `P2-VERIFY-ENTRY-VERSION-GUARD`
- **Probleme** : `verify_entry()` ne verifie pas `entry.version <= FEED_FORMAT_VERSION`.
- **Justification carry** : policy pre-launch CLAUDE.md dit "Pas de tolerant decoder multi-version (v == 1 seul)". Aucun noeud tiers, aucune v2 possible pre-launch. Le check devient obligatoire post-v1.0.
- **Owner** : S65 before go-live.
- **Trigger** : tag v1.0 pousse vers origin OU ouverture S65.
- **Exit** : `verify_entry()` reject `version > FEED_FORMAT_VERSION`, `ingest_doc_entry()` check en amont.

### P2 process — LOC estimates kickoff
- kickoff.md contient estimations LOC amont (~15, ~25, ~80, ~5, ~150 LOC) contraires §6.7. Couvert par clause exemption retroactive. Carry process : kickoffs S65+ sans LOC.

### P2 tests — couverture wiring partielle
- Les 6 tests Phase B+fix prouvent les primitives DB et l'arithmetique backoff, pas le wiring integration (publish→fail→rollback, ingest_doc_entry avec rate-limit bypass, stream.next()→None→reconnect).
- Faisable avec mocks DocHandle/DocsEntry (~500 LOC), mais hors scope dette Phase B.
- Non-bloquant : les primitives sont correctes, le wiring est code-reviewed.

### P3 — delta tests plan vs reel
- Plan +4, reel +6 (split stream break en 2 + 1 test tail-safe fix). Non-bloquant.

## Recommendation
- Ready to proceed Phase C : oui (apres ce commit chore(planning))
- Carry-overs S65 obligatoires :
  - `P2-FEED-JOIN-HANDLE-LEAK` (owner S65 dette, trigger feed_join modif)
  - `P2-VERIFY-ENTRY-VERSION-GUARD` (owner S65 go-live, trigger tag v1.0)
  - `P2-ORPHAN-REPUBLISH-RECOVERY` (owner S65, mecanisme republish DB→iroh-docs pour entries local-only)
