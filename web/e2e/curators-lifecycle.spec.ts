// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Curators — full subscribe/unsubscribe lifecycle + normalisation (hermetic).
 *
 * Deepens curators-validation.spec.ts (single inline error) with the real
 * mutate → React-Query refetch → waiting-row → unsubscribe → empty cycle,
 * the refresh no-crash, and the client `.trim().toLowerCase()` normalisation
 * trap. All against the real empty daemon (Normal-mode, subscribe is real).
 */

import { test, expect } from "./fixtures";

test.describe("Curators — subscribe/unsubscribe lifecycle", () => {
  test("subscribes a synthetic 64-hex key, shows a waiting row, then unsubscribes back to empty", async ({
    page,
  }) => {
    await page.goto("/curators");
    const input = page.getByTestId("curator-pubkey-input");
    await expect(input).toBeVisible();

    await expect(page.getByText("Aucun curator suivi")).toBeVisible();
    await expect(page.getByTestId("curator-row")).toHaveCount(0);

    await input.fill("a".repeat(64));
    await page.getByTestId("curator-subscribe-submit").click();

    await expect(page.getByTestId("curator-row")).toHaveCount(1);
    await expect(input).toHaveValue("");
    await expect(page.getByTestId("curator-unsubscribe")).toBeVisible();
    // Waiting (not active): no gossip entry → no "projet(s) vouché(s)" badge.
    await expect(page.getByText(/En attente/)).toBeVisible();
    await expect(page.getByText(/vouch/)).toHaveCount(0);

    await page.getByTestId("curator-unsubscribe").click();
    await expect(page.getByTestId("curator-row")).toHaveCount(0);
    await expect(page.getByText("Aucun curator suivi")).toBeVisible();
    await expect(page.getByTestId("curator-list")).toHaveCount(0);
  });

  test("manual refresh on an empty daemon keeps the empty-state without crashing", async ({
    page,
  }) => {
    await page.goto("/curators");
    await expect(page.getByTestId("curator-pubkey-input")).toBeVisible();
    await expect(page.getByText("Aucun curator suivi")).toBeVisible();

    await page.getByTestId("curators-refresh").click();
    await expect(page.getByText("Aucun curator suivi")).toBeVisible();
    await expect(page.getByText("La page a crashé")).toHaveCount(0);
  });
});

test.describe("Curators — pubkey normalisation and validation variants", () => {
  test("rejects too-short, too-long and internal-non-hex keys with the French inline error and no daemon row", async ({
    page,
  }) => {
    await page.goto("/curators");
    const input = page.getByTestId("curator-pubkey-input");
    const submit = page.getByTestId("curator-subscribe-submit");
    const err = page.getByTestId("curator-form-error");

    await input.fill("abc123");
    await submit.click();
    await expect(err).toHaveText(
      "La clé publique doit faire 64 caractères hexadécimaux minuscules.",
    );

    await input.fill("a".repeat(65));
    await submit.click();
    await expect(err).toBeVisible();

    await input.fill("z".repeat(64));
    await submit.click();
    await expect(err).toBeVisible();

    await expect(page.getByTestId("curator-row")).toHaveCount(0);
    await expect(page.getByTestId("curator-list")).toHaveCount(0);
  });

  test("an uppercase + space-padded 64-hex key is normalised and accepted (subscribes, not an error)", async ({
    page,
  }) => {
    await page.goto("/curators");
    const input = page.getByTestId("curator-pubkey-input");

    await input.fill("  " + "A".repeat(64) + "  ");
    await page.getByTestId("curator-subscribe-submit").click();

    // trim+lowercase → the same 64 'a' key → real subscribe, not an error.
    await expect(page.getByTestId("curator-form-error")).toHaveCount(0);
    await expect(page.getByTestId("curator-row")).toHaveCount(1);
    await expect(input).toHaveValue("");

    // Clean up: the daemon is a shared singleton across the run, so a
    // residual subscription would break later /nodes cold-start assertions.
    await page.getByTestId("curator-unsubscribe").click();
    await expect(page.getByTestId("curator-row")).toHaveCount(0);
  });
});
