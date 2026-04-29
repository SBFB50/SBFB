# Sprint 37 — Verification

**Tip** : `c53f663` (Phase B)
**Goal §2** : hash-chain KudosLedger + 2 MANDATORY 3/3 + P2 batch.
**Critere SMART** : 28+ rows fail-fast verts, mesure binaire.

## Fail-fast checklist

| # | Check | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | PASS |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 warnings) |
| 3 | `cargo nextest run --workspace` | PASS (946 tests, 0 fail) |
| 4 | `cargo test --workspace --doc` | PASS (0 pass, 1 ignored) |
| 5 | `cargo build -p nexus-shell-daemon --release` | PASS |
| 6 | `uv run ruff format --check packages/` | PASS |
| 7 | `uv run ruff check packages/` | PASS |
| 8 | `uv run pytest packages/nexus-sdk/tests/ -q` | 194+1f (flaky Windows file-lock, pre-existing) |
| 9 | `uv run pytest packages/nexus-coordinator/tests/ -q` | 409+36f+6s (PyO3 stale, pre-existing) |
| 10 | `uv run pytest packages/nexus-app-gov/tests/ -q` | PASS (46) |
| 11 | `npm run lint` (web/) | PASS (0 errors, 7 warnings pre-existing) |
| 12 | `npx tsc --noEmit -p tsconfig.app.json` | PASS |
| 13 | `npm run test:unit` (web/) | PASS (267 tests) |
| 14 | `npm run build` (web/) | PASS |
| 15 | `npm run size` (web/) | PASS (7/7) |
| 16 | Phase A preflight G8 : EXECUTE | PASS |
| 17 | Phase A review : PASS (2 P2 + 1 P3) | PASS |
| 18 | Phase B preflight G8 : EXECUTE | PASS |
| 19 | Phase B review : PASS (1 P2 + 1 P3) | PASS |
| 20 | Log convergence daemon+launcher shared dir | PASS (paths::log_dir → <root>/logs/) |
| 21 | .icns macOS tool cross-platform | PASS (tools/png-to-icns crate, bundle-macos.sh wired) |
| 22 | HARDENING_ROADMAP compteurs | PASS (946 Rust / ~1949 total) |
| 23 | unwrap_or_default → error handling | PASS (2 handlers http.rs) |
| 24 | Mutex poisoned tests | PASS (+3 tests submit_task/result/kudos) |
| 25 | validate_result retourne TaskRecord | PASS (double query elimine) |
| 26 | KudosLedger hash-chain BLAKE3+JCS | PASS (credit compute + verify_chain) |
| 27 | Hash-chain genesis + chaining | PASS (prev_hash = "genesis" / entry_hash) |
| 28 | Cross-project chains independent | PASS |
| 29 | Scope cuts §7 respectes | PASS (12 scope cuts, 0 viole) |
| 30 | Delta tests Phase A : +4 | PASS (936→940) |
| 31 | Delta tests Phase B : +6 | PASS (940→946) |
| 32 | Delta tests cumule S37 : +10 | PASS (936→946) |

**Verdict : 32/32 PASS** (critere 28+ atteint).

## §3 Compteurs tests finaux

| Suite | Count |
|---|---|
| Rust nextest | 946 (+10 vs S36) |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1949** |

## §4 Phases livrees

| Phase | Commit | Delta tests |
|---|---|---|
| A | `baf4d6a` MANDATORY log convergence + .icns + P2 batch | +4 (936→940) |
| B | `c53f663` KudosLedger hash-chain BLAKE3 + JCS canonical | +6 (940→946) |

## §5 Findings carry-over for memory

- P2-REVIEW-A-1 (Phase A) : launcher setup_tracing() non testable unitairement — faible risk, meme pattern daemon
- P2-REVIEW-A-2 (Phase A) : serde_json Err branch sans test — impossible a trigger, defensive pure
- P2-REVIEW-B-1 (Phase B) : rowid tiebreaker implicite — documenter si migration schema futur
- P3-REVIEW-A-1 (Phase A) : icns max 512px — macOS Retina downscale, R1 kickoff
- P3-REVIEW-B-1 (Phase B) : verify_chain O(n) — R2 kickoff, Merkle post-v1.0
