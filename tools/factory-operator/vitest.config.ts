// SPDX-License-Identifier: AGPL-3.0-or-later
import path from 'node:path'
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

// Dedicated Vitest config (no React Compiler babel preset — tests run
// without the compiler transform; a fixture that mutates state opts out
// with `"use no memo"`, Day-0 D1). Unit suites land Phase I (re-covering
// the single-Done SSE intention PO-14 via `useTokenStream`).
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    css: false,
    include: ['src/**/*.test.{ts,tsx}'],
    coverage: {
      // `all: true` makes untested source files visible in the report (honest
      // debt) instead of inflating the % over only-imported modules. The
      // presentational components (Rail, OrientationBar, SteerScene, Atelier,
      // VerifyPlaceholder) are exercised by the Playwright E2E, not Vitest.
      all: true,
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/**/*.test.{ts,tsx}', 'src/test/**', 'src/main.tsx', 'src/vite-env.d.ts'],
    },
  },
})
