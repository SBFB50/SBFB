// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Curators — client-side pubkey validation (hermetic).
 *
 * Submitting an invalid curator key must surface the French validation
 * error WITHOUT a daemon round-trip (Curators.tsx:110-121 rejects
 * before mutating). Proves the real form wiring + FR copy in the live
 * shell.
 */

import { test, expect } from "./fixtures";

test.describe("Curators — invalid key validation", () => {
  test("rejects a malformed pubkey with the French error", async ({ page }) => {
    await page.goto("/curators");

    const input = page.getByTestId("curator-pubkey-input");
    await expect(input).toBeVisible();

    // Not 64 lowercase hex chars → isValidCuratorPubkey (daemon.ts:692) false.
    await input.fill("pas-une-cle-valide");
    await page.getByTestId("curator-subscribe-submit").click();

    await expect(page.getByTestId("curator-form-error")).toHaveText(
      "La clé publique doit faire 64 caractères hexadécimaux minuscules.",
    );
  });
});
