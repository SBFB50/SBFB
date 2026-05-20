# {{name}}

SBFB app (v{{version}}) created with sbfb-factory.

## Structure

- `index.html` — entry point
- `sbfb-bridge.js` — SBFB Bridge SDK (communication with the network)
- `SBFB.json` — app manifest (schema v2)
- `factory.template.lock` — generation metadata

## Development

Open `index.html` in a browser for local preview.
The bridge methods require the SBFB shell iframe host.

## Validate

```bash
sbfb-factory validate .
```
