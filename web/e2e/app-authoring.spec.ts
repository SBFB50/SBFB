// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * T1 (Sprint 79 Phase H) — hermetic RUNTIME self-check of the Factory
 * app-authoring CSP capability. BLOCKING and UNTAGGED, so it runs in the
 * default hermetic suite (`test:e2e` = `--grep-invert @compute`, wired in
 * GHA ci.yml step 10c + verify.sh step 15).
 *
 * It is the runtime complement of the static authoring gate (Phase E,
 * `run_gate_csp_authoring`): the static gate is a string scan, blind to
 * violations assembled at runtime (`fetch` via `atob`, dynamic `url()` /
 * `@font-face`). This spec replays two fixture apps inside the PRODUCTION
 * iframe host (`BrowsedProject`, `sandbox="allow-scripts"` WITHOUT
 * `allow-same-origin`, opaque origin) under the REAL CSP the daemon serves
 * via `blob_serve_csp_middleware` (`connect-src 'none'`, COEP require-corp),
 * and observes violations at BROWSER level (`page.on('console')`) across the
 * opaque iframe. No in-app shim, no cooperation from the served app — so it
 * works for ANY app, including the flagship `daisyui` template which carries
 * no `sbfb-bridge.js`.
 *
 *   - clean fixture -> ZERO CSP violation               (positive control)
 *   - dirty fixture -> >=1 CSP violation                (negative control)
 *
 * The negative control is load-bearing: a clean-only check proves the harness
 * runs, not that it detects (README §4). It also confirms the SERVED CSP
 * header == the single-source contract (`csp-contract.json`, machine mirror of
 * `nexus_core_rs::csp::BLOB_SERVE_CSP`) — the browser-side witness of
 * `blob_serve_csp_equals_contract`, complementing the Rust byte-exact test in
 * `blob_serve_http.rs::blob_serve_csp_header_byte_exact_matches_contract`.
 *
 * Hermetic-only: in external-daemon mode (`SBFB_E2E_BASE_URL`, the compute
 * flagship) the daemon has its own apps and token, so seeding is skipped.
 */

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import type { APIRequestContext } from "@playwright/test";

import { test, expect, computeFrame } from "./fixtures";
import { TEST_AUTH_TOKEN, TEST_COORD_URL } from "../tests/global-setup";

const __dirname = dirname(fileURLToPath(import.meta.url));
const FIXTURES = resolve(__dirname, "fixtures/app-authoring");

/** Single-source CSP contract (machine mirror of `nexus_core_rs::csp::BLOB_SERVE_CSP`). */
const CSP_CONTRACT = (
  JSON.parse(
    readFileSync(
      resolve(__dirname, "../../crates/nexus-core-rs/csp-contract.json"),
      "utf-8",
    ),
  ) as { csp: string }
).csp;

const HERMETIC = !process.env.SBFB_E2E_BASE_URL;

/** A CSP violation surfaces as a browser console error mentioning the policy. */
function isCspViolation(text: string): boolean {
  return /content security policy/i.test(text) || /connect-src/i.test(text);
}

/**
 * Seed one fixture archive into the hermetic daemon (publish-blob -> publish)
 * and return its `project_id` (= blake3(project_name)) plus the archive hash.
 * Loopback auth = `x-sbfb-token`; Host is loopback, Origin is absent — exactly
 * what the daemon's authenticated middleware accepts.
 */
async function seedFixture(
  request: APIRequestContext,
  variant: "clean" | "dirty",
): Promise<{ projectId: string; hash: string }> {
  const zip = readFileSync(resolve(FIXTURES, `${variant}.zip`));
  const blobRes = await request.post(
    `${TEST_COORD_URL}/api/daemon/publish-blob`,
    {
      headers: {
        "x-sbfb-token": TEST_AUTH_TOKEN,
        "content-type": "application/octet-stream",
      },
      data: zip,
    },
  );
  expect(blobRes.ok(), `publish-blob ${variant}`).toBeTruthy();
  const { hash } = (await blobRes.json()) as { hash: string };

  const projectName = `csp-${variant}-fixture`;
  const pubRes = await request.post(`${TEST_COORD_URL}/api/daemon/publish`, {
    headers: {
      "x-sbfb-token": TEST_AUTH_TOKEN,
      "content-type": "application/json",
    },
    data: {
      project_name: projectName,
      category: "test",
      description: `CSP ${variant} fixture`,
      apps: ["app"],
      archive_hash: hash,
    },
  });
  expect(pubRes.ok(), `publish ${variant}`).toBeTruthy();

  const browseRes = await request.get(`${TEST_COORD_URL}/api/daemon/browse`, {
    headers: { "x-sbfb-token": TEST_AUTH_TOKEN },
  });
  expect(browseRes.ok(), `browse ${variant}`).toBeTruthy();
  const browse = (await browseRes.json()) as {
    entries: { project_id: string; project_name: string }[];
  };
  const entry = browse.entries.find((e) => e.project_name === projectName);
  expect(entry, `seeded ${variant} entry present in /browse`).toBeTruthy();
  return { projectId: entry!.project_id, hash };
}

test.describe("app-authoring CSP runtime self-check (T1)", () => {
  test.skip(
    !HERMETIC,
    "hermetic-only: seeds fixtures into the spawned daemon",
  );

  test("served CSP header is byte-equal to the single-source contract", async ({
    request,
  }) => {
    const { hash } = await seedFixture(request, "clean");
    // blob-serve is a public route (the sandboxed iframe carries no token).
    const res = await request.get(
      `${TEST_COORD_URL}/blob-serve/${hash}/index.html`,
    );
    expect(res.ok()).toBeTruthy();
    expect(res.headers()["content-security-policy"]).toBe(CSP_CONTRACT);
  });

  test("CLEAN app replays under the real CSP with zero violation", async ({
    page,
    request,
  }) => {
    const violations: string[] = [];
    page.on("console", (msg) => {
      if (isCspViolation(msg.text())) violations.push(msg.text());
    });

    const { projectId } = await seedFixture(request, "clean");
    await page.goto(`/browse/${projectId}`);

    // The app ran inside the opaque iframe under the production CSP.
    await expect(computeFrame(page).locator("#title")).toHaveText(
      "clean-app-ready",
    );
    // Give any late violation a chance to surface, then assert none did.
    await page.waitForTimeout(500);
    expect(
      violations,
      `clean fixture must emit no CSP violation, got: ${violations.join(" | ")}`,
    ).toHaveLength(0);
  });

  test("DIRTY app's runtime-assembled fetch is caught by the CSP at runtime", async ({
    page,
    request,
  }) => {
    const violations: string[] = [];
    page.on("console", (msg) => {
      if (isCspViolation(msg.text())) violations.push(msg.text());
    });

    const { projectId } = await seedFixture(request, "dirty");
    await page.goto(`/browse/${projectId}`);

    await expect(computeFrame(page).locator("#title")).toHaveText(
      "dirty-app-ran",
    );
    // The fetch() to the atob-decoded host violates connect-src 'none' — the
    // browser refuses it and logs a CSP error the runtime self-check captures.
    await expect
      .poll(() => violations.length, {
        message: "dirty fixture must trigger >=1 CSP violation",
        timeout: 5_000,
      })
      .toBeGreaterThan(0);
  });
});
