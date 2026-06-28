// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ErrorBoundary } from './ErrorBoundary'

function Bomb({ boom }: { boom: boolean }): React.JSX.Element {
  if (boom) throw new Error('kaboom')
  return <p>contenu vivant</p>
}

describe('ErrorBoundary (résilience de rendu)', () => {
  beforeEach(() => {
    // The boundary logs the raw error on catch — silence it in the test output.
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })
  afterEach(() => vi.restoreAllMocks())

  it('rend les enfants quand il n y a pas d erreur', () => {
    render(
      <ErrorBoundary>
        <p>contenu vivant</p>
      </ErrorBoundary>,
    )
    expect(screen.getByText('contenu vivant')).toBeInTheDocument()
    expect(screen.queryByTestId('error-boundary-fallback')).toBeNull()
  })

  it('restitue le message d erreur dans le panneau de secours quand un enfant jette', () => {
    render(
      <ErrorBoundary>
        <Bomb boom />
      </ErrorBoundary>,
    )
    const fallback = screen.getByTestId('error-boundary-fallback')
    expect(fallback).toBeInTheDocument()
    // The boundary RESTITUTES the raw message, never swallows it.
    expect(screen.getByText('kaboom')).toBeInTheDocument()
  })

  it('nomme le scope quand il est fourni', () => {
    render(
      <ErrorBoundary scope="surface focale">
        <Bomb boom />
      </ErrorBoundary>,
    )
    expect(screen.getByText('surface focale — erreur de rendu')).toBeInTheDocument()
  })

  it('réessaye le rendu et restaure le contenu si l erreur a disparu', async () => {
    const user = userEvent.setup()
    function Flaky(): React.JSX.Element {
      // First mount throws; after reset the same subtree re-renders cleanly.
      return <Bomb boom={shouldBoom} />
    }
    let shouldBoom = true
    const { rerender } = render(
      <ErrorBoundary>
        <Flaky />
      </ErrorBoundary>,
    )
    expect(screen.getByTestId('error-boundary-fallback')).toBeInTheDocument()
    shouldBoom = false
    await user.click(screen.getByRole('button', { name: 'Réessayer le rendu' }))
    rerender(
      <ErrorBoundary>
        <Flaky />
      </ErrorBoundary>,
    )
    expect(screen.getByText('contenu vivant')).toBeInTheDocument()
    expect(screen.queryByTestId('error-boundary-fallback')).toBeNull()
  })
})
