/**
 * Sprint 5 Phase D — /browse and /curators stub pages.
 *
 * Both pages are intentionally minimal Sprint 6 placeholders
 * per decision D4. This spec just asserts the routes are
 * reachable (no 404), render a recognisable card, and mention
 * Sprint 6 in the copy so a future reader knows these are
 * scope-cut pages and not forgotten TODOs.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(
    ([url, nickname]) => {
      window.localStorage.setItem(
        "nexus-grid:shell:v1",
        JSON.stringify({
          state: {
            knownCoordinators: [{ url, nickname, nodeId: null }],
            activeCoordinatorUrl: url,
          },
          version: 0,
        }),
      );
    },
    [TEST_COORD_URL, TEST_COORD_NAME],
  );
});

test("/browse renders a Sprint 6 stub, not a 404", async ({ page }) => {
  await page.goto("/browse");
  await expect(
    page.getByRole("heading", { name: "Explorer" }),
  ).toBeVisible();
  await expect(page.getByText(/Arrive Sprint 6/i)).toBeVisible();
  await expect(page.getByText(/DHT iroh-pkarr/)).toBeVisible();
});

test("/curators renders a Sprint 6 stub, not a 404", async ({ page }) => {
  await page.goto("/curators");
  await expect(
    page.getByRole("heading", { name: "Curators" }),
  ).toBeVisible();
  await expect(page.getByText(/Arrive Sprint 6/i)).toBeVisible();
  // The stub mentions the iroh-blobs gossip flow + Ed25519 signing
  // in its explanation copy. Match a phrase that's stable across
  // Sprint 6 design iterations.
  await expect(page.getByText(/gossip/i).first()).toBeVisible();
});
