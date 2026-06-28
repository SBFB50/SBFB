// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { TokenCount } from './TokenCount'

describe('TokenCount (signature 1 — token settle)', () => {
  it('renders the value with the settle keyframe class + tabular-nums', () => {
    render(<TokenCount value={7} />)
    const el = screen.getByTestId('token-count')
    expect(el).toHaveTextContent('7')
    expect(el.className).toContain('motion-settle')
    expect(el.className).toContain('tabular-nums')
  })

  it('updates the rendered value on change (the new token arrives)', () => {
    const { rerender } = render(<TokenCount value={1} />)
    expect(screen.getByTestId('token-count')).toHaveTextContent('1')
    rerender(<TokenCount value={2} />)
    expect(screen.getByTestId('token-count')).toHaveTextContent('2')
  })
})
