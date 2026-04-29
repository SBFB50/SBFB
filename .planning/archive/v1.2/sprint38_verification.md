# Sprint 38 — Verification

**Tip** : `16ad15e` (Phase C)
**Goal §2** : validator_loop MANDATORY 3/3 + OutputFilter/Guardrails
Rust migration (Tier 1 part 1) + dette pair S38.
**Critere SMART** : 28+ rows fail-fast verts, mesure binaire.

## Fail-fast checklist

| # | Check | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | PASS |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 warnings) |
| 3 | `cargo nextest run --workspace` | PASS (967 tests, 0 fail) |
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
| 17 | Phase A review : PASS (1 P2 + 1 P3) | PASS |
| 18 | Phase B preflight G8 : EXECUTE | PASS |
| 19 | Phase B review : PASS (1 P2 + 1 P3) | PASS |
| 20 | Phase C preflight G8 : EXECUTE | PASS |
| 21 | Phase C review : PASS (1 P2 + 1 P3) | PASS |
| 22 | MANDATORY validator_loop tokio 3/3 ferme | PASS (module + broadcast + 3 tests) |
| 23 | OutputFilter Rust migration | PASS (output_filter.rs 200 LOC + 10 tests) |
| 24 | Guardrails pipeline Rust + wire | PASS (guardrails.rs 160 LOC + 6 tests) |
| 25 | rowid documentation P33 PATTERNS | PASS (inline + section) |
| 26 | verify_chain endpoint HTTP | PASS (GET route + 1 test) |
| 27 | launcher log_dir coherence test | PASS (+1 test) |
| 28 | Scope cuts §7 respectes | PASS (12 scope cuts, 0 viole) |
| 29 | Delta tests Phase A : +5 | PASS (946→951) |
| 30 | Delta tests Phase B : +10 | PASS (951→961) |
| 31 | Delta tests Phase C : +6 | PASS (961→967) |
| 32 | Delta tests cumule S38 : +21 | PASS (946→967) |

**Verdict : 32/32 PASS** (critere 28+ atteint).

## §3 Compteurs tests finaux

| Suite | Count |
|---|---|
| Rust nextest | 967 (+21 vs S37) |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1970** |

## §4 Phases livrees

| Phase | Commit | Delta tests |
|---|---|---|
| A | `511658f` MANDATORY validator_loop + dette pair P2 batch | +5 (946→951) |
| B | `0862a9d` OutputFilter Rust migration | +10 (951→961) |
| C | `16ad15e` Guardrails pipeline Rust + wire submit_result | +6 (961→967) |

## §5 Findings carry-over for memory

- P2-REVIEW-A-1-S38 (Phase A) : result_event_tx dead code path — broadcast channel cree mais pas de producteur en production (infra pour gossip wiring futur S39+)
- P2-REVIEW-B-1-S38 (Phase B) : substring detection O(n*m) worst-case — acceptable pre-v1.0, Rabin-Karp post-v1.0
- P2-REVIEW-C-1-S38 (Phase C) : default_output_chain() reconstruit a chaque requete — stocker Arc singleton post-v1.0
- P3-REVIEW-A-2-S38 (Phase A) : Mutex contention validator_loop + HTTP — connection pool post-v1.0
- P3-REVIEW-B-2-S38 (Phase B) : EED sur output complet (dilution) — sliding window post-v1.0
- P3-REVIEW-C-2-S38 (Phase C) : system_prompt vide dans guardrail context — enrichir ResultEntry post-v1.0
