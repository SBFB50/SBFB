// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 16 Phase A (D1) — loopback bearer auth client.
 *
 * The launcher writes a 256-bit hex token at boot, exposes it on
 * an ephemeral loopback port, and persists the port in
 * `~/.sbfb/launcher.json`. This module:
 *
 *   1. Resolves the launcher URL (env override `VITE_SBFB_LAUNCHER_URL`
 *      falls back to `http://127.0.0.1:<launcher_port>` from
 *      `/launcher/info` on the coordinator — Phase C+).
 *   2. Fetches the token once at shell boot and caches it.
 *   3. Exposes `authFetch(url, init?)` — a `fetch` drop-in that
 *      injects `X-SBFB-Token: <token>` on every request. The
 *      coordinator and daemon proxy helpers route through this
 *      wrapper so no component has to know the token exists.
 *
 * If the launcher is unreachable the shell still boots with an
 * empty token: the middleware returns 401 on every
 * authenticated call, which the React layer renders as a
 * "daemon not ready" state.
 */

/** HTTP header name matching the Rust / Python middleware. */
export const AUTH_HEADER = "x-sbfb-token";

/**
 * Response shape of `GET /auth/token` on the launcher.
 * Mirrors `crates/nexus-launcher/src/auth.rs::TokenResponse`.
 */
interface TokenResponse {
  token: string;
}

/** Cached token resolved via {@link fetchAuthToken}. */
let cachedToken: string | null = null;

/** Overrideable launcher base URL. Set by {@link setLauncherUrl}. */
let launcherBaseUrl: string | null = null;

/**
 * Seed the token cache from a caller that already knows it —
 * Playwright's `page.addInitScript`, a packaging bootstrap that
 * reads the token out of `~/.sbfb/auth_token` and hands it over,
 * or tests. When seeded, subsequent {@link fetchAuthToken} calls
 * return the seeded value without hitting the launcher.
 *
 * Pass `null` to clear the cache (tests only).
 */
export function primeAuthToken(token: string | null): void {
  cachedToken = token;
}

/** Test-only alias retained for clarity in vitest files. */
export const __setAuthTokenForTest = primeAuthToken;

/** Override the launcher base URL (tests + bootstrap). */
export function setLauncherUrl(url: string | null): void {
  launcherBaseUrl = url;
}

/** Resolve the launcher URL. Prefers the value injected via
 *  {@link setLauncherUrl}, otherwise the `VITE_SBFB_LAUNCHER_URL`
 *  build-time env var, otherwise returns `null` — which forces
 *  {@link fetchAuthToken} to surface the missing-launcher state. */
function resolveLauncherUrl(): string | null {
  if (launcherBaseUrl) return launcherBaseUrl;
  const env =
    (typeof import.meta !== "undefined" &&
      (import.meta as { env?: Record<string, string | undefined> }).env
        ?.VITE_SBFB_LAUNCHER_URL) ||
    null;
  return env ?? null;
}

/**
 * Fetch the loopback token from the launcher, caching the
 * result for the lifetime of the page. Subsequent calls return
 * the cache without a network hit.
 */
export async function fetchAuthToken(): Promise<string> {
  if (cachedToken) return cachedToken;
  const base = resolveLauncherUrl();
  if (!base) {
    throw new Error(
      "launcher URL unknown — set VITE_SBFB_LAUNCHER_URL or call setLauncherUrl()",
    );
  }
  const res = await fetch(`${base}/auth/token`, { method: "GET" });
  if (!res.ok) {
    throw new Error(`launcher /auth/token returned ${res.status}`);
  }
  const body = (await res.json()) as TokenResponse;
  if (typeof body.token !== "string" || body.token.length !== 64) {
    throw new Error("launcher /auth/token returned malformed body");
  }
  cachedToken = body.token;
  return body.token;
}

/**
 * `fetch` wrapper that injects the loopback bearer on every
 * request to the daemon or coordinator. Falls back to an
 * unauthenticated fetch if the token is not cached yet — the
 * server will reply 401 and the caller renders the error as a
 * "daemon not ready" state rather than crashing.
 *
 * Exported so every API helper (`coordinator.ts`, `daemon.ts`)
 * routes through a single point. No component in the shell
 * should call the native `fetch` directly — see R2 in
 * `.planning/sprint5_plan.md`.
 */
export async function authFetch(
  url: string,
  init?: RequestInit,
): Promise<Response> {
  const headers = new Headers(init?.headers);
  if (cachedToken && !headers.has(AUTH_HEADER)) {
    headers.set(AUTH_HEADER, cachedToken);
  }
  return fetch(url, { ...init, headers });
}
