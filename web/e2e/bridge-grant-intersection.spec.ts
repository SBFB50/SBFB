// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase C — bridge authority (intersection) FOUNDATION harness (hermetic).
 *
 * Frames the future Hub bridge-grant T1. The AUTHORITATIVE proof of the per-call
 * authority intersection (`declared ∩ grant ∩ policy ∩ consent`, fail-closed — a
 * VALID iframe message for a non-authorized method is refused, the offline/local
 * pii_redact effect is governed, expiry/revocation are honoured per call) is the
 * unit + integration harness, not this e2e:
 *   - web  : src/bridge/__tests__/intersection_authoritative.test.ts (useBridge
 *            dispatch consults the intersection at the choke-point) +
 *            src/bridge/__tests__/authority.test.ts (the model)
 *   - rust : sbfb_manifest::grant_v0::tests::{usurpation_refused, expiry_refused,
 *            revocation_per_call} + tests::bridge_methods_intersect_grants
 *
 * The hermetic harness deploys no browsable app, so — exactly like the a11y
 * foundation spec (`workflow-hub-a11y.spec.ts`, which tests `/browse` and points to
 * its unit test as authoritative) — this e2e is a SHELL smoke that guards the render
 * path Phase C touches: `BrowsedProject` resolves the app's release-declared
 * authority (grandfather UNTIL resolved / on any failure) and feeds it to the bridge.
 * It cannot, on its own, drive a real iframe→host refusal. The full Hub bridge-grant T1
 * (a real Hub app calling a declared + an undeclared method) lands with the
 * authoring/run surfaces (R..V) + the AI2 acceptance.
 */
import { test, expect } from "./fixtures";

test.describe("Bridge authority foundation — shell render path (hermetic smoke)", () => {
  test("the shell boots and /browse renders its primary control", async ({ page }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();
  });

  test("BrowsedProject renders without crashing on an unknown project id", async ({ page }) => {
    // Exercises the /browse/:id viewer route Phase C touched (the new
    // `@/bridge/authority` import + the authorityQuery, dormant without a real
    // archive). Either terminal render — "project not found" (coordinator active) or
    // "no active node" — proves the viewer mounts without a white-screen crash.
    await page.goto(`/browse/${"ab".repeat(32)}`);
    await expect(
      page
        .getByTestId("project-not-found")
        .or(page.getByText("Aucun noeud actif")),
    ).toBeVisible();
  });
});
