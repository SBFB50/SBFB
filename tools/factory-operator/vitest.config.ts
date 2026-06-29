// SPDX-License-Identifier: AGPL-3.0-or-later
import path from 'node:path'
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import { lingui, linguiTransformerBabelPreset } from '@lingui/vite-plugin'

// Dedicated Vitest config (no React Compiler babel preset — tests run
// without the compiler transform; a fixture that mutates state opts out
// with `"use no memo"`, Day-0 D1). The Lingui plugins ARE shared with the
// build: `lingui()` compiles the `.po` catalogs a test imports, and
// `linguiTransformerBabelPreset()` expands the <Trans>/t macros so any tested
// component/lib using them runs (App renders <Trans>, gateStatusLabel uses t).
export default defineConfig({
  plugins: [
    react(),
    lingui(),
    babel({ presets: [linguiTransformerBabelPreset()] }),
  ],
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
      // VerifyScene, Terminal) are exercised by the Playwright E2E, not Vitest.
      // The xterm-mounting modules (TerminalXterm, CastXterm) need a real DOM /
      // canvas and are E2E-only by nature — excluded from coverage rather than
      // reported as permanent 0% debt.
      all: true,
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/*.test.{ts,tsx}',
        'src/test/**',
        'src/main.tsx',
        'src/vite-env.d.ts',
        'src/components/verify/TerminalXterm.tsx',
        'src/components/surfaces/CastXterm.tsx',
      ],
    },
  },
})
