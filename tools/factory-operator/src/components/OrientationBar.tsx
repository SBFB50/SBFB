// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the altitude-0 orientation bar: full-width, permanent,
// NEVER part of a transition (Day-0 #8). It restitutes the ambient context
// from `GET /api/context` — sprint · phase · branch · dirty/staged — plus a
// loopback reachability dot. The "pouls gates" is a muted placeholder: it is
// not wired until Phase G (`/api/gates`) and the front never fabricates a
// verdict (plan-adaptation #7).

import type { RailStatus } from '../state/useRailStatus'
import type { ExecProvider } from '../catalog/intentions'
import { EXEC_PROVIDERS } from '../catalog/intentions'
import { TokenCount } from './motion/TokenCount'

function providerLabel(provider: ExecProvider): string {
  const opt = EXEC_PROVIDERS.find((p) => p.id === provider)
  return opt ? `${opt.label.toLowerCase()} · ${opt.note}` : provider
}

export function OrientationBar({
  status,
  provider,
}: {
  status: RailStatus
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
        <span className="font-mono text-[11px] font-semibold tracking-wide text-tx2">
          FACTORY&nbsp;OPERATOR
        </span>
      </div>

      <div className="flex items-center gap-2.5 overflow-hidden pl-4 font-mono text-[11.5px] tabular-nums text-tx2">
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
        {/* Gates pulse — not wired before Phase G; never a verdict. */}
        <span className="text-tx4" title="pouls des gates — câblage Phase G">
          gates —
        </span>
      </div>

      <div className="ml-auto flex items-center gap-3.5 pl-4 font-mono text-[10.5px] text-tx3">
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
