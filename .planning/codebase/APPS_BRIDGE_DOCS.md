# Example Apps, Bridge SDK, Documentation & Developer Experience

**Analysis Date:** 2026-05-18

---

## 1. Universal Render Model

SBFB supports any technology that produces HTML. Apps are distributed as **zip archives** containing an `index.html` at the root. The daemon's blob-serve module decompresses the zip, caches files in an LRU in-memory store, and serves them via `GET /blob-serve/{hash}/{path}`. The shell React host renders each app inside a sandboxed iframe.

**Supported technologies:** React, Vue, Svelte, Python/Pyodide, WASM, Jupyter/JupyterLite, plain HTML, Elm, any static site generator. The only requirement is that `index.html` exists at the zip root.

**Key constraint:** CSP `connect-src 'none'` blocks all outbound network requests from the iframe. Apps communicate with the network exclusively through the postMessage bridge.

---

## 2. SBFB Bridge SDK (`sbfb-bridge.js`)

### 2.1 Source of Truth and Distribution

The canonical copy lives at `web/public/sbfb-bridge.js` (398 lines). It is synchronized to example apps via the script `scripts/sync-bridge-sdk.sh`, which copies the file to each `examples/*/sbfb-bridge.js` and verifies SHA256 post-copy.

Copies exist at:
- `web/public/sbfb-bridge.js` -- canonical source
- `examples/sbfb-explorer/sbfb-bridge.js` -- synced copy
- `examples/sbfb-ideas/sbfb-bridge.js` -- synced copy
- `web/dist/sbfb-bridge.js` -- build output

### 2.2 Architecture

The bridge is a single ES5-compatible `class SBFBBridge` with no dependencies. It communicates via `window.postMessage` between the sandboxed iframe (app) and the parent window (shell host).

**Message flow:**

```
iframe app                    shell host (React)
-----------                   ------------------
bridge._call(method, payload)
  └─ parent.postMessage({        handler in useBridge.ts
       type: "sbfb-bridge-request",    │
       id: <uuid>,                     │
       method: "task_submit",     ──►  BridgeRequestSchema.safeParse()
       payload: {...}                  │
     })                                │
                                       ├─ dispatch(coordUrl, appName, req)
                                       │   └─ coordinator API calls
                                       │
     iframe.contentWindow.postMessage( │
       { type: "sbfb-bridge-response", ◄──┤
         id: <uuid>,                   │
         success: true,                │
         data: {...}                   │
       })                              │
```

**Correlation IDs:** Every request includes a UUID v4 `id` field. The SDK generates it via `crypto.randomUUID()` with a manual polyfill fallback. The host matches responses to pending requests by this `id`. Requests time out after `options.timeout` ms (default 10000).

### 2.3 Bridge Methods -- Complete API Reference

The `BridgeMethodSchema` in `web/src/bridge/protocol.ts` enumerates all 13 allowed methods:

#### Core Methods (Sprint 13)

| Method | SDK Method | Purpose | Payload | Response |
|--------|-----------|---------|---------|----------|
| `task_submit` | `bridge.submitTask(payload)` | Submit compute task to network | `{prompt, task_type, ...}` | `{task_id, status}` |
| `storage_get` | `bridge.getStorage(key)` | Read from coordinator storage | `{key}` | stored value |
| `storage_set` | `bridge.setStorage(key, value)` | Write to coordinator storage | `{key, ...value}` | `{ok: true}` |

#### PII Redaction (Sprint 21)

| Method | SDK Method | Purpose | Payload | Response |
|--------|-----------|---------|---------|----------|
| `pii_redact` | `bridge.piiRedact(text, policy?)` | Redact PII before task submission | `{text, policy?}` | `{redacted_text, findings_count}` |

The `pii_redact` method is dispatched **locally in the host shell** (no coordinator round-trip). It uses GLiNER ONNX when available, falling back to curated regex (email, phone, CC, SSN, IBAN). Text limit: 50,000 chars (`PII_REDACT_MAX_TEXT_LENGTH`).

#### Bridge Extensions (Sprint 56)

| Method | SDK Method | Purpose | Payload | Response |
|--------|-----------|---------|---------|----------|
| `storage_list` | `bridge.listStorage(prefix?)` | List keys by prefix | `{prefix}` | `{entries, count}` |
| `storage_delete` | `bridge.deleteStorage(key)` | Remove a storage key | `{key}` | `{ok: boolean}` |
| `identity_pubkey` | `bridge.getIdentityPubkey()` | Get node's Ed25519 public key | `{}` | `{pubkey}` |
| `node_status` | `bridge.getNodeStatus()` | Get daemon status | `{}` | `{node_id, version, uptime_seconds, peers}` |
| `browse_list` | `bridge.getBrowseList()` | List apps on the network | `{}` | `{entries: [...]}` |

#### Storage Sync (Sprint 58)

| Method | SDK Method | Purpose | Payload | Response |
|--------|-----------|---------|---------|----------|
| `storage_version` | `bridge.getStorageVersion(appName)` | Get sync version counter | `{app}` | `{app, version}` |

The `onStorageUpdate(appName, callback)` helper polls `storage_version` every 3 seconds and fires the callback when the version changes (indicating remote iroh-docs sync).

#### Verification (Sprint 63)

| Method | SDK Method | Purpose | Payload | Response |
|--------|-----------|---------|---------|----------|
| `provenance_get` | `bridge.getProvenanceRecord(projectId)` | Get provenance record | `{project_id}` | `{record, provenance_hash}` |
| `provenance_verify` | `bridge.verifyRelease(projectId)` | Verify release Ed25519 signature | `{project_id}` | `{verified, record, provenance_hash}` |
| `feed_cursor_get` | `bridge.getPublicFeedCursor()` | Get feed materializer cursor | `{}` | `{last_seq, last_entry_hash}` |

### 2.4 Push Events (Sprint 15 Phase A)

The host can push fire-and-forget events to the iframe:

```javascript
// Host side (React useBridge hook)
pushEvent("task_result_ready", { task_id: "t-42", result: "ok" });

// Iframe app side
bridge.onEvent("task_result_ready", (payload) => {
  console.log("task done:", payload.task_id);
});
```

Message format: `{ type: "sbfb-bridge-event", name: <string 1-64 chars>, payload: <any> }`

`onEvent()` returns an unsubscribe function. Multiple handlers per event name are supported.

### 2.5 Heartbeat / Watchdog (Sprint 15 Phase B)

The bridge automatically emits heartbeat pings to the host every `heartbeatInterval` ms (default 1000). Message format: `{ type: "sbfb-bridge-heartbeat", ts: <Date.now()> }`. The host watchdog (in `useBridge.ts`) tracks the last heartbeat timestamp and declares the iframe `stalled` after 5000ms (`STALL_THRESHOLD_MS`) without a heartbeat.

Watchdog states: `unknown` (no heartbeat received yet) -> `healthy` (heartbeat received within threshold) -> `stalled` (no heartbeat for >5s).

Tests can disable the heartbeat via `new SBFBBridge({ heartbeatInterval: 0 })`.

### 2.6 Lifecycle

```javascript
// Instantiate once per iframe mount
const bridge = new SBFBBridge({ timeout: 5000 });

// Use the bridge...
const result = await bridge.submitTask({ prompt: "Hello" });

// On iframe unmount -- MUST call to prevent memory leaks
bridge.destroy();
```

`destroy()` removes the message listener, stops the heartbeat timer, and rejects all pending requests with `"bridge destroyed"`.

### 2.7 Host-Side Implementation

Located at `web/src/bridge/useBridge.ts` (366 lines). The React hook `useBridge(coordUrl, appName, iframeRef)` does:

1. Listens for `window.message` events
2. Validates against `BridgeRequestSchema` (Zod) -- rejects unknown methods or malformed UUIDs
3. Validates source: `event.source === iframe.contentWindow` (prevents cross-iframe spoofing)
4. Dispatches to coordinator API via `authFetch()` (bearer token)
5. Sends typed `BridgeResponse` back to iframe via `postMessage`

Special case: `pii_redact` is dispatched locally (no coordinator round-trip) so it works even when coordinator is offline.

### 2.8 Protocol Types

Defined in `web/src/bridge/protocol.ts` with Zod schemas:

- `BridgeRequestSchema`: `{type: "sbfb-bridge-request", id: uuid, method: BridgeMethod, payload: Record<string, unknown>}`
- `BridgeResponseSchema`: `{type: "sbfb-bridge-response", id: uuid, success: boolean, data?: unknown, error?: string}`
- `BridgeEventSchema`: `{type: "sbfb-bridge-event", name: string(1-64), payload: unknown}`
- `BridgeHeartbeatSchema`: `{type: "sbfb-bridge-heartbeat", ts: number}`
- `PiiRedactPayloadSchema`: `{text: string(max 50000), policy?: {...}}`

---

## 3. Blob-Serve Daemon

### 3.1 Source

`crates/nexus-shell-daemon-core/src/blob_serve.rs` (484 lines including tests).

### 3.2 Architecture

The `BlobServeCache` is a concurrent in-memory cache using `DashMap`. Each entry maps a blob hash (hex string) to a `HashMap<String, Vec<u8>>` of decompressed files.

```rust
pub struct BlobServeCache {
    entries: DashMap<String, Arc<HashMap<String, Vec<u8>>>>,
    insertion_order: DashMap<String, std::time::Instant>,
    max_entries: usize,  // default: 32
}
```

**API:**
- `BlobServeCache::new(max_entries)` -- create cache
- `cache.load(hash, zip_bytes, max_decompressed_bytes)` -- decompress and cache
- `cache.get_file(hash, path)` -- retrieve a single file
- `cache.has(hash)` -- check if cached

**LRU eviction:** When the cache is at capacity, the oldest entry (by insertion time) is evicted.

### 3.3 Security Layers

**Path traversal rejection** (`validate_zip_path`):
- Rejects `..`, absolute paths (`/`), backslash paths (`\`), empty paths

**Decompressed size limit:**
- Default `DEFAULT_MAX_DECOMPRESSED_BYTES = 100 * 1024 * 1024` (100 MB)
- Prevents zip bombs

**CSP headers** (injected on every blob-serve response):
```
Content-Security-Policy: default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:;
  connect-src 'none'; worker-src 'none'; frame-src 'none'; object-src 'none';
  base-uri 'none'; form-action 'none'; frame-ancestors *; sandbox allow-scripts
```

**COOP/COEP headers:**
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

The `sandbox allow-scripts` in the CSP gives an opaque origin even if someone navigates directly to `/blob-serve/{hash}/index.html` in a top-level tab (blocks localStorage, cookies, Service Workers on the daemon origin).

### 3.4 Content-Type Detection

`detect_content_type(filename, data)` returns the HTTP Content-Type:
- Extension-based: html, js, mjs, css, json, svg, png, jpg, gif, webp, ico, woff, woff2, ttf, otf, wasm, xml, txt, map
- Magic bytes fallback: PNG, JPEG, GIF, WEBP, WASM signatures
- HTML heuristic: checks first 50 bytes for `<!doctype` or `<html`
- Default: `application/octet-stream`

### 3.5 E2E Test

`crates/nexus-test-harness/tests/blob_serve_coep.rs` validates COOP/COEP/CSP headers on a real daemon serving a real zip archive.

---

## 4. Example App: Protocol Explorer (`examples/sbfb-explorer/`)

### 4.1 Files

| File | Size | Purpose |
|------|------|---------|
| `examples/sbfb-explorer/index.html` | 461 lines | 6-section educational page |
| `examples/sbfb-explorer/app.js` | 247 lines | Live status, verification demo |
| `examples/sbfb-explorer/sbfb-bridge.js` | 398 lines | Bridge SDK copy |
| `examples/sbfb-explorer/SBFB.json` | 5 lines | App manifest |
| `examples/sbfb-explorer/style.css` | 578 lines | Dark theme (GitHub-style) |

### 4.2 Purpose

Demonstrates the SBFB protocol through 6 interactive sections:

1. **Architecture** -- ASCII diagram of Browser/Shell Daemon/Workers/Coordinator/iroh P2P
2. **App Lifecycle** -- 5-step process: Code source -> Verified deploy -> Archive zip -> Distribution P2P -> Rendu iframe
3. **Task Lifecycle** -- 5-step process: Submission -> Dispatch -> Execution -> Validation -> Kudos
4. **Security** -- 6 cards: Loopback HTTP, Sandbox iframe, Curators Ed25519, Sybil resistance, Deploy verifie, PII protection
5. **Verification & Provenance** -- Interactive demo: select a project, click "Verifier la provenance", see live Ed25519 verification result
6. **Philosophy** -- 4 cards: Zero administration, Open source par construction, Pas de monnaie, Decentralisation reelle

### 4.3 Bridge Usage

The Explorer uses the bridge for:
- `bridge.getNodeStatus()` -- polls every 15s for node status
- `bridge.getIdentityPubkey()` -- displays local Ed25519 pubkey
- `bridge.getBrowseList()` -- populates app list and project selector
- `bridge.verifyRelease(projectId)` -- interactive provenance verification

### 4.4 App Manifest (`SBFB.json`)

```json
{
  "node_id": "PLACEHOLDER",
  "name": "sbfb-explorer",
  "version": "1.0.0"
}
```

The `node_id: "PLACEHOLDER"` is replaced with the actual daemon node_id at deploy time. The coordinator verifies this match during `deploy-from-repo`.

### 4.5 Technical Details

- Pure vanilla JS (IIFE, no build step, no dependencies)
- Source links point to `https://github.com/SBFB50/SBFB/tree/master/` via `data-path` attributes
- French-language UI (`lang="fr"`)
- Responsive design (640px breakpoint)
- Dark theme: `--bg: #0d1117`, `--accent: #58a6ff` (GitHub dark palette)

---

## 5. Example App: Ideas Hub (`examples/sbfb-ideas/`)

### 5.1 Files

| File | Size | Purpose |
|------|------|---------|
| `examples/sbfb-ideas/index.html` | 69 lines | Proposal form + ideas list |
| `examples/sbfb-ideas/app.js` | 354 lines | CRUD + voting + sync |
| `examples/sbfb-ideas/sbfb-bridge.js` | 398 lines | Bridge SDK copy |
| `examples/sbfb-ideas/SBFB.json` | 5 lines | App manifest |
| `examples/sbfb-ideas/style.css` | 407 lines | Dark theme (matching Explorer) |

### 5.2 Purpose

A fully functional P2P idea voting application. Demonstrates real-world use of the bridge storage API for collaborative data.

### 5.3 Features

- **Submit ideas:** Title (max 120 chars) + optional description (max 2000 chars)
- **Vote/unvote:** Per-user voting tied to Ed25519 pubkey identity
- **Delete:** Author can delete their own ideas (cascades vote deletions)
- **Sort:** By votes (default) or by recency
- **Live P2P sync:** `bridge.onStorageUpdate("sbfb-ideas", callback)` polls for remote changes every 3s

### 5.4 Data Model (Storage Keys)

```
ideas/{uuid}          → { title, description, author, created_at }
votes/{ideaId}/{pubkey} → { timestamp }
```

This key structure enables prefix-based listing (`bridge.listStorage("ideas/")` and `bridge.listStorage("votes/")`) and per-user vote deduplication.

### 5.5 Bridge Usage

The Ideas Hub uses 6 bridge methods:
- `bridge.getIdentityPubkey()` -- identify the local user
- `bridge.listStorage("ideas/")` -- load all ideas
- `bridge.listStorage("votes/")` -- load all votes
- `bridge.setStorage("ideas/" + id, data)` -- create idea
- `bridge.setStorage("votes/" + ideaId + "/" + pubkey, data)` -- cast vote
- `bridge.deleteStorage(key)` -- delete idea or vote
- `bridge.onStorageUpdate("sbfb-ideas", callback)` -- live sync

### 5.6 UI Pattern: No HTML Forms

Per the constraint `sandbox="allow-scripts"` without `allow-forms`, the app uses `<button type="button">` with click handlers instead of `<form>` submission. Inputs use `<input>` and `<textarea>` directly without a wrapping `<form>` tag.

---

## 6. Legacy Example: Hello World App (`examples/hello-world-app/`)

### 6.1 Files

| File | Purpose |
|------|---------|
| `examples/hello-world-app/pyproject.toml` | Python package config |
| `examples/hello-world-app/src/hello_world_app/__init__.py` | NexusApp class |

### 6.2 Status

This is a **legacy app from the pre-pivot Python SDK era** (Sprint 4-6). It uses the old `nexus-sdk` Python package with `NexusApp`, `@nexus_route`, `@nexus_worker`, `@nexus_tab` decorators and `TabView` schema-driven rendering. It does NOT use the current universal render model (zip + iframe + bridge).

This app is not deployable on the current platform as-is. It serves as a historical reference for the old SDK architecture.

---

## 7. Deploy Verification Flow

### 7.1 Source

`crates/nexus-shell-daemon/src/deploy.rs` (753 lines including tests).

### 7.2 Verified Deploy Pipeline (`POST /api/v1/deploy-from-repo`)

```
1. Validate repo_url (must be HTTP(S))
2. Validate commit_sha (40 hex chars if provided)
3. HEAD request to repo_url (verify publicly accessible, 10s timeout)
4. git clone --depth 1 --single-branch (30s timeout)
5. If commit_sha provided: git fetch origin <sha> + git checkout FETCH_HEAD
6. Check clone size < 500 MB
7. Read SBFB.json → verify node_id matches daemon's node_id
8. Verify index.html exists at root
9. Resolve commit_sha via git rev-parse HEAD (if not provided)
10. zip_directory() → zip bytes (excludes .git/, symlinks, suspicious paths)
11. BLAKE3 hash of zip bytes → artifact_hash
12. generate_provenance() → ProvenanceRecord signed Ed25519
13. Record contributor attestation (best-effort)
14. Inject provenance.json into zip
15. Store zip via iroh-blobs → content-addressed hash
16. Persist provenance record in coordinator DB
17. Broadcast ProjectAnnouncement via gossip
18. Return { deployed: true, hash, provenance_hash, commit_sha }
```

### 7.3 Private Deploy (`POST /api/v1/deploy`)

Simple zip upload (max 100 MB). No verification, no provenance. Returns `{ deployed: true, hash }`.

### 7.4 SBFB.json Manifest

Required at repository root for verified deploy:

```json
{
  "node_id": "<daemon_ed25519_node_id_hex>",
  "name": "<project_name>",
  "version": "<semver>"
}
```

The `node_id` must match the local daemon's node_id. This is the Keyoxide pattern: the repo declares which node is authorized to deploy it.

### 7.5 Provenance Record

Defined in `crates/nexus-coordinator-rs/src/provenance.rs` (212 lines):

```rust
pub struct ProvenanceRecord {
    pub schema_version: u32,     // always 1 pre-v1.0
    pub repo_url: String,
    pub commit_sha: String,
    pub artifact_hash: String,   // BLAKE3 of zip
    pub node_id: String,         // coordinator Ed25519 hex
    pub timestamp: String,       // RFC 3339
    pub signature: String,       // Ed25519 hex
    pub app_version: Option<String>,  // from SBFB.json
}
```

**Canonical bytes** for signing use domain separation: `b"nexus-provenance-v1" || 0x00 || JCS(sorted JSON)`.

**Verification:** `verify_provenance(record_json, public_key_bytes)` reconstructs canonical bytes and verifies the Ed25519 signature.

---

## 8. Publish States (4 States)

Documented in `docs/architecture/PUBLISH_MODEL.md`:

| State | Source | Badge | Workers | Proof |
|-------|--------|-------|---------|-------|
| **Local Draft** | dev disk | none | no | none |
| **Unverified Build** | zip uploaded, no provenance | "non verifie" | opt-in only | artifact hash only |
| **Verified Release** | public commit + SLSA L1 provenance | "open source verifie" | yes, consent L2+ | repo_url + commit_sha + artifact_hash + provenance_hash |
| **Stale Source** | repo unreachable/diverged | "source indisponible" | per worker policy | provenance exists, not live-reverifiable |

---

## 9. Bridge Tests

### 9.1 Playwright E2E Tests

| Test File | Coverage |
|-----------|----------|
| `web/tests/bridge-heartbeat.spec.ts` | Real iframe emits 2+ heartbeats within 2s |
| `web/tests/bridge-push-event.spec.ts` | Host->iframe push events fire `onEvent` callbacks; non-subscribed events ignored |
| `web/tests/bridge-pii-redact.spec.ts` | `piiRedact()` callable from iframe; regex fallback replaces emails; `enabled:false` passthrough; error on non-string; correlation ID preserved |

### 9.2 Vitest Unit Tests

| Test File | Coverage |
|-----------|----------|
| `web/src/bridge/__tests__/protocol.test.ts` | Schema validation: valid/invalid requests, responses, events, PII payload |
| `web/src/bridge/__tests__/useBridge.test.ts` | Host-side hook behavior |
| `web/src/bridge/__tests__/watchdog.test.ts` | Watchdog state transitions |

### 9.3 Rust Tests

| Test File | Coverage |
|-----------|----------|
| `crates/nexus-shell-daemon-core/src/blob_serve.rs` (inline) | Path validation, content-type detection, cache load/get/eviction, zip bomb rejection, traversal rejection |
| `crates/nexus-test-harness/tests/blob_serve_coep.rs` | Real daemon serves real zip with COOP/COEP/CSP headers |
| `crates/nexus-shell-daemon/src/deploy.rs` (inline) | SHA validation, zip validation, SBFB.json parsing, zip creation + provenance injection, .git exclusion |
| `crates/nexus-coordinator-rs/src/provenance.rs` (inline) | Generate + verify provenance, wrong key rejection, tamper detection, BLAKE3 determinism, app_version serialization |

---

## 10. Developer Documentation

### 10.1 React Migration Guide

`docs/apps/REACT_MIGRATION.md` (398 lines) provides a complete guide for porting existing React apps to SBFB:

- Iframe constraints: no cookies, no localStorage, no fetch, no CDN resources
- Vite config: `base: "./"` is **mandatory** (relative paths for blob-serve)
- TypeScript declarations for `window.SBFBBridge`
- React hook pattern (`useSBFBBridge`) for proper lifecycle management
- Migration examples: `fetch()` -> `bridge.submitTask()`, `localStorage` -> `bridge.setStorage()`, `WebSocket` -> `bridge.onEvent()`
- Self-hosting assets: `@fontsource` for fonts, bundled images
- Dev mode: bridge no-op mock vs real daemon deploy
- Anti-patterns list (fetch directly, SBFBBridge in render, missing `base: "./"`, missing `destroy()`)
- `SBFB.json` manifest spec
- `POST /project/deploy-from-repo` curl example
- Verification checklist

### 10.2 Launcher Architecture

`docs/architecture/LAUNCHER.md` (623 lines) documents:
- Single binary vision (~21 MB), component breakdown
- First launch flow (10 steps: token generate -> iroh boot -> frontend fetch -> browser open)
- Normal launch flow, frontend update flow
- File structure under `~/.sbfb/`
- Security model: token lifecycle, frontend integrity (BLAKE3 via iroh-blobs), iframe isolation
- Frontend vs blob-serve app distinction (same-origin trusted vs sandboxed)

### 10.3 Publish Model

`docs/architecture/PUBLISH_MODEL.md` (195 lines) defines the 4 publish states and the full release lifecycle from Local Draft to Verified Release to Stale Source.

### 10.4 Security Documentation

Extensive security docs in `docs/security/`:

| File | Content |
|------|---------|
| `docs/security/THREAT_MODEL.md` | STRIDE + LINDDUN model, 7 assets, 5 adversary personas |
| `docs/security/RUNTIME_ISOLATION.md` | VM isolation roadmap (WSL2/Virtualization.framework/nspawn) |
| `docs/security/HARDENING_ROADMAP.md` | Security hardening phases |
| `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` | 3-tier trust model (AUTO/CONFIRM_PROMPT/BIOMETRIC_GATE) |
| `docs/security/GUARDRAILS_ARCHITECTURE.md` | Guardrails system |
| `docs/security/ADVERSARIES.md` + `adversaries/T0-T5` | 6 adversary tiers from curious user to state-targeted |
| `docs/security/PROCESS_ARCHITECTURE.md` | Process isolation design |

### 10.5 Protocol Specification

`docs/protocol/PUBLIC_FEED_SPEC.md` -- append-only signed event log specification. Operations: `ReleasePublished`, `SourceBecameStale`, future `CuratorVouched`, `BuildQuorumReached`, etc. Uses JCS (RFC 8785) canonical serialization with domain separation.

### 10.6 App Ideas / Showcase

| File | Content |
|------|---------|
| `docs/apps/README.md` | 9 planned apps with LOC estimates (~14900 LOC total) |
| `docs/apps/LAUNCH_SHOWCASE.md` | 3 showcase apps: Alexandria (knowledge), Forest Surveillance (GPU vision), D&D (distributed LLM) |
| `docs/apps/CHAT_IA_RESEAU.md` | Chat IA design |
| `docs/apps/DND_P2P.md` | D&D P2P design |
| `docs/apps/RENDER_FARM.md` | Blender render farm |
| `docs/apps/CATASTROPHE_HUMANITAIRE.md` | Crisis platform (10 sub-apps) |
| `docs/apps/ELDER_CARE.md` | Elder care dashboard |
| `docs/apps/EHPAD_LIEN_FAMILLE.md` | EHPAD family link |
| Plus others | IoT sensors, generative composition, LLM chat collab |

### 10.7 Shell/Coordinator Patterns

`docs/shell/PATTERNS.md` documents critical patterns:
- **P1**: Typed coordinator client is the only allowed fetch path (Zod schemas)
- **P2**: base-ui `render` prop, not Radix `asChild`
- **P3**: Zustand 5 curried `create` syntax
- **P4**: React Query is the only cache (no manual `useEffect + fetch`)
- **P5**: CORS allowed on loopback origins only
- **P6**: `NEXUS_GRID_ROOT` env override for tests
- **P8**: TabView is the only contract for app-provided tabs

### 10.8 Rust Patterns

`docs/rust/PATTERNS.md` captures compile-time lessons from iroh 0.97/0.98 API, PyO3 0.28, and SBFB-specific patterns.

### 10.9 Agent/Workflow Documentation

| File | Content |
|------|---------|
| `docs/claude/README.md` | Sprint lifecycle, audit gate, commit discipline |
| `docs/claude/SPRINT_LOG.md` | Sprint history table |
| `docs/agent/PROCESS.md` | Agent process |
| `docs/agent/codex-process/` | 6 files: session start, phase driver, reviewer audit, commit gate, domain smoke matrices, automation backlog |

---

## 11. How to Build an SBFB App -- Step by Step

Based on the codebase patterns, a new app developer should:

### 11.1 Create the App

```
my-app/
  index.html      -- entry point (required)
  app.js           -- application logic
  style.css        -- styling
  sbfb-bridge.js   -- copy from web/public/sbfb-bridge.js
  SBFB.json        -- { "node_id": "...", "name": "my-app", "version": "0.1.0" }
```

### 11.2 Use the Bridge

```html
<script src="./sbfb-bridge.js"></script>
<script>
  var bridge = new SBFBBridge({ timeout: 5000 });

  // Read/write storage
  bridge.setStorage("my-key", { data: "value" });
  bridge.getStorage("my-key").then(console.log);
  bridge.listStorage("prefix/").then(console.log);
  bridge.deleteStorage("my-key");

  // Submit compute tasks
  bridge.submitTask({ prompt: "Translate this", task_type: "analysis" });

  // Listen for results
  bridge.onEvent("task_result_ready", function(payload) {
    console.log("Result:", payload);
  });

  // PII redaction before task submission
  bridge.piiRedact("Contact alice@example.com").then(function(result) {
    bridge.submitTask({ prompt: result.redacted_text });
  });

  // Network info
  bridge.getIdentityPubkey().then(console.log);
  bridge.getNodeStatus().then(console.log);
  bridge.getBrowseList().then(console.log);

  // Provenance verification
  bridge.verifyRelease("project-id").then(console.log);

  // Live P2P sync
  bridge.onStorageUpdate("my-app", function() {
    console.log("Remote data changed, reload...");
  });
</script>
```

### 11.3 Constraints to Follow

- **No `fetch()` or `XMLHttpRequest`** -- blocked by `connect-src 'none'`
- **No `localStorage` / `sessionStorage` / `IndexedDB`** -- opaque origin
- **No CDN resources** -- all assets must be bundled in the zip
- **No `<form>` submission** -- use `<button type="button">` + click handlers
- **No `window.open()` or navigation** -- blocked by sandbox
- **Relative paths only** -- `./file.js` not `/file.js` (blob-serve serves from a subpath)
- **Call `bridge.destroy()` on unmount** -- prevents memory leaks

### 11.4 Deploy

```bash
# Verified (public, open source)
curl -X POST http://127.0.0.1:8080/api/v1/deploy-from-repo \
  -H "Content-Type: application/json" \
  -H "X-SBFB-Token: <token>" \
  -d '{"repo_url": "https://github.com/user/my-app", "project_name": "my-app"}'

# Private (direct zip upload)
curl -X POST http://127.0.0.1:8080/api/v1/deploy \
  -H "X-SBFB-Token: <token>" \
  --data-binary @my-app.zip
```

---

## 12. Design Theme and Visual Identity

Both example apps share a consistent GitHub-dark-inspired theme:

```css
:root {
  --bg: #0d1117;
  --bg-surface: #161b22;
  --bg-card: #1c2128;
  --border: #30363d;
  --text: #ffffff;
  --text-muted: #8b949e;
  --accent: #58a6ff;
  --accent-dim: #1f6feb;
  --green: #3fb950;
  --red: #f85149;
  --yellow: #d29922;
  --radius: 8px;
  --header-h: 56px;
}
```

Font stack: `-apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif`
Monospace: `"SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace`

All apps use `lang="fr"` (French interface language per project convention).

---

## 13. License

AGPL-3.0-or-later. All source files include `// SPDX-License-Identifier: AGPL-3.0-or-later` header. License file at `LICENSE` in project root.

---

## 14. Key File Paths Reference

### Bridge SDK
- `web/public/sbfb-bridge.js` -- canonical bridge SDK (398 lines)
- `web/src/bridge/protocol.ts` -- Zod schemas for bridge messages (176 lines)
- `web/src/bridge/useBridge.ts` -- host-side React hook (366 lines)
- `scripts/sync-bridge-sdk.sh` -- sync script to example apps

### Blob-Serve
- `crates/nexus-shell-daemon-core/src/blob_serve.rs` -- cache + CSP + content-type (484 lines)
- `crates/nexus-test-harness/tests/blob_serve_coep.rs` -- E2E header test

### Deploy
- `crates/nexus-shell-daemon/src/deploy.rs` -- verified + private deploy (753 lines)
- `crates/nexus-coordinator-rs/src/provenance.rs` -- provenance SLSA L1 (212 lines)

### Example Apps
- `examples/sbfb-explorer/` -- Protocol Explorer (5 files)
- `examples/sbfb-ideas/` -- Ideas Hub (5 files)
- `examples/hello-world-app/` -- legacy Python SDK example (2 files)

### Bridge Tests
- `web/tests/bridge-heartbeat.spec.ts` -- Playwright heartbeat E2E
- `web/tests/bridge-push-event.spec.ts` -- Playwright push event E2E
- `web/tests/bridge-pii-redact.spec.ts` -- Playwright PII redaction E2E
- `web/src/bridge/__tests__/protocol.test.ts` -- Vitest schema tests
- `web/src/bridge/__tests__/useBridge.test.ts` -- Vitest hook tests
- `web/src/bridge/__tests__/watchdog.test.ts` -- Vitest watchdog tests

### Documentation
- `docs/apps/REACT_MIGRATION.md` -- React migration guide
- `docs/architecture/LAUNCHER.md` -- launcher architecture
- `docs/architecture/PUBLISH_MODEL.md` -- 4 publish states
- `docs/protocol/PUBLIC_FEED_SPEC.md` -- feed protocol spec
- `docs/security/THREAT_MODEL.md` -- STRIDE/LINDDUN threat model
- `docs/security/RUNTIME_ISOLATION.md` -- VM isolation roadmap
- `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` -- 3-tier trust
- `docs/shell/PATTERNS.md` -- shell/coordinator patterns
- `docs/rust/PATTERNS.md` -- Rust patterns

---

*Apps, Bridge, and Docs analysis: 2026-05-18*
