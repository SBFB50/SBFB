// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Browse — top bar, empty grid, per-keystroke search, refresh, cross-nav
 * (hermetic). Deepens browse-empty.spec.ts (single search fill) with the
 * empty-grid structure, the no-debounce multi-keystroke path, the clear →
 * browse transition, the by-node pill navigation, and the refresh cycle.
 */

import { test, expect } from "./fixtures";

test.describe("Browse — top bar + headings", () => {
  test("renders the 'Explorer' heading in browse mode", async ({ page }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Explorer", level: 1 }),
    ).toBeVisible();
  });
});

test.describe("Browse — empty grid", () => {
  test("shows the 'Aucune app' empty-state (no grid, no discovered section) on an empty daemon", async ({
    page,
  }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();
    await expect(page.getByText("Aucune app", { exact: true })).toBeVisible();
    await expect(page.getByTestId("browse-grid")).toHaveCount(0);
    await expect(page.getByTestId("browse-discovered-section")).toHaveCount(0);
  });
});

test.describe("Browse — search (no debounce)", () => {
  test("multi-keystroke search stabilises on the no-result state and clears back to browse", async ({
    page,
  }) => {
    await page.goto("/browse");
    const s = page.getByTestId("browse-search-input");
    await expect(s).toBeVisible();

    await s.fill("a");
    await s.fill("ab");
    await s.fill("abc");

    await expect(page.getByTestId("browse-search-empty")).toBeVisible();
    await expect(page.getByTestId("browse-search-clear")).toBeVisible();

    await page.getByTestId("browse-search-clear").click();
    await expect(s).toHaveValue("");
    await expect(page.getByText("Aucune app", { exact: true })).toBeVisible();
  });
});

test.describe("Browse — cross-page navigation", () => {
  test("the 'Parcourir par noeud' pill navigates to /nodes cold-start", async ({
    page,
  }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    const byNode = page.getByTestId("browse-by-node-link");
    await expect(byNode).toBeVisible();
    await byNode.click();

    await expect(page).toHaveURL(/\/nodes$/);
    await expect(page.getByRole("heading", { name: "Noeuds" })).toBeVisible();
    await expect(page.getByTestId("nodes-cold-start")).toBeVisible();
  });
});

test.describe("Browse — refresh", () => {
  test("the Rafraichir button completes a pull cycle and re-enables without crashing", async ({
    page,
  }) => {
    await page.goto("/browse");
    const refresh = page.getByTestId("browse-refresh");
    await expect(refresh).toBeVisible();

    await refresh.click();
    // Tolerate the 2s deferred refetch + post-fetch settle.
    await expect(refresh).toBeEnabled({ timeout: 15000 });
    await expect(page.getByText("Aucune app", { exact: true })).toBeVisible();
    await expect(page.getByText("La page a crashé")).toHaveCount(0);
  });
});
