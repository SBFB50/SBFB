/**
 * Sprint 21 Phase B — end-to-end iframe PII redaction bridge test.
 *
 * Validates that the `piiRedact(text, policy)` method exposed by the
 * real `sbfb-bridge.js` SDK travels through postMessage correctly
 * and that a minimal host-side handler running the SDK semantics can
 * reply with a redacted payload. The full GLiNER ONNX path is not
 * exercised here — CI has no model asset — so the test uses a tiny
 * regex fallback (email-only) mirrored inline inside the parent
 * fixture to keep the spec hermetic.
 */

import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const PARENT_HANDLER_JS = `
window.__lastRequest = null;
window.__pii = null;

function redactText(text, policy) {
  var enabled = policy && policy.enabled === false ? false : true;
  var replacement =
    (policy && typeof policy.replacement === "string" && policy.replacement) ||
    "[REDACTED:{ENTITY}]";
  if (!enabled) return { redacted_text: text, findings_count: 0 };
  var findings = [];
  var re = /[\\w.+-]+@[\\w-]+\\.[\\w.-]{2,}/g;
  var m;
  while ((m = re.exec(text)) !== null) {
    findings.push({
      entity: "EMAIL_ADDRESS",
      start: m.index,
      end: m.index + m[0].length,
    });
  }
  findings.sort(function (a, b) {
    return a.start - b.start;
  });
  var out = "";
  var cursor = 0;
  for (var i = 0; i < findings.length; i++) {
    var f = findings[i];
    out += text.slice(cursor, f.start);
    out += replacement.replace("{ENTITY}", f.entity);
    cursor = f.end;
  }
  out += text.slice(cursor);
  return { redacted_text: out, findings_count: findings.length };
}

window.addEventListener("message", function (e) {
  var msg = e.data;
  if (!msg) return;
  // Iframe echoes results back here because sandboxed iframes can
  // not share window state with the parent directly.
  if (msg.type === "iframe-pii-result") {
    window.__pii = msg.payload;
    return;
  }
  if (msg.type !== "sbfb-bridge-request") return;
  window.__lastRequest = msg;
  var target = e.source;
  if (!target) return;
  if (msg.method !== "pii_redact") {
    target.postMessage(
      {
        type: "sbfb-bridge-response",
        id: msg.id,
        success: false,
        error: "unknown method",
      },
      "*",
    );
    return;
  }
  if (typeof msg.payload.text !== "string") {
    target.postMessage(
      {
        type: "sbfb-bridge-response",
        id: msg.id,
        success: false,
        error: "text must be a string",
      },
      "*",
    );
    return;
  }
  var result = redactText(msg.payload.text, msg.payload.policy);
  target.postMessage(
    {
      type: "sbfb-bridge-response",
      id: msg.id,
      success: true,
      data: result,
    },
    "*",
  );
});
`;

async function setupBridgeFixtures(
  page: import("@playwright/test").Page,
  iframeBodyScript: string,
) {
  const bridgeJsRaw = await readFile(
    resolve(process.cwd(), "public/sbfb-bridge.js"),
    "utf-8",
  );
  const bridgeJs = bridgeJsRaw.replace(/<\/script>/gi, "<\\/script>");

  const iframeHtml = `<!DOCTYPE html><html><body>
<script>${bridgeJs}</script>
<script>${iframeBodyScript}</script>
</body></html>`;

  const parentHtml = `<!DOCTYPE html><html><body>
<iframe id="child" src="/bridge-test/iframe" sandbox="allow-scripts" style="width:600px;height:400px"></iframe>
<script>${PARENT_HANDLER_JS}</script>
</body></html>`;

  await page.route("**/bridge-test/iframe", async (route) => {
    await route.fulfill({ contentType: "text/html", body: iframeHtml });
  });
  await page.route("**/bridge-test/parent", async (route) => {
    await route.fulfill({ contentType: "text/html", body: parentHtml });
  });
  await page.goto("http://127.0.0.1:5173/bridge-test/parent");
  // Let the iframe script run and instantiate SBFBBridge before
  // tests post their requests.
  await page.waitForTimeout(300);
}

test.describe("bridge pii_redact iframe → host (Sprint 21 Phase B)", () => {
  test("piiRedact method is callable from an iframe app", async ({ page }) => {
    await setupBridgeFixtures(
      page,
      `var bridge = new SBFBBridge({ heartbeatInterval: 0 });
bridge.piiRedact("clean text no pii").then(function (r) {
  parent.postMessage({ type: "iframe-pii-result", payload: r }, "*");
}).catch(function (err) {
  parent.postMessage({ type: "iframe-pii-result", payload: { error: String(err) } }, "*");
});`,
    );

    await page.waitForFunction(
      () => (window as unknown as { __pii: unknown }).__pii !== null,
      undefined,
      { timeout: 3000 },
    );

    const result = await page.evaluate(
      () =>
        (
          window as unknown as {
            __pii: {
              redacted_text?: string;
              findings_count?: number;
              error?: string;
            };
          }
        ).__pii,
    );
    expect(result.error).toBeUndefined();
    expect(result.redacted_text).toBe("clean text no pii");
    expect(result.findings_count).toBe(0);
  });

  test("piiRedact replaces emails via fallback regex", async ({ page }) => {
    await setupBridgeFixtures(
      page,
      `var bridge = new SBFBBridge({ heartbeatInterval: 0 });
bridge.piiRedact("Please contact me at alice@example.com today").then(function (r) {
  parent.postMessage({ type: "iframe-pii-result", payload: r }, "*");
});`,
    );

    await page.waitForFunction(
      () => (window as unknown as { __pii: unknown }).__pii !== null,
      undefined,
      { timeout: 3000 },
    );

    const result = await page.evaluate(
      () =>
        (
          window as unknown as {
            __pii: { redacted_text: string; findings_count: number };
          }
        ).__pii,
    );
    expect(result.redacted_text).toContain("[REDACTED:EMAIL_ADDRESS]");
    expect(result.redacted_text).not.toContain("alice@example.com");
    expect(result.findings_count).toBeGreaterThan(0);
  });

  test("piiRedact respects an enabled:false policy (pass-through)", async ({ page }) => {
    await setupBridgeFixtures(
      page,
      `var bridge = new SBFBBridge({ heartbeatInterval: 0 });
bridge.piiRedact("Email secret@corp.example", { enabled: false }).then(function (r) {
  parent.postMessage({ type: "iframe-pii-result", payload: r }, "*");
});`,
    );

    await page.waitForFunction(
      () => (window as unknown as { __pii: unknown }).__pii !== null,
      undefined,
      { timeout: 3000 },
    );

    const result = await page.evaluate(
      () =>
        (
          window as unknown as {
            __pii: { redacted_text: string; findings_count: number };
          }
        ).__pii,
    );
    expect(result.redacted_text).toBe("Email secret@corp.example");
    expect(result.findings_count).toBe(0);
  });

  test("piiRedact surfaces an error on non-string text payload", async ({ page }) => {
    await setupBridgeFixtures(
      page,
      `var bridge = new SBFBBridge({ heartbeatInterval: 0 });
bridge.piiRedact(42).then(function (r) {
  parent.postMessage({ type: "iframe-pii-result", payload: { ok: r } }, "*");
}).catch(function (err) {
  parent.postMessage({ type: "iframe-pii-result", payload: { error: String(err) } }, "*");
});`,
    );

    await page.waitForFunction(
      () => (window as unknown as { __pii: unknown }).__pii !== null,
      undefined,
      { timeout: 3000 },
    );

    const result = await page.evaluate(
      () =>
        (
          window as unknown as {
            __pii: { error?: string; ok?: unknown };
          }
        ).__pii,
    );
    expect(result.error).toBeTruthy();
    expect(result.ok).toBeUndefined();
  });

  test("piiRedact preserves the request correlation id", async ({ page }) => {
    await setupBridgeFixtures(
      page,
      `var bridge = new SBFBBridge({ heartbeatInterval: 0 });
bridge.piiRedact("hello world").then(function () {
  parent.postMessage({ type: "iframe-pii-result", payload: true }, "*");
});`,
    );

    await page.waitForFunction(
      () => (window as unknown as { __pii: unknown }).__pii === true,
      undefined,
      { timeout: 3000 },
    );

    const lastRequest = await page.evaluate(
      () =>
        (
          window as unknown as {
            __lastRequest: { id: string; method: string } | null;
          }
        ).__lastRequest,
    );
    expect(lastRequest).not.toBeNull();
    expect(lastRequest?.method).toBe("pii_redact");
    // The SDK mints a UUID v4; confirm a valid shape landed at the host.
    expect(lastRequest?.id).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
    );
  });
});
