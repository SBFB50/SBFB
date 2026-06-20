// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Shared Playwright fixtures for the SBFB shell E2E.
 *
 * `seededAuth` (auto fixture): pre-seeds `window.__SBFB_AUTH_TOKEN`
 * before the app boots so `main.tsx` takes the `primeAuthToken`
 * branch (no live launcher needed). The seeded value matches the
 * hermetic daemon's `SBFB_AUTH_TOKEN` (set in `tests/global-setup.ts`),
 * so same-origin `/api/daemon/*` calls authenticate.
 *
 * In external-daemon mode (`SBFB_E2E_BASE_URL` set — the compute
 * flagship), the token is NOT seeded: the operator daemon has its own
 * token and the app resolves it same-origin via `/auth/token`. Seeding
 * the test token there would force a wrong bearer and 401 every call.
 *
 * `computeFrame(page)` returns the FrameLocator for the sandboxed app
 * iframe (`data-testid="remote-iframe-element"`, opaque origin — a raw
 * `page.locator` cannot pierce it, only a frameLocator can).
 */

import {
  test as base,
  expect,
  type Page,
  type FrameLocator,
} from "@playwright/test";

import { TEST_AUTH_TOKEN } from "../tests/global-setup";

export const test = base.extend<{ seededAuth: void }>({
  seededAuth: [
    async ({ page }, use) => {
      if (!process.env.SBFB_E2E_BASE_URL) {
        await page.addInitScript((token) => {
          (
            window as unknown as { __SBFB_AUTH_TOKEN?: string }
          ).__SBFB_AUTH_TOKEN = token;
        }, TEST_AUTH_TOKEN);
      }
      await use();
    },
    { auto: true },
  ],
});

export { expect };

/** FrameLocator for the sandboxed remote-app iframe. */
export function computeFrame(page: Page): FrameLocator {
  return page.frameLocator('[data-testid="remote-iframe-element"]');
}
