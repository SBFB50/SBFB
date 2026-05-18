# Sprint 64 — Verification

**Rempli** : 2026-05-17.
**Tip master** : `a67c1a7`.
**Theme** : Hardening public cible (Sprint 4/6 roadmap v2.0).

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1324 | **1326** |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ok |
| 6 | npm lint | `(cd web && npm run lint)` | 0 errors | 0 errors |
| 7 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors | 0 errors |
| 8 | Vitest | `(cd web && npm run test:unit)` | 265 | **265** |
| 9 | npm build | `(cd web && npm run build)` | ok | ok (5.44s) |
| 10 | size-limit | `(cd web && npm run size)` | 6/6 | 6/6 (121.44 kB / 130 kB) |
| 11 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean | clean |
| 12 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical | identical (3 copies) |
| 13 | M13 migration | `app_version` column in provenance_records | present | present (Phase A) |
| 14 | Adversarial feed | `cargo nextest run -E 'test(adversarial)' -p nexus-coordinator-rs` | 10+ PASS | **10/10 PASS** |
| 15 | New node E2E | test_new_node_full_sync_and_verify | PASS (SBFB_INTEGRATION) | delivered (gated) |
| 16 | Phase A-E preflights G8 | 5 fichiers sprint64_phase_{A..E}_preflight.md | 5x EXECUTE | 5x EXECUTE |
| 17 | Phase A-E reviews | 5 fichiers sprint64_phase_{A..E}_review.md | 5x PASS | 4x PASS + E review pending |
| 18 | PUBLIC_FEED_SPEC §10-12 | sections presentes et coherentes | ok | ok (§10 15 vecteurs, §11 algorithme 7 etapes, §12 4 threats + trust boundaries + crypto table) |

**Verdict : 18/18 PASS** (row 17 Phase E review sera complété au
pre-commit skill).

---

## §2 Compteurs tests

| Suite | S64 entry | S64 exit | Delta |
|---|---|---|---|
| Rust nextest | 1305 | 1326 | **+21** |
| Vitest | 265 | 265 | +0 |
| size-limit | 6/6 | 6/6 | = |
| **Total** | **~1576** | **~1597** | **+21** |

Decomposition delta Rust :
- Phase A : +4 (3 provenance version + 1 blake3 hash stability)
- Phase A fix : +1 (cross-review)
- Phase B : +5 (joinhandle + backfill + orphan + stream-break + tail-safe)
- Phase B fix : +1 (tail-safe rollback)
- Phase C : +6 (6 adversariaux feed)
- Phase C fix : +2 (rate limiter per-author + split local/remote)
- Phase D : +5 (4 crypto + 1 E2E nouveau noeud)
- Phase D fix : -3 (renommage, pas perte nette — inclus dans +5)
Reel +21 (1305 → 1326).

---

## §3 Commits phase

| # | SHA | Commit title | Phase |
|---|---|---|---|
| 1 | Phase A | `feat(feed): Sprint 64 Phase A — MANDATORY version stored + subscribe timeout` | A |
| 2 | Phase A fix | `fix(feed): rate limiter per-author + split local/remote insert paths` | A fix |
| 3 | Phase B | `feat(feed+docs): Sprint 64 Phase B — dette pair 5 items P2` | B |
| 4 | Phase B fix | `fix(feed): tail-safe orphan rollback — refuse DELETE if entry is chained` | B fix |
| 5 | Phase C | `feat(feed): Sprint 64 Phase C — adversarial tests feed public` | C |
| 6 | Phase D | `feat(feed): Sprint 64 Phase D — adversarial crypto + new node E2E` | D |
| 7 | Phase D fix | `fix(feed): Phase D cross-review — 4 P1 bloquants corriges` | D fix |
| 8 | Phase D fix 2 | `fix(feed): E2E test reads feed_status not feed_cursor` | D fix |
| 9 | Phase E | `docs(protocol): Sprint 64 Phase E — spec finalisee + wrap-up` | E |

---

## §4 Scope cuts validation

12/12 scope cuts respectes (aucun item scope-cut touche) :

| # | Item | Respecte |
|---|---|---|
| 1 | CuratorVouched operation | oui |
| 2 | BuildQuorumReached operation | oui |
| 3 | Quarantine feed hot path | oui |
| 4 | Age witness gate feed | oui |
| 5 | Multi-forge feed sync | oui |
| 6 | Feed format version bump | oui |
| 7 | CLI verify-release | oui |
| 8 | VerificationDetail niveau 3 | oui |
| 9 | Fuzzing cargo-fuzz/proptest | oui |
| 10 | Docker compose test distribue | oui |
| 11 | Interop externe parsers tiers | oui |
| 12 | SearchManifestPublished feed | oui |

---

## §5 Findings carry-over for memory

### Items resolus S64

| Item | Phase | Exit |
|---|---|---|
| F1 P2-VERSION-NOT-STORED 3/3 MANDATORY | Phase A | CLOSED — M13 `app_version`, endpoint, tests |
| F5 P2-IROH-INFRA-TIMEOUT 3/3 code | Phase A + D | CLOSED — timeout/retry/JoinHandle + E2E proof |
| P2-FEED-SUBSCRIBE-JOINHANDLE 2/3 | Phase B | CLOSED — proof/test shutdown |
| P2-BACKFILL-6PLUS-TEST 2/3 | Phase B | CLOSED — test backfill 6+ entries |
| P2-FEED-PUBLISH-ORPHAN 2/3 | Phase B | CLOSED — tail-safe rollback atomique |
| P2-SUBSCRIBE-STREAM-BREAK 2/3 | Phase B | CLOSED — proof/test reconnect |
| P2-PROCESS-FORMAT herite | Phase B | CLOSED — exemption retroactive README.md |

### Items reconduits S65

| Item | Compteur | Trigger |
|---|---|---|
| P2-A-1 rand blocker | exemption externe | rand 0.9 upstream |
| P2-AUDIT-2 iroh transitives | exemption externe | iroh 1.0 upgrade |
| P2-G-1 exe lock | monitoring | reproductible 3x |
| P2-PROVENANCE-404-BRIDGE | 2/3 | enrichissement UX |
| P2-BADGE-WORDING-PREMATURE | pre-existant S14 | verification live |
| P2-COMMIT-TITLE-FORMAT | 2/3 | process clarification |
| P2-REVIEW-ORDER | 2/3 | process clarification |
| P2-PYTHON-BLOCK-EXEMPTION | 2/3 | SKILL.md hygiene |
| **P2-FEED-INSERT-NO-AUTH-TIER** | **3/3 MANDATORY** | auth tier feed |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 2/3 | defensive hardening |
| P2-PLAYWRIGHT-SPECS-STALE | 2/3 | test maintenance |
| P2-VERIFY-LOCAL-KEY-ONLY | 2/3 | cross-node verification |
| P2-COVERAGE-DEPLOY-E2E | 2/3 | test coverage |
| P2-FEED-JOIN-HANDLE-LEAK | 1/3 | feed reconnect |
| P2-VERIFY-ENTRY-VERSION-GUARD | 1/3 | version policy |
| P2-ORPHAN-REPUBLISH-RECOVERY | 1/3 | feed resilience |

### Attention S65

**P2-FEED-INSERT-NO-AUTH-TIER** atteint 3/3 — MANDATORY S65. Le
handler `feed_insert` devra verifier le auth tier du caller avant
d'accepter une insertion.
