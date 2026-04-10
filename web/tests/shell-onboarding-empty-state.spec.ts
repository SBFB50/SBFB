/**
 * Sprint 5 Phase B — onboarding empty state.
 *
 * With an empty localStorage, the shell must land on the
 * "Bienvenue" page, show the CLI init/start commands, and
 * offer the "Ajouter un coordinateur" CTA. No live
 * coordinator interaction is required for this test.
 */

import { test, expect } from "@playwright/test";

test("shell renders onboarding when no coordinator is known", async ({
  page,
}) => {
  await page.addInitScript(() => {
    window.localStorage.removeItem("nexus-grid:shell:v1");
  });

  await page.goto("/");

  // The top-level heading is a real <h1>
  await expect(
    page.getByRole("heading", { name: "Bienvenue sur nexus-grid" }),
  ).toBeVisible();

  // Card titles are divs in shadcn — match by text directly.
  await expect(page.getByText("1. Démarre un coordinateur")).toBeVisible();
  await expect(page.getByText("2. Ajoute-le au shell")).toBeVisible();

  // Both CLI commands should be visible
  await expect(page.getByText("nexus-coordinator init demo")).toBeVisible();
  await expect(
    page.getByText("nexus-coordinator start demo").first(),
  ).toBeVisible();

  // The CTA button opens the add dialog
  await page
    .getByRole("button", { name: /Ajouter un coordinateur/ })
    .first()
    .click();
  await expect(
    page.getByText("Entre l'URL d'un nexus-coordinator"),
  ).toBeVisible();
});
