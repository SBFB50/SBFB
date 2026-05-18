# Frontend Architecture — React Shell

**Analysis Date:** 2026-05-18

## Overview

The nexus-grid frontend is a **React 19 SPA** that serves as an iframe host for P2P distributed apps. It communicates with a local Rust `nexus-shell-daemon` via authenticated HTTP, manages coordinator connections via Zustand + localStorage, and renders remote web archives in sandboxed iframes with a postMessage bridge. The UI is French-language, dark-theme-only, glassmorphism-styled.

---

## 1. Technology Stack

| Layer | Technology | Version | Config File |
|---|---|---|---|
| Framework | React | 19.2.4 | `web/package.json` |
| Build | Vite | 8.0.1 | `web/vite.config.ts` |
| Language | TypeScript | ~5.9.3 | `web/tsconfig.app.json` |
| CSS | Tailwind CSS v4 | 4.2.2 | `web/src/index.css` (no tailwind.config) |
| UI Primitives | shadcn/ui (base-nova style) + Radix UI | 4.2.0 | `web/components.json` |
| Icons | lucide-react | 1.7.0 | N/A |
| Routing | react-router-dom | 7.14.0 | `web/src/App.tsx` |
| State | Zustand | 5.0.12 | `web/src/stores/projectStore.ts` |
| Data Fetching | @tanstack/react-query | 5.96.2 | `web/src/App.tsx` |
| Schema Validation | Zod | 3.25.76 | Used in every API module |
| Font | Geist Variable | via @fontsource-variable | `web/src/index.css` |
| PII Detection | @huggingface/transformers + onnxruntime-web | 4.0.0 / 1.24.3 | `web/src/sdk/pii/` |
| Unit Testing | Vitest | 4.1.4 | `web/vitest.config.ts` |
| E2E Testing | Playwright | 1.59.1 | `web/playwright.config.ts` |
| Linting | ESLint + typescript-eslint | 9.39.4 | `web/eslint.config.js` |
| Bundle Analysis | size-limit + rollup-plugin-visualizer | 12.0.1 | `web/.size-limit.json` |

## 2. Routing Architecture

**Router:** `createBrowserRouter` (React Router v7, data router pattern)

**Route table** (defined in `web/src/App.tsx`):

```
/                    → Redirect to /my-projects
/my-projects         → Projects page (lazy)
/project/:name       → ProjectDetail page (lazy)
/my-network          → Network page (lazy)
/browse              → Browse page (lazy)
/browse/:projectId   → BrowsedProject page (lazy, full-screen)
/curators            → Curators page (lazy)
/deploy              → Deploy page (lazy)
```

**Layout:** All routes nest under `<AppShell />` which provides:
- Left rail navigation (68px fixed)
- Top bar with coordinator picker + command palette trigger
- `<Outlet />` wrapped in `<RouteErrorBoundary />`
- `<CommandPalette />` (lazy-loaded)
- `<PanicWipeKeybind />` (invisible security component)

**Full-screen mode:** Routes starting with `/browse/` hide the shell chrome. The `AppShell` checks `location.pathname.startsWith("/browse/")` and renders only `<Outlet />` + palette + panic wipe.

**Code splitting:** Every page is loaded via `lazy: () => import("@/pages/X")`. Each page module exports `Component` (named export) for React Router's lazy resolution. The command palette is `React.lazy()` loaded separately.

## 3. Application Shell

**File:** `web/src/components/AppShell.tsx`

The shell renders:
1. **Left rail** (68px) with nav icons: Explorer, Projets, Reseau, Curators, Deployer + "Add coordinator" button at bottom
2. **Top bar** (h-14) with `CoordinatorPicker` dropdown + command palette trigger (Ctrl+K)
3. **Main content** via `<Outlet />`
4. **AddCoordinatorDialog** (modal for adding new coordinators)
5. **CommandPalette** (Ctrl+K global command palette, lazy-loaded)
6. **PanicWipeKeybind** (invisible Ctrl+Shift+Alt+W x5 security wipe)

**CoordinatorPicker** (inline in AppShell): dropdown showing all known coordinators from Zustand store, with health status dot (green/yellow/red). Polls `GET /api/v1/coordinator/health` every 5s.

## 4. State Management

### 4.1 Zustand Store

**Single store:** `web/src/stores/projectStore.ts`

```typescript
interface ProjectStoreState {
  knownCoordinators: KnownCoordinator[];
  activeCoordinatorUrl: string | null;
  addCoordinator(url, opts?): KnownCoordinator;
  removeCoordinator(url): void;
  setActive(url | null): void;
  updateCoordinator(url, patch): void;
  clear(): void;
}
```

**Persistence:** `zustand/middleware/persist` to localStorage under key `"nexus-grid:shell:v1"`. Versioned for future migration.

**Key invariants:**
- URLs are normalized (trailing slash stripped) via `normalizeApiUrl()`
- First added coordinator is auto-selected as active
- Removing the active coordinator selects the next available
- `setActive` throws if URL not in `knownCoordinators`
- Deduplication by normalized URL on add

**Selector:** `selectActiveCoordinator(s)` returns the active `KnownCoordinator` or null.

### 4.2 React Query

**Client:** Single `QueryClient` in `web/src/App.tsx` with `staleTime: 5000, retry: 1` defaults.

**Query patterns per page:**

| Page | Query Key | Endpoint | Interval |
|---|---|---|---|
| Projects | `["health", url]` | `/api/v1/coordinator/health` | 5s |
| ProjectDetail | `["project", url]` | `/project` | stale 10s |
| ProjectDetail | `["tasks", url]` | `/api/v1/tasks?limit=100` | 3s |
| ProjectDetail | `["kudos", url]` | `/api/v1/kudos/entries` | 5s |
| ProjectDetail | `["kudos-verify", url]` | `/api/v1/kudos/{pid}/verify` | 5s |
| ProjectDetail | `["invites", url]` | `/api/v1/invite` | stale 3s |
| ProjectDetail | `["apps", url]` | `/app` | stale 10s |
| Network | `["worker-state", url]` | `/api/v1/worker/state` | 2s |
| Network | `["consent", url]` | `/consent/get` | stale 30s |
| Browse | `["daemon-browse", url]` | `/api/daemon/browse` | stale 30s |
| Curators | `["daemon-curators", url]` | `/api/daemon/curators` | stale 30s |
| BrowsedProject | `["daemon-info", url]` | `/api/daemon/info` | stale 10s |
| CommandPalette | `["palette-apps", url]` | `/app` | stale 30s (enabled when open) |

**Mutations:**
- `subscribeCurator` / `unsubscribeCurator` (Curators page)
- `createInvite` / `revokeInvite` (InvitesTab)
- `deployFromRepo` (Deploy page)
- `addToWhitelist` / `removeFromWhitelist` (BrowsedProject GPU button)
- `setConsent` (GpuConsentDialog)

All mutations invalidate related queries on success.

## 5. API Layer

### 5.1 Authentication

**File:** `web/src/api/auth.ts`

Every HTTP call goes through `authFetch(url, init?)` which injects `X-SBFB-Token: <hex64>` header. Token resolution:
1. Seeded via `window.__SBFB_AUTH_TOKEN` (Playwright E2E)
2. Fetched from launcher at `VITE_SBFB_LAUNCHER_URL/auth/token`
3. Fallback: same-origin `/auth/token`

Cached for page lifetime. If missing, calls proceed unauthenticated (server returns 401, UI shows "daemon not ready").

### 5.2 Coordinator API

**File:** `web/src/api/coordinator.ts`

Generic helpers:
- `getJson<T>(baseUrl, path, zodSchema)` — GET + Zod parse
- `postJson<T>(baseUrl, path, body, zodSchema)` — POST + Zod parse
- `deleteJson<T>(baseUrl, path, zodSchema)` — DELETE + Zod parse

All throw `ApiHttpError` on non-200 or `ApiProtocolError` on Zod failure.

**Endpoints served:** `/api/v1/coordinator/health`, `/project`, `/api/v1/tasks`, `/api/v1/tasks/submit`, `/api/v1/kudos/entries`, `/api/v1/kudos/{pid}/verify`, `/api/v1/invite`, `/api/v1/invite/create`, `/api/v1/invite/{id}`, `/app`, `/app/{name}/manifest`, `/app/{name}/tabs/{tab}/descriptor`, `/app/{name}/tasks/submit`, `/app/{name}/commands`, `/app/{name}/commands/{cmd}/invoke`, `/app/{name}/state/{ns}`, `/api/v1/shell/discover`, `/api/v1/worker/state`.

### 5.3 Daemon API

**File:** `web/src/api/daemon.ts`

Returns `DaemonResult<T>` discriminated union:
- `{ kind: "data", status, body }` — success
- `{ kind: "unavailable", reason }` — network error or 503
- `{ kind: "error", reason }` — other HTTP error

This lets the UI render "daemon offline" as a normal state rather than error boundary crash.

**Endpoints:** `/api/daemon/info`, `/api/daemon/curators`, `/api/daemon/curators/subscribe`, `/api/daemon/curators/{pubkey}`, `/api/daemon/browse`, `/api/daemon/browse/pull`, `/api/daemon/panic/wipe`, `/api/v1/deploy-from-repo`.

### 5.4 Consent API

**File:** `web/src/api/consent.ts`

Manages GPU sharing consent (levels 1-4). Endpoints: `/consent/get`, `/consent/set`, `/consent/whitelist/add`, `/consent/whitelist/remove`.

### 5.5 Zod Schema Convention

**Every API response is validated through Zod before reaching React.** Schemas mirror Rust/Python wire types with `.strict()` to reject unknown fields. Key schemas:

- `HealthSchema`, `ProjectSchema`, `TaskRowSchema`, `KudosEntrySchema` (coordinator.ts)
- `DaemonInfoSchema`, `BrowseEntrySchema`, `CuratorListSchema` (daemon.ts)
- `ConsentConfigSchema` (consent.ts)
- `TabViewSchema` (v1/v2 discriminated by `schema_version`) in tabview/schema.ts
- `BridgeRequestSchema`, `BridgeResponseSchema`, `BridgeEventSchema` (bridge/protocol.ts)

## 6. Page Architecture

### 6.1 `/my-projects` — Projects

**File:** `web/src/pages/Projects.tsx`

Shows `OnboardingEmpty` when no coordinators known. Otherwise displays a grid of `CoordinatorCard` components, each polling `/api/v1/coordinator/health` every 5s. Clicking "Ouvrir" navigates to `/project/{nickname}`.

### 6.2 `/project/:name` — ProjectDetail

**File:** `web/src/pages/ProjectDetail.tsx`

Resolves coordinator by matching `:name` against `knownCoordinators` (by nickname then URL). Five tabs via Radix Tabs:
- **Vue d'ensemble** (`web/src/components/project/OverviewTab.tsx`): stat cards + coordinator identity
- **Taches** (`web/src/components/project/TasksTab.tsx`): task table with status badges
- **Kudos** (`web/src/components/project/KudosTab.tsx`): hash-chain integrity badge + entries table
- **Invites** (`web/src/components/project/InvitesTab.tsx`): invite list + create/revoke flows
- **Apps** (`web/src/components/project/AppsTab.tsx`): app accordion with manifest details + TabView rendering

### 6.3 `/my-network` — Network

**File:** `web/src/pages/Network.tsx`

Polls `GET /api/v1/worker/state` every 2s. Shows worker identity, GPU stats (VRAM/utilization/temp/power), enrolled projects, last task. Includes `ConsentBadge` that opens `GpuConsentDialog` for L1-L4 GPU sharing configuration.

### 6.4 `/browse` — Browse

**File:** `web/src/pages/Browse.tsx`

Netflix-style app browser. Hero section featuring first entry, grid of `AppCard` components with deterministic gradient colors from project name hash. Each card shows status dot, category badge, verified/P2P/source badges. Click navigates to `/browse/{project_id}`.

### 6.5 `/browse/:projectId` — BrowsedProject

**File:** `web/src/pages/BrowsedProject.tsx`

**Full-screen immersive app viewer.** This is the core iframe host:

1. Auto-hide glassmorphism top bar (reveals on mouse near top 48px)
2. Remote app rendered in `<iframe sandbox="allow-scripts">` via `blob-serve` daemon URL
3. Watchdog overlay when app stops sending heartbeats (5s threshold)
4. "Contribuer mon GPU" toggle button (L3 whitelist)
5. Verification detail dialog (provenance SLSA L1)
6. Fallback: local TabView SDK apps for projects hosted on same node

**Key pattern:** The iframe `src` is built via `blobServeUrl(daemonBaseUrl, archiveHash)` which resolves to `http://{host}:{port}/blob-serve/{hash}/index.html`.

### 6.6 `/curators` — Curators

**File:** `web/src/pages/Curators.tsx`

Subscribe/unsubscribe to curator public keys (Ed25519 hex). Validates 64-char hex format client-side. Uses mutations to POST/DELETE subscriptions, invalidates browse queries on change.

### 6.7 `/deploy` — Deploy

**File:** `web/src/pages/Deploy.tsx`

Form for deploying apps from Git repos. Fields: repo URL, project name, description. Calls `POST /api/v1/deploy-from-repo`. Shows success with hash/provenance/commit or error.

### 6.8 OnboardingEmpty

**File:** `web/src/pages/OnboardingEmpty.tsx`

Shown when no coordinators are configured. Step-by-step CLI instructions with copy buttons. Opens `AddCoordinatorDialog`.

## 7. Iframe Host Model

### 7.1 Sandbox Security

Remote apps are rendered in:
```html
<iframe sandbox="allow-scripts" src="http://{host}:{port}/blob-serve/{hash}/index.html" />
```

- `sandbox="allow-scripts"` without `allow-same-origin` gives opaque origin
- No `allow-forms` (forms must use div+button+click handler pattern)
- Blob-serve runs on separate port (7000) for origin isolation
- CSP `connect-src 'none'` for untrusted content (enforced daemon-side)

### 7.2 postMessage Bridge Protocol

**Protocol files:**
- `web/src/bridge/protocol.ts` — Zod schemas for request/response/event/heartbeat
- `web/src/bridge/useBridge.ts` — React hook for host-side bridge listener

**Message types:**

| Direction | Type | Purpose |
|---|---|---|
| iframe -> host | `sbfb-bridge-request` | Method call with UUID correlation |
| host -> iframe | `sbfb-bridge-response` | Reply to a request (success/error) |
| host -> iframe | `sbfb-bridge-event` | Fire-and-forget push notification |
| iframe -> host | `sbfb-bridge-heartbeat` | Liveness ping (every 1s) |

**Bridge methods** (16 total, defined in `BridgeMethodSchema`):

| Method | Category | Round-trip |
|---|---|---|
| `task_submit` | Core | Coordinator POST |
| `storage_get` | Core | Coordinator GET |
| `storage_set` | Core | Coordinator POST |
| `pii_redact` | SDK | Local (no network) |
| `storage_list` | Extension (S56) | Coordinator GET |
| `storage_delete` | Extension (S56) | Coordinator DELETE |
| `identity_pubkey` | Extension (S56) | Daemon GET |
| `node_status` | Extension (S56) | Health + Info |
| `browse_list` | Extension (S56) | Daemon GET |
| `storage_version` | Extension (S58) | Daemon GET |
| `provenance_get` | Verification (S63) | Coordinator GET |
| `provenance_verify` | Verification (S63) | Coordinator GET |
| `feed_cursor_get` | Feed (S63) | Daemon GET |

**Correlation:** Each request carries a UUID `id`. Response echoes the same `id`. Events have no correlation.

**Source validation:** The `useBridge` hook validates `event.source === iframe.contentWindow` for requests. Heartbeats only validate source, no side effects.

### 7.3 Watchdog

**File:** `web/src/bridge/useBridge.ts`

States: `unknown` -> `healthy` (on first heartbeat) -> `stalled` (no heartbeat for 5s).

When stalled, BrowsedProject shows an overlay: "Application ne repond plus" with Reload/Close buttons. Reload cycles iframe through `about:blank` then back to the blob-serve URL.

## 8. TabView SDK Rendering

**Directory:** `web/src/components/app/tabview/`

Schema-driven rendering system for SDK apps (legacy path, pre-iframe era). Still used for local projects.

**Schema:** `web/src/components/app/tabview/schema.ts`
- `TabView` has `schema_version` (1 or 2), `tab_name`, `title`, `blocks[]`
- Blocks are a discriminated union by `kind` field
- 12 block kinds: heading, text, kv, metric, table, badge_list, button, chart_line, chart_bar, empty, section (recursive), file_upload (v2 only)

**Renderer chain:**
1. `TabViewRenderer` (`TabViewRenderer.tsx`) — top-level, iterates blocks
2. `TabBlockRenderer` (`TabBlockRenderer.tsx`) — switch on `block.kind`, exhaustive check via `never`
3. Individual block components in `blocks/` directory:
   - `HeadingBlock.tsx`, `TextBlock.tsx`, `KVBlock.tsx`, `MetricBlock.tsx`
   - `TableBlock.tsx`, `BadgeListBlock.tsx`, `ButtonBlock.tsx`
   - `ChartLineBlock.tsx`, `ChartBarBlock.tsx` (inline SVG, no chart library)
   - `EmptyBlock.tsx`, `SectionBlock.tsx` (recursive)
   - `FileUploadBlock.tsx` (drag-and-drop, v2 only)

**Context:** `TabAppContext` provides `coordinatorUrl` + `appName` to blocks that need API access (e.g., ButtonBlock for task_submit).

## 9. Command Palette

**Files:**
- `web/src/components/command-palette/CommandPalette.tsx`
- `web/src/components/command-palette/useCommandPalette.ts`
- `web/src/components/command-palette/extractNavigationPath.ts`

**Trigger:** Ctrl+K / Cmd+K (physical key `KeyK`, layout-independent).

**Command groups:**
1. **Navigation:** 5 entries (Mes projets, Mon reseau, Explorer, Curators, Deployer)
2. **Projets:** One entry per known coordinator -> navigates to `/project/{nickname}`
3. **App commands:** Per-app group from `listAppCommands()` -> `invokeAppCommand()` -> optional navigation path extraction
4. **Actions:** Ajouter un coordinateur, Recharger la page

Built on `cmdk` (command menu primitive) + shadcn Command components.

## 10. PII Redaction SDK

**Directory:** `web/src/sdk/pii/`

Client-side PII detection for iframe apps via `pii_redact` bridge method.

**Architecture:**
- `policy.ts` — PII entity types, policy config, filtering
- `fallback.ts` — Regex-based detection (EMAIL, PHONE, CREDIT_CARD with Luhn, SSN, IBAN)
- `decoder.ts` — GLiNER ONNX span-logits decoder (pure, testable)
- `wrapper.ts` — ORT runtime wrapper, lazy model loading, singleton pattern
- `index.ts` — `detectAndRedact()` public API

**Fallback cascade:** ONNX model -> regex fallback (always available). Model failure is transparent to callers.

## 11. UI Component Library

### 11.1 shadcn/ui Components

**Directory:** `web/src/components/ui/` (22 components)

Style: `base-nova`, dark theme, Tailwind CSS v4 with CSS variables. Icon library: lucide-react.

**Available components:**
`badge`, `button`, `card`, `command`, `dialog`, `dropdown-menu`, `input`, `input-group`, `progress`, `radio-group`, `scroll-area`, `select`, `separator`, `sheet`, `sidebar`, `skeleton`, `slider`, `tabs`, `textarea`, `toggle`, `toggle-group`, `tooltip`

**Path alias:** `@/components/ui/X` resolves to `web/src/components/ui/X.tsx`.

### 11.2 CSS Architecture

**File:** `web/src/index.css`

- Imports: `tailwindcss`, `tw-animate-css`, `shadcn/tailwind.css`, Geist font
- **Always-dark theme** (no `.dark` class toggle needed)
- Custom CSS variables for NEXUS brand colors (bg-primary: #0a0a0f, etc.)
- shadcn token overrides mapped to custom values
- Glassmorphism utilities: `.glass-card`, `.glass-pill`
- Custom scrollbar styling
- `@theme inline` block maps CSS vars to Tailwind tokens

### 11.3 Utility

**File:** `web/src/lib/utils.ts`

```typescript
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

**File:** `web/src/lib/format.ts`

Pure formatting helpers (never throw):
- `formatHash(value, chars)` — truncate hex identifiers
- `formatUptime(secs)` — "1h 23m 45s"
- `formatRelativeTime(value)` — French relative time ("il y a 3 min")
- `formatMemoryMb(mb)` — "1.2 GiB" / "42 MiB"

## 12. Build Pipeline

### 12.1 Vite Config

**File:** `web/vite.config.ts`

- Plugins: `@vitejs/plugin-react`, `@tailwindcss/vite`, optional `rollup-plugin-visualizer` (ANALYZE_MODE=true)
- Path alias: `@ -> ./src`
- Dev proxy: `/api -> http://localhost:8000`, `/ollama -> http://localhost:11434`
- Manual chunks:
  - `vendor-react`: react, react-dom, react-router-dom, scheduler
  - `vendor-query`: @tanstack/*, zustand, zod
  - `vendor-ui`: @base-ui/*, @radix-ui/*, cmdk, tailwind-merge, clsx, cva
  - Source files left unassigned for natural per-route splitting

### 12.2 Size Limits

**File:** `web/.size-limit.json` (6 budgets, raw size without compression):

| Chunk | Limit |
|---|---|
| main (index) | 50 KB |
| vendor-react | 290 KB |
| vendor-query | 120 KB |
| vendor-ui | 270 KB |
| CommandPalette | 20 KB |
| CSS | 130 KB |

### 12.3 TypeScript

**Target:** ES2023, strict mode, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`, `verbatimModuleSyntax`. Path alias `@/* -> ./src/*`.

## 13. Testing

### 13.1 Vitest (Unit)

**Config:** `web/vitest.config.ts`
- Environment: jsdom
- Setup file: `web/src/test/setup.ts` (jest-dom matchers, localStorage stub, matchMedia/ResizeObserver/scrollIntoView stubs)
- Coverage: v8 provider, thresholds: 85% lines, 90% functions, 78% branches, 85% statements

**Test organization:** Co-located `__tests__/` directories next to source files.

**Test files (265 tests total):**
- `web/src/stores/__tests__/projectStore.test.ts` — Zustand store CRUD, persistence, selectors
- `web/src/api/__tests__/coordinator.test.ts` — API wrapper Zod validation, URL encoding
- `web/src/api/__tests__/daemon.test.ts` — Daemon result union handling
- `web/src/api/__tests__/auth.test.ts` — Token caching, auth header injection
- `web/src/bridge/__tests__/protocol.test.ts` — Bridge schema validation
- `web/src/bridge/__tests__/useBridge.test.ts` — Bridge dispatch
- `web/src/bridge/__tests__/watchdog.test.ts` — Watchdog state machine
- `web/src/components/__tests__/RouteErrorBoundary.test.tsx` — Error boundary rendering
- `web/src/components/__tests__/PanicWipeKeybind.test.tsx` — 5-tap gesture detection
- `web/src/components/__tests__/GpuConsentDialog.test.tsx` — Consent dialog interactions
- `web/src/components/__tests__/VerificationDetail.test.tsx` — Provenance display
- `web/src/components/app/tabview/__tests__/TabViewRenderer.test.tsx` — Block rendering
- `web/src/components/app/tabview/__tests__/FileUploadBlock.test.tsx` — File upload
- `web/src/components/command-palette/__tests__/useCommandPalette.test.ts` — Ctrl+K
- `web/src/components/command-palette/__tests__/CommandPalette.test.tsx` — Palette rendering
- `web/src/lib/__tests__/format.test.ts` — Formatting helpers
- `web/src/sdk/pii/__tests__/policy.test.ts` — PII policy resolution
- `web/src/sdk/pii/__tests__/fallback.test.ts` — Regex PII detection
- `web/src/sdk/pii/__tests__/decoder.test.ts` — GLiNER span decoder
- `web/src/sdk/pii/__tests__/wrapper.test.ts` — ONNX wrapper
- `web/src/pages/__tests__/BrowsedProject.test.tsx` — BrowsedProject rendering
- `web/src/pages/__tests__/Deploy.test.tsx` — Deploy form

**Mocking pattern:** `vi.stubGlobal("fetch", vi.fn())` for API tests. Store tests call `useProjectStore.getState().clear()` in beforeEach/afterEach.

### 13.2 Playwright (E2E)

**Config:** `web/playwright.config.ts`
- Chromium-only, single worker, 60s timeout
- Global setup spawns real `nexus-shell-daemon` subprocess against hermetic `tests/.tmp/nexus-grid/`
- Auth token injected via `extraHTTPHeaders: { "x-sbfb-token": TEST_AUTH_TOKEN }`
- Locale: `fr-FR`, viewport: 1440x900

**E2E test files (28 specs):**
- `shell-add-coordinator.spec.ts` — Onboarding happy path
- `shell-onboarding-empty-state.spec.ts` — Empty state
- `my-projects-live.spec.ts` — Live coordinator card
- `my-network-live.spec.ts` — Worker state display
- `project-detail-manifest.spec.ts` — Project detail tabs
- `apps-tab-render.spec.ts` — App manifest rendering
- `curators-flow.spec.ts` — Subscribe/unsubscribe flow
- `browse-daemon-offline.spec.ts` — Daemon offline banner
- `browse-click-project.spec.ts` — Browse grid navigation
- `command-palette.spec.ts` — Ctrl+K palette
- `loopback-auth.spec.ts` — Bearer token injection
- `bridge-heartbeat.spec.ts` — Iframe heartbeat watchdog
- `bridge-push-event.spec.ts` — Host -> iframe events
- `bridge-pii-redact.spec.ts` — PII redaction bridge
- `blob-serve-coep.spec.ts` — COOP/COEP headers
- `tabview-schema-driven.spec.ts` — TabView rendering
- Various `gov-*.spec.ts` — Legacy gov app specs (10 files)

**Run commands:**
```bash
npm run test:unit        # Vitest (all unit tests)
npm run test:unit:watch  # Vitest watch mode
npm run test:coverage    # Vitest with coverage
npm run test:e2e         # Playwright (requires daemon binary)
```

## 14. i18n / Language

**All UI strings are French.** Enforced by `web/scripts/scan-en-strings.sh` which greps for English words (Welcome, Dashboard, Sign in, etc.) in `.tsx`/`.ts` files under `src/` (excluding `ui/`, `tests/`, `scripts/`). Exit 1 on any match.

**Code identifiers and error strings remain English** per project convention (CLAUDE.md).

## 15. Error Handling Patterns

### 15.1 Route Error Boundary

**File:** `web/src/components/RouteErrorBoundary.tsx`

Class component wrapping `<Outlet />`. On crash: shows French error message + "Reessayer" / "Recharger" buttons + expandable stack trace. Shell chrome (sidebar, palette) stays alive.

### 15.2 API Error Classes

- `ApiHttpError(endpoint, status, statusText)` — non-2xx HTTP response
- `ApiProtocolError(endpoint, zodIssues, rawBody)` — Zod validation failure

### 15.3 Daemon Unavailable

Daemon API returns `{ kind: "unavailable" }` on network error or 503. UI renders `<DaemonOfflineBanner />` instead of crashing.

## 16. Security Components

### 16.1 PanicWipeKeybind

**File:** `web/src/components/PanicWipeKeybind.tsx`

Invisible component. Fires `POST /api/daemon/panic/wipe` after Ctrl+Shift+Alt+W pressed 5 times within 3 seconds. No visual feedback by design (deniability). Mounted at AppShell root.

### 16.2 GPU Consent Dialog

**File:** `web/src/components/GpuConsentDialog.tsx`

4-level consent (L1 private -> L4 all public). BOINC/GDPR-style opt-in. Includes resource caps (watts, VRAM, hours/day) and L3 whitelist management. Auto-opens once per browser profile (localStorage flag).

### 16.3 Verification Detail

**File:** `web/src/components/VerificationDetail.tsx`

Dialog showing SLSA L1 provenance: repo URL, commit SHA, artifact hash, signature, node ID, timestamp. Hash mismatch warning when provenance_hash differs from network announcement.

## 17. Key Architectural Decisions

1. **Shell is an iframe host** — knows nothing about app technology. Apps are web archives served via blob-serve daemon.
2. **postMessage is the only iframe-to-network channel** — no direct fetch from sandboxed iframes.
3. **Every API response is Zod-validated** — protocol errors surface as structured errors, never crashes.
4. **DaemonResult union** — daemon offline is a normal UI state, not an exception.
5. **Zustand for client state, React Query for server state** — clean separation.
6. **French-only UI** — no i18n framework, strings are inline, enforced by CI script.
7. **Always-dark theme** — no light mode. CSS variables set once on `:root`.
8. **Lazy routes** — all pages code-split via React Router `lazy()`.
9. **shadcn/ui convention** — components copied into `ui/`, modified only when necessary (T1 policy: keep regen-safe).

## 18. File Quick Reference

### Entry Points
- `web/src/main.tsx` — Bootstrap, auth token resolution, React root mount
- `web/src/App.tsx` — Router + QueryClient + TooltipProvider setup

### API Layer
- `web/src/api/auth.ts` — Token management, `authFetch()` wrapper
- `web/src/api/coordinator.ts` — Coordinator/app endpoints + all Zod schemas
- `web/src/api/daemon.ts` — Daemon endpoints + `DaemonResult<T>` union
- `web/src/api/consent.ts` — GPU consent CRUD

### State
- `web/src/stores/projectStore.ts` — Zustand store (single store, persisted)

### Bridge
- `web/src/bridge/protocol.ts` — Zod schemas for all message types
- `web/src/bridge/useBridge.ts` — Host-side message handler hook

### Pages
- `web/src/pages/Projects.tsx` — `/my-projects`
- `web/src/pages/ProjectDetail.tsx` — `/project/:name`
- `web/src/pages/Network.tsx` — `/my-network`
- `web/src/pages/Browse.tsx` — `/browse`
- `web/src/pages/BrowsedProject.tsx` — `/browse/:projectId` (iframe host)
- `web/src/pages/Curators.tsx` — `/curators`
- `web/src/pages/Deploy.tsx` — `/deploy`
- `web/src/pages/OnboardingEmpty.tsx` — First-run onboarding

### Shell Components
- `web/src/components/AppShell.tsx` — Layout shell + CoordinatorPicker
- `web/src/components/AddCoordinatorDialog.tsx` — Add coordinator modal
- `web/src/components/GpuConsentDialog.tsx` — GPU sharing consent
- `web/src/components/PanicWipeKeybind.tsx` — Emergency wipe keybind
- `web/src/components/RouteErrorBoundary.tsx` — Route crash boundary
- `web/src/components/VerificationDetail.tsx` — Provenance verification

### Command Palette
- `web/src/components/command-palette/CommandPalette.tsx`
- `web/src/components/command-palette/useCommandPalette.ts`
- `web/src/components/command-palette/extractNavigationPath.ts`

### TabView SDK
- `web/src/components/app/tabview/schema.ts` — Zod schemas (v1/v2)
- `web/src/components/app/tabview/TabViewRenderer.tsx` — Top-level renderer
- `web/src/components/app/tabview/TabBlockRenderer.tsx` — Kind switch
- `web/src/components/app/tabview/TabAppContext.tsx` — Coordinator context
- `web/src/components/app/tabview/blocks/` — 12 block components

### PII SDK
- `web/src/sdk/pii/index.ts` — Public API
- `web/src/sdk/pii/policy.ts` — Entity types + policy
- `web/src/sdk/pii/fallback.ts` — Regex detector
- `web/src/sdk/pii/decoder.ts` — GLiNER ONNX decoder
- `web/src/sdk/pii/wrapper.ts` — ORT runtime wrapper

### Project Detail Tabs
- `web/src/components/project/OverviewTab.tsx`
- `web/src/components/project/TasksTab.tsx`
- `web/src/components/project/KudosTab.tsx`
- `web/src/components/project/InvitesTab.tsx`
- `web/src/components/project/AppsTab.tsx`

### Utilities
- `web/src/lib/utils.ts` — `cn()` classname merger
- `web/src/lib/format.ts` — Pure formatting helpers
- `web/src/hooks/use-mobile.ts` — Mobile breakpoint hook (768px)

### Config
- `web/vite.config.ts` — Build config + chunking + dev proxy
- `web/vitest.config.ts` — Test runner config
- `web/playwright.config.ts` — E2E config
- `web/eslint.config.js` — Lint rules
- `web/.size-limit.json` — Bundle size budgets
- `web/components.json` — shadcn/ui config
- `web/tsconfig.app.json` — TS strict config

---

*Frontend architecture analysis: 2026-05-18*
