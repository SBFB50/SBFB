// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C → front rapid-add. The ambient rail's data source.
// `GET /api/context` carries sprint, phase, branch and the working-tree
// dirty/staged counts (process.rs context_data). Since Phase G `/api/gates`
// exists and restitutes a per-gate status, the rail also restitutes a COUNT
// per gate status — never an aggregate verdict (the cardinal "0 verdict calculé
// UI" holds: a count of restituted statuses is restitution, not a fabricated
// PASS/score). Context and gates degrade INDEPENDENTLY (Promise.allSettled):
// a gates failure never blanks the rail.
//
// Freshness: the load runs at mount, on tab refocus (visibilitychange/focus),
// and on a manual `refresh()` — so after an in-session commit the counts and
// "N modifiés · N indexés" stop lying (the S80 review P1-1 freshness gap). The
// refocus path coalesces (in-flight guard) so the two events Chrome fires on a
// tab return don't double-fetch; `loading` flips true for the duration so the
// refresh affordance can show its busy state.

import { useCallback, useEffect, useRef, useState } from 'react'
import { getContext, getGates, type GateStatus } from '../api/operator'
import { GATE_STATUS } from '../lib/gateStatus'

/** Count of gates PER restituted status — never collapsed to one verdict. */
export type GateCounts = Record<GateStatus, number>

export interface RailStatus {
  sprint: number | null
  phase: string | null
  branch: string | null
  dirty: number | null
  staged: number | null
  /** Per-status gate counts, or null until/unless `/api/gates` resolves. */
  gateCounts: GateCounts | null
  /** Backend reachability — the only honest live signal the rail has. */
  reachable: boolean
  loading: boolean
}

const EMPTY: RailStatus = {
  sprint: null,
  phase: null,
  branch: null,
  dirty: null,
  staged: null,
  gateCounts: null,
  reachable: false,
  loading: true,
}

/** A zeroed count for every status, built from the single GATE_STATUS source
 * (named-constants invariant — no inline status-key literal list). */
function emptyCounts(): GateCounts {
  return Object.fromEntries(Object.values(GATE_STATUS).map((s) => [s, 0])) as GateCounts
}

function countByStatus(gates: { status: GateStatus }[]): GateCounts {
  const counts = emptyCounts()
  for (const g of gates) counts[g.status] += 1
  return counts
}

export interface RailHandle extends RailStatus {
  /** Manual refetch of the ambient context + gates (after a commit). */
  refresh: () => void
}

export function useRailStatus(): RailHandle {
  const [status, setStatus] = useState<RailStatus>(EMPTY)
  const inFlight = useRef(false)

  const load = useCallback((signal?: AbortSignal) => {
    // Coalesce the visibilitychange + focus pair Chrome fires on a tab return.
    if (inFlight.current) return
    inFlight.current = true
    setStatus((s) => (s.loading ? s : { ...s, loading: true }))
    void Promise.allSettled([getContext(signal), getGates(signal)]).then(([ctxR, gatesR]) => {
      inFlight.current = false
      if (signal?.aborted) return
      if (ctxR.status === 'rejected') {
        setStatus({ ...EMPTY, reachable: false, loading: false })
        return
      }
      const ctx = ctxR.value
      setStatus({
        sprint: typeof ctx.sprint === 'number' ? ctx.sprint : null,
        phase: ctx.phase ?? null,
        branch: ctx.branch ?? null,
        dirty: Array.isArray(ctx.dirty_files) ? ctx.dirty_files.length : null,
        staged: Array.isArray(ctx.staged_files) ? ctx.staged_files.length : null,
        gateCounts: gatesR.status === 'fulfilled' ? countByStatus(gatesR.value.gates) : null,
        reachable: true,
        loading: false,
      })
    })
  }, [])

  useEffect(() => {
    const controller = new AbortController()
    load(controller.signal)
    // Refetch when the operator returns to the tab — cheap, coalesced.
    const onFocus = () => {
      if (document.visibilityState === 'visible') load()
    }
    document.addEventListener('visibilitychange', onFocus)
    window.addEventListener('focus', onFocus)
    return () => {
      controller.abort()
      // Release the in-flight guard on unmount so a StrictMode remount (or a
      // later mount) is never blocked by the aborted load still "in flight".
      inFlight.current = false
      document.removeEventListener('visibilitychange', onFocus)
      window.removeEventListener('focus', onFocus)
    }
  }, [load])

  const refresh = useCallback(() => load(), [load])

  return { ...status, refresh }
}
