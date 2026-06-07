# {{name}}

SBFB **Python / Pyodide** app (v{{version}}) — **EXPERIMENTAL scaffold**.

> **Heads up:** this template does **not run yet** in the SBFB sandbox. The app
> CSP (`connect-src 'none'` + an opaque iframe origin) blocks the `fetch()` /
> `WebAssembly.instantiateStreaming` that Pyodide uses to load its runtime, and
> no CDN can be reached. A working Python app needs the **extended hosting mode**
> for verified apps — a future (post-S74) capability that vendors the Pyodide
> runtime same-origin under a relaxed CSP.

This scaffold is a starting point for that future mode: `index.html` carries the
Pyodide wiring (commented) and shows an honest banner when the runtime is
unavailable.

## Structure

- `index.html` — entry point with the (currently inert) Pyodide wiring
- `sbfb-bridge.js` — SBFB Bridge SDK (communication with the network)
- `SBFB.json` — app manifest (schema v2)
- `factory.template.lock` — generation metadata

## Status

Until the extended hosting mode ships, prefer the `static`, `static-reader` or
`react` templates for apps that must run in today's sandbox.

## Validate

```bash
sbfb-factory validate .
```
