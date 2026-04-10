/**
 * Global Vitest setup. Loaded once via `vitest.config.ts`'s
 * `setupFiles`.
 *
 * - Pulls in `@testing-library/jest-dom` matchers so tests can
 *   use `expect(el).toBeInTheDocument()` etc.
 * - Stubs matchMedia for any code that reads it at import time
 *   (the Sidebar primitive uses it); jsdom does not ship a
 *   default implementation.
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
