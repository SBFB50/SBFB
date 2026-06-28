// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../../api/operator'
import { ProcedeSurface } from './ProcedeSurface'

vi.mock('../../api/operator', () => ({
  getSprintHistory: vi.fn(),
  getStatus: vi.fn(),
  getAllSprints: vi.fn(),
  getCommitDiff: vi.fn(),
  getAudit: vi.fn(),
  getLint: vi.fn(),
  OperatorError: class extends Error {},
}))

const getSprintHistory = vi.mocked(api.getSprintHistory)
const getStatus = vi.mocked(api.getStatus)
const getAllSprints = vi.mocked(api.getAllSprints)
const getCommitDiff = vi.mocked(api.getCommitDiff)
const getAudit = vi.mocked(api.getAudit)
const getLint = vi.mocked(api.getLint)

const HISTORY = {
  sprint: 80,
  status: 'in_progress',
  branch: 'master',
  head: '6991d51',
  entry_tip: 'cf1100b',
  exit_tip: '6991d51',
  roadmap: null,
  total_commits: 5,
  phase_commits: 4,
  chore_commits: 1,
  phases: [
    {
      letter: 'A',
      title: 'auth cookie',
      commit_sha: 'a5ace8d',
      commit_date: '2026-06-27',
      commit_type: 'feat',
      preflight_verdict: 'PLAN-ADAPT',
      review_verdict: 'PASS',
      codex_confirmed: 8,
      codex_partial: 0,
      codex_gap: 0,
      rust_delta: 10,
      vitest_delta: 0,
      files_changed: [{ path: 'crates/sbfb-factory/src/auth.rs', insertions: 40, deletions: 2, status: 'M' }],
      deliverables: ['auth.rs cookie fallback', 'GET /?token bootstrap'],
      findings: [
        { severity: 'P1', code: 'P1-CSRF', description: 'cross-port cookie path CORRIGE', status: 'resolved' },
      ],
    },
  ],
  commits: [
    {
      sha: 'a5ace8d0000000000000000000000000000000000',
      short: 'a5ace8d',
      title: 'feat(factory): Sprint 80 Phase A — auth cookie',
      author: 'veffix',
      date: '2026-06-27T10:00:00+02:00',
      commit_type: 'feat',
      scope: 'factory',
      is_phase: true,
      phase: 'A',
      insertions: 40,
      deletions: 2,
      files: ['crates/sbfb-factory/src/auth.rs'],
      body_sections: [],
    },
  ],
  preflight_bilan: {
    total: 1,
    execute: 0,
    plan_adapt: 1,
    design_conflict: 0,
    phases: [{ phase: 'A', verdict: 'PLAN-ADAPT', file: 'sprint80_phase_a_preflight.md' }],
  },
  tests: { rust_entry: 2000, rust_exit: 2009, rust_delta: 9, vitest_entry: 0, vitest_exit: 52, vitest_delta: 52, size_limit: '6/6', per_phase: [] },
  scope_cuts: [],
  carries_open: [],
  carries_closed: [],
  verification: null,
} as unknown as api.SprintHistory

const STATUS = {
  sprint: 80,
  branch: 'master',
  head: '6991d51',
  current_phase: 'I',
  has_kickoff: true,
  has_plan: true,
  has_design_review: true,
  has_audit_plan: true,
  phases: [],
} as unknown as api.OperatorStatus

beforeEach(() => {
  vi.clearAllMocks()
  getSprintHistory.mockResolvedValue(HISTORY)
  getStatus.mockResolvedValue(STATUS)
  getAllSprints.mockResolvedValue({
    total: 2,
    sprints: [
      { sprint: 80, version: 'v2.1', status: 'in_progress', phase_count: 8, phases_pass: 8, has_verification: false, dir: 'active' },
      { sprint: 79, version: 'v2.1', status: 'closed', phase_count: 9, phases_pass: 9, has_verification: true, dir: 'archive/v2.1' },
    ],
  })
  getCommitDiff.mockResolvedValue({ sha: 'a5ace8d', title: 'auth', files: [] })
  getAudit.mockResolvedValue({ rev: 'a5ace8d', title: 'auth', is_phase_commit: true, ok: true, issues: [] })
  getLint.mockResolvedValue({ ok: true, errors: [], warnings: [] })
})
afterEach(() => vi.restoreAllMocks())

describe('ProcedeSurface (arbre de procédé)', () => {
  it('restitutes the recorded verdicts (never a score) and the frise', async () => {
    render(<ProcedeSurface />)
    await waitFor(() => expect(screen.getByTestId('procede-surface')).toBeInTheDocument())
    expect(screen.getByText('PLAN-ADAPT')).toBeInTheDocument()
    expect(screen.getByText('A')).toBeInTheDocument()
    expect(screen.getByText(/Sprint 80/)).toBeInTheDocument()
    // No verdict is rendered as a numeric score / percentage.
    expect(screen.queryByText(/%/)).toBeNull()
  })

  it('reveals the verdict provenance (U2) and files-changed when a phase is expanded', async () => {
    render(<ProcedeSurface />)
    await waitFor(() => expect(screen.getByTestId('procede-phase')).toBeInTheDocument())
    fireEvent.click(screen.getByTestId('procede-phase'))
    expect(await screen.findByText(/sprint80_phase_a_preflight\.md/)).toBeInTheDocument()
    expect(screen.getByTestId('phase-deliverables')).toHaveTextContent('auth.rs cookie fallback')
    expect(screen.getByTestId('phase-findings')).toHaveTextContent('cross-port cookie path')
    // Files-changed are restituted in the expanded node.
    expect(screen.getByTestId('phase-files')).toHaveTextContent('auth.rs')
  })

  it('restitutes the LIVE current phase from /api/status (où on en est)', async () => {
    render(<ProcedeSurface />)
    const live = await screen.findByTestId('live-process')
    expect(within(live).getByText('I')).toBeInTheDocument()
    expect(within(live).getByText(/pas encore committée/)).toBeInTheDocument()
  })

  it('reveals the commit timeline on demand', async () => {
    const user = userEvent.setup()
    render(<ProcedeSurface />)
    const timeline = await screen.findByTestId('commit-timeline')
    expect(screen.queryByTestId('commit-row')).toBeNull()
    await user.click(within(timeline).getByRole('button'))
    expect(screen.getByTestId('commit-row')).toHaveTextContent('a5ace8d')
  })

  it('filters phases locally', async () => {
    render(<ProcedeSurface />)
    await waitFor(() => expect(screen.getByTestId('procede-phase')).toBeInTheDocument())
    fireEvent.change(screen.getByTestId('phase-filter'), { target: { value: 'zzz' } })
    expect(screen.queryByTestId('procede-phase')).toBeNull()
    expect(screen.getByText(/aucune phase ne correspond/)).toBeInTheDocument()
  })

  it('drills into a past sprint from the cross-sprint index', async () => {
    const user = userEvent.setup()
    render(<ProcedeSurface />)
    await screen.findByTestId('procede-surface')
    await user.click(screen.getByRole('button', { name: /tous les sprints/ }))
    await user.click(await screen.findByText('S79'))
    await waitFor(() => expect(getSprintHistory).toHaveBeenCalledWith(79, expect.anything()))
    // Drilling a non-active sprint replaces the live banner with a back affordance.
    expect(await screen.findByTestId('drill-banner')).toBeInTheDocument()
  })

  it('restitue la table de vérification §1 (résultats par-check, jamais un PASS agrégé)', async () => {
    getSprintHistory.mockResolvedValue({
      ...HISTORY,
      verification: {
        total_checks: 5,
        passed: 4,
        failed: 1,
        checks: [
          { number: 1, name: 'fmt', command: 'cargo fmt --check', result: 'PASS' },
          { number: 2, name: 'clippy', command: 'cargo clippy', result: 'FAIL' },
        ],
      },
    } as unknown as api.SprintHistory)
    render(<ProcedeSurface />)
    const table = await screen.findByTestId('verification-table')
    // RESTITUTED count, not a fabricated sprint verdict.
    expect(within(table).getByText(/4\/5/)).toBeInTheDocument()
    // Per-check names + results are restituted verbatim (PASS for fmt, FAIL for clippy).
    expect(within(table).getByText('fmt')).toBeInTheDocument()
    expect(within(table).getByText('clippy')).toBeInTheDocument()
    expect(within(table).getByText('PASS')).toBeInTheDocument()
    expect(within(table).getByText('FAIL')).toBeInTheDocument()
  })

  it('restitue le registre de dette portée (carries ouverts/fermés)', async () => {
    getSprintHistory.mockResolvedValue({
      ...HISTORY,
      carries_open: [{ code: 'CARRY-1', description: 'dette ouverte', disposition: 'différé S81', phase_closed: null }],
      carries_closed: [{ code: 'CARRY-2', description: 'dette fermée', disposition: '', phase_closed: 'C' }],
    } as unknown as api.SprintHistory)
    render(<ProcedeSurface />)
    const carries = await screen.findByTestId('carries')
    expect(within(carries).getByText(/CARRY-1/)).toBeInTheDocument()
    expect(within(carries).getByText(/différé S81/)).toBeInTheDocument()
    expect(within(carries).getByText(/CARRY-2/)).toBeInTheDocument()
    expect(within(carries).getByText(/phase C/)).toBeInTheDocument()
  })

  it('restitue le bilan tests entrée→sortie quand disponible', async () => {
    render(<ProcedeSurface />)
    const bilan = await screen.findByTestId('tests-bilan')
    expect(within(bilan).getByText(/2000→2009/)).toBeInTheDocument()
    expect(within(bilan).getByText(/\(\+9\)/)).toBeInTheDocument()
  })

  it('bascule le bilan tests sur « au wrap-up » quand entrée/sortie absentes', async () => {
    getSprintHistory.mockResolvedValue({
      ...HISTORY,
      tests: { rust_entry: 0, rust_exit: 0, rust_delta: 0, vitest_entry: 0, vitest_exit: 0, vitest_delta: 0, size_limit: '6/6', per_phase: [] },
    } as unknown as api.SprintHistory)
    render(<ProcedeSurface />)
    const bilan = await screen.findByTestId('tests-bilan')
    expect(within(bilan).getByText(/au wrap-up/)).toBeInTheDocument()
  })

  it('LiveProcessBanner restitue « committée » quand la phase courante a un commit', async () => {
    getStatus.mockResolvedValue({ ...STATUS, current_phase: 'A' } as unknown as api.OperatorStatus)
    render(<ProcedeSurface />)
    const live = await screen.findByTestId('live-process')
    expect(live).toHaveTextContent('committée')
  })

  it('LiveProcessBanner restitue l avancement de la phase courante non committée', async () => {
    getStatus.mockResolvedValue({
      ...STATUS,
      current_phase: 'I',
      phases: [{ letter: 'I', has_preflight: true, has_review: true, review_verdict: null, has_codex: false }],
    } as unknown as api.OperatorStatus)
    render(<ProcedeSurface />)
    const live = await screen.findByTestId('live-process')
    expect(live).toHaveTextContent('préflight→review')
  })

  it('degrades gracefully when /api/status fails (history still renders)', async () => {
    getStatus.mockRejectedValue(new Error('status down'))
    render(<ProcedeSurface />)
    await waitFor(() => expect(screen.getByTestId('procede-surface')).toBeInTheDocument())
    const live = screen.getByTestId('live-process')
    // current phase unknown → em-dash, history still present.
    expect(within(live).getByText('—')).toBeInTheDocument()
    expect(screen.getByText('A')).toBeInTheDocument()
  })
})
