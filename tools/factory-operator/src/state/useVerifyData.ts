// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase H — the VERIFY-plein data source. Co-fetches the working-tree
// diff (`/api/git/diff`) AND the live gates (`/api/gates`) in ONE cycle so the
// gates band can be stamped with the diff's `head` (`run@<rev>`). The two fetch
// INDEPENDENTLY (`Promise.allSettled`, review P2): a `/api/gates` 500 must not
// hide the working-tree diff (the central, default surface), and vice versa —
// each surface restitutes its own error. Abort-clean on unmount; `reload`
// re-fetches both (the "relancer" affordance). State is written ONLY from the
// async resolution (never synchronously in the effect — react-hooks/
// set-state-in-effect); `loading` is DERIVED (state === null), the
// useCommitDiff / TechDetails pattern.
import { useEffect, useState } from 'react'
import {
  getGates,
  getWorkingTreeDiff,
  OperatorError,
  type GateEntryView,
  type WorkingTreeDiff,
} from '../api/operator'

export interface VerifyData {
  diff: WorkingTreeDiff | null
  gates: GateEntryView[] | null
  /** The working-tree diff failed to load (the gates may still be present). */
  diffError: string | null
  /** The gates failed to load (the diff may still be present). */
  gatesError: string | null
  loading: boolean
  reload: () => void
}

interface Resolved {
  diff: WorkingTreeDiff | null
  gates: GateEntryView[] | null
  diffError: string | null
  gatesError: string | null
}

function mapError(reason: unknown): string {
  return reason instanceof OperatorError ? `VERIFY indisponible (${reason.status})` : 'VERIFY indisponible'
}

export function useVerifyData(): VerifyData {
  const [resolved, setResolved] = useState<Resolved | null>(null)
  const [tick, setTick] = useState(0)

  useEffect(() => {
    const controller = new AbortController()
    const { signal } = controller
    void (async () => {
      // allSettled: independent degradation — a gates failure never rejects the
      // diff (and vice versa). On abort both reject with AbortError; the
      // `signal.aborted` guard skips the write, so an abort is never an error.
      const [diffRes, gatesRes] = await Promise.allSettled([
        getWorkingTreeDiff(signal),
        getGates(signal),
      ])
      if (signal.aborted) return
      setResolved({
        diff: diffRes.status === 'fulfilled' ? diffRes.value : null,
        diffError: diffRes.status === 'rejected' ? mapError(diffRes.reason) : null,
        gates: gatesRes.status === 'fulfilled' ? gatesRes.value.gates : null,
        gatesError: gatesRes.status === 'rejected' ? mapError(gatesRes.reason) : null,
      })
    })()
    return () => controller.abort()
  }, [tick])

  return {
    diff: resolved?.diff ?? null,
    gates: resolved?.gates ?? null,
    diffError: resolved?.diffError ?? null,
    gatesError: resolved?.gatesError ?? null,
    loading: resolved === null,
    reload: () => {
      setResolved(null)
      setTick((t) => t + 1)
    },
  }
}
