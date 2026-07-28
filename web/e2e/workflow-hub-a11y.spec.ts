// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase B — accessibility FOUNDATION harness (hermetic).
 *
 * DoD is AM-8: this frames the future Hub T1 spec and proves the a11y harness
 * is EXECUTABLE and GREEN on an existing shell surface — NOT full WCAG 2.2 AA
 * conformity (that belongs to the authoring surfaces + the AI1 panel). So:
 *   - media emulation (reduced-motion / forced-colors / zoom-reflow / RTL) must render;
 *   - keyboard-only reach (real Tab traversal) of the primary control;
 *   - a shell smoke that no `data-needs` leaks in the shell DOM;
 *   - a foundation axe-core rule subset asserts 0 violations (binding), while
 *     the cumulative WCAG 2.2 AA scan RUNS and is reported for triage.
 *
 * The AUTHORITATIVE proof of the needs-free invariant is the web unit test
 * `src/a11y/__tests__/profile_resolution.test.ts` (it resolves ALL 13 needs,
 * calls the real `applyResolvedAxes`, and asserts no `data-needs` / no need as a
 * value). The `/browse` check below is only a shell smoke — /browse does not
 * consume the profile, so it cannot, on its own, prove the Hub path.
 */
import AxeBuilder from "@axe-core/playwright";

import { test, expect } from "./fixtures";

// Foundation rules the shell already satisfies — the binding, green subset.
const FOUNDATION_RULES = [
  "html-has-lang",
  "html-lang-valid",
  "document-title",
  "meta-viewport",
];

// A real cumulative WCAG 2.2 AA scan = 2.0 + 2.1 + 2.2 A/AA tags. The lone
// `wcag22aa` tag only selects the NEW 2.2 rules (target-size), not the full set.
const WCAG22AA_TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22a", "wcag22aa"];

// Tab from the page start until the primary control gains focus, bounded.
async function tabToPrimaryControl(page: import("@playwright/test").Page, testId: string) {
  for (let i = 0; i < 25; i++) {
    const focused = await page.evaluate(
      (id) => document.activeElement?.getAttribute("data-testid") === id,
      testId,
    );
    if (focused) return true;
    await page.keyboard.press("Tab");
  }
  return await page.evaluate(
    (id) => document.activeElement?.getAttribute("data-testid") === id,
    testId,
  );
}

test.describe("A11y foundation — media emulation renders", () => {
  test("reduced-motion: /browse still renders its primary control", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();
  });

  test("forced-colors: /browse still renders its primary control", async ({ page }) => {
    await page.emulateMedia({ forcedColors: "active" });
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();
  });

  test("zoom-reflow: primary control is usable + overflow is measured at 320px", async ({ page }) => {
    // WCAG 2.2 SC 1.4.10 Reflow condition: content usable at 320 CSS px wide
    // (~400% zoom on a 1280px viewport). Foundation check (AM-8, NOT full
    // 1.4.10 conformity — the shell predates this work and is not reflow-tuned):
    // BINDING = the primary control is present + focusable at 320px; the
    // horizontal overflow is MEASURED and reported for triage (full reflow
    // conformity is an authoring-surface + AI1 concern, non-blocking here).
    await page.setViewportSize({ width: 320, height: 512 });
    await page.goto("/browse");
    const search = page.getByTestId("browse-search-input");
    await expect(search).toBeVisible();
    await search.focus();
    await expect(search).toBeFocused();

    const metrics = await page.evaluate(() => {
      const el = document.querySelector('[data-testid="browse-search-input"]');
      const rect = el?.getBoundingClientRect();
      return {
        docScrollWidth: document.documentElement.scrollWidth,
        innerWidth: window.innerWidth,
        controlRight: rect ? Math.round(rect.right) : null,
      };
    });
    const overflow = metrics.docScrollWidth - metrics.innerWidth;
    console.log(
      `[a11y-foundation] 320px reflow: docScrollWidth=${metrics.docScrollWidth} ` +
        `innerWidth=${metrics.innerWidth} horizontalOverflow=${overflow}px ` +
        `controlRight=${metrics.controlRight} (reported for triage — full reflow is a surface concern)`,
    );
  });

  test("RTL: an rtl attempt does not break the shell render", async ({ page }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();
    // The shell is LTR-first and manages its own <html> attributes; forcing
    // dir=rtl must not break rendering. Real RTL layout (logical properties +
    // the Hub dir toggle) is a Hub/authoring-surface concern, out of this
    // shell foundation scan (AM-8).
    await page.evaluate(() => document.documentElement.setAttribute("dir", "rtl"));
    await expect(page.getByTestId("browse-search-input")).toBeVisible();
  });
});

test.describe("A11y foundation — keyboard + needs-free smoke", () => {
  test("the primary control is reachable by real keyboard Tab traversal", async ({ page }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();
    const reached = await tabToPrimaryControl(page, "browse-search-input");
    expect(reached).toBe(true);
    await expect(page.getByTestId("browse-search-input")).toBeFocused();
  });

  test("shell smoke: no `data-needs` leaks in the shell DOM", async ({ page }) => {
    // A weak smoke — /browse does not consume the profile. The AUTHORITATIVE
    // needs-free proof is the web unit test on the real applyResolvedAxes path.
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    await expect(page.locator("html")).not.toHaveAttribute("data-needs", /.*/);
    const leaked = await page.locator("[data-needs]").count();
    expect(leaked).toBe(0);
    const html = await page.content();
    expect(html).not.toContain("data-needs");
  });
});

test.describe("A11y foundation — axe-core harness is executable", () => {
  test("foundation rule subset has 0 violations on /browse (binding)", async ({ page }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    const results = await new AxeBuilder({ page }).withRules(FOUNDATION_RULES).analyze();
    expect(results.violations).toEqual([]);
  });

  test("cumulative WCAG 2.2 AA scan RUNS and is reported (framing, non-blocking)", async ({ page }) => {
    await page.goto("/browse");
    await expect(page.getByTestId("browse-search-input")).toBeVisible();

    const results = await new AxeBuilder({ page }).withTags(WCAG22AA_TAGS).analyze();
    // The harness must EXECUTE (produce a result set) — conformity to zero is
    // the surfaces' + AI1's job, so this is reported, not asserted to empty.
    expect(Array.isArray(results.violations)).toBe(true);
    console.log(
      `[a11y-foundation] cumulative WCAG 2.2 AA violations on /browse: ${results.violations.length} ` +
        `(reported for triage — conformity is deferred to the authoring surfaces + AI1)`,
    );
  });
});

// Sprint 83 Phase L1 (T1 / G-HITL) — the HITL approval affordance is rendered
// by the trusted HOST CHROME (the shell), accessible, with no approval
// dark-pattern, and its queue is host-owned (an iframe cannot render it). This
// is the minimal L1 surface; the full Run Center is Phase V.
// The approval fixture is INJECTED by the test (the production queue is empty —
// a demo approval served in a real build would be fabricated authority data,
// ex-review P1-7). Same discipline as the auth-token seeding in `fixtures.ts`.
// TWO entries on purpose (review r2 P2-20): with a single-entry queue, removing
// the decided approval is indistinguishable from flushing the whole queue, and a
// parallel run normally has several requests pending at once.
const HITL_FIXTURE = [
  {
    requestId: "hitl-request-e2e-0001",
    request: {
      runId: "run-2f9c",
      planDigest: "plan-a1b2c3",
      nodeInstanceId: "root/cloud-model#1",
      effectId: "eff-7d21",
      capabilityRef: "cloud_provider_token",
      operation: "model.cloud.invoke",
      destination: "https://api.cloud-provider.example/v1/messages",
      outgoingDigest: "out-9e44f0",
      effectClass: "ModelCloud",
      maxCost: "5000",
      principal: "operator@node.local",
      nonce: "wait-nonce-5f3a",
      seq: "1",
      expiresAtMs: "4102444800000",
    },
  },
  {
    requestId: "hitl-request-e2e-0002",
    request: {
      runId: "run-8b41",
      planDigest: "plan-a1b2c3",
      nodeInstanceId: "root/notify#1",
      effectId: "eff-0c55",
      capabilityRef: "webhook_token",
      operation: "net.http.post",
      destination: "https://hooks.other-provider.example/notify",
      outgoingDigest: "out-31aa02",
      effectClass: "NetworkFixedDestination",
      maxCost: "10",
      principal: "operator@node.local",
      nonce: "wait-nonce-77c1",
      seq: "2",
      expiresAtMs: "4102444800000",
    },
  },
];

async function seedApprovalQueue(page: import("@playwright/test").Page) {
  await page.addInitScript((queue) => {
    (window as unknown as Record<string, unknown>).__SBFB_HITL_QUEUE = queue;
  }, HITL_FIXTURE);
}

test.describe("HITL approval — host chrome, accessible, no dark-pattern", () => {
  test("production queue is EMPTY (no fabricated authority data)", async ({ page }) => {
    // No seeding here on purpose: this is what a real build shows.
    await page.goto("/approvals");
    await expect(page.getByTestId("hitl-host-chrome")).toBeVisible();
    await expect(page.getByTestId("hitl-empty")).toBeVisible();
    expect(await page.getByTestId("hitl-approval-card").count()).toBe(0);
  });

  test("the exact binding tuple is rendered by the host chrome (WYSIWYS)", async ({ page }) => {
    await seedApprovalQueue(page);
    await page.goto("/approvals");
    // Rendered by the trusted host chrome, at the TOP-LEVEL document: no iframe
    // AT ALL on this route (counting every `iframe`, not just the shell's
    // remote-app test id — ex-review N5).
    await expect(page.getByTestId("hitl-host-chrome")).toBeVisible();
    expect(await page.locator("iframe").count()).toBe(0);

    const card = page.getByTestId("hitl-approval-card").first();
    await expect(card).toBeVisible();
    // WYSIWYS: destination, outgoing-data digest and bound capability are shown
    // in full, so the human approves what is actually sent.
    await expect(card).toContainText("api.cloud-provider.example");
    await expect(card).toContainText("out-9e44f0");
    await expect(card).toContainText("ModelCloud");
    await expect(card).toContainText("cloud_provider_token");
    // A PENDING request carries NO verdict (review r2 R7).
    await expect(card).not.toContainText("Verdict");
  });

  // NOTE (ex-review r2 R5) — the structural reason an iframe cannot approve is
  // that the bridge allowlist is CLOSED and contains no hitl/approval verb.
  // That is NOT asserted here: an earlier version tried, found the allowlist is
  // not published to the page, and fell back to a `return` — a test that could
  // never fail. The assertion now lives where the symbols exist, and it names
  // the property: `web/src/bridge/__tests__/authority.test.ts`, "no verb grants
  // HITL/approval/signing authority to an iframe" (`npm run test:unit`).
  // NOT `scripts/check-frontier-contracts.sh`: its bridge leg compares the TS
  // and Rust lists for PARITY, so adding an approval verb to BOTH mirrors in
  // lockstep leaves it green. Deleting the vacuous test rather than keeping a
  // green no-op is the point.

  test("focus is managed on mount and the approve control is Tab-reachable (no keyboard trap)", async ({
    page,
  }) => {
    await seedApprovalQueue(page);
    await page.goto("/approvals");
    await expect(page.getByTestId("hitl-heading")).toBeVisible();
    // Managed focus: the approval-queue heading takes focus on mount so a
    // screen reader announces the approval context.
    await expect(page.getByTestId("hitl-heading")).toBeFocused();

    // Real keyboard traversal reaches the Approve button — no keyboard trap.
    const reached = await tabToPrimaryControl(page, "hitl-approve");
    expect(reached).toBe(true);
    await expect(page.getByTestId("hitl-approve").first()).toBeFocused();

    // And the focus can LEAVE the card (escapability): two more Tabs move it
    // off the first card's two decision buttons.
    await page.keyboard.press("Tab");
    await page.keyboard.press("Tab");
    await expect(page.getByTestId("hitl-approve").first()).not.toBeFocused();
    await expect(page.getByTestId("hitl-reject").first()).not.toBeFocused();
  });

  test("no type-to-confirm and no forced delay: one activation decides", async ({ page }) => {
    await seedApprovalQueue(page);
    await page.goto("/approvals");
    const card = page.getByTestId("hitl-approval-card").first();
    await expect(card).toBeVisible();

    // No text input anywhere in the approval card — approving is a single
    // activation, never a "type APPROVE to confirm" dark-pattern.
    expect(await card.locator("input, textarea").count()).toBe(0);

    // Enabled at the FIRST observation. `isEnabled()` does not retry, unlike
    // `expect(...).toBeEnabled()` which would wait out a forced delay and call
    // it a pass (ex-review N3).
    const approve = page.getByTestId("hitl-approve").first();
    expect(await approve.isEnabled()).toBe(true);
    expect(await page.getByTestId("hitl-reject").first().isEnabled()).toBe(true);

    // Keyboard activation decides and the card leaves the host-owned queue.
    await approve.focus();
    await page.keyboard.press("Enter");
    // The announcement states what ACTUALLY happened — no signature exists yet.
    await expect(page.getByTestId("hitl-decision-status")).toContainText(
      "pris dans cet écran",
    );
    // ONLY the decided card leaves: the second pending approval is still there
    // (review r2 P2-20 — a decision is not a queue flush).
    await expect(page.getByTestId("hitl-approval-card")).toHaveCount(1);
    await expect(page.getByTestId("hitl-approval-card")).toHaveAttribute(
      "data-request-id",
      HITL_FIXTURE[1].requestId,
    );
    // Focus is not lost to the document root, and it lands on the element that
    // CARRIES the announcement — moving focus in the same tick as a `polite`
    // update makes screen readers drop the pending message (review r2 a11y).
    await expect(page.getByTestId("hitl-decision-status")).toBeFocused();
  });

  test("foundation rule subset has 0 violations on /approvals (binding)", async ({ page }) => {
    await seedApprovalQueue(page);
    await page.goto("/approvals");
    await expect(page.getByTestId("hitl-host-chrome")).toBeVisible();

    // The FOUNDATION subset alone is document-level (lang, title, viewport): it
    // is satisfied by `index.html` and is insensitive to this page's content —
    // replacing the whole body with an empty div would keep it green (review r2
    // P2-19). A NEW surface picks rules that actually exercise what it renders:
    // the definition list carrying the signed tuple, the decision buttons'
    // accessible names, contrast, and heading structure.
    //
    // SCOPED to the HITL region on purpose, and the reason is measured, not
    // assumed: run page-wide, `color-contrast` reports 123 violations that all
    // belong to the SHARED CHROME (the command-palette trigger at
    // `text-white/40` is 3.82:1, the header, the rail) — pre-existing debt of
    // every route, which the cumulative WCAG scan and AI1 own. Attributing them
    // to this phase would make the gate red for something L1 does not render,
    // and the reflex fix would be to drop `color-contrast` — trading a real
    // control for a green tick. `include` keeps the strong rule set AND makes
    // the failure attributable.
    //
    // Two honest limits of this scoping, stated so the gate is not read as
    // proving more than it does (ex-review r3 L1-D5-10):
    //  - the FOUNDATION rules are INEVALUABLE under `include` — their axe
    //    selectors are `html:not(html *)` and `meta[name=viewport]`, both
    //    outside the included subtree, so they match 0 nodes here. They stay in
    //    the list for symmetry with the other specs; the run that actually
    //    exercises them is the unscoped one on `/browse`.
    //  - the content rules need CONTENT. Without the assertion below, an empty
    //    queue renders the container and nothing else, `button-name`/`dlitem`/
    //    `listitem` lose every target, and this test would go green having seen
    //    no approval card at all. WITH it, this test is the FOURTH of the five
    //    HITL controls that turn red when the seed is closed off — bound 20 of
    //    §19.4 counts it among the red ones for exactly that reason.
    await expect(page.getByTestId("hitl-approval-card")).toHaveCount(2);

    const results = await new AxeBuilder({ page })
      .include('[data-testid="hitl-host-chrome"]')
      .withRules([
        ...FOUNDATION_RULES,
        "color-contrast",
        "button-name",
        "definition-list",
        "dlitem",
        "listitem",
        "aria-allowed-role",
        "aria-valid-attr-value",
        "heading-order",
      ])
      .analyze();
    expect(results.violations).toEqual([]);
  });
});
