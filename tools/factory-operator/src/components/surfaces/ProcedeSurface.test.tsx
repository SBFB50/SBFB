// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../../api/operator'
import { ProcedeSurface } from './ProcedeSurface'

vi.mock('../../api/operator', () => ({
  getSprintHistory: vi.fn(),
  getCommitDiff: vi.fn(),
  getAudit: vi.fn(),
  getLint: vi.fn(),
  OperatorError: class extends Error {},
}))

const getSprintHistory = vi.mocked(api.getSprintHistory)
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
      files_changed: [],
      deliverables: ['auth.rs cookie fallback', 'GET /?token bootstrap'],
      findings: [
        { severity: 'P1', code: 'P1-CSRF', description: 'cross-port cookie path CORRIGE', status: 'resolved' },
      ],
    },
  ],
  preflight_bilan: {
    total: 1,
    execute: 0,
    plan_adapt: 1,
    design_conflict: 0,
    phases: [{ phase: 'A', verdict: 'PLAN-ADAPT', file: 'sprint80_phase_a_preflight.md' }],
  },
  tests: { rust_entry: 2000, rust_exit: 2009, vitest_entry: 0, vitest_exit: 52, per_phase: [] },
  scope_cuts: [],
  carries_open: [],
  carries_closed: [],
} as unknown as api.SprintHistory

beforeEach(() => {
  vi.clearAllMocks()
  getSprintHistory.mockResolvedValue(HISTORY)
  getCommitDiff.mockResolvedValue({ sha: 'a5ace8d', title: 'auth', files: [] })
  getAudit.mockResolvedValue({ rev: 'a5ace8d', title: 'auth', is_phase_commit: true, ok: true, issues: [] })
  getLint.mockResolvedValue({ ok: true, errors: [], warnings: [] })
})
afterEach(() => vi.restoreAllMocks())

describe('ProcedeSurface (arbre de procédé)', () => {
  it('restitutes the recorded verdicts (never a score) and the frise', async () => {
    render(<ProcedeSurface />)
    await waitFor(() => expect(screen.getByTestId('procede-surface')).toBeInTheDocument())

    // The phase is rendered with its RESTITUTED verdicts — read off the
    // artifacts by Rust, not computed/scored by the UI.
    expect(screen.getByText('PLAN-ADAPT')).toBeInTheDocument()
    expect(screen.getByText('A')).toBeInTheDocument()
    expect(screen.getByText(/Sprint 80/)).toBeInTheDocument()

    // No verdict is rendered as a numeric score / percentage.
    expect(screen.queryByText(/%/)).toBeNull()
  })

  it('reveals the verdict provenance (U2) when a phase is expanded', async () => {
    render(<ProcedeSurface />)
    await waitFor(() => expect(screen.getByTestId('procede-phase')).toBeInTheDocument())

    fireEvent.click(screen.getByTestId('procede-phase'))

    // U2: the preflight verdict carries its SOURCE artifact filename.
    expect(await screen.findByText(/sprint80_phase_a_preflight\.md/)).toBeInTheDocument()
    // A1/U1: the phase node surfaces the restituted deliverables AND review
    // findings from PhaseHistory (the "→ artefact" leaf), never recomputed.
    expect(screen.getByTestId('phase-deliverables')).toHaveTextContent('auth.rs cookie fallback')
    expect(screen.getByTestId('phase-findings')).toHaveTextContent('cross-port cookie path')
  })
})
