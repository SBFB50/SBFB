// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Accessibility + resilience (hermetic): html lang=fr, the static title, the
 * keyboard-reachable rail nav links, and Escape-closes + focus-trap on the
 * three dialog families (command palette, GPU consent auto-open,
 * add-coordinator). Framework-provided behaviors asserted on observable DOM.
 */

import { test, expect } from "./fixtures";

const PLACEHOLDER = "Rechercher une action ou un projet…";

test.describe("A11y — document + nav", () => {
  test("html lang is fr, title is 'web', and rail nav links are present", async ({
    page,
  }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await expect(page.locator("html")).toHaveAttribute("lang", "fr");
    await expect(page).toHaveTitle("web");
    await expect(page.getByRole("link", { name: "Explorer" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Reseau" })).toBeVisible();
  });
});

test.describe("A11y — dialogs", () => {
  test("Escape closes the command palette (opened via Ctrl+K)", async ({
    page,
  }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await page.keyboard.press("Control+KeyK");
    await expect(page.getByPlaceholder(PLACEHOLDER)).toBeVisible();
    await expect(page.getByText("Navigation")).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(page.getByText("Navigation")).toHaveCount(0);
    await expect(page.getByPlaceholder(PLACEHOLDER)).toHaveCount(0);
  });

  test("the auto-opened GPU consent dialog exposes role=dialog and Escape dismisses it", async ({
    page,
  }) => {
    await page.goto("/my-network");
    await expect(page.getByTestId("gpu-consent-dialog")).toBeVisible();
    await expect(page.getByRole("dialog")).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(page.getByTestId("gpu-consent-dialog")).toHaveCount(0);
  });

  test("the add-coordinator dialog autofocuses its URL input and Escape closes it", async ({
    page,
  }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await page.getByTestId("command-palette-trigger").click();
    await page.getByRole("option", { name: "Se connecter a un noeud" }).click();
    await expect(
      page.getByRole("heading", { name: "Se connecter a un noeud" }),
    ).toBeVisible();

    await expect(page.locator("#coord-url")).toBeFocused();

    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("heading", { name: "Se connecter a un noeud" }),
    ).toHaveCount(0);
  });
});
