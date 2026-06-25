# {{name}}

SBFB **daisyUI + anime.js** app (v{{version}}) created with sbfb-factory.

daisyUI gives the **structure** (components), anime.js gives the **motion**. Both
are CSP-safe **by construction**: the runtime archive ships only compiled,
vendored, same-origin assets — **no CDN, no runtime fetch, no build at runtime**.

## Why this is not the React no-build template

The React template renders with vendored UMD scripts and **no build step**.
daisyUI is different: Tailwind utilities + daisyUI components must be **compiled
ahead of time** into `app.css`. So this template **has a build step**
(`npm run build:css`), but its **output** (`app.css`) is a static, same-origin
stylesheet — the published archive still has **zero runtime dependency**.

## Structure

- `index.html` — entry point. `<link rel="stylesheet" href="app.css">` (classic),
  `<script src="vendor/anime.umd.js">` loaded **before** `app.js` (classic,
  never `type="module"`), `data-theme="sbfb-reflect"`.
- `app.js` — interaction layer; uses the global `window.anime` (anime.js v4).
- `app.css` — **compiled** Tailwind v4 + daisyUI output (committed, runtime asset).
- `src/input.css` — build source. Imports Tailwind, scopes `@source` to the two
  authored files, loads daisyUI with **`themes: false`** (none of the 35 built-in
  themes — 0/35) plus the single custom `sbfb-reflect` oklch theme. **Lean.**
- `vendor/anime.umd.js` — anime.js v4.5.0 UMD bundle, vendored same-origin (MIT).
- `scripts/vendor-anime.mjs` — re-vendors `anime.umd.js` from node_modules.
- `package.json` — **build-time** devDependencies, pinned to exact resolved
  versions (no carets): daisyUI 5.5.23, Tailwind 4.3.1, anime.js 4.5.0.
- `SBFB.json` — app manifest (schema v2).

## CSP doctrine (why it stays sandbox-safe)

The SBFB sandbox serves untrusted content under `connect-src 'none'` + COEP
`require-corp` at an **opaque origin**. Consequences this template respects:

- **No CDN, no remote fetch.** Tailwind-CDN and Google Fonts are forbidden — the
  iframe cannot reach them anyway. CSS and JS are compiled/vendored into the zip.
- **Classic scripts only.** `<script type="module">` is fetched in CORS mode,
  which an opaque-origin document cannot satisfy for its own assets. anime.js is
  loaded as a classic `<script src>`.
- **No worker, no fetch, no remote `url()`** in authored code — these are blocked
  at runtime *and* rejected by the Factory authoring gate before publish.

`node_modules/` is excluded by `.gitignore`, so it never reaches the archive; any
stray build residue is caught by the gate (FAIL), not silently shipped.

## Development

```bash
npm install          # build-time tooling only
npm run build        # vendor anime.js + compile app.css
sbfb-factory validate .
```

Edit `index.html` (daisyUI markup) and `app.js` (anime.js motion), then re-run
`npm run build:css`. Open `index.html` in a browser for a rough preview.

## Vendored runtime

`vendor/anime.umd.js` is third-party (MIT); its license header is preserved.
Update anime.js by bumping `animejs` in `package.json` and running
`npm run vendor:anime`, then re-running the gates.
