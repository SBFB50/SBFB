// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase E — the motion vocabulary. "Le mouvement est du SENS,
// jamais de la décoration" (kickoff Day-0 D3). Motion is allowed in exactly
// FIVE signatures, each tied to a meaning; anything outside this list is decor
// and must not ship. No script mechanises this allowlist (the eslint gate only
// forbids the heavy `motion`/`<motion.*>` entrypoints, not which signatures
// exist), so it lives here as a NAMED, reviewable constant (README §6.9 —
// "domaine énuméré ⇒ constante nommée unique") and is enforced at review/Codex.
//
// Anti-déco invariant (preflight §3.2): `MotionConfig reducedMotion="user"`
// instantaneises ONLY transform/positional keys, NEVER opacity/color. Every
// signature is therefore ANCHORED on a transform (y / scale / rotate) so it
// collapses to its final state natively under prefers-reduced-motion, and any
// non-transform value (opacity) is additionally gated by `useReducedMotion()`
// at the call site + the belt-and-braces `@media (prefers-reduced-motion)`
// rule in index.css.

import type { Transition } from 'motion/react'

/** The reduced-motion media query — the single source of truth for the JS
 *  guards (altitudeShift + usePrefersReducedMotion). Motion animates via WAAPI
 *  (`element.animate`), which the CSS `@media (prefers-reduced-motion)` reset
 *  does NOT touch, so the JS-driven signatures must gate on this themselves. */
export const REDUCED_MOTION_QUERY = '(prefers-reduced-motion: reduce)'

/** The five — and only five — motion signatures (the doctrinal allowlist). */
export const MOTION_SIGNATURES = {
  /** (1) a live counter posts its new value with a small upward settle. */
  tokenSettle: 'token-settle',
  /** (2) a restituted gate/état value flips on change (the value is read from
   *  the backend; the flip never FABRICATES a verdict). */
  gateFlip: 'gate-flip',
  /** (3) a verification surface reveals its artifacts in a short stagger. */
  verificationReveal: 'verification-reveal',
  /** (4) the bi-focal STEER⇄VERIFY bascule shifts altitude (native View
   *  Transition; the rail is excluded). */
  altitudeShift: 'altitude-shift',
  /** (5) the governance MUR enters with weight — physics = gravity = the
   *  weight of the consequence (0 Forcer / Override / Bypass). */
  confirmationGravity: 'confirmation-gravity',
} as const

export type MotionSignature = (typeof MOTION_SIGNATURES)[keyof typeof MOTION_SIGNATURES]

/** All signature ids, for enumeration (tests, review). */
export const MOTION_SIGNATURE_IDS: readonly MotionSignature[] = Object.values(MOTION_SIGNATURES)

// --- JS transitions for the Motion-lib signatures (flip, reveal). The
// CSS-driven signatures (token settle, gravity) and the native View-Transition
// altitude shift carry their durations directly in index.css — CSS cannot
// import these TS values, so duplicating them here as a "kept in sync" constant
// would be a false source of truth (and dead code). `ease: [0.2,0,0,1]` ≈ a
// calm easeOut.

/** (2) gate flip — JS transition (Motion lib, confined to async surfaces). */
export const GATE_FLIP_TRANSITION: Transition = { duration: 0.18, ease: [0.2, 0, 0, 1] }
/** (3) verification reveal — per-child JS transition + inter-child stagger. */
export const REVEAL_CHILD_TRANSITION: Transition = { duration: 0.22, ease: [0.2, 0, 0, 1] }
export const REVEAL_STAGGER_S = 0.04
/** The transform travel of a revealed/flipped child (px) — positional ⇒
 *  collapses to 0 natively under reduced-motion. */
export const MOTION_TRAVEL_PX = 8
