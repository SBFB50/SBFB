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
