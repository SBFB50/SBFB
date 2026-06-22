// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Shard session panel E2E — Sprint 77 Phase J.
 *
 * Two layers, mirroring `compute-tester.spec.ts`:
 *
 *  1. HERMETIC (no tag) — runs in the default `test:e2e`
 *     (`--grep-invert @compute` does NOT exclude it) and in CI. It drives the
 *     real `/compute` panel against the hermetic daemon (which has no live
 *     shard session — the `sbfb/shard/1` data plane is not wired to a
 *     control-plane store yet, Phase K), asserting the FR intents (PO-9, zero
 *     jargon) and the honest empty state. BLOQUANT-vert au wrap-up.
 *
 *  2. CROSS-MACHINE (`@shard`) — env-gated like the compute flagship. It needs
 *     a real multi-machine private compute group, absent from hermetic CI, so
 *     it is SKIPPED unless `SBFB_E2E_SHARD=1`. It is gated by `test.skip`, NOT
 *     by `--grep-invert`: tagging the hermetic layer `@shard` or grep-inverting
 *     it would silently drop the panel coverage from CI.
 *       SBFB_E2E_SHARD=1
 *       SBFB_E2E_SHARD_SESSION=<a live group/session id on that daemon>
 *     Set the env in YOUR shell first, then `npm run test:e2e`.
 */

import { test, expect } from "./fixtures";

test.describe("shard session panel (hermetic)", () => {
  test("renders the FR intents and the empty state", async ({ page }) => {
    await page.goto("/compute");

    const panel = page.getByTestId("shard-session-panel");
    await expect(panel).toBeVisible();

    // Intentions utilisateur, byte-exact, zéro jargon shard/ALPN/ComputeGroup.
    await expect(page.getByTestId("cta-launch-large-model")).toHaveText(
      "Lancer un gros modèle en réseau",
    );
    await expect(page.getByTestId("cta-join-compute-group")).toHaveText(
      "Rejoindre un groupe de calcul",
    );

    // No live session on the hermetic daemon → the honest empty state.
    await expect(page.getByTestId("shard-session-empty")).toBeVisible();
    await expect(page.getByText("Aucune session active")).toBeVisible();
  });

  test("join reveals the group-id lookup form", async ({ page }) => {
    await page.goto("/compute");
    await page.getByTestId("cta-join-compute-group").click();
    await expect(page.getByTestId("shard-group-id-input")).toBeVisible();
    // The lookup is disabled until an id is entered (no blind empty query).
    await expect(page.getByTestId("shard-join-submit")).toBeDisabled();
  });
});

const SHARD_ENABLED = process.env.SBFB_E2E_SHARD === "1";
const SHARD_SESSION = process.env.SBFB_E2E_SHARD_SESSION ?? "";

test.describe("@shard cross-machine pipeline", () => {
  test.skip(
    !SHARD_ENABLED || SHARD_SESSION.length === 0,
    "gated: set SBFB_E2E_SHARD=1 + SBFB_E2E_SHARD_SESSION (a live multi-machine private compute group)",
  );

  test("shows the aggregate status of a live shard session", async ({
    page,
  }) => {
    await page.goto("/compute");
    await page.getByTestId("cta-join-compute-group").click();
    await page.getByTestId("shard-group-id-input").fill(SHARD_SESSION);
    await page.getByTestId("shard-join-submit").click();

    // A real group reports an aggregate member count — never a member identity.
    await expect(page.getByTestId("shard-session-status")).toBeVisible();
    await expect(page.getByTestId("shard-member-count")).not.toBeEmpty();
  });
});
