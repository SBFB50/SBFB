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
  )
}
