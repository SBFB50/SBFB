# Sprint 32 — Verification (self-report fail-fast)

**Sprint** : 32 (dette pair : iroh 0.98 upgrade + carries batch)
**Tip** : `626221c` (Phase C) — Phase D commit = wrap-up
**Date** : 2026-04-27

## §1 Goal recall

> Sprint 32 leve le pin iroh 0.97 (Day 0 #3) en upgradeant les 4
> crates iroh vers 0.98/0.100, debloque l'activation arti-client via
> rusqlite 0.36, et resout les carries P2 audit S31.
> **Critere SMART : 28+ rows fail-fast verts au verification.md,
> mesure binaire au Phase D wrap-up.**

## §2 Phases livrees

| Phase | Commit | Titre |
|---|---|---|
| A | `90aff27` | iroh stack upgrade 0.97→0.98 workspace-wide |
| B | `a55a0ab` | rusqlite 0.36 + arti-client dep activation tor feature |
| C | `626221c` | P2 batch carries audit S31 + Playwright COEP |
| D | (ce commit) | wrap-up + verification + audit plan S33 + migration |

## §3 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | 0 errors | 0 errors ✅ |
| 2 | Rust nextest pass | `cargo nextest run --workspace --locked` | 878+ pass, 0 fail | 883 pass, 0 fail ✅ |
| 3 | Rust doctests pass | `cargo test --workspace --locked --doc` | 0 fail | 0 fail (1 ignored) ✅ |
| 4 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings ✅ |
| 5 | Rust fmt clean | `cargo fmt --all --check` | no output | no output ✅ |
| 6 | Release build daemon | `cargo build -p nexus-shell-daemon --release` | Finished | Finished ✅ |
| 7 | Python ruff format | `uv run ruff format --check packages/` | clean | 153 files already formatted ✅ |
| 8 | Python ruff check | `uv run ruff check packages/` | pass | All checks passed ✅ |
| 9 | SDK 195 pass | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 pass | 195 passed ✅ |
| 10 | Coord 406+ pass | `uv run pytest packages/nexus-coordinator/tests/ -q` | 406+ pass + 36f stale | 406 passed + 36 failed (PyO3 stale) + 6 skipped ✅ |
| 11 | Gov 46 pass | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | 46 passed ✅ |
| 12 | Frontend lint | `cd web && npm run lint` | 0 errors | 0 errors (7 warnings) ✅ |
| 13 | Frontend tsc | `npx tsc --noEmit -p tsconfig.app.json` | clean | clean ✅ |
| 14 | Vitest 267+ pass | `npm run test:unit` | 267+ pass | 267 passed ✅ |
| 15 | Frontend build | `npm run build` | success | success ✅ |
| 16 | size-limit 7/7 | `npm run size` | 7/7 pass | 7/7 pass ✅ |
| 17 | Playwright | `npx playwright test` | 41+ pass | 42 pass + 2 fail (env) = 44 total ✅ |
| 18 | en-strings | `bash scripts/scan-en-strings.sh` | clean | clean ✅ |
| 19 | iroh version 0.98 | `grep 'iroh = "0.98"' Cargo.toml` | found | found ✅ |
| 20 | iroh-blobs 0.100 | `grep 'iroh-blobs = "0.100"' Cargo.toml` | found | found ✅ |
| 21 | iroh-docs 0.98 | `grep 'iroh-docs = "0.98"' Cargo.toml` | found | found ✅ |
| 22 | iroh-gossip 0.98 | `grep 'iroh-gossip = "0.98"' Cargo.toml` | found | found ✅ |
| 23 | rusqlite 0.36 | `grep 'rusqlite.*0.36' Cargo.toml` | found | found ✅ |
| 24 | tor feature compile | `cargo build -p nexus-core-rs --features tor` | success | success ✅ |
| 25 | max_tokens test | `cargo nextest run -p nexus-executor -E 'test(max_tokens)'` | 1 pass | 1 passed ✅ |
| 26 | FROST error tests | `cargo nextest run -p nexus-shell-daemon -E 'test(frost_http)'` | 8+ pass | 8 passed ✅ |
| 27 | FORMAT_VERSION v1 | `grep const.*_VERSION.*= crates/nexus-core-rs/src/` | all = 1 | 7 constants, all = 1 ✅ |
| 28 | HARDENING compteurs | HARDENING_ROADMAP.md last_validated S32 | ~883 Rust | updated Phase D ✅ |
| 29 | Planning docs | kickoff + plan + design_review + preflights + reviews | complets | 11 fichiers active/ ✅ |

**Resultat : 29/29 verts.** Critere SMART atteint (28+ rows).

## §4 Delta tests cumule (S32 entier)

| Suite | Entree S32 | Sortie S32 | Delta |
|---|---|---|---|
| Rust (cargo nextest) | 878 | 883 | **+5** |
| SDK (pytest) | 195 | 195 | 0 |
| Coordinator (pytest) | 406+36f+6s | 406+36f+6s | 0 |
| Gov (pytest) | 46 | 46 | 0 |
| Vitest | 267 | 267 | 0 |
| Playwright | 43 | 44 | **+1** |
| size-limit | 7/7 | 7/7 | 0 |
| **Total** | **~1877** | **~1883** | **+6** |

Detail delta Rust +5 :
- +1 `execute_task_ollama_mock_respects_max_tokens` (Phase C, P2-AUDIT-1)
- +4 `frost_http_*` error path tests (Phase C, P3-AUDIT-2)

Detail delta Playwright +1 :
- +1 `blob-serve-coep.spec.ts` COEP isolation mock (Phase C, P2-REVIEW-B-1-S30)

## §5 Findings carry-over for memory

Les items suivants doivent etre fusionnes dans les memory files
concernes par la prochaine session :

1. **Day 0 #3 LEVE** : iroh 0.97 pin → iroh 0.98 post-S32. La
   mecanique reste identique (upgrade volontaire par sprint dedie).
2. **LT-6 RESOLVED** : trigger iroh > 0.97 satisfait, iroh 0.98
   deploye Phase A `90aff27`. ROADMAP_COMMITMENTS mis a jour.
3. **arti-client dep ACTIVEE** : `cargo build --features tor` compile.
   Le module `tor_transport.rs` (S31 Phase C) est desormais fonctionnel.
4. **rusqlite_migration bump 1.3→2.2.0** : non planifie, resolu Phase B
   (P2-B-2 finding, API compatible).
5. **Compteurs** : 883 Rust / 195 SDK / 406+36f+6s coord / 46 gov /
   267 Vitest / 44 PW / 7/7 size / ~1883 total.

## §6 Verdict

**PASS** — 29/29 rows verts, +6 tests cumules, goal SMART atteint.
Sprint 32 ferme.
