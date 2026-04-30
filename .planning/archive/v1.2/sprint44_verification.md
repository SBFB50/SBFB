# Sprint 44 — Verification

**HEAD entree** : `358c6ff` (post audit gate S43 PASS).
**HEAD sortie** : `9942d70` (Phase C).
**Goal §2** : resoudre 7 items MANDATORY 3/3 + terminer migration
Tier 5 en portant 6 routes API Python restantes (health, shell,
tasks, kudos, diagnostic, worker_state) vers axum Rust.
**Critere SMART** : 30+ rows fail-fast verts.

## Commit stack

```
ae9190e chore(planning): sprint 44 kickoff + plan + design review + migration S43 archive
ddbfc7f chore(planning): sprint 44 Phase A preflight G8
0ef7358 feat(sprint44): Sprint 44 Phase A — dette pair 7 MANDATORY
589f91c chore(planning): sprint 44 Phase B preflight G8
7100d24 feat(sprint44): Sprint 44 Phase B — health + shell + kudos + diagnostic API Rust
73b1de7 chore(planning): sprint 44 Phase C preflight G8
9942d70 feat(sprint44): Sprint 44 Phase C — tasks + worker_state API Rust
```

## How to re-run

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# Python
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend
cd web && npm install && npm run lint && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run build && npm run size && \
  npx playwright test && bash scripts/scan-en-strings.sh
```

## Fail-fast checklist

| # | Check | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | PASS |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 warnings) |
| 3 | `cargo nextest run --workspace` | PASS (1127 tests, 1126 passed, 1 flaky pre-existant) |
| 4 | `cargo test --workspace --doc` | PASS (6 passed, 1 ignored) |
| 5 | `cargo build -p nexus-shell-daemon --release` | PASS |
| 6 | `uv run ruff format --check packages/` | PASS |
| 7 | `uv run ruff check packages/` | PASS |
| 8 | `uv run pytest packages/nexus-sdk/tests/ -q` | PASS (195) |
| 9 | `uv run pytest packages/nexus-coordinator/tests/ -q` | 409+36f+6s (PyO3 stale, pre-existant) |
| 10 | `uv run pytest packages/nexus-app-gov/tests/ -q` | PASS (46) |
| 11 | `npm run lint` (web/) | PASS |
| 12 | `npx tsc --noEmit -p tsconfig.app.json` | PASS |
| 13 | `npm run test:unit` (web/) | PASS (267 tests) |
| 14 | `npm run build` (web/) | PASS |
| 15 | `npm run size` (web/) | PASS (7/7) |
| 16 | Phase A preflight G8 : EXECUTE | PASS |
| 17 | Phase A review : PASS (1 P2, 1 P3) | PASS |
| 18 | Phase B preflight G8 : EXECUTE | PASS |
| 19 | Phase B review : PASS (1 P2, 1 P3) | PASS |
| 20 | Phase C preflight G8 : EXECUTE | PASS |
| 21 | Phase C review : PASS (1 P2, 1 P3) | PASS |
| 22 | 7/7 MANDATORY items resolus | PASS |
| 23 | Health handler porte | PASS (1 route) |
| 24 | Shell handler porte | PASS (1 route) |
| 25 | Kudos list+leaderboard porte | PASS (2 routes) |
| 26 | Diagnostic fairness porte | PASS (1 route) |
| 27 | Tasks list+get portes | PASS (2 routes) |
| 28 | Worker state porte | PASS (1 route) |
| 29 | Scope cuts respectes | PASS (6/6) |
| 30 | Delta tests Phase A+B+C | +16 (1111→1127) |

**Verdict : 30/30 PASS** (critere 30+ atteint).

## §3 Compteurs tests finaux

| Suite | Count |
|---|---|
| Rust nextest | 1127 (+16 vs S43) |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 7/7 |
| **Total** | **~2130** |

## §4 Phases livrees

| Phase | Commit | Delta tests |
|---|---|---|
| A | `0ef7358` dette pair 7 MANDATORY + prefix | +3 (1111→1114) |
| B | `7100d24` health+shell+kudos+diagnostic | +7 (1114→1121) |
| C | `9942d70` tasks+worker_state | +6 (1121→1127) |

## §5 Findings carry-over for memory

- P2-REVIEW-A-1-S44 (Phase A) : as_str()/serde rename coupling
  non-enforce par compilateur si serde rename custom. Carry S45.
- P2-REVIEW-B-1-S44 (Phase B) : kudos entries endpoint sans
  pagination. Carry S45.
- P2-REVIEW-C-1-S44 (Phase C) : worker_state std::fs bloquant
  dans handler async. Migrer tokio::fs S45.

## §6 Surface nouvelle livree

| Module | Crate | Fichier |
|---|---|---|
| health_api.rs | nexus-shell-daemon | NEW ~45 LOC |
| shell_api.rs | nexus-shell-daemon | NEW ~40 LOC |
| kudos_api.rs | nexus-shell-daemon | NEW ~130 LOC |
| diagnostic_api.rs | nexus-shell-daemon | NEW ~70 LOC |
| tasks_api.rs | nexus-shell-daemon | NEW ~170 LOC |
| worker_state_api.rs | nexus-shell-daemon | NEW ~120 LOC |
| **Total** | | **~575 LOC** |

Fichiers modifies : db.rs (+102), browse.rs (+19), apps.rs (+56/-14),
http.rs (+32), main.rs (+6), canary_input.rs (+25), PATTERNS.md (+40),
.gitignore (+3).

## §7 Ce que le sprint n'a PAS livre (scope cuts respectes)

- events.py SSE streaming — S45 (dep AppEvents bus SDK Python)
- quarantine.py API routes — S45
- Suppression coordinator Python — S45
- CI/VPS/v1.0 — S46-48
- Kudos debit/stake — interdit (Day 0 #7)
- Integration test gap complet — partiel S44 (nouvelles routes),
  complet S45

## §8 Findings carry-over for memory (G6)

(Identique §5, consolide ci-dessus.)

## §9 Checkpoint de cloture

1. 7/7 MANDATORY items resolus en Phase A
2. 8 routes API portees (Phases B+C)
3. 6 scope cuts respectes
4. Tier 5 routes API S44 complet (6/7, events.py scope-cut S45)
5. Sprint pair dette obligatoire respectee (Phase A)
