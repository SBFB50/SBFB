// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Header navigation (hermetic) — catch-up E2E coverage.
 *
 * The shell's left-rail nav links must route between the main sections.
 * Exercises React Router client navigation in the real served bundle
 * (not just a direct goto), against the real daemon.
 */

import { test, expect } from "./fixtures";

test.describe("header navigation", () => {
  test("nav links route between the main sections", async ({ page }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await page.getByRole("link", { name: "Projets" }).click();
    await expect(
      page.getByRole("heading", { name: "Mes projets" }),
    ).toBeVisible();

    await page.getByRole("link", { name: "Publier" }).click();
    await expect(
      page.getByRole("heading", { name: "Publier une app" }),
    ).toBeVisible();

    await page.getByRole("link", { name: "Explorer" }).click();
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    // /my-network auto-opens a GPU-consent dialog on a fresh node, whose
    // overlay would intercept later nav clicks — so visit it LAST.
    await page.getByRole("link", { name: "Reseau" }).click();
    await expect(page.getByTestId("offer-power-card")).toBeVisible();
  });
});
