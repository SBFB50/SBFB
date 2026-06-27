// SPDX-License-Identifier: AGPL-3.0-or-later
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { defineConfig } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
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

export default defineConfig({
  plugins: [
    react(),
    babel({ presets: [reactCompilerPreset()] }),
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
          if (!nid.includes('/node_modules/')) return
          if (
            nid.includes('/node_modules/react/') ||
            nid.includes('/node_modules/react-dom/') ||
            nid.includes('/node_modules/scheduler/')
          ) {
            return 'vendor-react'
          }
        },
      },
    },
  },
  server: {
    port: 5174,
    proxy: {
      '/api/terminal/ws': {
        target: 'ws://127.0.0.1:3001',
        ws: true,
        configure: (proxy) => {
          proxy.on('proxyReqWs', (proxyReq) => {
            if (OPERATOR_TOKEN) proxyReq.setHeader(TOKEN_HEADER, OPERATOR_TOKEN)
          })
        },
      },
      '/api': {
        target: 'http://127.0.0.1:3001',
        configure: (proxy) => {
          proxy.on('proxyReq', (proxyReq) => {
            if (OPERATOR_TOKEN) proxyReq.setHeader(TOKEN_HEADER, OPERATOR_TOKEN)
          })
        },
      },
    },
  },
})
