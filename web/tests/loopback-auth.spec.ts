/**
 * Sprint 16 Phase A (D1) — end-to-end checks of the coordinator's
 * LoopbackAuthMiddleware. Drives the real coordinator spawned by
 * `tests/global-setup.ts` via `page.request` (Playwright's
 * low-level HTTP client) so each request's headers are under
 * the spec's control.
 *
 * The suite does not go through the Vite-served shell — we probe
 * the coordinator directly at TEST_COORD_URL. Behaviour tested:
 *
 * - `/health` is public (no token required) — the launcher probe
 *   must keep working.
 * - Bearer token missing → 401.
 * - Bearer token present + wrong → 401.
 * - Valid triple (token + loopback Host + no Origin) → 200.
 * - Valid token + rebound Host (`Host: attacker.com`) → 403.
 * - Valid token + cross-origin (`Origin: https://attacker.com`) → 403.
 */

import { test, expect, request as apiRequest } from "@playwright/test";

import { TEST_AUTH_TOKEN, TEST_COORD_URL } from "./global-setup";

// A freshly built request context (no Playwright defaults) so
// every header is explicit. The config-level `extraHTTPHeaders`
// lives on `page.context()`; this scope is independent.
async function bareContext() {
  return await apiRequest.newContext({
    baseURL: TEST_COORD_URL,
    extraHTTPHeaders: {},
  });
}

test("health is public — no bearer needed", async () => {
  const ctx = await bareContext();
  const res = await ctx.get("/health");
  expect(res.status()).toBe(200);
  const body = (await res.json()) as { status: string };
  expect(body.status).toBe("ok");
  await ctx.dispose();
});

test("authenticated routes reject requests without the bearer token", async () => {
  const ctx = await bareContext();
  const res = await ctx.get("/project");
  expect(res.status()).toBe(401);
  await ctx.dispose();
});

test("authenticated routes reject a wrong bearer token", async () => {
  const ctx = await bareContext();
  const res = await ctx.get("/project", {
    headers: {
      "x-sbfb-token": "0".repeat(64),
    },
  });
  expect(res.status()).toBe(401);
  await ctx.dispose();
});

test("authenticated routes accept the valid triple", async () => {
  const ctx = await bareContext();
  const res = await ctx.get("/project", {
    headers: {
      "x-sbfb-token": TEST_AUTH_TOKEN,
    },
  });
  expect(res.status()).toBe(200);
  await ctx.dispose();
});

test("authenticated routes reject a cross-origin request", async () => {
  const ctx = await bareContext();
  const res = await ctx.get("/project", {
    headers: {
      "x-sbfb-token": TEST_AUTH_TOKEN,
      origin: "https://attacker.example",
    },
  });
  expect(res.status()).toBe(403);
  await ctx.dispose();
});
