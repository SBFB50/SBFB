/**
 * Sprint 8 Phase B — gov Dashboard tab e2e.
 *
 * The gov app ships seven tabs after Phase B: one legacy
 * Contradictions stub plus six Batch 1 read-only tabs driven by
 * ``ctx.db`` queries. In the Playwright environment the legacy
 * ``nexus/gov/govdata.db`` file is absent, so every Batch 1 tab
 * hits its empty-state branch — ``dashboard_tab`` falls back to
 * a TabView carrying the five metric blocks at zero plus an
 * empty-state notice for the top-subjects chart.
 *
 * The test expands the gov accordion, clicks "Invoquer" on the
 * Dashboard row, and asserts the TabView renders with the
 * expected heading, the five metric labels, and the empty-state
 * placeholder for the chart.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov Dashboard tab renders metrics from AppContext.db", async ({
  page,
}) => {
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

  await page.goto(`/project/${TEST_COORD_NAME}`);
  await page.getByRole("tab", { name: "Apps" }).click();

  await expect(page.getByText(/gov/).first()).toBeVisible({ timeout: 10_000 });
  await page.getByText("gov", { exact: true }).click();

  await expect(page.getByText("Routes").first()).toBeVisible({
    timeout: 5_000,
  });

  // Dashboard is an async tab — click its specific "Invoquer"
  // button rather than the first one in the list, which would
  // target the Contradictions stub.
  const dashboardRow = page.locator("li").filter({ hasText: "Dashboard" }).first();
  await dashboardRow.getByRole("button", { name: /Invoquer|Recharger/ }).click();

  // Heading block for the dashboard.
  await expect(
    page.getByText("Tableau de bord Gov"),
  ).toBeVisible({ timeout: 10_000 });

  // The five metric labels (data is zero in CI since govdata.db
  // is absent, but the labels still render).
  await expect(page.getByText("Politiciens", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("Politiciens actifs")).toBeVisible();
  await expect(page.getByText("Positions", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("Contradictions", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("Partis", { exact: true })).toBeVisible();

  // The empty-state message for the top-subjects chart — this is
  // the fallback when no gov_positions rows exist.
  await expect(
    page.getByText(/Aucun sujet indexé/),
  ).toBeVisible();

  // Sanity check: no legacy descriptor warning and no raw JSON.
  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
