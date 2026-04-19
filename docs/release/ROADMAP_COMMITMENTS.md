# Roadmap commitments long-terme

Cree : 2026-04-19 (Sprint 21, session fraiche).
Regle de gestion : `docs/claude/README.md` §6.2.1 « Cap carry-overs :
max 2 par sprint (G7) », sous-section « Reclassification long-term
commitments (amendement 2026-04-18, audit gate S20) ».

## Preambule

Ce fichier est le registre des **engagements long-terme** du projet
nexus-grid / SBFB — distincts des carry-overs courts-termes du cycle
de sprint.

- **Carry-over court-terme** : dette resorbable identifiee en Phase F
  wrap-up, consignee dans `sprint{N+1}_carry_summary.md`, soumise au
  cap G7 (max 2 carries par sprint). Destinee a etre livree dans le
  sprint suivant ou au maximum le sous-suivant.
- **Long-term commitment** : engagement conditionnel au declenchement
  d'un evenement externe (tag v1.0 go-live, release d'une dep amont,
  seuil empirique atteint en prod, CVE critique sur une dep cle). Vit
  ici, **hors cap G7**, et ne consomme pas la focus d'un sprint
  courant tant que sa condition de declenchement n'est pas realisee.

Un item entre dans ce fichier via deux voies :

1. **Reclassification automatique** : un carry-over present dans
   3 carry_summary consecutifs (cf. §6.2.1) est promu long-term
   commitment en Phase F wrap-up du sprint N+2 et sort du cap G7.
2. **Net-new identifie en cours de route** : un item ne comme
   engagement conditionnel long-terme (pas issu d'un carry), par
   exemple une refonte produit planifiee post-v1.0. Il est ecrit
   directement ici au moment de son identification, sans transiter
   par carry_summary.

Chaque entree contient les 7 champs definis en §6.2.1 (ID, Title,
Origine, Condition de declenchement, Owner, Runbook pointer,
Derniere revue).

## Index

| ID   | Title                                                         | Status     | Owner          | Last reviewed |
|------|---------------------------------------------------------------|------------|----------------|---------------|
| LT-1 | Kudos-v2 fairness reform (log-utility + DRF + EMA fitness)    | latent     | `<post-v1.0>`  | 2026-04-19    |

## LT-1 Kudos-v2 fairness reform

- **ID** : LT-1
- **Title** : Kudos-v2 fairness reform (combo log-utility + DRF +
  EMA fitness-aging, horizon post-v1.0). La formule courante
  `kudos = tokens × quality × trust` est multiplicative et produit
  un Matthew effect (concentration sur les workers les mieux
  equipes). La refonte vise une formule additive/bornee combinant
  utilite logarithmique (rendement decroissant du hardware haut de
  gamme), Dominant Resource Fairness (repartition equitable
  multi-ressources), et moyenne glissante exponentielle sur la
  fitness workers (anti-decrochage des nouveaux et petits noeuds).
- **Origine** : Discussion fairness orchestrateur 2026-04-19
  (session fraiche). Aucun commit encore — premiere trace factuelle
  dans les deux research outputs archives le meme jour :
  `.planning/research/S21_research_p2p_compute_scoring_systems.md`
  et `.planning/research/S21_research_fair_allocation_mechanisms.md`.
  Item net-new (pas issu d'un carry reclassifie) — entre directement
  dans ce registre via la voie 2 du preambule.
- **Condition de declenchement** : les trois sous-conditions
  suivantes doivent etre simultanement satisfaites pour reouvrir
  LT-1 comme carry actif (reintegration dans le cap G7 du sprint qui
  pose le declenchement) :
  - (a) tag `v1.0` go-live pose sur master ;
  - (b) design doc `docs/FAIRNESS_VISION.md` valide par les
    stakeholders (user + au moins un contributeur externe), acte par
    un commit de validation explicite dans l'historique ;
  - (c) au moins une des trois conditions empiriques suivantes
    verifiable dans le kudos ledger de production :
    - (c1) Gini coefficient kudos > 0.70 mesure sur 30+ workers
      actifs sur une fenetre de 30 jours glissants,
    - (c2) top-5% des workers captent > 50% du total kudos emis
      sur la meme fenetre,
    - (c3) correlation statistiquement significative entre le
      churn-rate workers et le hardware-tier (les petits workers
      decrochent plus vite que les gros).
  Tant qu'aucun des trois seuils (c1/c2/c3) n'est atteint, le
  reseau n'a pas encore le Matthew effect empiriquement mesurable
  et le commitment reste latent. Les trois seuils sont des
  indicateurs factuels, pas des opinions — ils se calculent sur
  le kudos ledger existant.
- **Owner** : `<post-v1.0>`. Sera remplace par le handle du lead
  fairness au moment de la reactivation. Pas d'owner assigne tant
  que la condition de declenchement n'est pas remplie (le cap G7
  n'est pas consomme).
- **Runbook pointer** : `docs/FAIRNESS_VISION.md` §« Direction
  produit pour Kudos v2 » (document cree en parallele de ce fichier
  le 2026-04-19). Contient la vision produit, le rationale academique
  des trois briques (log-utility / DRF / EMA fitness-aging), et la
  procedure d'activation quand la condition est declenchee.
- **Derniere revue** : 2026-04-19 (creation du registre). Aucun
  commit de revalidation posterieur — les recherches factuelles
  S21 `.planning/research/S21_research_p2p_compute_scoring_systems.md`
  et `.planning/research/S21_research_fair_allocation_mechanisms.md`
  sont confirmees fraiches au meme jour. Prochaine revue attendue
  en Phase 0 audit du sprint qui detecte le premier seuil (c1/c2/c3)
  franchi, ou en Phase F wrap-up du sprint qui pose le tag v1.0.

## Reservation IDs futurs

- **LT-2** : reserve a priori pour **Meta-1 Radicle-v1.0 activation
  tracking**. Actuellement en carry S18 → S19 → S20 → S21 (4e sprint
  consecutif, sur le fil de la reclassification §6.2.1). La
  reclassification deviendra effective en Phase F wrap-up du
  Sprint 21 : Meta-1 sortira de `sprint22_carry_summary.md` et une
  section detaillee `## LT-2` sera ajoutee ici, avec condition de
  declenchement « tag v1.0 go-live » et runbook pointer
  `docs/release/MIRROR_FALLBACK.md §3 "Flip sequence Codeberg →
  Radicle"`. Pas de section detaillee pour LT-2 tant que la
  reclassification Phase F S21 n'est pas effective — l'ID est
  simplement reserve pour eviter une collision.

Les IDs suivants (LT-3, LT-4, ...) seront alloues dans l'ordre
d'entree au registre, sans reutilisation des IDs liberes.
