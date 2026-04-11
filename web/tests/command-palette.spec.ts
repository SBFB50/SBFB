/**
 * Sprint 6 Phase C — Ctrl+K command palette.
 *
 * Verifies that the global Ctrl+K keybind opens the dialog,
 * that typing + Enter navigates, and that Escape closes the
 * palette.
 */

import { test, expect } from "@playwright/test";
import { TEST_COORD_NAME, TEST_COORD_URL } from "./global-setup";

test("Ctrl+K opens the command palette and navigates", async ({ page }) => {
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

  // Open via the header trigger — primary UI path, always
  // reachable regardless of keyboard event routing quirks.
  const trigger = page.getByTestId("command-palette-trigger");
  await trigger.waitFor({ state: "visible" });
  await trigger.click();

  await expect(page.getByPlaceholder(/Rechercher une action/)).toBeVisible({
    timeout: 5_000,
  });

  // Typing "réseau" narrows to the Mon réseau entry, then Enter
  // navigates — React Router updates the URL.
  await page.getByPlaceholder(/Rechercher une action/).fill("réseau");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/my-network$/, { timeout: 5_000 });

  // Escape closes after reopening via trigger
  await page.getByTestId("command-palette-trigger").click();
  await expect(page.getByPlaceholder(/Rechercher une action/)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByPlaceholder(/Rechercher une action/)).toHaveCount(0, {
    timeout: 5_000,
  });

  // Ctrl+K opens via the global keyboard listener. Dispatch the
  // event directly on document so it bypasses any focus routing
  // quirks in headless Chromium. This still exercises the real
  // useCommandPalette hook end-to-end.
  //
  // Sprint 6 audit F-1: the handler now matches `e.code === "KeyK"`
  // (not `e.key`) so caps lock and non-QWERTY layouts still trip
  // the binding. The synthetic event must supply `code`.
  await page.evaluate(() => {
    document.dispatchEvent(
      new KeyboardEvent("keydown", { key: "k", code: "KeyK", ctrlKey: true }),
    );
  });
  await expect(page.getByPlaceholder(/Rechercher une action/)).toBeVisible({
    timeout: 5_000,
  });
});
