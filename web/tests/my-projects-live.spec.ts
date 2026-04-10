/**
 * Sprint 5 Phase B — /my-projects live view.
 *
 * Pre-seeds localStorage with the test coordinator and verifies
 * the card renders with a live "En ligne" badge driven by a
 * successful /health poll.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("my-projects shows a live card for the known coordinator", async ({
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

  await page.goto("/my-projects");

  await expect(
    page.getByRole("heading", { name: "Mes projets" }),
  ).toBeVisible();

  // The nickname (pw-demo) shows up as the card title
  await expect(page.getByText(TEST_COORD_NAME).first()).toBeVisible();
  await expect(page.getByText(TEST_COORD_URL).first()).toBeVisible();

  // And the live health badge should flip to "En ligne" within a
  // few seconds of the first refetch.
  await expect(page.getByText("En ligne")).toBeVisible({
    timeout: 10_000,
  });
});
