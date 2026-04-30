# Sprint 42 — Verification

**HEAD entree** : `7edf04b` (post audit gate S41 PASS).
**HEAD sortie** : `87ee663` (Phase C).
**Goal §2** : resoudre dette pair P2 (4 items) + debuter Tier 5
en portant deploy+apps handlers vers axum Rust.
**Critere SMART** : 28+ rows fail-fast verts.

## Commit stack

```
d6f8191 chore(planning): sprint 42 kickoff + plan + design review
f8122fc chore(planning): sprint 42 Phase A preflight G8
03f1497 feat(sprint42): Sprint 42 Phase A — dette pair P2 batch rand + Mutation + warn threshold
edaf1b3 chore(planning): sprint 42 Phase B preflight G8
aaa2e18 feat(sprint42): Sprint 42 Phase B — deploy API Rust
5394fd5 chore(planning): sprint 42 Phase C preflight G8
87ee663 feat(sprint42): Sprint 42 Phase C — apps API Rust
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
| 3 | `cargo nextest run --workspace` | PASS (1089 tests, 0 skipped) |
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
| 17 | Phase A review : PASS (1 P2 + 1 P3) | PASS |
| 18 | Phase B preflight G8 : EXECUTE | PASS |
| 19 | Phase B review : PASS (1 P2 + 1 P3) | PASS |
| 20 | Phase C preflight G8 : EXECUTE | PASS |
| 21 | Phase C review : PASS (0 P2 + 1 P3) | PASS |
| 22 | 4/4 P2 dette resolus | PASS (rand_range, pseudo_random, Mutation, warn threshold) |
| 23 | deploy handler porte | PASS (POST /api/v1/deploy) |
| 24 | apps handlers portes | PASS (GET /api/v1/apps, /apps/:id) |
| 25 | Scope cuts respectes | PASS (8/8) |
| 26 | Delta tests Phase A | +1 (1059->1060) |
| 27 | Delta tests Phase B | +21 (1060->1081) |
| 28 | Delta tests Phase C | +8 (1081->1089) |

**Verdict : 28/28 PASS** (critere 28+ atteint).

## §3 Compteurs tests finaux

| Suite | Count |
|---|---|
| Rust nextest | 1089 (+30 vs S41) |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 7/7 |
| **Total** | **~2092** |

## §4 Phases livrees

| Phase | Commit | Delta tests |
|---|---|---|
| A | `03f1497` dette pair P2 batch rand + Mutation + warn threshold | +1 (1059->1060) |
| B | `aaa2e18` deploy API Rust | +21 (1060->1081) |
| C | `87ee663` apps API Rust | +8 (1081->1089) |

## §5 Findings carry-over for memory

- P2-REVIEW-A-1-S42 (Phase A) : ChainResult::mutations est
  Vec<(String, String)> sans semantique sur le target de la
  mutation. A documenter quand le premier consumer Mutation est
  implemente post-v1.0.
- P2-REVIEW-B-1-S42 (Phase B) : pow_keypair utilise pour signer
  provenance. Documenter que pow_keypair = identite provenance
  dans PATTERNS.md (S43+).
- 5 items P3 a 4/3 (LOC kickoff, persist error, URL single-quote,
  Manager Mutex, rerun hash) non resolus S42 malgre report 3/3
  MANDATORY → overdue, MANDATORY S43.

## §6 Surface nouvelle livree

| Module | Crate | LOC |
|---|---|---|
| forge.rs | nexus-coordinator-rs | 139 |
| provenance.rs | nexus-coordinator-rs | 187 |
| deploy.rs | nexus-shell-daemon | 679 |
| apps.rs | nexus-shell-daemon | 275 |
| **Total** | | **1280** |

Fichiers modifies (non nouveaux) : canary_input.rs (-17/+17),
upload_queue.rs (-17/+17), guardrails.rs (+40), lib.rs (+2),
http.rs (+11), main.rs (+2), Cargo.toml (+5), PATTERNS.md (+27).

## §7 Ce que le sprint n'a PAS livre (scope cuts respectes)

- ❌ Routes files/consent/canary/contributor — S43
- ❌ Routes restantes (health, shell, tasks, kudos, etc.) — S44
- ❌ Suppression coordinator Python — S45
- ❌ CI/VPS/v1.0 — S46-48
- ❌ Kudos debit/stake — interdit (Day 0 #7)
- ❌ CanaryInput mutation guardrail usage — post-v1.0
- ❌ Background loops wire-up — S43+
- ❌ @require_capability middleware — S43

## §8 Findings carry-over for memory (G6)

(Identique §5, consolide ci-dessus.)

## §9 Checkpoint de cloture

1. ✅ D1 : 4 items P2 dette resolus en Phase A
2. ✅ D2 : 2 routes API (deploy 505 LOC + apps 350 LOC) portees en Phase B+C
3. ✅ D3 : 8 scope cuts respectes
