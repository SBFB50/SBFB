/**
 * Playwright config — Sprint 5 Phase B.
 *
 * The suite spawns a real nexus-coordinator subprocess via
 * `tests/fixtures.ts` and runs the vite dev server over it,
 * so no mocks anywhere. Chromium-only to keep the install
 * footprint small; other browsers can be added later.
 */

import { defineConfig, devices } from "@playwright/test";

// Sprint 16 Phase A (D1): fixed loopback bearer token the
// coordinator child process + the browser both use. Hard-coded
// here so the config file doesn't import from global-setup.ts
// (which would import node:fs and blow up when playwright loads
// the config on a fresh worker).
const TEST_AUTH_TOKEN =
  "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false, // single coordinator at a time
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  timeout: 60_000,
  reporter: [["list"]],
  globalSetup: "./tests/global-setup.ts",
  globalTeardown: "./tests/global-teardown.ts",

  use: {
    baseURL: "http://127.0.0.1:5173",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    viewport: { width: 1440, height: 900 },
    locale: "fr-FR",
    // Browser-level injection of the bearer + a loopback Origin
    // header so the page's own fetch()s pass the coordinator's
    // auth middleware without every spec having to seed the
    // token into window. Tests that probe 401/403 paths override
    // this per-request with `page.request(url, { headers })`.
    extraHTTPHeaders: {
      "x-sbfb-token": TEST_AUTH_TOKEN,
    },
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  webServer: {
    command: "npm run dev -- --host 127.0.0.1 --port 5173",
    url: "http://127.0.0.1:5173",
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
