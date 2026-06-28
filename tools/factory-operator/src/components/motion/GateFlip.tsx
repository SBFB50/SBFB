// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase E — signature (2) "gate flip": a restituted gate/état value
// flips on change. The value is ALWAYS read from the backend (a restituted
// verdict / enumerated état); this component animates the TRANSITION between
// values, it never decides or fabricates one (kickoff "0 verdict calculé UI",
// scan-front-discipline gate). It is a reusable primitive — Phase H wires the
// full gates panel; here it dresses the VERIFY état slot.
//
// Anchored on `rotateX` (a transform). Under prefers-reduced-motion we render a
// PLAIN <span> — no motion component, zero WAAPI animation, the final state at
// once (the CSS reset does NOT reach WAAPI, and reducedMotion="user" does not
// instantaneise opacity; preflight §3.2). Motion lib usage is confined to this
// async-surface module (imported only by VerifyScene, a React.lazy chunk) so the
// engine never reaches the hero. The minimal motion component comes NAMED from
// 'motion/react-m' — never the full `motion` export (eslint gate 4) and never
// `import * as` (preflight §3.1.3).
import { AnimatePresence } from 'motion/react'
import { span as MSpan } from 'motion/react-m'
import { GATE_FLIP_TRANSITION } from '../../lib/motion'
import { usePrefersReducedMotion } from '../../lib/usePrefersReducedMotion'

export function GateFlip({ value, className }: { value: string; className?: string }) {
  const reduce = usePrefersReducedMotion()
  if (reduce) {
    return (
      <span data-testid="gate-flip" className={className}>
        {value}
      </span>
    )
  }
  return (
    <AnimatePresence mode="wait" initial={false}>
      <MSpan
        key={value}
        data-testid="gate-flip"
        initial={{ rotateX: -90, opacity: 0 }}
        animate={{ rotateX: 0, opacity: 1 }}
        exit={{ rotateX: 90, opacity: 0 }}
        transition={GATE_FLIP_TRANSITION}
        style={{ display: 'inline-block', transformOrigin: 'center', transformStyle: 'preserve-3d' }}
        className={className}
      >
        {value}
      </MSpan>
    </AnimatePresence>
  )
}
