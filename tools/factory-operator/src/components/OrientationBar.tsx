// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C → front rapid-add — the altitude-0 orientation bar:
// full-width, permanent, NEVER part of a transition (Day-0 #8). It restitutes
// the ambient context from `GET /api/context` — sprint · phase · branch ·
// dirty/staged — plus a loopback reachability dot. Since Phase G `/api/gates`
// exists, the "pouls gates" is now LIVE: it restitutes a COUNT per gate status
// (never an aggregate verdict — the cardinal "0 verdict calculé UI" holds, a
// count of restituted statuses is restitution, not a fabricated PASS/score).
// A manual refresh re-reads the context for freshness after a commit. (The
// full sprint commit timeline lives in the Procédé inspector, not the bar.)

import type { RailHandle } from '../state/useRailStatus'
import type { ExecProvider } from '../catalog/intentions'
import { EXEC_PROVIDERS } from '../catalog/intentions'
import { GATE_STATUS_ORDER, gateStatusGlyph, gateStatusLabel, gateStatusTone, toneText } from '../lib/gateStatus'
import { TokenCount } from './motion/TokenCount'

function providerLabel(provider: ExecProvider): string {
  const opt = EXEC_PROVIDERS.find((p) => p.id === provider)
  return opt ? `${opt.label.toLowerCase()} · ${opt.note}` : provider
}

function GatePulse({ counts }: { counts: RailHandle['gateCounts'] }) {
  if (counts === null) {
    return (
      <span className="text-tx4" title="pouls des gates — lecture…">
        gates …
      </span>
    )
  }
  const active = GATE_STATUS_ORDER.filter((s) => counts[s] > 0)
  if (active.length === 0) {
    return (
      <span className="text-tx4" title="aucun gate restitué">
        gates —
      </span>
    )
  }
  return (
    <span className="flex items-center gap-1.5" data-testid="gate-pulse" title="pouls des gates — compte par statut restitué">
      {active.map((s) => (
        <span key={s} className={`flex items-center gap-0.5 ${toneText(gateStatusTone(s))}`} title={gateStatusLabel(s)}>
          <span aria-hidden>{gateStatusGlyph(s)}</span>
          <span className="tabular-nums">{counts[s]}</span>
        </span>
      ))}
    </span>
  )
}

export function OrientationBar({
  status,
  provider,
}: {
  status: RailHandle
  provider: ExecProvider
}) {
  const sprint = status.sprint !== null ? `Sprint ${status.sprint}` : '—'
  const phase = status.phase ?? '—'
  const branch = status.branch ?? '—'

  return (
    <header
      data-testid="operator-orientation"
      className="flex h-12 flex-shrink-0 items-center gap-0 border-b border-bd bg-s2 px-4"
    >
      <div className="flex items-center gap-2 border-r border-bd pr-4">
        <span className="h-2 w-2 rounded-sm bg-tx2" aria-hidden />
        <span className="font-mono text-sec font-semibold tracking-wide text-tx2">
          FACTORY&nbsp;OPERATOR
        </span>
      </div>

      <div className="flex items-center gap-2.5 overflow-hidden pl-4 font-mono text-meta tabular-nums text-tx2">
        <span className="text-tx">{sprint}</span>
        <span className="text-tx3" aria-hidden>
          ·
        </span>
        <span className="truncate">{phase}</span>
        <span className="text-tx4" aria-hidden>
          ▸
        </span>
        <span className="truncate text-tx">{branch}</span>
        <span className="text-tx4" aria-hidden>
          ▸
        </span>
        <span className="text-info" aria-hidden>
          ●
        </span>
        {/* token settle (signature 1): the live counter settles in on change. */}
        <span>
          <TokenCount value={status.dirty ?? '—'} /> modifiés
        </span>
        <span className="text-tx3" aria-hidden>
          ·
        </span>
        <span className="text-tx3">
          <TokenCount value={status.staged ?? '—'} /> indexés
        </span>
        <span className="text-tx4" aria-hidden>
          ▸
        </span>
        {/* Gates pulse — LIVE since Phase G: restituted counts, never a verdict. */}
        <GatePulse counts={status.gateCounts} />
      </div>

      <div className="ml-auto flex items-center gap-3 pl-4 font-mono text-meta text-tx3">
        <button
          type="button"
          onClick={status.refresh}
          disabled={status.loading}
          data-testid="context-refresh"
          title="rafraîchir le contexte (sprint · arbre de travail · gates)"
          className="rounded-sm border border-bd bg-s1 px-1.5 py-0.5 text-tx3 hover:bg-s2 disabled:opacity-50"
          aria-busy={status.loading}
        >
          <span aria-hidden>↻</span>
          <span className="sr-only">rafraîchir le contexte</span>
        </button>
        <span>
          agent : <span className="text-tx2">{providerLabel(provider)}</span>
        </span>
        <span className="flex items-center gap-1.5" title="lien loopback avec le nœud">
          <span
            className={status.reachable ? 'h-1.5 w-1.5 rounded-full bg-ok' : 'h-1.5 w-1.5 rounded-full bg-tx4'}
            aria-hidden
          />
          {status.reachable ? 'loopback' : status.loading ? 'liaison…' : 'hors-ligne'}
        </span>
      </div>
    </header>
  )
}
