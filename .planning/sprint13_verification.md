# Sprint 13 — Verification (self-report fail-fast)

**HEAD entree** : `53a9e32` (Sprint 12 P1 fix deploy size limit)
**HEAD sortie** : `72cf5ad` (Sprint 13 Phase D — launcher)
**Ecrit** : 2026-04-13

---

## 1. Commit stack

```
72cf5ad feat(launcher): Sprint 13 Phase D — minimal Rust launcher with browser open
c32d9c7 feat(bridge): Sprint 13 Phase C — postMessage bridge MVP with task submit + storage
7d669f2 feat(p2p): Sprint 13 Phase B — open source enforcement for public apps
c0a655b feat(web): Sprint 13 Phase A — UI Netflix glassmorphism + T37-T40
b44f40c docs(sprint13): kickoff + plan detaille with D1-D6
53a9e32 fix(sprint12): add 100MB upload size limit to deploy endpoint
```

5 commits Sprint 13 (1 planning + 4 phases A-D) + 1 Phase E docs a venir.

---

## 2. How to re-run

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

# Python
uv run ruff format --check packages/ examples/
uv run ruff check packages/ examples/
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

# SPDX
bash scripts/check-spdx.sh
```

---

## 3. Checklist fail-fast

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | exit 0 | exit 0 |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | 0 warning |
| 3 | cargo test | `cargo test --workspace --locked` | >= 369 | 369 (80+3+39+6+110+11+10+105+5) |
| 4 | ruff format | `uv run ruff format --check packages/ examples/` | exit 0 | exit 0 |
| 5 | ruff check | `uv run ruff check packages/ examples/` | exit 0 | exit 0 |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | >= 183 | 183 passed |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | >= 99+1 | 99 passed, 1 skipped |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 | 46 passed |
| 9 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | exit 0 | exit 0 |
| 10 | eslint | `npm run lint` | 0 errors | 0 errors (6 warnings pre-existing) |
| 11 | vitest | `npm run test:unit` | >= 187 | 191 passed |
| 12 | build | `npm run build` | exit 0 | exit 0 |
| 13 | size-limit | `npm run size` | 7/7 green | 7/7 green (css 117/130 KB) |
| 14 | playwright | `npx playwright test` | >= 30 | 30 passed |
| 15 | scan-en | `bash scripts/scan-en-strings.sh` | exit 0 | exit 0 |
| 16 | SPDX | `bash scripts/check-spdx.sh` | >= 217 | 220 |
| 17 | T37-T40 CLOSED | `grep CLOSED docs/shell/PATTERNS.md` | 4 | 4 (T37-T40) |
| 18 | repo_url serde | `cargo test repo_url` | >= 2 green | 3 passed (v3 roundtrip + v2 compat + omit) |
| 19 | bridge tests | `npm run test:unit -- bridge` | >= 5 green | 9 passed |
| 20 | launcher build | `cargo build -p nexus-launcher` | exit 0 | exit 0 |
| 21 | public deploy no repo | `uv run pytest -k test_deploy_public_without_repo` | 1 pass | 1 passed |
| 22 | private deploy no repo | `uv run pytest -k test_deploy_private_without_repo` | 1 pass | 1 passed |

**22/22 verte.**

---

## 4. Metriques sprint

| Suite | Avant (53a9e32) | Apres (72cf5ad) | Delta |
|---|---|---|---|
| Rust workspace | 362 | 369 | +7 |
| Python SDK | 182 | 183 | +1 |
| Python coordinator | 96+1 | 99+1 | +3 |
| Python app-gov | 46 | 46 | = |
| Vitest unit | 180 | 191 | +11 |
| Playwright | 30 | 30 | = |
| size-limit | 7/7 | 7/7 | = |
| SPDX | 215 | 220 | +5 |

**Total projet : ~908 tests** (+22 vs Sprint 12 sortie ~897).

---

## 5. Surface nouvelle livree

| Module | LOC | Description |
|---|---|---|
| web/src/bridge/protocol.ts | 75 | Zod schemas bridge request/response |
| web/src/bridge/useBridge.ts | 125 | Host-side bridge listener hook |
| web/public/sbfb-bridge.js | 120 | SDK client pour apps iframe |
| web/src/bridge/__tests__/ | 130 | 9 tests protocol + useBridge |
| crates/nexus-launcher/ | 200 | Launcher binary (spawn + open + shutdown) |
| deploy.py (delta) | 30 | repo_url validation + propagation |
| publish.rs (delta) | 40 | v3 repo_url field + tests |
| browse.rs (delta) | 10 | repo_url propagation |
| http.rs (delta) | 20 | PublishRequest repo_url + CSP middleware |
| Pages glassmorphism (delta) | 600 | Projects/ProjectDetail/Network/Curators rewrite |

**Total delta : ~1350 LOC nouvelles/modifiees.**

---

## 6. Ce que le sprint n'a PAS livre (scope cuts respectes)

- CPU watchdog iframe → Sprint 14 (D6)
- Branding SBFB (nom, logo, favicon) → Sprint 14
- Runtime templates → Sprint 14
- Re-publish auto → Sprint 14
- Origin separee subdomain → Sprint 14+
- GitHub API verification repo_url → Sprint 14
- Bidirectional push (host → iframe events) → Sprint 14
- System tray icon → Sprint 14+
- Windows installer → Sprint 14+
- Multi-writer iroh-docs → v1.1+

---

## 7. Checkpoint de cloture

1. Checklist 22/22 verte
2. 6 commits atomiques (planning + A-D + E docs)
3. `sprint13_verification.md` + `sprint13_audit_plan.md` ecrits
4. PATTERNS.md a jour (T37-T40 CLOSED)
5. Bridge: une app iframe peut appeler `bridge.submitTask()` via postMessage
6. Open source: publish public sans repo_url → 400
7. Launcher: `cargo run -p nexus-launcher -- --help` fonctionne
8. Glassmorphism: toutes les pages ont le design Netflix dark
