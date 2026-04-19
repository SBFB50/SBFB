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
| LT-2 | Meta-1 Radicle-v1.0 activation tracking (flip Codeberg→Radicle) | latent   | `<post-v1.0>`  | 2026-04-19    |

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

## LT-2 Meta-1 Radicle-v1.0 activation tracking

- **ID** : LT-2
- **Title** : Meta-1 Radicle-v1.0 activation tracking (flip
  Codeberg mirror → Radicle primary post go-live).
- **Origine** : S18 Phase E3 `95807b1` scope-cut Radicle-v1.0
  (Radicle P2P public-only incompatible repo GitHub privé pré-
  launch). Carry S18 → S19 → S20 → S21 = 3 consecutives (règle
  §6.2.1 trigger). **Rattrapage reclassification en Phase F
  S21 oublié**, régularisé au kickoff S22 (2026-04-19) via G1
  acknowledgement P3-G1-7 + audit gate S21 PASS confirmé
  (`96a953b`). Entre directement via voie 1 (reclassification
  automatique).
- **Condition de declenchement** : **tag `v1.0` go-live posé sur
  master**. Déclencheur unique. Au moment du tag :
  - (a) Réouvrir Meta-1 comme carry actif dans le sprint qui pose
    le tag (reintegration G7 cap du sprint concerné).
  - (b) Exécuter flip sequence `docs/release/MIRROR_FALLBACK.md §3
    "Flip sequence Codeberg → Radicle"`.
  - (c) Activer Radicle Heartwood node primary + DHT pkarr 3-
    quorum 2/3 (S18 Phase C) + warrant canary FROST K-of-N (S20
    Phase E + S30 FROST Niveau 1).
- **Owner** : `<post-v1.0>`. Sera remplacé par le handle du lead
  release au moment du tag v1.0. Pas d'owner assigné tant que la
  condition de déclenchement n'est pas remplie.
- **Runbook pointer** : `docs/release/MIRROR_FALLBACK.md §3 "Flip
  sequence Codeberg → Radicle"` — procédure self-contained
  documentée depuis S18 Phase E3. Inclut :
  - Désactivation mirror GitHub privé
  - Publication seed nodes Radicle officiels (`iris.radicle.xyz`,
    `rosa.radicle.xyz` cités agent research 6 — à re-vérifier
    post-Heartwood 1.8.x vuln replay attack corrigée 2026-03-30).
  - Migration `docs/release/PKARR_RELAY_OPS.md §1-§7` pattern
    self-hosted docker image (S19 Phase E `2fd4d72`).
  - Re-calibration Radicle adoption vs S22 Couche 3 Radicle cross-
    validate (S25-S27 implem).
- **Derniere revue** : 2026-04-19 (régularisation reclassification
  kickoff S22 via §6.2.1 auto-trigger + audit gate S21 PASS).
  Prochaine revue attendue au sprint qui pose le tag `v1.0` OU
  lors d'une revue trimestrielle long-term commitments si tag
  retardé.

## Reservation IDs futurs

- **LT-3+** : IDs alloués dans l'ordre d'entrée au registre
  (reclassification carry ≥3 consecutives OU net-new identifié
  route § préambule voie 2), sans réutilisation des IDs libérés.
