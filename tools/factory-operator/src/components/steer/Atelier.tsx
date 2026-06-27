// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the observable atelier: the live transcript of the
// current turn (accumulated SSE deltas, then the Done result), the honest
// turn status, and the turn controls. "Interrompre" stops LISTENING (S6a,
// honest abort — the server turn may continue). "Relancer" is a NEW
// full-cost turn, never an idempotent no-op (plan-adaptation #1) — the
// label and title say so. No status word is ever a verdict (PASS / Vérifié
// / Approuvé) — the front restitutes, it never asserts.

import type { OperatorTurn } from '../../state/useOperator'

function statusLabel(status: OperatorTurn['status'], busy: boolean): { dot: string; text: string } {
  if (busy) return { dot: 'bg-info', text: 'préparation du tour…' }
  switch (status) {
    case 'streaming':
      return { dot: 'bg-ok', text: "l'agent travaille" }
    case 'done':
      return { dot: 'bg-ok', text: 'terminé · prêt à examiner' }
    case 'aborted':
      return { dot: 'bg-warn', text: "écoute arrêtée — le tour serveur peut continuer" }
    case 'ended':
      return { dot: 'bg-warn', text: 'flux clos — aucun résultat final' }
    case 'error':
      return { dot: 'bg-bad', text: 'interrompu — erreur de flux' }
    case 'gate':
      return { dot: 'bg-mur', text: 'intention sous barrière' }
    default:
      return { dot: 'bg-tx4', text: 'prêt' }
  }
}

export function Atelier({
  turn,
  onRelaunch,
  onInterrupt,
  onNewSession,
}: {
  turn: OperatorTurn
  onRelaunch: () => void
  onInterrupt: () => void
  onNewSession: () => void
}) {
  const status = statusLabel(turn.status, turn.busy)
  const streaming = turn.status === 'streaming' || turn.busy
  // Network arm: the Done carries the full result with zero deltas; show it.
  const body = turn.status === 'done' ? turn.result ?? turn.text : turn.text

  return (
    <section
      data-testid="atelier"
      aria-label="Atelier observable"
      className="flex min-h-0 flex-1 flex-col gap-3 bg-s0 p-5"
    >
      <div className="flex items-center gap-2.5">
        <span className={`h-1.5 w-1.5 rounded-full ${status.dot}`} aria-hidden />
        <span className="font-sans text-[9px] font-semibold uppercase tracking-[0.15em] text-tx4">
          atelier observable
        </span>
        <span className="ml-auto font-mono text-[10px] text-tx3" data-testid="turn-status">
          {status.text}
        </span>
      </div>

      {turn.message ? (
        <div className="rounded-md border border-bd2 bg-s1 px-3.5 py-2.5 font-sans text-[12.5px] leading-snug text-tx">
          {turn.message}
        </div>
      ) : null}

      <div className="min-h-0 flex-1 overflow-auto rounded-md border border-bd bg-s1 px-3.5 py-3 font-mono text-[11px] leading-relaxed text-tx2">
        {turn.thinking ? (
          <pre className="mb-2 whitespace-pre-wrap break-words border-l-2 border-bd pl-2 text-tx4">
            {turn.thinking}
          </pre>
        ) : null}
        {body ? <pre className="whitespace-pre-wrap break-words text-tx">{body}</pre> : null}
        {/* An error surfaces even when partial deltas already streamed. */}
        {turn.status === 'error' && turn.error ? (
          <div className="mt-2 text-bad">{turn.error}</div>
        ) : null}
        {!body && turn.status !== 'error' ? (
          streaming ? (
            <span className="text-tx3">l'agent lit la phase · prépare le tour…</span>
          ) : (
            <span className="text-tx4">aucune sortie</span>
          )
        ) : null}
        {streaming ? <span className="text-tx">&nbsp;▌</span> : null}
      </div>

      <div className="flex items-center gap-2.5">
        {streaming ? (
          <button
            type="button"
            data-testid="turn-interrupt"
            onClick={onInterrupt}
            title="arrête d'écouter ce flux (le tour serveur peut continuer)"
            className="rounded-sm border border-bd2 px-3 py-1.5 font-sans text-[11px] font-medium text-tx2 hover:border-bd2 hover:text-tx"
          >
            ⏸ Interrompre l'écoute
          </button>
        ) : (
          <>
            <button
              type="button"
              data-testid="turn-relaunch"
              onClick={onRelaunch}
              title="relancer = nouveau tour assistant à coût d'inférence plein (jamais un no-op)"
              className="rounded-sm border border-bd px-3 py-1.5 font-sans text-[11px] font-medium text-tx3 hover:border-bd2 hover:text-tx2"
            >
              ↻ Relancer le tour
            </button>
            <button
              type="button"
              data-testid="turn-new-session"
              onClick={onNewSession}
              title="repartir d'une session vierge"
              className="rounded-sm border border-bd px-3 py-1.5 font-sans text-[11px] font-medium text-tx4 hover:border-bd2 hover:text-tx3"
            >
              ＋ Nouvelle session
            </button>
          </>
        )}
        {turn.launchError ? (
          <span className="font-mono text-[10px] text-bad">{turn.launchError}</span>
        ) : null}
      </div>
    </section>
  )
}
