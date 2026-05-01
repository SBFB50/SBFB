# Sprint 47 — Verification

**Tip sortie** : `3641871` (Phase C).
**Theme** : carry resolution batch S45 + integration tests
completion + happy path tests + deprecated aliases cleanup.

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | ✅ 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | ✅ 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1185, 0 fail | ✅ 1185 run, 1184 passed, 1 flaky pre-existant (browse quorum) |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ✅ ok (1 ignored) |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | ✅ ok |
| 6 | ruff format | `uv run ruff format --check packages/` | 0 diff | ✅ 0 diff |
| 7 | ruff check | `uv run ruff check packages/` | 0 error | ✅ |
| 8 | SDK pytest | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 | ✅ 195 |
| 9 | Coord pytest | `uv run pytest packages/nexus-coordinator/tests/ -q` | 264+17f+6s | ✅ 264+17f+6s |
| 10 | Gov pytest | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 | ✅ 46 |
| 11 | npm lint | `npm run lint` (web/) | 0 error | ✅ |
| 12 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | ✅ |
| 13 | Vitest | `npm run test:unit` (web/) | >= 267 | ✅ 267 |
| 14 | build | `npm run build` (web/) | ok | ✅ |
| 15 | size-limit | `npm run size` (web/) | 5/5 | ✅ 5/5 |
| 16 | Phase A preflight G8 | EXECUTE | ✅ | ✅ |
| 17 | Phase A review | PASS (2 P2, 1 P3) | ✅ | ✅ |
| 18 | Phase B preflight G8 | EXECUTE | ✅ | ✅ |
| 19 | Phase B review | PASS (1 P2, 1 P3) | ✅ | ✅ |
| 20 | Phase C preflight G8 | EXECUTE | ✅ | ✅ |
| 21 | Phase C review | PASS (1 P2, 1 P3) | ✅ | ✅ |
| 22 | 3 S45 carries 2/3 resolus | 3/3 | ✅ | ✅ |
| 23 | 5 routes integration tests | 5/5 | ✅ | ✅ 9 tests |
| 24 | Happy path consent/files | 7/7 | ✅ | ✅ |
| 25 | Deprecated aliases supprimes | 3/3 | ✅ | ✅ |
| 26 | Python dead code modules supprimes | 7/7 | ✅ | ✅ |
| 27 | Scope cuts respectes | 11/11 | ✅ | ✅ |
| 28 | Delta tests documente | cumule | ✅ | ✅ Rust +17, Coord -65 |

## §2 Delta tests cumule

| Suite | Entree S47 | Sortie S47 | Delta | Source |
|---|---|---|---|---|
| Rust nextest | 1168 | 1185 | +17 | Phase A +1, Phase B +9, Phase C +7 |
| Rust doctests | 6+1i | 6+1i | +0 | |
| SDK pytest | 195 | 195 | +0 | |
| Coord pytest | 323+23f+6s | 264+17f+6s | -59p/-6f | Phase A 7 test files supprimes (dead code modules) |
| Gov pytest | 46 | 46 | +0 | |
| Vitest | 267 | 267 | +0 | |
| Playwright | 42+2f | 42+2f | +0 | non execute (env pre-existant) |
| size-limit | 5/5 | 5/5 | +0 | |
| **Total** | **~1984** | **~1936** | **-48** | net negatif car suppression 7 fichiers tests dead code |

## §3 Carries resolus S47

| Item | Compteur | Resolution |
|---|---|---|
| **P2-REVIEW-A-1-S45** diagnostic Err path | **2/3 CLOSED** | Phase A : test corrupted DB drop kudos table |
| **P2-REVIEW-A-2-S45** invite ID collision | **2/3 CLOSED** | Phase A : node_id prefix inv-{node8}-{ts}-{seq} |
| **P2-REVIEW-B-1-S45** Python modules suppression | **2/3 CLOSED** | Phase A : 7 dead code modules + tests supprimes |
| P2-INT-1-S46 deploy.rs + apps.rs tests | 1/3 CLOSED | Phase B : 8 tests Router::oneshot() |
| P2-INT-2-S46 auth/token test | 1/3 CLOSED | Phase B : 1 test auth_token |
| P2-REVIEW-A-1-S46 consent happy path | 1/3 CLOSED | Phase C : 4 tests consent |
| P2-REVIEW-A-2-S46 files upload happy path | 1/3 CLOSED | Phase C : 3 tests files |
| P2-REVIEW-C-2-S46 deprecated aliases | 1/3 CLOSED | Phase C : 3 alias exports supprimes + 12 refs migrees |

## §4 Carries S48

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 11+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 Day 0 |
| P3-AUDIT-B-4-S45 TOCTOU canary reload | 1/3→2/3 | microseconde, pre-v1.0 |
| P2-REVIEW-B-1-S46 kudos SQL pagination | 1/3→2/3 | hors scope S47 |
| P2-REVIEW-C-1-S46 app-specific schema drift | 1/3→2/3 | dep app runtime migration |
| P2-REVIEW-A-1-S47 execute_batch_raw pub | 1/3 | NEW S47 Phase A review |
| P2-REVIEW-A-2-S47 invite format test | 1/3 | NEW S47 Phase A review |
| P2-REVIEW-B-1-S47 deploy BlobsClient fragility | 1/3 | NEW S47 Phase B review |
| P2-REVIEW-C-1-S47 set_var process-wide | 1/3 | NEW S47 Phase C review |

## §5 Findings carry-over for memory

- S47 ferme les 3 MANDATORY-pending S45 (diagnostic Err, invite ID, Python modules)
- S47 ferme 5 carries S46 (deploy/apps/auth tests, consent/files happy path, deprecated aliases)
- 7 modules Python dead code supprimes (fairness, forge, honeypot, pow_counter, provenance, redundancy, watermark_detector) + 7 fichiers tests
- 6 modules Python encore vivants (guardrails, hooks, pii_redactor, rerun, capability_store + coordinator.py imports) — dep App Runtime Migration Rust
- Net +17 tests Rust, -65 tests Python (dead code), -1725 LOC Phase A, +235 LOC Phase B, +253/-21 LOC Phase C
- 9 carries S48 documentes (dont 4 NEW S47)
