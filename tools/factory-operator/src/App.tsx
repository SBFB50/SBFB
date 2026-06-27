// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the bi-focal shell: a permanent altitude-0 orientation
// bar + narrow rail framing ONE state-driven mono-focal scene (STEER now,
// VERIFY placeholder until Phase H). The MODE switch is manual (D6), never
// arrachée au stream. Wiring: useOperator (STEER turn lifecycle over
// useTokenStream) + useRailStatus (ambient context from /api/context).
import { OrientationBar } from './components/OrientationBar'
import { Rail } from './components/Rail'
import { SteerScene } from './components/steer/SteerScene'
import { VerifyPlaceholder } from './components/verify/VerifyPlaceholder'
import { useOperator } from './state/useOperator'
import { useRailStatus } from './state/useRailStatus'

export function App() {
  const op = useOperator()
  const status = useRailStatus()

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-s1 text-tx font-sans">
      <OrientationBar status={status} provider={op.provider} />
      <div className="flex min-h-0 flex-1">
        <Rail mode={op.mode} onMode={op.setMode} reachable={status.reachable} />
        {op.mode === 'steer' ? <SteerScene op={op} /> : <VerifyPlaceholder />}
      </div>
    </div>
  )
}
