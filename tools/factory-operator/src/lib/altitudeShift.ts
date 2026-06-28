// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase E — signature (4) "altitude shift": the bi-focal STEER⇄VERIFY
// bascule morphs the focal pane via a NATIVE View Transition.
//
// Why native (preflight §3.3, CSP verrou): the Operator is served under a
// strict self-origin CSP (`default-src 'self'`, no 'unsafe-inline'). Motion's
// `animateView()` wrapper injects a runtime `<style>` WITHOUT a nonce → it
// would be blocked. `document.startViewTransition` injects no markup; the
// pseudo-elements `::view-transition-*(focal)` are styled by the BUNDLED CSS
// (same-origin, allowed) and `view-transition-name` is set in that CSS, so the
// whole signature is CSP-safe. The rail is excluded (`view-transition-name:
// none`, index.css) so it never morphs (Day-0 D8).
//
// Reduced-motion: a View Transition is NOT auto-gated by `MotionConfig`, so we
// guard explicitly here — under prefers-reduced-motion (or when the API is
// absent) we apply the state change synchronously with no transition. The
// belt-and-braces `@media (prefers-reduced-motion)` rule in index.css is a
// second net for the pseudo-elements.

import { flushSync } from 'react-dom'
import { REDUCED_MOTION_QUERY } from './motion'

function prefersReducedMotion(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia(REDUCED_MOTION_QUERY).matches
  )
}

/**
 * Apply a focal state change inside a native View Transition. `apply` mutates
 * React state (e.g. the focal mode); `flushSync` forces it to the DOM
 * synchronously so the View Transition captures the NEW pane. Falls back to a
 * plain synchronous apply under reduced-motion or when the API is unavailable
 * (`startViewTransition` is typed on Document but absent at runtime in older
 * browsers + jsdom — the `typeof` guard covers both).
 */
export function altitudeShift(apply: () => void): void {
  if (
    typeof document === 'undefined' ||
    typeof document.startViewTransition !== 'function' ||
    prefersReducedMotion()
  ) {
    apply()
    return
  }
  document.startViewTransition(() => flushSync(apply))
}
