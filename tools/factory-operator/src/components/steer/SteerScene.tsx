// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the STEER focal scene (variante B), a single
// state-driven surface:
//   - empty  → composer in GRAND (discoverable)
//   - gated  → the intention echo + the MUR (the flow is split, not started)
//   - active → the atelier dominates, the composer docks at the bottom
// The MODE switch is manual (D6); this scene never auto-flips to VERIFY.

import type { Operator } from '../../state/useOperator'
import { Atelier } from './Atelier'
import { Composer } from './Composer'
import { Mur } from './Mur'

function SceneHeader({ subtitle }: { subtitle: string }) {
  return (
    <div className="flex items-center gap-2.5 border-b border-bd px-5 py-3">
      <span className="h-1.5 w-1.5 rounded-full bg-tx2" aria-hidden />
      <span className="font-sans text-scene font-semibold text-tx">STEER</span>
      <span className="font-sans text-sec text-tx3">— {subtitle}</span>
    </div>
  )
}

export function SteerScene({ op }: { op: Operator }) {
  const { turn } = op

  // Gated: the sensitive intention is restituted as the MUR; no stream ran.
  if (turn.gate && turn.message) {
    return (
      <div data-testid="steer-scene" className="flex min-h-0 flex-1 flex-col bg-s0">
        <SceneHeader subtitle="intention sensible détectée" />
        <div className="border-b border-bd bg-s0 px-5 py-4">
          <div className="mb-2 eyebrow">composeur d'intention</div>
          <div className="rounded-md border border-bd2 bg-s1 px-3.5 py-3 font-sans text-body leading-snug text-tx">
            {turn.message}
          </div>
        </div>
        <Mur message={turn.gate} onBack={op.dismissGate} onPrepare={op.preparePack} />
        <div className="px-5 py-4">
          <div className="rounded-md border border-dashed border-bd px-4 py-4 text-center font-mono text-meta text-tx4">
            le flux ne démarre pas tant que la barrière tient
          </div>
        </div>
      </div>
    )
  }

  // Empty: composer in grand, centred.
  if (!op.hasTurn) {
    return (
      <div data-testid="steer-scene" className="flex min-h-0 flex-1 flex-col bg-s0">
        <SceneHeader subtitle="exprimer une intention · observer l'agent" />
        <div className="flex flex-1 flex-col justify-center">
          <Composer
            variant="grand"
            provider={op.provider}
            onProvider={op.setProvider}
            busy={turn.busy}
            onLaunch={op.launch}
          />
          <p className="border-t border-bd px-7 pb-1 pt-4 text-center font-sans text-sec text-tx4">
            Une intention démarre une session observable — l'agent fabrique, vous vérifiez.
          </p>
        </div>
      </div>
    )
  }

  // Active: atelier dominant + composer docked.
  return (
    <div data-testid="steer-scene" className="flex min-h-0 flex-1 flex-col bg-s0">
      <SceneHeader subtitle="exprimer une intention · observer l'agent" />
      <Atelier
        turn={turn}
        onRelaunch={op.relaunch}
        onInterrupt={op.interrupt}
        onNewSession={op.newSession}
      />
      <Composer
        variant="dock"
        provider={op.provider}
        onProvider={op.setProvider}
        busy={turn.busy}
        onLaunch={op.launch}
      />
    </div>
  )
}
