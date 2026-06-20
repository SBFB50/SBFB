// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Boot + same-origin auto-register (hermetic).
 *
 * The honest core the old vacuous Playwright step never checked: the
 * REAL built shell, served by a REAL daemon over `--web-root`, boots,
 * authenticates, and `bootstrap.ts::autoRegisterLocalCoordinator`
 * (api/bootstrap.ts:46-66) seeds the same-origin node as the active
 * coordinator. Browse then renders its content (the search bar) rather
 * than the "Aucun noeud actif" wall (Browse.tsx:29-43), which returns
 * BEFORE the search bar exists — so a visible search input proves the
 * active coordinator was registered.
 */

import { test, expect } from "./fixtures";

test.describe("shell boot + same-origin auto-register", () => {
  test("boots from the real daemon and lands on Browse content", async ({
    page,
  }) => {
    await page.goto("/browse");

    // BrowseContent rendered (past the active-coordinator guard) ⟹
    // auto-register succeeded against the live daemon.
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    // No offline banner: the daemon answered /api/daemon/browse.
    await expect(page.getByTestId("daemon-offline-banner")).toHaveCount(0);
    // Not the "Aucun noeud actif" wall.
    await expect(page.getByText("Aucun noeud actif")).toHaveCount(0);
  });

  test("does not show the onboarding-empty wall once a node is registered", async ({
    page,
  }) => {
    await page.goto("/my-projects");

    // OnboardingEmpty (Projects.tsx:16-17) only renders when
    // knownCoordinators.length === 0. The auto-register seeded one, so
    // its hero heading must be absent.
    await expect(page.getByText("Bienvenue sur nexus-grid")).toHaveCount(0);
  });
});
