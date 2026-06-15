// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Route-prefix reconciliation for the consent client — Sprint 76
 * Phase A (D1).
 *
 * The daemon mounts every consent endpoint under `/api/v1/consent*`
 * (`http.rs`). Before this phase the client posted to bare
 * `/consent/*`, which fell through to the SPA GET fallback and never
 * reached the handlers — the panel was inert in a packaged build.
 * These tests pin the prefixed URLs so the wiring can't silently
 * regress.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { primeAuthToken } from "@/api/auth";
import {
  type ConsentConfig,
  DEFAULT_CONSENT,
  addToWhitelist,
  getConsent,
  removeFromWhitelist,
  setConsent,
} from "@/api/consent";

const BASE = "http://127.0.0.1:7777";
const NODE_ID = "a".repeat(64);

const sampleConfig: ConsentConfig = { ...DEFAULT_CONSENT, own_node_id: "self" };

function okJson(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { "content-type": "application/json" },
  });
}

beforeEach(() => {
  primeAuthToken("test-token");
  vi.stubGlobal("fetch", vi.fn());
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  primeAuthToken(null);
});

describe("consent client — /api/v1 route prefix (Sprint 76 Phase A)", () => {
  it("getConsent lit GET /api/v1/consent (pas /consent/get)", async () => {
    const fetchMock = vi.mocked(globalThis.fetch);
    fetchMock.mockResolvedValue(okJson(sampleConfig));

    await getConsent(BASE);

    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe(`${BASE}/api/v1/consent`);
    expect(init?.method ?? "GET").toBe("GET");
  });

  it("setConsent POST /api/v1/consent/set", async () => {
    const fetchMock = vi.mocked(globalThis.fetch);
    fetchMock.mockResolvedValue(okJson(sampleConfig));

    await setConsent(BASE, sampleConfig);

    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe(`${BASE}/api/v1/consent/set`);
    expect(init?.method).toBe("POST");
  });

  it("la whitelist add/remove POST sous /api/v1/consent/whitelist/*", async () => {
    const fetchMock = vi.mocked(globalThis.fetch);
    // Fresh Response per call — a Response body can only be read once.
    fetchMock.mockImplementation(async () => okJson(sampleConfig));

    await addToWhitelist(BASE, NODE_ID);
    expect(fetchMock.mock.calls[0][0]).toBe(
      `${BASE}/api/v1/consent/whitelist/add`,
    );

    await removeFromWhitelist(BASE, NODE_ID);
    expect(fetchMock.mock.calls[1][0]).toBe(
      `${BASE}/api/v1/consent/whitelist/remove`,
    );
  });
});
