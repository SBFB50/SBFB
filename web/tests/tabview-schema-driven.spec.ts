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
 * Sprint 8 Phase C rewrote the gov Contradictions tab from a
 * static Sprint 4 stub into a full TabView backed by
 * ``contradictions_overview_query``. In the Playwright
 * environment the legacy ``nexus/gov/govdata.db`` is absent,
 * so the handler falls back to the shared empty-state — still
 * a valid TabView that exercises the heading + empty blocks
 * through the full renderer pipeline (which is what this
 * regression test is here to guard).
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

  // Invoke the Contradictions tab descriptor. Sprint 8 Phase C
  // keeps the tab name stable while rewriting the handler body,
  // so the locator strategy inherited from Sprint 6 still works
  // — scope to the Contradictions row to avoid picking the
  // alphabetically-first "Biographie" button.
  const contradictionsRow = page
    .locator("li")
    .filter({ hasText: /^Contradictions/ })
    .first();
  await contradictionsRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();

  // Heading block — Phase C title.
  await expect(
    page.getByRole("heading", { name: "Détection de contradictions" }),
  ).toBeVisible({ timeout: 10_000 });

  // The empty-state placeholder is rendered by <EmptyBlock>
  // (legacy DB absent in CI → contradictions_overview_query
  // reports zero rows → fallback message).
  await expect(
    page.getByText(/Aucune contradiction détectée/),
  ).toBeVisible();

  // And crucially: no "legacy descriptor" warning, no raw JSON
  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
