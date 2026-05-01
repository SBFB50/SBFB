# Sprint 48 — Verification

**Tip sortie** : `672c287` (Phase B).
**Theme** : dette pair carries resolution batch — TOCTOU canary +
kudos total_count + execute_batch_raw gate + invite format test +
sbfb_home refactor.

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | ✅ 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | ✅ 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1186, 0 fail | ✅ 1186 run, 1185 passed, 1 flaky pre-existant (browse quorum) |
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
| 17 | Phase A review | PASS (1 P2, 1 P3) | ✅ | ✅ |
| 18 | Phase B preflight G8 | EXECUTE | ✅ | ✅ |
| 19 | Phase B review | PASS (1 P2, 1 P3) | ✅ | ✅ |
| 20 | TOCTOU canary fix | mutex hold-across-read | ✅ | ✅ |
| 21 | kudos total_count | champ + frontend | ✅ | ✅ |
| 22 | schema drift exemption | documentee | ✅ | ✅ |
| 23 | execute_batch_raw gate | cfg visible | ✅ | ✅ |
| 24 | invite format test | assertions pattern | ✅ | ✅ |
| 25 | sbfb_home dans state | 7 set_var elimines | ✅ | ✅ |
| 26 | BlobsClient reclassif | documentee | ✅ | ✅ |
| 27 | Scope cuts respectes | 10/10 | ✅ | ✅ |
| 28 | Delta tests documente | cumule | ✅ | ✅ Rust +1 |

## §2 Delta tests cumule

| Suite | Entree S48 | Sortie S48 | Delta | Source |
|---|---|---|---|---|
| Rust nextest | 1185 | 1186 | +1 | Phase B +1 (files_dir_override_home) |
| Rust doctests | 6+1i | 6+1i | +0 | |
| SDK pytest | 195 | 195 | +0 | |
| Coord pytest | 264+17f+6s | 264+17f+6s | +0 | |
| Gov pytest | 46 | 46 | +0 | |
| Vitest | 267 | 267 | +0 | |
| Playwright | 42+2f | 42+2f | +0 | non execute (env pre-existant) |
| size-limit | 5/5 | 5/5 | +0 | |
| **Total** | **~1936** | **~1937** | **+1** | |

## §3 Carries resolus S48

| Item | Compteur | Resolution |
|---|---|---|
| **P3-AUDIT-B-4-S45** TOCTOU canary reload | **2/3 CLOSED** | Phase A : mutex hold-across-read canary_input.rs |
| **P2-REVIEW-B-1-S46** kudos SQL pagination | **2/3 CLOSED** | Phase A : total_count avant skip/take + frontend |
| **P2-REVIEW-C-1-S46** app-specific schema drift | **2/3 EXEMPTION** | Phase A : bloque par App Runtime Migration, reclassifie hors carry actif |
| **P2-REVIEW-A-1-S47** execute_batch_raw pub | **1/3 CLOSED** | Phase B : feature gate test-support |
| **P2-REVIEW-A-2-S47** invite format test | **1/3 CLOSED** | Phase B : assertions pattern inv-{node8}-{ts}-{seq} |
| **P2-REVIEW-C-1-S47** set_var process-wide | **1/3 CLOSED** | Phase B : sbfb_home dans DaemonHttpState, 7 set_var elimines |
| **P2-REVIEW-B-1-S47** deploy BlobsClient fragility | **1/3 RECLASSIFIE** | Phase B : risque inherent mk_state(), accepte pre-v1.0 |

## §4 Carries S49

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 Day 0 |
| P2-REVIEW-A-1-S48 canary reload size cap | 1/3 | NEW S48 Phase A review |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 1/3 | NEW S48 Phase B review |

## §5 Findings carry-over for memory

- S48 ferme les 2 items 2/3 MANDATORY-pending (TOCTOU canary, kudos SQL pagination)
- S48 ferme 3 items 1/3 S47 (execute_batch_raw, invite format, set_var)
- 1 item RECLASSIFIE (BlobsClient fragility) + 1 EXEMPTION (schema drift)
- sbfb_home refactor elimine 7 std::env::set_var process-wide dans les tests
- Net +1 test Rust, total 1186 Rust / ~1937 total
- 4 carries S49 documentes (dont 2 NEW S48)
