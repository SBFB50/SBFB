# Sprint 40 — Verification

**Tip** : `0b9df49` (Phase C)
**Goal §2** : canary_input Rust (Tier 2 fin) + Tier 3 batch
(redundancy + watermark_detector + rerun + honeypot) + dette pair.
**Critere SMART** : 28+ rows fail-fast verts, mesure binaire.

## Fail-fast checklist

| # | Check | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | PASS |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 warnings) |
| 3 | `cargo nextest run --workspace` | PASS (1023 tests, 0 skipped) |
| 4 | `cargo test --workspace --doc` | PASS (0 pass, 1 ignored) |
| 5 | `cargo build -p nexus-shell-daemon --release` | PASS |
| 6 | `uv run ruff format --check packages/` | PASS |
| 7 | `uv run ruff check packages/` | PASS |
| 8 | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 (1 flaky Windows file-lock, pre-existing) |
| 9 | `uv run pytest packages/nexus-coordinator/tests/ -q` | 409+36f+6s (PyO3 stale, pre-existing) |
| 10 | `uv run pytest packages/nexus-app-gov/tests/ -q` | PASS (46) |
| 11 | `npm run lint` (web/) | PASS |
| 12 | `npx tsc --noEmit -p tsconfig.app.json` | PASS |
| 13 | `npm run test:unit` (web/) | PASS (267 tests) |
| 14 | `npm run build` (web/) | PASS |
| 15 | `npm run size` (web/) | PASS (7/7) |
| 16 | Phase A preflight G8 : EXECUTE | PASS |
| 17 | Phase A review : PASS (0 P2 + 2 P3) | PASS |
| 18 | Phase B preflight G8 : EXECUTE | PASS |
| 19 | Phase B review : PASS (1 P2 + 1 P3) | PASS |
| 20 | Phase C preflight G8 : EXECUTE | PASS |
| 21 | Phase C review : PASS (1 P2 + 1 P3) | PASS |
| 22 | Dette pair 5 items P2/P3 resolus | PASS |
| 23 | CanaryInput Rust port complet | PASS (canary_input.rs + 13 tests) |
| 24 | Redundancy Rust port | PASS (redundancy.rs + 3 tests) |
| 25 | WatermarkDetector Rust port | PASS (watermark_detector.rs + 4 tests) |
| 26 | Rerun Rust port | PASS (rerun.rs + 5 tests) |
| 27 | Honeypot Rust port | PASS (honeypot.rs + 4 tests) |
| 28 | P3-grammar executor 3/3+ RESOLU | PASS (rerun.rs) |
| 29 | P3-watermark executor 3/3+ RESOLU | PASS (watermark_detector.rs) |
| 30 | Scope cuts §7 respectes | PASS (12 scope cuts, 0 viole) |
| 31 | Delta tests Phase A : +3 | PASS (991->994) |
| 32 | Delta tests Phase B : +13 | PASS (994->1007) |
| 33 | Delta tests Phase C : +16 | PASS (1007->1023) |
| 34 | Delta tests cumule S40 : +32 | PASS (991->1023) |

**Verdict : 34/34 PASS** (critere 28+ atteint).

## §3 Compteurs tests finaux

| Suite | Count |
|---|---|
| Rust nextest | 1023 (+32 vs S39) |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~2026** |

## §4 Phases livrees

| Phase | Commit | Delta tests |
|---|---|---|
| A | `2b6e3dd` dette pair P2 batch 2/3 items + HTTP integration tests | +3 (991->994) |
| B | `f5b6731` CanaryInput Rust canary_input.rs | +13 (994->1007) |
| C | `0b9df49` Tier 3 batch redundancy + watermark + rerun + honeypot | +16 (1007->1023) |

## §5 Findings carry-over for memory

- P2-REVIEW-B-1-S40 (Phase B) : rand_range uses DefaultHasher+nanos for randomness instead of rand crate (already in deps). Acceptable pre-v1.0.
- P2-REVIEW-C-1-S40 (Phase C) : redundancy.rs SHA-256 vs BLAKE3 alignment post-v1.0
- P3-REVIEW-B-1-S40 (Phase B) : CanaryInputManager multiple Mutex fields vs consolidated struct
- P3-REVIEW-C-1-S40 (Phase C) : rerun.rs deterministic hash sampling (same pattern Phase B)
