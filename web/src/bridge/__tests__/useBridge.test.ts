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
  });

  afterEach(() => {
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
