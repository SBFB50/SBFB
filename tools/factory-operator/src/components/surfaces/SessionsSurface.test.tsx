// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../../api/operator'
import { SessionsSurface } from './SessionsSurface'

vi.mock('../../api/operator', () => ({
  getActionLog: vi.fn(),
  getTerminalSessions: vi.fn(),
  getChatLog: vi.fn(),
  getTerminalCast: vi.fn().mockResolvedValue(''),
  OperatorError: class extends Error {},
}))

const getActionLog = vi.mocked(api.getActionLog)
const getTerminalSessions = vi.mocked(api.getTerminalSessions)

beforeEach(() => {
  vi.clearAllMocks()
  getActionLog.mockResolvedValue([
    { timestamp: '2026-06-27T20:01:02Z', action: 'status-sprint', args: {}, result: 'ok' },
    {
      timestamp: '2026-06-27T20:03:04Z',
      action: 'rm -rf',
      args: {},
      result: 'rejected: not in allowlist',
    },
  ])
  getTerminalSessions.mockResolvedValue({ sessions: [], claude_sessions: [] })
})
afterEach(() => vi.restoreAllMocks())

describe('SessionsSurface (journal + registre des refus du mur)', () => {
  it('renders the MUR refusal with its reason (S8/U5), never a retry path', async () => {
    render(<SessionsSurface sessionId={null} />)
    const journal = await waitFor(() => screen.getByTestId('action-journal'))

    // The allowlisted action is logged plainly…
    expect(journal).toHaveTextContent('status-sprint')
    // …and the refusal carries its reason (the register is evidence — there is
    // no "réessayer en forçant" affordance anywhere in the surface).
    expect(journal).toHaveTextContent('rejected: not in allowlist')
    expect(journal.textContent).toContain('⛔')
    expect(screen.queryByText(/forcer|réessayer en forçant/i)).toBeNull()
  })
})
