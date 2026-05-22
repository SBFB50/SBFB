# {{name}}

SBFB Reader app (v{{version}}) created with sbfb-factory.

## Structure

- `index.html` — reader entry point (sections array + prev/next navigation)
- `sbfb-bridge.js` — SBFB Bridge SDK (communication with the network)
- `SBFB.json` — app manifest (schema v2, category "content")
- `factory.template.lock` — generation metadata

## Adding content

Edit the `sections` array in `index.html`. Each section has a `title`
and `content` (HTML string). Add as many sections as needed.

## Features

- Keyboard navigation (arrow keys)
- Reading position saved via SBFB bridge storage
- Dark theme, responsive layout

## Validate

```bash
sbfb-factory validate .
```

## Publish

```bash
sbfb-factory publish . --repo-url https://github.com/you/your-app
```
