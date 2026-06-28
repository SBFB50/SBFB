// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import { LazyMotion, domAnimation } from 'motion/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { Reveal, RevealItem } from './Reveal'

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

describe('Reveal (signature 3 — verification reveal)', () => {
  afterEach(() => vi.restoreAllMocks())

  it('renders every child (the stagger reveals evidence, never hides it)', () => {
    render(
      <LazyMotion features={domAnimation}>
        <Reveal>
          <RevealItem>premier artefact</RevealItem>
          <RevealItem>second artefact</RevealItem>
        </Reveal>
      </LazyMotion>,
    )
    expect(screen.getByTestId('reveal')).toBeInTheDocument()
    expect(screen.getByText('premier artefact')).toBeInTheDocument()
    expect(screen.getByText('second artefact')).toBeInTheDocument()
  })

  it('renders children plainly (no motion component) under reduced motion', () => {
    setReducedMotion(true)
    // No LazyMotion wrapper: the reduced-motion path renders plain <div>s.
    render(
      <Reveal>
        <RevealItem>artefact</RevealItem>
      </Reveal>,
    )
    expect(screen.getByTestId('reveal')).toBeInTheDocument()
    expect(screen.getByText('artefact')).toBeInTheDocument()
  })
})
