// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, fireEvent } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { VerifyScene } from './VerifyScene'
import type { Operator } from '../../state/useOperator'
import * as verifyData from '../../state/useVerifyData'

vi.mock('../../state/useVerifyData', () => ({ useVerifyData: vi.fn() }))
const useVerifyData = vi.mocked(verifyData.useVerifyData)

function makeOp(over: Partial<Operator> = {}): Operator {
  return {
    launch: vi.fn(),
    hasTurn: false,
    turn: {
      message: null,
      kind: null,
      status: 'idle',
      text: '',
      thinking: '',
      result: null,
      error: null,
      gate: null,
      busy: false,
      launchError: null,
    },
    ...over,
  } as unknown as Operator
}

const DATA = {
  diff: {
    head: 'd59ee32',
    truncated: false,
    unstaged: [
      {
        path: 'crates/sbfb-factory/src/auth.rs',
        insertions: 1,
        deletions: 0,
        hunks: [
          {
            header: '@@ -10,2 +10,3 @@ fn auth',
            lines: [{ kind: 'add' as const, content: 'let ok = true;', old_lineno: null, new_lineno: 11 }],
          },
        ],
      },
    ],
    staged: [],
  },
  gates: [{ gate: 'lint-planning', status: 'passed' as const, issues: [] }],
  diffError: null,
  gatesError: null,
  loading: false,
  reload: vi.fn(),
}

describe('VerifyScene (VERIFY-plein focal scene — Phase H)', () => {
  beforeEach(() => {
    useVerifyData.mockReturnValue(DATA)
  })

  it('renders the diff-viewer + the live gates band', () => {
    render(<VerifyScene op={makeOp()} />)
    expect(screen.getByTestId('verify-scene')).toBeInTheDocument()
    expect(screen.getByTestId('verify-diff')).toBeInTheDocument()
    expect(screen.getByTestId('verify-gates')).toBeInTheDocument()
  })

  it('the ÉTAT slot is a named state and never says a verdict word', () => {
    render(<VerifyScene op={makeOp()} />)
    const etat = screen.getByTestId('verify-etat').textContent ?? ''
    expect(etat).not.toMatch(/\bPASS\b/)
    expect(etat).toMatch(/Examen du diff/) // "inspecting" — changes present
  })

  it('disables the Aperçu scellé / Preuve tabs (à venir S81)', () => {
    render(<VerifyScene op={makeOp()} />)
    expect(screen.getByText('Aperçu scellé')).toBeInTheDocument()
    expect(screen.getByText('Preuve')).toBeInTheDocument()
    expect(screen.getAllByText('à venir').length).toBeGreaterThanOrEqual(2)
  })

  it('shows the unavailable état and the diff error when the diff fails to load', () => {
    useVerifyData.mockReturnValue({ ...DATA, diff: null, diffError: 'VERIFY indisponible (500)' })
    render(<VerifyScene op={makeOp()} />)
    expect(screen.getByTestId('verify-etat').textContent ?? '').toMatch(/indisponible/)
    expect(screen.getAllByText('VERIFY indisponible (500)').length).toBeGreaterThanOrEqual(1)
  })

  it('keeps the diff visible when only the gates fail (independent degradation)', () => {
    useVerifyData.mockReturnValue({ ...DATA, gates: null, gatesError: 'VERIFY indisponible (500)' })
    render(<VerifyScene op={makeOp()} />)
    // the working-tree diff is still rendered (the gates failure does not mask it)
    expect(screen.getByTestId('verify-diff')).toBeInTheDocument()
    // and the gates band restitutes its own error
    expect(screen.getByTestId('verify-gates')).toHaveTextContent('VERIFY indisponible (500)')
  })

  it('routes a hunk correction to the session as an intention', () => {
    const op = makeOp()
    render(<VerifyScene op={op} />)
    fireEvent.click(screen.getByTestId('hunk-intent'))
    expect(op.launch).toHaveBeenCalledWith(expect.stringContaining('auth.rs'), 'phase-review')
  })

  it('restitutes the MUR inline when a routed intention is gated (no execute)', () => {
    const op = makeOp({
      hasTurn: true,
      turn: { ...makeOp().turn, gate: 'Cette intention exige une vraie session agent.' },
    })
    render(<VerifyScene op={op} />)
    expect(screen.getByTestId('verify-intent-gate')).toHaveTextContent('vraie session agent')
  })
})
