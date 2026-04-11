/**
 * Sprint 8 Phase B — gov Politiciens tab e2e.
 *
 * The Politiciens tab lists up to 50 politicians pulled from the
 * legacy SQLite schema. In CI the legacy DB is absent so the
 * handler's empty-state branch renders a heading plus an empty
 * block explaining that no politician is referenced yet.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov Politiciens tab renders empty state without legacy DB", async ({
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

  const politiciansRow = page
    .locator("li")
    .filter({ hasText: "Politiciens" })
    .first();
  await politiciansRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();

  // Heading block for the tab.
  await expect(
    page.getByRole("heading", { name: "Politiciens" }),
  ).toBeVisible({ timeout: 10_000 });

  // Empty-state placeholder — the handler falls back here when
  // `politicians_list_query` returns zero rows (missing or empty
  // gov_politicians table).
  await expect(
    page.getByText(/Aucun politicien référencé/),
  ).toBeVisible();

  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
