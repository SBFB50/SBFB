# Staging — Kickoff S81 « Upgrade iroh 1.0 » (CONSOMMÉ — ACTIVÉ le 2026-07-02)

> Préparé le 2026-06-27 pendant que S80 était en cours. **ACTIVÉ le 2026-07-02**
> (Cas C) : `sprint81_kickoff.md` + `sprint81_plan.md` + `sprint81_design_review.md`
> ont été `git mv` vers `.planning/active/` avec les décisions PO actées
> (C1 sharding INCLUS au T2 — Phases I/J ; C4/C5 assoupli « personne sur le
> réseau » ; C2/C3/C6/C7 recos confirmées). Restent ici : ce README (provenance),
> `sprint81_dossier.md` (dossier canonique de recherche) et
> `verification_2026-07-02.md` (vérification du staging).

## Contenu (post-activation)
- `sprint81_dossier.md` — dossier canonique (source de recherche des 3 artefacts activés).
- `verification_2026-07-02.md` — vérification ultracode du staging.
- Artefacts actifs : `.planning/active/sprint81_{kickoff,plan,design_review}.md`.

## Provenance
Workflow ultracode `wf_523263ad-82e` (11 agents Opus 4.8 1M, ~1,16M tokens) :
5 lecteurs recherche + G1 board + sceptique adversarial + synthèse + 3 rédacteurs.
Ancré sur les recherches GuardianDB (`wf_03124107-dd9`/`wf_ccea7a9b-6f8`/`wf_7f37fec1-5a5`/`wf_cfa08123-8c8`),
voir `.planning/research/guardian_db_integration_eval.md`.

## Procédure d'activation (quand S80 ferme)
1. S80 atteint DONE (wrap-up + push).
2. Ouvrir S81 en **Cas C** : la Phase 0 = **audit gate S80** (joue d'abord ; son verdict
   fige la liste exacte des carries entrants + la baseline de tests).
3. **PO confirme les arbitrages C1..C7** (cf. kickoff §Arbitrages PO) — surtout :
   - C1 : DONE non-PROVISIONAL scopé sur l'axe **transport-convergence** ; sharding hors T2 → S82.
   - C3 : version cible = `=1.0.0` maintenant + re-pin sur la 1re `1.0.x` avant push live.
   - C4/C5 : migration données-live IN-PLACE + self-heal `runtime.rs:2515` **neutralisé** pendant la migration (PAS un backstop).
4. `git mv` les 4 fichiers vers `.planning/active/` (renommer le dossier au passage),
   actualiser les références « DRAFT/provisoire », ré-jouer un preflight Phase A.
5. Le corps A→I ne démarre qu'après Phase 0 PASS.

## Rappel décisionnel
- L'upgrade se justifie **sur ses propres mérites** (forcing function relais N0 EOL **2026-09-30**),
  **pas** pour adopter une dépendance tierce.
- Il **ne franchit PAS Gate 1** (R-iroh-audit P0 inchangé, 0 audit tiers iroh 1.0) et **ne débloque pas** le pilote public.
- iroh **strictement seul** (anti-bundle) ; bisectabilité = invariant du sprint.
