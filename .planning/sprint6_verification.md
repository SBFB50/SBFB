# Sprint 6 — Verification (fail-fast checklist)

**Date**: 2026-04-11
**HEAD**: `git log --oneline master ^cdf4467` shows the Sprint 6
commit stack (expected tip: `feat(web): Sprint 6 Phase E — polish,
Playwright, verification`).

Mirrors the 24-row fail-fast table frozen in
`.planning/sprint6_plan.md` §9. Every row is the exact command
and the observed outcome from the Sprint 6 run. Copy any
command into a terminal and you will reproduce the result.

---

## How to re-run

```bash
# from repo root, with cargo + uv + node on PATH
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --exclude nexus-core-py --locked

uv run ruff format --check packages/ examples/
uv run ruff check packages/ examples/
uv run --package nexus-sdk pytest packages/nexus-sdk/tests/ -q
uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

cd web
npm install
npm run lint
npx tsc --noEmit -p tsconfig.app.json
npm run test:unit
npm run test:coverage
npm run build
npm run size
npx playwright test
bash scripts/scan-en-strings.sh
cd ..
```

Playwright still requires `cargo build -p nexus-worker` first
for the state roundtrip test (unchanged from Sprint 5).

---

## Checklist

| # | Check | Command | Critère | Observed |
|---|---|---|---|---|
| 1 | SDK `view` module importable | `uv run python -c "from nexus_sdk.view import TabView, section, metric"` | exit 0 | **exit 0** |
| 2 | Pydantic TabView validates schema_version=1 | `pytest packages/nexus-sdk/tests/test_view.py::test_tabview_requires_schema_version_1` | pass | **pass** |
| 3 | Pydantic rejects schema_version=2 | `pytest packages/nexus-sdk/tests/test_view.py::test_tabview_rejects_unknown_schema_version` | pass | **pass** |
| 4 | SDK snapshot stable | `pytest packages/nexus-sdk/tests/test_view.py::test_view_schema_stable_snapshot` | pass | **pass** — `packages/nexus-sdk/tests/snapshots/tabview_schema.json` frozen |
| 5 | Coordinator validates TabView | `pytest packages/nexus-coordinator/tests/test_apps.py::test_schema_driven_descriptor_validates` | pass | **pass** |
| 6 | Coordinator falls back legacy | `pytest packages/nexus-coordinator/tests/test_apps.py::test_legacy_descriptor_falls_back` | pass | **pass** |
| 7 | All SDK tests | `uv run --package nexus-sdk pytest packages/nexus-sdk/tests/ -q` | ≥ 10 passed | **31 passed** (was 6 at Sprint 5, +25 for view.py) |
| 8 | All coordinator tests | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q` | ≥ 45 | **45 passed + 1 skipped** (was 43 + 1, +2 schema / legacy fallback tests) |
| 9 | All app-gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | all pass | **3 passed** |
| 10 | All Rust tests unchanged | `cargo test --workspace --exclude nexus-core-py --locked` | ≥ 193 | **193 total** — 62 core-rs lib + 11 worker bin + 10 worker e2e + 105 worker-core lib + 5 doctests (unchanged vs Sprint 5, no Rust code touched in Sprint 6) |
| 11 | cargo fmt + clippy | `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 | **exit 0** — clean |
| 12 | ruff format + check | `uv run ruff format --check packages/ examples/ && uv run ruff check packages/ examples/` | exit 0 | **exit 0** — 57 files already formatted, 0 lint issues |
| 13 | tsc strict | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 | **exit 0** — strict + noUnusedLocals + noUnusedParameters |
| 14 | ESLint | `cd web && npm run lint` | 0 err, 5 T1 warnings | **0 errors, 5 warnings** (T1 accepted) |
| 15 | Vite build | `cd web && npm run build` | exit 0, no warnings | **exit 0** — zero warnings, 2173 modules transformed in 439ms |
| 16 | size-limit budgets | `cd web && npm run size` | all within D5 budget | **main 455 kB ≤ 475 · vendor-react 189 kB ≤ 210 · vendor-ui 31 kB ≤ 110 · css 93 kB ≤ 100** |
| 17 | Vitest unit tests run | `cd web && npm run test:unit` | all pass, <2 s tests | **77 passed** in 354 ms tests time (3 files) |
| 18 | Vitest coverage thresholds | `cd web && npm run test:coverage` | lines ≥90%, funcs ≥90%, branches ≥85% | **97.34% lines · 98.18% funcs · 88.67% branches · 97.59% stmts** — all above thresholds |
| 19 | TabView renderer covers 11 kinds | (inside `TabViewRenderer.test.tsx`) | 11 tests pass | **22 cases pass** — 11 kinds + recursive section + empty section + parseTabView edges |
| 20 | Command palette Ctrl+K | Playwright `command-palette.spec.ts` | pass | **pass** (~1.2 s) — trigger button + Ctrl+K dispatch both reach the dialog |
| 21 | TabView schema-driven e2e | Playwright `tabview-schema-driven.spec.ts` | pass | **pass** (~13 s) — gov Contradictions renders heading + 2 metrics + empty block, no legacy JSON fallback visible |
| 22 | All Playwright | `cd web && npx playwright test` | **≥ 10 passed** | **10 passed in ~10 s** — onboarding, add-coord, my-projects, project-detail, apps-tab-render (ported), my-network, /browse stub, /curators stub, command-palette, tabview-schema-driven |
| 23 | French-only | `bash web/scripts/scan-en-strings.sh` | exit 0 | **clean** — "src/ is French-only, clean" |
| 24 | PATTERNS.md P8 + T2/T3 closed | `grep -q "P8 — TabView" docs/shell/PATTERNS.md && grep -q "CLOSED Sprint 6" docs/shell/PATTERNS.md` | exit 0 | **exit 0** — P8 added, T2 + T3 annotated with commit SHA |

---

## Summary

All 24 fail-fast rows pass. Sprint 6 delivers:

- **Day 0** (1 commit): `docs(sprint6): kickoff + detailed plan`
  (1127 lines total — kickoff splits the 8 Sprint-5-deferred
  items across Sprint 6/7/8, freezes D1..D5 Day-0 decisions,
  documents the 19-tab gov migration research).
- **Phase A — SDK + coordinator** (1 commit): new
  `packages/nexus-sdk/src/nexus_sdk/view.py` (TabView Pydantic
  discriminated-union schema v1 with 11 block kinds + 13
  constructor helpers, frozen + extra=forbid). Re-exports
  from `nexus_sdk.__init__`. Snapshot guard
  `tests/snapshots/tabview_schema.json`. 25 new tests in
  `test_view.py`. Coordinator
  `api/apps.py::GET /app/{name}/tabs/{tab_name}/descriptor`
  wires `_coerce_tab_view()` that runs the descriptor through
  `TabView.model_validate` and returns
  `{descriptor, legacy_descriptor}`; ValidationError triggers
  the legacy fallback with a WARNING log. Two new pytest cases
  (`test_schema_driven_descriptor_validates`,
  `test_legacy_descriptor_falls_back`). Hello-world-app and
  nexus-app-gov Contradictions tab ported as first reference
  apps.
- **Phase B — web renderer** (1 commit):
  `web/src/components/app/tabview/schema.ts` — Zod mirror via
  `z.object().strict()` per block kind, recursive section via
  `z.lazy` + 3-param `ZodType<Output, Def, unknown>` to keep
  the input side permissive without losing runtime validation.
  `TabViewRenderer.tsx` + `TabBlockRenderer.tsx` (exhaustive
  switch with TS never-branch). 11 per-block components under
  `blocks/` — charts are SVG-inline (chart_line ~80 LOC,
  chart_bar ~80 LOC) so zero legacy chart lib returns.
  Zod `TabViewSchema` wired into
  `api/coordinator.ts::getAppTabDescriptor()` which returns a
  discriminated `{schema | legacy | error}` result.
  `AppsTab.tsx` rewritten: « Invoquer » button calls the typed
  helper and delegates to `<TabViewRenderer>` on success, or a
  collapsible `<details>` raw-JSON fallback on legacy reason.
  `apps-tab-render.spec.ts` updated to assert the ported
  Hello tab renders the heading block "Bienvenue sur
  hello-world-app" instead of raw JSON.
- **Phase C — Ctrl+K palette** (1 commit): 
  `useCommandPalette.ts` hook (global `keydown` listener,
  `Ctrl+K` + `Cmd+K`) + `CommandPalette.tsx` (three groups:
  Navigation, Projets from Zustand store, Actions). Root
  fix: the shadcn vendored `CommandDialog` template omits the
  cmdk `<Command>` primitive so children crash with
  "Cannot read subscribe of undefined" — the palette wraps
  children in `<Command>` locally to keep
  `components/ui/command.tsx` regen-safe per T1 policy.
  `AppShell.tsx` mounts the palette at root (hors `<Outlet>`)
  with an OS-aware ⌘K/Ctrl K header trigger. Playwright spec
  `command-palette.spec.ts` exercises the trigger button +
  narrow-by-typing + Enter navigation + Escape close, plus
  a synthetic `KeyboardEvent` dispatch to verify the global
  keydown listener.
- **Phase D — Vitest + size-limit** (1 commit):
  `vitest.config.ts` (standalone, jsdom, @ alias, coverage v8
  scoped to format + projectStore + tabview/**, thresholds
  90/90/85/90). `src/test/setup.ts` loads jest-dom matchers
  and stubs `matchMedia` for jsdom. 31 cases for `format.ts`
  (null/undefined/boundary + relative-time past/future/instant
  branches). 24 cases for `projectStore.ts` (add/remove/
  setActive/update/clear, dedupe, persist writes to
  `nexus-grid:shell:v1`, `selectActiveCoordinator` stale
  branch). 22 cases for `TabViewRenderer.test.tsx` (every
  kind + recursive section + Zod schema edge cases). Four
  raw-byte size-limit budgets in `web/.size-limit.json` (via
  `brotli: false, gzip: false`). New scripts `test:unit`,
  `test:coverage`, `size`. ESLint extended with Vitest globals
  for `**/*.{test,spec}.{ts,tsx}` so `describe/it/expect/vi`
  resolve cleanly.
- **Phase E — polish, Playwright, verification** (this commit):
  Playwright `tabview-schema-driven.spec.ts` expands gov and
  asserts the Contradictions tab renders the ported heading +
  two metrics + empty block with no legacy fallback visible —
  a live end-to-end proof that Python TabView → coordinator
  validation → Zod parse → React render works in one round
  trip. `docs/shell/PATTERNS.md` adds **P8 — TabView is the
  only contract for app-provided tabs** and marks **T2** +
  **T3** CLOSED with the Phase D commit SHA. This verification
  document replaces the Sprint 5 one as the latest
  authoritative fail-fast checkpoint.

## What's NOT in this sprint (scope line)

Explicitly deferred per `.planning/sprint6_kickoff.md` §3 and
`.planning/sprint6_plan.md` §11:

- **nexus-shell-daemon** — sidecar crate with iroh Node for
  DHT / curator gossip. **Sprint 7**.
- **Curator list flow (Ed25519 sign + gossip + blobs)** —
  `DOMAIN_CURATOR_LIST_V1` not yet added to
  `crates/nexus-core-rs/src/canonical.rs`. **Sprint 7**.
- **DHT browse via pkarr** — `Browse.tsx` still a Sprint 6
  stub. iroh 0.97 already exposes this via `discovery_n0()`.
  **Sprint 7**.
- **Full 19-tab `nexus-app-gov` migration (v1.1)** — only the
  minimal Contradictions tab ported to TabView in Phase A as
  a reference implementation. **Sprint 8**.
- **SDK extensions** (`AppContext.storage`, `AppContext.events`,
  file upload helper, DB migration runner) needed for the gov
  migration. **Sprint 8 Phase A**.
- **Five native tabs ported to TabView** — D3 figé, they stay
  hard-coded.
- **Charts beyond `chart_line` / `chart_bar`** — no D3 / recharts
  reintroduction. If Sprint 8 needs more, bump schema to v2.
- **`@rjsf/shadcn` form rendering** — D1 rejected in favour of
  the custom vocabulary. Reconsider if Sprint 8 needs forms.
- **Mobile responsive < 1280px** — confirmed desktop-only.
- **Worker HTTP API** — still rejected (Sprint 5 D3 stands).

## Git summary

```
$ git log --oneline master ^cdf4467
<tip>    feat(web): Sprint 6 Phase E — polish, Playwright, verification
7a56828  feat(web): Sprint 6 Phase D — Vitest unit tests + size-limit CI
3d87ec3  feat(web): Sprint 6 Phase C — Ctrl+K command palette
c49e8ca  feat(web): Sprint 6 Phase B — TabView renderer + AppsTab schema-driven descriptor
667ae6b  feat(sdk,coordinator,app-gov,examples): Sprint 6 Phase A — TabView schema + coordinator validation
02ab9bf  docs(sprint6): kickoff + detailed plan
```

6 commits on top of `cdf4467` (Sprint 5 tip). Sprint 6 closed.

**Next sprint**: Sprint 7 — P2P Discovery Layer
(nexus-shell-daemon + curator list Ed25519 + DHT browse via
pkarr + Browse/Curators pages wired). Outline in
`.planning/sprint6_kickoff.md` §3.
