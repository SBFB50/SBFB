// SPDX-License-Identifier: AGPL-3.0-or-later
//
// P2-OPERATOR-NO-TEST-RUNNER (S73 Phase B): global Vitest setup, loaded once
// via `vitest.config.ts`'s `setupFiles`. Mirrors `web/src/test/setup.ts`
// (jest-dom matchers + localStorage / matchMedia stubs) and adds a controllable
// EventSource stub — jsdom ships none, and the execution chat opens an SSE
// stream the component must read and `close()`.

import "@testing-library/jest-dom/vitest";

function createMemoryStorage(): Storage {
  const data = new Map<string, string>();
  return {
    get length() {
      return data.size;
    },
    clear: () => data.clear(),
    getItem: (key: string) => data.get(String(key)) ?? null,
    key: (index: number) => Array.from(data.keys())[index] ?? null,
    removeItem: (key: string) => {
      data.delete(String(key));
    },
    setItem: (key: string, value: string) => {
      data.set(String(key), String(value));
    },
  };
}

if (typeof window !== "undefined") {
  const storage =
    typeof window.localStorage?.setItem === "function"
      ? window.localStorage
      : createMemoryStorage();
  Object.defineProperty(window, "localStorage", {
    configurable: true,
    value: storage,
  });
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: storage,
  });
}

if (typeof window !== "undefined" && !("matchMedia" in window)) {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }),
  });
}

/**
 * Controllable EventSource stub. jsdom has no EventSource, and the execution
 * chat opens one via `openStream`. Tests reach the live instance through the
 * static `instances` list, drive it with `emit(...)`, and assert it was
 * `close()`d exactly once (the no-reconnect-storm contract).
 */
export class MockEventSource {
  static instances: MockEventSource[] = [];
  static reset(): void {
    MockEventSource.instances = [];
  }

  url: string;
  readyState = 0;
  closeCount = 0;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;
  onopen: ((ev: Event) => void) | null = null;

  constructor(url: string) {
    this.url = url;
    MockEventSource.instances.push(this);
  }

  /** Push a server-sent data frame into the consumer's `onmessage`. */
  emit(data: string): void {
    this.onmessage?.(new MessageEvent("message", { data }));
  }

  /** Trigger the transport error path (`onerror`). */
  fail(): void {
    this.onerror?.(new Event("error"));
  }

  close(): void {
    this.closeCount += 1;
    this.readyState = 2;
  }

  addEventListener(): void {}
  removeEventListener(): void {}
  dispatchEvent(): boolean {
    return false;
  }
}

(globalThis as unknown as { EventSource: typeof MockEventSource }).EventSource =
  MockEventSource;
