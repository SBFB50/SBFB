// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase E — signature (3) "verification reveal": a verification
// surface reveals its artifacts in a short upward stagger, so the eye reads the
// pieces in order rather than all at once. Meaning, not decor: the stagger is
// the act of laying out evidence.
//
// Each child is anchored on `y` (a transform); the opacity is the part that
// reducedMotion="user" would NOT instantaneise (preflight §3.2), so under
// prefers-reduced-motion we render PLAIN <div>s — no motion component, zero
// WAAPI animation, every child in its final state at once. No `layout` prop
// (reserved to `domMax`, excluded). Motion lib usage confined to this
// async-surface module; the minimal component comes NAMED from 'motion/react-m'
// (never the full `motion`, never `import * as`; eslint gate 4 / preflight §3.1.3).
import type { ReactNode } from 'react'
import { div as MDiv } from 'motion/react-m'
import { REVEAL_CHILD_TRANSITION, REVEAL_STAGGER_S, MOTION_TRAVEL_PX } from '../../lib/motion'
import { usePrefersReducedMotion } from '../../lib/usePrefersReducedMotion'

export function Reveal({ children, className }: { children: ReactNode; className?: string }) {
  const reduce = usePrefersReducedMotion()
  if (reduce) {
    return (
      <div data-testid="reveal" className={className}>
        {children}
      </div>
    )
  }
  return (
    <MDiv
      data-testid="reveal"
      initial="hidden"
      animate="shown"
      variants={{ hidden: {}, shown: { transition: { staggerChildren: REVEAL_STAGGER_S } } }}
      className={className}
    >
      {children}
    </MDiv>
  )
}

export function RevealItem({ children, className }: { children: ReactNode; className?: string }) {
  const reduce = usePrefersReducedMotion()
  if (reduce) {
    return <div className={className}>{children}</div>
  }
  return (
    <MDiv
      variants={{ hidden: { opacity: 0, y: MOTION_TRAVEL_PX }, shown: { opacity: 1, y: 0 } }}
      transition={REVEAL_CHILD_TRANSITION}
      className={className}
    >
      {children}
    </MDiv>
  )
}
