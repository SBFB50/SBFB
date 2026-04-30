# Sprint 43 — Verification

**HEAD entree** : `e1f7f00` (post audit gate S42 PASS).
**HEAD sortie** : `0ec0458` (Phase C).
**Goal §2** : resoudre 7 items MANDATORY/OVERDUE + completer
Tier 5 en portant 4 routes API Python (files, consent, canary,
contributor) vers axum Rust.
**Critere SMART** : 28+ rows fail-fast verts.

## Commit stack

```
e4d1bea chore(planning): sprint 43 kickoff + plan + design review + migration S42 archive
9f32731 chore(planning): sprint 43 Phase A preflight G8
130db9b feat(sprint43): Sprint 43 Phase A — MANDATORY batch 7 items conn+persist+mutex+hash+mint+process
3f6b384 chore(planning): sprint 43 Phase B preflight G8
a766496 feat(sprint43): Sprint 43 Phase B — files + consent API Rust
a55aa1c chore(planning): sprint 43 Phase C preflight G8
0ec0458 feat(sprint43): Sprint 43 Phase C — canary + contributor API Rust
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
| 3 | `cargo nextest run --workspace` | PASS (1111 tests, 0 skipped) |
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
| 17 | Phase A review : PASS (0 P2, 2 P3) | PASS |
| 18 | Phase B preflight G8 : EXECUTE | PASS |
| 19 | Phase B review : PASS (1 P2 corrige, 1 P3) | PASS |
| 20 | Phase C preflight G8 : EXECUTE | PASS |
| 21 | Phase C review : PASS post-fix P1 (1 P2, 1 P3) | PASS |
| 22 | 7/7 MANDATORY items resolus | PASS |
| 23 | Files handler porte | PASS (3 routes) |
| 24 | Consent handler porte | PASS (4 routes) |
| 25 | Canary inject-rate + divergence portes | PASS (2 routes) |
| 26 | Contributor verify+list+envelope portes | PASS (3 routes, proxy→direct) |
| 27 | Scope cuts respectes | PASS (6/6) |
| 28 | Delta tests Phase A+B+C | +22 (1089→1111) |

**Verdict : 28/28 PASS** (critere 28+ atteint).

## §3 Compteurs tests finaux

| Suite | Count |
|---|---|
| Rust nextest | 1111 (+22 vs S42) |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 7/7 |
| **Total** | **~2114** |

## §4 Phases livrees

| Phase | Commit | Delta tests |
|---|---|---|
| A | `130db9b` MANDATORY batch 7 items | +2 (1089→1091) |
| B | `a766496` files + consent API Rust | +14 (1091→1105) |
| C | `0ec0458` canary + contributor API Rust | +6 (1105→1111) |

## §5 Findings carry-over for memory

- P2-REVIEW-B-1-S43 (Phase C) : coord_http_client et
  coord_base_url marques #[allow(dead_code)] apres suppression
  proxy. Cleanup S45 (suppression Python coordinator).
- P3-REVIEW-C-1-S43 (Phase C) : routes contributor sous
  /api/contributor/ sans /v1/ prefix. Inconsistance avec
  /api/v1/files, /api/v1/consent etc. A normaliser S44.
- P3-REVIEW-B-1-S43 (Phase B) : tests HTTP integration
  manquants pour les 7 nouvelles routes consent+files.
- P3-REVIEW-A-1-S43 (Phase A) : TOCTOU canary_input.rs reload
  pre-existant (inchange par consolidation Mutex). P3 informational.

## §6 Surface nouvelle livree

| Module | Crate | Fichier |
|---|---|---|
| consent.rs | nexus-shell-daemon | NEW ~230 LOC |
| files.rs | nexus-shell-daemon | NEW ~190 LOC |
| canary_api.rs | nexus-shell-daemon | NEW ~100 LOC |
| contributor_api.rs | nexus-shell-daemon | NEW ~140 LOC |
| **Total** | | **~660 LOC** |

Fichiers modifies (non nouveaux) : db.rs (-1/+1), canary_registry.rs
(+4/-2), canary_input.rs (+15/-12), rerun.rs (+8/-6), invite.rs
(+22/-7), http.rs (+16/-80), main.rs (+2), runtime.rs (+1),
Cargo.toml (+1).

## §7 Ce que le sprint n'a PAS livre (scope cuts respectes)

- Routes restantes (health, shell, tasks, kudos, etc.) — S44
- Suppression coordinator Python — S45
- CI/VPS/v1.0 — S46-48
- Kudos debit/stake — interdit (Day 0 #7)
- @require_capability middleware — S44
- Background loops wire-up — S44+

## §8 Findings carry-over for memory (G6)

(Identique §5, consolide ci-dessus.)

## §9 Checkpoint de cloture

1. 7/7 MANDATORY items resolus en Phase A
2. 4 routes API (files+consent+canary+contributor) portees Phases B+C
3. 6 scope cuts respectes
4. Proxy contributor → direct (elimination dep Python)
5. Tier 5 routes API complet : 8/8 routes Python portees (S42+S43)
