// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — verdict RESTITUTION helpers. The cardinal invariant:
// every verdict is GRAVED by Rust (preflight/review artifacts, gate runs) and
// the front only RESTITUTES it — it never computes, scores, or asserts one.
// These helpers map a restituted verdict STRING to a display tone; they never
// fabricate a verdict. The single allowed `PASS` literal is the `=== 'PASS'`
// comparison (the front reads a backend verdict) — the scan-front-discipline
// gate strips exactly that form, so no verdict WORD is ever rendered from a
// literal here.

/** A semantic tone keyed to the oklch state tokens (colour = meaning). */
export type Tone = 'ok' | 'warn' | 'bad' | 'info' | 'neu'

/**
 * The live VERIFY état slot — a NAMED, enumerated state machine. It restitutes
 * the scene state and NEVER says the recorded review word; the full
 * state-driven machine (fresh diff/gate → observable) lands Phase H. Phase D's
 * bootstrap surface only needs the honest "awaiting / inspecting" states.
 */
export const VERIFY_ETAT = {
  awaiting: 'En attente de session agent · 0 verdict auto-clos',
  bootstrap: 'Inspection bootstrap · terminal + procédé',
} as const
export type VerifyEtat = keyof typeof VERIFY_ETAT

/**
 * Tone of a RESTITUTED review verdict (`PASS` / `PASS-PENDING` / `CONCERN` /
 * `FAIL`). `=== 'PASS'` is the one allowed PASS literal — the gate strips it
 * before re-scanning, because reading a backend verdict is not asserting one.
 */
export function reviewTone(verdict: string | null | undefined): Tone {
  if (!verdict) return 'neu'
  if (verdict === 'PASS') return 'ok'
  if (verdict.includes('PENDING')) return 'warn'
  if (verdict.includes('CONCERN')) return 'warn'
  if (verdict.includes('FAIL')) return 'bad'
  return 'neu'
}

/** Tone of a RESTITUTED preflight verdict (EXECUTE / PLAN-ADAPT / … ). */
export function preflightTone(verdict: string | null | undefined): Tone {
  if (!verdict) return 'neu'
  if (verdict.includes('EXECUTE')) return 'ok'
  if (verdict.includes('PLAN-ADAPT')) return 'info'
  if (verdict.includes('SCOPE-CUT')) return 'info'
  if (verdict.includes('DESIGN-CONFLICT')) return 'bad'
  return 'neu'
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
