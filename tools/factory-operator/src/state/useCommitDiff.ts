// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — fetch a past commit's diff (J11) for the arbre de
// procédé. The hunks are computed in Rust (single source of truth); this hook
// only transports them, abort-clean on unmount / sha change. State is written
// ONLY from the async resolution (never synchronously in the effect body —
// react-hooks/set-state-in-effect); `loading` is DERIVED by comparing the
// resolved sha to the requested one (the TechDetails pattern).
import { useEffect, useState } from 'react'
import { getCommitDiff, OperatorError, type CommitDiff } from '../api/operator'

export interface CommitDiffState {
  diff: CommitDiff | null
  error: string | null
  loading: boolean
}

interface Resolved {
  sha: string
  diff: CommitDiff | null
  error: string | null
}

export function useCommitDiff(sha: string | null): CommitDiffState {
  const [resolved, setResolved] = useState<Resolved | null>(null)

  useEffect(() => {
    if (!sha) return
    const controller = new AbortController()
    getCommitDiff(sha, controller.signal)
      .then((diff) => {
        if (!controller.signal.aborted) setResolved({ sha, diff, error: null })
      })
      .catch((err) => {
        if (controller.signal.aborted) return
        setResolved({
          sha,
          diff: null,
          error: err instanceof OperatorError ? `diff indisponible (${err.status})` : 'diff indisponible',
        })
      })
    return () => controller.abort()
  }, [sha])

  const ready = resolved !== null && resolved.sha === sha
  return {
    diff: ready ? resolved.diff : null,
    error: ready ? resolved.error : null,
    loading: sha !== null && !ready,
  }
}
