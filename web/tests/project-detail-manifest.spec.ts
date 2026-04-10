/**
 * Sprint 5 Phase B — /project/:name detail view.
 *
 * Opens the detail page for the test coordinator and verifies
 * that the 5 tabs render, that the Overview tab shows the
 * project identity, and that the Apps tab lists both
 * hello-world-app and nexus-app-gov.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test.beforeEach(async ({ page }) => {
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
});

test("project detail renders the 5 tabs and resolves the coordinator", async ({
  page,
}) => {
  await page.goto(`/project/${TEST_COORD_NAME}`);

  // Header reflects the project name from /project
  await expect(
    page.getByRole("heading", { name: TEST_COORD_NAME }),
  ).toBeVisible();

  // The 5 tabs must be visible
  for (const label of ["Vue d'ensemble", "Tâches", "Kudos", "Invites", "Apps"]) {
    await expect(page.getByRole("tab", { name: label })).toBeVisible();
  }

  // Overview tab content
  await expect(page.getByText("Identité du coordinateur")).toBeVisible();
  await expect(page.getByText(/apps installées/i).first()).toBeVisible();

  // Apps tab: both installed apps visible
  await page.getByRole("tab", { name: "Apps" }).click();
  await expect(page.getByText(/hello/).first()).toBeVisible({ timeout: 10_000 });
  await expect(page.getByText(/gov/).first()).toBeVisible();
});
