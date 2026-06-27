// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../../api/operator'
import { ConformiteCard } from './ConformiteCard'

vi.mock('../../api/operator', () => ({
  getAudit: vi.fn(),
  getLint: vi.fn(),
  OperatorError: class extends Error {},
}))

const getAudit = vi.mocked(api.getAudit)
const getLint = vi.mocked(api.getLint)

beforeEach(() => {
  vi.clearAllMocks()
  getLint.mockResolvedValue({ ok: true, errors: [], warnings: [] })
})
afterEach(() => vi.restoreAllMocks())

describe('ConformiteCard (U3/A9/V10 — manques, jamais une coche)', () => {
  it('lists the missing things as "manques", never a green tick', async () => {
    getAudit.mockResolvedValue({
      rev: 'deadbee',
      title: 'phase',
      is_phase_commit: true,
      ok: false,
      issues: ['missing review file'],
    })
    render(<ConformiteCard rev="deadbee" />)
    await waitFor(() => expect(screen.getByTestId('conformite-card')).toBeInTheDocument())

    expect(screen.getByText('missing review file')).toBeInTheDocument()
    expect(screen.getByText(/manques relevés/)).toBeInTheDocument()
    // The card never renders an approval tick — it restitutes gaps, not a PASS.
    expect(screen.queryByText('✓')).toBeNull()
  })

  it('shows an honest empty state instead of an approval', async () => {
    getAudit.mockResolvedValue({ rev: 'deadbee', title: 'phase', is_phase_commit: true, ok: true, issues: [] })
    render(<ConformiteCard rev="deadbee" />)
    expect(await screen.findByText(/0 manque relevé/)).toBeInTheDocument()
  })
})
