Vérification ciblée exécutée : `cargo test --locked -p nexus-shell-daemon e2e -- --nocapture` -> OK, 3 tests passés, dont `runtime::tests::test_e2e_restart_full_cycle` et `runtime::tests::test_e2e_crash_recovery`.

### Livrable 1 : `test_e2e_restart_full_cycle`
- Statut : PARTIEL
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2110`
- Evidence :
```rust
2110:    async fn test_e2e_restart_full_cycle() {
2115:        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
2117:        let node_id_1 = rt1.node.as_ref().unwrap().node_id();
2120:        rt1.curator_runtime()
2124:        let blobs = nexus_core_rs::BlobsClient::new(rt1.node.as_ref().unwrap().blobs_store());
```
```rust
2150:        assert_eq!(node_id_1, node_id_2, ...);
2155:        assert!(rt2.curator_runtime().is_subscribed(&kp.public_bytes()), ...);
2162:        assert_eq!(data2, b"e2e-restart-payload", ...);
2167:        assert!(rt2.feed_handle.is_some(), ...);
2176:        assert_eq!(entries.len(), 1, ...);
2183:        assert_eq!(rt2.revocation_cache().read().unwrap().len(), 0, ...);
```
- Si GAP : le test couvre bien restart, node_id, curator, blob FsStore, SQLite feed entry, revocation cache. Point partiel strict : `feed_handle active` est seulement vérifié par `is_some()` (`runtime.rs:2167-2170`), pas par une preuve de liveness type `!is_finished()`.

### Livrable 2 : `test_e2e_crash_recovery`
- Statut : PARTIEL
- Fichier(s) : `crates/nexus-shell-daemon/src/runtime.rs:2193`
- Evidence :
```rust
2193:    async fn test_e2e_crash_recovery() {
2199:        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
2210:            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
2216:        rt1.shutdown().await.unwrap();
2230:        raw_write_running(&stale, &running_json).unwrap();
```
```rust
2234:        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
2236:        assert!(rt2.feed_handle.is_some(), ...);
2245:            assert_eq!(entries.len(), 1, ...);
2252:        rt2.shutdown().await.unwrap();
2253:        assert!(!running_json.exists(), ...);
```
- Si GAP : même réserve que livrable 1 : `feed_handle active` est vérifié comme présence de handle, pas comme tâche encore vivante. Le reste est réellement asserté.

### Livrable 3 : `sprint66_verification.md`
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint66_verification.md:12`, `:14`, `:19`, `:21`, `:37`, `:47`
- Evidence :
```md
14:| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1349 | PASS (1349) |
19:| 8 | Vitest | `(cd web && npm run test:unit)` | >= 269 | PASS (269) |
21:| 10 | size-limit | `(cd web && npm run size)` | 6/6 | PASS |
37:| 26 | Vitest badge absent | `(cd web && npm run test:unit)` includes badge test | PASS | PASS |
39:| 28 | audit_plan S67 | `test -f .planning/active/sprint67_audit_plan.md` | exists | PASS |
```
- Controle additionnel : 28 lignes checklist numérotées, 0 non-PASS. Compteurs sortie présents aux lignes `47-49`.

### Livrable 4 : `sprint67_audit_plan.md`
- Statut : CONFIRME
- Fichier(s) : `.planning/active/sprint67_audit_plan.md:11`, `:20`, `:30`, `:38`, `:45`, `:55`, `:62`, `:70`, `:78`
- Evidence :
```md
20:### Track B — iroh persistence (D1/D2)
30:### Track C — Feed republish + orphan recovery (D3)
45:### Track E — Provenance 3 etats + cross-node (D4/D5)
55:### Track F — RevocationCache persistence
70:### Track H — E2E restart + crash recovery
```
- Les 9 tracks A-I existent. Couverture demandée confirmée : persistence B, feed C/D, provenance E, RevocationCache F, dette G, E2E H, scope cuts I.

### Livrable 5 : `CLAUDE.md` état actuel
- Statut : CONFIRME
- Fichier(s) : `CLAUDE.md:150`, `:151`, `:166`, `:167`, `:170`, `:174`
- Evidence :
```md
150:## Etat actuel
151:- **Sprints 0-66 CLOSED**, v2.1 ouverte. **Tag v1.0 pose.**
166:  Phase E test E2E restart full cycle + crash recovery + wrap-up.
167:  Arc 1 Fondations COMPLET (S65 contrat public + S66 durabilite).
170:- **~1624 tests total** (1349 Rust / 269 Vitest / 6/6 size-limit)
174:- Carry S67 :
```

### Livrable 6 : `docs/claude/SPRINT_LOG.md` row S66
- Statut : CONFIRME
- Fichier(s) : `docs/claude/SPRINT_LOG.md:19`
- Evidence :
```md
19:| 66 | DONE + 5 phases A-E livrees sur theme durabilite Arc 1 Fondations 2/2 ...
19:... Phase E test E2E restart full cycle blob+feed+curator+node_id persistent ...
19:... test crash recovery stale running.json + verification.md 28/28 + sprint67_audit_plan.md 9 tracks ...
19:... Rust +16 tests (1333→1349 ...) + Vitest +1 (268→269 ...) ...
```

## Résumé final
- Total livrables : 6
- Confirmés : 4
- Gaps : 0
- Partiels : 2