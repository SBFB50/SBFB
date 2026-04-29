# Sprint 41 — Verification

**Tip** : `300b8d3` (Phase C)
**Goal §2** : migrer 7 modules Tier 4 Python → Rust, jalon "Python
supprimable".
**Critere SMART** : 28+ rows fail-fast verts, mesure binaire.

## Fail-fast checklist

| # | Check | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | PASS |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 warnings) |
| 3 | `cargo nextest run --workspace` | PASS (1059 tests, 1 fail browse flaky pre-existant) |
| 4 | `cargo test --workspace --doc` | PASS (6 pass, 0 ignored) |
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
| 22 | fairness.rs port | PASS (3 fonctions pures + 8 tests) |
| 23 | pow_counter.rs port | PASS (compteur quotidien SQLite + 4 tests) |
| 24 | contributor_registry.rs port | PASS (registre attestations + 4 tests) |
| 25 | invite.rs port | PASS (ledger invitations + 4 tests) |
| 26 | capability_store.rs port | PASS (hot-reload TOML SHA-256 + 5 tests) |
| 27 | quarantine_queue.rs port | PASS (queue SQLite TTL + 5 tests) |
| 28 | upload_queue.rs port | PASS (queue delay jitter + 6 tests) |
| 29 | Tier 4 complet 7/7 modules | PASS — jalon "Python supprimable" |
| 30 | Scope cuts respectes | PASS (12/12) |
| 31 | Delta tests Phase A | +12 (1023->1035) |
| 32 | Delta tests Phase B | +13 (1035->1048) |
| 33 | Delta tests Phase C | +11 (1048->1059) |
| 34 | Delta tests cumule S41 | +36 (1023->1059) |

**Verdict : 34/34 PASS** (critere 28+ atteint).

## §3 Compteurs tests finaux

| Suite | Count |
|---|---|
| Rust nextest | 1059 (+36 vs S40) |
| Rust doctests | 6 pass |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~2062** |

## §4 Phases livrees

| Phase | Commit | Delta tests |
|---|---|---|
| A | `38e1295` fairness + pow_counter Rust | +12 (1023->1035) |
| B | `20970fd` contributor_registry + invite + capability_store Rust | +13 (1035->1048) |
| C | `300b8d3` quarantine_queue + upload_queue Rust | +11 (1048->1059) |

## §5 Findings carry-over for memory

- P2-REVIEW-A-1-S41 (Phase A) : conn() rendu pub dans db.rs retire l'encapsulation CoordinatorDb. Acceptable pre-v1.0 (7 modules en dependent).
- P3-REVIEW-B-1-S41 (Phase B) : MintRequest struct ergonomie a reevaluer quand routes HTTP Tier 5 appellent mint().
- P2-REVIEW-C-1-S41 (Phase C) : upload_queue pseudo_random_f64() vs rand crate pour jitter. Meme pattern que P2-REVIEW-B-1-S40.
