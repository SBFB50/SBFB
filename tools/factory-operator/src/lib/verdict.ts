// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D → front rapid-add — verdict RESTITUTION helpers. The
// cardinal invariant: every verdict is GRAVED by Rust (preflight/review
// artifacts, gate runs) and the front only RESTITUTES it — it never computes,
// scores, or asserts one.
//
// The SMALL, eager-safe gate-status + tone primitives live in `gateStatus.ts`
// (so the orientation-bar hero can restitute the live gate pulse without
// pulling this whole module). They are RE-EXPORTED here so the VERIFY-surface
// consumers (GatesPanel, ProcedeSurface) keep importing from `verdict`
// unchanged. This module additionally carries the VERIFY-only machinery
// (VERIFY_ETAT, pickVerifyEtat, reviewTone, preflightTone).
import type { Tone } from './gateStatus'

export type { Tone } from './gateStatus'
export {
  GATE_STATUS,
  GATE_STATUS_ORDER,
  gateStatusGlyph,
  gateStatusTone,
  gateStatusLabel,
  toneText,
  toneBg,
} from './gateStatus'

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
