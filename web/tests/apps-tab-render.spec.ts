/**
 * Sprint 5 Phase B / Sprint 6 Phase B — apps tab render.
 *
 * Expands the hello-world app row in the Apps tab, confirms the
 * manifest fetch happened, clicks « Invoquer » on the Hello tab,
 * and asserts the schema-driven renderer renders the ported
 * heading block ("Bienvenue sur hello-world-app").
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("apps tab renders the hello-world manifest and descriptor", async ({
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

  // Wait for the apps list to load, then click the hello accordion
  // by its title to expand it.
  await expect(page.getByText(/hello/).first()).toBeVisible({ timeout: 10_000 });

  // Click the card header containing "hello" to toggle the
  // accordion. The header is the clickable target that wraps the
  // title + badges.
  await page.getByText("hello", { exact: true }).click();

  // Routes section must appear
  await expect(page.getByText("Routes").first()).toBeVisible({
    timeout: 5_000,
  });

  // Sprint 6 Phase B: the Hello tab now returns a TabView. The
  // descriptor is lazy-loaded when the user clicks « Invoquer ».
  await page.getByRole("button", { name: /Invoquer|Recharger/ }).first().click();

  // Renderer must show the ported heading block.
  await expect(
    page.getByText("Bienvenue sur hello-world-app"),
  ).toBeVisible({ timeout: 10_000 });

  // Sanity check: the raw JSON fallback is NOT shown.
  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
