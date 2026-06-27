// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — VERIFY is a placeholder here: the MODE toggle (D6) is
// delivered now, but the full VERIFY surface (diff-viewer bespoke + gates
// panel + sealed preview + provenance) is Phase H. We render the honest
// frame — the permanent bottom band whose ÉTAT slot NEVER says "PASS" and
// the gates pulse that is not wired before Phase G — so the state-driven
// scene is real even while its body is deferred. No verdict is fabricated.

export function VerifyPlaceholder() {
  return (
    <div data-testid="verify-scene" className="flex min-h-0 flex-1 flex-col bg-s0">
      <div className="flex items-center gap-2.5 border-b border-bd px-5 py-3">
        <span className="h-1.5 w-1.5 rounded-full bg-info" aria-hidden />
        <span className="font-sans text-xs font-semibold tracking-wide text-tx">VERIFY</span>
        <span className="font-sans text-xs text-tx3">— examiner le diff · lire les gates · preuve</span>
        <span className="ml-auto font-mono text-[10px] text-tx4">la vérité = git diff, pas un buffer</span>
      </div>

      <div className="flex flex-1 items-center justify-center p-7">
        <div className="max-w-prose rounded-md border border-dashed border-bd bg-s1 px-6 py-6 text-center">
          <div className="font-sans text-sm text-tx2">
            La surface VERIFY complète — visualiseur de diff, panneau de gates, aperçu scellé,
            preuve — est livrée en Phase&nbsp;H.
          </div>
          <div className="mt-2 font-mono text-[10.5px] text-tx4">
            le diff est la vérité de Rust ; l'Operator ne calcule aucun verdict
          </div>
        </div>
      </div>

      {/* permanent gates + état band — honest, never a verdict */}
      <div className="flex items-stretch border-t border-bd2 bg-s2">
        <div className="flex flex-1 items-center gap-2 px-4 py-2.5 font-mono text-[10.5px] text-tx4">
          <span className="font-sans text-[8.5px] font-semibold uppercase tracking-wider text-tx4">
            gates
          </span>
          <span title="câblage Phase G">non câblées — Phase G</span>
        </div>
        <div className="flex items-center gap-3 border-l border-bd2 bg-s3 px-4 py-2.5">
          <span className="font-mono text-[8px] font-semibold uppercase tracking-wide text-info">état</span>
          <span className="font-mono text-[11px] tabular-nums text-tx2">
            En attente de session agent · 0 verdict auto-clos
          </span>
        </div>
      </div>
    </div>
  )
}
