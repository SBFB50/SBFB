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
| LT-3 | Contribution family Sybil matrix (3 couches asymetriques post-v1.0) | latent | `<post-v1.0>`  | 2026-04-20    |

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

## LT-3 Contribution family Sybil matrix

- **ID** : LT-3
- **Title** : Contribution family Sybil matrix (3 couches asymetriques
  compute+funding / code+docs / review+mod+design+a11y, post-v1.0).
  Le systeme kudos actuel (`packages/nexus-coordinator/src/nexus_
  coordinator/kudos.py:52-94`) mesure uniquement la contribution
  compute et repose sur le cout physique (GPU + electricite) comme
  Sybil defense naturelle. Si le ledger est etendu a d'autres familles
  (code, docs, review, moderation, design, accessibility), l'asymetrie
  de cout de production LLM-era (un agent genere 200 pages de docs par
  heure a cout marginal quasi-nul) casse la defense naturelle. La
  reponse consolidee = composition asymetrique de 3 patterns valides
  empiriquement 15-30 ans chacun : (A) compute+funding gardent la
  defense cost-based existante ; (B) code+docs via evaluator committee
  clawback pattern RetroPGF-style (Optimism 2022-2026) ; (C) review+
  moderation+design+accessibility via invitation Protocol-Guild-style
  time-weighted (Protocol Guild $100M+ cumule 2022-2025 zero capture,
  Apache/Debian/Linux MAINTAINERS 30+ ans). Design-only post-v1.0 —
  rien n'est implemente pre-v1.0.
- **Origine** : Discussion orchestrateur 2026-04-20 (hors-sprint S22,
  post Phase A commit `0bc499f`). Reaction utilisateur a la realisation
  factuelle que la defense cost-Sybil compute ne s'etend pas aux 5
  familles non-compute vulnerables LLM-farming. Synthese de 4 agents
  paralleles independants (sprint audit + OSS 15 systemes + academique
  17 sources ICLR/USENIX/IEEE S&P/ACM CHI + faisabilite codebase).
  Research doc trace : `.planning/research/S22_contribution_family_
  sybil_matrix.md` (commit `dbc4ceb`). Item net-new (pas issu d'un
  carry reclassifie) — entre directement dans ce registre via la voie 2
  du preambule.
- **Condition de declenchement** : au moins UNE des trois sous-
  conditions suivantes doit etre realisee pour reouvrir LT-3 comme
  carry actif (reintegration dans le cap G7 du sprint qui pose le
  declenchement) :
  - (a) tag `v1.0` go-live pose sur master **ET** au moins une app
    Gate 2 (TransLingua, FamilyScan, EHPAD-Lien) compte >= 3
    contributeurs non-compute reels actifs pendant 30j glissants ;
  - (b) `/diagnostic/fairness` endpoint (livre S23 Phase D per
    `docs/security/HARDENING_ROADMAP.md §3 S23`) reporte Gini > 0.70
    OR top-5% > 50% sur ledger compute de production, mesurable
    empiriquement ;
  - (c) audit externe Cure53/ToB S29 (`docs/security/HARDENING_
    ROADMAP.md §3 S29`) signale explicitement vulnerabilite
    contribution-family dans son rapport de findings.
  Tant qu'aucune des trois n'est satisfaite, le commitment reste
  latent. Le declenchement (b) est le plus probable : l'endpoint
  d'observabilite rend la condition factuellement mesurable des
  que Gate 2 produit de la data en production (fin S22+).
- **Owner** : `<post-v1.0>`. Sera remplace par le handle du lead
  contribution-family au moment de la reactivation.
- **Runbook pointer** : `.planning/research/S22_contribution_family_
  sybil_matrix.md` (research complet 40+ sources, Option F consolidee
  specifiee par couche) + `.planning/reserved/S31_contribution_
  families_kickoff.md` (sprint stub pre-rempli activable). Quand
  active, copier le stub dans `.planning/active/sprint{N}_kickoff.md`,
  amender sur la base des donnees empiriques collectees via
  `/diagnostic/fairness`, et executer le sprint dedie.
- **Derniere revue** : 2026-04-20 (creation de l'entree). Prochaine
  revue attendue en Phase 0 audit du sprint qui detecte le premier
  trigger (a/b/c) franchi, ou en revue trimestrielle long-term
  commitments si aucun trigger ne se declenche.

## Reservation IDs futurs

- **LT-4+** : IDs alloués dans l'ordre d'entrée au registre
  (reclassification carry ≥3 consecutives OU net-new identifié
  route § préambule voie 2), sans réutilisation des IDs libérés.
