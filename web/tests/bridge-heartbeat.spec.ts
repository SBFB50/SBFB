/**
 * Sprint 15 Phase D — end-to-end iframe heartbeat emission test.
 *
 * Validates that the real `sbfb-bridge.js` shipped under
 * `web/public/` emits heartbeats at the configured interval when
 * loaded inside a sandboxed iframe. The host-side watchdog logic
 * is covered by Vitest (`watchdog.test.ts`) — this spec focuses on
 * the real cross-origin postMessage channel that unit tests can't
 * exercise.
 */

import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

test.describe("iframe bridge heartbeat (Sprint 15 Phase D)", () => {
  test("iframe with SBFBBridge emits two heartbeats within 2s", async ({ page }) => {
    const bridgeJsRaw = await readFile(
      resolve(process.cwd(), "public/sbfb-bridge.js"),
      "utf-8",
    );
    // The HTML parser closes a <script> tag at the first </script>
    // it sees — even inside JS comments. bridge.js has a </script>
    // in its JSDoc @example, so we escape it before inlining.
    const bridgeJs = bridgeJsRaw.replace(/<\/script>/gi, "<\\/script>");

    // Minimal iframe doc: loads the real bridge SDK with a short
    // heartbeat interval so the test doesn't have to wait 1s between
    // pings.
    const iframeHtml =
      "<!DOCTYPE html><html><body>" +
      "<script>" +
      bridgeJs +
      "</script>" +
      "<script>new SBFBBridge({ heartbeatInterval: 200 });</script>" +
      "</body></html>";

    // Serve the iframe source via a Playwright route so the browser
    // fetches real HTML with a real origin — srcdoc inheritance of
    // opaque origin + HTML-entity escaping was too brittle.
    await page.route("**/bridge-test/iframe", async (route) => {
      await route.fulfill({ contentType: "text/html", body: iframeHtml });
    });

    const parentHtml =
      "<!DOCTYPE html><html><body>" +
      '<iframe id="child" src="/bridge-test/iframe" sandbox="allow-scripts" style="width:600px;height:400px"></iframe>' +
      "<script>" +
      "window.__heartbeats = [];" +
      'window.addEventListener("message", function (e) {' +
      '  if (e.data && e.data.type === "sbfb-bridge-heartbeat") {' +
      "    window.__heartbeats.push(e.data);" +
      "  }" +
      "});" +
      "</script>" +
      "</body></html>";

    await page.route("**/bridge-test/parent", async (route) => {
      await route.fulfill({ contentType: "text/html", body: parentHtml });
    });

    await page.goto("http://127.0.0.1:5173/bridge-test/parent");

    // Wait until we collect at least 2 heartbeats. The first should
    // fire immediately (ping() call in _startHeartbeat) and the
    // second after 200ms.
    await page.waitForFunction(
      () => (window as unknown as { __heartbeats: unknown[] }).__heartbeats.length >= 2,
      undefined,
      { timeout: 5000 },
    );

    const heartbeats = await page.evaluate(
      () => (window as unknown as { __heartbeats: Array<{ type: string; ts: number }> }).__heartbeats,
    );

    expect(heartbeats.length).toBeGreaterThanOrEqual(2);
    for (const hb of heartbeats) {
      expect(hb.type).toBe("sbfb-bridge-heartbeat");
      expect(typeof hb.ts).toBe("number");
      expect(hb.ts).toBeGreaterThan(0);
    }
    // Second beat should land within a reasonable time after the
    // first — allow slack for CI scheduling.
    const delta = heartbeats[1].ts - heartbeats[0].ts;
    expect(delta).toBeGreaterThan(50);
    expect(delta).toBeLessThan(1500);
  });
});
