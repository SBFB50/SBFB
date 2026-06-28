// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C → front rapid-add. The rail reads /api/context AND (since
// Phase G) /api/gates. The cardinal invariant holds: it restitutes a COUNT per
// gate status, never an aggregate verdict. Context + gates degrade
// INDEPENDENTLY (allSettled) and a manual refresh re-reads both.
import { act, renderHook, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as api from '../api/operator'
import { useRailStatus } from './useRailStatus'

vi.mock('../api/operator', () => ({ getContext: vi.fn(), getGates: vi.fn() }))
const getContext = vi.mocked(api.getContext)
const getGates = vi.mocked(api.getGates)

function gate(name: string, status: api.GateStatus): api.GateEntryView {
  return { gate: name, status, issues: [] }
}

afterEach(() => vi.restoreAllMocks())

describe('useRailStatus', () => {
  it('maps /api/context into the rail fields and marks the backend reachable', async () => {
    getContext.mockResolvedValue({
      branch: 'master',
      head: 'abc1234',
      sprint: 80,
      phase: 'Phase C',
      dirty_files: ['a', 'b', 'c'],
      staged_files: ['d'],
    })
    getGates.mockResolvedValue({ gates: [] })
    const { result } = renderHook(() => useRailStatus())
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current).toMatchObject({
      sprint: 80,
      phase: 'Phase C',
      branch: 'master',
      dirty: 3,
      staged: 1,
      reachable: true,
    })
  })

  it('falls back to placeholders and reachable=false on a context failure', async () => {
    getContext.mockRejectedValue(new Error('down'))
    getGates.mockResolvedValue({ gates: [] })
    const { result } = renderHook(() => useRailStatus())
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.reachable).toBe(false)
    expect(result.current.sprint).toBeNull()
    expect(result.current.dirty).toBeNull()
    expect(result.current.gateCounts).toBeNull()
  })

  it('restitutes a COUNT per gate status, never an aggregate verdict', async () => {
    getContext.mockResolvedValue({ branch: 'm', head: 'h', dirty_files: [], staged_files: [] })
    getGates.mockResolvedValue({
      gates: [gate('lint', 'passed'), gate('csp', 'passed'), gate('plan', 'blocking'), gate('e2e', 'not_run')],
    })
    const { result } = renderHook(() => useRailStatus())
    await waitFor(() => expect(result.current.gateCounts).not.toBeNull())
    expect(result.current.gateCounts).toEqual({
      passed: 2,
      blocking: 1,
      not_run: 1,
      not_applicable: 0,
      informational: 0,
    })
  })

  it('degrades gates independently: a /api/gates failure never blanks the context', async () => {
    getContext.mockResolvedValue({ branch: 'm', head: 'h', sprint: 80, dirty_files: [], staged_files: [] })
    getGates.mockRejectedValue(new Error('gates down'))
    const { result } = renderHook(() => useRailStatus())
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.reachable).toBe(true)
    expect(result.current.sprint).toBe(80)
    expect(result.current.gateCounts).toBeNull()
  })

  it('refresh re-reads context + gates', async () => {
    getContext.mockResolvedValue({ branch: 'm', head: 'h', dirty_files: [], staged_files: [] })
    getGates.mockResolvedValue({ gates: [] })
    const { result } = renderHook(() => useRailStatus())
    await waitFor(() => expect(result.current.loading).toBe(false))
    const before = getContext.mock.calls.length
    act(() => result.current.refresh())
    await waitFor(() => expect(getContext.mock.calls.length).toBeGreaterThan(before))
    expect(getGates.mock.calls.length).toBeGreaterThan(0)
  })
})
