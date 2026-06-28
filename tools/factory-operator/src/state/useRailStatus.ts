// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the ambient rail's data source. `GET /api/context`
// carries every field the altitude-0 orientation bar needs in ONE call:
// sprint, phase, branch, and the working-tree dirty/staged counts
// (operator_server.rs handle_context / process.rs context_data). The
// "pouls gates" is deliberately NOT fetched here: `/api/gates` does not
// exist before Phase G, and the front never fabricates a verdict
// (plan-adaptation #7 + the cardinal "0 verdict calculé UI" invariant).

import { useEffect, useState } from 'react'
import { getContext } from '../api/operator'

export interface RailStatus {
  sprint: number | null
  phase: string | null
  branch: string | null
  dirty: number | null
  staged: number | null
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
  reachable: false,
  loading: true,
}

export function useRailStatus(): RailStatus {
  const [status, setStatus] = useState<RailStatus>(EMPTY)

  useEffect(() => {
    const controller = new AbortController()
    getContext(controller.signal)
      .then((ctx) => {
        setStatus({
          sprint: typeof ctx.sprint === 'number' ? ctx.sprint : null,
          phase: ctx.phase ?? null,
          branch: ctx.branch ?? null,
          dirty: Array.isArray(ctx.dirty_files) ? ctx.dirty_files.length : null,
          staged: Array.isArray(ctx.staged_files) ? ctx.staged_files.length : null,
          reachable: true,
          loading: false,
        })
      })
      .catch(() => {
        // An abort (unmount / StrictMode cleanup) is not a real failure.
        if (controller.signal.aborted) return
        setStatus({ ...EMPTY, reachable: false, loading: false })
      })
    return () => controller.abort()
  }, [])

  return status
}
