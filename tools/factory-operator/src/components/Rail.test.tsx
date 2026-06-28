// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { Rail } from './Rail'

// The rail holds the MANUAL focal toggle (D6). Phase H adds the verify-ready
// affordance: a dot that lights when a COMPLETE turn awaits examination — it is
// only an affordance, the toggle stays manual (never an auto-switch).
const base = {
  mode: 'steer' as const,
  onMode: vi.fn(),
  surface: null,
  onSurface: vi.fn(),
  reachable: true,
}

describe('Rail (D6 manual toggle + verify-ready affordance)', () => {
  it('lights the verify-ready dot when a complete turn awaits and VERIFY is not active', () => {
    render(<Rail {...base} mode="steer" verifyReady />)
    expect(screen.getByTestId('verify-ready')).toBeInTheDocument()
  })

  it('hides the dot when VERIFY is already the active focal scene', () => {
    render(<Rail {...base} mode="verify" verifyReady />)
    expect(screen.queryByTestId('verify-ready')).toBeNull()
  })

  it('hides the dot when no complete turn is ready', () => {
    render(<Rail {...base} mode="steer" verifyReady={false} />)
    expect(screen.queryByTestId('verify-ready')).toBeNull()
  })

  it('keeps the bascule MANUAL (the affordance never auto-switches)', () => {
    render(<Rail {...base} verifyReady />)
    expect(screen.getByText(/bascule manuelle · jamais auto/)).toBeInTheDocument()
  })
})
