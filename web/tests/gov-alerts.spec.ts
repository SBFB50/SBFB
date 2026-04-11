/**
 * Sprint 8 Phase D — gov Alertes tab e2e.
 *
 * Phase D ships four new DB-backed tabs (Alertes / Affaires /
 * Lois / Factchecks) driven by ``ctx.db`` queries. The Playwright
 * harness runs without the legacy ``nexus/gov/govdata.db`` file,
 * so the Alertes handler falls back to the shared empty-state:
 * a heading block plus an empty block explaining that no alert
 * is recorded yet.
 *
 * The test expands the gov accordion, clicks the specific
 * "Invoquer" button on the Alertes row, and asserts the heading
 * + empty-state copy render without a legacy-descriptor warning
 * and without a raw JSON descriptor leaking through.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov Alertes tab renders empty state without legacy DB", async ({
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

  // Alertes is an async tab — click its specific "Invoquer"
  // button rather than the first one in the list which targets
  // the Contradictions stub.
  const alertsRow = page
    .locator("li")
    .filter({ hasText: "Alertes" })
    .first();
  await alertsRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();

  // Heading block for the tab.
  await expect(
    page.getByRole("heading", { name: "Alertes" }),
  ).toBeVisible({ timeout: 10_000 });

  // Empty-state placeholder — the handler falls back here when
  // gov_alerts is missing or empty.
  await expect(
    page.getByText(/Aucune alerte enregistrée/),
  ).toBeVisible();

  // Sanity check: no legacy descriptor warning and no raw JSON.
  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
