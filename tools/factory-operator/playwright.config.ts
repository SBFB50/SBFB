// SPDX-License-Identifier: AGPL-3.0-or-later
import os from 'node:os'
import path from 'node:path'
import { defineConfig, devices } from '@playwright/test'
import { OPERATOR_TEST_PORT, OPERATOR_TEST_TOKEN } from './e2e/fixtures'

// Sprint 80 Phase B — hermetic T1 skeleton. `serve-operator.mjs` builds
// the greenfield front and spawns the REAL Operator Rust server over the
// BUILT bundle (not `vite dev`), so the spec exercises the production
// cookie-auth + the self-origin CSP. SBFB_AUTH_TOKEN + SBFB_HOME are
// injected here (no real ~/.sbfb is touched). The further sub-tests
// (composeur→session, SSE single-Done, MUR, diff-viewer) land in
// Phases C / H / I.
export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL: `http://127.0.0.1:${OPERATOR_TEST_PORT}`,
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: 'node e2e/serve-operator.mjs',
    url: `http://127.0.0.1:${OPERATOR_TEST_PORT}/`,
    timeout: 180_000,
    reuseExistingServer: false,
    stdout: 'pipe',
    stderr: 'pipe',
    env: {
      OPERATOR_TEST_PORT: String(OPERATOR_TEST_PORT),
      SBFB_AUTH_TOKEN: OPERATOR_TEST_TOKEN,
      SBFB_HOME: path.join(os.tmpdir(), 'sbfb-operator-e2e'),
    },
  },
})
