// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 16 Phase A (D1) — unit tests for the loopback bearer
 * auth client (`src/api/auth.ts`).
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  AUTH_HEADER,
  __setAuthTokenForTest,
  authFetch,
  fetchAuthToken,
  setLauncherUrl,
} from "@/api/auth";

const VALID_TOKEN =
  "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef";

describe("api/auth", () => {
  beforeEach(() => {
    __setAuthTokenForTest(null);
    setLauncherUrl(null);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("fetchAuthToken", () => {
    it("returns the cached token without a network hit", async () => {
      __setAuthTokenForTest(VALID_TOKEN);
      const spy = vi.spyOn(globalThis, "fetch");
      await expect(fetchAuthToken()).resolves.toBe(VALID_TOKEN);
      expect(spy).not.toHaveBeenCalled();
    });

    it("falls back to same-origin /auth/token when no launcher URL", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response(JSON.stringify({ token: VALID_TOKEN }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );
      await expect(fetchAuthToken()).resolves.toBe(VALID_TOKEN);
      expect(globalThis.fetch).toHaveBeenCalledWith("/auth/token", {
        method: "GET",
      });
    });

    it("rejects when same-origin fallback returns non-2xx", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("nope", { status: 403 }),
      );
      await expect(fetchAuthToken()).rejects.toThrow(/403/);
    });

    it("rejects when the launcher returns a non-2xx", async () => {
      setLauncherUrl("http://127.0.0.1:33333");
      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("nope", { status: 503 }),
      );
      await expect(fetchAuthToken()).rejects.toThrow(/503/);
    });

    it("rejects when the launcher body shape is wrong", async () => {
      setLauncherUrl("http://127.0.0.1:33333");
      vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response(JSON.stringify({ token: "tooShort" }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );
      await expect(fetchAuthToken()).rejects.toThrow(/malformed/);
    });

    it("caches the token after a successful fetch", async () => {
      setLauncherUrl("http://127.0.0.1:33333");
      const spy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response(JSON.stringify({ token: VALID_TOKEN }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );
      await expect(fetchAuthToken()).resolves.toBe(VALID_TOKEN);
      // Second call must be served from cache.
      await expect(fetchAuthToken()).resolves.toBe(VALID_TOKEN);
      expect(spy).toHaveBeenCalledTimes(1);
    });
  });

  describe("authFetch", () => {
    it("injects the X-SBFB-Token header when the token is cached", async () => {
      __setAuthTokenForTest(VALID_TOKEN);
      const spy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("ok", { status: 200 }),
      );
      await authFetch("http://127.0.0.1/some/path");
      expect(spy).toHaveBeenCalledOnce();
      const init = spy.mock.calls[0][1] as RequestInit | undefined;
      const hdrs = new Headers(init?.headers);
      expect(hdrs.get(AUTH_HEADER)).toBe(VALID_TOKEN);
    });

    it("skips header injection when no token is cached", async () => {
      const spy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("ok", { status: 200 }),
      );
      await authFetch("http://127.0.0.1/some/path");
      const init = spy.mock.calls[0][1] as RequestInit | undefined;
      const hdrs = new Headers(init?.headers);
      expect(hdrs.has(AUTH_HEADER)).toBe(false);
    });

    it("preserves caller headers alongside the token", async () => {
      __setAuthTokenForTest(VALID_TOKEN);
      const spy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("ok", { status: 200 }),
      );
      await authFetch("http://127.0.0.1/some/path", {
        headers: { "x-custom": "yes" },
      });
      const init = spy.mock.calls[0][1] as RequestInit | undefined;
      const hdrs = new Headers(init?.headers);
      expect(hdrs.get("x-custom")).toBe("yes");
      expect(hdrs.get(AUTH_HEADER)).toBe(VALID_TOKEN);
    });

    it("does not override a token header already set by the caller", async () => {
      __setAuthTokenForTest(VALID_TOKEN);
      const spy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("ok", { status: 200 }),
      );
      await authFetch("http://127.0.0.1/some/path", {
        headers: { [AUTH_HEADER]: "override-token" },
      });
      const init = spy.mock.calls[0][1] as RequestInit | undefined;
      const hdrs = new Headers(init?.headers);
      expect(hdrs.get(AUTH_HEADER)).toBe("override-token");
    });

    it("forwards method + body to fetch verbatim", async () => {
      __setAuthTokenForTest(VALID_TOKEN);
      const spy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
        new Response("ok", { status: 200 }),
      );
      await authFetch("http://127.0.0.1/submit", {
        method: "POST",
        body: JSON.stringify({ hello: "world" }),
        headers: { "content-type": "application/json" },
      });
      const init = spy.mock.calls[0][1] as RequestInit | undefined;
      expect(init?.method).toBe("POST");
      expect(init?.body).toBe('{"hello":"world"}');
    });
  });
});
