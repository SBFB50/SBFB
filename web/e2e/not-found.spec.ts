// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Not-found states (hermetic) — catch-up E2E coverage.
 *
 * Unknown ids must land on honest "introuvable" states rendered from the
 * real (empty) daemon's `/api/daemon/*` responses, never a crash or an
 * error boundary. Exercises the id → lookup → not-found path of the
 * detail routes.
 */

import { test, expect } from "./fixtures";

test.describe("not-found states for unknown ids", () => {
  test("/node/:id unknown → catalogue introuvable", async ({ page }) => {
    await page.goto("/node/does-not-exist-node");
    await expect(page.getByTestId("node-not-found")).toBeVisible();
    await expect(page.getByTestId("back-to-nodes")).toBeVisible();
  });

  test("/project/:name unknown → projet introuvable", async ({ page }) => {
    await page.goto("/project/does-not-exist-project");
    await expect(
      page.getByRole("heading", { name: "Projet introuvable" }),
    ).toBeVisible();
  });

  test("/browse/:id unknown → projet introuvable (fullscreen)", async ({
    page,
  }) => {
    await page.goto("/browse/does-not-exist-app");
    await expect(page.getByTestId("project-not-found")).toBeVisible();
    await expect(page.getByTestId("back-to-browse")).toBeVisible();
  });
});
