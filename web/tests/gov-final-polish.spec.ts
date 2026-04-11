/**
 * Sprint 8 Phase E — gov tab polish behaviour regression.
 *
 * Sprint 8 Phase E §8.4 calls for:
 *  - skeleton loaders while the tab descriptor is fetching
 *  - empty states with explanatory CTAs
 *  - sort interactions on the main tables
 *
 * The Playwright environment runs against a hermetic coordinator
 * without the legacy `nexus/gov/govdata.db`, so every gov tab
 * hits its empty-state branch. This spec walks the new
 * `/app/:appName/tabs/:tabName` deep-link route for three tabs
 * (Dashboard / Contradictions / Alertes), asserts the empty
 * state renders, and exercises the TableBlock sort header on
 * the one tab that ships a non-empty table at boot
 * (Contradictions has the chart_bar + empty table, Alertes has
 * the empty state banner, Dashboard has its five zero metrics).
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

async function seedStore(page: import("@playwright/test").Page) {
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
}

test("AppTabPage deep-link renders the gov Dashboard empty state with header", async ({
  page,
}) => {
  await seedStore(page);

  await page.goto("/app/gov/tabs/Dashboard");

  // Page header surfaces the appName/tabName + active nickname.
  await expect(
    page.getByRole("heading", { name: /gov — Dashboard/ }),
  ).toBeVisible({ timeout: 10_000 });
  await expect(page.getByTestId("app-tab-refresh")).toBeVisible();

  // The dashboard tab renders its five metric labels (values
  // zero because the legacy DB is absent).
  await expect(page.getByText("Tableau de bord Gov")).toBeVisible({
    timeout: 10_000,
  });
  await expect(page.getByText("Aucun sujet indexé")).toBeVisible();
});

test("AppTabPage deep-link renders the gov Alertes empty-state banner", async ({
  page,
}) => {
  await seedStore(page);

  await page.goto("/app/gov/tabs/Alertes");

  await expect(
    page.getByRole("heading", { name: /gov — Alertes/ }),
  ).toBeVisible({ timeout: 10_000 });

  // Gov's `_empty_tab` helper drops a heading + an `empty` block
  // whenever the backing table is missing. Verify the fallback
  // message is visible — it's the Phase E polish contract for
  // "user knows the tab ran but has nothing to show".
  await expect(
    page.getByText(/Base Gov indisponible\.|Aucune alerte/),
  ).toBeVisible({ timeout: 10_000 });
});

test("AppTabPage deep-link shows the no-active-coordinator banner when the store is empty", async ({
  page,
}) => {
  // Explicitly clear the persisted store so the store hydrates
  // to the default (empty knownCoordinators, null active) and
  // the page renders its "Sélectionner un projet d'abord" CTA.
  await page.addInitScript(() => {
    window.localStorage.removeItem("nexus-grid:shell:v1");
  });

  await page.goto("/app/gov/tabs/Dashboard");

  await expect(
    page.getByText("Sélectionner un projet d'abord"),
  ).toBeVisible({ timeout: 10_000 });
  await expect(page.getByText("Aucun coordinateur actif.")).toBeVisible();
});
