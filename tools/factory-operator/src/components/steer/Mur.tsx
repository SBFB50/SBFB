// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the MUR, restituted INLINE when the backend returns
// `requires_gate` (a sensitive intention: shell / commit / push / valider).
// The front RESTITUTES the barrier — it never decides it: the keyword gate
// lives in the backend (SENSITIVE_ACTIONS, operator_server.rs:37), runs
// BEFORE any spawn, and no front pre-filter is "smarter". There is ZERO
// "Forcer / Override / Bypass" affordance — the only control is going back.
//
// This is the inline restitution; the full-width MUR with the "Préparer le
// pack" forward action lands in Phase D. The amber is gravity, not decor.

export function Mur({ message, onBack }: { message: string; onBack: () => void }) {
  return (
    <section
      data-testid="mur"
      role="alert"
      aria-label="Barrière de gouvernance"
      className="border-y border-mur bg-mur-bg px-6 py-5"
    >
      <div className="flex items-start gap-5">
        <div className="mt-0.5 flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-md border-2 border-mur text-mur" aria-hidden>
          {/* lock glyph */}
          <span className="font-mono text-base leading-none">⚿</span>
        </div>
        <div className="min-w-0 flex-1">
          <div className="mb-2 font-mono text-[9px] font-semibold uppercase tracking-[0.1em] text-mur">
            barrière de gouvernance
          </div>
          <div className="mb-2 font-sans text-[15px] font-semibold leading-snug text-tx">
            Cette intention exige une vraie session agent.
          </div>
          <p className="max-w-prose font-sans text-[12.5px] leading-relaxed text-tx2">
            {message} Commit, push, shell et validation ne s'exécutent jamais depuis le composeur :
            ils passent par une session tracée qui produit gates et preuves — c'est la barrière, pas
            un bouton à franchir. La préparation du pack pour la session arrive à l'étape suivante.
          </p>
          <div className="mt-3 font-mono text-[9.5px] text-mur">
            — aucun « Forcer » · aucun « Override » · aucun « Bypass » · aucun « Exécuter quand même » —
          </div>
          <div className="mt-4">
            <button
              type="button"
              data-testid="mur-back"
              onClick={onBack}
              className="rounded-sm border border-bd2 px-4 py-2 font-sans text-[12px] font-medium text-tx2 hover:border-bd2 hover:text-tx"
            >
              Retour à la composition
            </button>
          </div>
        </div>
      </div>
    </section>
  )
}
