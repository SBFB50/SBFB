// SPDX-License-Identifier: AGPL-3.0-or-later
import '@testing-library/jest-dom/vitest'
import { vi } from 'vitest'

// Sprint 80 Phase E — jsdom ships no `matchMedia`; Motion's `useReducedMotion`
// and the altitude-shift helper both query it. Default: motion ENABLED
// (matches: false). A test that asserts the reduced-motion path overrides this.
if (!window.matchMedia) {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }))
}
