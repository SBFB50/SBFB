// SPDX-License-Identifier: AGPL-3.0-or-later
import '@testing-library/jest-dom/vitest'
import { vi } from 'vitest'
import { i18n } from '../i18n/i18n'

// Sprint 80 (front rapid-add) — activate the source locale on the global i18n
// so the <Trans>/t macros resolve to their source (FR) message in every test
// without each suite loading a catalog. A test that proves another locale
// (verdict.test.ts) loads + activates it, then restores `fr`.
i18n.load('fr', {})
i18n.activate('fr')

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
