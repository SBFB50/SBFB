// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase E — the Motion-library boundary. Wraps the surfaces that use
// the JS-driven signatures (flip, reveal). It is imported ONLY by async
// (React.lazy) surfaces (VERIFY today), so the ~30 KB engine core + the
// `domAnimation` feature bundle load lazily WITH that surface, never at the hero
// `index` chunk (preflight §5.1 — the empirical core size forced the provider
// out of the eager root). Because this async surface is the SOLE importer of the
// Motion packages, rolldown's natural split lands them all in the async
// `VerifyScene-*.js` chunk — verified at the bundle: index.html does NOT
// modulepreload them and the hero imports nothing motion. That chunk is MEASURED
// by size-limit (`verify-surface` entry, .size-limit.json), so nothing
// motion-related escapes the gate. (A forced `manualChunk` for Motion was tried
// and REJECTED: it leaked React core into the motion chunk, dragging it eager
// into the hero — review P1.)
//
// `reducedMotion="user"` is the global belt-and-braces for the contained m.* (it
// instantaneises transform/positional keys). The actual reduced-motion gate is
// stronger: GateFlip/Reveal render PLAIN (no motion component) under
// `usePrefersReducedMotion`, so opacity never tweens either. `<LazyMotion strict>`
// is the runtime twin of the eslint gate: only the minimal `m` components are
// allowed, never the full `motion.*`. `features={domAnimation}` is a STATIC
// import here (allowed — only the `motion` named import is banned) because this
// whole module already sits behind an async boundary.
import type { ReactNode } from 'react'
import { LazyMotion, MotionConfig, domAnimation } from 'motion/react'

export function MotionProvider({ children }: { children: ReactNode }) {
  return (
    <MotionConfig reducedMotion="user">
      <LazyMotion strict features={domAnimation}>
        {children}
      </LazyMotion>
    </MotionConfig>
  )
}
