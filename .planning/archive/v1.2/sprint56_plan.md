# Sprint 56 — Plan d'execution detaille

**Tip d'entree** : `e5d6242`
**Date** : 2026-05-09

---

## Etat verifie a l'entree

| Suite | Count |
|---|---|
| Rust nextest | 1216 |
| Rust doctests | 6 passed, 1 ignored |
| Vitest | 250 |
| Playwright | 42 + 2 fail (env) |
| size-limit | 6/6 |
| clippy | 0 warnings |
| fmt | 0 diff |

---

## Decisions Day 0 (gelees)

- **D1** : outbox persistence via table SQLite coordinator.db (M6)
- **D2** : rate-limit per-peer governor GCRA keyed par NodeId
- **D3** : bridge extensions 5 methodes postMessage
- **D4** : dette batch 5 items (forbid-deny, rustfmt, lightcheck,
  build-timeout, remap-path)

---

## Research consulte

- governor 0.10.2 : deja workspace (S21 Phase A), GCRA per-key via
  `DefaultKeyedRateLimiter<K>`, DashMap backend
- rusqlite_migration : 5 migrations existantes, WAL mode,
  pattern M{N} valide
- Bridge postMessage : 4 methodes actuelles (task_submit,
  storage_get, storage_set, pii_redact), format BridgeRequest
  {type, id, method, payload}, SDK sbfb-bridge.js
- Outbox actuel : Vec<Vec<u8>> runtime.rs:1003, rempli via
  GossipCmd::Outbox, replay sur NeighborUp + browse_request +
  periodic republish
- Browse_request : is_browse_request() publish.rs:190, reception
  runtime.rs:1045, 0 rate-limit, keyed par delivered_from NodeId
- Daemon endpoints existants : GET /api/daemon/browse (browse list),
  GET /api/v1/coordinator/health (status), GET /app/{name}/state/{key}
  (storage get), POST /app/{name}/state/{key} (storage set)

---

## Phase A — Outbox gossip persistent SQLite

### §A.1 Scope

Migration M6 dans coordinator.db : table `gossip_outbox` pour
persister les enveloppes gossip du noeud local. Au boot du daemon,
`load_outbox()` pre-remplit le Vec en memoire. Sur chaque publish,
`insert_outbox()` ecrit en DB. Le replay (NeighborUp, browse,
periodic) reste identique — il opere sur le Vec.

CLOSE P2-S53-outbox non-persistant (3/3 MANDATORY).

### §A.2 Fichiers touches

| Fichier | Role |
|---------|------|
| crates/nexus-coordinator-rs/src/db.rs | Migration M6 + load_outbox() + insert_outbox() + clear_outbox() |
| crates/nexus-shell-daemon/src/runtime.rs | Boot load + publish insert |

### §A.3 Tests plan

1. `test_insert_and_load_outbox` — insert 3 envelopes, load, verify 3
2. `test_clear_outbox` — insert, clear, load, verify 0
3. `test_outbox_survives_reopen` — insert, close DB, reopen, load, verify present

### §A.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked -E 'test(outbox)'
# 3 tests pass
cargo clippy --workspace --all-targets --locked -- -D warnings
# 0 warnings
```

### §A.5 Commit cible

```
feat(sprint56): Sprint 56 Phase A — outbox gossip persistent SQLite

## Contexte
[...]

## Fichiers
[table]

## Tests delta cumule
Entree: 1216 Rust / 250 Vitest
Phase A: +3 Rust / +0 Vitest
Cumule: 1219 / 250

## Scope cuts respectes
13/13 non touches.

## Carry-over S57
[si applicable]
```

---

## Phase B — Browse_request rate-limit governor per-peer

### §B.1 Scope

Nouveau module `browse_limiter.rs` dans nexus-shell-daemon-core.
`BrowseRequestLimiter` wrapping governor GCRA keyed par NodeId hex.
Quota 10 req/min/peer. Injection dans le gossip loop runtime.rs
avant replay outbox sur browse_request.

CLOSE P2-S53-browse_request rate-limit (3/3 MANDATORY).

### §B.2 Fichiers touches

| Fichier | Role |
|---------|------|
| crates/nexus-shell-daemon-core/src/browse_limiter.rs | NEW — BrowseRequestLimiter struct |
| crates/nexus-shell-daemon-core/src/lib.rs | pub mod browse_limiter |
| crates/nexus-shell-daemon-core/Cargo.toml | dep governor (workspace) |
| crates/nexus-shell-daemon/src/runtime.rs | Injection check avant replay |

### §B.3 Tests plan

1. `test_allows_under_quota` — 5 requests from same peer, all pass
2. `test_rejects_over_quota` — 15 requests from same peer, some rejected
3. `test_independent_peers` — 2 peers, each under quota, both pass
4. `test_quota_recovers` — peer over quota, wait, passes again

### §B.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon-core --locked -E 'test(browse_limiter)'
# 4 tests pass
cargo clippy --workspace --all-targets --locked -- -D warnings
# 0 warnings
```

### §B.5 Commit cible

```
feat(sprint56): Sprint 56 Phase B — browse_request rate-limit governor per-peer

## Contexte
[...]

## Tests delta cumule
Entree: 1216 Rust / 250 Vitest
Phase A: +3 / +0
Phase B: +4 / +0
Cumule: 1223 / 250

## Scope cuts respectes
13/13 non touches.
```

---

## Phase C — Bridge extensions 5 methodes

Dependencies : Phase A + B completees (outbox + rate-limit).

### §C.1 Scope

Etendre le bridge postMessage avec 5 methodes pour les apps
pre-v1.0. Chaque methode suit le pattern existant :

1. **storage_list** — `GET /app/{name}/state?prefix={prefix}`
   (nouvel endpoint daemon) → liste des cles + valeurs
2. **storage_delete** — `DELETE /app/{name}/state/{key}`
   (nouvel endpoint daemon) → suppression
3. **identity_pubkey** — pubkey locale du noeud, pas de nouvel
   endpoint (lire depuis DaemonHttpState)
4. **node_status** — proxy vers `GET /api/v1/coordinator/health`
   enrichi (peers, uptime, version)
5. **browse_list** — proxy vers `GET /api/daemon/browse` existant

### §C.2 Fichiers touches

| Fichier | Role |
|---------|------|
| crates/nexus-shell-daemon/src/http.rs | 2 nouveaux endpoints (storage list + delete) |
| web/src/bridge/protocol.ts | 5 methodes dans BridgeMethodSchema + payload schemas Zod |
| web/src/bridge/useBridge.ts | 5 cases dans dispatch() |
| web/public/sbfb-bridge.js | 5 nouvelles methodes SDK |

### §C.3 Tests plan

1. `test_storage_list_by_prefix` — Rust integration test (list endpoint)
2. `test_storage_delete_key` — Rust integration test (delete endpoint)
3. `test_bridge_storage_list_dispatch` — Vitest (bridge handler)
4. `test_bridge_storage_delete_dispatch` — Vitest (bridge handler)
5. `test_bridge_identity_pubkey` — Vitest (bridge handler)
6. `test_bridge_node_status` — Vitest (bridge handler)
7. `test_bridge_browse_list` — Vitest (bridge handler)

### §C.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon --locked -E 'test(storage_list) | test(storage_delete)'
# 2 tests pass
(cd web && npm run test:unit)
# >= 255 (250 + 5 bridge tests minimum)
npm run lint && npx tsc --noEmit -p tsconfig.app.json
# 0 error
```

### §C.5 Commit cible

```
feat(sprint56): Sprint 56 Phase C — bridge extensions 5 methodes

## Contexte
[...]

## Tests delta cumule
Entree: 1216 Rust / 250 Vitest
Phase A: +3 / +0
Phase B: +4 / +0
Phase C: +2 / +5
Cumule: 1225 / 255

## Scope cuts respectes
13/13 non touches.
```

---

## Phase D — Dette pair P2 batch

Dependencies : Phase C completee.

### §D.1 Scope

Sprint pair → phase dette obligatoire (§6.2.1 Regle 1). 5 items :

1. **P2-S54-forbid-deny-doc** (2/3) : ajouter dans
   `docs/rust/PATTERNS.md` un §P-NN documentant la convention
   deny vs forbid et l'incompatibilite cfg_attr+forbid (edition
   2024 wrapping set_var unsafe).

2. **P2-S54-rustfmt-drift-sessions** (2/3) : investiguer la cause
   du drift (rustfmt version 1.94 vs 1.95 entre dev et CI) et
   documenter la solution (pin rustfmt nightly dans CI ou
   configurer rustfmt.toml stable_features).

3. **P2-S54-lightcheck-edition-faux-positif** (2/3) : corriger le
   faux-positif du hook lightcheck lie a l'edition 2024 (grep
   sur content markers dans les fichiers planning). Investigation
   + fix hook ou exemption documentee.

4. **P2-BUILD-TIMEOUT** (1/3) : ajouter un parametre `timeout:
   Duration` a `execute_build()` avec default 30min. Appliquer
   via `tokio::time::timeout()` wrapping la commande cargo build.

5. **P2-REMAP-PATH** (1/3) : ajouter `--remap-path-prefix` au
   cargo build dans le build executor. Env var ou argument direct.
   Aligne avec reproducible-builds.org.

### §D.2 Fichiers touches

| Fichier | Role |
|---------|------|
| docs/rust/PATTERNS.md | §P-NN forbid-deny convention |
| .claude/hooks/ ou .claude/skills/ | lightcheck fix |
| crates/nexus-worker-core/src/build_executor.rs | timeout + remap-path |

### §D.3 Tests plan

1. `test_build_timeout_expires` — build with 1ms timeout → error
2. `test_remap_path_flag_present` — verify --remap-path-prefix in command args

### §D.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-worker-core --locked -E 'test(build)'
# tests pass (+ timeout + remap tests)
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

### §D.5 Commit cible

```
feat(sprint56): Sprint 56 Phase D — dette pair P2 batch

## Contexte
[...]

## Tests delta cumule
Entree: 1216 Rust / 250 Vitest
Phase A: +3 / +0
Phase B: +4 / +0
Phase C: +2 / +5
Phase D: +2 / +0
Cumule: 1227 / 255

## Scope cuts respectes
13/13 non touches.

## Items CLOSED
- P2-S54-forbid-deny-doc (2/3) CLOSE
- P2-S54-rustfmt-drift-sessions (2/3) CLOSE
- P2-S54-lightcheck-edition-faux-positif (2/3) CLOSE
- P2-BUILD-TIMEOUT (1/3) CLOSE
- P2-REMAP-PATH (1/3) CLOSE
```

---

## Phase E — Wrap-up + verification + audit plan S57

### §E.1 Scope

- CLAUDE.md : update S56 CLOSED, compteurs, carries S57
- HARDENING_ROADMAP : update last_validated S56
- SPRINT_LOG.md : ligne S56
- verification.md : 24+ fail-fast rows
- sprint57_audit_plan.md : 7+ tracks (outbox integrity, rate-limit
  effectiveness, bridge security, dette resolution, CI health,
  process compliance, carries accuracy)

### §E.2 Commit cible

```
chore(sprint56): Phase E — wrap-up + verification + audit plan S57
```

---

## Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1227, 0 fail | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | npm lint | `npm run lint` (web/) | 0 error | |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | |
| 8 | Vitest | `npm run test:unit` (web/) | >= 255 | |
| 9 | npm build | `npm run build` (web/) | ok | |
| 10 | size-limit | `npm run size` (web/) | 6/6 | |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | |
| 12 | Phase A preflight G8 | verdict | EXECUTE | |
| 13 | Phase A review | verdict | PASS | |
| 14 | Phase B preflight G8 | verdict | EXECUTE | |
| 15 | Phase B review | verdict | PASS | |
| 16 | Phase C preflight G8 | verdict | EXECUTE | |
| 17 | Phase C review | verdict | PASS | |
| 18 | Phase D preflight G8 | verdict | EXECUTE | |
| 19 | Phase D review | verdict | PASS | |
| 20 | outbox survives restart | test | pass | |
| 21 | rate-limit rejects spam | test | pass | |
| 22 | bridge storage_list | test | pass | |
| 23 | bridge identity_pubkey | test | pass | |
| 24 | Scope cuts | 13/13 respectes | all checked | |
| 25 | Delta tests | cumule documente | documented | |

---

## Git plan

1. `chore(planning): Sprint 56 kickoff + plan + design review G1`
2. `chore(planning): Sprint 56 Phase A preflight G8 EXECUTE`
3. `feat(sprint56): Sprint 56 Phase A — outbox gossip persistent SQLite`
4. `chore(planning): Sprint 56 Phase A review PASS`
5. `chore(planning): Sprint 56 Phase B preflight G8 EXECUTE`
6. `feat(sprint56): Sprint 56 Phase B — browse_request rate-limit governor per-peer`
7. `chore(planning): Sprint 56 Phase B review PASS`
8. `chore(planning): Sprint 56 Phase C preflight G8 EXECUTE`
9. `feat(sprint56): Sprint 56 Phase C — bridge extensions 5 methodes`
10. `chore(planning): Sprint 56 Phase C review PASS`
11. `chore(planning): Sprint 56 Phase D preflight G8 EXECUTE`
12. `feat(sprint56): Sprint 56 Phase D — dette pair P2 batch`
13. `chore(planning): Sprint 56 Phase D review PASS`
14. `chore(sprint56): Phase E — wrap-up + verification + audit plan S57`

---

## Scope cuts

1. LT-7 Tier 3 (N builders, auto-deploy) — S57+
2. E2E multi-noeuds automatise — S57 (3/3 MANDATORY)
3. windows-test-cfg-unix CI — S57 (3/3 MANDATORY)
4. Protocol Explorer MVP — S57
5. Ideas Hub MVP — S57
6. Outbox rotation/compaction TTL — S57+
7. Rate-limit policy hot-reload TOML — S57+
8. Bridge batch operations — S57+
9. Podman rootless build sandbox — S57+
10. Build log streaming — S57+
11. P2-JITTER-SCOPE test integration — S57
12. P2-INVITE-U16-WIRE doc post-v1.0 — post-v1.0
13. LT-1 Kudos-v2 fairness reform — S58+

---

## Risks (R1..R5)

Cf. kickoff §9. Mitigations listees.

---

## Checkpoint de cloture

1. 25/25 fail-fast rows vertes
2. 14 commits (4 feat + 8 chore planning + 1 chore wrap-up
   + fix si necessaire)
3. verification.md + sprint57_audit_plan.md ecrits
4. CLAUDE.md + HARDENING_ROADMAP + SPRINT_LOG mis a jour
5. 2 items 3/3 MANDATORY FERMES
6. 5 items P2 dette FERMES (sprint pair)
7. 5 methodes bridge fonctionnelles
