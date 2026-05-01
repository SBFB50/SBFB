# Sprint 46 — Verification

**Tip sortie** : `812f3ba` (Phase C).
**Theme** : integration tests MANDATORY + dette pair S44 + frontend direct-daemon.

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | ✅ 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | ✅ 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1168, 0 fail | ✅ 1168 passed, 0 fail |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ✅ ok (1 ignored) |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ✅ ok |
| 6 | ruff format | `uv run ruff format --check packages/` | 0 diff | ⚠️ 1 pre-existing (test_redundancy.py S45) |
| 7 | ruff check | `uv run ruff check packages/` | 0 error | ✅ |
| 8 | SDK pytest | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 | ✅ 195 |
| 9 | Coord pytest | `uv run pytest packages/nexus-coordinator/tests/ -q` | 323+23f+6s | ✅ 323+23f+6s |
| 10 | Gov pytest | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 | ✅ 46 |
| 11 | npm lint | `npm run lint` (web/) | 0 error | ✅ |
| 12 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | ✅ |
| 13 | Vitest | `npm run test:unit` (web/) | >= 267 | ✅ 267 |
| 14 | build | `npm run build` (web/) | ok | ✅ |
| 15 | size-limit | `npm run size` (web/) | 5/5 | ✅ 5/5 |
| 16 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | ✅ |
| 17 | Phase A preflight G8 | EXECUTE | ✅ | ✅ |
| 18 | Phase A review | PASS (2 P2, 1 P3) | ✅ | ✅ |
| 19 | Phase B preflight G8 | EXECUTE | ✅ | ✅ |
| 20 | Phase B review | PASS (2 P2, 1 P3) | ✅ | ✅ |
| 21 | Phase C preflight G8 | EXECUTE | ✅ | ✅ |
| 22 | Phase C review | PASS (2 P2, 1 P3) | ✅ | ✅ |
| 23 | 12 routes MANDATORY integration tests | 12/12 | ✅ | ✅ 19 tests couvrent 12 routes |
| 24 | 14 routes recentes integration tests | 14/14 | ✅ | ✅ 17 tests couvrent 14 routes |
| 25 | 5 items dette S44 resolus | 5/5 | ✅ | ✅ |
| 26 | Frontend direct-daemon compile | ok | ✅ | ✅ |
| 27 | Scope cuts respectes | 13/13 | ✅ | ✅ |
| 28 | Delta tests documente | cumule | ✅ | ✅ Rust +36, Vitest -1 |

## §2 Delta tests cumule

| Suite | Entree S46 | Sortie S46 | Delta | Source |
|---|---|---|---|---|
| Rust nextest | 1132 | 1168 | +36 | Phase A +19, Phase B +17 |
| Rust doctests | 6+1i | 6+1i | +0 | |
| SDK pytest | 195 | 195 | +0 | |
| Coord pytest | 323+23f+6s | 323+23f+6s | +0 | |
| Gov pytest | 46 | 46 | +0 | |
| Vitest | 268 | 267 | -1 | Phase C proxy envelope test retire |
| Playwright | 42+2f | 42+2f | +0 | non execute (env pre-existant) |
| size-limit | 7/7 | 5/5 | -2 budgets retires | |
| **Total** | **~1949** | **~1984** | **+35** | |

## §3 Carries resolus S46

| Item | Compteur | Resolution |
|---|---|---|
| **P2-AUDIT-A-1-S43** integration test gap 12 routes | **3/3 MANDATORY** | Phase A : 19 tests Router::oneshot() |
| P2-REVIEW-A-1-S44 as_str/serde coupling | 2/3 | Phase B : verifie clean (grep = 0) |
| P2-REVIEW-B-1-S44 kudos entries pagination | 2/3 | Phase B : limit/offset ajoutes |
| P3-REVIEW-B-2-S44 shell discover self-only | 2/3 | Phase B : test integration verifie |
| P3-AUDIT-A-1-S44 test pagination handler-level | 2/3 | Phase B : tests kudos+tasks limit |
| P3-AUDIT-B-1-S44 diagnostic silent fallback | 2/3 | Phase B : unwrap_or_default→500 |

## §4 Carries S47

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 10+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 Day 0 |
| P2-REVIEW-A-1-S45 diagnostic Err path non teste | 1/3→2/3 | NEW S45 |
| P2-REVIEW-A-2-S45 invite ID collision multi-daemon | 1/3→2/3 | NEW S45 |
| P2-REVIEW-B-1-S45 modules Python suppression differee | 1/3→2/3 | NEW S45 |
| P3-AUDIT-B-4-S45 TOCTOU canary reload fenetre microseconde | 1/3 | NEW S45 |
| P2-INT-1-S46 integration tests deploy.rs + apps.rs | 1/3 | NEW S46 scope cut |
| P2-INT-2-S46 integration test auth/token | 1/3 | NEW S46 scope cut |
| P2-REVIEW-A-1-S46 consent happy path non teste | 1/3 | NEW S46 Phase A review |
| P2-REVIEW-A-2-S46 files upload happy path non teste | 1/3 | NEW S46 Phase A review |
| P2-REVIEW-B-1-S46 kudos SQL pagination | 1/3 | NEW S46 Phase B review |
| P2-REVIEW-C-1-S46 app-specific schema drift | 1/3 | NEW S46 Phase C review |
| P2-REVIEW-C-2-S46 deprecated error class aliases | 1/3 | NEW S46 Phase C review |

## §5 Findings carry-over for memory

- S46 ferme le MANDATORY P2-AUDIT-A-1-S43 (integration test gap 12 routes, 3/3 depuis S43)
- S46 ferme les 5 items dette S44 (as_str, kudos pagination, shell discover, test pagination, diagnostic fallback)
- Frontend migre vers daemon direct (coordinator.ts→/api/v1/*, daemon.ts proxy envelope supprime)
- Net +36 tests Rust, -1 Vitest, -260 LOC frontend
- 13 carries S47 documentes (dont 5 NEW S46)
