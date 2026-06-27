// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { Mur } from './Mur'

describe('Mur (requires_gate restitution)', () => {
  it('restitutes the barrier message and the no-bypass invariant', () => {
    render(<Mur message="Cette intention exige une vraie session agent." onBack={() => {}} />)
    expect(screen.getByTestId('mur')).toBeInTheDocument()
    expect(screen.getByRole('alert')).toBeInTheDocument()
    // The invariant line is the whole point of the MUR.
    expect(screen.getByText(/aucun « Forcer »/)).toBeInTheDocument()
  })

  it('offers ZERO Forcer / Override / Bypass affordance — only going back', () => {
    render(<Mur message="m" onBack={() => {}} />)
    const buttons = screen.getAllByRole('button')
    expect(buttons).toHaveLength(1)
    expect(buttons[0]).toHaveTextContent(/Retour/)
    for (const b of buttons) {
      expect(b.textContent ?? '').not.toMatch(/Forcer|Override|Bypass|Exécuter/)
    }
  })

  it('calls onBack from the only control', async () => {
    const onBack = vi.fn()
    render(<Mur message="m" onBack={onBack} />)
    await userEvent.click(screen.getByTestId('mur-back'))
    expect(onBack).toHaveBeenCalledOnce()
  })
})
