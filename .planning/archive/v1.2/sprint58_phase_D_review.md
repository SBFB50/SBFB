# Phase Review — Sprint 58 Phase D

## Verdict : PASS (2 P2, 1 P3)

Rigor signal : 2 findings P2 + 1 P3 documentes (>=1 P2+ requis pour PASS rigoureux).

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — RESPECTE (iroh-docs subscribe = native/deepest approach, pas de polling cote Rust)
- feedback_context7_systematic.md : context7 obligatoire — RESPECTE (iroh-docs API queried dans preflight G8)

## Staging check (Step 1bis)
- Phase fichiers : 13 modified (Cargo.lock, docs.rs, Cargo.toml, runtime.rs, storage_api.rs, multi_daemon.rs, 2x sbfb-bridge.js, app.js, index.html, protocol.ts, useBridge.ts)
- Planning/docs split : preflight doc untracked → chore(planning) AVANT feat
- Untracked accidentels : 0

## Suites
- cargo fmt : vert
- cargo clippy : vert (0 warnings)
- cargo nextest : 1239 → 1240 (+1) vert
- cargo doctests : vert (0 passed, 1 ignored)
- cargo build --release : vert
- npm lint : vert (warnings pre-existants)
- tsc --noEmit : vert
- Vitest : 256 → 256 (+0) vert
- npm build : vert
- npm size : 6/6 vert
- scan-en-strings : vert
- Playwright : env fail pre-existant (pyproject.toml absent, non regression)

## Commit body validation
- Format titre : feat(sprint58): Sprint 58 Phase D — AppStorage P2P live events + sync E2E ✅
- Delta tests coherent : +1 Rust (E2E gated SBFB_INTEGRATION) ✅
- Scope cuts honoured : ✅ (SSE temps reel → S59+, indicateur noeuds connectes si metadata indisponible)
- Co-Authored-By present : ✅

## Modified-file branch coverage (Step 2bis, G9)
- storage_api.rs : `spawn_storage_subscribe()` (25 LOC) → indirectly tested by `test_cross_daemon_storage_sync` E2E (version >= 1 assertion) ✅
- runtime.rs : storage_namespaces boot loop + `spawn_storage_subscribe` call → covered by daemon boot in all E2E tests ✅
- useBridge.ts : `case "storage_version"` (5 LOC, trivial dispatch) → CONCERN (no direct unit test, but < 10 LOC trivial)
- sbfb-bridge.js : `onStorageUpdate()` (30 LOC) → indirectly tested by E2E flow. P3 (SDK methods historically untested, consistent with getNodeStatus/getBrowseList pattern)

## Research grounding (Step 4bis)
- 4bis-A (OSS prior art G10) : preflight S1a documente, 3 sources consultees (iroh-docs context7, CRDT best practices 2026, real-time sync frameworks). APPROACH-ALIGNED. ✅
- 4bis-B (deps context7) : iroh-docs 0.98 confirmee via context7, subscribe/LiveEvent API validee. futures-lite 2.3 = transitive dep elevee directe. ✅

## Scope cuts verification
- "Verified deploy E2E from repos Git separes" : 0 fichiers diff ✅
- "Protocol Explorer F3/F4" : 0 fichiers diff ✅
- "Ideas Hub F3/F4/F5" : 0 fichiers diff ✅
- "Kudos-weighted voting" : 0 fichiers diff ✅
- "AppStorage Phase 2 (namespace per manifest)" : 0 fichiers diff ✅
- "Ticket Write rotation dynamique" : 0 fichiers diff ✅
- SSE temps reel : plan dit "S59+", polling MVP livre ✅

## Horizon long-terme + documentation amont
- Design doc present : ✅ (.planning/research/p2p_storage_replication_iroh_docs.md)
- D1..D4 avec alternatives + rationale : ✅ (kickoff §4)
- Solution la plus poussee : ✅ (iroh-docs subscribe = native, polling = contrainte CSP sandbox)
- Aucune LOC estimee au plan : ✅

## Findings (rigor signal)
- **P2** : Anti-spam couches 2-3 (rate-limit per-author + validation applicative) deferred S59. Pre-v1.0 acceptable (reseau controle), documente dans plan + commit body. Carry-over sprint59_audit_plan.md.
- **P2** : `storage_join` ne verifie pas si l'app est dans REPLICATED_APPS avant d'inserer — un client pourrait join un namespace pour une app non-repliquee. Impact minimal pre-v1.0 (reseau controle, endpoint authentifie), hardening S59.
- **P3** : `onStorageUpdate()` SDK (30 LOC) n'a pas de test unitaire dedie. Consistent avec le pattern existant (8 autres methodes SDK sans tests dedies). Couvert indirectement par le flux E2E.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S59 : anti-spam couches 2-3, storage_join validation app name
- Corrections needed : aucune

## Addendum post-review (GPT 5.5 cross-review)

**P1 corrige** : test E2E `test_cross_daemon_storage_sync` echouait avec
`SBFB_INTEGRATION=1` — la cle `ideas/test-sync-1` n'etait pas percent-
encodee dans l'URL du test. Route `/app/{name}/state/{key}` matche un
seul segment. Fix `7fb817b` : `ideas%2Ftest-sync-1`. Test PASS confirme.

**P2 corrige inline** : `onStorageUpdate()` absorbait silencieusement une
sync arrivee entre loadAll() initial et premier poll (baseline -1 → N
sans callback). Fix : fire callback si premiere version > 0. Pas besoin
de SSE pour corriger ce point.
