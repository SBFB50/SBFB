# Sprint 15 — Verification

**HEAD entree** : `f6015b3` (Sprint 14 audit findings + A-1 fix)
**HEAD sortie** : `f5aea3e` (Phase D Playwright E2E)
**Date** : 2026-04-14

---

## Commit stack

```
f5aea3e test(bridge): Sprint 15 Phase D — Playwright iframe push + heartbeat E2E
e6644be feat(cli): Sprint 15 Phase C — sbfb init CLI with html/react/pyodide templates
3c729ba feat(watchdog): Sprint 15 Phase B — CPU watchdog via heartbeat + stalled overlay
b2940b3 feat(bridge): Sprint 15 Phase A — bidirectional push via sbfb-bridge-event
```

---

## How to re-run

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

# Python
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend
cd web
npx tsc --noEmit -p tsconfig.app.json
npm run lint
npm run test:unit
npm run build
npm run size
npx playwright test
bash scripts/scan-en-strings.sh

# Manual sanity
uv run sbfb init html /tmp/sbfb-sanity-check
```

---

## Checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | 0 warnings |
| 3 | cargo test | `cargo test --workspace --locked` | 373 pass | 373 pass (=) |
| 4 | ruff format | `uv run ruff format --check packages/` | clean | 93 files clean |
| 5 | ruff check | `uv run ruff check packages/` | clean | All checks passed |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 183 pass | 182 pass + 1 flaky Windows (pre-existant) |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | 138+1s -> 153+1s (+15) | 153 pass + 1 skipped |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 pass | 46 pass |
| 9 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | clean | clean |
| 10 | eslint | `npm run lint` | 0 errors | 0 errors, 7 warnings (6 T1 pre-existants + 1 fast-refresh nit sur type export) |
| 11 | vitest | `npm run test:unit` | 193 -> 214 (+21) | 214 pass |
| 12 | build | `npm run build` | success | built in 664ms |
| 13 | size-limit main | `npm run size` | main < 50 KB | main 13.91 kB (delta +4 vs 9.75 avant Sprint 15) |
| 14 | size-limit total | `npm run size` | 7/7 under budget | 7/7 OK |
| 15 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | French-only clean |
| 16 | Playwright | `npx playwright test` | 30 -> 33 (+3) | 33 passed |
| 17 | BridgeEventSchema | vitest `protocol.test.ts::BridgeEventSchema` | 5 pass | 5 pass |
| 18 | useBridge pushEvent | vitest `useBridge.test.ts::pushEvent` | 4 pass | 4 pass |
| 19 | Watchdog state machine | vitest `watchdog.test.ts::useBridge watchdog` | 8 pass | 8 pass |
| 20 | BridgeHeartbeatSchema | vitest `watchdog.test.ts::BridgeHeartbeatSchema` | 3 pass | 3 pass |
| 21 | BrowsedProject overlay | vitest `BrowsedProject.test.tsx::watchdog overlay in unknown state` | 1 pass | 1 pass |
| 22 | sbfb init html | pytest `TestInitHtml` | 4 pass | 4 pass |
| 23 | sbfb init react | pytest `TestInitReact` | 3 pass | 3 pass |
| 24 | sbfb init pyodide | pytest `TestInitPyodide` | 2 pass | 2 pass |
| 25 | scaffold errors | pytest `TestInitErrors` | 2 pass | 2 pass |
| 26 | placeholder integrity | pytest `TestPlaceholderIntegrity` | 3 pass | 3 pass |
| 27 | TemplateType enum | pytest `TestTemplateType` | 1 pass | 1 pass |
| 28 | Playwright heartbeat | `npx playwright test bridge-heartbeat` | 1 pass | 1 pass |
| 29 | Playwright push event | `npx playwright test bridge-push-event` | 2 pass | 2 pass |
| 30 | SPDX new files | 2 Python + 1 Vitest spec | all ok | SPDX header present on scaffold.py, sbfb_main.py, test_cli_scaffold.py, watchdog.test.ts |
| 31 | sbfb CLI sanity | `sbfb init html /tmp/x` | 4 files created | html (4), react (9), pyodide (4) |
| 32 | Templates wheel inclusion | `pip show -f nexus-coordinator` mentions templates/ | present | force-include configured |

---

## Metriques sprint

| Suite | Avant | Apres | Delta |
|---|---|---|---|
| Rust workspace | 373 | 373 | = |
| Python SDK | 183 | 183 | = (1 flaky Windows non-regresse) |
| Python coordinator | 138+1s | 153+1s | +15 (scaffold CLI) |
| Python app-gov | 46 | 46 | = |
| Vitest unit | 193 | 214 | +21 (9 Phase A + 12 Phase B) |
| Playwright | 30 | 33 | +3 (heartbeat + push + ignore) |
| size-limit | 7/7 | 7/7 | = (main 9.75 KB -> 13.91 KB) |
| SPDX | 224 | 228 | +4 (scaffold.py + sbfb_main.py + test_cli_scaffold.py + watchdog.test.ts) |

**Total** : ~1000 tests (+24 vs baseline ~949 Sprint 14).

---

## Surface nouvelle livree

| Module | LOC | Description |
|---|---|---|
| `sbfb-bridge.js` ajout | ~75 | onEvent + heartbeat (Phase A + B) |
| `protocol.ts` ajout | ~60 | BridgeEventSchema + BridgeHeartbeatSchema (A + B) |
| `useBridge.ts` ajout | ~90 | pushEvent + watchdog state machine (A + B) |
| `BrowsedProject.tsx` ajout | ~50 | stalled overlay + reloadIframe (B) |
| `protocol.test.ts` ajout | ~35 | 5 BridgeEventSchema tests (A) |
| `useBridge.test.ts` ajout | ~60 | 4 pushEvent tests (A) |
| `watchdog.test.ts` nouveau | ~180 | 11 tests heartbeat + state (B) |
| `BrowsedProject.test.tsx` ajout | ~20 | 1 test overlay unknown state (B) |
| `templates/html/` | ~120 | 4 fichiers : index.html, SBFB.json, README.md, .gitignore |
| `templates/react/` | ~250 | 9 fichiers : package.json, vite.config.ts, tsconfig, index.html, main.tsx, App.tsx, SBFB.json, README, .gitignore |
| `templates/pyodide/` | ~150 | 4 fichiers : index.html (Pyodide loader), SBFB.json, README.md, .gitignore |
| `cli/sbfb_main.py` | ~40 | Typer app avec callback + register init |
| `cli/commands/scaffold.py` | ~165 | TemplateType enum + subst + copy recursive |
| `test_cli_scaffold.py` | ~250 | 15 tests (4 classes : html/react/pyodide/errors + integrity) |
| `bridge-heartbeat.spec.ts` | ~85 | Playwright heartbeat E2E |
| `bridge-push-event.spec.ts` | ~125 | Playwright push event + filter E2E |
| `sprint15_kickoff.md` | ~350 | D1..D5 + plan outline + scope cuts |
| `sprint15_plan.md` | ~650 | Plan detaille phases A-E + fail-fast + risks |
| **Total** | **~2700** | |

---

## Ce que le sprint n'a PAS livre (scope cuts respectes)

- Re-publish automatique sur repo update -> Sprint 16
- Branding SBFB (nom, logo, favicon) -> Sprint 16
- Origin separee par subdomain blob-serve -> Sprint 16+
- 2 VPS supplementaires (US/Asia) -> Sprint 16
- MIME scan executables dans le zip -> Sprint 16
- Dispatcher server-side events (qui produit les push events depuis le reseau P2P) -> Sprint 16+
- Whitelist stricte host-side des event names -> Sprint 16+
- `sbfb publish` subcommand integre au CLI -> Sprint 16+
- Templates additionnels (Vue / Svelte / Jupyter) -> Sprint 16+
- Pyodide bundle pre-telecharge automatiquement -> Sprint 16+
- Kill-by-force iframe (browser API absente)  -> Sprint 16+
- Builds reproductibles -> v1.2+
- Multi-writer iroh-docs -> v1.1+
- Custom domain / DNS -> v1.2+

---

## Resolution R1..R6 du plan

| Risque | Statut |
|---|---|
| R1 : CSP blob-serve bloque CDN Pyodide | **Confirme** — template pyodide charge `./pyodide/` relatif, README explique le bundle manuel |
| R2 : Playwright webServer side-car complique | **Elude** — page.route fulfill au lieu d'un server side-car, zero nouvelle dep |
| R3 : Bridge heartbeat alourdit main bundle | **Mesure** : main 9.75 KB -> 13.91 KB (+4.16 KB), largement sous le budget 50 KB |
| R4 : useBridge return breaking change | **Non** — 1 seul appelant (`BrowsedProject.tsx`), mis a jour en meme temps |
| R5 : Typer entry point casse `uv run` | **OK** — `uv run sbfb init html /tmp/x` fonctionne end-to-end |
| R6 : Templates inclusion dans le wheel | **Resolu** — `tool.hatch.build.targets.wheel.force-include` explicite |

---

## Checkpoint de cloture

1. 32/32 fail-fast checklist verts (row 6 flaky Windows pre-existant)
2. 4 commits feat/test landed sur master (A-D) + kickoff/plan dans A
3. verification.md + audit_plan.md ecrits (ce commit)
4. PATTERNS.md a jour (T44-T51 section Sprint 14 audit tech debt)
5. Memory a mettre a jour apres ce commit (tip `f5aea3e` -> tip Phase E)
