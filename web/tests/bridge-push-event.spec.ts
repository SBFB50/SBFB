/**
 * Sprint 15 Phase D — end-to-end host → iframe push event test.
 *
 * Validates that a `sbfb-bridge-event` message posted from the host
 * toward a sandboxed iframe triggers the registered
 * `bridge.onEvent` callback. The iframe echoes back via a custom
 * message type so the Playwright page can observe the result
 * without needing `iframe.contentDocument` access (blocked by
 * sandbox without allow-same-origin).
 */

import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

async function setupBridgeFixtures(
  page: import("@playwright/test").Page,
  iframeBodyScript: string,
) {
  const bridgeJsRaw = await readFile(
    resolve(process.cwd(), "public/sbfb-bridge.js"),
    "utf-8",
  );
  // Escape </script> sequences inside bridge.js comments so the
  // inlined <script> tag isn't closed prematurely by the HTML parser.
  const bridgeJs = bridgeJsRaw.replace(/<\/script>/gi, "<\\/script>");

  const iframeHtml =
    "<!DOCTYPE html><html><body>" +
    "<script>" +
    bridgeJs +
    "</script>" +
    "<script>" +
    iframeBodyScript +
    "</script>" +
    "</body></html>";

  const parentHtml =
    "<!DOCTYPE html><html><body>" +
    '<iframe id="child" src="/bridge-test/iframe" sandbox="allow-scripts" style="width:600px;height:400px"></iframe>' +
    "<script>" +
    "window.__echoes = [];" +
    'window.addEventListener("message", function (e) {' +
    '  if (e.data && e.data.type === "iframe-echo") {' +
    "    window.__echoes.push(e.data.payload);" +
    "  }" +
    "});" +
    "window.__pushToIframe = function (name, payload) {" +
    '  var frame = document.getElementById("child");' +
    "  frame.contentWindow.postMessage(" +
    '    { type: "sbfb-bridge-event", name: name, payload: payload },' +
    '    "*"' +
    "  );" +
    "};" +
    "</script>" +
    "</body></html>";

  await page.route("**/bridge-test/iframe", async (route) => {
    await route.fulfill({ contentType: "text/html", body: iframeHtml });
  });
  await page.route("**/bridge-test/parent", async (route) => {
    await route.fulfill({ contentType: "text/html", body: parentHtml });
  });
  await page.goto("http://127.0.0.1:5173/bridge-test/parent");
}

test.describe("bridge push event host → iframe (Sprint 15 Phase D)", () => {
  test("iframe onEvent callback fires for posted sbfb-bridge-event", async ({ page }) => {
    await setupBridgeFixtures(
      page,
      'var bridge = new SBFBBridge({ heartbeatInterval: 0 });' +
        'bridge.onEvent("task_result_ready", function (payload) {' +
        '  parent.postMessage({ type: "iframe-echo", payload: payload }, "*");' +
        "});",
    );

    // Wait for the iframe to finish loading and register its handler.
    await page.waitForTimeout(300);

    const expectedPayload = { task_id: "t-42", result: "ok", value: 3.14 };
    await page.evaluate((payload) => {
      (window as unknown as {
        __pushToIframe: (name: string, payload: unknown) => void;
      }).__pushToIframe("task_result_ready", payload);
    }, expectedPayload);

    await page.waitForFunction(
      () => (window as unknown as { __echoes: unknown[] }).__echoes.length >= 1,
      undefined,
      { timeout: 3000 },
    );

    const echoes = await page.evaluate(
      () => (window as unknown as { __echoes: unknown[] }).__echoes,
    );

    expect(echoes).toHaveLength(1);
    expect(echoes[0]).toEqual(expectedPayload);
  });

  test("iframe ignores events for non-subscribed names", async ({ page }) => {
    await setupBridgeFixtures(
      page,
      'var bridge = new SBFBBridge({ heartbeatInterval: 0 });' +
        'bridge.onEvent("subscribed_event", function (payload) {' +
        '  parent.postMessage({ type: "iframe-echo", payload: payload }, "*");' +
        "});",
    );

    await page.waitForTimeout(300);

    await page.evaluate(() => {
      (window as unknown as {
        __pushToIframe: (name: string, payload: unknown) => void;
      }).__pushToIframe("unrelated_event", { foo: 1 });
    });

    await page.waitForTimeout(800);

    const echoes = await page.evaluate(
      () => (window as unknown as { __echoes: unknown[] }).__echoes,
    );
    expect(echoes).toHaveLength(0);
  });
});
