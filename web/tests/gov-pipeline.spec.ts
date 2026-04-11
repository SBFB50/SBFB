/**
 * Sprint 8 Phase C — gov Pipeline tab e2e.
 *
 * The Pipeline tab surfaces a status chart_bar plus two
 * sections (``En cours`` and ``Historique récent``) driven by
 * ``gov_scan_log``. In the Playwright environment the legacy
 * ``nexus/gov/govdata.db`` file is absent, so the handler falls
 * back to the shared empty-state — a heading block plus an
 * empty block explaining that no pipeline activity has been
 * recorded yet.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov Pipeline tab renders empty state without legacy DB", async ({
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

  const pipelineRow = page
    .locator("li")
    .filter({ hasText: "Pipeline" })
    .first();
  await pipelineRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();

  // Heading block for the tab.
  await expect(
    page.getByRole("heading", { name: "Pipeline ETL" }),
  ).toBeVisible({ timeout: 10_000 });

  // Empty-state placeholder — the handler falls back here when
  // ``pipeline_state_query`` reports no scan_log rows.
  await expect(
    page.getByText(/Aucune activité pipeline/),
  ).toBeVisible();

  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
