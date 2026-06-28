// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C → D — the narrow vertical rail (158px). It holds the focal
// MODE toggle (D6: MANUAL STEER⇄VERIFY, never auto, the toggle is core) and,
// since Phase D, the WIRED secondary inspectors (Procédé / Sessions /
// Knowledge). Selecting an inspector opens it over the focal scene; selecting
// a MODE returns to that focal scene (closing any inspector). The rail is the
// stable altitude-0 frame: it never transitions. The terminal lives in the
// VERIFY focal scene (not here) — "terminal élevé en surface VERIFY".
//
// `data-testid="operator-rail"` keeps the boot E2E (asserts the shell
// rendered) green against the real rail.
import { cn } from '../lib/cn'
import type { FocalMode } from '../state/useOperator'
import { SECONDARY_SURFACES, type SecondarySurface } from '../catalog/surfaces'

function ModeButton({
  active,
  label,
  onClick,
  ready,
}: {
  active: boolean
  label: string
  onClick: () => void
  /** D6 disponibilité: light a "ready" dot when a complete turn awaits
   * examination. The toggle stays manual — this is an affordance, not a switch. */
  ready?: boolean
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
      <span className={cn('h-1.5 w-1.5 rounded-full', active ? 'bg-tx' : 'border border-bd2')} aria-hidden />
      <span className={cn('font-sans text-xs', active ? 'font-semibold text-tx' : 'text-tx3')}>{label}</span>
      {ready && !active ? (
        <span
          data-testid="verify-ready"
          className="ml-auto h-1.5 w-1.5 rounded-full bg-info"
          title="un tour terminé attend l’examen — bascule manuelle"
          aria-label="prêt à examiner"
        />
      ) : null}
    </button>
  )
}

export function Rail({
  mode,
  onMode,
  surface,
  onSurface,
  reachable,
  verifyReady,
}: {
  mode: FocalMode
  onMode: (mode: FocalMode) => void
  surface: SecondarySurface | null
  onSurface: (surface: SecondarySurface) => void
  reachable: boolean
  verifyReady?: boolean
}) {
  return (
    <nav
      data-testid="operator-rail"
      aria-label="Orientation"
      className="flex w-[158px] flex-shrink-0 flex-col border-r border-bd bg-s1 px-2.5 py-3"
    >
      <div className="mb-2 font-sans text-[8.5px] font-semibold uppercase tracking-[0.16em] text-tx4">mode focal</div>
      <div
        className="mb-1.5 flex flex-col overflow-hidden rounded-sm border border-bd"
        role="group"
        aria-label="Mode focal"
      >
        <ModeButton active={surface === null && mode === 'steer'} label="STEER" onClick={() => onMode('steer')} />
        <div className="border-t border-bd" />
        <ModeButton
          active={surface === null && mode === 'verify'}
          label="VERIFY"
          onClick={() => onMode('verify')}
          ready={verifyReady}
        />
      </div>
      <div className="mb-4 font-mono text-[8.5px] leading-tight text-tx4">bascule manuelle · jamais auto</div>

      <div className="mb-2 font-sans text-[8.5px] font-semibold uppercase tracking-[0.16em] text-tx4">inspecteurs</div>
      <div className="flex flex-col gap-px">
        {SECONDARY_SURFACES.map((s) => {
          const active = surface === s.id
          return (
            <button
              key={s.id}
              type="button"
              data-testid={`rail-surface-${s.id}`}
              aria-pressed={active}
              onClick={() => onSurface(s.id)}
              title={s.hint}
              className={cn(
                'flex items-center gap-2.5 rounded-sm px-2 py-1.5 text-left transition-none',
                active ? 'bg-s3' : 'hover:bg-s2',
              )}
            >
              <span className="w-3 font-mono text-[11px] text-tx4" aria-hidden>
                {s.glyph}
              </span>
              <span className={cn('font-sans text-[11.5px]', active ? 'font-semibold text-tx' : 'text-tx3')}>
                {s.label}
              </span>
            </button>
          )
        })}
      </div>

      <div className="mt-auto border-t border-bd pt-3 font-mono text-[8.5px] leading-relaxed text-tx4">
        <span className={reachable ? 'text-ok' : 'text-tx4'}>{reachable ? 'token ✓' : 'token ·'}</span> · nœud souverain
        <br />
        factory-operator · v0
      </div>
    </nav>
  )
}
