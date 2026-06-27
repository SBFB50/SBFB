// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the Knowledge inspector (J15) + the sealed context-pack
// (folds S2 + D2). It is also where "Préparer le pack" lands from the MUR (the
// non-authoritative handoff brouillon, J13). The knowledge packs (animejs +
// daisyui, fold D1 backend) are CONSUMED, never authoritative: hashed path
// references, dashed mono chips, read-only — the Operator displays provenance,
// it grants nothing (decision S79-D6). The full sealed pack + hash-drift
// markers live in <ContextPackInspector>.
import { ContextPackInspector } from '../verify/ContextPackInspector'

export function KnowledgeSurface({ sessionId }: { sessionId: string | null }) {
  return (
    <div data-testid="knowledge-surface" className="flex min-h-0 flex-1 flex-col overflow-auto p-5">
      <div className="mb-4 rounded-md border border-dashed border-bd2 bg-s1 px-4 py-3">
        <div className="mb-1 font-sans text-[12px] font-semibold text-tx">Connaissance consultative</div>
        <p className="font-sans text-[11.5px] leading-relaxed text-tx2">
          Les packs de connaissance (animejs, daisyui) et les documents de procédé sont consommés par
          une session agent, jamais autoritaires. On en montre la provenance — chemin + empreinte
          blake3 — pas le contenu : une référence vérifiable, pas une vérité gravée par l'Operator.
        </p>
        <div className="mt-2 font-mono text-[9.5px] text-tx4">
          fraîcheur ≠ verdict · une empreinte qui dérive signale « relu », pas « approuvé »
        </div>
      </div>

      <ContextPackInspector sessionId={sessionId} />
    </div>
  )
}
