# Sprint 83 — Phase L review (round r3) — Permissions et atténuation

## Verdict: PASS-PENDING

Round r3 = réconciliation après **deux rounds Codex GPT-5.6 Sol** (r1, r2)
sur l'arbre corrigé. Codex a trouvé **deux P1 réels que la review Workflow
8-agents avait manqués** — validant la gate externe :
- **P1 r1** : `join_grant_bounds` (LUB champ-par-champ) fabriquait une capacité
  « Frankenstein » plus permissive que chaque grant (`{a}`/100/Human +
  `{b}`/1/Auto → `{a,b}`/100/Auto : autorise `a` sans HITL et 100 appels vers `b`).
- **P1 r2** : le correctif r1 (`combine_same_audience`, fold pairwise gauche)
  restait dépendant de l'ordre pour TROIS grants (`{a}`,`{b}`,`{a,b}` :
  `[C,A,B]`→C survit, `[A,B,C]`→drop).

**Fermeture finale** : `dominating_grant` — parmi toutes les bornes grantées
d'une ref, garde l'UNIQUE MAXIMUM que toutes les autres raffinent (une borne
réellement grantée, jamais un LUB synthétique) ; sinon DROP fail-closed.
Calcul sur l'ENSEMBLE (pas de fold), donc **permutation-invariant** — prouvé
par 6 permutations de 3 grants dans `multi_grant_same_audience_combines_soundly`
et par mutation (le left-fold rougit). Reste 0 P0/P1 après re-vérification.

`PASS-PENDING` jusqu'au verdict Codex r3 sur ce dernier correctif.

## La phase en une ligne

Module NEUF `crates/sbfb-workflow-engine/src/capability.rs` (~900 l + **19
tests**) : `effective_capabilities` = intersection monotone des 7 couches sur
17 classes typées, en miroir discipliné (jamais appelé) de `grant_v0`. DS-1 :
module PUR, **0 call-site** dans le dispatch.

## Défauts trouvés par Codex (r1+r2) et fermés

### P1 (fermés)

- **P1-r1 LUB Frankenstein** → FERMÉ. `join_grant_bounds` supprimé.
- **P1-r2 fold non permutation-invariant** → FERMÉ. `dominating_grant` = max
  global de l'ordre de raffinement, order-indépendant. Test 6-permutations +
  mutations left-fold/always-first tuées.

### P2 (fermés)

- **HITL order (r1)** : test réécrit en 3 cas ISOLÉS (élévation par local
  SEUL / release SEUL / pack SEUL). Mutation « gate entre release et pack »
  tuée.
- **EffectNeed vacuous (r1)** : `effect_need_narrows_to_single_surviving_ref`
  (2 refs survivent 1-5, `One("a")` → seul a). Mutation « supprimer le filtre »
  tuée.
- **class-unknown (r1/r2)** : garde d'exhaustivité `ALL` (match sans wildcard).
  T-CAP-CLASS-UNKNOWN-FAILOPEN re-qualifiée **bornée** (frontière de résolution
  String→BoundedCapability = reliure DS-1) — cohérent dans la row ET l'historique
  v27.
- **docs « max data-policy » périmées (r1)** : corrigées en « union de flags »
  partout (module doc, llms.txt, THREAT). Asset A13 corrigé.
- **tests négatifs `refines` manquants (r2)** : `refines_negative_cases_pinned`
  (Ceiling/DestinationBound/EffectiveCapabilities — extra key, borne plus lâche).
  Mutations `refines→true` tuées.
- **audience pack→release (r1)**, **décision HITL par-ref (r1)**, **ordre
  ApprovalLevel + DataPolicy::refines (r1)** : cas ajoutés, mutations tuées.
- **docs T-CAP-WIDEN-BYPASS / llms (r2)** : réécrites en sémantique
  « domination-globale-ou-drop permutation-invariante » (plus « order-indépendant »
  sans qualification).
- **review canonique stale (r2)** : ce fichier r3 remplace le r2 périmé
  (qui affirmait `DataPolicy::join`, 9 mutations, 15/15).

### P2 PARTIEL / P3 (documentés, non-code)

- **Contrat de reliure V (r2 P2)** : §Phase V.1 du plan étendu — 7 exigences
  explicites (recompute par-attempt ; refus journalisés ; résolveur
  inconnu⇒refus ; provenance EffectNeed ; validation effet réel vs borne ;
  unicité-ou-erreur multi-grants ; distinction API candidat/résultat).
- **compile.rs:606-610 (r1/r2 P3)** : champ schemars LAISSÉ byte-intact
  (préserve le golden core) — assumé comme **dette possédée par V** (inscrit
  §V.1). Le texte « per-effect binding is Phases J/K/L » devient exact à la
  reliure.
- **depth_enforced PARTIEL (r2)** : fold de meets génériques, pas le pipeline
  7-couches — la monotonie du pipeline réel EST couverte par
  `intersection_only_attenuates` (témoins non-vacuité assertés). Documenté.
- **Drop silencieux multi-grants incomparables (r2)** : borne honnête — la
  reliure DS-1 doit le surfacer comme erreur de config (noté module doc +
  §V.1 + THREAT).

## Points vérifiés OK (Codex CONFIRME + review)

- **Meet cœur** : ∩ destinations, min quota/timeout, max approval, UNION
  DataPolicy (flags orthogonaux). Aucun widening : constructeur unique
  `from_declared`, 0 insert/widen public, `inner` privé, agrégation
  multi-grants = borne réellement grantée re-bornée par le meet externe.
- **18 mutations discriminantes tuées** + garde d'exhaustivité `ALL`
  compile-time.
- **Expiry/revoke** miroir exact de `grant_v0` (`now_ms >= exp`, per-call) ;
  engine ne dépend PAS de `sbfb-manifest`.
- **DS-1** : 0 call-site hors tests ; `EffectContext` non touché.
- **Charte** : 0 serde derive, 0 I/O, trait `CapabilityPolicy` injecté.
- **Pré-launch** : 0 bump wire (4 versions chantier à 1), census DOMAIN 29,
  `contracts.rs` len==7, `contracts.schema.json` byte-intact, 0 dep neuve.

## Suites §7.4

- Gate G-CAPABILITY = **19/19 vert**.
- Workspace Windows + Docker sbfb-ci relancés sur l'arbre post-Codex-r2 —
  résultats à réconcilier avant commit (Docker engine+core 246 confirmé sur
  l'état intermédiaire ; re-run complet après le fix P1-r2).

## Codex reconciliation

Rounds `rounds/sprint83_phase_l_codex_review_r1.md` (P1 LUB + 5 P2 + 3 P3) et
`_r2.md` (P1 permutation + 3 P2 + 1 PARTIEL + 1 P3). Tous les P0/P1/P2 code
fermés ; les PARTIEL/P3 sont des contrats de reliure (V) et une dette golden
documentée. Round r3 Codex à lancer sur ce correctif final (critère d'arrêt :
CLEAN ou P2/P3 documentés).

## Décision

0 P0/P1 après r1+r2. Promotion PASS conditionnée à Codex r3 clean +
réconciliation §7.4 dual-platform.
