// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Global Vitest setup. Loaded once via `vitest.config.ts`'s
 * `setupFiles`.
 *
 * - Pulls in `@testing-library/jest-dom` matchers so tests can
 *   use `expect(el).toBeInTheDocument()` etc.
 * - Stubs matchMedia for any code that reads it at import time
 *   (the Sidebar primitive uses it); jsdom does not ship a
 *   default implementation.
 * - Stubs ResizeObserver so `cmdk` (Sprint 6 Phase C command
 *   palette) and any other Radix-style primitive that observes
 *   layout can mount under jsdom. The stub is a no-op; Vitest
 *   assertions do not care about real layout metrics.
 */

import "@testing-library/jest-dom/vitest";

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

if (typeof globalThis !== "undefined" && !("ResizeObserver" in globalThis)) {
  class ResizeObserverStub {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
  (globalThis as unknown as { ResizeObserver: typeof ResizeObserverStub }).ResizeObserver =
    ResizeObserverStub;
}

// jsdom does not implement `Element.prototype.scrollIntoView`;
// cmdk calls it on its selected item in a layout effect, so
// any palette render that mounts a CommandItem throws without
// this stub.
if (
  typeof Element !== "undefined" &&
  typeof Element.prototype.scrollIntoView !== "function"
) {
  Element.prototype.scrollIntoView = function () {};
}
