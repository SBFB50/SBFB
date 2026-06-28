// SPDX-License-Identifier: AGPL-3.0-or-later
import { act, renderHook } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { usePrefersReducedMotion } from './usePrefersReducedMotion'

function setMatch(matches: boolean) {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }))
}

describe('usePrefersReducedMotion', () => {
  afterEach(() => vi.restoreAllMocks())

  it('returns false when the user expresses no reduced-motion preference', () => {
    setMatch(false)
    const { result } = renderHook(() => usePrefersReducedMotion())
    expect(result.current).toBe(false)
  })

  it('returns true synchronously on first render under prefers-reduced-motion', () => {
    setMatch(true)
    const { result } = renderHook(() => usePrefersReducedMotion())
    expect(result.current).toBe(true)
  })

  it('updates live when the OS reduced-motion setting changes', () => {
    let matches = false
    let handler: (() => void) | null = null
    window.matchMedia = vi.fn().mockImplementation((query: string) => ({
      get matches() {
        return matches
      },
      media: query,
      onchange: null,
      addEventListener: (_event: string, cb: () => void) => {
        handler = cb
      },
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))
    const { result } = renderHook(() => usePrefersReducedMotion())
    expect(result.current).toBe(false)
    act(() => {
      matches = true
      handler?.()
    })
    expect(result.current).toBe(true)
  })
})
