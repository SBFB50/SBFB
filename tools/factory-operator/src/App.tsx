// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C → H — the bi-focal shell: a permanent altitude-0
// orientation bar + narrow rail framing ONE state-driven scene. Phase D added
// the VERIFY-bootstrap focal scene; Phase H made VERIFY the full diff-viewer +
// live gates panel (the terminal-PTY is now a secondary tool there). The rail's
// secondary INSPECTORS (procédé / sessions / knowledge) replace the focal scene
// body while open. The MODE switch is manual (D6), never arrachée au stream;
// `verifyReady` only lights an availability hint, never an auto-switch. Wiring:
// useOperator (STEER turn lifecycle + focal mode + open surface + verifyReady) +
// useRailStatus (ambient context from /api/context + gate pulse from /api/gates)
// + useFocalKeys (keyboard s/v focal switch, D6 manual).
import { lazy, Suspense } from 'react'
import { OrientationBar } from './components/OrientationBar'
import { Rail } from './components/Rail'
import { ErrorBoundary } from './components/ErrorBoundary'
import { SteerScene } from './components/steer/SteerScene'
import { useOperator } from './state/useOperator'
import { useRailStatus } from './state/useRailStatus'
import { useFocalKeys } from './state/useFocalKeys'

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
  // Keyboard focal switch (D6 manual): `s`/`v` when not typing.
  useFocalKeys(op.setMode)

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-s1 text-tx font-sans">
      <OrientationBar status={status} provider={op.provider} />
      {/* Offline banner — one honest global signal when the loopback link is
         down, instead of N isolated per-surface errors. Reuses the rail's
         `reachable` (no extra fetch); hidden while the first probe is still in
         flight so it never flashes during boot. */}
      {!status.reachable && !status.loading && (
        <div
          role="status"
          data-testid="offline-banner"
          className="flex h-7 flex-shrink-0 items-center gap-2 border-b border-bd bg-bad-bg px-4 font-mono text-[11px] text-bad"
        >
          <span className="h-1.5 w-1.5 rounded-full bg-bad" aria-hidden />
          Nœud injoignable — l'Operator ne répond pas sur le lien loopback. Les surfaces
          resteront vides jusqu'au rétablissement.
        </div>
      )}
      <div className="flex min-h-0 flex-1">
        <Rail
          mode={op.mode}
          onMode={op.setMode}
          surface={op.surface}
          onSurface={op.openSurface}
          reachable={status.reachable}
          verifyReady={op.verifyReady}
        />
        {/* The focal pane is the only element that morphs on the STEER⇄VERIFY
           bascule (signature 4, altitude shift). `view-transition-name: focal`
           (index.css) lifts it out of the root snapshot so the rail +
           orientation bar stay fixed (Day-0 D8). The View Transition is native
           (CSS-driven) — no Motion-lib weight here. */}
        <div className="motion-focal flex min-h-0 flex-1 flex-col">
          {/* A scoped boundary around the focal pane: a throw in a surface
             shows the recoverable fallback there while the rail + orientation
             bar stay alive (the global boundary in main.tsx is the last net). */}
          <ErrorBoundary scope="surface focale">
            <Suspense fallback={<div className="flex-1 bg-s0" />}>
              {op.surface !== null ? (
                <SurfaceHost op={op} />
              ) : op.mode === 'steer' ? (
                <SteerScene op={op} />
              ) : (
                <VerifyScene op={op} />
              )}
            </Suspense>
          </ErrorBoundary>
        </div>
      </div>
    </div>
  )
}
