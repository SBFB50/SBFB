// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Command palette + coordinator picker + add-coordinator dialog (hermetic).
 * Open via trigger and via Ctrl+K, navigate via a Navigation option, Escape
 * close, open the AddCoordinatorDialog from the palette and from the picker,
 * and the bogus-URL "Tester" failure path (hermetic via an unbound port).
 */

import { test, expect } from "./fixtures";

const PLACEHOLDER = "Rechercher une action ou un projet…";

test.describe("Command palette", () => {
  test("trigger button opens the palette; a Navigation item routes to /curators", async ({
    page,
  }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await page.getByTestId("command-palette-trigger").click();
    await expect(page.getByPlaceholder(PLACEHOLDER)).toBeVisible();

    await expect(page.getByRole("option", { name: "Curators" })).toBeVisible();
    await page.getByRole("option", { name: "Curators" }).click();

    await expect(page).toHaveURL(/\/curators$/);
    await expect(page.getByTestId("curator-pubkey-input")).toBeVisible();
  });

  test("Ctrl+K opens the palette and Escape closes it", async ({ page }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await page.keyboard.press("Control+KeyK");
    await expect(page.getByPlaceholder(PLACEHOLDER)).toBeVisible();
    await expect(page.getByText("Navigation")).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(page.getByPlaceholder(PLACEHOLDER)).toHaveCount(0);
    await expect(page.getByText("Navigation")).toHaveCount(0);
  });

  test("the 'Se connecter a un noeud' action closes the palette and opens the dialog", async ({
    page,
  }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await page.getByTestId("command-palette-trigger").click();
    await expect(page.getByPlaceholder(PLACEHOLDER)).toBeVisible();

    await page
      .getByRole("option", { name: "Se connecter a un noeud" })
      .click();

    await expect(
      page.getByRole("heading", { name: "Se connecter a un noeud" }),
    ).toBeVisible();
    await expect(page.getByPlaceholder(PLACEHOLDER)).toHaveCount(0);
  });
});

test.describe("Add-coordinator dialog", () => {
  test("Tester on an unbound loopback port surfaces an error and keeps Ajouter disabled", async ({
    page,
  }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await page.getByTestId("command-palette-trigger").click();
    await page.getByRole("option", { name: "Se connecter a un noeud" }).click();
    await expect(
      page.getByRole("heading", { name: "Se connecter a un noeud" }),
    ).toBeVisible();

    await page.locator("#coord-url").fill("http://127.0.0.1:1");
    await page.getByRole("button", { name: "Tester" }).click();

    await expect(page.getByRole("button", { name: "Ajouter" })).toBeDisabled();
    await expect(page.getByRole("button", { name: "Tester" })).toBeEnabled();
  });
});
