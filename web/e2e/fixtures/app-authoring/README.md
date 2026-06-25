# app-authoring CSP self-check fixtures (Sprint 79 Phase H)

Two minimal SBFB app archives that drive the **runtime** CSP self-check
(`web/e2e/app-authoring.spec.ts`, the T1 gate). They are replayed inside the
production iframe host (`BrowsedProject`, `sandbox="allow-scripts"` without
`allow-same-origin`, opaque origin) under the **real** `BLOB_SERVE_CSP` the
daemon serves, and the browser-level console is watched for CSP violations.

| Fixture | Role | Runtime behaviour |
|---|---|---|
| `clean.zip` | positive control | inline DOM mutation only — **zero** CSP violation |
| `dirty.zip` | negative control | `fetch(atob("…"))` to an external host — violates `connect-src 'none'` |

The **dirty** fixture assembles its network target with `atob()` at runtime, so
the *static* authoring gate (`run_gate_csp_authoring` / `check-csp.mjs`, both
string scans) cannot see the URL. The runtime self-check catches it at browser
level. Without this negative control the BLOQUANT gate would only prove "the
harness runs", not "it detects" — a vacuous gate (README §4).

## Why committed binaries

`POST /api/daemon/publish-blob` needs a real `.zip` body the daemon
decompresses with the `zip` crate. Neither Node nor Playwright ships a native
zip writer, and the runtime-0-dep rule forbids adding `jszip`/`fflate`. So the
archives are committed and seeded as-is.

## Regenerate

Edit a source under `src/{clean,dirty}/` then rebuild (Node built-ins only,
deterministic):

```
node web/e2e/fixtures/app-authoring/build-fixtures.mjs
```

The builder asserts the dirty fixture's `atob()` target still decodes to an
external `https://` URL, so the fixture can never silently stop exercising the
violation.
