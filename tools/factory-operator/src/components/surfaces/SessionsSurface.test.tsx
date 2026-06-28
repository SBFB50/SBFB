// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../../api/operator'
import { SessionsSurface } from './SessionsSurface'
import { formatSessionDate } from '../../lib/sessionDate'

vi.mock('../../api/operator', () => ({
  getActionLog: vi.fn(),
  getTerminalSessions: vi.fn(),
  getChatLog: vi.fn(),
  getTerminalCast: vi.fn().mockResolvedValue(''),
  OperatorError: class extends Error {},
}))

// Stub the xterm-mounting children so the surface logic (Set multi-cast,
// resume panel toggle) is testable without a real DOM/canvas terminal.
vi.mock('./CastReplay', () => ({ CastReplay: ({ name }: { name: string }) => <div data-testid="cast-replay">{name}</div> }))
vi.mock('../verify/TerminalXterm', () => ({ default: () => <div data-testid="resume-xterm" /> }))

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

  it('ouvre plusieurs rejeux .cast à la fois (Set) et en referme un isolément', async () => {
    const user = userEvent.setup()
    getTerminalSessions.mockResolvedValue({
      sessions: [
        { name: 'a.cast', path: 'p/a.cast', size_bytes: 2048 },
        { name: 'b.cast', path: 'p/b.cast', size_bytes: 4096 },
      ],
      claude_sessions: [],
    })
    render(<SessionsSurface sessionId={null} />)
    await user.click(await screen.findByRole('button', { name: /a\.cast/ }))
    await user.click(screen.getByRole('button', { name: /b\.cast/ }))
    expect(screen.getAllByTestId('cast-replay')).toHaveLength(2)
    // Close one → the other stays mounted (toggleCast remove isolé).
    await user.click(screen.getAllByRole('button', { name: 'fermer' })[0])
    expect(screen.getAllByTestId('cast-replay')).toHaveLength(1)
  })

  it('reprend une session CLI : monte le panneau PTY et bascule le label', async () => {
    const user = userEvent.setup()
    getTerminalSessions.mockResolvedValue({
      sessions: [],
      claude_sessions: [{ session_id: 'sess-1', name: 'front', updated_at: 1782600000000 }],
    })
    render(<SessionsSurface sessionId={null} />)
    const resume = await screen.findByTestId('claude-resume')
    expect(resume).toHaveTextContent('reprendre')
    await user.click(resume)
    expect(await screen.findByTestId('resume-terminal')).toBeInTheDocument()
    expect(resume).toHaveTextContent('fermer')
    await user.click(resume)
    expect(screen.queryByTestId('resume-terminal')).toBeNull()
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

describe('formatSessionDate', () => {
  it('rend une date pour un epoch en ms (>1e12) et en secondes', () => {
    expect(formatSessionDate(1782600000000)).not.toBe('')
    expect(formatSessionDate(1782600000)).not.toBe('')
  })
  it('rend une chaîne vide pour 0 / NaN (garde)', () => {
    expect(formatSessionDate(0)).toBe('')
    expect(formatSessionDate(Number.NaN)).toBe('')
  })
})
