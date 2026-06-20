// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Page render smokes (hermetic) — catch-up E2E coverage.
 *
 * Every navigable shell page must render its REAL content against a real
 * (empty) daemon with the auto-registered local coordinator: no crash, no
 * global RouteErrorBoundary. These are the honest browser-level mirror of
 * the Vitest page smokes — they exercise the actual route + daemon round
 * trips (`/api/daemon/*`) the unit smokes mock away.
 */

import { test, expect } from "./fixtures";

const ERROR_BOUNDARY = "La page a crashé";

test.describe("shell pages render against a real empty daemon", () => {
  test("/my-projects lists the auto-registered local node", async ({
    page,
  }) => {
    await page.goto("/my-projects");
    await expect(
      page.getByRole("heading", { name: "Mes projets" }),
    ).toBeVisible();
    await expect(page.getByText(ERROR_BOUNDARY)).toHaveCount(0);
  });

  test("/my-network renders the contribute panel", async ({ page }) => {
    await page.goto("/my-network");
    // offer-power-card is the Network-specific, ASCII-stable anchor (the
    // page heading carries an accent whose NFC/NFD form is not worth
    // asserting on).
    await expect(page.getByTestId("offer-power-card")).toBeVisible();
    await expect(page.getByText(ERROR_BOUNDARY)).toHaveCount(0);
  });

  test("/nodes shows the cold-start when no node directory is known", async ({
    page,
  }) => {
    await page.goto("/nodes");
    await expect(page.getByRole("heading", { name: "Noeuds" })).toBeVisible();
    await expect(page.getByTestId("nodes-cold-start")).toBeVisible();
    await expect(page.getByText(ERROR_BOUNDARY)).toHaveCount(0);
  });

  test("/deploy renders the publish form (submit gated until filled)", async ({
    page,
  }) => {
    await page.goto("/deploy");
    await expect(
      page.getByRole("heading", { name: "Publier une app" }),
    ).toBeVisible();
    await expect(page.getByTestId("repo-url")).toBeVisible();
    await expect(page.getByTestId("deploy-submit")).toBeDisabled();
    await expect(page.getByText(ERROR_BOUNDARY)).toHaveCount(0);
  });
});
