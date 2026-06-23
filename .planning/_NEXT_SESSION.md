# ⇨ ENTRÉE PROCHAINE SESSION — lire EN PREMIER

> Posé le 2026-06-23 (session découverte hors-process). Ce fichier route la
> prochaine session après le bootstrap canonique (`docs/claude/README.md` §0 + §7.1).
>
> **MAJ 2026-06-23 (session S77 L/M/N)** : S77 a été ROUVERT par avenant (3 phases
> doc L/M/N, cf. `.planning/active/sprint77_plan.md` §20). Tant que S77 n'est pas
> CLOS, le package S79 ne peut pas vivre dans `.planning/active/` (convention
> « active/ = 1 seul sprint » + il casse `operator_sprint_history_endpoint`). Il a
> donc été relocalisé à la **racine `.planning/`** (intact, untracked). Au vrai boot
> S79 (après clôture S77 + audit gate), remettre le package dans `active/`.

## Directive PO (de cette session)
**Démarrer le sprint Factory « app-authoring » (S79) à la prochaine session et rendre
Factory opérationnel.** Tout est déjà conçu, durci et planifié — il n'y a rien à
re-concevoir, seulement à **exécuter**.

## Ce qui est PRÊT (relocalisé à la racine `.planning/`, à remettre dans `active/` à l'ouverture S79)
- `sprint79_factory_kickoff.md` — objectif, scope (A→G), assets, contrat CSP, Day-0 gelé.
- `sprint79_factory_plan.md` — Phase 0 (audit gate) + phases A→G + gate de testabilité (T1 E2E + T2 JSON) + titres de commit.
- `sprint79_factory_design_review.md` — 8 questions tranchées (preuve / reco / PO), contrat CSP, risques.

## Les assets (déjà construits, à relocaliser par les phases A/E)
- Pack **anime.js** : `examples/daisyui-animejs-showcase/knowledge/` (93 primitives, 419 pages doc, synthesis, types).
- Pack **daisyUI** : `examples/daisyui-animejs-showcase/knowledge/daisyui/` (68 composants, 35 thèmes, `MANIFEST.json` + hashes, `docs-llms.txt`).
- Design : `…/knowledge/factory-integration-design.md` + `…/factory-integration-hardened.md`.

## Routage bootstrap
Le **kickoff + plan existent déjà** → ce n'est pas un Cas C « concevoir » mais un
quasi-Cas B « exécuter » : lire `sprint79_factory_kickoff.md` + `sprint79_factory_plan.md`,
faire le **pre-flight Phase 0 = audit gate** du dernier sprint réellement CLOSED, puis
dérouler A→G (sprint ultra-complet, 0 defer du cœur). Capacité = module de connaissance
versionné + prompt-kind `app-authoring` + gate CSP déterministe Rust (importe `BLOB_SERVE_CSP`).

## ⚠️ ARBITRAGE PO REQUIS AU BOOT — ordre vs S78 sharding
Ce sprint Factory est **orthogonal au compute**. Or l'état canonique dit « **S78 à ouvrir**
= orchestrateur de session sharding in-vivo + benchmark live » — un **carry P1 PROVISIONAL**,
avec `sprint78_audit_plan.md` déjà posé (qui cible l'audit de S77). Démarrer Factory à la
prochaine session **précède donc S78**.

**À confirmer par le PO au démarrage :**
- (a) **Factory d'abord** (directive de cette session) → on diffère le carry sharding S78. Phase 0 Factory = audit gate de **S77** (réutiliser/adapter `sprint78_audit_plan.md` qui cible déjà S77).
- (b) **S78 sharding d'abord** (fermer le carry P1) → Factory devient S79 après. Le package Factory reste prêt, intact.

Numérotation « S79 » = étiquette de slot du design durci ; l'ordre réel d'exécution est l'arbitrage ci-dessus. Ne pas trancher sans le PO.

## Invariants du sprint Factory
0 bump wire · 0 dépendance nouvelle · scellage 100% Factory intact · gate CSP **bloquant**
dès son introduction · connaissance **consommée jamais autoritaire** (aucun verdict PASS) ·
versions figées daisyUI 5.5.23 / Tailwind 4.3.1 / anime 4.5.0.
