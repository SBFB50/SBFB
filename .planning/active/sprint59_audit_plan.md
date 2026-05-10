# Sprint 59 — Audit plan

**Ecrit** : 2026-05-10 (Phase E Sprint 58), mis a jour 2026-05-10 (roadmap revision)
**Scope attendu S59** : launcher readiness (bundle + MessageBox + seed demo) + verified deploy E2E + stabilisation — early adopter ready (tag v1.0 reporte S60 pour qualite end user)

---

## Track A — Phase integrity

1. Verifier que chaque phase A-D a son preflight G8 + review dans `.planning/active/`
2. Verifier coherence delta tests cumule (verification.md §2 vs git log)
3. Verifier que les 2 fixes post-Phase D (`7fb817b`, `3ca0ba1`) sont documentes
4. Verifier 4/4 preflights EXECUTE dans `.planning/active/sprint58_phase_{A..D}_preflight.md`

## Track B — MANDATORY resolution

1. P2-JITTER-SCOPE 3/3 : CLOSED Phase A — verifier `jitter_bounds_are_within_range` dans runtime.rs
2. P2-INVITE-U16-WIRE 3/3 : CLOSED Phase A — verifier §P47 dans PATTERNS.md
3. P2-RETAIN-RECENT 2/3 : CLOSED Phase B — verifier `retain_recent_interval` dans runtime.rs
4. P2-BRIDGE-SYNC NEW 1/3 : CLOSED Phase B — verifier `scripts/sync-bridge-sdk.sh` + SHA256 match

## Track C — AppStorage P2P correctness

1. Namespace iroh-docs cree au boot : `boot_storage_namespace()` dans runtime.rs
2. Routing dual backend : `is_replicated_app()` dans storage_api.rs separe iroh-docs vs HashMap
3. Migration M8 `storage_namespaces` table dans db.rs
4. Tombstone filtering : entries `{ "deleted": true }` filtrees dans get_many
5. DocsClient helpers : `get_many_latest_per_key_prefix` (dedup) + `get_latest_by_key` dans docs.rs
6. Ticket Write generation : endpoint `/api/daemon/storage/ticket/{app}` dans http.rs
7. Ticket join : endpoint `/api/daemon/storage/join` dans http.rs

## Track D — Live events + sync E2E

1. `spawn_storage_subscribe()` : subscribe LiveEvent::InsertRemote → AtomicU64 version
2. Bridge polling : `storage_version` methode HTTP + `onStorageUpdate(app, cb)` SDK (3s interval)
3. Ideas Hub refresh on remote sync : `app.js` handles version change
4. Test E2E `test_cross_daemon_storage_sync` : 2 daemons, write A → join B → sync visible
5. Fix `7fb817b` : percent-encode slash in key (test stability)
6. Fix `3ca0ba1` : fire callback on first poll if version > 0 (UX correctness)

## Track E — Dette pair Phase B quality

1. retain_recent timer 60s dans tokio::select! gossip loop : `retain_recent_interval.tick()`
2. Script `scripts/sync-bridge-sdk.sh` : copie web/public/sbfb-bridge.js → examples/*/
3. SHA256 verification post-copie dans le script
4. Aucun test ajoute Phase B (justification : appels methodes existantes deja testees)

## Track F — Carries residuels

| Item | Compteur S59 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 18+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| LT-1 Kudos-v2 | pre-v1.0 | ROADMAP_COMMITMENTS |

## Track G — Roadmap coherence + pre-v1.0 readiness

1. CLAUDE.md S58 CLOSED avec carries S59 corrects
2. HARDENING_ROADMAP last_validated = S58
3. SPRINT_LOG row S58 presente et complete
4. Memory nexus_grid_pivot.md tip = `1734cfb` post-Phase E
5. Roadmap : S59 = launcher readiness + verified deploy E2E + stabilisation (early adopter) → S60 = installer + tag v1.0 (end user)
6. 0 P0/P1 residuels bloquant S59
7. AppStorage P2P Phase 2 (manifest) correctement scope S60+
8. LT-1 Kudos-v2 : confirmer S59 ou S60 selon capacite sprint
