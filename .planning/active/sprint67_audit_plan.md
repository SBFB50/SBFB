# Sprint 67 — Audit plan pour Sprint 66

**Sprint audite** : 66 — Durabilite (Arc 1 Fondations, 2/2).
**Tip cloture attendu** : Phase E commit.
**Phase 0 S67 jouera cet audit AVANT la Phase A du Sprint 67.**

---

## Tracks d'audit

### Track A — Suites et compteurs

Verifier que les compteurs annonces dans les commit bodies
Phase A-E correspondent aux compteurs reels mesures par
`cargo nextest run --workspace --locked` et `npm run test:unit`.
Verifier 0 test `#[ignore]` ajoute. Verifier deltas annonces :
Phase A (+7 Rust), Phase B (+1 Rust), Phase C (+5 Rust +1 Vitest),
Phase D (+5 Rust), Phase E (+2 Rust).

### Track B — iroh persistence (D1/D2)

Verifier que `with_data_dir` est bien passe dans `NodeConfig`
(runtime.rs). Verifier que `BlobStore` enum existe dans `node.rs`
avec variants `Mem` et `Fs`. Verifier `FsStore::shutdown()` appele
dans `Node::shutdown()`. Verifier `blobs_store()` retourne `&Store`.
Verifier test `test_persistent_fsstore_survives_reboot` passe.
Verifier `boot_feed_namespace` et `boot_storage_namespace` ont le
fallback list_docs robustesse.

### Track C — Feed republish + orphan recovery (D3)

Verifier que `test_feed_republish_at_boot` passe (entries SQLite →
iroh-docs au boot). Verifier `test_orphan_republish_recovery` passe
(entries SQLite sans iroh-docs correspondence → republish). Verifier
tail-safe skip (entries avec prev_hash invalide sont ignorees, pas
de crash).

### Track D — feed_join handle + shutdown (D3)

Verifier `feed_join_handles` et `feed_join_shutdown` existent sur
`DaemonRuntime`. Verifier `test_feed_join_handles_tracked_and_shutdown`
passe. Verifier le shutdown drain tous les handles (code path dans
`DaemonRuntime::shutdown`).

### Track E — Provenance 3 etats + cross-node (D4/D5)

Verifier que `get_provenance` retourne `status: "absent"` (pas 404)
quand pas de record. Verifier `status: "verified"` et
`status: "failed"` selon verification. Verifier test
`test_provenance_endpoint_absent_status` et
`test_provenance_cross_node_verified` passent. Verifier badge
BrowsedProject 4 etats visuels. Verifier useBridge.ts propage
`status`.

### Track F — RevocationCache persistence

Verifier migration M14 `key_rotations` table dans `db.rs`.
Verifier `insert_key_rotation` et `load_key_rotations` fonctionnent.
Verifier test `test_key_rotation_persistence_survives_reboot` passe.
Verifier `populate_cache` au boot dans `runtime.rs`.

### Track G — Dette pair Phase B

Verifier THREAT_MODEL.md section "Feed surface" avec threats
T-FEED-1..4 presente. Verifier PATTERNS.md pattern raw-op
store+forward documente. Verifier README.md §4.1 note deletions
source code. Verifier SQLite `synchronous=FULL` pragma actif
(test `test_coordinator_db_synchronous_full`).

### Track H — E2E restart + crash recovery

Verifier `test_e2e_restart_full_cycle` passe (blob + feed + curator
+ node_id persistent). Verifier `test_e2e_crash_recovery` passe
(stale running.json + data intact). Verifier les deux tests
couvrent les 5 composants : iroh-docs, iroh-blobs FsStore,
coordinator.db, curator subscriptions, node identity.

### Track I — Scope cuts + carries

Verifier les 14 scope cuts kickoff §7 tous respectes (aucun item
scope-cut livre accidentellement). Verifier les carries reconduits
S67 documentes : P2-A-1 rand, P2-AUDIT-2 iroh transitives,
P2-G-1 exe lock, T-NN+2 iframe Rust-wasm,
P2-THREAT-MODEL-FEED-SURFACE (2/3). Verifier les 2 MANDATORY 3/3
(P2-PROVENANCE-404-BRIDGE + P2-VERIFY-LOCAL-KEY-ONLY) bien CLOSED.
Verifier les 5 carries absorbes tous CLOSED.

---

## Verdicts attendus

| Verdict | Condition |
|---------|-----------|
| PASS | 0 P0, 0 P1, >= 1 P2+ documente |
| CONDITIONAL PASS | 0 P0, 1-2 P1 fixables dans la session |
| FAIL | >= 1 P0 OU >= 3 P1 |
