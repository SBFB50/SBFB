// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
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
const getChatLog = vi.mocked(api.getChatLog)

beforeEach(() => {
  vi.clearAllMocks()
  getActionLog.mockResolvedValue([
    { timestamp: '2026-06-27T20:01:02Z', action: 'status-sprint', args: {}, result: 'ok' },
    { timestamp: '2026-06-27T20:03:04Z', action: 'rm -rf', args: {}, result: 'rejected: not in allowlist' },
  ])
  getTerminalSessions.mockResolvedValue({ sessions: [], claude_sessions: [] })
})
afterEach(() => vi.restoreAllMocks())

describe('SessionsSurface (journal + registre des refus du mur)', () => {
  it('renders the MUR refusal with its reason (S8/U5), never a retry path', async () => {
    render(<SessionsSurface sessionId={null} />)
    const journal = await waitFor(() => screen.getByTestId('action-journal'))
    expect(journal).toHaveTextContent('status-sprint')
    expect(journal).toHaveTextContent('rejected: not in allowlist')
    expect(journal.textContent).toContain('⛔')
    expect(screen.queryByText(/forcer|réessayer en forçant/i)).toBeNull()
  })

  it('restitutes resumable claude sessions with a "reprendre" affordance', async () => {
    getTerminalSessions.mockResolvedValue({
      sessions: [],
      claude_sessions: [{ session_id: 'sess-abc123', name: 'sprint 80 front', updated_at: 1782600000000 }],
    })
    render(<SessionsSurface sessionId={null} />)
    const list = await waitFor(() => screen.getByTestId('claude-sessions'))
    expect(list).toHaveTextContent('sprint 80 front')
    expect(screen.getByTestId('claude-resume')).toHaveTextContent('reprendre')
  })

  it('shows "aucune session reprenable" when none are scoped to the repo', async () => {
    render(<SessionsSurface sessionId={null} />)
    expect(await screen.findByText(/aucune session claude reprenable/)).toBeInTheDocument()
  })

  it('expands a clamped chat message on demand', async () => {
    const user = userEvent.setup()
    getChatLog.mockResolvedValue({
      id: 's1',
      context_pack: {},
      messages: [{ role: 'user', content: 'x'.repeat(300) }],
    })
    render(<SessionsSurface sessionId="s1" />)
    const expand = await screen.findByTestId('message-expand')
    expect(expand).toHaveTextContent('voir plus')
    await user.click(expand)
    expect(expand).toHaveTextContent('voir moins')
  })
})
