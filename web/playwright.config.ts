// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Playwright E2E configuration (process-evolution Commit 2).
 *
 * Decision 3 (PO): the E2E runs against a REAL `nexus-shell-daemon`,
 * never a mock — a mock daemon recreates the very vacuity the old
 * no-op Playwright step had. `tests/global-setup.ts` spawns the
 * daemon against a hermetic dir and serves the freshly-built shell
 * (`web/dist`) via `--web-root`, so the app and its `/api/daemon/*`
 * routes share a single loopback origin (Host/Origin checks pass).
 *
 * `testDir` is `./e2e` — deliberately OUTSIDE `src/` (Vitest claims
 * `src/**` and `scan-en-strings.sh` scans `src/`) and OUTSIDE
 * `tests/` (which holds the global setup/teardown). The e2e tree is
 * therefore invisible to both Vitest and the FR string scanner.
 *
 * Two run profiles, both single-worker (the daemon is a strict
 * singleton — frozen Day-0 decision):
 *   - hermetic (default, CI-safe): `npm run test:e2e`
 *     → `--grep-invert @compute`, real daemon, zero browse entries.
 *   - compute flagship (local, gated): `npm run test:e2e:compute`
 *     → needs SBFB_E2E_COMPUTE=1 + SBFB_E2E_PROJECT_ID + an external
 *       daemon (SBFB_E2E_BASE_URL) with Ollama and the compute-tester
 *       app deployed. Mirrors `scripts/acceptance/phase_h_compute_local.sh`.
 *
 * When `SBFB_E2E_BASE_URL` is set, the global setup does NOT spawn a
 * daemon (the operator already runs one); `baseURL` points at it.
 */

import { defineConfig, devices } from "@playwright/test";

import { TEST_COORD_URL } from "./tests/global-setup";

const BASE_URL = process.env.SBFB_E2E_BASE_URL ?? TEST_COORD_URL;
const IS_CI = !!process.env.CI;

export default defineConfig({
  testDir: "./e2e",
  // The daemon is a singleton: never run specs in parallel against it.
  fullyParallel: false,
  workers: 1,
  forbidOnly: IS_CI,
  retries: IS_CI ? 1 : 0,
  reporter: IS_CI ? [["list"], ["html", { open: "never" }]] : "list",
  globalSetup: "./tests/global-setup.ts",
  globalTeardown: "./tests/global-teardown.ts",
  timeout: 60_000,
  expect: { timeout: 10_000 },
  use: {
    baseURL: BASE_URL,
    locale: "fr-FR",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
