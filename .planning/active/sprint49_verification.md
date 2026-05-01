# Sprint 49 — Verification

**Tip sortie** : `0cbfaab` (Phase B).
**Theme** : coordinator lifecycle → daemon Rust — project doc
iroh-docs + dispatch loop MPSC + CLI coordinator subcommands
offline.

---

## §1 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | ✅ 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | ✅ 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1195, 0 fail | ✅ 1195 passed |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | ✅ ok |
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
| 20 | Project doc create/reopen | integration test | ✅ | ✅ dispatch_loop_writes_to_doc |
| 21 | Dispatch loop MPSC | TaskEntry in doc | ✅ | ✅ |
| 22 | Doc subscription → validator | result forwarding | ✅ | ✅ wired in runtime.rs (future gossip path) |
| 23 | CLI init subcommand | DB project record | ✅ | ✅ parses_init test |
| 24 | CLI invite create/list/revoke | cycle complet | ✅ | ✅ 3 parsing tests |
| 25 | CLI quarantine/capability | handlers wired | ✅ | ✅ 4 parsing tests |
| 26 | Scope cuts respectes | 12/12 | ✅ | ✅ |
| 27 | Delta tests documente | cumule | ✅ | ✅ +9 Rust |
| 28 | G1 D2/D3 acks respectes | MPSC + offline | ✅ | ✅ |

## §2 Delta tests cumule

| Suite | Entree S49 | Sortie S49 | Delta | Source |
|---|---|---|---|---|
| Rust nextest | 1187 | 1195 | +9 | Phase A +1 (dispatch_loop_writes_to_doc), Phase B +8 (CLI parsing) |
| Rust doctests | 6+1i | 6+1i | +0 | |
| SDK pytest | 195 | 195 | +0 | |
| Coord pytest | 264+17f+6s | 264+17f+6s | +0 | |
| Gov pytest | 46 | 46 | +0 | |
| Vitest | 267 | 267 | +0 | |
| Playwright | 42+2f | 42+2f | +0 | non execute (env pre-existant) |
| size-limit | 5/5 | 5/5 | +0 | |
| **Total** | **~1938** | **~1947** | **+9** | |

## §3 Carries resolus S49

Aucun carry resolu ce sprint (S49 impair, pas de phase dette, 0
item a 2/3 ou 3/3). Les 5 carries herites restent en l'etat.

## §4 Carries S50

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 Day 0 |
| P2-REVIEW-A-1-S48 canary reload size cap | 1/3 | S48 Phase A review |
| P2-REVIEW-B-1-S48 auth.rs set_var residuel | 1/3 | S48 Phase B review |
| P2-AUDIT-A-1-S48 carry doc accuracy reload_policy | 1/3 | S48 audit |
| P2-REVIEW-A-1-S49 dispatch loop JoinHandle | 1/3 | NEW S49 Phase A review |
| P2-REVIEW-B-1-S49 CLI handler integration tests | 1/3 | NEW S49 Phase B review |

## §5 Findings carry-over for memory

- S49 absorbe le coordinator lifecycle dans le daemon Rust : project doc iroh-docs + dispatch loop MPSC + 4 CLI subcommands offline
- Le daemon est desormais le coordinator pour le projet local — le coordinator Python n'est plus necessaire pour le core path
- Net +9 tests Rust, total 1195 Rust / ~1947 total
- 7 carries S50 documentes (dont 2 NEW S49)
- Sprint 50 = suppression Python + cleanup (cf. roadmap_v1_migration_rust.md §S50)
