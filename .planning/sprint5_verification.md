# Sprint 5 — Verification (fail-fast checklist)

**Date**: 2026-04-11
**HEAD**: `git log --oneline master ^3b5c162` shows the Sprint 5
commit stack (expected tip: `feat(web): Sprint 5 Phase D — stubs,
polish, verification`).

Mirrors the 22-row fail-fast table frozen in
`.planning/sprint5_plan.md` §8. Every row is the exact command
and the observed outcome from the Sprint 5 run. The "Observed"
column is filled in verbatim — copy the command into a terminal
and you will see the same thing.

---

## How to re-run

```bash
# from repo root, with cargo + uv + node on PATH
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --exclude nexus-core-py --locked

uv run ruff format --check packages/ examples/
uv run ruff check packages/ examples/
uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q

cd web
npm install
npm run lint
npx tsc --noEmit -p tsconfig.app.json
npm run build
npx playwright test
bash scripts/scan-en-strings.sh
cd ..
```

Playwright requires `cargo build -p nexus-worker` first (the
state roundtrip test locates the binary under `target/debug/`).

---

## Checklist

| # | Check | Command | Critère | Observed |
|---|---|---|---|---|
| 1 | running.json écrit au start | `pytest packages/nexus-coordinator/tests/test_registry.py::test_running_json_written_on_start` | pass | **green** — file written, schema v1, all fields populated from live Coordinator state |
| 2 | running.json retiré au stop | `pytest packages/nexus-coordinator/tests/test_registry.py::test_running_json_removed_on_clean_stop` | pass | **green** — `remove_running_state` idempotent, second call is a no-op |
| 3 | /shell/discover liste les running | `pytest packages/nexus-coordinator/tests/test_shell_discover.py` | pass (3/3) | **3/3** — self entry, multi coordinator list, empty state |
| 4 | /worker-state proxy valide + stale | `pytest packages/nexus-coordinator/tests/test_worker_state_proxy.py` | pass (5/5) | **5/5** — absent, fresh, stale >15s, invalid JSON, schema mismatch |
| 5 | worker state_writer | `cargo test -p nexus-worker-core --lib state_writer` | pass (9/9) | **9/9** — schema v1, snapshot shape, null gpu, enrolled projects, atomic write, mkdir parents, overwrite atomicity, error swallow, iso_utc format |
| 6 | worker state roundtrip e2e | `pytest packages/nexus-coordinator/tests/test_worker_state_roundtrip.py` | pass | **green** — spawns `nexus-worker start --stub-ollama` in hermetic `NEXUS_GRID_ROOT`, waits for first flush, proxy returns live body matching Rust output |
| 7 | legacy cold-case pages removed | `test ! -f web/src/pages/Dashboard.tsx && test ! -f web/src/pages/Evidence.tsx` | exit 0 | **exit 0** — entire `web/src/pages/` rewritten; no stale cold-case files |
| 8 | zero legacy deps | `grep -cE "(antv/g6\|leaflet\|recharts\|reagraph\|sigma\|nivo\|force-graph\|graphology\|parliament\|@number-flow\|axios\|moment\|motion\|lenis)" web/package.json` | 0 | **0** — all 24 cold-case deps gone after D5 |
| 9 | TypeScript strict clean | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 | **exit 0** — strict mode with noUnusedLocals + noUnusedParameters, no errors |
| 10 | ESLint clean | `cd web && npm run lint` | exit 0 | **exit 0** — 0 errors, 5 pre-existing shadcn warnings (documented in `docs/shell/PATTERNS.md` T1) |
| 11 | Build prod clean | `cd web && npm run build` | exit 0, no warnings | **exit 0** — 425 KB main, 190 KB vendor, 90 KB CSS, zero warnings |
| 12 | Shell onboarding empty state | Playwright `shell-onboarding-empty-state.spec.ts` | pass | **pass** (~400 ms) |
| 13 | Add coordinator flow | Playwright `shell-add-coordinator.spec.ts` | pass | **pass** (~750 ms) |
| 14 | /my-projects live | Playwright `my-projects-live.spec.ts` | pass (live coord) | **pass** (~280 ms) |
| 15 | /project/:name manifest | Playwright `project-detail-manifest.spec.ts` | pass | **pass** (~400 ms) |
| 16 | Apps tab render | Playwright `apps-tab-render.spec.ts` | pass | **pass** (~560 ms) |
| 17 | /my-network reads worker live | Playwright `my-network-live.spec.ts` | pass | **pass** (~330 ms) — fixture state.json |
| 18 | /browse + /curators stubs | Playwright `stub-pages.spec.ts` | pass (2/2) | **pass** — both render their Sprint 6 copy, no 404 |
| 19 | All Rust tests | `cargo test --workspace --exclude nexus-core-py --locked` | ≥ 170 | **193 total** — 62 core-rs lib + 11 worker bin lib + 10 worker e2e + 105 worker-core lib (was 94 at Sprint 4, +9 state_writer + 2 paths) + 5 core-rs doctests |
| 20 | All Python coordinator tests | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q` | ≥ 38 | **43 passed + 1 skipped** — was 27 + 1 at Sprint 4. New: 6 registry, 3 shell_discover, 5 worker_state_proxy, 1 app_tab_descriptor, 1 worker_state_roundtrip |
| 21 | French UI | `bash web/scripts/scan-en-strings.sh` | 0 | **clean** — "src/ is French-only, clean" |
| 22 | E2E global shell → app | Playwright full `npx playwright test` | all pass | **8 passed in 7.8 s** — onboarding, add coord, my-projects, project detail, apps tab render, my-network, /browse stub, /curators stub — all against one live coordinator subprocess spawned by globalSetup |

---

## Summary

All 22 fail-fast rows pass. Sprint 5 delivers:

- **Day 0** (2 commits): `docs(sprint5): kickoff + detailed plan`
  (1113-line plan + 950-line kickoff, every decision frozen) and
  `refactor(web): drop legacy cold-case UI` (71 files deleted,
  24 top-level deps removed, -17747 LOC / +1815 LOC including
  the lockfile regen, npm audit fix bumped vite within the
  existing ^8.0.1 range).
- **Phase A — worker-core state_writer** (1 commit):
  `nexus_worker_core::paths`, `engine::state_writer`,
  `WorkerStateSnapshot` schema v1, atomic write, `config::Engine.state_flush_secs`
  (default 5 s, clamped ≥1 s on load), main-loop flush tick on
  boot / per interval / on graceful shutdown, `last_task`
  tracking on the success branch, `time` crate added at the
  workspace level for RFC 3339 timestamps, preexisting env-var
  test race fixed via broader `env_var_test_lock` hold.
- **Phase A — coordinator extensions** (1 commit): `registry.py`
  (RunningState pydantic model + write/remove/discover), two
  new routers `api/shell.py` (`/shell/discover`) and
  `api/worker_state.py` (`/worker-state` proxy with Pydantic
  WorkerStateV1 mirror of the Rust shape, staleness at 15 s),
  new `paths.running_state_path` / `paths.worker_state_path`
  helpers, CLI `start.py` writes + removes `running.json` in
  the boot + finally blocks.
- **Phase A — frontend shell** (1 commit): typed
  `api/coordinator.ts` (Zod-schema-first, 13 endpoints), Zustand 5
  persist `stores/projectStore.ts`, shadcn-based `AppShell` with
  sidebar + header coordinator picker, `AddCoordinatorDialog`
  with test+add flow, `OnboardingEmpty` landing, five Phase A
  stub pages, `App.tsx` + `main.tsx` rewrite with
  `QueryClientProvider`, eslint config with `allowConstantExport`.
- **Phase B — project detail + backend patch** (1 commit): five
  rich tab components under `components/project/`
  (OverviewTab, TasksTab, KudosTab, InvitesTab, AppsTab),
  `ProjectDetail.tsx` rewrite with shadcn `Tabs` and parallel
  React Query prefetch, `format.ts` pure helpers, new coordinator
  endpoint `GET /app/{name}/tabs/{tab_name}/descriptor` with its
  pytest.
- **Phase B — Playwright** (1 commit): Playwright 1.59 devDep,
  chromium install, `playwright.config.ts`, global-setup /
  global-teardown (hermetic `NEXUS_GRID_ROOT` + spawn real
  `nexus-coordinator init && start`), 5 specs — onboarding,
  add-coordinator, my-projects, project-detail, apps-tab.
  Root-cause fixes discovered by the first run: `CORSMiddleware`
  for loopback origins on the coordinator, `exclude_none=True`
  on `CoordinatorConfig.save` (pre-existing `init`-path bug),
  `NEXUS_GRID_ROOT` env support in `nexus_coordinator.paths`.
- **Phase C — /my-network live** (1 commit):
  `Network.tsx` rewrite (Identité, GPU, Projets enrôlés, Dernière
  tâche cards, 2 s polling, stale banner, worker-offline card),
  `nexus_worker_core::paths::nexus_grid_root` env override for
  the Rust side, end-to-end `test_worker_state_roundtrip.py`
  that spawns a real `nexus-worker start --stub-ollama` and
  reads the live snapshot through the coordinator proxy,
  Playwright `my-network-live.spec.ts` with a fixture snapshot.
- **Phase D — stubs, polish, verification** (this commit):
  Playwright `stub-pages.spec.ts` for /browse and /curators,
  `scripts/scan-en-strings.sh` French-only guard,
  `docs/shell/PATTERNS.md` (P1..P7 rules + T1..T3 tech debt),
  this verification document.

## What's NOT in this sprint (scope line)

Explicitly deferred per sprint5_plan.md §10:

- **nexus-shell-daemon** — sidecar with iroh Node for DHT /
  curator gossip. Sprint 6.
- **Schema-driven tab rendering** — vocabulary frozen in §2.2
  D2 but not implemented. Sprint 6 migrates hello-world-app
  and gov.
- **Curator list flow** — Ed25519 signing + gossip propagation
  + subscribe/unsubscribe. Sprint 6.
- **DHT browse (pkarr)** — stubbed on `/browse`. Sprint 6.
- **Full 19-tab `nexus-app-gov` migration** — still v1.1 per
  Sprint 4 scope.
- **Worker HTTP API (axum)** — rejected by D3, replaced by
  state.json flush + coordinator proxy. Sprint 6+ revisits.
- **Mobile responsive < 1280px** — confirmed desktop-only.
- **Vitest unit tests for format helpers / projectStore** —
  deferred to Sprint 6 with the schema-driven tab work (T3
  in `docs/shell/PATTERNS.md`).
- **Command palette Ctrl+K + keyboard shortcuts** — planned
  in §7.2 but deprioritised once the Playwright suite
  landed. Nice-to-have for Sprint 6 polish.
- **Bundle size CI check** — T2 in `docs/shell/PATTERNS.md`.

## Git summary

```
$ git log --oneline master ^3b5c162
<tip>      feat(web): Sprint 5 Phase D — stubs, polish, verification
376292a    feat(web,worker-core,coordinator): Sprint 5 Phase C — /my-network live worker state
902117b    feat(web,coordinator): Sprint 5 Phase B — Playwright e2e against live coordinator
e445da2    feat(web,coordinator): Sprint 5 Phase B — project detail with 5 tabs + async descriptor endpoint
127e011    feat(web): Sprint 5 Phase A — shell chrome, coordinator client, project store
c1be4cd    feat(coordinator): Sprint 5 Phase A — running.json registry + shell discover + worker state proxy
d77f122    feat(worker-core): Sprint 5 Phase A — state writer for shell integration
2410c0a    refactor(web): drop legacy cold-case UI, archive via git history
82691ce    docs(sprint5): kickoff + detailed plan
```

9 commits on top of `3b5c162` (Sprint 4 tip). Sprint 5 closed.

**Next sprint**: Sprint 6 — schema-driven tab rendering, curator
list gossip flow, nexus-shell-daemon, DHT browse, Ctrl+K command
palette, and the 19-tab nexus-app-gov migration tracked under
v1.1 of the phoenix plan.
