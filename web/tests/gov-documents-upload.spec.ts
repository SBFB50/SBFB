/**
 * Sprint 9 Phase E — gov Documents tab e2e.
 *
 * Navigates to the Documents tab, asserts the empty state with
 * "Aucun document" text and the file_upload drop zone rendered by
 * the v2 TabView. The upload flow (drag a real file) is not
 * exercised end-to-end in this spec because the coordinator's
 * CAS store requires a real filesystem write — that path is
 * covered by the coordinator unit tests in test_files.py.
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

test("gov Documents tab renders empty state with file upload block", async ({
  page,
}) => {
  await seedStore(page);

  await page.goto("/app/gov/tabs/Documents");

  // The page header surfaces the appName/tabName.
  await expect(
    page.getByRole("heading", { name: /gov — Documents/ }),
  ).toBeVisible({ timeout: 10_000 });

  // The empty-state block — no documents uploaded yet.
  await expect(page.getByText("Aucun document")).toBeVisible({
    timeout: 5000,
  });

  // The file_upload drop zone is rendered by FileUploadBlock.
  const dropzone = page.getByTestId("file-upload-dropzone");
  await expect(dropzone).toBeVisible({ timeout: 5000 });

  // The accept list is displayed.
  await expect(page.getByText("image/*, application/pdf")).toBeVisible();
});
