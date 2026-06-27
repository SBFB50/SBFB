// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the narrow vertical rail (158px). It holds the focal
// MODE toggle (D6: MANUAL STEER⇄VERIFY, never auto, the toggle is core) and
// the secondary-surface entries. The rail is the stable altitude-0 frame: it
// never transitions. The secondary surfaces (Terminal / Sessions / Historique
// / Knowledge) are INERT placeholders here — they are wired in Phase D; we
// render them disabled rather than faking working buttons.
//
// This component carries `data-testid="operator-rail"` so the Phase B boot
// E2E (which asserts the shell rendered) stays green against the real rail.

import { cn } from '../lib/cn'
import type { FocalMode } from '../state/useOperator'

const SECONDARY_SURFACES: readonly { id: string; glyph: string; label: string; badge?: string }[] = [
  { id: 'terminal', glyph: '⌃', label: 'Terminal' },
  { id: 'sessions', glyph: '≣', label: 'Sessions' },
  { id: 'history', glyph: '◴', label: 'Historique' },
  { id: 'knowledge', glyph: '◇', label: 'Knowledge', badge: 'consult.' },
]

function ModeButton({
  active,
  label,
  hint,
  onClick,
}: {
  active: boolean
  label: string
  hint?: string
  onClick: () => void
}) {
  return (
    <button
      type="button"
      aria-pressed={active}
      onClick={onClick}
      className={cn(
        'flex items-center gap-2 border-l-2 px-2.5 py-2.5 text-left transition-none',
        active ? 'border-l-tx bg-s3' : 'border-l-transparent bg-transparent hover:bg-s2',
      )}
    >
      <span
        className={cn(
          'h-1.5 w-1.5 rounded-full',
          active ? 'bg-tx' : 'border border-bd2',
        )}
        aria-hidden
      />
      <span className={cn('font-sans text-xs', active ? 'font-semibold text-tx' : 'text-tx3')}>
        {label}
      </span>
      {hint ? <span className="ml-auto font-mono text-[8.5px] text-info">{hint}</span> : null}
    </button>
  )
}

export function Rail({
  mode,
  onMode,
  reachable,
}: {
  mode: FocalMode
  onMode: (mode: FocalMode) => void
  reachable: boolean
}) {
  return (
    <nav
      data-testid="operator-rail"
      aria-label="Orientation"
      className="flex w-[158px] flex-shrink-0 flex-col border-r border-bd bg-s1 px-2.5 py-3"
    >
      <div className="mb-2 font-sans text-[8.5px] font-semibold uppercase tracking-[0.16em] text-tx4">
        mode focal
      </div>
      <div className="mb-1.5 flex flex-col overflow-hidden rounded-sm border border-bd" role="group" aria-label="Mode focal">
        <ModeButton active={mode === 'steer'} label="STEER" onClick={() => onMode('steer')} />
        <div className="border-t border-bd" />
        <ModeButton
          active={mode === 'verify'}
          label="VERIFY"
          onClick={() => onMode('verify')}
        />
      </div>
      <div className="mb-4 font-mono text-[8.5px] leading-tight text-tx4">
        bascule manuelle · jamais auto
      </div>

      <div className="mb-2 font-sans text-[8.5px] font-semibold uppercase tracking-[0.16em] text-tx4">
        surfaces
      </div>
      <div className="flex flex-col gap-px">
        {SECONDARY_SURFACES.map((s) => (
          <button
            key={s.id}
            type="button"
            disabled
            aria-disabled
            title="surface secondaire — câblage Phase D"
            className="flex cursor-not-allowed items-center gap-2.5 rounded-sm px-2 py-1.5 text-left opacity-60"
          >
            <span className="w-3 font-mono text-[11px] text-tx4" aria-hidden>
              {s.glyph}
            </span>
            <span className="font-sans text-[11.5px] text-tx3">{s.label}</span>
            {s.badge ? (
              <span className="ml-auto rounded-sm border border-dashed border-bd2 px-1 py-0.5 font-mono text-[7.5px] uppercase text-tx4">
                {s.badge}
              </span>
            ) : null}
          </button>
        ))}
      </div>

      <div className="mt-auto border-t border-bd pt-3 font-mono text-[8.5px] leading-relaxed text-tx4">
        <span className={reachable ? 'text-ok' : 'text-tx4'}>{reachable ? 'token ✓' : 'token ·'}</span>{' '}
        · nœud souverain
        <br />
        factory-operator · v0
      </div>
    </nav>
  )
}
