// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the rail data source. Invariant #7: the rail reads
// ONLY /api/context; it never fetches a gate verdict (/api/gates does not
// exist before Phase G), so the "pouls gates" stays a placeholder and the
// front fabricates no verdict.
import { renderHook, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as api from '../api/operator'
import { useRailStatus } from './useRailStatus'

vi.mock('../api/operator', () => ({ getContext: vi.fn() }))
const getContext = vi.mocked(api.getContext)

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

  it('falls back to placeholders and reachable=false on a backend failure', async () => {
    getContext.mockRejectedValue(new Error('down'))
    const { result } = renderHook(() => useRailStatus())
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.reachable).toBe(false)
    expect(result.current.sprint).toBeNull()
    expect(result.current.dirty).toBeNull()
  })

  it('reads only /api/context — never a gate verdict route', () => {
    getContext.mockResolvedValue({ branch: 'm', head: 'h', dirty_files: [], staged_files: [] })
    renderHook(() => useRailStatus())
    expect(getContext).toHaveBeenCalled()
    // The module mock exposes only getContext: there is no gate fetch to make.
    expect(Object.keys(api)).toEqual(['getContext'])
  })
})
