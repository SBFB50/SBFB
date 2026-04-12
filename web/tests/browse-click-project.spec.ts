/**
 * Sprint 11 Phase C — `/browse/:projectId` route + navigation.
 *
 * These specs verify the routing layer without a running daemon.
 * The coordinator spawned by `global-setup.ts` has no daemon, so
 * `/daemon/browse` returns 503 (unavailable). The BrowsedProject
 * page gracefully renders a "project not found" state.
 *
 * Full Browse → click card → app rendering is covered by manual
 * testing (requires a live daemon with browse entries).
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

test("/browse/:projectId route renders without 404", async ({ page }) => {
  await page.goto("/browse/aabbccdd11223344aabbccdd11223344aabbccdd11223344aabbccdd11223344");

  // The route must exist (no "page not found" from the router).
  // The back link is always rendered regardless of data state.
  const backLink = page.getByTestId("back-to-browse");
  await expect(backLink).toBeVisible({ timeout: 10_000 });

  // The page renders a "project not found" card because the daemon
  // is offline and no browse entries match the fake project id.
  await expect(
    page.getByText(/Projet introuvable/),
  ).toBeVisible({ timeout: 10_000 });
});

test("/browse/:projectId back link navigates to /browse", async ({ page }) => {
  await page.goto("/browse/aabbccdd11223344aabbccdd11223344aabbccdd11223344aabbccdd11223344");

  const backLink = page.getByTestId("back-to-browse");
  await expect(backLink).toBeVisible({ timeout: 10_000 });
  await backLink.click();

  // After clicking back, we land on /browse which shows the
  // page header "Explorer".
  await expect(
    page.getByRole("heading", { name: "Explorer" }),
  ).toBeVisible({ timeout: 10_000 });
});

test("/browse/:projectId does not crash the error boundary", async ({ page }) => {
  await page.goto("/browse/0000000000000000000000000000000000000000000000000000000000000000");

  // The page must NOT show the RouteErrorBoundary fallback.
  await expect(page.getByText(/La page a crashé/)).toHaveCount(0);

  // The back link proves the page rendered correctly.
  await expect(page.getByTestId("back-to-browse")).toBeVisible({
    timeout: 10_000,
  });
});
