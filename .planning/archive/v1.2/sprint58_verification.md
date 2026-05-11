# Sprint 58 — Verification

**Date** : 2026-05-10
**Tip Phase D** : `2957719` | **Tip fixes** : `3ca0ba1` (2 fix post-D)

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1240, 0 fail | 1240 pass, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ok (6 pass, 1 ignored) |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error | 0 error |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 256 | 256 pass |
| 9 | npm build | `npm run build` (web/) | ok | ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 | 6/6 |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | clean |
| 12 | Phase A preflight G8 | verdict | EXECUTE | EXECUTE |
| 13 | Phase A review | verdict | PASS | PASS (1 P2) |
| 14 | Phase B preflight G8 | verdict | EXECUTE | EXECUTE |
| 15 | Phase B review | verdict | PASS | PASS (1 P2) |
| 16 | Phase C preflight G8 | verdict | EXECUTE | EXECUTE |
| 17 | Phase C review | verdict | PASS | PASS (2 P2, 1 P3) |
| 18 | Phase D preflight G8 | verdict | EXECUTE | EXECUTE |
| 19 | Phase D review | verdict | PASS | PASS (2 P2, 1 P3) |
| 20 | MANDATORY JITTER-SCOPE | test bounds present | present | `jitter_bounds_are_within_range` in runtime.rs |
| 21 | MANDATORY INVITE-U16-WIRE | §P47 present | present | `grep "§P47" docs/rust/PATTERNS.md` |
| 22 | retain_recent timer | periodic call in gossip loop | present | `retain_recent_interval` in runtime.rs |
| 23 | sync-bridge-sdk.sh | script + SHA256 match | present | `scripts/sync-bridge-sdk.sh` exit 0 |
| 24 | Storage namespace M8 | `storage_namespaces` table | present | `grep "storage_namespaces" crates/nexus-coordinator-rs/src/db.rs` |
| 25 | Storage iroh-docs routing | sbfb-ideas → iroh-docs | present | `is_replicated_app("sbfb-ideas")` in storage_api.rs |
| 26 | Live events subscribe | `spawn_storage_subscribe` | present | runtime.rs storage subscribe spawn |
| 27 | Bridge storage_version | SDK method | present | `onStorageUpdate` in sbfb-bridge.js |
| 28 | E2E storage sync test | `test_cross_daemon_storage_sync` | present | multi_daemon.rs (gated SBFB_INTEGRATION) |
| 29 | Scope cuts | 12/12 respectes | all checked | all checked |
| 30 | Delta tests | cumule documente | documented | +8 Rust (1232→1240), +0 Vitest (256→256) |
| 31 | Fix post-D #1 | percent-encode slash key | committed | `7fb817b` |
| 32 | Fix post-D #2 | fire onStorageUpdate first poll | committed | `3ca0ba1` |

---

## §2 Delta tests cumule

| Suite | Avant S58 | Apres S58 | Delta | Detail |
|---|---|---|---|---|
| Rust nextest | 1232 | 1240 | +8 | Phase A +1 (jitter bounds), Phase C +6 (iroh-docs CRUD + namespace + tombstone + routing), Phase D +1 (E2E storage sync) |
| Vitest | 256 | 256 | +0 | Aucune phase ne touche le frontend React |
| Playwright | 42+2f | 42+2f | +0 | inchange (2 env fail pre-existant pyproject.toml absent) |
| size-limit | 6/6 | 6/6 | +0 | inchange |
| **Total** | ~1494 | ~1502 | +8 | |

---

## §3 Scope cuts honoured (12/12)

1. Verified deploy E2E from repos Git separes — S59 : non touche
2. Protocol Explorer F3 avance (gossip stats) — S59+ : non touche
3. Protocol Explorer F4 (tutoriel interactif) — S59+ : non touche
4. Ideas Hub F3 (lier repos Git) — S59 : non touche
5. Ideas Hub F4 (groupes de travail) — post-v1.0 : non touche
6. Ideas Hub F5 (integration reseau) — post-v1.0 : non touche
7. Kudos-weighted voting — S59+ : non touche
8. AppStorage Phase 2 (namespace per manifest) — S59+ : non touche
9. AppStorage Phase 3 (optimisations, purge) — post-v1.0 : non touche
10. LT-1 Kudos-v2 fairness reform — S59 pre-v1.0 : non touche
11. LT-7 Tier 3 (N builders, auto-deploy) — S59+ : non touche
12. Ticket Write rotation dynamique (Option B/C) — post-v1.0 : non touche

---

## §4 MANDATORY items resolution

| Item | Compteur entree | Resolution | Commit |
|---|---|---|---|
| P2-JITTER-SCOPE | 3/3 MANDATORY | CLOSED Phase A | `b449d62` (feat) |
| P2-INVITE-U16-WIRE | 3/3 MANDATORY | CLOSED Phase A | `b449d62` (feat) |
| P2-RETAIN-RECENT | 2/3 | CLOSED Phase B | `c287c61` (feat) |
| P2-BRIDGE-SYNC | NEW 1/3 | CLOSED Phase B | `c287c61` (feat) |

---

## §5 Findings carry-over for memory

- **AppStorage P2P operationnel** : Ideas Hub data repliquee entre
  2+ noeuds via iroh-docs namespace. Ticket Write dans archive zip.
  Live events via subscribe InsertRemote + AtomicU64 version + SDK
  polling 3s. Phase 2 (manifest-based generalisation) et Phase 3
  (purge/optimisation) reportees S59+/post-v1.0.
- **Sprint pair dette obligatoire respectee** : retain_recent timer
  60s + sync-bridge-sdk.sh SHA256 (Phase B).
- **Dual backend stable** : HashMap+SQLite local (apps non repliquees)
  et iroh-docs (apps repliquees, detect par nom hardcode). Routing
  clean dans storage_api.rs.
- **2 fixes post-Phase D** : percent-encode slash dans key test E2E +
  fire onStorageUpdate callback au 1er poll si version > 0. Les 2
  sont des bug fixes mineurs post-review, pas des features.
- **Test E2E gated** : test_cross_daemon_storage_sync derriere
  SBFB_INTEGRATION=1. Necessite 2 instances daemon. Vert en local.
- **futures-lite 2.3** : dep directe ajoutee Phase D pour
  StreamExt sur subscribe stream.
