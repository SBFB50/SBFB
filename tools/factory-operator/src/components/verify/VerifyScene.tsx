// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the VERIFY focal scene of BOOTSTRAP. The bespoke
// diff-viewer + rich gates panel are Phase H; until then the terminal-PTY is
// elevated as the verification tool (the operator inspects the repo by hand)
// and the rail's Procédé / Sessions inspectors carry the past-commit diffs and
// the journal. The permanent bottom band is honest: the gates pulse is not
// wired before Phase G, and the ÉTAT slot is a NAMED enumerated state that
// never says the recorded review word (scan-front-discipline gate; kickoff
// "0 verdict calculé UI").
import { Terminal } from './Terminal'
import { VERIFY_ETAT } from '../../lib/verdict'

export function VerifyScene() {
  return (
    <div data-testid="verify-scene" className="flex min-h-0 flex-1 flex-col bg-s0">
      <div className="flex items-center gap-2.5 border-b border-bd px-5 py-3">
        <span className="h-1.5 w-1.5 rounded-full bg-info" aria-hidden />
        <span className="font-sans text-xs font-semibold tracking-wide text-tx">VERIFY</span>
        <span className="font-sans text-xs text-tx3">— examiner le diff · lire les gates · preuve</span>
        <span className="ml-auto font-mono text-[10px] text-tx4">la vérité = git diff, pas un buffer</span>
      </div>

      <Terminal />

      {/* permanent gates + état band — honest, never a verdict */}
      <div className="flex items-stretch border-t border-bd2 bg-s2">
        <div className="flex flex-1 items-center gap-2 px-4 py-2.5 font-mono text-[10.5px] text-tx4">
          <span className="font-sans text-[8.5px] font-semibold uppercase tracking-wider text-tx4">gates</span>
          <span title="câblage Phase G">non câblées — Phase G</span>
        </div>
        <div className="flex items-center gap-3 border-l border-bd2 bg-s3 px-4 py-2.5">
          <span className="font-mono text-[8px] font-semibold uppercase tracking-wide text-info">état</span>
          <span data-testid="verify-etat" className="font-mono text-[11px] tabular-nums text-tx2">
            {VERIFY_ETAT.bootstrap}
          </span>
        </div>
      </div>
    </div>
  )
}
