// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 9 Phase C — `useAppEvents` Vitest contract tests.
 *
 * Six contracts pinning the hook's behaviour:
 *
 * 1. Opens an EventSource against
 *    `${url}/app/${app}/events?pattern=…` on mount.
 * 2. Filtering: a matching envelope schedules
 *    `invalidateQueries`; a malformed payload does not.
 * 3. `invalidateQueries` is called with the configured query
 *    key on every parsed envelope.
 * 4. Closing the host component closes the underlying
 *    EventSource.
 * 5. Error events trigger a reconnect via the injected
 *    factory (timer-driven backoff).
 * 6. Two hooks with different patterns coexist without
 *    leaking subscriptions or invalidations.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import * as React from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { useAppEvents } from "../useAppEvents";

class MockEventSource {
  url: string;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  closed = false;

  constructor(url: string) {
    this.url = url;
    MockEventSource.instances.push(this);
  }

  static instances: MockEventSource[] = [];

  dispatchData(data: unknown): void {
    if (this.onmessage) {
      this.onmessage(new MessageEvent("message", { data: JSON.stringify(data) }));
    }
  }

  dispatchRawData(raw: string): void {
    if (this.onmessage) {
      this.onmessage(new MessageEvent("message", { data: raw }));
    }
  }

  dispatchError(): void {
    if (this.onerror) {
      this.onerror(new Event("error"));
    }
  }

  close(): void {
    this.closed = true;
  }
}

function envelope(topic: string, payload: Record<string, unknown> = {}) {
  return {
    topic,
    payload,
    timestamp: "2026-04-12T00:00:00+00:00",
    trace_id: "abc1234567890def",
  };
}

interface HarnessOptions {
  pattern?: string;
  appName?: string | null;
  url?: string | null;
  invalidateQueryKey?: ReadonlyArray<unknown>;
  onEvent?: (env: any) => void;
}

function renderUseAppEvents(opts: HarnessOptions = {}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const wrapper = ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client: queryClient }, children);
  const factory = vi.fn((url: string) => new MockEventSource(url) as unknown as EventSource);
  const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
  const result = renderHook(
    (props: HarnessOptions = opts) =>
      useAppEvents({
        coordinatorUrl: props.url ?? "http://127.0.0.1:6000",
        appName: props.appName ?? "gov",
        pattern: props.pattern ?? "party.refreshed",
        invalidateQueryKey: (props.invalidateQueryKey ?? ["app-tab", "gov", "Politiciens"]) as any,
        onEvent: props.onEvent,
        eventSourceFactory: factory as unknown as (url: string) => EventSource,
      }),
    { wrapper },
  );
  return { result, factory, invalidateSpy, queryClient };
}

beforeEach(() => {
  MockEventSource.instances = [];
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("useAppEvents", () => {
  it("1. opens an EventSource on mount with the pattern in the query string", () => {
    const { factory } = renderUseAppEvents({
      pattern: "party.refreshed",
      appName: "gov",
      url: "http://127.0.0.1:6000",
    });
    expect(factory).toHaveBeenCalledTimes(1);
    expect(factory.mock.calls[0]?.[0]).toBe(
      "http://127.0.0.1:6000/app/gov/events?pattern=party.refreshed",
    );
    expect(MockEventSource.instances.length).toBe(1);
  });

  it("2. ignores malformed envelopes without invalidating the cache", () => {
    const { invalidateSpy } = renderUseAppEvents();
    const source = MockEventSource.instances[0]!;
    act(() => {
      source.dispatchRawData("not-a-json-payload");
    });
    expect(invalidateSpy).not.toHaveBeenCalled();
    act(() => {
      source.dispatchData({ topic: "", payload: {}, timestamp: "x", trace_id: "" });
    });
    expect(invalidateSpy).not.toHaveBeenCalled();
  });

  it("3. invalidates the configured query key on every parsed envelope", () => {
    const { invalidateSpy } = renderUseAppEvents({
      invalidateQueryKey: ["app-tab", "gov", "Politiciens"],
    });
    const source = MockEventSource.instances[0]!;
    act(() => {
      source.dispatchData(envelope("party.refreshed", { count: 5 }));
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["app-tab", "gov", "Politiciens"],
    });
    act(() => {
      source.dispatchData(envelope("party.refreshed", { count: 6 }));
    });
    expect(invalidateSpy).toHaveBeenCalledTimes(2);
  });

  it("4. closes the underlying EventSource on unmount", () => {
    const { result } = renderUseAppEvents();
    const source = MockEventSource.instances[0]!;
    expect(source.closed).toBe(false);
    result.unmount();
    expect(source.closed).toBe(true);
  });

  it("5. reconnects with backoff after an error event", () => {
    const { factory } = renderUseAppEvents();
    const first = MockEventSource.instances[0]!;
    expect(factory).toHaveBeenCalledTimes(1);
    act(() => {
      first.dispatchError();
    });
    // The first source is closed and a reconnect is scheduled.
    expect(first.closed).toBe(true);
    // Initial backoff is 500 ms — advance the fake timer to fire the reconnect.
    act(() => {
      vi.advanceTimersByTime(500);
    });
    expect(factory).toHaveBeenCalledTimes(2);
    expect(MockEventSource.instances.length).toBe(2);
    expect(MockEventSource.instances[1]?.closed).toBe(false);
  });

  it("6. supports multiple coexisting subscribers without leaking", () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const wrapper = ({ children }: { children: React.ReactNode }) =>
      React.createElement(QueryClientProvider, { client: queryClient }, children);
    const factory = vi.fn(
      (url: string) => new MockEventSource(url) as unknown as EventSource,
    );
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const { unmount: unmountA } = renderHook(
      () =>
        useAppEvents({
          coordinatorUrl: "http://127.0.0.1:6000",
          appName: "gov",
          pattern: "party.refreshed",
          invalidateQueryKey: ["app-tab", "gov", "Politiciens"],
          eventSourceFactory: factory as unknown as (url: string) => EventSource,
        }),
      { wrapper },
    );
    const { unmount: unmountB } = renderHook(
      () =>
        useAppEvents({
          coordinatorUrl: "http://127.0.0.1:6000",
          appName: "gov",
          pattern: "politician.created",
          invalidateQueryKey: ["app-tab", "gov", "Politicien"],
          eventSourceFactory: factory as unknown as (url: string) => EventSource,
        }),
      { wrapper },
    );

    expect(factory).toHaveBeenCalledTimes(2);
    expect(MockEventSource.instances.length).toBe(2);

    const sourceA = MockEventSource.instances[0]!;
    const sourceB = MockEventSource.instances[1]!;

    act(() => {
      sourceA.dispatchData(envelope("party.refreshed", { count: 1 }));
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["app-tab", "gov", "Politiciens"],
    });

    act(() => {
      sourceB.dispatchData(envelope("politician.created", { id: 42 }));
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["app-tab", "gov", "Politicien"],
    });

    unmountA();
    expect(sourceA.closed).toBe(true);
    expect(sourceB.closed).toBe(false);
    unmountB();
    expect(sourceB.closed).toBe(true);
  });
});
