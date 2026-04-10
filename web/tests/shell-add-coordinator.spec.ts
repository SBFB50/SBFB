/**
 * Sprint 5 Phase B — add-coordinator happy path.
 *
 * Empty localStorage → onboarding → add dialog → type URL →
 * Tester → Ajouter → my-projects shows the coordinator card.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_URL } from "./global-setup";

test("user adds a live coordinator from the onboarding CTA", async ({
  page,
}) => {
  await page.addInitScript(() => {
    window.localStorage.removeItem("nexus-grid:shell:v1");
  });

  await page.goto("/");

  // Open the dialog via the header "Add coordinator" button so we
  // exercise the same code path as an already-onboarded user.
  await page.getByRole("button", { name: /Ajouter un coordinateur/ }).first().click();

  // Clear default and type the live coord URL
  const urlInput = page.getByLabel("URL du coordinateur");
  await urlInput.fill(TEST_COORD_URL);

  await page.getByRole("button", { name: "Tester" }).click();

  await expect(
    page.getByText(/Coordinateur joignable/),
  ).toBeVisible({ timeout: 10_000 });

  await page.getByRole("button", { name: "Ajouter" }).click();

  // After closing the dialog the shell should show the project card.
  await expect(
    page.getByRole("heading", { name: "Mes projets" }),
  ).toBeVisible();
  await expect(page.getByText(TEST_COORD_URL).first()).toBeVisible();
});
