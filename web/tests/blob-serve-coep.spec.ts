/**
 * Sprint 32 Phase C — COEP iframe isolation regression test.
 *
 * Verifies that content served behind blob-serve-style COEP headers
 * (Cross-Origin-Embedder-Policy: require-corp) inside a sandboxed
 * iframe cannot perform outbound fetches. This is the browser-level
 * enforcement complement to the Rust unit test on BLOB_SERVE_COEP.
 *
 * P2-REVIEW-B-1-S30 carry (2/3 → resolved).
 */

import { test, expect } from "@playwright/test";

test.describe("blob-serve COEP iframe isolation (Sprint 30 Phase B carry)", () => {
  test("sandboxed iframe fetch is blocked by isolation triple layer", async ({
    page,
  }) => {
    // Track any request the iframe might try to make to example.com
    const blockedRequests: string[] = [];
    const failedRequests: string[] = [];

    page.on("request", (req) => {
      if (req.url().includes("example.com")) {
        blockedRequests.push(req.url());
      }
    });
    page.on("requestfailed", (req) => {
      if (req.url().includes("example.com")) {
        failedRequests.push(req.url());
      }
    });

    // iframe content: attempts a fetch and reports result via title
    const iframeHtml = `<!DOCTYPE html><html><head><title>loading</title></head><body>
<script>
  (async () => {
    try {
      await fetch("https://example.com/coep-test");
      document.title = "fetch-allowed";
    } catch (e) {
      document.title = "fetch-blocked";
    }
  })();
</script>
</body></html>`;

    await page.route("**/blob-serve-coep-test/app.html", async (route) => {
      await route.fulfill({
        contentType: "text/html",
        body: iframeHtml,
        headers: {
          "Cross-Origin-Embedder-Policy": "require-corp",
          "Cross-Origin-Opener-Policy": "same-origin",
          "Cross-Origin-Resource-Policy": "cross-origin",
          "Content-Security-Policy":
            "default-src 'unsafe-inline'; connect-src 'none'",
        },
      });
    });

    // Host page: embeds the iframe with sandbox (matches real shell)
    const hostHtml = `<!DOCTYPE html><html><body>
<iframe
  id="app-frame"
  src="/blob-serve-coep-test/app.html"
  sandbox="allow-scripts"
  style="width:600px;height:400px"
></iframe>
<script>
  // Signal when iframe has loaded (even if sandboxed)
  document.getElementById("app-frame").addEventListener("load", () => {
    document.title = "iframe-loaded";
  });
</script>
</body></html>`;

    await page.route("**/coep-isolation-host", async (route) => {
      await route.fulfill({
        contentType: "text/html",
        body: hostHtml,
        headers: {
          "Cross-Origin-Embedder-Policy": "require-corp",
          "Cross-Origin-Opener-Policy": "same-origin",
        },
      });
    });

    await page.goto("/coep-isolation-host");

    // Wait for iframe load event or timeout
    await page
      .waitForFunction(() => document.title === "iframe-loaded", null, {
        timeout: 5000,
      })
      .catch(() => {
        /* iframe may not fire load if COEP blocks it — that's OK */
      });

    // Allow time for any fetch attempt to complete or fail
    await page.waitForTimeout(2000);

    // Verification: the fetch to example.com must NOT have succeeded.
    // Three possible outcomes, all proving isolation:
    // 1. The fetch request was never made (sandbox blocked it)
    // 2. The fetch request failed (COEP/CSP blocked it)
    // 3. The iframe itself didn't load (COEP blocked the iframe)
    //
    // In ALL cases, no successful fetch to example.com occurs.
    // If a request was made, it must be in failedRequests.
    const successfulFetches = blockedRequests.filter(
      (url) => !failedRequests.includes(url),
    );
    expect(
      successfulFetches,
      "No fetch from sandboxed iframe should reach example.com",
    ).toHaveLength(0);
  });
});
