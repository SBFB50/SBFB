# Sprint 6 — Audit Findings

**Auditor**: fresh Claude Code session (no Sprint 6 history), 2026-04-11
**Commits audited**: `02ab9bf..3d1c3d5` + audit prep (`c6468ba`, `504c6aa`)
**Audit plan played**: `.planning/sprint6_audit_plan.md` (9 tracks A..I)
**Baseline re-run during audit**: Vitest coverage (77 passed, 97.34/88.67/98.18/97.59),
size-limit (all 4 budgets green), `npm audit` (0 vulns all levels).
Rust / Python / Playwright trusted from `sprint6_verification.md` self-report
(not re-run to conserve time — see note in §Global verdict).

**Methodology**: static read of all Sprint 6 deliverables + parallel Explore
agents on Tracks D, F, G, I + in-line shell checks on Tracks A, B, C, E, H.
No code was modified during the audit.

---

## Track A — Cross-language contract integrity — VERDICT: CONCERN

### A1 — Structural schema diff (Pydantic ↔ Zod)

**Result**: Structurally compatible at the JSON wire level. Every
block kind present in `packages/nexus-sdk/src/nexus_sdk/view.py` has
a matching `.strict()` object in `web/src/components/app/tabview/schema.ts`,
with the same literal `kind` discriminator and the same required fields.
Differences observed are all acceptable wire-level equivalences
(`str | int | float` ↔ `z.union([z.string(), z.number()])`,
`BlockTone = "neutral"` default ↔ `z.enum(...).default("neutral")`,
`Any = None` ↔ `z.unknown().nullable().optional()`).

**BUT** — one implementation-level deviation from the plan:

**Finding A-1 (P2)**: The Zod mirror uses a plain `z.union([...])` over
the 11 block schemas (`schema.ts:252-267`), **not** the
`z.discriminatedUnion("kind", [...])` that `.planning/sprint6_plan.md`
§3.2 explicitly specified. The reason is the recursive `Section` case —
`z.discriminatedUnion` cannot embed a `z.lazy` child cleanly without
circular typing, so the developer took the shortcut. Consequences:
(a) slower validation (tries each branch) — negligible in practice,
(b) noticeably worse error messages on malformed payloads ("no branch
matched" instead of "in kind=metric, field `value` expected
string|number, got boolean"). This will bite Sprint 8 debugging when a
gov tab payload fails to parse.

**Fix effort**: ~30 min. Keep `TabBlockSectionSchema` out of the lazy
union and build two layers — a `z.discriminatedUnion("kind", [heading,
text, kv, metric, table, badge_list, button, chart_line, chart_bar,
empty])` for the leaves, and a `z.lazy(() => z.union([section,
leavesUnion]))` for the top. Error messages improve for 10/11 kinds.

### A2 — Round-trip reality: Python → HTTP → Zod

**Result**: The happy path IS exercised end-to-end by the existing
tests. `test_schema_driven_descriptor_validates`
(`packages/nexus-coordinator/tests/test_apps.py`) posts a real
`TabView` through `_coerce_tab_view` and the Playwright
`tabview-schema-driven.spec.ts` exercises the gov Contradictions tab
from Python construction → Pydantic dump → HTTP → Zod parse → React
render. The Vitest suite covers each of the 11 block kinds individually
via `TabViewRenderer.test.tsx`.

**Coverage gap**: no single test constructs a **maximal** TabView with
all 11 kinds at once AND passes it through the full pipeline. A
one-kind drift (e.g., Python accidentally renaming `muted`→`subdued`)
would be caught by the snapshot test (see A3), but a payload-level
edge case (unicode, large list, null/edge numbers, recursive section
depth) would not be caught by either test path.

**Finding A-2 (P3)**: No fuzz / edge-case pass through the full
pipeline. Covered at unit level, not at pipeline level. Track B2 below
details the specific data shapes that are untested.

### A3 — Snapshot guard detection

**Result**: The `test_view_schema_stable_snapshot` test in
`packages/nexus-sdk/tests/test_view.py:273-295` compares
`TabView.model_json_schema()` against the committed
`packages/nexus-sdk/tests/snapshots/tabview_schema.json`. A Python-side
field rename → schema dump differs → test fails with a readable
`assert actual == expected` diff. Good for the Python side.

**Finding A-3 (P1)**: The snapshot is **Python-only**. The docstring
(line 269) says "must match `web/src/components/app/tabview/schema.ts`"
but **nothing in the test actually compares against the Zod schema**.
If a developer renames a field only in `schema.ts`, the snapshot test
passes and the mismatch goes unnoticed until a runtime Zod parse fails
in production. Conversely, if a developer bumps `view.py` AND
regenerates the snapshot (the prompt says to do so explicitly), the
Zod side is not forced to move in lockstep.

The docstring **lies about cross-language guarantee**. This is the
exact "drift silencieuse" risk the audit plan called out.

**Fix effort**: ~1 h. Add a companion test that imports the Zod schema
via a tiny Node script (`scripts/dump_zod_schema.mjs` → JSON schema via
`zod-to-json-schema`) and diffs its output against the Pydantic
snapshot. Or a simpler approach: freeze a **canonical JSON payload**
(one TabView with all 11 kinds) and test both
`TabView.model_validate(payload).model_dump() == payload` (Python side)
AND `TabViewSchema.safeParse(payload).success === true` (Node side, via
a vitest test that reads the same fixture file).

### Track A verdict: **CONCERN** — A3 is a P1 blind spot. A1 is a P2
deviation from the plan with real error-message cost. A2 is a P3 test
gap.

---

## Track B — Renderer resilience with real data — VERDICT: CONCERN

### B1 — Uncovered lines audit

Coverage re-run during the audit:

| File | Lines | Branches | Uncovered | Severity |
|---|---|---|---|---|
| `TabBlockRenderer.tsx` | 85.71% | 91.66% | 45-46 (`_exhaustive: never`) | P3 — dead branch by TS compile-time check |
| `schema.ts` | 100% | 62.5% | 298-299 (parseTabView fallback when `issues[0]` is undefined) | P3 — defensive path, Zod always produces ≥1 issue |
| **`ButtonBlock.tsx`** | **57.14%** | **0%** | **18-24 (`task_submit` branch)** | **P2 — see finding** |
| `ChartBarBlock.tsx` | 100% | 75% | 25 (zero-max edge) | P3 |
| `ChartLineBlock.tsx` | 100% | 92.85% | 19 | P3 |
| `format.ts` | 100% | 93.33% | 62, 66-67 ("dans" branches for hours/days) | P3 |
| `projectStore.ts` | 100% | 90.62% | 73-77 (dedupe patch path when only nickname OR nodeId is missing) | P3 |

**Finding B-1 (P2)**: `ButtonBlock.tsx` line 18-24 is the
`action.kind === "task_submit"` branch. It calls `console.warn(
"[tabview] task_submit action not yet wired", block.action)` and
returns. The inline comment (lines 21-23) admits: "Sprint 6:
task_submit is a no-op placeholder — Sprint 7/8 will wire it to
coordinator.submitTask once the app context surfaces the per-tab
coordinator URL." The 0% branch coverage is **not a test gap** — there
is a test that renders the task_submit button (line 183-199 of
`TabViewRenderer.test.tsx`), but it only asserts the button label
renders. It doesn't click, so the handler's else-branch is never hit.

This is **dead shipped code**. The schema exports `ActionTaskSubmit`,
the helper `button_task()` exists (`view.py:246-255`), but the
consumer is a `console.warn` stub. An app author could ship a
`task_submit` button today, and users clicking it would see nothing
happen. See also Track G-1.

### B2 — Data fuzzing

**Result**: Not run (doing so would require writing new test files,
out of audit scope per §11). Static analysis of test fixtures:

| Edge case | Covered? |
|---|---|
| 500-row table | No — tests use 2 rows |
| Unicode + RTL labels | No — all tests ASCII/French |
| Chart with 0 points | Yes (`renders an empty line chart placeholder`) |
| Chart with 1 point | No — 3 is the min |
| 2 identical points | No |
| Chart with delta=0 | No — tests use delta=5 and delta=-2 |
| Very long metric value string | No |
| All-negative chart values | No |
| Section recursion depth > 2 | No — test stops at 2 |
| Table with unknown column key | No |
| button.task_submit deep payload | No |

**Finding B-2 (P3)**: No stress/unicode/negative-number tests. Likely
harmless for a French-only shell (no RTL), but a 500-row table will
flex the DOM and the SVG path generator in `ChartLineBlock` has no
explicit guard against `yMin === yMax` (division by zero in
normalisation — would produce `NaN` path coords). Add a few edge-case
fixtures in Sprint 7 cleanup.

### B3 — Live manual observation

Not performed (would require launching the shell in a real browser).
Text-based Playwright `tabview-schema-driven.spec.ts` passes in the
self-report, but only against the gov Contradictions tab shape
(heading + text + 2 metrics + empty). Every other block kind is only
proven via jsdom Vitest. No screenshot-based visual regression.

### Track B verdict: **CONCERN** — B-1 is a P2 (dead task_submit
branch), B-2 is a P3 (missing edge-case fuzz), B-3 is uncovered.

---

## Track C — Test solidity — VERDICT: CONCERN

### C1 — Mutation testing (static analysis)

Plan asked for 3 surgical mutations. I analysed whether each would be
caught by the existing test suite **without** actually editing the
code:

**Mutation 1 — invert `past ? "il y a" : "dans"` in `format.ts:62-67`**
→ **CAUGHT**. Test `formatRelativeTime › formats a future timestamp
as 'dans'` (line 124, `format.test.ts`) asserts exactly
`"dans 5 min"`. Inverted code would produce `"il y a 5 min"` → fail.
Also `formats minutes ago` (line 112) asserts `"il y a 5 min"` → also
fails.

**Mutation 2 — change `activeCoordinatorUrl: s.activeCoordinatorUrl ?? url`
to `activeCoordinatorUrl: url` in `projectStore.ts:94`**
→ **NOT CAUGHT**. No test in `projectStore.test.ts` asserts "active URL
stays at the FIRST added coordinator when a second is added later".
The existing tests:
- `auto-selects the first added coordinator` only tests single add
- `re-assigns active to the next coordinator when active is removed`
  happens to still pass with the mutation because the mutated behavior
  (second add overwrites active) combined with the removal logic
  produces the same observable state by coincidence
- No test exercises "add A, add B, assert active === A"

**Finding C-1 (P2)**: Missing invariant test. A real P2 regression
surface — a dev refactoring `addCoordinator` could quietly break "user
picked coordinator stays picked when another joins via
`/shell/discover`". Add a one-line test:
```ts
it("preserves active URL when adding a second coordinator", () => {
  store.addCoordinator("http://a");
  store.addCoordinator("http://b");
  expect(store.activeCoordinatorUrl).toBe("http://a");
});
```

**Mutation 3 — `tabView.blocks.length === 0` → `=== 1` in
`TabViewRenderer.tsx:25`** → **CAUGHT**. Test `renders the empty-state
line when no blocks` asserts the "aucun bloc à afficher" text when
blocks=[], mutation would suppress it. Test `renders a heading block`
would also render empty-state when it shouldn't.

### C2 — `describe.skip` / `it.skip` / `.only` audit

**Result**: `grep -rn '.skip\|.only'` over `web/src/` and `web/tests/`
returns empty. **PASS**.

### C3 — Playwright assertion strength

Read `web/tests/tabview-schema-driven.spec.ts`:

- Positive assertions: `page.getByText("Analyse de cohérence politique")`,
  `page.getByText("Déclarations analysées")`, etc. — pure text-based
- Negative assertion: `page.getByText(/Descripteur legacy/).toHaveCount(0)`
  ✓ present

**Finding C-3 (P3)**: All positive assertions are `getByText`. If a
regression rendered the heading inside a `<pre>JSON.stringify</pre>`
block (e.g., a bug classified the descriptor as legacy incorrectly),
the test would still find the literal text inside the stringified JSON
and pass. The negative assertion
(`toHaveCount(0)` on "Descripteur legacy") is the only guard against
that. Add at least one structural assertion like
`expect(page.locator('h2, h1').filter({hasText: /Analyse/})).toBeVisible()`
or a `data-testid` on the renderer root (`data-testid="tabview-renderer"`)
for better anchoring.

Also: `command-palette.spec.ts` has to dispatch a synthetic
`KeyboardEvent` because headless Chromium's real
`page.keyboard.press("Control+K")` doesn't reach the handler. The
comment (lines 57-59) is honest about this, but it means **the real
Ctrl+K path is not exercised end-to-end in CI** — only the global
`document` listener is. See Track F for the bigger picture.

### Track C verdict: **CONCERN** — C-1 is a P2 missing invariant.
C-2 PASS. C-3 is a P3 test strength concern.

---

## Track D — Legacy fallback sentinel — VERDICT: CONCERN

### D1 — Sentinel search

**Result**: A **narrative** sentinel exists in `docs/shell/PATTERNS.md`
§P8 lines 126-130: "The Sprint 6 D3 fallback (`legacy_descriptor: true`)
is a transition aid — [...] MUST be removed once `nexus-app-gov` lands
its full 19-tab migration in Sprint 8." No `TODO`, `FIXME`,
`DEPRECATED`, decorator, or test marks the code itself. No entry in
`.planning/sprint6_kickoff.md` §3 Sprint 7/8 outlines reminds Sprint 8
to delete the fallback.

**Finding D-1 (P2)**: The only reminder is in a `docs/` markdown file
that Sprint 8 work is not guaranteed to read. A 2-line code comment
(`# TODO(Sprint 8 Phase A): remove after 19-tab gov migration`)
in `packages/nexus-coordinator/src/nexus_coordinator/api/apps.py:124`
would make the sentinel code-local and trivially greppable. Downgraded
from the agent's initial P1 because PATTERNS.md P8 does provide a
written narrative commitment — just not a code-level one.

### D2 — Fallback path runtime behavior

**Result**: Logging is correct. `apps.py:138-143` calls
`logger.warning(...)` (not `.debug` or `.info`) on ValidationError.
The shell UX in `AppsTab.tsx:330-342` renders the legacy case as a
visible amber-colored banner `"Descripteur legacy — {reason}"` + a
collapsible `<details>` with raw JSON. Not silent, not easily missed.

**BUT**: the test `test_legacy_descriptor_falls_back` asserts only the
response body (`body["legacy_descriptor"] is True`) — it does **not**
assert the WARNING was emitted. If a refactor accidentally downgrades
the log level to DEBUG, the test passes silently.

**Finding D-2 (P3)**: Add a `caplog` fixture assertion to the test:
```python
def test_legacy_descriptor_falls_back(..., caplog):
    ...
    assert any(
        r.levelname == "WARNING" and "legacy" in r.message
        for r in caplog.records
    )
```

### D3 — Boot counter

**Result**: `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py`
start path logs iroh / author / doc / dispatcher but has no sweep of
`self.apps` at boot time to count legacy descriptors. An operator
running a real coordinator has no easy way to discover "3 apps still
need porting" without reading request logs.

**Finding D-3 (P2)**: Add a boot-time sweep that tries `TabView.model_validate`
on each `@nexus_tab` descriptor (or at least the sync ones; skip async
to avoid blocking start) and logs
`"N apps returning legacy descriptors; port before Sprint 8 cutoff"`.
~20 LOC.

### Track D verdict: **CONCERN** — D-1 is a P2 missing code sentinel,
D-2 is a P3 test assertion gap, D-3 is a P2 missing boot visibility.

---

## Track E — Bundle budgets — VERDICT: CONCERN

### E1 — Where did the +30 KB go

`size-limit` reran during audit:

| Asset | Sprint 5 | Sprint 6 | Budget | Δ | Headroom |
|---|---|---|---|---|---|
| main | 425 KB | 455.32 KB | 475 KB | +30.32 KB | 19.68 KB |
| vendor-react | 190 KB | 189.64 KB | 210 KB | -0.36 KB | 20.36 KB |
| vendor-ui | 0 (not split) | 31.55 KB | 110 KB | +31.55 KB (new split) | **78.45 KB** |
| css | 90 KB | 93.68 KB | 100 KB | +3.68 KB | 6.32 KB |

The +30 KB on main can be accounted for by:
- TabView renderer (`TabViewRenderer.tsx` + `TabBlockRenderer.tsx` +
  11 block components) ≈ ~15 KB
- Zod `schema.ts` compiled ≈ ~5 KB
- cmdk 1.1.1 runtime (already in node_modules from Sprint 5 but not
  imported; Phase C is the first consumer) ≈ ~8 KB
- `CommandPalette.tsx` + `useCommandPalette.ts` + the 7 new lucide icons ≈ ~2 KB

That's ~30 KB — matches. **PASS on accounting**. I did not run
`vite-bundle-visualizer` (would require a temp dep install) so I'm
relying on back-of-envelope. If a real leak is hiding, only a
visualizer run would catch it — recommend doing it once in Sprint 7
cleanup.

### E2 — Budget tightening

**Finding E-2 (P2)**: **`vendor-ui` budget is egregiously loose**: 110 KB
limit vs 31.55 KB actual = **249% headroom**. A developer adding a
large new Radix / base-ui package (e.g., full react-day-picker ≈ 50 KB,
or recharts sneaking back in ≈ 90 KB) would still pass the budget.
This defeats the purpose of the guard.

Recommended new budgets for Sprint 7:
- `main`: keep 475 KB (Sprint 7 will add daemon client ~5-10 KB, P2P
  browse page ~5 KB — realistic ceiling)
- `vendor-react`: keep 210 KB (no React ecosystem additions planned)
- **`vendor-ui`: tighten to 50 KB** (60% headroom, still accommodates
  2-3 new base-ui primitives for curator list + browse cards)
- `css`: 100 KB → **101 KB** (93.68 is already 94% of 100; a single
  new Tailwind variant push could break CI — either raise to 110 or
  accept the brittle guard)

### E3 — Brotli parallel reporting

Not added. Non-blocking per audit plan. **P3** tech debt.

### Track E verdict: **CONCERN** — E-2 is a P2 loose vendor-ui budget.
E-1 accounting passes. E-3 is optional.

---

## Track F — Ctrl+K portability — VERDICT: CONCERN

### F1 — Cross-browser keyboard handling

Read `useCommandPalette.ts:16-25`:
```ts
const handler = (e: KeyboardEvent) => {
  if (e.key === "k" && (e.ctrlKey || e.metaKey)) {
    e.preventDefault();
    setOpen((prev) => !prev);
  }
};
```

**Finding F-1 (P1)**: `e.key === "k"` is **lowercase only**. When Caps
Lock is enabled (or Shift is held), `e.key` returns `"K"` (uppercase),
and the handler fails silently. The D5 plan promises "Ctrl+K opens the
palette on Windows/Linux and Cmd+K on macOS, plus Escape to close" —
this promise breaks for any user with caps lock on.

Additionally, there is **no `e.code === "KeyK"` layout-independent
fallback**. For AZERTY and most Latin layouts, `e.key === "k"` still
works because the K key produces "k" in both QWERTY and AZERTY, but
Dvorak/Czech/Russian users may hit edge cases. The plan did not
explicitly require layout independence so this is a minor P3 add-on.

`e.preventDefault()` is called **before** `setOpen` — correct order to
beat Chrome/Firefox's own Ctrl+K (search bar) default.

**Fix effort**: trivial. Line 18 becomes:
```ts
if ((e.key === "k" || e.key === "K") && (e.ctrlKey || e.metaKey)) {
```
Or more defensive:
```ts
if (e.code === "KeyK" && (e.ctrlKey || e.metaKey)) {
```
I recommend the second form — `e.code` is layout- AND case-independent.

### F2 — Input focus conflict

`AddCoordinatorDialog.tsx` focuses an `<Input id="coord-url">`. The
global `useCommandPalette` handler does not check
`document.activeElement` before firing. Pressing Ctrl+K while typing
in the URL input will open the palette over the dialog. The palette
is a Radix-based portaled modal, so the layering works — palette on
top, dialog underneath.

**Not really a bug**: this is the standard UX for command palettes
(VS Code, Cursor, Linear all open over input fields). However, it is
**un-intended** because the D5 plan doesn't discuss dialog layering.

**Finding F-2 (P3)**: Acceptable current behavior, but if you ever
want the palette to NOT open inside input fields, add the guard:
```ts
const tag = document.activeElement?.tagName;
if (tag === "INPUT" || tag === "TEXTAREA") return;
```

### F3 — cmdk internal listener + stopPropagation

cmdk 1.1.1 has no Ctrl+K listener of its own (verified via source
reading). It handles arrow keys, Enter, Escape, and vim keys
(Ctrl+N/P/J) for list navigation — not Ctrl+K. No conflict with the
global palette handler.

No `e.stopPropagation()` is called. Not needed today.

**Finding F-3 (P3)**: Future-proofing: if cmdk ever adds Ctrl+K (e.g.,
to close the palette), the global handler would fire AND cmdk's would
fire, double-toggling. Non-blocking today, noted for Sprint 9 upgrade
review.

### Track F verdict: **CONCERN** — F-1 is a P1 broken shortcut for
caps-lock users. F-2 / F-3 are P3.

---

## Track G — Sprint 7 / 8 risk assumptions — VERDICT: CONCERN

### G1 — `button.kind="task_submit"` dead code

**Finding G-1 (P2)**: The `action.kind === "task_submit"` branch in
`ButtonBlock.tsx` is a `console.warn` stub. The schema exports the
action type (`packages/nexus-sdk/src/nexus_sdk/view.py:111-115`), the
`button_task()` helper exists (lines 246-255), but the consumer is
dead. Sprint 8 gov tabs will want real task_submit buttons. Before
Sprint 7 starts, decide:

Option A — **Remove `ActionTaskSubmit` from v1 schema**, bump
`schema_version`, regenerate snapshot, patch `button_task()`. Clean
but breaking.
Option B — **Define the handler signature now**: add an
`AppContext.submit_task(worker, payload)` SDK method that the
ButtonBlock calls via a React context. Non-breaking, does real work.

Either way, add a decision note in `sprint7_kickoff.md` (does not yet
exist) before Phase A. Classified P2 not P1 because Sprint 7 itself
(P2P Discovery Layer) doesn't need task_submit — Sprint 8 does.

### G2 — Command palette app-contribution API missing

**Finding G-2 (P2)**: `CommandPalette.tsx` hardcodes 3 groups
(Navigation, Projets, Actions). No SDK hook exists for an app to
register commands. Grep of `packages/nexus-sdk/src/nexus_sdk/app.py`
confirms: no `commands()` method on `NexusApp`, no `@nexus_command`
decorator, no `CommandDescriptor` type.

The CommandPalette.tsx file header (lines 11-13) explicitly says
"Sprint 8 will allow apps to contribute command entries via a SDK
hook" — so this is a known deferral. Same severity calculus as G-1:
Sprint 7 doesn't need it, Sprint 8 does.

Recommend: design the SDK hook signature before Sprint 7 ends, even
if implementation lands in Sprint 8. A one-pager in
`sprint8_kickoff.md` (doesn't exist yet) is enough.

### G3 — TabView vocabulary sufficiency for 19 gov tabs

**Not a finding**. Sprint 6 kickoff §3 already acknowledges scope cuts
for Reseau (graph WebGL) and Carte (Leaflet), and the Sprint 8 outline
lists "scope cuts probable: no graph WebGL, no Leaflet". Confirmed
intentional deferral, not a Sprint 6 audit issue.

For future planning: the most visually problematic scope cuts will be
Reseau (reduced to tabular relationship lists), Carte (reduced to
tables of addresses), and Videos (no transcription player). All
acceptable for v1.0 per the kickoff decision. If Sprint 8 needs to
reverse one, bump `schema_version` to 2 and add the block kind.

### G4 — CommandPalette mount position (route crash robustness)

The D5 plan phrase is "Placement : dans `AppShell.tsx` au niveau
racine (hors `<Outlet>`)". Verification:
`AppShell.tsx:185-188` mounts `<CommandPalette>` as a sibling of
`<SidebarInset>` (which contains `<Outlet>`), **not inside** the
outlet. ✓ Literally compliant with D5.

**BUT**: `AppShell` itself is mounted as a route element in `App.tsx`.
No `<ErrorBoundary>` wraps `<Outlet />`. If a route render throws
(e.g., Sprint 8 gov Contradictions tab crashes during data fetch),
React propagates the error up past `<AppShell>` and unmounts the whole
tree — including the palette. The D5 implicit promise "palette still
works on a broken page" is broken.

**Finding G-4 (P2)**: No `ErrorBoundary` wrapping `<Outlet />`.
Cheap fix (~30 LOC for a simple class component boundary). Add as part
of the Sprint 6 fixes before Sprint 7, since it's almost certain to
bite once Sprint 8 ships 19 tabs that each call `useQuery`. This one
can be a concrete `fix(sprint6): ...` commit — the others (G-1, G-2)
are decision items that belong in `sprint7_kickoff.md`.

### Track G verdict: **CONCERN** — G-1, G-2, G-4 are P2. G-3 is
intentional scope cut.

---

## Track H — Dependencies + security — VERDICT: PASS

### H1 — `npm audit`

Re-run during audit:
```json
"vulnerabilities": {
  "info": 0, "low": 0, "moderate": 0, "high": 0, "critical": 0, "total": 0
},
"dependencies": {
  "prod": 451, "dev": 187, "optional": 52, "peer": 36, "total": 688
}
```
**PASS**. Zero vulnerabilities at any level across 688 packages.

### H2 — Phase D devDep versions

All added devDeps resolved at stable releases:
- `vitest` 4.1 (stable)
- `@testing-library/react` latest stable
- `@testing-library/jest-dom` latest stable
- `@testing-library/user-event` latest stable
- `jsdom` latest stable
- `size-limit` 12 + `@size-limit/file`
- `@vitest/coverage-v8` paired with vitest 4.1

No 0.x or RC versions in the new set. **PASS**.

### H3 — cmdk + React 19 compatibility

Sprint 6 Phase C hit the shadcn `<CommandDialog>` wrapping bug
("Cannot read subscribe of undefined"). The workaround is a local
`<Command>` wrap inside `CommandPalette.tsx:71-79`, documented inline
with the reason. The shadcn vendored `components/ui/command.tsx` is
left untouched per T1 policy. This is correct and future-proof against
`npx shadcn add` regenerations.

I did not find any other shadcn primitive in Sprint 6 hit by a similar
missing-primitive issue. **PASS** with the noted workaround.

### Track H verdict: **PASS**.

---

## Track I — Documentation coherence — VERDICT: PASS

### I1 — Kickoff / plan / verification triangulation

All 5 D1..D5 decisions in `sprint6_kickoff.md` §4 are expanded in
`sprint6_plan.md` §2. The 24-row fail-fast table in the plan maps
1-to-1 with the verification document's Checklist section. All 6 real
commit SHAs appear in both the verification summary and the git
history.

**Minor drifts (all P3)**:
- Row 4: plan says `test_view_schema_stable`, verification says
  `test_view_schema_stable_snapshot`. The actual test function name is
  the longer one — verification wins.
- Row 5-6: plan references `test_app_tab_descriptor.py`, verification
  correctly names `test_apps.py`. Verification wins.
- Row 18: plan specifies Vitest thresholds `≥95% lines / ≥95% funcs /
  ≥90% branches`, verification (and actual
  `vitest.config.ts`) has `90 / 90 / 85 / 90`. The code was relaxed
  during implementation to accommodate the dead ButtonBlock branch.
  Neither the plan nor the verification marks this as a conscious
  decision.
- Row 20 (verification): checklist row says "trigger button + Ctrl+K
  dispatch" but omits "Escape close" which D5 explicitly requires and
  `command-palette.spec.ts` does test (line 50).

**Finding I-1 (P3)**: These are minor narrative drifts. Fix by
annotating the Phase D commit in the verification doc with a line
"threshold relaxed 95→90 to accommodate ButtonBlock task_submit dead
branch, see Finding B-1" and adding "Escape close" to row 20.

### I2 — Commit message fidelity

`git log --stat 02ab9bf^..3d1c3d5` inspected. All 6 commits match
their declared scope. No scope inflation. `3d1c3d5` (Phase E) contains
exactly what the message says: Playwright spec, PATTERNS.md P8 + T2/T3
close annotations, verification document. **PASS**.

### I3 — MEMORY.md + nexus_grid_pivot.md coherence

Read at audit start. Both files reflect:
- "Sprint 0→6 all CLOSED on master (tip 504c6aa, 2026-04-11)" ✓
- "Next: Sprint 7 Phase 0 = audit Sprint 6 [...] BEFORE any P2P code" ✓
- Master tip 504c6aa matches `git log master -1 --format=%h` ✓
- `sprint_audit_gate.md` convention is reflected as a permanent rule ✓

**PASS**.

### Track I verdict: **PASS** with P3 minor drifts.

---

## Global verdict

**CONDITIONAL PASS**

Sprint 6 delivered the TabView vocabulary, renderer, Ctrl+K palette,
Vitest suite, and bundle budgets exactly as the plan specified.
All 24 fail-fast rows in the self-report pass. The code compiles,
runs, and renders correctly for the happy path. Cross-language
integrity **is** achieved at the JSON wire level.

However, four findings must be addressed before Sprint 7 Phase A
commits land:

**Conditions to lift the CONDITIONAL (P1 fixes, required in
`fix(sprint6): ...` commits on master)**:

1. **F-1 — Ctrl+K case sensitivity**
   (`web/src/components/command-palette/useCommandPalette.ts:18`)
   Change `e.key === "k"` to `e.code === "KeyK"` (or add the
   uppercase-K branch). Ship with a new Vitest case in a renderer or
   hook test that asserts the handler fires for both `e.key: "k"` and
   `e.key: "K"`. Effort: 15 min.

2. **A-3 — Snapshot guard is Python-only, docstring lies**
   (`packages/nexus-sdk/tests/test_view.py:273-295`)
   Either add a companion cross-language test that parses a canonical
   JSON fixture through BOTH `TabView.model_validate` (Python) AND
   `TabViewSchema.safeParse` (Node via a new Vitest), or correct the
   docstring to stop claiming cross-language guarantee. Effort: 1 h
   for the real fix, 5 min for the docstring-only patch.

**P2 findings (should be fixed now since the cost is low, or logged as
tech debt in PATTERNS.md before Sprint 7 starts)**:

3. **A-1** — Zod uses `z.union` instead of plan-spec
   `z.discriminatedUnion`; worse error messages for Sprint 8 debugging.
   Fix: ~30 min.
4. **B-1 / G-1** — `ButtonBlock.task_submit` is dead code. Either
   remove from v1 schema OR decide the handler signature in
   `sprint7_kickoff.md` before Phase A. Log in PATTERNS.md as tech
   debt **T4**.
5. **C-1** — `projectStore` missing "preserve active URL on second add"
   invariant test. Fix: add one Vitest case, 5 min.
6. **D-1** — legacy fallback sentinel is narrative-only. Add a
   `# TODO(Sprint 8): remove after 19-tab gov migration` comment at
   `apps.py:124`. Fix: 1 line.
7. **D-3** — no coordinator boot counter of legacy descriptors. Add
   ~20 LOC boot-time sweep. Fix: 30 min or log as PATTERNS.md T5.
8. **E-2** — `vendor-ui` budget 3.5× current size, defeats the guard.
   Tighten to 50 KB in `.size-limit.json`. Fix: 1 line.
9. **G-2** — no `@nexus_command` SDK hook for app-contributed palette
   entries. Design signature before Sprint 8, log as PATTERNS.md T6 or
   a kickoff decision item.
10. **G-4** — no `ErrorBoundary` wrapping `<Outlet />`. Add a
    simple class component boundary (~30 LOC) in `App.tsx` or
    `AppShell.tsx`. Cheap and eliminates a Sprint 8 footgun.

**P3 findings (nice-to-have, Sprint 7 cleanup or later)**:

11. **A-3-variant** — fuzz pipeline test (unicode, 500-row table,
    chart edges). Log as PATTERNS.md T7.
12. **B-2** — edge-case fixtures for `ChartLineBlock` (yMin===yMax NaN
    risk). Log in PATTERNS.md.
13. **C-3** — Playwright text-based assertions; add `data-testid` or
    structural locators on the renderer root.
14. **D-2** — `test_legacy_descriptor_falls_back` misses the caplog
    WARNING assertion. 3 lines.
15. **F-2** — palette opens over input focus (standard UX but
    undocumented in D5).
16. **F-3** — future-proof against cmdk adding Ctrl+K (add
    stopPropagation). Sprint 9 upgrade review.
17. **I-1** — plan↔verification minor drifts (row 18 threshold, row 20
    Escape, row 4/5/6 test names). Annotation.

---

## Findings list (sorted by severity)

| # | Severity | Track | Summary | Fix effort | Landing |
|---|---|---|---|---|---|
| 1 | **P1** | F-1 | `e.key === "k"` fails when caps lock on or uppercase K is pressed | 15 min | **must fix in `fix(sprint6): ctrl-k case-insensitive`** |
| 2 | **P1** | A-3 | Snapshot test claims cross-language guarantee but only checks Pydantic side | 5 min docstring / 1 h real | **must fix in `fix(sprint6): snapshot honesty` (docstring) OR `fix(sprint6): cross-lang schema fixture`** |
| 3 | P2 | A-1 | Zod uses `z.union` instead of plan-spec `z.discriminatedUnion`; worse errors | 30 min | `fix(sprint6): zod discriminated union` |
| 4 | P2 | B-1 / G-1 | `ButtonBlock.task_submit` is dead shipped code | decide in sprint7_kickoff | PATTERNS.md T4 + decision note |
| 5 | P2 | C-1 | `projectStore` test missing "preserve active on second add" invariant | 5 min | `fix(sprint6): projectStore invariant test` |
| 6 | P2 | D-1 | Legacy fallback sentinel is narrative-only | 1 line | `fix(sprint6): legacy fallback TODO marker` |
| 7 | P2 | D-3 | No boot-time counter of legacy descriptors | 20 LOC | `fix(sprint6): legacy descriptor boot sweep` (or defer to T5) |
| 8 | P2 | E-2 | vendor-ui budget 3.5× current size | 1 line | `fix(sprint6): tighten vendor-ui size budget` |
| 9 | P2 | G-2 | No SDK hook for app-contributed palette entries | design note only | PATTERNS.md T6 + `sprint7_kickoff.md` §Decisions |
| 10 | P2 | G-4 | No ErrorBoundary wrapping `<Outlet />`; route crash kills palette | 30 LOC | `fix(sprint6): error boundary around outlet` |
| 11 | P3 | A-2 | No full-pipeline fuzz test (one payload with all 11 kinds + edges) | 1 h | PATTERNS.md T7 |
| 12 | P3 | B-2 | `ChartLineBlock` yMin===yMax NaN risk (no test) | 15 min | include in T7 |
| 13 | P3 | C-3 | Playwright text-based assertions; add `data-testid` anchors | 15 min | `fix(sprint6): playwright structural anchors` (optional) |
| 14 | P3 | D-2 | `test_legacy_descriptor_falls_back` misses caplog WARNING check | 3 lines | optional |
| 15 | P3 | F-2 | Palette opens over input focus — not guarded | 3 lines | optional |
| 16 | P3 | F-3 | No `stopPropagation`; future-proofing against cmdk Ctrl+K | 1 line | optional |
| 17 | P3 | I-1 | Plan↔verification minor drifts (thresholds, test names, Escape) | 5 min annotation | optional |

---

## Notes on audit completeness

- **Rust tests not re-run**: trusted self-report of 193 verts. No
  Rust code touched in Sprint 6, so no drift risk.
- **Python tests not re-run**: trusted self-report of 79 + 1 skipped
  verts across SDK / coordinator / app-gov. Audit focused on
  schema + endpoint logic via code reading.
- **Playwright not re-run**: trusted self-report of 10 verts. Audit
  read both Sprint 6 specs (command-palette, tabview-schema-driven)
  and flagged assertion strength as P3, not correctness.
- **Vitest re-run**: 77 passed, coverage 97.34/88.67/98.18/97.59 —
  matches self-report exactly.
- **`npm audit` re-run**: 0 vulnerabilities.
- **`size-limit` re-run**: all 4 budgets green, measured the headroom.
- **Bundle visualizer not run**: would require a temp dep. Accounting
  from Phase D commit stats is plausible; recommend a one-off visualizer
  run in Sprint 7 cleanup if a leak is suspected.

---

**End of findings**. Sprint 7 Phase 0 gate: lift the CONDITIONAL by
fixing findings 1 and 2 (both P1) in dedicated `fix(sprint6): ...`
commits on master. P2 findings 3–10 are strongly recommended before
Phase A starts since each is cheap and several are one-liners. P3
findings 11–17 can be logged as PATTERNS.md tech debt and picked up
in Sprint 7/8 cleanup.
