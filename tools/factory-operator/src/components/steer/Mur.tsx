// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C → D — the MUR, restituted when the backend returns
// `requires_gate` (a sensitive intention: shell / commit / push / valider).
// The front RESTITUTES the barrier — it never decides it: the keyword gate
// lives in the backend (SENSITIVE_ACTIONS, operator_server.rs:37), runs
// BEFORE any spawn, and no front pre-filter is "smarter". There is ZERO
// "Forcer / Override / Bypass" affordance.
//
// Sprint 80 Phase D — the wall now carries its ONE forward affordance:
// "Préparer le pack" (onPrepare) opens the sealed context-pack to hand off to
// a real agent session. It is the only way past the wall, and it is NOT
// "execute" — it is a handoff that produces gates and proofs. The amber is
// gravity, not decor.

export function Mur({
  message,
  onBack,
  onPrepare,
}: {
  message: string
  onBack: () => void
  onPrepare?: () => void
}) {
  return (
    <section
      data-testid="mur"
      role="alert"
      aria-label="Barrière de gouvernance"
      // confirmation gravity (signature 5): the wall enters with weight — the
      // physics IS the meaning (the consequence has mass). CSS-only (transform
      // keyframe `.motion-gravity`, index.css) so the eager STEER scene carries
      // no Motion-lib weight; transform-only ⇒ instant under reduced-motion.
      className="motion-gravity border-y border-mur bg-mur-bg px-6 py-5"
    >
      <div className="flex items-start gap-5">
        <div className="mt-0.5 flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-md border-2 border-mur text-mur" aria-hidden>
          {/* lock glyph */}
          <span className="font-mono text-base leading-none">⚿</span>
        </div>
        <div className="min-w-0 flex-1">
          <div className="mb-2 font-sans text-meta font-semibold text-mur">
            barrière de gouvernance
          </div>
          <div className="mb-2 font-sans text-scene font-semibold leading-snug text-tx">
            Cette intention exige une vraie session agent.
          </div>
          <p className="max-w-prose font-sans text-body leading-relaxed text-tx2">
            {message} Commit, push, shell et validation ne s'exécutent jamais depuis le composeur :
            ils passent par une session tracée qui produit gates et preuves — c'est la barrière, pas
            un bouton à franchir.
          </p>
          <div className="mt-3 font-mono text-meta text-mur">
            — aucun « Forcer » · aucun « Override » · aucun « Bypass » · aucun « Exécuter quand même » —
          </div>
          <div className="mt-4 flex flex-wrap items-center gap-2.5">
            {onPrepare ? (
              <button
                type="button"
                data-testid="mur-prepare"
                onClick={onPrepare}
                title="ouvrir le pack scellé à transmettre à une vraie session agent"
                className="rounded-sm border border-mur bg-mur/10 px-4 py-2 font-sans text-body font-semibold text-mur hover:bg-mur/20"
              >
                Préparer le pack
              </button>
            ) : null}
            <button
              type="button"
              data-testid="mur-back"
              onClick={onBack}
              className="rounded-sm border border-bd2 px-4 py-2 font-sans text-body font-medium text-tx2 hover:border-bd2 hover:text-tx"
            >
              Retour à la composition
            </button>
          </div>
        </div>
      </div>
    </section>
  )
}
