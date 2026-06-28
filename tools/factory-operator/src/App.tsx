// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C → D — the bi-focal shell: a permanent altitude-0
// orientation bar + narrow rail framing ONE state-driven scene. Phase D adds
// the VERIFY-bootstrap focal scene (terminal-PTY + gates band) and the rail's
// secondary INSPECTORS (procédé / sessions / knowledge), which replace the
// focal scene body while open. The MODE switch is manual (D6), never arrachée
// au stream. Wiring: useOperator (STEER turn lifecycle + focal mode + open
// surface) + useRailStatus (ambient context from /api/context).
import { lazy, Suspense } from 'react'
import { OrientationBar } from './components/OrientationBar'
import { Rail } from './components/Rail'
import { SteerScene } from './components/steer/SteerScene'
import { useOperator } from './state/useOperator'
import { useRailStatus } from './state/useRailStatus'

// Sprint 80 Phase E — the Motion LIBRARY does NOT live at the hero. Empirically
// (preflight §5.1) `<LazyMotion>` + `<MotionConfig>` pull ~30 KB RAW of engine
// core; at the eager root that busts the 40 KB `index` budget. The eager shell
// uses only the CSS-driven signatures (token settle, gravity) + the native
// View-Transition altitude shift — ZERO Motion-lib weight. The Motion provider
// (LazyMotion + MotionConfig reducedMotion="user") and the JS-driven signatures
// (flip/reveal) are confined to the async VERIFY surface (components/motion/
// MotionProvider), so the whole engine + features load only when VERIFY is
// entered. Reduced-motion stays honoured everywhere: the CSS belt-and-braces
// rule (index.css) covers the CSS signatures, the JS guard covers the View
// Transition, and MotionConfig covers the VERIFY m.* (preflight §3.2).

// STEER is the default focal scene (eager). The VERIFY scene (with its lazy
// terminal) and the secondary-inspector host are CODE-SPLIT: they load when
// the operator switches to VERIFY or opens an inspector, keeping the hero
// `index` chunk under the size-limit gate (Day-0 D4/D5).
const VerifyScene = lazy(() => import('./components/verify/VerifyScene').then((m) => ({ default: m.VerifyScene })))
const SurfaceHost = lazy(() => import('./components/surfaces/SurfaceHost').then((m) => ({ default: m.SurfaceHost })))

export function App() {
  const op = useOperator()
  const status = useRailStatus()

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-s1 text-tx font-sans">
      <OrientationBar status={status} provider={op.provider} />
      <div className="flex min-h-0 flex-1">
        <Rail
          mode={op.mode}
          onMode={op.setMode}
          surface={op.surface}
          onSurface={op.openSurface}
          reachable={status.reachable}
        />
        {/* The focal pane is the only element that morphs on the STEER⇄VERIFY
           bascule (signature 4, altitude shift). `view-transition-name: focal`
           (index.css) lifts it out of the root snapshot so the rail +
           orientation bar stay fixed (Day-0 D8). The View Transition is native
           (CSS-driven) — no Motion-lib weight here. */}
        <div className="motion-focal flex min-h-0 flex-1 flex-col">
          <Suspense fallback={<div className="flex-1 bg-s0" />}>
            {op.surface !== null ? (
              <SurfaceHost op={op} />
            ) : op.mode === 'steer' ? (
              <SteerScene op={op} />
            ) : (
              <VerifyScene />
            )}
          </Suspense>
        </div>
      </div>
    </div>
  )
}
