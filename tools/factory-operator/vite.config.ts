// SPDX-License-Identifier: AGPL-3.0-or-later
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { defineConfig } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import { lingui, linguiTransformerBabelPreset } from '@lingui/vite-plugin'
import tailwindcss from '@tailwindcss/vite'

// Sprint 80 Phase B — greenfield scaffold (preflight verdict PLAN-ADAPT).
//
// ADAPT-1: `@vitejs/plugin-react` v6 dropped Babel for oxc. React
// Compiler (Day-0 D1, GA v1.0) is wired through the SEPARATE
// `@rolldown/plugin-babel` with `reactCompilerPreset()` — the legacy
// `react({ babel: { plugins } })` path is dead in v6. Confirmed against
// the official vite-plugin-react README (context7, 2026-06-27).
//
// CSP discipline (Operator self-origin CSP set Phase A `a5ace8d`:
// `default-src 'self'; connect-src 'self'`, operator_server.rs:348):
//   - modulePreload.polyfill:false → no inline bootstrap <script>
//   - assetsInlineLimit:0          → no `data:` URIs for small assets
// The hermetic T1 (e2e/boot.spec.ts) asserts 0 CSP violation on the
// BUILT bundle (not `vite dev`, where the CSP is absent).
//
// outDir = `bundle` (NOT the legacy Vite `dist`): the Operator's
// ServeDir is rooted at `tools/factory-operator/bundle`
// (operator_server.rs:47, Phase A contract).

// Dev-only: the Vite proxy is a trusted server-to-server client of the
// Operator (:3001) and injects the bearer header on proxied WS/api so
// `vite dev` works before the cookie bootstrap exists. In prod the SPA
// is served by the Operator's ServeDir and authenticates via the
// HttpOnly cookie (Phase A) — no proxy. Token source: SBFB_AUTH_TOKEN
// env, then `<SBFB_HOME|~/.sbfb>/auth_token`.
const TOKEN_HEADER = 'x-sbfb-token'
function operatorToken(): string {
  const env = process.env.SBFB_AUTH_TOKEN
  if (env && env.trim()) return env.trim()
  try {
    const home = process.env.SBFB_HOME || path.join(os.homedir(), '.sbfb')
    return fs.readFileSync(path.join(home, 'auth_token'), 'utf8').trim()
  } catch {
    return ''
  }
}
const OPERATOR_TOKEN = operatorToken()

// Dev-only: the Operator's listen port. Defaults to 3001 (its CLI default,
// `operator serve --port`). Override with `OPERATOR_PORT` when 3001 is taken
// by another local process (e.g. a Docker-forwarded container). Committed
// behaviour is unchanged when the env is unset.
const OPERATOR_PORT = (process.env.OPERATOR_PORT || '3001').trim()
const OPERATOR_HTTP = `http://127.0.0.1:${OPERATOR_PORT}`
const OPERATOR_WS = `ws://127.0.0.1:${OPERATOR_PORT}`

export default defineConfig({
  plugins: [
    react(),
    // i18n via Lingui. `lingui()` compiles the `.po` catalogs to eval-free
    // message modules; the Lingui macros (<Trans>, t) are expanded by
    // `linguiTransformerBabelPreset()` grafted onto the EXISTING
    // @rolldown/plugin-babel call — NOT via `react({ babel })`, which is dead
    // under @vitejs/plugin-react v6 (oxc, ADAPT-1 above). Babel runs presets in
    // reverse, so the Lingui macro transform runs BEFORE the React Compiler.
    lingui(),
    babel({ presets: [reactCompilerPreset(), linguiTransformerBabelPreset()] }),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    outDir: 'bundle',
    emptyOutDir: true,
    // CSP `default-src 'self'`: no inline modulepreload polyfill, no
    // `data:` asset URIs.
    modulePreload: { polyfill: false },
    assetsInlineLimit: 0,
    rolldownOptions: {
      output: {
        manualChunks(id) {
          const nid = id.replace(/\\/g, '/')
          // Sprint 80 Phase H: the bespoke diff-viewer (DiffViewer + the
          // in-house word-diff) into its OWN async chunk. Imported by BOTH the
          // VerifyScene hero surface AND the Procédé inspector (fold V2/U7), so
          // hoisting it out keeps the VerifyScene `verify-surface` chunk under
          // its budget (.size-limit.json `verify-surface`, bumped to 96 KB this
          // phase — review P3-g). Measured by .size-limit.json `diff-viewer`;
          // motion-free, so it never drags the Motion engine.
          if (nid.includes('/src/components/verify/plein/')) return 'diff-viewer'
          if (!nid.includes('/node_modules/')) return
          if (
            nid.includes('/node_modules/react/') ||
            nid.includes('/node_modules/react-dom/') ||
            nid.includes('/node_modules/scheduler/')
          ) {
            return 'vendor-react'
          }
          // i18n engine (@lingui/core + @lingui/react) into its OWN chunk. The
          // <I18nProvider> wraps the root so this chunk is EAGER, but isolating
          // it keeps the i18n cost measured by its own `vendor-i18n`
          // size-limit entry instead of silently inflating the `app` hero
          // (design doc §1.9 #3 — PO-chosen over bumping `app`). React is
          // already claimed above, so its modules never land here.
          if (nid.includes('/node_modules/@lingui/')) {
            return 'vendor-i18n'
          }
          // Sprint 80 Phase D: xterm (~345 kB) into its OWN chunk. Only the
          // lazy <TerminalXterm>/<CastXterm> import it, so this chunk stays
          // ASYNC (loaded when the operator starts/replays a session) and never
          // bloats the 40 kB `index` hero (.size-limit.json `vendor-xterm`).
          if (nid.includes('/node_modules/@xterm/')) {
            return 'vendor-xterm'
          }
        },
      },
    },
  },
  server: {
    port: 5174,
    proxy: {
      '/api/terminal/ws': {
        target: OPERATOR_WS,
        ws: true,
        configure: (proxy) => {
          proxy.on('proxyReqWs', (proxyReq) => {
            if (OPERATOR_TOKEN) proxyReq.setHeader(TOKEN_HEADER, OPERATOR_TOKEN)
          })
        },
      },
      '/api': {
        target: OPERATOR_HTTP,
        configure: (proxy) => {
          proxy.on('proxyReq', (proxyReq) => {
            if (OPERATOR_TOKEN) proxyReq.setHeader(TOKEN_HEADER, OPERATOR_TOKEN)
          })
        },
      },
    },
  },
})
