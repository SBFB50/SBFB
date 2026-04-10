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
