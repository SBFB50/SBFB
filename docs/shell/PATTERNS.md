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

The Sprint 6 D3 fallback (`legacy_descriptor: true`) is a
transition aid — it preserves an unported app's raw dict
for one release only. It MUST be removed once
`nexus-app-gov` lands its full 19-tab migration in
Sprint 8.

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

### T4 — TabView.button task_submit action needs a real consumer

Sprint 6 audit finding G-1 / B-1. The `action.kind === "task_submit"`
branch in `web/src/components/app/tabview/blocks/ButtonBlock.tsx` is
a `console.warn` placeholder. The schema exports the action type,
`nexus_sdk.view.button_task()` constructs it, but nothing runs the
button click. Sprint 6 audit coverage shows ButtonBlock at 57% lines
/ 0% branches because no test ever hits the second branch.

**Decision deferred to Sprint 7 kickoff (phase 0)**. Two mutually
exclusive options:

- Option A — **Remove `ActionTaskSubmit` from v1 schema**, bump
  `schema_version`, regenerate both snapshots + the cross-lang
  canonical fixture, patch `button_task()`. Clean but breaking.
- Option B — **Define the handler signature now** in Sprint 7:
  add an `AppContext.submit_task(worker, payload)` SDK method +
  a React context that the ButtonBlock reads to resolve the target
  coordinator. Non-breaking, does real work.

Either way, the decision MUST land in `sprint7_kickoff.md` §Day 0
Decisions before Sprint 7 Phase A commits. Until then, tab authors
that ship `button_task` expect it to work but get a silent no-op.

Audit reference: `.planning/sprint6_audit_findings.md` §G-1.

### T5 — No SDK hook for app-contributed command palette entries

Sprint 6 audit finding G-2. The command palette hardcodes three
groups (Navigation, Projets, Actions) with no mechanism for an app
to register commands ("Nouveau fact-check", "Rechercher dans les
votes"). Sprint 8 gov v1.1 will want this. The SDK currently has
`NexusApp.routes()`, `.workers()`, `.tabs()` but no `.commands()`.

**Prerequisite for Sprint 8**. Design sketch: add a `@nexus_command`
decorator in `packages/nexus-sdk/src/nexus_sdk/decorators.py`, a
`commands()` method on `NexusApp`, and a coordinator route
`GET /app/{name}/commands` that returns a list of
`{name, description, icon, action}` entries. Mirror on the shell
via a Zod-parsed `CommandDescriptor[]` and an `items.push(...)`
loop in `CommandPalette.tsx` that merges app-provided commands
into the existing groups.

Design spike belongs in `sprint7_kickoff.md` §Decisions so that
Sprint 8 starts with the signature frozen. Implementation lands
in Sprint 8 Phase A alongside the other `AppContext` extensions.

Audit reference: `.planning/sprint6_audit_findings.md` §G-2.

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
