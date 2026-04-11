/**
 * Sprint 8 Phase E — gov @nexus_command palette flow e2e.
 *
 * End-to-end regression for the new palette → deep-link flow:
 *
 *  1. Seed the Zustand store with the Playwright coordinator so
 *     the palette finds an active coordinator.
 *  2. Open the palette via the header trigger and verify the
 *     four gov commands appear under the "App : gov" heading.
 *  3. Type "Détecter" to narrow to the contradictions command,
 *     press Enter, and assert the URL lands on
 *     `/app/gov/tabs/Contradictions` and the tab heading
 *     renders.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("gov @nexus_command deep-links into the Contradictions tab", async ({
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

  // Open the palette via the header trigger (Ctrl+K is covered by
  // the dedicated spec — here we validate the new "App : gov"
  // group).
  await page.getByTestId("command-palette-trigger").click();
  await expect(page.getByPlaceholder(/Rechercher une action/)).toBeVisible({
    timeout: 5_000,
  });

  // The four gov commands land under "App : gov".
  await expect(page.getByText("App : gov")).toBeVisible({ timeout: 10_000 });
  await expect(
    page.getByText("Lancer un nouveau scan des politiciens"),
  ).toBeVisible();
  await expect(
    page.getByText("Détecter les contradictions politiques"),
  ).toBeVisible();
  await expect(
    page.getByText("Rechercher dans les fact-checks"),
  ).toBeVisible();
  await expect(
    page.getByText("Consulter les alertes récentes"),
  ).toBeVisible();

  // Type "Détecter" to narrow to the single matching entry, then
  // Enter. cmdk selects the first highlighted item on Enter.
  await page.getByPlaceholder(/Rechercher une action/).fill("Détecter");
  await page.keyboard.press("Enter");

  // React Router lands on the deep-link route.
  await expect(page).toHaveURL(/\/app\/gov\/tabs\/Contradictions$/, {
    timeout: 10_000,
  });

  // The AppTabPage renders the gov Contradictions TabView —
  // heading block matches the Python handler's title.
  await expect(
    page.getByText("Détection de contradictions"),
  ).toBeVisible({ timeout: 10_000 });
});
