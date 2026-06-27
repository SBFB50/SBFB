// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { Composer } from './Composer'

function renderComposer(overrides: Partial<Parameters<typeof Composer>[0]> = {}) {
  const onLaunch = vi.fn()
  const onProvider = vi.fn()
  render(
    <Composer
      variant="grand"
      provider="claude"
      onProvider={onProvider}
      busy={false}
      onLaunch={onLaunch}
      {...overrides}
    />,
  )
  return { onLaunch, onProvider }
}

describe('Composer', () => {
  it('renders the three intention CTAs (intentions-pas-jargon)', () => {
    renderComposer()
    expect(screen.getByRole('button', { name: 'Préparer la phase' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Vérifier avant validation' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Transmettre à un autre agent' })).toBeInTheDocument()
  })

  it('launches with the typed text and the active intention kind', async () => {
    const { onLaunch } = renderComposer()
    await userEvent.type(screen.getByTestId('composer-input'), 'scelle l’aperçu')
    await userEvent.click(screen.getByTestId('composer-launch'))
    expect(onLaunch).toHaveBeenCalledWith('scelle l’aperçu', 'preflight')
  })

  it('routes a different intention to its kind', async () => {
    const { onLaunch } = renderComposer()
    await userEvent.click(screen.getByRole('button', { name: 'Vérifier avant validation' }))
    await userEvent.type(screen.getByTestId('composer-input'), 'revois le diff')
    await userEvent.click(screen.getByTestId('composer-launch'))
    expect(onLaunch).toHaveBeenCalledWith('revois le diff', 'phase-review')
  })

  it('disables the launch until an intention is described', async () => {
    const { onLaunch } = renderComposer()
    expect(screen.getByTestId('composer-launch')).toBeDisabled()
    await userEvent.click(screen.getByTestId('composer-launch'))
    expect(onLaunch).not.toHaveBeenCalled()
  })

  it('changes the execution provider attribute', async () => {
    const { onProvider } = renderComposer()
    await userEvent.selectOptions(screen.getByTestId('provider-select'), 'network')
    expect(onProvider).toHaveBeenCalledWith('network')
  })
})
