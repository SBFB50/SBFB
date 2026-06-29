// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { AdaptiveSurface } from './AdaptiveSurface'

describe('AdaptiveSurface', () => {
  it('marks a page or sub-page as an accessibility-adaptable surface', () => {
    render(
      <AdaptiveSurface as="section" kind="verify" testId="surface" labelledBy="title" className="extra">
        <h1 id="title">VERIFY</h1>
      </AdaptiveSurface>,
    )

    const surface = screen.getByTestId('surface')
    expect(surface.tagName).toBe('SECTION')
    expect(surface).toHaveAttribute('data-adaptive-surface', 'verify')
    expect(surface).toHaveAttribute('aria-labelledby', 'title')
    expect(surface.className).toContain('adaptive-surface')
    expect(surface.className).toContain('extra')
  })
})
