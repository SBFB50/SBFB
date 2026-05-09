// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 13 Phase C — useBridge hook tests.
 *
 * Tests the host-side bridge listener by firing synthetic
 * postMessage events and verifying the dispatched responses.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook } from "@testing-library/react";

import { useBridge } from "../useBridge";
import { primeAuthToken } from "@/api/auth";
import type { BridgeRequest, BridgeResponse } from "../protocol";

function makeRequest(overrides: Partial<BridgeRequest> = {}): BridgeRequest {
  return {
    type: "sbfb-bridge-request",
    id: "550e8400-e29b-41d4-a716-446655440000",
    method: "task_submit",
    payload: { prompt: "test" },
    ...overrides,
  };
}

describe("useBridge", () => {
  let iframeRef: React.RefObject<HTMLIFrameElement | null>;
  let fakeWindow: { postMessage: ReturnType<typeof vi.fn> };

  beforeEach(() => {
    fakeWindow = { postMessage: vi.fn() };
    // Create a fake iframe ref whose contentWindow we control.
    iframeRef = {
      current: {
        contentWindow: fakeWindow as unknown as Window,
      } as HTMLIFrameElement,
    };
    primeAuthToken("deadbeefcafebabe0123456789abcdef0123456789abcdef0123456789abcdef");
  });

  afterEach(() => {
    primeAuthToken(null);
    vi.restoreAllMocks();
  });

  it("ignores messages with wrong type", () => {
    renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

    window.dispatchEvent(
      new MessageEvent("message", {
        data: { type: "not-a-bridge", id: "x" },
        source: fakeWindow as unknown as Window,
      }),
    );

    expect(fakeWindow.postMessage).not.toHaveBeenCalled();
  });

  it("ignores messages from unknown source", () => {
    renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

    const otherWindow = {} as Window;
    window.dispatchEvent(
      new MessageEvent("message", {
        data: makeRequest(),
        source: otherWindow,
      }),
    );

    expect(fakeWindow.postMessage).not.toHaveBeenCalled();
  });

  it("sends error response when no coordinator URL", async () => {
    renderHook(() => useBridge(null, "gov", iframeRef));

    window.dispatchEvent(
      new MessageEvent("message", {
        data: makeRequest(),
        source: fakeWindow as unknown as Window,
      }),
    );

    // Wait for async dispatch.
    await vi.waitFor(() => {
      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
    });

    const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
    expect(resp.type).toBe("sbfb-bridge-response");
    expect(resp.success).toBe(false);
    expect(resp.error).toContain("no active coordinator");
  });

  describe("pii_redact dispatch (Sprint 21 Phase B)", () => {
    it("redacts without requiring a coordinator URL", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch");
      renderHook(() => useBridge(null, null, iframeRef));

      // `use_model: false` forces the regex fallback path — jsdom
      // has no WASM runtime, so the real ONNX loader would hang
      // otherwise. The model path is covered by the wrapper tests
      // that inject a stub ModelLoader.
      const req = makeRequest({
        id: "11111111-1111-4111-8111-111111111111",
        method: "pii_redact",
        payload: { text: "Email foo@bar.com", policy: { use_model: false } },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { redacted_text: string; findings_count: number };
      expect(data.redacted_text).toContain("[REDACTED:EMAIL_ADDRESS]");
      expect(data.redacted_text).not.toContain("foo@bar.com");
      expect(data.findings_count).toBeGreaterThan(0);

      // Local dispatch: the coord was never called.
      expect(fetchSpy).not.toHaveBeenCalled();
    });
  });

  describe("bridge extensions Sprint 56 Phase C", () => {
    it("dispatches storage_list via fetch", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ entries: [], count: 0 }), { status: 200 }),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "22222222-2222-4222-8222-222222222222",
        method: "storage_list",
        payload: { prefix: "user:" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      expect(fetchSpy).toHaveBeenCalledWith(
        expect.stringContaining("/app/gov/state?prefix=user%3A"),
        expect.any(Object),
      );
    });

    it("dispatches storage_delete via DELETE fetch", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ ok: true }), { status: 200 }),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "33333333-3333-4333-8333-333333333333",
        method: "storage_delete",
        payload: { key: "temp" },
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      expect(fetchSpy).toHaveBeenCalledWith(
        expect.stringContaining("/app/gov/state/temp"),
        expect.objectContaining({ method: "DELETE" }),
      );
    });

    it("dispatches identity_pubkey via daemon info", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ node_id: "abc123def456" }), { status: 200 }),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "44444444-4444-4444-8444-444444444444",
        method: "identity_pubkey",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      expect((resp.data as { pubkey: string }).pubkey).toBe("abc123def456");
    });

    it("dispatches node_status with peers enrichment", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch")
        .mockResolvedValueOnce(
          new Response(
            JSON.stringify({ status: "ok", uptime_secs: 42 }),
            { status: 200 },
          ),
        )
        .mockResolvedValueOnce(
          new Response(
            JSON.stringify({ subscribed_curators: ["aaa", "bbb"] }),
            { status: 200 },
          ),
        );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "55555555-5555-4555-8555-555555555555",
        method: "node_status",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { status: string; peers: number };
      expect(data.status).toBe("ok");
      expect(data.peers).toBe(2);
      expect(fetchSpy).toHaveBeenCalledTimes(2);
    });

    it("dispatches browse_list via daemon browse", async () => {
      vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(
          JSON.stringify({ entries: [{ project_name: "Hello" }] }),
          { status: 200 },
        ),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "66666666-6666-4666-8666-666666666666",
        method: "browse_list",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const resp = fakeWindow.postMessage.mock.calls[0][0] as BridgeResponse;
      expect(resp.success).toBe(true);
      const data = resp.data as { entries: Array<{ project_name: string }> };
      expect(data.entries).toHaveLength(1);
      expect(data.entries[0].project_name).toBe("Hello");
    });

    it("injects x-sbfb-token header via authFetch", async () => {
      const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(
        new Response(JSON.stringify({ entries: [] }), { status: 200 }),
      );
      renderHook(() => useBridge("http://localhost:8000", "gov", iframeRef));

      const req = makeRequest({
        id: "77777777-7777-4777-8777-777777777777",
        method: "browse_list",
        payload: {},
      });
      window.dispatchEvent(
        new MessageEvent("message", {
          data: req,
          source: fakeWindow as unknown as Window,
        }),
      );

      await vi.waitFor(() => {
        expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      });

      const [, init] = fetchSpy.mock.calls[0];
      const headers = init?.headers as Headers;
      expect(headers.get("x-sbfb-token")).toBe(
        "deadbeefcafebabe0123456789abcdef0123456789abcdef0123456789abcdef",
      );
    });
  });

  describe("pushEvent (Sprint 15 Phase A)", () => {
    it("posts a bridge-event to the iframe contentWindow", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );

      result.current.pushEvent("task_result_ready", { task_id: "t-42" });

      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(1);
      const [msg, targetOrigin] = fakeWindow.postMessage.mock.calls[0];
      expect(msg).toEqual({
        type: "sbfb-bridge-event",
        name: "task_result_ready",
        payload: { task_id: "t-42" },
      });
      expect(targetOrigin).toBe("*");
    });

    it("is a no-op when iframeRef.current is null", () => {
      const nullRef = { current: null } as React.RefObject<HTMLIFrameElement | null>;
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", nullRef),
      );

      expect(() => result.current.pushEvent("foo", {})).not.toThrow();
      expect(fakeWindow.postMessage).not.toHaveBeenCalled();
    });

    it("is a no-op when contentWindow is null", () => {
      const noWindowRef = {
        current: { contentWindow: null } as unknown as HTMLIFrameElement,
      } as React.RefObject<HTMLIFrameElement | null>;
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", noWindowRef),
      );

      expect(() => result.current.pushEvent("foo", null)).not.toThrow();
      expect(fakeWindow.postMessage).not.toHaveBeenCalled();
    });

    it("allows arbitrary payload types", () => {
      const { result } = renderHook(() =>
        useBridge("http://localhost:8000", "gov", iframeRef),
      );

      result.current.pushEvent("string_payload", "hello");
      result.current.pushEvent("number_payload", 42);
      result.current.pushEvent("array_payload", [1, 2, 3]);

      expect(fakeWindow.postMessage).toHaveBeenCalledTimes(3);
      expect(fakeWindow.postMessage.mock.calls[0][0].payload).toBe("hello");
      expect(fakeWindow.postMessage.mock.calls[1][0].payload).toBe(42);
      expect(fakeWindow.postMessage.mock.calls[2][0].payload).toEqual([1, 2, 3]);
    });
  });
});
