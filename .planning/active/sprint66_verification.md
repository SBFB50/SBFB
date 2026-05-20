# Sprint 66 — Verification (Durabilite)

**Ecrit** : 2026-05-20.
**Tip master** : a remplir post-commit Phase E.

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Resultat |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | PASS |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | PASS |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1349 | PASS (1349) |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | PASS |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | PASS |
| 6 | npm lint | `(cd web && npm run lint)` | 0 errors | PASS |
| 7 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors | PASS |
| 8 | Vitest | `(cd web && npm run test:unit)` | >= 269 | PASS (269) |
| 9 | npm build | `(cd web && npm run build)` | ok | PASS |
| 10 | size-limit | `(cd web && npm run size)` | 6/6 | PASS |
| 11 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean | PASS |
| 12 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean | PASS |
| 13 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical | PASS |
| 14 | FsStore persistent | `cargo nextest run -E 'test(persistent_fsstore)' -p nexus-core-rs` | PASS | PASS |
| 15 | namespace reopen | `cargo nextest run -E 'test(boot_.*namespace.*persistent)' -p nexus-shell-daemon` | PASS | PASS |
| 16 | SQLite sync FULL | `cargo nextest run -E 'test(synchronous_full)' -p nexus-coordinator-rs` | PASS | PASS |
| 17 | feed republish | `cargo nextest run -E 'test(feed_republish)' -p nexus-shell-daemon` | PASS | PASS |
| 18 | provenance absent | `cargo nextest run -E 'test(provenance_.*absent)' -p nexus-shell-daemon` | PASS | PASS |
| 19 | provenance cross | `cargo nextest run -E 'test(provenance_cross)' -p nexus-shell-daemon` | PASS | PASS |
| 20 | feed_join handle | `cargo nextest run -E 'test(feed_join_handle)' -p nexus-shell-daemon` | PASS | PASS |
| 21 | orphan recovery | `cargo nextest run -E 'test(orphan_republish)' -p nexus-shell-daemon` | PASS | PASS |
| 22 | RevocationCache | `cargo nextest run -E 'test(key_rotation_persistence)' -p nexus-shell-daemon` | PASS | PASS |
| 23 | E2E restart | `cargo nextest run -E 'test(e2e_restart)' -p nexus-shell-daemon` | PASS | PASS |
| 24 | THREAT_MODEL feed | `grep -q "T-FEED" docs/security/THREAT_MODEL.md` | present | PASS |
| 25 | PATTERNS raw-op | `grep -q "Raw-op" docs/rust/PATTERNS.md` | present | PASS |
| 26 | Vitest badge absent | `(cd web && npm run test:unit)` includes badge test | PASS | PASS |
| 27 | verification.md | `test -f .planning/active/sprint66_verification.md` | exists | PASS |
| 28 | audit_plan S67 | `test -f .planning/active/sprint67_audit_plan.md` | exists | PASS |

---

## §2 Compteurs sortie

| Suite | Entree S66 | Sortie S66 | Delta |
|---|---|---|---|
| Rust nextest | 1333 | 1349 | +16 |
| Vitest | 268 | 269 | +1 |
| size-limit | 6/6 | 6/6 | 0 |
| **Total** | **~1607** | **~1624** | **+17** |

### Delta par phase

| Phase | Rust delta | Vitest delta | Detail |
|---|---|---|---|
| A-D | +14 | +1 | (1333→1347 Rust, 268→269 Vitest) cf. commit bodies Phases A-D |
| E | +2 | +0 | test_e2e_restart_full_cycle + test_e2e_crash_recovery |
| **Total** | **+16** | **+1** | **1333→1349 Rust, 268→269 Vitest** |

Le plan estimait +16 Rust / +1 Vitest. Delta reel : +16 / +1. Match exact.
Compteur cumule reel : 1349 nextest + 269 Vitest (verifies).

---

## §3 Phases livrees

| Phase | Commit title | Tests |
|---|---|---|
| A | `feat(persistence): Sprint 66 Phase A — iroh data_dir + FsStore` | +7 Rust |
| B | `feat(dette): Sprint 66 Phase B — dette pair + THREAT_MODEL feed + PATTERNS raw-op` | +1 Rust |
| C | `feat(feed+provenance): Sprint 66 Phase C — feed republish + provenance cross-node` | +5 Rust +1 Vitest |
| D | `feat(persistence): Sprint 66 Phase D — orphan recovery + RevocationCache SQLite` | +5 Rust |
| E | `docs(sprint66): Sprint 66 Phase E — E2E restart test + wrap-up` | +2 Rust |

---

## §4 Scope cuts respectes

14/14 scope cuts du kickoff §7 respectes :
1. CuratorVouched/CuratorDisendorsed → S67
2. BuildQuorumReached feed → S67+
3. Quarantine feed hot path → S67+
4. Age witness gate → S67+
5. T1 CONFIRM_PROMPT complet → post-pilote S69
6. SBFB.json v2 code → S67 Phase A
7. node_id deprecation deploy.rs → S67 Phase A
8. Factory template scaffold → S67 Phase B+
9. Fuzzing cargo-fuzz/proptest → post-audit
10. CLI verify-release → S67+
11. VerificationDetail niveau 3 → S67+
12. Playwright E2E re-ecriture → S69
13. Feed format version bump → post-launch
14. Multi-curator trust overlay → S67 Phase D stretch

---

## §5 Carries S67

### Reconduits

| Item | Reports | Justification |
|---|---|---|
| P2-A-1 rand blocker | exemption externe | upstream rand 0.9 non publie |
| P2-AUDIT-2 iroh transitives | exemption externe | iroh 0.98 pinne |
| P2-G-1 exe lock intermittent | monitoring | non-reproductible depuis S62 |
| T-NN+2 iframe Rust-wasm | bloque upstream | toolchain gaps non resolus |
| P2-THREAT-MODEL-FEED-SURFACE | 2/3 | traite Phase B S66 (1/3→2/3). Prochain sprint 3/3 MANDATORY si non traite S67 |

### Items CLOSED S66

| Item | Phase | Exit |
|---|---|---|
| P2-PROVENANCE-404-BRIDGE | Phase C | 3/3 MANDATORY → CLOSED. Provenance retourne status absent/verified/failed |
| P2-VERIFY-LOCAL-KEY-ONLY | Phase C | 3/3 MANDATORY → CLOSED. Verification utilise node_id du record |
| P2-FEED-JOIN-HANDLE-LEAK | Phase C | 2/3→3/3→CLOSED. JoinHandle tracked + shutdown channel |
| P2-ORPHAN-REPUBLISH-RECOVERY | Phase D | 2/3→3/3→CLOSED. Orphan entries republishees au boot |
| P2-S65-CHORE-MISCLASSIFIED | Phase B | 1/3→CLOSED. README.md §4.1 amende |
| P2-S65-RAWOP-PATTERN-UNDOC | Phase B | 1/3→CLOSED. PATTERNS.md raw-op pattern ajoute |

### LT items

| Item | Etat |
|---|---|
| LT-2 Radicle | trigger PENDING (tag v1.0 pose, pas pousse) |
| LT-5 redundancy persistence | reclassifie S26, hors-sprint |
| LT-7 self-hosted build | Tier 1+2 DONE. Worker quorum E2E carry post-tag |
