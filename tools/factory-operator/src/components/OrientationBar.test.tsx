// SPDX-License-Identifier: AGPL-3.0-or-later
//
// The orientation bar restitutes the live gate pulse (counts per status) and
// exposes the manual refresh. Cardinal invariant: the pulse is a COUNT, never
// an aggregate verdict / PASS word.
import { cleanup, render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { OrientationBar } from './OrientationBar'
import type { RailHandle } from '../state/useRailStatus'
import { ACCESSIBILITY_STORAGE_KEY } from '../preferences/accessibility'

const base: RailHandle = {
  sprint: 80,
  phase: 'Phase I',
  branch: 'master',
  dirty: 2,
  staged: 0,
  gateCounts: null,
  reachable: true,
  loading: false,
  refresh: vi.fn(),
}

afterEach(() => {
  cleanup()
  if (typeof localStorage.removeItem === 'function') localStorage.removeItem(ACCESSIBILITY_STORAGE_KEY)
  document.documentElement.removeAttribute('data-contrast')
  document.documentElement.removeAttribute('data-pointer')
  document.documentElement.removeAttribute('data-text-spacing')
  document.documentElement.removeAttribute('data-font')
  document.documentElement.removeAttribute('data-motion')
  document.documentElement.removeAttribute('data-scale')
})

describe('OrientationBar — pouls de gates + refresh', () => {
  it('restitue un compte par statut de gate (jamais un agrégat PASS)', () => {
    render(
      <OrientationBar
        provider="claude"
        status={{
          ...base,
          gateCounts: { passed: 3, blocking: 1, not_run: 2, informational: 0, not_applicable: 0 },
        }}
      />,
    )
    const pulse = screen.getByTestId('gate-pulse')
    expect(within(pulse).getByText('3')).toBeInTheDocument()
    expect(within(pulse).getByText('1')).toBeInTheDocument()
    expect(within(pulse).getByText('2')).toBeInTheDocument()
    // No fabricated aggregate verdict word anywhere.
    expect(within(pulse).queryByText('PASS')).toBeNull()
    expect(within(pulse).queryByText(/tout vert/i)).toBeNull()
  })

  it('montre un placeholder neutre tant que les gates ne sont pas lus', () => {
    render(<OrientationBar provider="claude" status={base} />)
    expect(screen.queryByTestId('gate-pulse')).toBeNull()
    expect(screen.getByText('gates …')).toBeInTheDocument()
  })

  it('appelle refresh au clic sur le bouton de rafraîchissement', async () => {
    const user = userEvent.setup()
    const refresh = vi.fn()
    render(<OrientationBar provider="claude" status={{ ...base, refresh }} />)
    await user.click(screen.getByTestId('context-refresh'))
    expect(refresh).toHaveBeenCalledTimes(1)
  })

  it('désactive le bouton de refresh pendant le chargement', () => {
    render(<OrientationBar provider="claude" status={{ ...base, loading: true }} />)
    expect(screen.getByTestId('context-refresh')).toBeDisabled()
  })

  it('ouvre le panneau accessibilite et applique les preferences sur html', async () => {
    const user = userEvent.setup()
    render(<OrientationBar provider="claude" status={base} />)
    await user.click(screen.getByTestId('accessibility-toggle'))
    await user.selectOptions(await screen.findByLabelText('contraste'), 'high')
    await user.selectOptions(screen.getByLabelText('cibles'), 'large')
    await waitFor(() => expect(document.documentElement).toHaveAttribute('data-contrast', 'high'))
    expect(document.documentElement).toHaveAttribute('data-pointer', 'large')
  })
})
