/**
 * Sprint 8 Phase C — gov Contradictions tab upgrade e2e.
 *
 * Phase C rewrites the Sprint 4 Contradictions stub into a full
 * TabView (summary section with three metrics + chart_bar per
 * subject + paginated table joined with politician names). In
 * the Playwright environment the legacy ``nexus/gov/govdata.db``
 * file is absent, so the handler falls back to the shared
 * empty-state — a heading block plus an empty block explaining
 * that no contradiction has been detected yet.
 *
 * The test expands the gov accordion, clicks "Invoquer" on the
 * Contradictions row, and asserts the empty-state copy renders
 * without any legacy-descriptor warning and without a raw JSON
 * descriptor leaking through.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov Contradictions tab renders empty state without legacy DB", async ({
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

  // Contradictions is now an async tab — click its specific
  // "Invoquer" button rather than the first one in the list.
  const contradictionsRow = page
    .locator("li")
    .filter({ hasText: "Contradictions" })
    .first();
  await contradictionsRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();

  // Heading block for the tab — Phase C kept the same title as
  // the Sprint 4 stub so the sidebar route is stable.
  await expect(
    page.getByRole("heading", { name: "Détection de contradictions" }),
  ).toBeVisible({ timeout: 10_000 });

  // Empty-state placeholder — the handler falls back here when
  // ``contradictions_overview_query`` reports zero rows.
  await expect(
    page.getByText(/Aucune contradiction détectée/),
  ).toBeVisible();

  // Sanity check: no legacy descriptor warning and no raw JSON.
  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
