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

### P12 — Code splitting via `createBrowserRouter` lazy + manualChunks

Sprint 9 Phase A (D6). The shell router migrated from the
declarative `<BrowserRouter>` + `<Routes>` form to the
`createBrowserRouter([...])` data router so that each page
can be loaded through `lazy: () => import("@/pages/Page")`.
React Router v7 keeps the lazy API introduced with v6.4: a
lazy-loaded module exposes a named `Component` (and
optionally `loader` / `action`) export which the router
awaits on first navigation. Each page file therefore ends
with:

```tsx
export default function Browse() { /* ... */ }
// Sprint 9 Phase A (D6) — react-router lazy() Component export.
export const Component = Browse;
```

The `default` export is kept so `import Browse from "@/pages/Browse"`
still works from non-router callers (for example `Projects.tsx`
renders `<OnboardingEmpty />` inline when the store is empty).

**Vite `manualChunks` guards**. Rolldown conservatively hoists
a user-land module into the main chunk whenever it is named by
`manualChunks` while also being referenced from an implicit
lazy route chunk. The Sprint 9 refactor adopts the canonical
2025 pattern (lobe-chat, react.dev sunsetting-CRA guide):

```ts
manualChunks(id) {
  // src-side feature chunks (evaluated first)
  if (id.includes('/src/components/app/tabview/')) return 'tabview';
  if (id.includes('/src/components/command-palette/')) return 'palette';

  // Only split vendor code past this point.
  if (!id.includes('node_modules')) return;

  if (id.includes('node_modules/react/') || ...) return 'vendor-react';
  if (id.includes('node_modules/@tanstack/') || ...) return 'vendor-query';
  if (id.includes('node_modules/@radix-ui/')) return 'vendor-ui';
}
```

The `if (!id.includes('node_modules')) return` guard is the
critical line — without it, `manualChunks` returning `undefined`
for a src-side module is fine (rolldown picks the per-page
chunk), but as soon as a named chunk collides with a lazy
route chunk, the lazy import evaporates.

Sprint 5 left three dead chunks in `vite.config.ts` that
referenced packages the legacy-UI removal had already dropped
(`vendor-graph`, `vendor-charts`, `vendor-map`). Sprint 9
Phase A deletes them.

**size-limit budgets**. The 4 Sprint 6 budgets (main, vendor-
react, vendor-ui, css) become 7 Sprint 9 budgets:

| Chunk          | Budget | Rationale                                         |
|----------------|--------|---------------------------------------------------|
| `main`         | 350 KB | Shell chrome + providers + lazy-route bootstrap   |
| `vendor-react` | 170 KB | react + react-dom + react-router + scheduler      |
| `vendor-query` | 50 KB  | @tanstack/react-query + zustand                   |
| `vendor-ui`    | 40 KB  | @radix-ui/* vendored primitives                   |
| `tabview`      | 80 KB  | app/tabview/** renderer + 11 block components     |
| `palette`      | 40 KB  | command-palette/**                                |
| `css`          | 100 KB | tailwind-compiled stylesheet                      |

If the main chunk cannot land under 350 KB in one refactor
pass, the fallback is a documented `fix(sprint9): relax main
budget to 425 KB pending tree-shake pass` commit with a
`rollup-plugin-visualizer` screenshot in the body. **Never
above 425 KB**. Sprint 9 Phase B will tree-shake `lucide-react`
icon imports as a cheap follow-up lever.

A 8th `upload` budget joins the set in Sprint 9 Phase E when
the file upload renderer + CAS lands — not before.

Bundle visualization is opt-in via `ANALYZE_MODE=true npm run
build`, which activates `rollup-plugin-visualizer` and emits
`dist/stats.html`.

Reference: sprint9_kickoff.md §4 D6, sprint9_plan.md §4 Phase
A, commit `<SHA>`.

### P13 — `AppContext.storage` is per-app per-project JSON KV with typed namespaces

Sprint 9 Phase B (D1). Every app gets a per-instance
:class:`nexus_sdk.AppStorage` wired by the coordinator loader
on :attr:`nexus_sdk.AppContext.storage` BEFORE the app's
``on_start`` hook runs. The store persists a flat
``str -> JSON`` map at
``<projects_root>/<project>/apps/<app>/storage.json`` with
this on-disk shape:

```json
{"schema": 1, "payload": {"filters.politicians": {"chamber": "AN"}}}
```

The mirror of P11 (``AppContext.db``): P11 is the read path
on a precious external SQLite, P13 is the writable per-app
state surface for soft UI / filter / preference data that
the app owns end-to-end.

**Atomic rename via tmpfile + os.replace.** Writes go through
a sibling tmpfile created with ``tempfile.mkstemp(dir=parent)``
so it lives on the same filesystem (cross-device
``os.replace`` is unsupported on Linux), then swapped with
``os.replace``. The pattern is canonical across diskcache,
TinyDB and pydantic-settings — a partial write can never
leave the storage file in a half-updated state, even on a
crash.

**Write coalescing via ``asyncio.call_later(0.5, flush)``.**
Every mutation marks the in-memory state dirty and reschedules
a single deferred flush 500 ms later. A burst of N writes
collapses into one on-disk write, mirroring TinyDB's
``CachingMiddleware``. The delay is configurable via the
``flush_delay_seconds`` constructor kwarg for tests that need
to drive the timer deterministically (the SDK suite mocks
``_schedule_flush`` per-instance and ``os.replace`` to count
flush invocations).

**Per-instance ``asyncio.Lock``.** All mutators and accessors
hold a single per-storage lock so the in-memory dict is never
observed in a half-updated shape from a concurrent task. The
lock is released between mutations so a tab handler that
issues two sequential ``set`` calls in the same task does not
deadlock — the lock is single-acquire-per-call, not reentrant.

**Lifespan flush via :meth:`AppStorage.flush_on_shutdown`.**
The coordinator's ``stop()`` drains every app's storage by
awaiting the synchronous flush, which cancels the outstanding
timer and writes pending state under the lock. The
``test_lifespan_flushes_app_storage`` test pins this contract:
mutate during the live coordinator, stop, then verify the
on-disk file matches the in-memory state. After the call the
instance is marked closed; subsequent mutations are accepted
in memory but no new timer is scheduled — the host owns the
next write decision.

**Typed namespaces for drift detection.**
:meth:`AppStorage.namespace(key, Schema)` returns a
:class:`TypedNamespace` that wraps a single key with a
Pydantic model. ``await ns.get()`` runs ``Schema.model_validate()``
on the raw value and returns a model instance — or raises
:class:`StorageSchemaError` with the key and schema name if
the stored payload no longer matches. ``await ns.set(value)``
validates the input and writes the dumped JSON form. The
drift detection is the critical Sprint 9 promise: a future
app version that opens an older storage file with an
incompatible schema sees a structured error instead of a
silently half-typed model. The ``TypedNamespace.set`` test
asserts both the model-instance and dict-payload paths flow
through ``model_validate`` so a malformed dict never reaches
disk.

**Consumer-side typed namespace registry on AppContext.**
The coordinator route ``POST /app/{name}/state/{ns_key}``
takes the ``ns_key`` and dispatches the JSON body through
the namespace the app registered on
``ctx.namespaces`` from its ``on_start`` hook. The coord
never imports the schema directly — every typed namespace
consumer is a pure app-side change (Sprint 9 nexus-app-gov
registers ``politicians_filter`` for the Politiciens tab
filter). Validation failures bubble up from
:class:`StorageSchemaError` as HTTP 422 with the underlying
``pydantic.ValidationError`` message in the detail field.

**Anti-patterns explicitly rejected.** SQLite (redundant with
P11), iroh-docs (cross-node replication is out of scope),
file locking via ``fcntl``/``msvcrt`` (the coordinator is a
strict singleton — Sprint 7 D1), pickle-backed stores like
``shelve`` and ``sqlitedict`` (non-portable, security
footgun). JSON is the only serialisation format on the
storage surface.

**Why not iroh-docs?** Storage is strictly local to the
coordinator process. The Sprint 9 use case is per-app UI
state (filters, last-selected items, feature flags) — adding
a network replication layer would buy nothing and break the
"writes return as soon as the in-memory dict is updated"
contract that handlers rely on.

Reference: sprint9_kickoff.md §4 D1, sprint9_plan.md §5 Phase
B, commit `<SHA>`.

### P14 — `AppContext.events` is a per-app in-process anyio pub/sub bus

Sprint 9 Phase C (D2). Every app gets a per-instance
:class:`nexus_sdk.AppEvents` wired by the coordinator loader on
:attr:`nexus_sdk.AppContext.events` BEFORE the app's
``on_start`` hook runs. The bus is the asynchronous mirror of
P13: P13 is the writable per-app KV, P14 is the in-process
fan-out for "something happened" events that consumers want to
react to in real time.

**Wrapper around `anyio.create_memory_object_stream`.** Each
``async with bus.subscribe(pattern)`` allocates its own
``(send_stream, receive_stream)`` pair with the bus's
``buffer_size`` (default 1024). The dispatcher iterates the
matching subscribers and pushes the envelope onto every send
stream — there is no shared queue and no clone-per-subscriber
juggling, which sidesteps the only fragile spot of anyio's
memory stream API and keeps the contract straightforward.

**Frozen `EventEnvelope`.** The envelope shape is
``{topic: str, payload: dict, timestamp: datetime, trace_id:
str}`` with ``model_config={"frozen": True, "extra": "forbid"}``.
``trace_id`` is ``uuid4().hex[:16]`` and ``timestamp`` is
``datetime.now(UTC)`` at publish time. The ``payload`` field
runs through a JSON-serializability validator at construction
so the bus refuses an envelope the SSE bridge would later fail
to ``json.dumps``.

**Fnmatch glob matching.** Subscribers pass shell-style
patterns (``politician.*``, ``*.refreshed``, ``file.upload.*``)
that the bus checks via :func:`fnmatch.fnmatch`. The ``.``
character is literal, not a delimiter, and ``**`` is NOT
treated as a recursive wildcard — producers that need
single-segment semantics use a more specific suffix.

**Sync dispatch via `send_nowait`.** :meth:`AppEvents.publish`
is ``async def`` so callers can await it like every other SDK
helper, but the fan-out loop never awaits inside the body for
the ``drop_oldest`` / ``drop_newest`` policies — every push is
a ``send_nowait`` wrapped in ``except anyio.WouldBlock``. The
``block`` policy is the only path that awaits a real ``send``
and is documented as risky (a single slow consumer stalls every
other subscriber that comes after it in the dispatch loop).

**Overflow policy enum.** Three modes:

- ``drop_oldest`` (default): drain one envelope from the
  receive side via ``receive_nowait`` then retry the
  ``send_nowait``. The drop is logged once per minute per
  subscriber via a tiny throttle helper so a slow consumer
  cannot flood the structlog stream.
- ``drop_newest``: skip the publish for that subscriber and log
  the throttled warning.
- ``block``: ``await send_stream.send(envelope)``. Documented
  as a foot-gun because a single slow consumer stalls the
  whole bus.

**Context manager subscribe.** ``async with
events.subscribe(pattern) as stream:`` registers a fresh
subscription on enter, yields the receive stream, and
unregisters / closes both halves on exit — even when the body
raises. Anti-pattern explicitly rejected: weak references to
coroutines, which the GC tears down before the coroutine has a
chance to run.

**Per-app, in-process scope.** A given :class:`AppEvents`
instance is bound to a single app. Cross-app and cross-node
fan-out are explicitly out of scope for Sprint 9 (P1 Sprint 10+).
Events do not survive a coordinator restart — there is no
replay buffer.

**Lifespan close via `aclose()`.** The coordinator's ``stop()``
closes every app's bus alongside ``AppStorage.flush_on_shutdown``
before the app's ``on_stop`` hook so any subscriber currently
iterating its receive stream sees a clean ``EndOfStream``
instead of a hung receive.

**SSE bridge to the React shell.** A new
:func:`nexus_coordinator.api.events.render_sse_stream`
helper wraps the per-app bus inside an
``async with bus.subscribe(pattern):`` and yields ``data:
<json>\n\n`` envelopes plus ``: ping\n\n`` heartbeats every
30 s. The route handler ``GET /app/{name}/events?pattern=…``
returns a :class:`fastapi.responses.StreamingResponse` over
this generator with ``content-type: text/event-stream``. The
shell's :func:`useAppEvents` hook opens an :class:`EventSource`
against this URL, parses each envelope through a Zod mirror of
:class:`EventEnvelope`, and calls
``queryClient.invalidateQueries({queryKey})`` so the live grids
re-fetch without a manual refresh.

**Cleanup on disconnect (R7).** The streaming generator is the
load-bearing R7 mitigation: the
``async with bus.subscribe(pattern):`` lives inside the body
and its ``finally:`` aclose runs on every cancellation path —
including the brutal-disconnect path Starlette translates into
a :class:`asyncio.CancelledError` propagated into the
generator. The dedicated
``test_events_sse_disconnect_unregisters_subscriber`` test
pins this contract by closing the generator manually and
asserting ``bus.stats()['subscribers'] == 0``.

**Anti-patterns explicitly rejected.** ``asyncio.Queue`` with
weak refs (re-invents the wheel; weak refs on coroutines are GC
footguns), iroh-gossip (cross-node, way too heavy for the
in-process surface this primitive serves), MQTT topic
``+`` / ``#`` (more expressive but adds a parser dependency
for no Sprint 9 win), :mod:`blinker` (sync only, not
async-friendly), :mod:`aiopubsub` (peu maintained).

Reference: sprint9_kickoff.md §4 D2, sprint9_plan.md §6 Phase
C, commit `<SHA>`.

### P15 — DB migration runner is forward-only with SHA256 tamper detection

Sprint 9 Phase D (D4). Apps that need a mutable schema ship SQL
files under `<app_package>/migrations/` and declare
`AppManifest.migrations_dir` on their manifest. The coordinator
runs :class:`nexus_sdk.MigrationRunner` at boot after each app's
`on_start` hook, before the dispatcher starts accepting tasks.

**Sqitch-inspired, not Alembic-inspired.** The runner scans
`NNN_slug.sql` files in lexicographic order (no timestamp prefix,
no DAG, no autogenerate-from-models). Each file is a plain SQL
script split on `;` and executed statement-by-statement inside a
`BEGIN IMMEDIATE` transaction. The choice is deliberate: Alembic's
model-diffing magic adds a large dependency tree and breaks in
subtle ways when the models are scattered across entry-point
plugins. A simple lexico scanner with SHA256 tamper detection
covers the Sprint 9 use case (a single `001_documents.sql` for
gov) and scales to ~50 migrations before the O(n) re-hash at
boot becomes measurable (each file < 10 KB, SHA256 ~10 µs/file).

**Tracking table.** `_nexus_migrations(version INT PRIMARY KEY,
slug TEXT, sha256 TEXT, applied_at TEXT)`. Created lazily on the
first run via `CREATE TABLE IF NOT EXISTS`. The version is
extracted from the filename prefix (`001` → 1).

**SHA256 tamper detection.** At apply time, the SHA256 of the
file content is stored in the tracking table. On every subsequent
boot the runner re-hashes every applied migration and compares.
If the hash diverges, a `MigrationTamperedError` is raised and
the coordinator refuses to boot. This catches accidental edits
to already-applied migrations. The fix is: revert the edit (the
runner is forward-only), or write a new migration that undoes
the effect.

**Forward-only.** No down-migration. No `repair`. No manual
hash override. The rollback pattern is `git revert` + a new
migration. Flyway's `repair` command is explicitly rejected as
an anti-pattern because it masks the root cause of a divergence.

**BEGIN IMMEDIATE.** Each migration runs in a `BEGIN IMMEDIATE`
transaction so a second concurrent coordinator boot on the same
SQLite file is blocked (receives `OperationalError: database is
locked`). This prevents double-apply in a race condition.

**Opt-in per app.** An app without `migrations_dir` is silently
skipped. The coordinator boot step checks
`app.manifest.migrations_dir is not None` before constructing a
runner.

**AppContext.dbs dict (R6).** Sprint 9 Phase D adds
`AppContext.dbs: dict[str, AppDatabaseClient]`. The coordinator
wires `dbs["default"]` alongside the legacy `ctx.db` field at
boot via `__post_init__` sync. The migration runner targets
`dbs["default"]` — the writable per-app SQLite — regardless of
what the app did to `ctx.db` in its `on_start` hook. This is
the load-bearing contract: the gov app swaps `ctx.db` to
point at the read-only legacy `govdata.db`, but `dbs["default"]`
remains the writable `app.sqlite` that migrations run against.
Gov additionally wires `dbs["gov"]` (read-only legacy alias) and
`dbs["app"]` (writable alias for `dbs["default"]`).

**CLI.** `nexus-coordinator migrate --project <name>
[--app <app>] --plan|--apply`. `--plan` lists pending migrations
without touching the database. `--apply` runs them. When `--app`
is omitted, every discovered app with `migrations_dir` is
processed.

Reference: sprint9_kickoff.md §4 D4, sprint9_plan.md §7 Phase
D.

### P16 — File upload + CAS with SHA256 sharding and magic bytes whitelist

Sprint 9 Phase E (D3). Apps that accept file uploads declare
`@nexus_app_files(accept=["image/*", "application/pdf"])` at
class level. The coordinator wires an `AppFileStore` per app at
`projects/<p>/apps/<a>/uploads/` before `on_start`.

**CAS layout.** SHA256 sharding `<sha[:2]>/<sha[2:]>` mirrors git
objects. Manifest JSON adjacent `<sha[:2]>/<sha>.json` carries
metadata (size, content_type, original_name, uploaded_at,
uploaded_by, app_name).

**Magic bytes validation.** Five types whitelisted: PNG, JPEG,
PDF, WebP, SVG. Validation runs after the full stream is
consumed (the first 256 bytes are probed against known magic byte
signatures). No `python-magic` dependency — portable Windows
support.

**Dedup pre-write.** If both CAS blob and manifest exist, the
store returns the existing `FileHandle` without re-writing.
Pattern from Restic.

**Soft delete.** `AppFileStore.delete` removes the manifest only;
the CAS blob stays for dedup integrity and audit trail.

**TabView v2.** `schema_version: Literal[2]` discriminated union
via `AnyTabView`. The `file_upload` block kind is v2-only; a v1
parser rejects it via `extra="forbid"`. Constructor helper
`file_upload_block()` builds the block. Cross-language fixture
`tabview_v2_canonical.json` exercises Python + Zod roundtrip.

Reference: sprint9_kickoff.md §4 D3, sprint9_plan.md §8 Phase E.

### P17 — D2/D3 wiring via SSE progress events

Sprint 9 Phase E. The coordinator file upload router publishes
`file.upload.progress` events onto the per-app `AppEvents` bus
(P14) after each successful store. The frontend can subscribe via
the `GET /app/{name}/events?pattern=file.upload.*` SSE endpoint
(Phase C) to show real-time progress. The current implementation
fires a single completion event per upload; chunked progress
streaming is deferred to Sprint 10+.

Reference: sprint9_plan.md §8 Phase E, sprint9_kickoff.md §4 D3.

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

### T8 — `CardTitle` is a `<div>`, not an `<h2>`/`<h3>` (a11y) — CLOSED Sprint 9 Phase A

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

**Resolution (Sprint 9 Phase A)**: option (a) — edit
`web/src/components/ui/card.tsx` once so `CardTitle` renders
an `<h3>` with the same className. The `<h1>` still comes
from `PageHeader`; `CardTitle` becomes the default `<h3>`
beneath it. Callers that want a different level can pass a
`className` override; a future `as` prop is not needed
because no consumer has required it so far.

Audit reference: `.planning/sprint7_audit_findings.md` §F-3.

### T9 — Coordinator `httpx.AsyncClient` per-call, no `Limits` — CLOSED Sprint 9 Phase A (as T10 in `sprint9_plan.md`)

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

**Resolution (Sprint 9 Phase A)**: option (a) — the FastAPI
`lifespan` in `packages/nexus-coordinator/src/nexus_coordinator/api/app.py`
builds a singleton
`httpx.AsyncClient(timeout=..., limits=httpx.Limits(max_connections=10, max_keepalive_connections=5))`
and stashes it on `app.state.daemon_httpx_client`. Every
handler in `api/daemon.py` reaches it via
`request.app.state.daemon_httpx_client` instead of opening
an ephemeral client per call. Regression guard
`test_daemon_proxy_shares_httpx_client` asserts the instance
reference is stable across two consecutive requests.

Audit reference: `.planning/sprint7_audit_findings.md` §G-1.

### T10 — Main bundle 0.5 KB headroom under size-limit budget — CLOSED Sprint 9 Phase A (D6)

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
because it gates Sprint 9 commits. Sprint 9 Day 0 picked
option (c) — split lazy route chunks + feature chunks — and
Phase A ships it, see **P12** above for the full contract
(createBrowserRouter lazy + feature chunks + 7 budgets). The
0.5 KB headroom is gone because the main chunk target drops
from 475 KB down to 350 KB after code splitting.

Audit reference: `.planning/sprint8_verification.md` §Notes
row 24.

### T11 — `CommandPalette` swallows `invokeAppCommand` errors — CLOSED Sprint 9 Phase A

Sprint 9 Phase 0 audit gate finding C-FX-1.
`web/src/components/command-palette/CommandPalette.tsx::runAppCommand`
closes the palette before awaiting `invokeAppCommand` and
catches the resulting promise into `console.error("[palette]
invokeAppCommand failed", ...)`. The user sees nothing — no
toast, no inline banner, no modal — and the palette stays
closed as if the command had succeeded. A 500 from the
coordinator vanishes into devtools.

Fix options:
- (a) keep the palette open in a `pending → error` state until
  the user dismisses (mirrors `ButtonBlock.tsx` inline status
  pattern from Sprint 8 Phase A)
- (b) introduce sonner for a toast layer (new dep, but
  shadcn-friendly and reusable for `submitAppTask` errors too)
- (c) `react-hot-toast` (smaller footprint, but yet-another
  dep)

Sprint 9 polish — pick (a) if no other use case for toasts
appears, otherwise (b). Track C-FX-1 of
`.planning/sprint8_audit_findings.md`.

**Resolution (Sprint 9 Phase A)**: option (a). The palette no
longer closes before awaiting the invocation. `runAppCommand`
tracks a `pending` tuple (`{appName, cmdName}`) and an
`errored` tuple (`{appName, cmdName, message}`). On success,
`setPending(null)` + `palette.setOpen(false)` runs after the
await. On error, `setErrored(...)` renders an inline
`<span data-testid="palette-cmd-error-{app}-{cmd}">` beneath
the row that failed, and the palette stays open so the user
can retry or dismiss. No new dependency — sonner is not
introduced until there is a second error surface that would
benefit from a global toast layer. Regression guards:
`CommandPalette.test.tsx::shows inline error state ...` and
`... allows retrying an errored command`.

Audit reference: `.planning/sprint8_audit_findings.md` §C-FX-1.

### T12 — `NexusApp.commands()` order depends on `dir(cls)` — CLOSED Sprint 9 Phase A

Sprint 9 Phase 0 audit gate finding C-FX-2. The Sprint 8
implementation in
`packages/nexus-sdk/src/nexus_sdk/registry.py::collect_decorators`
walks `dir(cls)` and pushes decorated methods into the
`commands` bucket in the order CPython returns them — which
is **alphabetical by attribute name** as a documented but
implementation-specific detail. The
`test_list_app_commands_ordered` test passes because of this
ordering, and the four gov commands `cmd_detect_contradictions`,
`cmd_new_scan`, `cmd_search_factchecks`, `cmd_view_alerts`
happen to sort the way the user wants.

Risk: a renaming of a method, a CPython implementation
change, or a switch to PyPy could silently scramble the
order. The same fragility applies to `_workers` and `_tabs`.

Fix: replace the implicit `dir()` ordering with an explicit
`sorted(buckets["commands"], key=lambda d: d["name"])` at the
top of `NexusApp.commands()` (and the matching helpers for
`workers()` / `tabs()` for symmetry). One-liner change, but
the test coverage needs updating to assert the sort key
explicitly.

Sprint 9 polish, low-effort. Same finding also notes that
two commands with the same `name` should raise at decorator
collection time (currently `first match wins` silently for
`resolve_worker` too — see B-FX-1 in the same audit findings).

**Resolution (Sprint 9 Phase A)**: explicit sort. At the end
of `collect_decorators`, the function now calls
`routes.sort(key=lambda d: d["path"])`,
`workers.sort(key=lambda d: d["name"])`,
`tabs.sort(key=lambda d: d["name"])`,
`commands.sort(key=lambda d: d["name"])`. Regression guard
`test_commands_ordered_by_name_explicitly` constructs an app
whose method names do NOT sort the way the descriptor names
should (`cmd_alpha` → `zeta`, `cmd_gamma` → `beta`,
`cmd_zeta` → `alpha`) and asserts the descriptor output is
`["alpha", "beta", "zeta"]`. The B-FX-1 duplicate-name raise
is still deferred — it's a separate audit finding and not
part of this fix.

Audit reference: `.planning/sprint8_audit_findings.md` §C-FX-2.

### T13 — Size-limit headroom fragile on 3 vendor chunks

Sprint 9 audit gate finding H1-A/B/C. Three size-limit budgets
have less than 10% headroom after Sprint 9:

- `vendor-react`: 274.69 / 290 KB = 5.3% headroom
- `css`: 95.16 / 100 KB = 4.8% headroom
- `vendor-ui`: 246.02 / 270 KB = 8.9% headroom

Any new dependency or component that lands in these chunks will
fail the size-limit check immediately. Before adding a dep that
contributes to vendor-react, vendor-ui, or the CSS bundle, run
`ANALYZE_MODE=true npm run build` and check `dist/stats.html`
to verify the headroom is not exhausted. If it is, bump the
budget with explicit justification in the commit body.

Audit reference: `.planning/sprint9_audit_findings.md` §H1-A/B/C.

### T14 — `FileUploadBlock.tsx` Vitest coverage below thresholds

Sprint 9 audit gate finding A3-COV / G2-A. The Sprint 9
`verify.sh` step 12 (`npm run test:coverage`) passes only
because the coverage thresholds were temporarily relaxed to
lines 85% / branches 78% (from the Sprint 6 baseline of
90% / 85%). The main offender is `FileUploadBlock.tsx` at ~35%
line coverage.

Fix: write dedicated Vitest tests for `FileUploadBlock` (file
selection, upload progress, error states, accept filter) and
restore the thresholds to 90/85. Until then, the relaxed
thresholds remain as a documented exception.

Audit reference: `.planning/sprint9_audit_findings.md` §A3-COV,
§G2-A.

### T15 — SVG BOM UTF-8 false negative in magic bytes check

Sprint 9 audit gate finding E3-A. The file upload magic bytes
validation in `packages/nexus-coordinator/src/nexus_coordinator/
api/files.py:234` uses `lstrip()` to strip whitespace before
checking for `<svg`. However, `lstrip()` does not strip the
UTF-8 BOM bytes `\xef\xbb\xbf`. An SVG file exported by
Illustrator or Inkscape with a BOM prefix is rejected as
"unsupported content type" — a false negative.

Fix: strip BOM explicitly (`content.lstrip(b'\xef\xbb\xbf')`
or decode + re-encode) before the `lstrip()` call.

Audit reference: `.planning/sprint9_audit_findings.md` §E3-A.

### T16 — CAS manifest `content_type` is client-controlled

Sprint 9 audit gate finding E3-B. The `content_type` stored
in the CAS file manifest is taken from
`file.content_type` (the multipart `Content-Type` header),
which is client-controlled. The real defense is the magic
bytes validation on the written content, which works correctly.
But the manifest stores whatever the client claims, not the
canonicalized type from magic bytes.

Fix: after magic bytes detection succeeds, overwrite
`content_type` in the manifest with the detected type before
writing. This closes the gap between "what we validated" and
"what we stored".

Audit reference: `.planning/sprint9_audit_findings.md` §E3-B.

### T17 — `AppFileStore.open()` reads entire file into memory

Sprint 9 audit gate finding E6-A.
`packages/nexus-coordinator/src/nexus_coordinator/api/files.py`
`AppFileStore.open()` calls `cas.read_bytes()` which reads the
entire file content into memory before chunking it for the HTTP
response. For a 50 MB file (the max_size_bytes limit), this
means 50 MB of RAM per concurrent download.

Risk is bounded by the E6-B fix (max_size_bytes is now enforced
at upload time), so the worst case is 50 MB, not unlimited.
For the loopback-only use case this is acceptable. A streaming
`AsyncIterator[bytes]` read path is a nice-to-have for
Sprint 11+ if large file handling becomes a real use case.

Audit reference: `.planning/sprint9_audit_findings.md` §E6-A.

### T18 — `test_concurrent_store_same_sha256_dedup_safe` flaky on Windows

Sprint 9 audit gate finding E-FLAKY. The test spawns two
concurrent uploads of the same SHA256 content. On Windows,
`os.replace()` on the manifest file occasionally raises
`PermissionError [WinError 5]` because the other task has the
file open for writing. This is a Windows-specific race on
`os.replace` that does not occur on Linux/macOS (where
`rename(2)` is atomic even if the target is open).

Fix: wrap the `os.replace` call in a retry loop with
exponential backoff (3 attempts, 50ms base delay). The
deduplication logic is correct — the race only affects the
manifest write, not the blob content.

Audit reference: `.planning/sprint9_audit_findings.md` §E-FLAKY.

### T20 — `asyncio.wait_for()` in anyio-based SSE generator

Sprint 9 audit gate finding C3-1. The SSE event streaming
generator in `packages/nexus-coordinator/src/nexus_coordinator/
api/events.py:86-89` uses `asyncio.wait_for()` for the receive
timeout. This is an asyncio-specific API that would break if
the coordinator ever ran on a Trio backend.

Risk is nil in practice (FastAPI = uvicorn = asyncio), but it
is an impurity in code that otherwise uses anyio primitives.

Fix: replace with `anyio.fail_after()` or `anyio.move_on_after()`
which work on both asyncio and Trio backends.

Audit reference: `.planning/sprint9_audit_findings.md` §C3-1.

### T21 — `useAppEvents` creates one EventSource per component mount

Sprint 9 audit gate finding C4-1. The React hook
`web/src/hooks/useAppEvents.ts` creates a new `EventSource`
connection every time the `AppTabPage` component mounts. In
the current SPA (one `AppTabPage` at a time), this is fine.
But if multiple tab pages coexist in the future (e.g., split
view), N simultaneous SSE connections will open.

Fix: extract the EventSource into a singleton at the store
level (similar to how `projectStore` works), shared across
all mounted tab pages. Sprint 11+ if the use case materializes.

Audit reference: `.planning/sprint9_audit_findings.md` §C4-1.

### T22 — `test_gov_documents.py` schema diverges from `001_documents.sql`

Sprint 9 audit gate finding D4-A. The test file
`packages/nexus-app-gov/tests/test_gov_documents.py:37-45`
defines a `_DOCUMENTS_SCHEMA` with column names `original_name`
and `size` that do not match the real migration
`001_documents.sql:8-18` which uses `filename` and `size_bytes`.
The test assertions verify a phantom schema.

Fix: align the test schema with the real migration column names
and re-run assertions.

Audit reference: `.planning/sprint9_audit_findings.md` §D4-A.

### P18 — Self-publish via gossip: coordinator → daemon → BrowseAggregator

Sprint 11 Phase A (`65af280`). When a coordinator starts with
`visibility = "public"`, it calls `POST /project/publish` on the
shell daemon, which broadcasts a `ProjectAnnouncement` (v=1,
type="project") on the curator gossip topic. Other daemons
receive it via `process_project_announcement()` in
`iroh_runtime.rs` and add it to `BrowseAggregator` as a
`BrowseSource::Direct` entry (no curator intermediary).

Key invariant: the gossip topic is shared between curator
announcements and project announcements. The discriminator is
the `"type"` field in the JSON payload. A message without
`"type": "project"` falls through to the curator handler.

Files: `publish.rs`, `browse.rs` (BrowseSource enum +
add_direct_entry), `iroh_runtime.rs` (process_project_announcement),
`http.rs` (POST /publish), coordinator `health.py` (POST
/project/publish), coordinator `coordinator.py` (auto-publish
step in start()).

### P19 — Default curators via config, idempotent auto-subscription

Sprint 11 Phase B (`e5cc165`). The daemon config gains a
`[curator]` section with `default_curators: Vec<String>` (hex
pubkeys). At boot, the daemon iterates these and calls
`CuratorRuntime::subscribe()` for any not already present in
`subscriptions.json`. This is idempotent: restarting the daemon
does not duplicate subscriptions.

The coordinator exposes `GET /daemon/default-curators` as a proxy
for the shell web to display "default curators" in the Curators
page.

Files: `config.rs` (CuratorConfig), `runtime.rs` (auto-subscribe
loop), `http.rs` (GET /default-curators), coordinator `daemon.py`
(proxy), `deploy/config.toml.example`.

### P20 — Browse → full-screen app rendering via `/browse/:projectId`

Sprint 11 Phase C (`6bdd089`). Clicking a BrowseCard navigates to
`/browse/:projectId` which renders `BrowsedProject.tsx`: sidebar
with project metadata + TabView full-screen for local projects.
Remote projects show a placeholder ("hosted on a remote node").

The `WebAppFrame.tsx` component is a skeleton iframe sandbox for
Sprint 12+ web app blob rendering. Currently shows a placeholder.

The `BrowseEntry` Zod schema gains an optional `source` field
(Curator | Direct) for backward compatibility with daemons that
do not emit the field.

Files: `BrowsedProject.tsx` (~421 LOC), `WebAppFrame.tsx` (~35
LOC), `Browse.tsx` (clickable cards), `App.tsx` (lazy route),
`daemon.ts` (source field), `coordinator.ts` (getProjectApps).

### P21 — Daemon blob-serve: zip decompression + LRU cache + CSP isolation

Sprint 12 Phase A (`32a1dca`). Archives zip stockees comme blobs
iroh sont servies via `GET /blob-serve/{hash}/{path}`. Le daemon
decompresse le zip en memoire (crate `zip` 2.6), cache les
fichiers dans un `BlobServeCache` LRU (32 entries par defaut),
et sert chaque fichier avec :
- `Content-Security-Policy: connect-src 'none'` (bloque les
  requetes sortantes des scripts dans l'iframe)
- `X-Content-Type-Options: nosniff`
- `Cache-Control: public, max-age=3600, immutable`
- Content-type detection par extension + magic bytes

Protections : path traversal rejection (`.., \, /` absolus),
zip bomb limit (100MB decompresse), validation hex du hash.

Files: `blob_serve.rs` (~270 LOC), `http.rs` (handler + route).

### P22 — TabView pre-render: Python HTML generator

Sprint 12 Phase B (`52d4004`). `nexus_sdk.html_render` convertit
un descriptor TabView (dict) en page HTML self-contained. 12 block
kinds supportes avec inline CSS dark theme miroir des tokens
Tailwind. Charts line/bar en SVG inline. Blocks interactifs
(button, file_upload) degradent en placeholder lecture seule.

Le coordinator auto-publie le HTML au boot : chaque tab de chaque
app montee est pre-rendu, emballe dans un zip avec redirects
index.html, stocke comme blob, et annonce via ProjectAnnouncement
v2 avec `archive_hash`.

Files: `html_render.py` (~460 LOC), `deploy.py` (POST /project/deploy),
`coordinator.py` (_build_and_store_archive).

### P23 — Cross-node iframe rendering: sandboxed untrusted content

Sprint 12 Phase C (`fccea74`). Quand un projet distant a publie
une archive (archive_hash present dans BrowseEntry), le shell
affiche le contenu dans une iframe `sandbox="allow-scripts"` (sans
`allow-same-origin` — origin opaque). Une banniere "Contenu publie
par un tiers" s'affiche au-dessus. Si pas d'archive, le placeholder
texte est affiche.

L'URL de l'iframe est construite via `blobServeUrl(daemonBaseUrl,
hash)` pointant vers le daemon local. Le hash BLAKE3 hex est porte
par `BrowseEntry.archive_hash` (ajoute en Phase C) car le
BlobTicket est base32-opaque.

Files: `daemon.ts` (archive_hash, blobServeUrl, daemonBaseUrlFromInfo),
`BrowsedProject.tsx` (RemoteProjectFrame), `browse.rs` (archive_hash),
`http.rs` (archive_hash in BrowseEntry), `nginx-nexus.conf` (/blob-serve/).

### T23 — SPDX scope excludes `nexus/` legacy Python files

Sprint 10 audit gate finding A-1. The `scripts/check-spdx.sh` guard
covers `crates/`, `packages/`, `web/src/` (204 files) but does not
include `nexus/` (~30+ legacy .py files). D6 decision said "every
source file" but the plan scoped it to active modules only. Since
the project is AGPL-3.0, all distributed source should carry the
header. Low priority — treat when `nexus/` code is next touched.

Audit reference: `.planning/sprint10_audit_findings.md` §A-1.

### T24 — `provision.sh` UDP firewall rule too broad

Sprint 10 audit gate finding E-1. Line 47 of `deploy/provision.sh`
runs `ufw allow proto udp from any to any`, opening ALL UDP ports.
Required for iroh QUIC (ephemeral ports), but broader than ideal.
Document rationale inline. If iroh supports a fixed port range in
a future release, restrict the rule.

Audit reference: `.planning/sprint10_audit_findings.md` §E-1.

### T25 — `deploy/README.md` missing operational runbook

Sprint 10 audit gate finding E-2. The deploy README covers initial
provisioning and deployment but lacks operational sections: rollback
procedure, identity key backup, log monitoring, SSH key management,
health check endpoints. Enrich as real VPS operations begin.

Audit reference: `.planning/sprint10_audit_findings.md` §E-2.

### T26 — nginx config duplicated inline in `provision.sh`

Sprint 11 Phase D (`999fec6`). The nginx site config is defined
both in `deploy/nginx-nexus.conf` (canonical) and inline in
`deploy/provision.sh` (heredoc). If one is updated without the
other, the VPS config drifts from the reference file. Consider
having `provision.sh` copy `nginx-nexus.conf` from repo instead
of embedding. Requires the repo to be cloned on the VPS, which is
not always the case for initial provisioning.

### T27 — `deploy-web.sh` destructive `rm -rf` without rollback

Sprint 11 Phase D (`999fec6`). `deploy-web.sh` runs
`rm -rf /opt/nexus-grid/web/*` before uploading the new build.
If the upload fails mid-way, the VPS serves a broken/empty site.
Consider deploying to a timestamped directory and atomically
swapping the symlink, or keeping the previous build as a backup.

### T28 — `node_id` not validated in `ProjectAnnouncement::from_gossip_bytes()` — CLOSED Sprint 12 Phase E

Sprint 11 audit A-02. `publish.rs` validates `v` and `msg_type`
but applies no length or hex-format check to `node_id`. A peer
can inject an announcement with `node_id: ""` that gets stored as
a `BrowseEntry` with `project_id: ""` and surfaced in the UI as
Unreachable. Add a 64-char hex validation in `from_gossip_bytes`.

### T29 — No test for truncated gossip message — CLOSED Sprint 12 Phase E

Sprint 11 audit A-01. `is_project_announcement` and
`from_gossip_bytes` handle truncated JSON correctly (serde returns
Err), but no test exercises this path. Add
`is_project_announcement(b"{\"type\": \"project\"")` test.

### T30 — Missing coordinator tests (daemon 500 + auto-publish private) — CLOSED Sprint 12 Phase E

Sprint 11 audit B-01/B-02. No test when daemon returns HTTP 500
(proxy wraps as `{kind: "data", status: 500}` — untested). No
test asserting a private coordinator does NOT call publish at boot.

### T31 — `default_curators` not validated as hex at config load — CLOSED Sprint 12 Phase E

Sprint 11 audit C-01. `CuratorConfig.default_curators` is
`Vec<String>` with no format constraint. Validation only fires at
`subscribe()` time. Add hex validation at config parse with a
descriptive error message.

### T32 — DRY nginx config: `provision.sh` inline vs `nginx-nexus.conf` — CLOSED Sprint 12 Phase E

Sprint 11 audit F-02. `provision.sh` embeds a full inline copy
of the nginx config (lines 42-71) separately from
`deploy/nginx-nexus.conf`. Replace the heredoc with
`scp nginx-nexus.conf` or `cp` from the repo.

### T33 — HTTPS/certbot for VPS — CLOSED Sprint 12 Phase E

Sprint 11 audit F-05. Both nginx configs are HTTP-only
(`listen 80`). No certbot/Let's Encrypt provisioning exists.
Scope cut for Sprint 11 but needed for production.

### T34 — `BrowsedProject.tsx` missing from vitest coverage.include — CLOSED Sprint 12 Phase E

Sprint 11 audit H-01. The primary Sprint 11 component (421 LOC)
is excluded from `vitest.config.ts` coverage.include, so its
branch/line coverage is invisible to the 85%/78% threshold gate.
Add `src/pages/BrowsedProject.tsx` to the coverage scope.

### T35 — `aggregate_flattens_curator_lists_with_cached_status` test is hollow — CLOSED Sprint 12 Phase E

Sprint 11 audit H-03. browse.rs test creates a curator list
entry but discards it with `let _ = entry` and asserts
`out.is_empty()` (same as the empty-curator test). Rewrite to
actually exercise the flattening scenario.

### T36 — `X-Forwarded-Proto` missing in `/daemon/` nginx location — CLOSED Sprint 12 Phase E

Sprint 11 audit F-06. The `/api/` proxy block sets
`X-Forwarded-Proto $scheme` but the `/daemon/` block does not,
in both `nginx-nexus.conf` and `provision.sh` inline. Add the
header for consistency.

### T37 — CSP middleware for all blob-serve responses — CLOSED

Sprint 12 audit A-P2. Fixed in Sprint 13 Phase A: added
`blob_serve_csp_middleware` (axum `from_fn`) that injects CSP +
X-Content-Type-Options on ALL responses (200, 400, 404, 500).
Blob-serve routes now use a nested Router with this middleware
layer. The 200 handler no longer duplicates the headers. Test
`blob_serve_error_responses_have_csp` verifies 404 has CSP.

### T38 — Align html_render SVG chart dimensions with React — CLOSED

Sprint 12 audit B-P2. Fixed in Sprint 13 Phase A: updated
`html_render.py` constants to match React: H=120, PAD_L=32,
PAD_R=16, PAD_T=16, PAD_B=16 (line) / PAD_T=24, PAD_B=24 (bar).

### T39 — Test file_upload block in test_html_render.py — CLOSED

Sprint 12 audit B-P2. Fixed in Sprint 13 Phase A: added
`test_render_file_upload()` covering the placeholder HTML output
including label escaping and the "upload non disponible" text.

### T40 — X-Real-IP header in /blob-serve/ nginx block — CLOSED

Sprint 12 audit F-P2. Fixed in Sprint 13 Phase A: added
`proxy_set_header X-Real-IP $remote_addr;` to the `/blob-serve/`
location block in `deploy/nginx-nexus.conf`.

## Sprint 13 patterns

### P24 — postMessage bridge protocol

**Rule**: all communication between sandboxed iframe apps and the
host shell goes through a typed postMessage bridge. The protocol
uses `BridgeRequest` (iframe → host) and `BridgeResponse` (host →
iframe) with UUID correlation IDs for async matching. The method
enum is a whitelist extended additively across sprints — never
bumped as a protocol version. As of Sprint 21 it contains four
entries: `task_submit`, `storage_get`, `storage_set` (Sprint 13)
and `pii_redact` (Sprint 21 Phase B, local host dispatch, no
coordinator round-trip). The host validates source (`event.source
=== iframe.contentWindow`), parses with Zod, dispatches to the
coordinator API (or locally for `pii_redact`), and replies. No
direct network access from the iframe (CSP `connect-src 'none'`).

SHA: `c32d9c7` (S13 baseline), extended Sprint 21 Phase B. Files:
`web/src/bridge/protocol.ts`, `useBridge.ts`, `web/public/sbfb-
bridge.js`.

### P25 — open source enforcement for public apps

**Rule**: any project published with `visibility=public` must provide
a `repo_url` (URL of a public source code repository). The coordinator
validates this at `POST /project/deploy` and returns HTTP 400 if
missing. The daemon propagates `repo_url` in the gossip announcement
(ProjectAnnouncement v3) and the BrowseEntry. The shell displays a
clickable "Source" link on Browse cards and BrowsedProject top bar.
Private projects have no constraint. Sprint 13 does NOT verify that
the repo actually matches the deployed zip — deferred to Sprint 14.

SHA: `7d669f2`. Files: `publish.rs`, `browse.rs`, `http.rs`,
`deploy.py`, `daemon.ts`, `Browse.tsx`, `BrowsedProject.tsx`.

### P26 — launcher pattern (spawn daemon + poll running.json)

**Rule**: the nexus-launcher binary spawns `nexus-shell-daemon start`
as a child process, polls `running.json` until it appears (max 15s),
reads `api_host:api_port`, opens the default browser via the `open`
crate, then waits for Ctrl+C. On shutdown, kills the child and waits
5s for exit. If the daemon is already running (running.json exists and
parses), skips spawn. No Tauri, no native window — the browser IS the
client. Decision D4 from Sprint 13 kickoff.

SHA: `72cf5ad`. File: `crates/nexus-launcher/src/main.rs`.

## Sprint 13 audit tech debt

### T41 — repo_url XSS via javascript: protocol — SUPERSEDED

Sprint 13 audit B-P2. `repo_url` accepted any non-empty string
including `javascript:alert(1)`. The frontend rendered it as
`<a href={entry.repo_url}>` — XSS possible on click. Superseded
by Sprint 14 verified deploy from source: the coordinator clones
the repo itself, so `repo_url` is no longer a user-provided
trust-based field. The `<a>` links already have `rel="noopener
noreferrer"` and `target="_blank"`.

### T42 — text-white/30 contrast below WCAG AA — CLOSED

Sprint 13 audit D-P2. Two instances of `text-white/30` on 11px
text: BrowsedProject.tsx:271 (sandbox label) and
ProjectDetail.tsx:131 (coordinator URL). Contrast ratio ~3.3:1,
below WCAG AA 4.5:1 threshold. Fixed in Sprint 14 Phase C:
both raised to `text-white/40` (~4.4:1).

### T43 — SVG PAD_R 16 ≠ React PAD_X 32 — CLOSED

Sprint 13 audit G-P2. `_SVG_PAD_R = 16` in `html_render.py`
diverged from React's symmetric `PAD_X = 32`. Python charts
were 16px wider than React charts. Fixed in Sprint 14 Phase C:
`_SVG_PAD_R = 32` to match React.

## Sprint 14 audit tech debt

Logged in Sprint 15 Phase E from `sprint14_audit_findings.md`.
The Sprint 14 audit verdict was CONDITIONAL PASS — the single
P1 (A-1, commit_sha passed to `git clone --branch`) was fixed
in `542479f` before Sprint 15 Phase A. The eight P2 items
below are non-blocking but tracked here so they surface in
future sprints or reviews.

### T44 — `_dir_size` check is post-clone, not streaming

Sprint 14 audit A-P2. `_clone_repo` runs `git clone` to
completion, then `_dir_size` checks if the clone exceeded
500 MB. A malicious repository hosting 499 MB of content would
still use 499 MB of tmpfs during the clone. The 30s
`CLONE_TIMEOUT_SECS` is the only real defense against large
repos — attackers on slow links can't exceed the limit in
time, but fast-link attackers can fill the tmpdir before the
check fires.

Mitigation (future sprint): stream `git clone --progress` and
tee stderr to a byte counter, aborting when the 500 MB mark
is passed. Or use `GIT_HTTP_MAX_REQUEST_BUFFER` env var.

Ref: `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py:_dir_size`.

### T45 — `_git_rev_parse` has no timeout

Sprint 14 audit A-P2. `_git_rev_parse(clone_dir)` is a
subprocess without a `asyncio.wait_for` wrapper. Post-clone
on a shallow `--depth 1` repo, this is fast (reads
`.git/HEAD`), so the risk is low. But a corrupt clone or a
git binary hang would block the deploy indefinitely.

Mitigation: add `timeout=5` and HTTPException on timeout.

Ref: `deploy.py:_git_rev_parse`.

### T46 — `startswith("http")` accepts `http://`

Sprint 14 audit A-P2. The guard `if not repo_url.startswith(
"http")` accepts both `http://` and `https://`. A MITM on a
plain-text git clone could swap the SBFB.json or insert
additional commits. For public forges (GitHub, GitLab,
Codeberg), `http://` redirects to `https://` but the redirect
itself is not cryptographically verified.

Mitigation: tighten to `startswith("https://")` or use a
regex `^https://`.

Ref: `deploy.py:deploy_from_repo`.

### T47 — `provenance.py` uses `json.dumps` instead of `jcs`

Sprint 14 audit B-P2. The provenance canonical bytes are
built with `json.dumps(sort_keys=True, separators=(',',':'))`
which is equivalent to JCS only for flat string/int schemas
with ASCII content. The convention across the project is
`serde_jcs` (Rust) and `jcs` PyPI (Python). Today's schema
is fine; a future field with non-ASCII Unicode would diverge.

Mitigation: switch to `jcs.canonicalize(payload)` for
forward-compat.

Ref: `packages/nexus-coordinator/src/nexus_coordinator/provenance.py:_canonical_bytes`.

### T48 — `verify_provenance` ignores `schema_version`

Sprint 14 audit B-P2. `verify_provenance(record_json, pk)`
pulls `data["schema_version"]` into the signable payload but
doesn't assert it matches `PROVENANCE_SCHEMA_VERSION`. If a
v2 schema is introduced later with different field semantics,
a v1 verifier would still validate v2 payloads whose common
fields look right — a cross-version replay trap.

Mitigation: add `if data.get("schema_version") !=
PROVENANCE_SCHEMA_VERSION: return False` at the top of
`verify_provenance`.

Ref: `provenance.py:verify_provenance`.

### T49 — PA v4 bump breaks forward compat for additive field

Sprint 14 audit D-P2. `publish.rs:from_gossip_bytes` rejects
announcements with `v > PROJECT_ANNOUNCEMENT_VERSION` (v4).
A v3 daemon hearing a v4 announcement therefore drops the
message, even though the only delta is an optional
`provenance_hash` field that serde would happily ignore.

This is a design decision from the Sprint 14 kickoff D3 and
not a bug, but future additive fields should consider
**not** bumping the version and instead relying on serde
`#[serde(default)]` for graceful unknowns.

Ref: `crates/nexus-shell-daemon-core/src/publish.rs:131`.

### T50 — D4 clone protections lack dedicated tests

Sprint 14 audit G-P2. Of the seven D4 kickoff protections
(depth 1, single-branch, size 500 MB, timeout 30s, no .git/,
path traversal rejection, no submodules), only two have
dedicated tests: `.git/` exclusion is asserted in
`test_deploy_from_repo_provenance_in_zip`, and depth is
implicit in the happy path. The remaining five lack
end-to-end tests — a regression that removes a protection
(e.g. deleting the `".."` check in `_zip_directory`) would
not be caught.

Mitigation: add integration tests with synthetic malicious
repos (symlinks, `../` paths, oversized content).

Ref: `packages/nexus-coordinator/tests/test_deploy.py`.

### T51 — `_clone_repo` never exercised against a real subprocess

Sprint 14 audit G-P2 (related to T50). Every test of the
deploy-from-repo endpoint mocks `_clone_repo` via
`_make_mock_clone` which does a `shutil.copytree`. The real
`git clone` subprocess is never run in unit tests. Sprint 15
Phase 0 gate fix (A-1 commit_sha SHA pinning) added three
integration tests that DO exercise real `git clone` against
a local `file://` repo — these tests are the blueprint for
expanding coverage to the D4 protections listed in T50.

Ref: `packages/nexus-coordinator/tests/test_deploy.py::test_clone_repo_*`.

## Sprint 16 patterns

### P27 — Defense en profondeur loopback (bearer + Host + Origin + peer creds)

**Rule**: every HTTP request against the coordinator FastAPI
(`:8080`) or the shell daemon HTTP surface (`:7777`) is gated
by a **triple-check middleware** plus an orthogonal **peer
credential bypass** for UDS / Named Pipe connections. A request
is accepted if and only if one of these holds:

1. (TCP loopback path) `X-SBFB-Token` header matches the bearer
   from `~/.sbfb/auth_token` **AND** `Host:` in `{localhost,
   127.0.0.1, [::1]}` (optional port) **AND** `Origin:` absent
   or in the shell allowlist (`http://localhost:<shell_port>`).
2. (UDS / Named Pipe path) the connection was accepted by the
   UDS / NP listener which verified peer credentials, and the
   accept loop injected a private-typed `PeerCredsVerified`
   marker into the request extensions.

The single exception is `/health` (unauthenticated probe).
`/blob-serve/*` is also open: the content is iframe-sandboxed
(CSP `connect-src 'none'`) and already public by BLAKE3 hash on
the P2P network — bearer would add no protection.

Why all three headers:

- **Bearer alone** fails against DNS rebinding (CVE-2025-49596
  Anthropic MCP Inspector, CVSS 9.4): a malicious public site
  resolves to `127.0.0.1` via TOCTOU DNS flip and hits `localhost`
  from the victim's browser — no cross-origin preflight, bearer
  absent so defense fires, but if we ever add a cookie-based
  session we'd be trivially breakable.
- **Host allowlist** blocks DNS rebinding outright: any `Host:
  attacker.com` is rejected even with a valid token.
- **Origin check** blocks opt-in CORS endpoints from being
  abused by a malicious site that DOES have a valid token
  (e.g. leaked via an extension).

Why peer creds are orthogonal, not a replacement:

- The browser cannot connect over UDS / Named Pipes. Bearer is
  mandatory for the React shell path.
- CLI `sbfb` and coord-to-daemon internal calls prefer the UDS /
  NP path when available — peer creds give native OS-level auth
  without token rotation concerns.
- `PeerCredsVerified` is a **private Rust type** injected only
  by the accept loops. A remote caller cannot spoof it via a
  header — there's no header version.

**SHA**: `d7c265a` (bearer + Host + Origin) + `1cfde89` (UDS
SO_PEERCRED + Named Pipes SDDL DACL user-only).

**Files** (Rust):
- `crates/nexus-launcher/src/auth.rs` (460 LOC) — token
  generation + persistent file perm 0600 + `/auth/token`
  endpoint on ephemeral loopback port.
- `crates/nexus-shell-daemon-core/src/auth.rs` (708 LOC) —
  `auth_required` axum middleware + `is_loopback_host` /
  `is_loopback_origin` + `PeerCredsVerified` marker type.
- `crates/nexus-shell-daemon/src/uds_server.rs` (366 LOC) —
  accept loop `UnixListener` + `SO_PEERCRED` via `getsockopt`
  (Linux) / `getpeereid` (macOS / BSD).
- `crates/nexus-shell-daemon/src/named_pipe_server.rs` (417
  LOC) — `CreateNamedPipeW` with `SECURITY_ATTRIBUTES` built
  via SDDL `D:(A;;GA;;;<current-user-SID>)`. Prevents default
  permissive DACL on Windows.

**Files** (Python):
- `packages/nexus-coordinator/src/nexus_coordinator/auth.py`
  (229 LOC) — `LoopbackAuthMiddleware` Starlette, same rules.
  Wired in `api/app.py` **inside** the CORS middleware so CORS
  answers OPTIONS preflight before bearer fires.
- `packages/nexus-coordinator/src/nexus_coordinator/peer_creds.py`
  (92 LOC) — `SO_PEERCRED` via `socket.getsockopt` + `struct`.
  Helper only, not yet wired into ASGI (uvicorn doesn't expose
  the raw FD in `scope` — scope cut Sprint 17+).

**Files** (TypeScript):
- `web/src/api/auth.ts` (122 LOC) — `primeAuthToken` fetches
  the token from the launcher once at boot, caches it, exposes
  `authFetch(url, init)` that injects `X-SBFB-Token`. All
  `coordinator.ts` / `daemon.ts` helpers go through `authFetch`.
- `web/src/main.tsx` — seeds `window.__SBFB_AUTH_TOKEN` if
  provided by Playwright global setup.
- `web/playwright.config.ts` — `extraHTTPHeaders` injects the
  bearer globally so no test needs to think about auth.

**Upgrade note**: v1.1 users restart daemon + coord + launcher
after v1.2 install. Launcher generates the token on first run.
CLI callers of `/project/*` / `/app/*` must send `X-SBFB-Token`
(export from `~/.sbfb/auth_token`). `/health` unchanged.

### P28 — GPU consent 4 levels + caps enforced worker-side

**Rule**: the worker `crates/nexus-worker-core` refuses to claim
a task unless it passes `should_accept_task(&task, &consent,
&mut usage)` — a pure function over the task, the user's
consent config, and the daily usage tracker. This is the **only**
gate; the UI's sliders and radios are persisted into
`~/.sbfb/consent.json` but are **not trusted** on their own. The
worker re-reads that file live via a `notify` watcher (50 ms
debounce) so the user can revoke consent without restarting.

Level filter semantics:
- **L1 OwnProjects** — reject if `task.project_id !=
  consent.own_node_id`.
- **L2 OpenSource** — reject if `!task.is_open_source` (Sprint
  16 Phase D flag, derived server-side from deploy-from-repo).
- **L3 Whitelist** — reject if `task.project_id ∉
  consent.allowed_project_ids` (HashSet lookup O(1)).
- **L4 All** — accept level check; caps still apply.

Cap filter semantics (after level passes):
- `task.estimated_watts > consent.caps.max_watts` → reject.
- `task.estimated_vram_mb > consent.caps.max_vram_mb` → reject.
- `usage.hours_used_today() + task.estimated_hours >
  consent.caps.max_hours_day` → reject.

Usage reset runs on local midnight via `chrono::Local::now().date_naive()`
comparison at each `reserve_hours` call — no timer thread, no
DST bugs (tested with `TZ=America/Chicago` fake clock).

All writes to both `consent.json` and `usage.json` are atomic
(`tmp + rename`) so a crash mid-write never leaves the worker
reading garbage.

**SHA**: `3247e88` Phase C.

**Files**:
- `crates/nexus-worker-core/src/consent.rs` (952 LOC) — all of
  the above, self-contained module next to the existing
  `allowlist.rs` (invite-token enrollment, distinct concern).
- `crates/nexus-worker-core/src/engine/runtime.rs` — claim loop
  calls `should_accept_task` after `verify_signature`, logs
  structured rejection reason (observability).
- `web/src/components/GpuConsentDialog.tsx` (385 LOC) —
  shadcn/ui dialog, default L1 (GDPR-safe, no pre-checked L2+).
- `packages/nexus-coordinator/src/nexus_coordinator/api/consent.py`
  (227 LOC) — four endpoints: `GET /consent/get`, `POST
  /consent/set`, `POST /consent/whitelist/add`, `POST
  /consent/whitelist/remove`. All gated by P27.

**Trade-off**: L2 relies on `is_open_source` being trustworthy.
Sprint 16 Phase D (`10bbc63`) derives it server-side at publish
time (`true` only for deploy-from-repo, `false` for private
zip), making it non-user-settable — pattern npm provenance /
cosign self-managed keys. A publisher cannot flag a private zip
as open source.

**GDPR mapping**: Art.6(1)(a) lawful basis via explicit opt-in;
Art.7(3) withdrawal via the same dialog ("Modifier consentement"
on Network page); Art.25 privacy-by-design via L1 default.
Detail in `docs/security/THREAT_MODEL.md` §6.1.

## Sprint 19 patterns

### P29 — Delayed upload queue (exponential jitter, SQLite-persisted)

> **⚠️ WARNING — payload stored plaintext on disk, no encryption at
> rest.** `upload_queue.sqlite` WAL file stores task payload +
> metadata in clear until the scheduler flushes. Encryption at rest
> is the Sprint 20 big rock (`HARDENING_ROADMAP §3 S20`). Operators
> enabling `[coordinator.upload_queue] enabled = true` on a shared
> host before S20 keypair wrap must accept this trade-off. Audit
> finding S19 D-1 (cf. `.planning/active/sprint19_audit_findings.md
> §Track D`) promoted this caveat from the tech-debt subsection to
> the header for operator visibility.

**Rule**: every `/tasks/submit` is routed through
`nexus_coordinator.upload_queue.UploadQueue` before the
dispatcher writes to the project doc. The queue draws a
cryptographically-random delay from an exponential distribution
(default mean=90 s, clamp 300 s), persists the payload in
`upload_queue.sqlite` (WAL mode), and a background scheduler
flushes every 30 s. The observer at the network edge sees a
POST → gossip emit correlation window of 0–5 min instead of
<100 ms, breaking short-window timing correlation attacks (T2/T3
in `docs/security/P2P_THREATS.md §6.3`).

**Why exponential, not uniform or Poisson process**: Cornell
ESORICS 2006 proved fat-tail distributions dominate uniform for
anti-correlation; the Loopix 2017 Poisson process is
theoretically stronger but requires a mix-net with k-anonymity
>> 1, which is post-launch infrastructure (Sprint 25+).
Exponential mean=90 s is the minimum viable anti-correlation
compatible with our single-user pre-launch anonymity set = 1.
Full decision matrix in
`.planning/research/S19_phase_D_delayed_upload_queue_design.md §3`.

**Why 0-5 min range (not 30 min Tor-style, not 2 min)**:
DnD Forge UX target is "<2 min median, <5 min p99" — sits
between SimpleX (1–5 s) and Briar (30 s–minutes). Observed
median = ln(2) × mean ≈ 62 s, p99 clamped to 300 s. Design
§4.1 for the latency tolerance study.

**Why SQLite-persisted (not in-memory as plan §7.4 originally
suggested)**: a coord crash within 90 s of a `/tasks/submit`
would silently drop the task — user sees `{"task_id": "..."}`
but the task never reaches the network. The design doc upgrades
to SQLite WAL in a dedicated `upload_queue.sqlite` file next to
`state.sqlite`. Cost: ~10 LOC schema + ~10 LOC INSERT/DELETE,
<10 µs per op under WAL. Recovery: `UploadQueue.start()` calls
`_rerandomize_stale_on_boot` so rows whose `deliver_at` is
already past at reboot get a fresh delay — mitigates the
thundering-herd burst after a long downtime (design §6.7).

**Idempotency invariant**: `Dispatcher.submit` is now idempotent
on `task_id`. If a row for the id already exists in
`task_state`, the method returns the id without re-signing,
re-writing the doc, or re-inserting. This makes a queue retry
after a partial-commit crash safe — the second emit is a no-op
instead of producing a duplicate TaskEntry on the doc.

**Tunable**: `coordinator.toml [upload_queue]` exposes
`enabled`, `mean_jitter_s`, `max_jitter_s`, `flush_interval_s`,
`soft_cap`, `hard_cap`. `enabled = false` is the dev escape
hatch — submit passes through to the dispatcher synchronously,
no row ever written to SQLite. Production must keep
`enabled = true`; changing the default in a deployed build
breaks the cross-coord anonymity set (design §6.6).

**Backpressure**: soft cap (default 10 000) logs WARN; hard cap
(default 100 000) raises `QueueFullError` → HTTP 429 with
`Retry-After: 30`.

**Internal clamp**: the queue clamps each drawn delay to
`max_jitter_s - flush_interval_s` internally (270 s with the
defaults) so the observable p99 — drawn delay plus scheduler
granularity — still respects the advertised `max_jitter_s`
ceiling (300 s).

**Test injection**: `UploadQueue(..., rng=random.Random(seed),
now_fn=mutable_clock)` gives the pytest suite full determinism
without a `freezegun` dep. Production uses
`secrets.SystemRandom` (CSPRNG) and `time.time`.

**Files**:
- `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py`
  (~345 LOC) — `UploadQueue`, `QueueFullError`, `_bucket`
  helper for log INFO histogram.
- `packages/nexus-coordinator/src/nexus_coordinator/config.py`
  — `UploadQueue` pydantic section.
- `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py`
  — `_dispatcher_emit_adapter` (dict → SubmitRequest reinflate)
  + start/stop wiring.
- `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py`
  — `submit_task` routed through `coord.upload_queue.schedule`,
  `QueueFullError` → HTTP 429.
- `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py`
  — idempotency check + `INSERT OR IGNORE` on `task_state`.

**Tech debt (queued for later sprints)**:
- `T-S19-D-1`: no per-payload retry budget — an emit_fn that
  raises every tick loops forever on the same row. Operator
  monitoring required; Sprint 22+ will add a backoff-with-dead-
  letter pattern.
- `T-S19-D-2`: no priority queue — a high-priority task waits
  as long as a background one. Design choice for uniform anti-
  correlation; priority scheduling trivially leaks ordering
  info to a timing observer.
- `T-S19-D-3`: no per-app sub-queue — every app shares the same
  flush tick. Sprint 22+ per-app sub-queues will let each app
  tune its own privacy/UX trade-off independently.
- Mix-net Loopix integration (Sprint 25+) will replace
  `emit_fn` with a Sphinx-packet route; the UploadQueue front
  stays unchanged.

Reference: Phase D plan at
`.planning/active/sprint19_plan.md §7`, design doc at
`.planning/research/S19_phase_D_delayed_upload_queue_design.md`.

## Sprint 20 patterns

### §P30 — LLM backend dual-build operator runbook

Sprint 20 Phase D introduces a dual-backend LLM abstraction in
`crates/nexus-worker-core/src/llm`. Operators pick the backend
in `worker.toml` :

```toml
[llm]
backend = "llama_cpp"   # recommended production
# or
backend = "ollama"      # development / fallback
```

#### Default build — Ollama only

```bash
cargo build --release -p nexus-worker
```

Compiled worker talks HTTP to `localhost:11434` (Ollama daemon).
Zero extra build dependency. Schema enforcement via
Ollama v0.5+ `format` param.

Prereq for runtime : install Ollama from
`https://ollama.com/download` and run `ollama serve`.

#### Production build — in-process llama.cpp + llguidance

```bash
# Linux (Ubuntu / Debian)
sudo apt-get install -y cmake build-essential libclang-dev
# macOS
brew install cmake llvm
# Windows
# 1. Install Visual Studio Build Tools 2022 (C++ workload)
# 2. Install LLVM from https://github.com/llvm/llvm-project/releases
# 3. Set env var LIBCLANG_PATH=C:\Program Files\LLVM\bin
# 4. Install cmake (winget: winget install Kitware.CMake)

# With CPU-only inference
cargo build --release -p nexus-worker --features llm_llama_cpp

# With CUDA 12.6+ (RTX 40xx / 50xx)
cargo build --release -p nexus-worker --features llm_llama_cpp_cuda

# With Apple Metal (M-series Macs)
cargo build --release -p nexus-worker --features llm_llama_cpp_metal

# With Vulkan (cross-vendor GPU)
cargo build --release -p nexus-worker --features llm_llama_cpp_vulkan
```

Then point `[llm.llama_cpp] model_path` at a GGUF file on disk :

```toml
[llm]
backend = "llama_cpp"

[llm.llama_cpp]
model_path = "~/.nexus-grid/models/qwen2.5-7b-instruct-q4_k_m.gguf"
n_ctx = 4096
n_gpu_layers = -1
n_threads = 0
```

Test the backend :

```bash
nexus-worker stats    # healthcheck path prints GGUF ready / not running
nexus-worker start    # full engine boot
```

#### Fallback semantics

If `backend = "llama_cpp"` in `worker.toml` but the binary was
built **without** `--features llm_llama_cpp`, the worker refuses
to boot with :

```
unsupported llm backend "llama_cpp": feature "llm_llama_cpp" is
not compiled in this binary
hint: rebuild with `cargo build --features llm_llama_cpp` or set
`[llm] backend = "ollama"` in worker.toml
```

Rationale : we prefer a loud startup failure over a silent
fallback because the operator explicitly asked for the in-process
backend.

#### Security remark — grammar ≠ prompt-injection defense

Both backends enforce JSON Schema on the LLM output. This
guarantees the signature chain never signs garbled JSON, but does
**not** defend against prompt injection producing schema-valid
malicious content. See `docs/rust/PATTERNS.md §P30` for the full
warning and the layered defense roadmap (Sprint 21 client-side
redaction, Sprint 22 tool-calling sandbox).

Reference : design doc
`.planning/research/S20_phase_D_structured_output_design.md`,
plan § 7 in `.planning/active/sprint20_plan.md`.

### P31 — Warrant canary federation foundations (Sprint 20 Phase E)

Sprint 20 Phase E (G8 pivot Option C, codified
2026-04-18) introduces the **federation primitives** for the
warrant canary signing surface. From the shell daemon
perspective :

- The Rust crate `nexus-shell-daemon-core::canary` is now a
  **module directory** (was a single file) holding the
  `CanarySigner` trait + `Ed25519CanarySigner` (S18 baseline,
  default) + `FrostCanarySigner` (K-of-N opt-in for cross-
  juridiction maintainer federation, RFC 9591 jan 2025) +
  `DuressAck` (separate gossip topic for daily-cadence
  anti-coercion signal) + `AttestationProvider` /
  `NoopAttestation` (decoupled from signing, prep TEE Sprint
  25-30).
- The Python coordinator package adds a `CanaryRegistry`
  aggregator + `GET /api/canary/network-health` endpoint that
  the React shell can render as a fleet freshness panel.
- The full strategy + L0..L1..L2 ladder lives in
  `docs/security/WARRANT_CANARY_HARDENING.md`.
- The Rust pattern detail lives in `docs/rust/PATTERNS.md §P31`
  (CanarySigner trait + FROST + Federated registry pattern).

The federation does not introduce any way to **automate** canary
signing — every canary signature still requires a human (Niveau
0) or K humans cooperating in a synchronous interactive FROST
ceremony (Niveau 1, S25-30 enforcement). This honours the
S18 E2 decision (commit `04c9621`) that any key-accessible-to-a-
scheduler ≡ key-compromise-under-gag-order ≡ dead-man-switch
broken.

### P32 — Transport probe = observability-only (Sprint 20 Phase E.6 ajusté)

The S20 G8 phase pre-flight S1 scan discovered that iroh 0.91+
has no `relay_wss_only` flag — relays speak WSS-over-TCP-443
exclusively since 0.91 (`iroh-0-91-0-the-last-relay-break`
blog post), and the fall-back from a failed UDP QUIC hole-punch
to a relay-WSS path is automatic. The
`crates/nexus-shell-daemon-core/src/transport_probe.rs` module
is therefore deliberately **observability-only** : it dials up
to N attempts, emits a structured `tracing::warn!` with a
`transport_degraded_mode = true` field on failure, and never
touches `iroh::Endpoint` to mutate the relay mode.

Operator picks up the metric via log shipper / dashboard filter
on `transport_degraded_mode=true` to surface "this daemon is
running on the relay-WSS fallback path" — the data plane keeps
working because iroh handled the fallback automatically. See
`docs/rust/PATTERNS.md §P32` for the Rust-side detail.

---

## Sprint 53 patterns

### P34 — Daemon JSON routes namespaced under `/api/daemon/`

**Rule**: daemon-specific JSON routes live under `/api/daemon/*`
(`/api/daemon/info`, `/api/daemon/browse`, `/api/daemon/curators`,
etc.). SPA document routes like `/browse` and `/curators` are
React Router paths served by the `ServeDir` fallback when
`--web-root` is active. Mixing bare JSON routes and SPA paths
caused 401 on F5/direct navigation (the auth middleware rejected
the document request before React could bootstrap the token).

The namespace split:
- `/api/daemon/*` — daemon JSON, requires `x-sbfb-token`
- `/api/v1/*` — coordinator JSON, requires `x-sbfb-token`
- `/api/canary/*` — FROST/canary JSON, requires `x-sbfb-token`
- `/health`, `/auth/token` — public, no token
- `/blob-serve/{hash}/*` — public, separate CSP middleware
- `/*` — SPA fallback (when `--web-root`)

SHA: Sprint 53 Phase A. Files: `crates/nexus-shell-daemon/src/http.rs`,
`web/src/api/daemon.ts`.

---

## Sprint 38 patterns

### P33 — rowid tiebreaker in kudos ORDER BY queries

SQLite `created_at` is stored as seconds (INTEGER). When two kudos
entries share the same second (burst credit, tests), `ORDER BY
created_at` alone is non-deterministic. Both `get_last_entry_hash`
and `get_project_entries` add `, rowid ASC/DESC` as tiebreaker.
`rowid` is SQLite's implicit auto-increment and reflects insertion
order within the same connection. The hash-chain depends on
deterministic entry ordering (prev_hash = previous entry_hash), so
the tiebreaker is a correctness invariant, not a preference.

---

## Sprint 24 patterns

### PyO3 wheel rebuild procedure

The coordinator tests import `nexus_core` (PyO3 bindings from
`crates/nexus-core-py`). When the Rust API evolves (new binding,
changed signature), the installed wheel in `.venv/` becomes stale
and tests fail with `AttributeError: module 'nexus_core' has no
attribute '<name>'`.

Rebuild procedure:

```bash
unset CONDA_PREFIX CONDA_DEFAULT_ENV
VIRTUAL_ENV=$PWD/.venv maturin develop --release \
    --manifest-path crates/nexus-core-py/Cargo.toml
```

After rebuild, re-run coordinator tests:

```bash
uv run pytest packages/nexus-coordinator/tests/ -q
```

The 32 stale failures (pre-existing since Sprint 23 Phase F)
trace back to `sign_bytes` binding rename. This is an environment
issue, not a code regression — the source code in `nexus-core-py`
is correct.

## Sprint 71 patterns

### P35 — Sprint 71 Phase C : Factory Operator server loopback hardening

The Factory **Operator** HTTP server (`crates/sbfb-factory/src/
operator_server.rs`) writes files and **spawns agent subprocesses**
(`claude --permission-mode bypassPermissions`). The off-sprint block
shipped it with **CORS `Any` and zero auth** (G7/P1) and an **SSE
stream that bypassed the `SENSITIVE_ACTIONS` gate** the JSON endpoints
already enforced (G2/P0). Phase C (D3/D4/D5/D6) brings it under the
same loopback model as the daemon (P27), scaled to the token+Host+Origin
subset (no UDS / peer-creds — the Operator is TCP loopback only):

- **Auth** : a per-boot token (`sbfb-factory/src/auth.rs`, mirrors
  `daemon_client.rs:64-65`) is required via `X-SBFB-Token`; `Host:` must
  be loopback; `CorsLayer` is pinned to the known local origin (no more
  `allow_origin(Any)`). Same threat (DNS rebinding / CSRF on a
  write+spawn surface) and same defense rationale as **P27** — read it
  there, not duplicated here. `constant_time_eq` on the token compare.
- **SSE gate** : `handle_chat_stream` now runs the *same*
  `SENSITIVE_ACTIONS` filter as `handle_chat_message` / `handle_chat_
  send`. A last user message carrying `shell` / `commit` / `push` /
  `PASS` returns `requires_gate` instead of spawning an autonomous
  `bypassPermissions` agent. `bypassPermissions` is **kept** (PO-2: the
  "base prompt + autonomous discussion" mode is contract, not a bug) but
  never on an ungated path. Non-sensitive messages still stream
  (`sse_allows_nonsensitive`).
- **Model** : the SSE spawn no longer hardcodes `"sonnet"` (G9, violates
  the model rule `feedback_model_46`); it reads `ChatSendRequest.model`
  with default `claude-opus-4-8[1m]`.
- **Spawn safety** : `spawn_claude_stream` (`llm_bridge.rs`) gained a
  configurable timeout (subprocess killed if exceeded) and a pre-spawn
  resolution check emitting a clear "claude CLI not found in PATH"
  diagnostic instead of an opaque `Failed to spawn`.

Threat boundary (D5 ⚠️) : token+Host defends CSRF / DNS-rebinding from a
browser, **not** a hostile local process that can read the token — the
same accepted model as the daemon loopback (node-level OS sandbox, not
HTTP-server-level).

Tests (`crates/sbfb-factory`) : `auth::tests::*` (5), `server_rejects_
missing_token`, `server_rejects_foreign_host`, `cors_restricts_origin`,
`token_request_succeeds`, `sse_gates_sensitive_action`,
`sse_allows_nonsensitive`, `chat_stream_uses_opus_model`,
`llm_bridge::tests::spawn_times_out`, `missing_claude_diagnostic`.

Contract: `docs/agent/RRV_FACTORY_CONTRACT.md §4` amended to authorize
the **gated** privileged local agent pilot explicitly (PO-2).

Cross-ref: **P27** (daemon loopback hardening), S71 Phase C (`a0337c6`),
preflight SCOPE-CUT-CONSISTENT, kickoff §5 D3/D4/D5/D6.
