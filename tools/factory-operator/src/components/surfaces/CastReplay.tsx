// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — lazy boundary for the `.cast` replay (U6/V9). Fetches
// the raw recording text on demand, then mounts the write-only <CastXterm>
// (the xterm chunk is shared with the live terminal, async). The replay is
// the RAW recorded output, never re-diffed or summarised.
import { lazy, Suspense, useEffect, useState } from 'react'
import { getTerminalCast, OperatorError } from '../../api/operator'

const CastXterm = lazy(() => import('./CastXterm'))

interface Resolved {
  name: string
  raw: string | null
  error: string | null
}

export function CastReplay({ name }: { name: string }) {
  // State written ONLY from the async fetch; loading derived by name (react-hooks).
  const [resolved, setResolved] = useState<Resolved | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    getTerminalCast(name, controller.signal)
      .then((text) => {
        if (!controller.signal.aborted) setResolved({ name, raw: text, error: null })
      })
      .catch((err) => {
        if (controller.signal.aborted) return
        setResolved({
          name,
          raw: null,
          error: err instanceof OperatorError ? `enregistrement indisponible (${err.status})` : 'enregistrement indisponible',
        })
      })
    return () => controller.abort()
  }, [name])

  const ready = resolved !== null && resolved.name === name
  const raw = ready ? resolved.raw : null
  const error = ready ? resolved.error : null

  if (error) return <div className="p-4 font-mono text-[11px] text-warn">{error}</div>
  if (raw === null) return <div className="p-4 font-mono text-[11px] text-tx4">chargement de l'enregistrement…</div>

  return (
    <div data-testid="cast-replay" className="min-h-0 flex-1 overflow-hidden bg-s0 p-2">
      <Suspense fallback={<div className="p-4 font-mono text-[11px] text-tx4">rendu…</div>}>
        <CastXterm raw={raw} />
      </Suspense>
    </div>
  )
}
