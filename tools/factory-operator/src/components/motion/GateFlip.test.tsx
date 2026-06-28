// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import { LazyMotion, domAnimation } from 'motion/react'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { GateFlip } from './GateFlip'

function setReducedMotion(matches: boolean) {
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

// GateFlip needs the LazyMotion features context (it renders a minimal motion
// component). The static `domAnimation` import is fine in tests (not bundled).
function wrap(ui: ReactNode) {
  return render(<LazyMotion features={domAnimation}>{ui}</LazyMotion>)
}

describe('GateFlip (signature 2 — gate flip)', () => {
  afterEach(() => vi.restoreAllMocks())

  it('restitutes the value (it animates a restituted value, never fabricates one)', () => {
    wrap(<GateFlip value="Inspection bootstrap · terminal + procédé" />)
    expect(screen.getByTestId('gate-flip')).toHaveTextContent('Inspection bootstrap')
  })

  it('never renders a forbidden verdict word', () => {
    wrap(<GateFlip value="En attente de session agent" />)
    expect(screen.getByTestId('gate-flip').textContent).not.toMatch(/PASS|Vérifié|Approuvé/)
  })

  it('renders the value plainly (no motion component) under reduced motion', () => {
    setReducedMotion(true)
    // No LazyMotion wrapper: the reduced-motion path renders a plain <span>.
    render(<GateFlip value="Inspection bootstrap" />)
    expect(screen.getByTestId('gate-flip')).toHaveTextContent('Inspection bootstrap')
  })
})
