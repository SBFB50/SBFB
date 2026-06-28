// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 (front rapid-add) — the SMALL, eager-safe gate-status + tone
// display primitives, split out of verdict.ts so the always-visible
// orientation bar (the eager `index` hero) can restitute the live gate pulse
// WITHOUT pulling the VERIFY-only machinery (VERIFY_ETAT, reviewTone,
// preflightTone, pickVerifyEtat) into the hero chunk (Day-0 D4/D5 budget).
// verdict.ts re-exports everything here, so existing `from '../lib/verdict'`
// imports (GatesPanel, ProcedeSurface) keep working unchanged.
//
// Cardinal invariant preserved: these helpers map a RESTITUTED status/verdict
// STRING to a display tone/glyph; they never compute, score, or assert one.
import type { GateStatus } from '../api/operator'

/** A semantic tone keyed to the oklch state tokens (colour = meaning). */
export type Tone = 'ok' | 'warn' | 'bad' | 'info' | 'neu'

/** Named mirror of the five wire gate statuses (gates.rs:75-89, snake_case).
 * README §6.9: an enumerated domain is ONE named constant reused everywhere. */
export const GATE_STATUS = {
  notRun: 'not_run',
  notApplicable: 'not_applicable',
  passed: 'passed',
  informational: 'informational',
  blocking: 'blocking',
} as const satisfies Record<string, GateStatus>

/** Display order for the restituted gate pulse — most-actionable first
 * (blocking → passed → informational → not-run → not-applicable). A single
 * named ordering reused by every gate-count restitution (rail pulse, legends),
 * never an inline literal list. */
export const GATE_STATUS_ORDER: readonly GateStatus[] = [
  GATE_STATUS.blocking,
  GATE_STATUS.passed,
  GATE_STATUS.informational,
  GATE_STATUS.notRun,
  GATE_STATUS.notApplicable,
]

/** Restituted glyph for a gate status — never a verdict word (✓/✕/•/—). The
 * passing glyph is a tick, never the literal "PASS" (scan-front gate). */
export function gateStatusGlyph(status: GateStatus): string {
  switch (status) {
    case 'passed':
      return '✓'
    case 'blocking':
      return '✕'
    case 'not_applicable':
      return '—'
    case 'informational':
    case 'not_run':
      return '•'
  }
}

/** Honest tone of a restituted gate status (colour = meaning). */
export function gateStatusTone(status: GateStatus): Tone {
  switch (status) {
    case 'passed':
      return 'ok'
    case 'blocking':
      return 'bad'
    case 'informational':
      return 'warn'
    case 'not_run':
    case 'not_applicable':
      return 'neu'
  }
}

/** FR label of a restituted gate status (for a11y/title) — deliberately NOT
 * the verdict words PASS / Vérifié / Approuvé (scan-front-discipline gate). */
export function gateStatusLabel(status: GateStatus): string {
  switch (status) {
    case 'passed':
      return 'tenue'
    case 'blocking':
      return 'bloquant'
    case 'informational':
      return 'informatif'
    case 'not_run':
      return 'non exécutée'
    case 'not_applicable':
      return 'hors périmètre'
  }
}

/** Tailwind text-colour class for a tone (the verdict is restituted, the tone
 * is only its honest colour — green for a recorded pass, amber for a concern). */
export function toneText(tone: Tone): string {
  switch (tone) {
    case 'ok':
      return 'text-ok'
    case 'warn':
      return 'text-warn'
    case 'bad':
      return 'text-bad'
    case 'info':
      return 'text-info'
    default:
      return 'text-tx3'
  }
}

/** Tailwind bg-colour class for a tone. Returns a LITERAL class name (never a
 * runtime-built string) so the Tailwind v4 compiler emits the utility. */
export function toneBg(tone: Tone): string {
  switch (tone) {
    case 'ok':
      return 'bg-ok'
    case 'warn':
      return 'bg-warn'
    case 'bad':
      return 'bg-bad'
    case 'info':
      return 'bg-info'
    default:
      return 'bg-tx4'
  }
}
