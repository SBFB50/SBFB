// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase E — read the user's reduced-motion preference, reliably.
// Motion animates via WAAPI (`element.animate`), which the CSS
// `@media (prefers-reduced-motion)` reset does NOT affect, and
// `MotionConfig reducedMotion="user"` deliberately does NOT instantaneise
// opacity/color (preflight §3.2). So the JS-driven signatures (flip, reveal)
// must gate themselves: under reduced motion they render PLAIN (no motion
// component, zero animation) — the final, instant state. This hook reads
// `matchMedia` SYNCHRONOUSLY in the useState initializer (correct on the very
// first render, unlike a value that resolves in an effect) and subscribes to
// live changes.
import { useEffect, useState } from 'react'
import { REDUCED_MOTION_QUERY } from './motion'

export function usePrefersReducedMotion(): boolean {
  const [reduce, setReduce] = useState(
    () =>
      typeof window !== 'undefined' &&
      typeof window.matchMedia === 'function' &&
      window.matchMedia(REDUCED_MOTION_QUERY).matches,
  )

  useEffect(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return
    const mql = window.matchMedia(REDUCED_MOTION_QUERY)
    const onChange = () => setReduce(mql.matches)
    mql.addEventListener('change', onChange)
    return () => mql.removeEventListener('change', onChange)
  }, [])

  return reduce
}
