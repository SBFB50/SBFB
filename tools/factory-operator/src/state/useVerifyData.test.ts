// SPDX-License-Identifier: AGPL-3.0-or-later
import { act, renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '../api/operator'
import { useVerifyData } from './useVerifyData'

// The co-fetch hook transports the working-tree diff + the live gates, degrading
// INDEPENDENTLY (a gates failure never hides the diff). The fetches are mocked.
vi.mock('../api/operator', () => ({
  getWorkingTreeDiff: vi.fn(),
  getGates: vi.fn(),
  OperatorError: class extends Error {
    status: number
    constructor(status: number) {
      super('operator error')
      this.status = status
    }
  },
}))

const getWorkingTreeDiff = vi.mocked(api.getWorkingTreeDiff)
const getGates = vi.mocked(api.getGates)

const DIFF = { head: 'd59ee32', unstaged: [], staged: [], truncated: false }
const GATES = { gates: [{ gate: 'lint-planning', status: 'passed' as const, issues: [] }] }

beforeEach(() => {
  vi.clearAllMocks()
  getWorkingTreeDiff.mockResolvedValue(DIFF)
  getGates.mockResolvedValue(GATES)
})
afterEach(() => vi.restoreAllMocks())

describe('useVerifyData (co-fetch diff + gates, independent degradation)', () => {
  it('co-fetches the diff and the gates (happy path)', async () => {
    const { result } = renderHook(() => useVerifyData())
    expect(result.current.loading).toBe(true)
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.diff).toEqual(DIFF)
    expect(result.current.gates).toEqual(GATES.gates)
    expect(result.current.diffError).toBeNull()
    expect(result.current.gatesError).toBeNull()
  })

  it('degrades independently: a gates 500 keeps the working-tree diff', async () => {
    getGates.mockRejectedValue(new api.OperatorError(500, '/api/gates') as Error)
    const { result } = renderHook(() => useVerifyData())
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.diff).toEqual(DIFF) // diff still present — never masked
    expect(result.current.gates).toBeNull()
    expect(result.current.gatesError).toBe('VERIFY indisponible (500)')
    expect(result.current.diffError).toBeNull()
  })

  it('maps an OperatorError to a status message and a generic error otherwise', async () => {
    getWorkingTreeDiff.mockRejectedValue(new Error('network down'))
    const { result } = renderHook(() => useVerifyData())
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.diffError).toBe('VERIFY indisponible')
    expect(result.current.gates).toEqual(GATES.gates) // gates unaffected
  })

  it('reload re-fetches both', async () => {
    const { result } = renderHook(() => useVerifyData())
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(getWorkingTreeDiff).toHaveBeenCalledTimes(1)
    act(() => result.current.reload())
    expect(result.current.loading).toBe(true) // back to loading on reload
    await waitFor(() => expect(getWorkingTreeDiff).toHaveBeenCalledTimes(2))
    expect(getGates).toHaveBeenCalledTimes(2)
  })
})
