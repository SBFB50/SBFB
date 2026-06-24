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
>
> **MAJ 2026-06-24 (session S77 Phase N)** : **Phase N committée `a795700` → avenant
> doc L/M/N COMPLET → S77 ENTIÈREMENT LIVRÉ** (cœur A-K + doc L-N). Il ne reste QUE
> l'audit gate pour clore S77. **ARBITRAGE PO TRANCHÉ : Factory-first (option a).**
> Vérifié orthogonal au compute (le plan S79 ne touche AUCUNE surface
> sharding/worker/coordinator ; 0 dépendance sharding↔authoring ; le seul gate réel
> du sharding live est le rig 2-machines, pas l'ordre — Factory-first achète même du
> temps pour le monter). **Prochaine session fraîche = Cas A / Phase 0 = audit gate
> S77** (`nexus-audit-gate` ou Workflow lit `sprint78_audit_plan.md` qui cible déjà
> S77 → produit `sprint77_audit_findings.md`), PUIS relocaliser le package S79 dans
> `active/` + dérouler Factory A→G. Le carry P1 sharding (S78, RIG-ABSENT T2) reste
> ouvert et tracké (sprint78_audit_plan + plan S79 l.156).

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

## ✅ ARBITRAGE PO RÉSOLU (2026-06-24) — Factory-first (option a)
Le PO a tranché : **Factory (S79) d'abord, sharding live (S78) après.** Le carry P1
sharding (orchestrateur de session in-vivo + benchmark live + 4 carries 3/3) est
**différé**, intact et tracké (`sprint78_audit_plan.md` + plan S79 l.156).

Vérifié sans risque technique (session S77 Phase N) : le plan S79 ne touche aucune
surface sharding/compute (knowledge packs + gate CSP `gates.rs`/`blob_serve.rs` +
templates) ; 0 dépendance sharding↔authoring (plan S79 A0-1 + invariants) ; aucune
dépendance dans les deux sens ; l'audit gate S77 tourne d'abord dans tous les cas ;
le sharding GPU est un investissement « pont » long terme (pas d'expiration). Seul
résiduel : le sharding reste PROVISIONAL plus longtemps (gated sur le rig matériel,
pas sur l'ordre) — honesty-gate CI armé pour empêcher tout claim « done » prématuré.

**Séquence prochaine session fraîche (Cas A puis Factory) :**
1. **Phase 0 = audit gate S77** (Cas A) : ingérer le diff complet S77 (cœur A-K +
   doc L/M/N), jouer les 9 tracks via `sprint78_audit_plan.md` (cible S77),
   produire `sprint77_audit_findings.md` (verdict PASS / CONDITIONAL / FAIL +
   commits fix(sprint77) pour P0/P1).
2. Audit PASS/CONDITIONAL levé → **git mv** S77 `active/` → `archive/v2.1/`.
3. **Relocaliser** `sprint79_factory_{kickoff,plan,design_review}.md` racine →
   `active/` ; dérouler Factory **A→G** (sprint ultra-complet, 0 defer du cœur).

Numérotation « S79 » = étiquette de slot ; Factory s'exécute avant le S78 sharding.

## Invariants du sprint Factory
0 bump wire · 0 dépendance nouvelle · scellage 100% Factory intact · gate CSP **bloquant**
dès son introduction · connaissance **consommée jamais autoritaire** (aucun verdict PASS) ·
versions figées daisyUI 5.5.23 / Tailwind 4.3.1 / anime 4.5.0.
