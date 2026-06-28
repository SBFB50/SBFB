// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, fireEvent } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { GateEntryView } from '../../api/operator'
import { GatesPanel } from './GatesPanel'

// The gates band restitutes /api/gates 1:1: each entry keeps its DISTINCT
// status, never flattened, never aggregated, never a "PASS" word or a % score.
// A single gate name recurs under two statuses (lint-planning), keyed
// (gate, status). V5/V6 are degraded to S81 (no line anchor in the wire).
const GATES: GateEntryView[] = [
  { gate: 'FG7-preview', status: 'passed', issues: [] },
  { gate: 'FG4-diff', status: 'not_run', issues: [] },
  { gate: 'FG-CSP-authoring', status: 'not_applicable', issues: [] },
  {
    gate: 'lint-planning',
    status: 'blocking',
    issues: [{ message: 'phase body missing a section', file: 'sprint80_phase_h_review.md', line: null }],
  },
  { gate: 'lint-planning', status: 'informational', issues: [{ message: 'stale meta note', file: null, line: null }] },
]

describe('GatesPanel (live gates restitution — fold V4-core)', () => {
  const base = { loading: false, error: null, runRev: 'd59ee32', onReload: vi.fn() }

  it('restitutes each gate with its distinct glyph, keyed (gate, status)', () => {
    render(<GatesPanel gates={GATES} {...base} />)
    expect(screen.getByTestId('verify-gates')).toBeInTheDocument()
    // lint-planning appears TWICE (blocking + informational) — never flattened.
    expect(screen.getAllByText('lint-planning').length).toBeGreaterThanOrEqual(2)
    // the distinct status glyphs are present (✓ is a tick, never "PASS").
    expect(screen.getAllByText('✓').length).toBeGreaterThanOrEqual(1) // passed
    expect(screen.getAllByText('✕').length).toBeGreaterThanOrEqual(1) // blocking
    expect(screen.getAllByText('—').length).toBeGreaterThanOrEqual(1) // not_applicable
    expect(screen.getAllByText('•').length).toBeGreaterThanOrEqual(1) // not_run / informational
  })

  it('never renders a verdict word or an aggregated score', () => {
    render(<GatesPanel gates={GATES} {...base} />)
    const root = screen.getByTestId('verify-gates')
    expect(root.textContent ?? '').not.toMatch(/\bPASS\b/)
    expect(root.textContent ?? '').not.toMatch(/%/)
  })

  it('expands a tray with each gate issue (message + .planning file)', () => {
    render(<GatesPanel gates={GATES} {...base} />)
    expect(screen.queryByTestId('verify-gates-tray')).toBeNull()
    fireEvent.click(screen.getByTestId('gates-tray-toggle'))
    expect(screen.getByTestId('verify-gates-tray')).toBeInTheDocument()
    expect(screen.getByText(/phase body missing a section/)).toBeInTheDocument()
    expect(screen.getByText(/sprint80_phase_h_review\.md/)).toBeInTheDocument()
  })

  it('restitutes the run@rev of the displayed diff (no fabricated freshness verdict)', () => {
    render(<GatesPanel gates={GATES} {...base} />)
    const root = screen.getByTestId('verify-gates')
    expect(screen.getByText(/run@d59ee32/)).toBeInTheDocument()
    // S80 ships no lying "obsolète" badge (review P1-1 → carry S81).
    expect(root.textContent ?? '').not.toMatch(/obsolète/)
  })

  it('relancer triggers a reload', () => {
    const onReload = vi.fn()
    render(<GatesPanel gates={GATES} {...base} onReload={onReload} />)
    fireEvent.click(screen.getByTestId('gates-reload'))
    expect(onReload).toHaveBeenCalledOnce()
  })

  it('shows honest loading / error states', () => {
    const { rerender } = render(<GatesPanel gates={null} {...base} loading error={null} />)
    expect(screen.getByText(/lecture des gates/)).toBeInTheDocument()
    rerender(<GatesPanel gates={null} {...base} loading={false} error="VERIFY indisponible (500)" />)
    expect(screen.getByText('VERIFY indisponible (500)')).toBeInTheDocument()
  })
})
