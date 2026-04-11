# nexus-grid shell — patterns and tech debt

Mirror of `docs/rust/PATTERNS.md` for the React shell landed in
Sprint 5. Any non-trivial decision that is not captured by the
code or the git history lives here. Keep entries dated and
reference the commit SHA that introduced them so future readers
can `git show <sha>` for full context.

## Patterns

### P1 — Typed coordinator client is the only allowed fetch path

**Rule**: no component, hook or test is allowed to call `fetch`
directly. Every request against a `nexus-coordinator` goes
through a helper in `web/src/api/coordinator.ts`. The helpers
`safeParse` the response against a Zod schema and throw a
typed `CoordinatorProtocolError` or `CoordinatorHttpError` on
mismatch. This guarantees:

- every response shape has one source of truth (the Zod schema);
- component types are `z.infer<>`red from the schemas so a
  backend schema drift triggers a TS failure;
- manual JSON parsing (`as any`, `!` assertions on untyped
  objects) cannot creep in.

The single narrow exception is `AppsTab.tsx`'s "Invoquer"
button, which hits `/app/{name}/tabs/{tab}/descriptor` with a
plain `fetch` because the descriptor shape is app-defined and
Zod would require a discriminated union per app. This is
documented inline on the call site.

Reference: commit for the Phase A frontend shell.

### P2 — base-ui `render` prop, not Radix `asChild`

shadcn v4 in this repo is built on `@base-ui/react`. base-ui
does not support the `asChild` pattern — the equivalent is a
`render` prop that takes a ReactElement to substitute for the
default button / trigger / link. Example:

```tsx
<DropdownMenuTrigger
  render={<Button size="sm" variant="ghost" />}
>
  Label
</DropdownMenuTrigger>
```

Trying to pass `asChild={true}` to anything in `components/ui/`
will not compile. If you see it in a copy-pasted shadcn snippet
from an older Radix-based repo, translate it to `render`.

### P3 — Zustand 5 curried create syntax

Every store that uses middleware (persist, devtools) must use
the curried form:

```ts
create<MyState>()(persist((set) => ({ ... }), { name: "..." }));
```

The non-curried `create<MyState>(...)` form drops the middleware
typing. TypeScript will not always flag the omission — look at
`src/stores/projectStore.ts` for the canonical pattern.

### P4 — React Query is the only cache

No manual `useEffect(() => fetch(...).then(setState))` patterns.
Every server interaction goes through `useQuery` /
`useMutation`. Polling cadences (2 s for `/worker-state`, 5 s
for `/health`, etc.) are configured per-query via
`refetchInterval`, not via ad-hoc `setInterval`.

### P5 — CORS allowed on loopback origins only

The coordinator mounts `CORSMiddleware` with
`allow_origin_regex=r"^https?://(127\.0\.0\.1|localhost)(:\d+)?$"`.
This is needed so the vite dev server (5173) can hit the
coordinator (8765) during development and in Playwright tests.
**Do not** relax this to `allow_origin_regex=".*"` or add a
wildcard — the coordinator is loopback-only by default and the
tight allow-list keeps an exposed coordinator from being
trivially hit by a malicious site the user happens to visit.

Reference: `packages/nexus-coordinator/src/nexus_coordinator/api/app.py`.

### P6 — NEXUS_GRID_ROOT env override for tests

Both the Python coordinator (`paths.py`) and the Rust worker
core (`paths.rs`) honour `NEXUS_GRID_ROOT` as a filesystem root
override. Integration tests (pytest, Playwright globalSetup,
future e2e harnesses) point this at a throwaway directory so
the real `~/.nexus-grid/` on the developer's machine is never
touched.

Setting `NEXUS_GRID_ROOT=` (empty) is equivalent to unset on
both sides.

### P7 — File-based shell ↔ worker integration

The Sprint 5 D3 decision is that the worker writes a JSON
`state.json` snapshot every `state_flush_secs` seconds and the
coordinator proxies it to the shell. The file path is the
single source of truth; both sides resolve it via their own
`paths.worker_state_file` / `paths.worker_state_path()` helper
and must agree byte-for-byte. The Rust schema lives in
`crates/nexus-worker-core/src/engine/state_writer.rs`; the
Python mirror lives in
`packages/nexus-coordinator/src/nexus_coordinator/api/worker_state.py`.
Any field rename on one side without the other is a breaking
change — bump `schema_version` in both.

Reference: sprint5_plan.md §2.3.

### P8 — TabView is the only contract for app-provided tabs

Every `@nexus_tab`-decorated method on a `NexusApp` MUST
return a `TabView` (schema_version=1) built via the
`nexus_sdk.view` constructor helpers. The coordinator runs
the return value through `TabView.model_validate` in
`packages/nexus-coordinator/src/nexus_coordinator/api/apps.py`
and the React shell parses the payload through the Zod
mirror in `web/src/components/app/tabview/schema.ts` before
the renderer walks the block tree.

**Sprint 8 D4 retired the `legacy_descriptor` fallback**
(commit `d321021`). The Sprint 6 D3 transition aid that
preserved an unported app's raw dict under
`{descriptor, legacy_descriptor: true}` is **gone**: a tab
that returns a payload that does not pass
`TabView.model_validate` now fails the request with
`HTTPException(422, detail=<ValidationError>)`. The
`_coerce_tab_view()` helper has been deleted from
`apps.py`; the route handler calls `model_validate`
directly. Test contract:
`tests/test_apps.py::test_tab_descriptor_raises_422_on_invalid_schema`
asserts the new behaviour, plus two
`assert "legacy_descriptor" not in body` checks lock the
field absence on every successful tab descriptor response.

The five native tabs in `ProjectDetail.tsx` (Overview,
Tasks, Kudos, Invites, Apps) stay out of scope: they
consume coordinator-native APIs directly, not app
descriptors.

Cross-language schema stability: the Pydantic source of
truth is `packages/nexus-sdk/src/nexus_sdk/view.py` and
`packages/nexus-sdk/tests/snapshots/tabview_schema.json`
is the frozen checkpoint. Any bump to `schema_version`
must land in one commit that touches both the Pydantic
model and the Zod schema at once, and must regenerate the
snapshot explicitly.

Reference: sprint6_plan.md §2 D1/D2/D3.

### P9 — nexus-shell-daemon reached exclusively through the coordinator proxy

Sprint 7 D1 (frozen in `sprint7_kickoff.md` §4): the React shell
NEVER fetches the `nexus-shell-daemon` HTTP surface directly.
Every `/browse`, `/curators`, `/info` call goes

```
shell → coordinator /daemon/* → nexus-shell-daemon 127.0.0.1:<ephemeral>
```

Rationale (four independent reasons that reinforce each other):

1. **Single trust boundary.** The coordinator already runs a
   CORS regex that only accepts `http://(127.0.0.1|localhost):*`
   origins. Routing daemon calls through the same process means
   we never have to double-maintain that policy on the daemon
   side. The daemon DOES ship its own loopback CORS layer
   (`crates/nexus-shell-daemon/src/http.rs::loopback_cors_layer`)
   as defense-in-depth, but the shell never triggers it.

2. **Ephemeral port.** The daemon binds `127.0.0.1:0` and writes
   the resolved port into `<root>/shell-daemon/running.json`. If
   the shell talked to the daemon directly it would need to read
   that file too — duplicating the singleton discovery across two
   codebases. The coordinator does it once and forwards.

3. **Daemon-offline UX.** When no daemon is running the coordinator
   returns `{"kind": "unavailable", "reason": "..."}` at 503. The
   shell's `DaemonResult<T>` union surfaces that as a first-class
   render path (`DaemonOfflineBanner`), NOT as an error boundary
   trip. Direct shell-to-daemon would have to re-invent this
   envelope in TypeScript.

4. **Proxy-side input validation.** `api/daemon.py::daemon_subscribe_curator`
   rejects non-object POST bodies at 400 before even forwarding,
   so a whole class of shell bugs lands in the test suite instead
   of on the daemon logs.

Contract shape — the discriminated envelope every `/daemon/*`
route returns, mirrored in `web/src/api/daemon.ts::DaemonResult<T>`:

```
{"kind": "data",        "status": int, "body": <daemon body>}    (200)
{"kind": "unavailable", "reason": "<transport / not-running>"}   (503)
{"kind": "error",       "reason": "<proxy-level 400>"}           (400)
```

The `status` field carries the upstream HTTP status code (200,
422, 500, …) so the shell can distinguish "daemon said no" from
"daemon offline". The reference test suite is
`packages/nexus-coordinator/tests/test_daemon_proxy.py` (10 tests,
including the ones that swap the fake daemon for a closed
ephemeral port to force a `httpx.ConnectError → 503`).

Reference: sprint7_kickoff.md §4 D1, sprint7_plan.md §8.

### P10 — Command palette extends with app-contributed entries via `@nexus_command`

Sprint 8 D2/D5 (frozen Sprint 7 D5, implemented commit
`d321021`): the React shell command palette
(`web/src/components/command-palette/CommandPalette.tsx`)
exposes a 4th group « Apps » that merges entries declared
by every enrolled `NexusApp` via the `@nexus_command`
decorator. The first three groups (Navigation / Projets /
Actions) stay hardcoded shell-side; the 4th is fully
data-driven from the coordinator.

Contract shape — the SDK side:

```python
# packages/nexus-sdk/src/nexus_sdk/decorators.py
def nexus_command(
    name: str,
    *,
    description: str,
    icon: str = "sparkles",
    group: str = "Actions",
) -> Callable[[F], F]:
    """Attach a CommandDescriptor to a NexusApp method."""
```

The decorator stamps a `CommandDescriptor` (frozen
Pydantic, `extra="forbid"`, `schema_version=1`) on
`__nexus_command__` of the wrapped method.
`NexusApp.commands()` walks the class's methods and returns
the descriptors sorted by `name` ascending so the route
ordering is deterministic and the test
`test_list_app_commands_ordered` is stable.

Contract shape — the coordinator routes:

```
GET  /app/{name}/commands                     → list[CommandDescriptor]
POST /app/{name}/commands/{cmd}/invoke        → command return value
```

The shell consumes both via `web/src/api/coordinator.ts`
(`listAppCommands()`, `invokeAppCommand()`), with the Zod
mirror `CommandDescriptorSchema` `.strict()` matching the
Pydantic source-of-truth field-for-field. Polling cadence
in `CommandPalette.tsx`: React Query
`staleTime: 15_000`, `refetchInterval: 30_000` per the
Sprint 8 R7 mitigation (the palette must not hammer the
coordinator with N-app-many concurrent fetches every render).

Click handling: a command can either deep-link into a tab
(when its metadata declares a `target_tab` and
`extractNavigationPath()` resolves it to
`/app/{appName}/tab/{tabName}`) or fire a server-side
`invoke_command` via the POST route. The
`AppTabPage.tsx` route exists to receive the deep-link
target without going through `ProjectDetail`'s tab strip.

Loopback trust applies: `/commands/.../invoke` carries no
auth header — same trust model as `/tasks/submit` Sprint 4.
Adding auth here would mean adding auth to the whole
coordinator surface, which is a Sprint 10+ release-prep
task, not a Sprint 8 hygiene fix.

Reference: sprint8_kickoff.md §4 D2, sprint8_plan.md §4
Phase A, commit `d321021`.

### P11 — `AppContext.db` is a read-only async wrapper, scope-cut from a writer

Sprint 8 D3 (commit `d321021`): `AppContext.db` exposes a
`AppDatabaseClient` that is **read-only by enforcement**,
not by convention. Open path:

```python
# packages/nexus-sdk/src/nexus_sdk/db.py
async def _connect(self) -> aiosqlite.Connection:
    return await aiosqlite.connect(
        f"file:{self.path}?mode=ro",
        uri=True,
    )
```

The SQLite URI `mode=ro` makes the connection refuse any
INSERT / UPDATE / DELETE / DDL at the engine level — a
write attempt raises `OperationalError: attempt to write
a readonly database`, not silently lost. Test contract
`test_db.py::test_readonly_enforced` asserts this.

Concurrency: an internal `asyncio.Lock` serialises
`fetchall` / `fetchone` calls per `AppDatabaseClient`
instance to dodge aiosqlite's connection-not-thread-safe
gotcha (Sprint 8 R9 mitigation). Test
`test_concurrent_fetchall` covers it. Sprint 9 may bump
to a per-tab connection pool if the lock contention bites
gov tabs that issue 5+ queries on render.

Path resolution: the coordinator computes the legacy DB
path absolutely from `__file__` via
`packages/nexus-coordinator/src/nexus_coordinator/paths.py::nexus_grid_repo_root()`,
NEVER from a user-supplied string. An app cannot ask for
an arbitrary file. The `NEXUS_GRID_ROOT` env override
exists for tests and respects the same anchor.

Read-only is a deliberate scope choice for Sprint 8: the
gov v1.1 migration only reads the legacy `nexus/gov/govdata.db`
(4 years of scraped data, treated as immutable input). The
4 deferred infra primitives (`AppContext.storage`, `events`,
file upload, migration runner) are the writer-side surface
that Sprint 9 will introduce when there is a real consumer.

Reference: sprint8_kickoff.md §4 D3, sprint8_plan.md §5
Phase B, commit `6efda53`.

## Tech debt — queued for Phase D or later

### T1 — Fast refresh warnings on 5 shadcn ui primitives

`web/src/components/ui/{badge,button,sidebar,tabs,toggle}.tsx`
each export a non-component constant (`buttonVariants` etc.)
or a hook (`useSidebar`) alongside the component. The eslint
rule `react-refresh/only-export-components` warns on these
with `allowConstantExport: true` still enabled. They are
shadcn's canonical shape, not ours, and splitting them out
would break future `npx shadcn add` upgrades.

Fix options if we want to silence the warnings:
- (a) move every variant / hook to a sibling `*-variants.ts` /
  `*-context.ts` file (fragile against shadcn CLI upgrades),
- (b) add a file-scoped eslint-disable comment on each line
  (cleaner than a global override),
- (c) accept the warning level permanently and document why.

Current decision: accept the warning level (5 warnings, no
errors), revisit if the shadcn team changes their convention.

### T2 — Bundle size not tracked in CI — CLOSED Sprint 6 Phase D

`web/.size-limit.json` enforces four raw-byte budgets on every
`npm run size`: main ≤ 475 KB, vendor-react ≤ 210 KB,
vendor-ui ≤ 110 KB, css ≤ 100 KB. The check is wired to the
`size` npm script and runs right after `vite build`. CI
integration: invoke `npm run build && npm run size` — a
single non-zero exit fails the Sprint 6 checklist row 16.

Closed by commit `7a56828` (Sprint 6 Phase D).

### T3 — No Vitest unit tests for format helpers — CLOSED Sprint 6 Phase D

Vitest 4.1 + @testing-library/react + jsdom added to devDeps.
`web/vitest.config.ts` restricts the coverage scope to
`src/lib/format.ts`, `src/stores/projectStore.ts`, and
`src/components/app/tabview/**`. Actual coverage after Phase D:
97.34% lines / 98.18% functions / 88.67% branches / 97.59%
statements across 77 unit tests in 3 files.

Closed by commit `7a56828` (Sprint 6 Phase D).

### T4 — TabView.button task_submit action needs a real consumer — CLOSED Sprint 8 Phase A

Sprint 6 audit finding G-1 / B-1. The `action.kind === "task_submit"`
branch in `web/src/components/app/tabview/blocks/ButtonBlock.tsx` is
a `console.warn` placeholder. The schema exports the action type,
`nexus_sdk.view.button_task()` constructs it, but nothing runs the
button click. Sprint 6 audit coverage shows ButtonBlock at 57% lines
/ 0% branches because no test ever hits the second branch.

**Status: CLOSED Sprint 8 Phase A** (commit `d321021`).

`ButtonBlock.tsx` now reads a `TabAppContext` provider
(`web/src/components/app/tabview/TabAppContext.tsx`) that
carries `{coordinatorUrl, projectName, appName}` from the
parent `AppsTab` / `AppTabPage`, builds the body
`{worker, payload, priority?, parent_task_id?}`, and POSTs
to `/app/{appName}/tasks/submit` via
`web/src/api/coordinator.ts::submitAppTask`. Success / error
states are surfaced inline in the button row (no toast
infrastructure introduced — kept consistent with Sprint 6
inline form errors).

Coverage status post-Sprint 8: `ButtonBlock.tsx` at 77.77%
lines / 76.47% branches (Vitest) plus the Playwright spec
`gov-rag-search.spec.ts` exercises the button against a
real coordinator. The remaining uncovered branches are
HTTP 500 fall-through paths exercised through coverage of
`coordinator.ts::submitAppTask` itself.

Reference: `.planning/sprint8_kickoff.md` §D1,
`.planning/sprint8_plan.md` §4 Phase A, commit `d321021`.
Audit history: `.planning/sprint6_audit_findings.md` §G-1,
`sprint7_kickoff.md` §D4 (signature freeze).

### T5 — No SDK hook for app-contributed command palette entries — CLOSED Sprint 8 Phase A

Sprint 6 audit finding G-2. The command palette hardcodes three
groups (Navigation, Projets, Actions) with no mechanism for an app
to register commands ("Nouveau fact-check", "Rechercher dans les
votes"). Sprint 8 gov v1.1 will want this. The SDK currently has
`NexusApp.routes()`, `.workers()`, `.tabs()` but no `.commands()`.

**Status: CLOSED Sprint 8 Phase A** (commit `d321021`).

The full surface lives now in
`packages/nexus-sdk/src/nexus_sdk/commands.py` (the
`CommandDescriptor` Pydantic frozen model), `decorators.py`
(the `@nexus_command` decorator that stamps
`__nexus_command__` metadata), and `registry.py`
(`NexusApp.commands()` collector). The coordinator routes
`GET /app/{name}/commands` and
`POST /app/{name}/commands/{cmd}/invoke` are wired in
`packages/nexus-coordinator/src/nexus_coordinator/api/apps.py`,
the React shell consumes them through
`web/src/api/coordinator.ts::{listAppCommands, invokeAppCommand}`,
and the 4th palette group lives in
`web/src/components/command-palette/CommandPalette.tsx`.
See **P10** above for the full pattern reference.

Audit history: `.planning/sprint6_audit_findings.md` §G-2,
`sprint7_kickoff.md` §D5 (signature freeze).

### T6 — Renderer fuzz + chart edge-case tests

Sprint 6 audit findings A-2 + B-2. The Vitest TabViewRenderer suite
covers each of the 11 block kinds on happy-path inputs (2-row tables,
3-point charts, ASCII labels). It does not exercise:

- Tables with 100+ rows (DOM perf)
- Unicode / RTL labels
- Charts with 1 point, 0 points (covered), 2 identical points
- Charts with all-negative values (yMin === yMax → SVG path would
  divide by zero and emit `M NaN,NaN`)
- Metric with delta = 0 (neutral delta rendering)
- Section recursion depth > 2
- Table rows with a column key not in the column list
- Very long metric value strings (layout overflow)

Not a runtime bug today — the gov Contradictions port uses plain
ASCII and small shapes — but Sprint 8 19-tab migration will have
much larger real-world payloads and at least chart edge cases are
likely to surface. Add a `renderer_fuzz.test.tsx` with ~10 edge
fixtures before Sprint 8 Phase C (when chart-heavy tabs land).

Audit reference: `.planning/sprint6_audit_findings.md` §A-2, §B-2.

### T7 — Playwright anchors + caplog assertion

Sprint 6 audit findings C-3 + D-2. The Playwright specs use pure
`getByText` assertions — if a regression rendered content inside a
legacy `<pre>JSON.stringify</pre>` fallback, the text would still
match. Add `data-testid="tabview-renderer"` on the renderer root
and assert on structural locators (`h1`, `h2`, `[data-testid]`) in
`tabview-schema-driven.spec.ts`. Also add a caplog assertion to
`test_legacy_descriptor_falls_back` so a refactor that accidentally
downgrades the WARNING log level to DEBUG fails the test.

Both are three-line tightenings with high return on reliability.
Recommended for Sprint 7 cleanup phase.

Audit reference: `.planning/sprint6_audit_findings.md` §C-3, §D-2.

### T8 — `CardTitle` is a `<div>`, not an `<h2>`/`<h3>` (a11y) — Sprint 9

Sprint 7 audit finding F-3. shadcn vendored
`web/src/components/ui/card.tsx` ships `CardTitle` as a
styled `<div>`, not a heading element. The Browse / Curators /
gov tab pages use `CardTitle` for every card header, so a
screen reader sees no `<h2>` / `<h3>` hierarchy beneath the
top-of-page `<h1>` from `PageHeader`. Icon-only buttons
(`<BookmarkPlus>`, `<Trash2>`) also lack `aria-label`s.

Fix options:
- (a) edit `card.tsx` once to render `<h3>` instead of `<div>`
  (vendored file, audit-friendly diff)
- (b) accept an `as` prop on `CardTitle` (intrusive vs
  upstream shadcn shape)
- (c) add file-level eslint a11y rules + fixup pass

Sprint 8 Phase A intentionally **did not** touch this — it
was scope-cut to Sprint 9 polish. Track G2 of
`.planning/sprint8_audit_plan.md` cross-checks the deferral.

Audit reference: `.planning/sprint7_audit_findings.md` §F-3.

### T9 — Coordinator `httpx.AsyncClient` per-call, no `Limits` — Sprint 9

Sprint 7 audit finding G-1.
`packages/nexus-coordinator/src/nexus_coordinator/api/daemon.py::_forward`
opens `async with httpx.AsyncClient(timeout=timeout)` on
**every** call. For loopback this is trivially fast (~0 ms
per handshake) but the absence of
`httpx.Limits(max_connections=...)` means a burst of F5
refreshes from the shell can accumulate clients without
bound. No vulnerability today, but worth fixing before
release.

Fix options:
- (a) module-level singleton
  `httpx.AsyncClient(limits=httpx.Limits(max_connections=10))`
  shared across requests, lifecycle managed by the FastAPI
  app `lifespan`
- (b) keep client-per-call but add explicit
  `limits=httpx.Limits(max_connections=10)`

Sprint 8 Phase A intentionally **did not** touch this. Track
G2 of `sprint8_audit_plan.md` cross-checks the deferral.

Audit reference: `.planning/sprint7_audit_findings.md` §G-1.

### T10 — Main bundle 0.5 KB headroom under size-limit budget — Sprint 9

Sprint 8 Phase E observed `main 474.49 kB / 475 budget` =
**0.5 KB headroom**. The Sprint 8 surface added
`AppTabPage.tsx`, `TabAppContext.tsx`,
`extractNavigationPath.ts`, the rewritten `CommandPalette.tsx`
4th group, the `submitAppTask` / `listAppCommands` /
`invokeAppCommand` clients, and the rewritten `coordinator.ts`
schemas. Net add ~3 KB.

The next React component or coordinator route added Sprint 9
**will** fail size-limit row 24 unless one of:
- (a) bump the budget to 500-525 KB in `web/.size-limit.json`
  with a documented rationale (commit body)
- (b) tree-shake the unused `lucide-react` icon imports (a
  Vitest spot-check shows ~30 icons imported, ~10 actually
  rendered)
- (c) split a heavy component out of the main chunk via
  `React.lazy` + Suspense (e.g. `AppTabPage` → its own chunk)

Track H3 of `sprint8_audit_plan.md` lists this as **P1**
because it gates Sprint 9 commits. Sprint 9 Day 0 should
pick a fix path before any new feature commit lands.

Audit reference: `.planning/sprint8_verification.md` §Notes
row 24.
