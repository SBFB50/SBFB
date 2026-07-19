// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase C — useBridge authority intersection (integration).
 *
 * Verifies the host consults `declared ∩ grant ∩ policy ∩ consent` at the per-call
 * choke-point BEFORE any dispatch: a VALID iframe message for a non-authorized
 * method is refused (fail-closed, rouge-avant/vert-apres), the offline/local
 * pii_redact effect is governed too, and expiry/revocation are honoured per call.
 * A legacy app (no authority arg) is grandfathered.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook } from "@testing-library/react";

import { useBridge } from "../useBridge";
import { primeAuthToken } from "@/api/auth";
import type { BridgeRequest, BridgeResponse } from "../protocol";
import { resolveDeclaredAuthority, type BridgeAuthority } from "../authority";

function makeRequest(overrides: Partial<BridgeRequest> = {}): BridgeRequest {
  return {
    type: "sbfb-bridge-request",
    id: "550e8400-e29b-41d4-a716-446655440000",
    method: "task_submit",
    payload: {},
    ...overrides,
  };
}

describe("useBridge authority intersection (Sprint 83 Phase C)", () => {
  let iframeRef: React.RefObject<HTMLIFrameElement | null>;
  let fakeWindow: { postMessage: ReturnType<typeof vi.fn> };

  beforeEach(() => {
    fakeWindow = { postMessage: vi.fn() };
    iframeRef = {
      current: {
        contentWindow: fakeWindow as unknown as Window,
      } as HTMLIFrameElement,
    };
    primeAuthToken(
      "deadbeefcafebabe0123456789abcdef0123456789abcdef0123456789abcdef",
    );
  });

  afterEach(() => {
    primeAuthToken(null);
    vi.restoreAllMocks();
  });

  function dispatch(req: BridgeRequest) {
    window.dispatchEvent(
      new MessageEvent("message", {
        data: req,
        source: fakeWindow as unknown as Window,
      }),
    );
  }

  it("refuses a VALID message for an UNDECLARED method (fail-closed, no fetch)", () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch");
    // The app declares storage_get only; task_submit is not in its release layer.
    const authority = resolveDeclaredAuthority(["storage_get"]);
    renderHook(() => useBridge("http://localhost:8000", "hub", iframeRef, authority));

    dispatch(
      makeRequest({
        id: "11111111-1111-4111-8111-111111111111",
        method: "task_submit",
        payload: { prompt: "x" },
      }),
    );

    // Synchronous refuse reply, BEFORE any dispatch.
    expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
    const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
    expect(resp.success).toBe(false);
    expect(resp.error).toContain("not authorized");
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it("dispatches a DECLARED method (vert-apres companion)", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(new Response(JSON.stringify({ value: 1 }), { status: 200 }));
    const authority = resolveDeclaredAuthority(["storage_get"]);
    renderHook(() => useBridge("http://localhost:8000", "hub", iframeRef, authority));

    dispatch(
      makeRequest({
        id: "22222222-2222-4222-8222-222222222222",
        method: "storage_get",
        payload: { key: "k" },
      }),
    );

    await vi.waitFor(() => expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1));
    const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
    expect(resp.success).toBe(true);
    expect(fetchSpy).toHaveBeenCalled();
  });

  it("grandfathers a legacy app (no authority arg) — a dispatch verb still flows", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ entries: [] }), { status: 200 }),
    );
    renderHook(() => useBridge("http://localhost:8000", "legacy", iframeRef)); // no authority

    dispatch(
      makeRequest({
        id: "33333333-3333-4333-8333-333333333333",
        method: "browse_list",
        payload: {},
      }),
    );

    await vi.waitFor(() => expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1));
    expect((fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse).success).toBe(true);
  });

  it("refuses when the grant has EXPIRED (per-call)", () => {
    const base = resolveDeclaredAuthority(["search"]);
    const expired: BridgeAuthority = {
      ...base,
      grant: { ...base.grant, expiresAtMs: 1 }, // Date.now() >> 1 -> expired
    };
    renderHook(() => useBridge("http://localhost:8000", "hub", iframeRef, expired));

    dispatch(
      makeRequest({
        id: "44444444-4444-4444-8444-444444444444",
        method: "search",
        payload: { q: "x" },
      }),
    );

    expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
    expect((fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse).success).toBe(false);
  });

  it("honours REVOCATION on the NEXT call (per-call, before/after)", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );
    const live = resolveDeclaredAuthority(["search"]);
    const revoked: BridgeAuthority = {
      ...live,
      grant: { ...live.grant, revoked: true },
    };
    // Re-render the hook with a changing authority prop so the host's per-call ref is
    // exercised (not a statically-revoked mount).
    const { rerender } = renderHook(
      ({ auth }: { auth: BridgeAuthority }) =>
        useBridge("http://localhost:8000", "hub", iframeRef, auth),
      { initialProps: { auth: live } },
    );

    // 1) live grant -> allowed (dispatches).
    dispatch(
      makeRequest({
        id: "55555555-5555-4555-8555-555555555555",
        method: "search",
        payload: { q: "x" },
      }),
    );
    await vi.waitFor(() => expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1));
    expect((fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse).success).toBe(true);

    // 2) revoke BETWEEN calls -> the same app is refused on the next call.
    rerender({ auth: revoked });
    dispatch(
      makeRequest({
        id: "5a5a5a5a-5a5a-45a5-8a5a-5a5a5a5a5a5a",
        method: "search",
        payload: { q: "y" },
      }),
    );
    expect(fakeWindow.postMessage).toHaveBeenCalledTimes(2);
    expect((fakeWindow.postMessage.mock.calls[1][0] as BridgeResponse).success).toBe(false);
  });

  it("governs the OFFLINE/local pii_redact effect too (refused when not authorized)", () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch");
    // Declares storage_get only; pii_redact NOT declared. coordUrl null (offline).
    const authority = resolveDeclaredAuthority(["storage_get"]);
    renderHook(() => useBridge(null, null, iframeRef, authority));

    dispatch(
      makeRequest({
        id: "66666666-6666-4666-8666-666666666666",
        method: "pii_redact",
        payload: { text: "Email foo@bar.com", policy: { use_model: false } },
      }),
    );

    // The intersection sits BEFORE the pii_redact branch, so the offline effect is
    // refused synchronously — the local PII SDK is never reached.
    expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
    const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
    expect(resp.success).toBe(false);
    expect(resp.error).toContain("not authorized");
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it("ALLOWS pii_redact offline when the app declares it", async () => {
    const authority = resolveDeclaredAuthority(["pii_redact"]);
    renderHook(() => useBridge(null, null, iframeRef, authority));

    dispatch(
      makeRequest({
        id: "77777777-7777-4777-8777-777777777777",
        method: "pii_redact",
        payload: { text: "Email foo@bar.com", policy: { use_model: false } },
      }),
    );

    await vi.waitFor(() => expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1));
    const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
    expect(resp.success).toBe(true);
    expect((resp.data as { redacted_text: string }).redacted_text).toContain(
      "[REDACTED:EMAIL_ADDRESS]",
    );
  });
});
