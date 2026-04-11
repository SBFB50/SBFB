/**
 * Sprint 8 Phase B — gov Politicien detail tab e2e.
 *
 * The Politicien tab picks the first politician (lexicographic
 * ORDER BY name LIMIT 1) and renders a fiche + recent positions.
 * In CI the legacy DB is absent so the handler's empty-state
 * branch renders a heading plus an empty block noting that the
 * per-tab selector lands in Sprint 9 polish.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov Politicien detail tab renders empty state without legacy DB", async ({
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

  // The `Politicien` tab name is a prefix of the sibling
  // `Politiciens` tab, so `hasText` is ambiguous. Filter the
  // `<li>` by a child element whose text is exactly
  // "Politicien" — the tab name header rendered by `TabRow` —
  // which disambiguates against the plural row.
  const politicienRow = page
    .locator("li")
    .filter({ has: page.getByText("Politicien", { exact: true }) })
    .first();
  await politicienRow
    .getByRole("button", { name: /Invoquer|Recharger/ })
    .click();

  // Heading block — fallback title when no politician is present.
  await expect(
    page.getByRole("heading", { name: "Fiche politicien" }),
  ).toBeVisible({ timeout: 10_000 });

  // Empty-state explanation.
  await expect(
    page.getByText(/Aucun politicien dans la base/),
  ).toBeVisible();

  await expect(page.getByText(/Descripteur legacy/)).toHaveCount(0);
});
