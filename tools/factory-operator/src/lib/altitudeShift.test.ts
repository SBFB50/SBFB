// SPDX-License-Identifier: AGPL-3.0-or-later
import { afterEach, describe, expect, it, vi } from 'vitest'
import { altitudeShift } from './altitudeShift'

// `startViewTransition` is declared on Document (lib.dom) but absent at runtime
// in jsdom, so we set/remove it through an `unknown` cast / Reflect (a plain
// `delete` is a type error on a non-optional member).
function setStartViewTransition(fn: ((cb: () => void) => unknown) | undefined) {
  if (fn) {
    ;(document as unknown as { startViewTransition: (cb: () => void) => unknown }).startViewTransition = fn
  } else {
    Reflect.deleteProperty(document, 'startViewTransition')
  }
}

function setReducedMotion(reduce: boolean) {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches: reduce,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }))
}

describe('altitudeShift', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    setStartViewTransition(undefined)
  })

  it('applies synchronously when the View Transition API is absent', () => {
    setReducedMotion(false)
    setStartViewTransition(undefined)
    const apply = vi.fn()
    altitudeShift(apply)
    expect(apply).toHaveBeenCalledTimes(1)
  })

  it('skips the View Transition under prefers-reduced-motion (anti-déco)', () => {
    setReducedMotion(true)
    const svt = vi.fn()
    setStartViewTransition(svt)
    const apply = vi.fn()
    altitudeShift(apply)
    expect(svt).not.toHaveBeenCalled()
    expect(apply).toHaveBeenCalledTimes(1)
  })

  it('runs the apply inside startViewTransition when motion is allowed', () => {
    setReducedMotion(false)
    const svt = vi.fn().mockImplementation((cb: () => void) => {
      cb()
      return { finished: Promise.resolve() }
    })
    setStartViewTransition(svt)
    const apply = vi.fn()
    altitudeShift(apply)
    expect(svt).toHaveBeenCalledTimes(1)
    expect(apply).toHaveBeenCalledTimes(1)
  })
})
