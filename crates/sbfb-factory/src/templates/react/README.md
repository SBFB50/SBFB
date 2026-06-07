# {{name}}

SBFB **React** app (v{{version}}) created with sbfb-factory.

Renders with React 18 — **no build step, no CDN**. React, ReactDOM and htm are
vendored same-origin so the app runs under the SBFB sandbox CSP
(`default-src 'self'; connect-src 'none'`): the iframe cannot fetch from a CDN,
so the runtime ships inside the archive.

## Structure

- `index.html` — entry point (htm + `React.createElement`, no JSX transform)
- `react.production.min.js`, `react-dom.production.min.js` — React 18 UMD (MIT)
- `htm.umd.js` — htm (Apache-2.0), JSX-like syntax without a build
- `sbfb-bridge.js` — SBFB Bridge SDK (communication with the network)
- `SBFB.json` — app manifest (schema v2)
- `factory.template.lock` — generation metadata

## Development

Edit `index.html`. Open it in a browser for a rough local preview; the bridge
methods require the SBFB shell iframe host.

## Vendored runtime

`react*.js` and `htm.umd.js` are third-party (MIT / Apache-2.0); their license
headers are preserved. Update them by replacing the files and re-running gates.

## Validate

```bash
sbfb-factory validate .
```
