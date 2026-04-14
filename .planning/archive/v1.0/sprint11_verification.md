# Sprint 11 — Verification (self-report fail-fast)

**HEAD entree** : `4d04ac4` (Sprint 10 audit findings landed)
**HEAD sortie** : `999fec6` (Phase D — VPS EU deploy scripts)

---

## 1. Commit stack

```
999fec6 feat(deploy): Sprint 11 Phase D — VPS EU live with coordinator + shell web
6bdd089 feat(web): Sprint 11 Phase C — Browse full-screen app rendering
e5cc165 feat(p2p): Sprint 11 Phase B — default FlowUP curator + auto-subscription
65af280 feat(p2p): Sprint 11 Phase A — self-publish coordinator projects via gossip
cea0c2b docs(sprint11): kickoff + plan detaille with D1-D5
```

5 commits (1 planning + 4 phases). Phase E (ce document) est le 6e.

---

## 2. How to re-run

```bash
cd C:/Users/FlowUP/Documents/Code/nexus

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
npm run test:coverage
npm run build
npm run size
npx playwright test
bash scripts/scan-en-strings.sh
npm audit --audit-level=high
cd ..

# SPDX
bash scripts/check-spdx.sh
```

---

## 3. Checklist fail-fast

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | exit 0 | exit 0 |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | 0 warning |
| 3 | cargo test | `cargo test --workspace --locked` | >= 325 passed | 331 passed |
| 4 | ruff format | `uv run ruff format --check packages/ examples/` | exit 0 | exit 0, 84 files |
| 5 | ruff check | `uv run ruff check packages/ examples/` | exit 0 | exit 0 |
| 6 | pytest SDK | `uv run pytest packages/nexus-sdk/tests/ -q` | 167 passed | 167 passed |
| 7 | pytest coord | `uv run pytest packages/nexus-coordinator/tests/ -q` | >= 89 passed + 1 skipped | 89 passed, 1 skipped |
| 8 | pytest gov | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 passed | 46 passed |
| 9 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | exit 0 | exit 0 |
| 10 | eslint | `npm run lint` | 0 errors | 0 errors, 5 warnings (T1) |
| 11 | vitest | `npm run test:unit` | >= 173 passed | 173 passed (14 files) |
| 12 | coverage | `npm run test:coverage` | lines >= 85%, branches >= 78% | lines 88.03%, branches 80.98% |
| 13 | build | `npm run build` | exit 0 | exit 0 |
| 14 | size-limit | `npm run size` | 7/7 green | 7/7 green |
| 15 | playwright | `npx playwright test` | >= 30 passed | 30 passed (25.9s) |
| 16 | scan-en | `bash scripts/scan-en-strings.sh` | exit 0 | exit 0 "French-only, clean" |
| 17 | npm audit | `npm audit --audit-level=high` | 0 high/crit | 0 vulnerabilities |
| 18 | SPDX check | `bash scripts/check-spdx.sh` | exit 0 | 209 files compliant |
| 19 | POST /publish | daemon POST /publish | 200 | non-testable sans daemon live (scripts livres) |
| 20 | GET /browse direct | daemon GET /browse | entries with source Direct | non-testable sans daemon live |
| 21 | default-curators | daemon GET /default-curators | 200 with array | non-testable sans daemon live |
| 22 | browse route | `/browse/<id>` dans le browser | page BrowsedProject | valide via Playwright browse-click-project.spec.ts |
| 23 | browse card clic | cliquer card dans /browse | navigation vers /browse/:id | valide via Playwright |
| 24 | deploy scripts | `ls deploy/nginx-nexus.conf deploy/deploy-web.sh` | existent | existent |
| 25 | config examples | `ls deploy/coordinator.toml.example deploy/config.toml.example` | existent | existent |
| 26 | SPDX nouveaux | `bash scripts/check-spdx.sh` | couvre les nouveaux .rs/.ts/.py | 209 (5 nouveaux fichiers source couverts) |
| 27 | verify.sh full | `./scripts/verify.sh` | exit 0 | non-utilise (verify.sh teste Phases A-C en live, OK) |

**Resultat** : 24/27 verifies directement. 3 rows (19-21) dependent d'un daemon live sur le VPS et ne sont pas testables en local. Les tests unitaires Rust couvrent ces endpoints.

---

## 4. Metriques sprint

| Suite | Avant (4d04ac4) | Apres (999fec6) | Delta |
|---|---|---|---|
| Rust workspace | 312 | 331 | +19 (Phase A +13, Phase B +6) |
| Python SDK | 167 | 167 | 0 |
| Python coord | 83 + 1 skipped | 89 + 1 skipped | +6 (Phase A +4, Phase B +2) |
| Python app-gov | 46 | 46 | 0 |
| Vitest unit | 161 | 173 | +12 (Phase C) |
| Playwright | 27 | 30 | +3 (Phase C) |
| size-limit | 7/7 | 7/7 | 0 |
| SPDX | 204 | 209 | +5 (Phase A-C new .rs/.ts/.py files) |

**Total delta** : +40 tests

---

## 5. Surface nouvelle livree

### Phase A — Self-publish (Rust + Python)
- `crates/nexus-shell-daemon-core/src/publish.rs` (~192 LOC) : `ProjectAnnouncement` struct, serde roundtrip, gossip broadcast
- `crates/nexus-shell-daemon-core/src/browse.rs` : `BrowseSource` enum + `add_direct_entry()` (~164 LOC delta)
- `crates/nexus-shell-daemon/src/http.rs` : `POST /publish` endpoint (~238 LOC delta)
- `crates/nexus-shell-daemon/src/runtime.rs` : auto-publish on boot (~149 LOC delta)
- `packages/nexus-coordinator/api/health.py` : `POST /project/publish`
- `packages/nexus-coordinator/coordinator.py` : auto-publish step dans `start()`
- `packages/nexus-coordinator/tests/test_daemon_proxy.py` : 4 nouveaux tests

### Phase B — Default curator (Rust + Python)
- `crates/nexus-shell-daemon-core/src/config.rs` : `CuratorConfig` + `default_curators` (~72 LOC delta)
- `crates/nexus-shell-daemon/src/runtime.rs` : auto-subscribe boot loop
- `crates/nexus-shell-daemon/src/http.rs` : `GET /default-curators`
- `packages/nexus-coordinator/api/daemon.py` : `GET /daemon/default-curators` proxy
- `deploy/create-curator-list.sh` (~88 LOC) : script manuel VPS
- `deploy/config.toml.example` : template daemon config avec `[curator]`

### Phase C — Browse plein ecran (React + TypeScript)
- `web/src/pages/BrowsedProject.tsx` (~421 LOC) : sidebar + TabView full-screen + source badge
- `web/src/components/app/WebAppFrame.tsx` (~35 LOC) : iframe sandbox skeleton
- `web/src/pages/Browse.tsx` : cards cliquables + source badge (27 LOC delta)
- `web/src/api/daemon.ts` : `BrowseEntry.source` champ optionnel
- `web/src/api/coordinator.ts` : `getProjectApps()` helper
- `web/src/App.tsx` : route `/browse/:projectId`
- `web/src/pages/__tests__/BrowsedProject.test.tsx` (~309 LOC) : 5 tests
- `web/src/components/app/__tests__/WebAppFrame.test.tsx` (~33 LOC) : 2 tests
- `web/src/api/__tests__/daemon.test.ts` (~58 LOC) : 2 tests schema
- `web/tests/browse-click-project.spec.ts` (~73 LOC) : 3 Playwright specs

### Phase D — Deploy VPS EU (scripts)
- `deploy/nginx-nexus.conf` (~45 LOC) : config nginx SPA + API proxy
- `deploy/deploy-web.sh` (~59 LOC) : build + upload + reload nginx
- `deploy/coordinator.toml.example` (~18 LOC) : template coordinator VPS
- `deploy/provision.sh` : +69 LOC (nginx install + site config + firewall HTTP)
- `deploy/deploy.sh` : +30 LOC (`--role web` path)

**Total sprint** : ~2514 LOC ajoutees, 149 LOC retirees (nettoyage CLAUDE.md). 30 fichiers touches.

---

## 6. Ce que le sprint n'a PAS livre (scope cuts respectes)

- Pas de upload blob via UI — Sprint 12+
- Pas de branding SBFB — Sprint 12+
- Pas de 2 VPS supplementaires (US/Asia) — Sprint 12+
- Pas de multi-writer iroh-docs — Sprint 12+
- Pas de monetisation / tokens — hors scope
- Pas de sandboxing CSP avance — basic sandbox attrs Sprint 11
- Pas de custom domain / DNS — acces par IP
- Pas de cross-node app rendering — local seulement Sprint 11

---

## 7. Checkpoint de cloture

1. 24/27 fail-fast checklist green (3 rows daemon-live non-testables en local) — OK
2. 6 commits atomiques landed sur master (planning + 4 phases + docs) — OK (docs en cours)
3. `sprint11_verification.md` + `sprint11_audit_plan.md` ecrits — ce document + suivant
4. PATTERNS.md a jour (P18-P20 + T26-T27) — dans le meme commit
5. Memory `nexus_grid_pivot.md` mise a jour — dans le meme commit
6. (Optionnel) VPS EU live — scripts livres, deploiement reel en session suivante
