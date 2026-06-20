// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Daemon error-UI coverage via interception of ONE named endpoint per test
 * (abort → offline banner; fulfill 500 → ErrorCard). This is error-branch
 * coverage of a single route, NOT a daemon mock: every other endpoint stays
 * live so the app still boots and auto-registers. Proves the DaemonResult
 * discriminant (unavailable vs error) across Browse / Curators / Nodes and
 * the per-view search / worker error branches. Routes are registered BEFORE
 * goto; globs use trailing ** to tolerate query strings.
 */

import { test, expect } from "./fixtures";

test.describe("Error states — interception", () => {
  test("aborting /api/daemon/browse renders the offline banner (not a crash)", async ({
    page,
  }) => {
    await page.route("**/api/daemon/browse**", (r) => r.abort("failed"));
    await page.goto("/browse");

    await expect(page.getByTestId("daemon-offline-banner")).toBeVisible();
    await expect(page.getByText("Noeud indisponible")).toBeVisible();
    await expect(
      page.getByText("nexus-shell-daemon", { exact: true }),
    ).toBeVisible();
    await expect(page.getByText("La page a crashé")).toHaveCount(0);
  });

  test("a 500 on /api/daemon/browse renders the ErrorCard, not the offline banner", async ({
    page,
  }) => {
    await page.route("**/api/daemon/browse**", (r) =>
      r.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "boom" }),
      }),
    );
    await page.goto("/browse");

    await expect(page.getByText(/Erreur r.seau/)).toBeVisible();
    await expect(page.getByText(/HTTP 500/)).toBeVisible();
    await expect(page.getByTestId("daemon-offline-banner")).toHaveCount(0);
  });

  test("aborting /api/daemon/curators shows the offline banner while the form stays up", async ({
    page,
  }) => {
    await page.route("**/api/daemon/curators**", (r) => r.abort("failed"));
    await page.goto("/curators");

    await expect(page.getByTestId("daemon-offline-banner")).toBeVisible();
    await expect(page.getByText("Noeud indisponible")).toBeVisible();
    await expect(page.getByTestId("curator-pubkey-input")).toBeVisible();
  });

  test("aborting /api/daemon/nodes shows the offline banner under the header, not the cold-start", async ({
    page,
  }) => {
    // Keep /api/daemon/curators live so only the nodes query is unavailable.
    await page.route("**/api/daemon/nodes**", (r) => r.abort("failed"));
    await page.goto("/nodes");

    await expect(page.getByTestId("daemon-offline-banner")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Noeuds" })).toBeVisible();
    await expect(page.getByTestId("nodes-cold-start")).toHaveCount(0);
  });

  test("a 500 on /api/daemon/search renders the search ErrorCard, not the no-result state", async ({
    page,
  }) => {
    await page.route("**/api/daemon/search**", (r) =>
      r.fulfill({
        status: 500,
        contentType: "application/json",
        body: '{"error":"index down"}',
      }),
    );
    await page.goto("/browse");

    await expect(page.getByTestId("browse-search-input")).toBeVisible();
    await page.getByTestId("browse-search-input").fill("anything");

    await expect(page.getByText(/Erreur r.seau/)).toBeVisible();
    await expect(page.getByTestId("browse-search-empty")).toHaveCount(0);
  });

  test("a 500 on /api/v1/worker/state renders the inline worker-error card without crashing", async ({
    page,
  }) => {
    await page.addInitScript(() =>
      window.localStorage.setItem("sbfb-consent-seen-v1", "1"),
    );
    await page.route("**/api/v1/worker/state", (r) =>
      r.fulfill({ status: 500, body: "boom" }),
    );
    await page.goto("/my-network");

    await expect(page.getByText(/Erreur lecture de l.*tat du worker/)).toBeVisible();
    await expect(page.getByTestId("offer-power-card")).toBeVisible();
    await expect(page.getByText("La page a crashé")).toHaveCount(0);
  });
});
