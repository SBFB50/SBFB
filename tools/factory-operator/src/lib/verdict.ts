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
import type { GateStatus } from '../api/operator'

/** A semantic tone keyed to the oklch state tokens (colour = meaning). */
export type Tone = 'ok' | 'warn' | 'bad' | 'info' | 'neu'

/**
 * The live VERIFY état slot — a NAMED, enumerated state machine (Phase H). It
 * restitutes the OBSERVABLE scene state (reading / changes present / clean /
 * unavailable) and NEVER says the recorded review word, never a "PASS", never
 * an aggregated verdict (the cardinal invariant + scan-front-discipline gate).
 * Each value is a fixed string; `pickVerifyEtat` selects one from scene facts
 * (loading, whether there is anything to examine) — selecting a named UI state
 * from observable facts is restitution, not a verdict.
 *
 * NOTE — no 'stale'/'obsolète' freshness state in S80: the only available live
 * head (the rail's `/api/context`) is fetched once at mount and never
 * refreshed, so a "◦ obsolète" badge derived from it would lie after the first
 * in-session commit (review P1-1). The honest freshness divergence indicator
 * needs a re-polled live head (or a rev field on `/api/gates`) → carry S81. S80
 * restitutes `run@<rev>` (the displayed diff's head) + a manual "relancer".
 */
export const VERIFY_ETAT = {
  awaiting: 'En attente de session agent · 0 verdict auto-clos',
  bootstrap: 'Inspection bootstrap · terminal + procédé',
  reading: 'Lecture du diff et des gates…',
  inspecting: 'Examen du diff en cours · 0 verdict auto-clos',
  empty: 'Arbre de travail propre · rien à examiner',
  unavailable: 'VERIFY indisponible — relancer la lecture',
} as const
export type VerifyEtat = keyof typeof VERIFY_ETAT

/** Select the named VERIFY état from observable scene facts. NOT a verdict:
 * it reads "is it loading / is there anything to examine", never "does it
 * pass". The error case ('unavailable') is selected by the caller. */
export function pickVerifyEtat(scene: { loading: boolean; hasChanges: boolean }): VerifyEtat {
  if (scene.loading) return 'reading'
  if (scene.hasChanges) return 'inspecting'
  return 'empty'
}

// --- Sprint 80 Phase H — live gate restitution (GET /api/gates) ---
//
// The single named mirror of the Rust `GateStatus` enum (gates.rs:75-89),
// README §6.9: an enumerated domain is ONE named constant reused everywhere,
// never a duplicated literal. `satisfies` pins every value to a real wire
// status at compile time.

/** Named mirror of the five wire gate statuses. */
export const GATE_STATUS = {
  notRun: 'not_run',
  notApplicable: 'not_applicable',
  passed: 'passed',
  informational: 'informational',
  blocking: 'blocking',
} as const satisfies Record<string, GateStatus>

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
