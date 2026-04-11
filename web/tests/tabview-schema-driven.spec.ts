/**
 * Sprint 6 Phase E — schema-driven tab renderer e2e.
 *
 * Exercises the full happy path end-to-end against the live
 * coordinator spawned by globalSetup:
 *   Python TabView constructor
 *   → coordinator validation (legacy_descriptor: false)
 *   → Zod parse on the client
 *   → TabViewRenderer
 *
 * Confirms that the gov Contradictions tab (ported to TabView
 * in Phase A) renders as heading + muted text + two metrics
 * + an empty block, with zero raw JSON visible on screen.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov Contradictions tab renders through the TabView renderer", async ({
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

  // Wait for /app list then expand the gov accordion
  await expect(page.getByText(/gov/).first()).toBeVisible({ timeout: 10_000 });
  await page.getByText("gov", { exact: true }).click();

  // Routes section must appear before we try to hit « Invoquer »
  await expect(page.getByText("Routes").first()).toBeVisible({
    timeout: 5_000,
  });

  // Invoke the Contradictions tab descriptor. Sprint 8 Phase B
  // rewrote the gov app with seven tabs — the coordinator sorts
  // them by name for the manifest endpoint, so the first button
  // in the list is Biographie, not Contradictions. Scope to the
  // Contradictions row before clicking.
  const contradictionsRow = page
    .locator("li")
    .filter({ hasText: /^Contradictions/ })
    .first();
  await contradictionsRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();

  // Heading block
  await expect(
    page.getByText("Analyse de cohérence politique"),
  ).toBeVisible({ timeout: 10_000 });

  // Both metric labels render
  await expect(page.getByText("Déclarations analysées")).toBeVisible();
  await expect(page.getByText("Contradictions détectées")).toBeVisible();

  // The empty-state placeholder is rendered by <EmptyBlock>
  await expect(
    page.getByText(/Aucune analyse en cours/),
  ).toBeVisible();

  // And crucially: no "legacy descriptor" warning, no raw JSON
  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
