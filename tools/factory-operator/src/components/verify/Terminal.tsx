// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the lazy boundary for the live PTY (J12). xterm is
// CODE-SPLIT behind a dynamic import() (preflight adaptation #1/#2): it loads
// only once the operator explicitly STARTS a session — never on a mode switch
// — which keeps the hero `index` chunk under the size-limit gate AND avoids
// spawning the server-side `claude` child just by glancing at VERIFY.
import { lazy, Suspense, useState } from 'react'
import type { TerminalStatus } from './TerminalXterm'
import { AdaptiveSurface } from '../AdaptiveSurface'

const TerminalXterm = lazy(() => import('./TerminalXterm'))

function StatusDot({ status }: { status: TerminalStatus | 'inactif' }) {
  const tone =
    status === 'session active'
      ? 'bg-ok'
      : status === 'erreur de liaison'
        ? 'bg-bad'
        : status === 'session close'
          ? 'bg-warn'
          : 'bg-tx4'
  return <span className={`h-1.5 w-1.5 rounded-full ${tone}`} aria-hidden />
}

export function Terminal() {
  const [started, setStarted] = useState(false)
  const [status, setStatus] = useState<TerminalStatus | 'inactif'>('inactif')

  if (!started) {
    return (
      <AdaptiveSurface kind="terminal" testId="terminal-surface" className="flex flex-1 items-center justify-center p-7">
        <div className="max-w-prose rounded-md border border-dashed border-bd bg-s1 px-6 py-6 text-center">
          <h2 className="mb-1 font-sans text-card font-semibold text-tx">Terminal de vérification</h2>
          <div className="mb-4 font-sans text-body leading-relaxed text-tx2">
            Une session PTY tracée pour inspecter le dépôt à la main (git diff, status, log) en
            complément du visualiseur de diff dédié de la surface VERIFY. La session est enregistrée et
            rejouable depuis l'inspecteur Sessions.
          </div>
          <button
            type="button"
            data-testid="terminal-start"
            onClick={() => setStarted(true)}
            className="rounded-sm bg-tx px-4 py-2 font-sans text-body font-semibold text-s0 hover:bg-tx/90"
          >
            Démarrer la session terminal
          </button>
          <div className="adaptive-secondary mt-3 font-mono text-meta text-tx4">
            commit · push · shell restent derrière le mur — ici, lecture et inspection
          </div>
        </div>
      </AdaptiveSurface>
    )
  }

  return (
    <AdaptiveSurface kind="terminal" testId="terminal-surface" className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-2 border-b border-bd bg-s1 px-4 py-1.5 font-mono text-meta text-tx3">
        <StatusDot status={status} />
        <span>terminal · {status}</span>
        <span className="adaptive-secondary ml-auto text-tx4">enregistré en .cast · rejouable</span>
      </div>
      <div className="min-h-0 flex-1 overflow-hidden bg-s0 p-2">
        <Suspense
          fallback={<div className="p-4 font-mono text-meta text-tx4">chargement du terminal…</div>}
        >
          <TerminalXterm onStatus={setStatus} />
        </Suspense>
      </div>
    </AdaptiveSurface>
  )
}
