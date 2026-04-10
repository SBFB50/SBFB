/**
 * Sprint 5 Phase B — apps tab render.
 *
 * Expands the hello-world app row in the Apps tab, confirms the
 * manifest fetch happened, and asserts the JSON descriptor for
 * the "Hello" tab contains the expected `description` field.
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

  // The hello tab descriptor JSON contains the literal string
  // "Hello world" as its description. Playwright's getByText is
  // substring by default.
  await expect(page.getByText(/"description":\s*"Hello world"/)).toBeVisible({
    timeout: 10_000,
  });
});
