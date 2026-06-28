// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the host frame for a secondary inspector opened from the
// rail (procédé / sessions / knowledge). It replaces the focal scene body
// while open and always offers "← retour" to the focal mode. The bi-focal
// STEER/VERIFY scene is the primary work; these are auxiliary read surfaces
// that surface the gisement already computed by Rust (sprint_history.rs,
// /api/actions/log, /api/context-pack) — restituted, never recomputed.
//
// Each inspector is CODE-SPLIT (lazy): they are reached only on demand, so
// their bodies (diff-viewer, conformity card, context-pack, cast replay) stay
// OUT of the 40 kB hero chunk (size-limit `app`, Day-0 D4/D5) and load when an
// inspector is first opened.
import { lazy, Suspense } from 'react'
import type { Operator } from '../../state/useOperator'
import { SECONDARY_SURFACES } from '../../catalog/surfaces'

const ProcedeSurface = lazy(() => import('./ProcedeSurface').then((m) => ({ default: m.ProcedeSurface })))
const SessionsSurface = lazy(() => import('./SessionsSurface').then((m) => ({ default: m.SessionsSurface })))
const KnowledgeSurface = lazy(() => import('./KnowledgeSurface').then((m) => ({ default: m.KnowledgeSurface })))

export function SurfaceHost({ op }: { op: Operator }) {
  const surface = op.surface
  if (surface === null) return null
  const def = SECONDARY_SURFACES.find((s) => s.id === surface)

  return (
    <div data-testid="surface-host" className="flex min-h-0 flex-1 flex-col bg-s0">
      <div className="flex items-center gap-2.5 border-b border-bd px-5 py-3">
        <button
          type="button"
          data-testid="surface-back"
          onClick={op.closeSurface}
          title={`retour à ${op.mode === 'steer' ? 'STEER' : 'VERIFY'}`}
          className="rounded-sm border border-bd px-2 py-1 font-mono text-meta text-tx3 hover:border-bd2 hover:text-tx2"
        >
          ← {op.mode === 'steer' ? 'STEER' : 'VERIFY'}
        </button>
        <span className="font-mono text-meta text-tx4" aria-hidden>
          {def?.glyph}
        </span>
        <h1 className="font-sans text-card font-semibold text-tx">{def?.label}</h1>
        <span className="font-sans text-sec text-tx3">— {def?.hint}</span>
      </div>

      <div className="min-h-0 flex-1 overflow-hidden">
        <Suspense fallback={<div className="p-5 font-mono text-meta text-tx4">ouverture de l'inspecteur…</div>}>
          {surface === 'procede' ? (
            <ProcedeSurface />
          ) : surface === 'sessions' ? (
            <SessionsSurface sessionId={op.sessionId} />
          ) : (
            <KnowledgeSurface sessionId={op.sessionId} />
          )}
        </Suspense>
      </div>
    </div>
  )
}
