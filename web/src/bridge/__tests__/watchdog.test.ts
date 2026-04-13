// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 15 Phase B — CPU watchdog state machine tests.
 *
 * Exercises the host-side transitions of {@link useBridge}'s
 * `watchdogState` via synthetic heartbeat postMessages and a fake
 * clock. Each test mounts the hook, simulates the stream of events
 * that would come from an iframe, and asserts the observable state
 * at the end.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";

import { STALL_THRESHOLD_MS, useBridge } from "../useBridge";

function makeHeartbeat(tsOverride?: number) {
  return {
    type: "sbfb-bridge-heartbeat" as const,
    ts: tsOverride ?? Date.now(),
  };
}

describe("useBridge watchdog (Sprint 15 Phase B)", () => {
  let iframeRef: React.RefObject<HTMLIFrameElement | null>;
  let fakeWindow: { postMessage: ReturnType<typeof vi.fn> };

  beforeEach(() => {
    vi.useFakeTimers();
    fakeWindow = { postMessage: vi.fn() };
    iframeRef = {
      current: {
        contentWindow: fakeWindow as unknown as Window,
      } as HTMLIFrameElement,
    };
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  function dispatchHeartbeat() {
    window.dispatchEvent(
      new MessageEvent("message", {
        data: makeHeartbeat(),
        source: fakeWindow as unknown as Window,
      }),
    );
  }

  it("starts in unknown state", () => {
    const { result } = renderHook(() =>
      useBridge("http://localhost:8000", "gov", iframeRef),
    );
    expect(result.current.watchdogState).toBe("unknown");
  });

  it("transitions to healthy on first heartbeat", () => {
    const { result } = renderHook(() =>
      useBridge("http://localhost:8000", "gov", iframeRef),
    );

    act(() => {
      dispatchHeartbeat();
    });

    expect(result.current.watchdogState).toBe("healthy");
  });

  it("stays unknown when no heartbeat arrives", () => {
    const { result } = renderHook(() =>
      useBridge("http://localhost:8000", "gov", iframeRef),
    );

    act(() => {
      vi.advanceTimersByTime(STALL_THRESHOLD_MS + 3000);
    });

    expect(result.current.watchdogState).toBe("unknown");
  });

  it("transitions to stalled after STALL_THRESHOLD without heartbeat", () => {
    const { result } = renderHook(() =>
      useBridge("http://localhost:8000", "gov", iframeRef),
    );

    act(() => {
      dispatchHeartbeat();
    });
    expect(result.current.watchdogState).toBe("healthy");

    act(() => {
      vi.advanceTimersByTime(STALL_THRESHOLD_MS + 2500);
    });

    expect(result.current.watchdogState).toBe("stalled");
  });

  it("recovers from stalled to healthy when heartbeat resumes", () => {
    const { result } = renderHook(() =>
      useBridge("http://localhost:8000", "gov", iframeRef),
    );

    act(() => {
      dispatchHeartbeat();
      vi.advanceTimersByTime(STALL_THRESHOLD_MS + 2500);
    });
    expect(result.current.watchdogState).toBe("stalled");

    act(() => {
      dispatchHeartbeat();
    });

    expect(result.current.watchdogState).toBe("healthy");
  });

  it("ignores heartbeats from unknown source", () => {
    const { result } = renderHook(() =>
      useBridge("http://localhost:8000", "gov", iframeRef),
    );

    act(() => {
      window.dispatchEvent(
        new MessageEvent("message", {
          data: makeHeartbeat(),
          source: {} as Window,
        }),
      );
    });

    expect(result.current.watchdogState).toBe("unknown");
  });

  it("resetWatchdog returns state to unknown", () => {
    const { result } = renderHook(() =>
      useBridge("http://localhost:8000", "gov", iframeRef),
    );

    act(() => {
      dispatchHeartbeat();
    });
    expect(result.current.watchdogState).toBe("healthy");

    act(() => {
      result.current.resetWatchdog();
    });
    expect(result.current.watchdogState).toBe("unknown");
  });

  it("does not re-transition to stalled after reset until a new heartbeat stale", () => {
    const { result } = renderHook(() =>
      useBridge("http://localhost:8000", "gov", iframeRef),
    );

    act(() => {
      dispatchHeartbeat();
      vi.advanceTimersByTime(STALL_THRESHOLD_MS + 2500);
    });
    expect(result.current.watchdogState).toBe("stalled");

    act(() => {
      result.current.resetWatchdog();
      vi.advanceTimersByTime(STALL_THRESHOLD_MS + 2500);
    });
    // With lastHeartbeatRef cleared, watchdog stays unknown instead
    // of immediately bouncing back to stalled.
    expect(result.current.watchdogState).toBe("unknown");
  });
});

describe("BridgeHeartbeatSchema", () => {
  it("rejects a heartbeat without a positive ts", async () => {
    const { BridgeHeartbeatSchema } = await import("../protocol");
    expect(
      BridgeHeartbeatSchema.safeParse({
        type: "sbfb-bridge-heartbeat",
        ts: -1,
      }).success,
    ).toBe(false);
  });

  it("rejects a message with the wrong type", async () => {
    const { BridgeHeartbeatSchema } = await import("../protocol");
    expect(
      BridgeHeartbeatSchema.safeParse({
        type: "sbfb-bridge-request",
        ts: 1,
      }).success,
    ).toBe(false);
  });

  it("accepts a valid heartbeat", async () => {
    const { BridgeHeartbeatSchema } = await import("../protocol");
    expect(
      BridgeHeartbeatSchema.safeParse({
        type: "sbfb-bridge-heartbeat",
        ts: Date.now(),
      }).success,
    ).toBe(true);
  });
});
