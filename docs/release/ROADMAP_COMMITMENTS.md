# Roadmap commitments long-terme

Cree : 2026-04-19 (Sprint 21, session fraiche).
Regle de gestion : `docs/claude/README.md` §6.2.1 « Carry-overs,
escalade et dette (G7) » — amendement 2026-04-24 : escalade a 3
reports, phase dette sprints pairs, items < 500 LOC non reclassifiables.

## Preambule

Ce fichier est le registre des **engagements long-terme** du projet
nexus-grid / SBFB — distincts des carry-overs courts-termes du cycle
de sprint.

- **Carry-over court-terme** : dette resorbable identifiee en Phase F
  wrap-up, consignee dans `sprint{N+1}_carry_summary.md`, soumise au
  escalade G7 (3 reports = obligatoire, cf. §6.2.1 Regle 2). Destinee
  a etre livree dans les sprints suivants.
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
| LT-1 | Kudos-v2 fairness reform (log-utility + DRF + EMA fitness)    | **reclassifie pre-v1.0** | S50 | 2026-04-30 |
| LT-2 | Meta-1 Radicle-v1.0 activation tracking (flip Codeberg→Radicle) | latent   | `<post-v1.0>`  | 2026-04-19    |
| LT-3 | Contribution family Sybil matrix (3 couches asymetriques post-v1.0) | latent | `<post-v1.0>`  | 2026-04-20    |
| LT-4 | OS biometric gate cross-platform (Windows Hello / TouchID / polkit) | latent | `<post-v1.0>` | 2026-04-20    |
| LT-5 | Redundancy persistence SQLite + wire-up prod               | latent     | `<post-v1.0>`  | 2026-04-22    |
| LT-6 | iroh neighborhood enrichment                                | **resolved** | Sprint 32      | 2026-04-27    |
| LT-7 | Self-hosted build — le reseau compile le reseau             | **pre-v1.0 obligatoire** | `<S54-S55>` | 2026-05-02 |

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
- **Condition de declenchement** : **RECLASSIFIE pre-v1.0**
  (decision utilisateur 2026-04-30 post-S44). Rationale : app
  trust/review S52 doit utiliser un signal de contribution
  corrige, pas le kudos v1 compute-only qui propage le Matthew
  effect 40x dans la gouvernance. Les conditions empiriques
  (c1/c2/c3) restent les seuils de monitoring post-deploy pour
  valider que la formule v2 resout effectivement le probleme.
  Sprint cible : **S50** (per roadmap_v1_migration_rust.md).
  Les anciennes conditions de declenchement (tag v1.0 + validation
  stakeholders + seuils Gini) deviennent des criteres de succes
  post-deploy, pas des prerequisites d'activation.
- **Owner** : S50 kickoff. Planification active.
- **Runbook pointer** : `docs/FAIRNESS_VISION.md` §« Direction
  produit pour Kudos v2 » (document cree en parallele de ce fichier
  le 2026-04-19). Contient la vision produit, le rationale academique
  des trois briques (log-utility / DRF / EMA fitness-aging), et la
  procedure d'activation quand la condition est declenchee.
- **Derniere revue** : 2026-04-30 (reclassification pre-v1.0).
  Recherches factuelles S21 toujours valides (< 2 semaines).
  Prochaine revue : S50 kickoff (G2 trigger sur research S21).

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

## LT-4 OS biometric gate cross-platform

- **ID** : LT-4
- **Title** : OS biometric gate cross-platform (Windows Hello /
  TouchID macOS / polkit Linux) appliqué sur ops critiques loopback
  (panic wipe, duress unlock, rotation token force, escalade consent
  tier `mes_projets → tous`, federation canary FROST cosign S30
  Niveau 1). Permet de bloquer malware user-mode avec browser
  compromise d'invoquer ces endpoints en DOM injection (gate
  OS-level non-forgeable par process unprivileged).
- **Origine** : Analyse `microsoft/sudo` 2026-04-20 — UAC utilisé
  comme single gate non-bypassable par process browser-level
  (absence de password, single OS-level gate). Identifié dans
  `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md
  §1 Cluster D (D4)` comme **feature différenciante populations
  Gate 4 cible** (journalistes/activistes/ONG face AD2 malware
  user-mode). Item net-new (pas issu d'un carry reclassifié) —
  entre directement dans ce registre via la voie 2 du préambule.
- **Condition de déclenchement** : les trois sous-conditions
  suivantes doivent être simultanément satisfaites pour réouvrir
  LT-4 comme carry actif :
  - (a) tag `v1.0` go-live posé sur master ;
  - (b) S30 Phase E FROST Niveau 1 enforcement livré (consumer
    natif du gate T2 `BIOMETRIC_GATE` sur endpoint `/canary/
    cosign`, cf. `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md
    §3`) ;
  - (c) partnership OpSec Amnesty/HRW/CPJ/EFF S30+ a signalé
    explicitement besoin biometric gate face adversary AD2 malware
    user-mode dans son retour sécurité (pattern symétrique LT-3
    condition c audit externe).
  Tant que l'une des trois n'est pas satisfaite, le commitment
  reste latent. La condition (c) est qualitative (retour expert
  partnership) mais factuelle (pas opinion — requiert trace écrite
  dans le registre outreach S28 Phase D + S30 partnership).
- **Owner** : `<post-v1.0>`. Sera remplacé par le handle du lead
  OpSec cross-platform au moment de la réactivation.
- **Runbook pointer** : `docs/security/LOOPBACK_ENDPOINTS_TRUST_
  TIERS.md §7 "Implémentation T2 BIOMETRIC_GATE (LT-4
  post-v1.0)"` — spécification cross-platform 3 OS (Windows Hello,
  TouchID macOS, polkit Linux) + workflow daemon T2 avec nonce
  + TTL + verrouillage après 3 échecs. Inclut 3 crates cibles :
  - Windows : `windows-rs 0.58+` `Windows.Security.Credentials.UI`
    namespace.
  - macOS : `security-framework 0.2+` ou `local-authentication-macos`.
  - Linux : `polkit-agent` binding ou `zbus` D-Bus direct.
  Spec cross-crate dans `docs/security/CAPABILITY_TOGGLES.md §2`
  table capability `biometric_gate` (gate-off by default).
- **Dernière revue** : 2026-04-20 (création de l'entrée). Prochaine
  revue attendue en Phase 0 audit du sprint qui détecte le premier
  trigger (a/b/c) franchi, ou en revue trimestrielle long-term
  commitments si aucun trigger ne se déclenche.

## LT-5 Redundancy persistence SQLite + wire-up prod

- **ID** : LT-5
- **Title** : Redundancy persistence SQLite + wire-up production
  (RedundancyDispatcher in-memory → SQLite WAL + wire collect_result
  dans le result path du dispatcher).
- **Origine** : S23 Phase D `dc163ea` — `RedundancyDispatcher` livré
  avec `Task.redundancy_factor` mais instancié nulle part en production
  (`dispatcher.py` reçoit `None` par défaut). Le wire-up + persistence
  SQLite ont été déférés S24 → S25 → S26 (3 carry consécutifs). S25
  audit_plan a clarifié l'état réel : "RedundancyDispatcher existe
  mais n'est instancié nulle part en production".
- **Condition de déclenchement** : **premier déploiement multi-worker**
  OU **tag `v1.0` go-live**. Pre-v1.0, in-memory suffisant (0 node
  externe, pas de state à survivre un restart). Post-v1.0, les workers
  tiers nécessitent la redondance effective pour détecter les résultats
  spoofés (C-ResultSpoof threat).
- **Owner** : `<post-v1.0>`. Sera remplacé par le handle du lead
  compute au moment de la réactivation.
- **Runbook pointer** : `packages/nexus-coordinator/src/nexus_coordinator/
  redundancy.py` (code existant) + `sprint23_plan.md §Phase D` (spec
  originale wire-up + persistence). Quand activé : (1) instancier
  `RedundancyDispatcher` dans `dispatcher.py` __init__, (2) wire
  `collect_result` dans `on_result_received` hook path, (3) ajouter
  SQLite WAL persistence (pattern `quarantine_queue.py` S21 Phase D).
- **Dernière revue** : 2026-04-22 (reclassification S26 kickoff,
  §6.2.1 auto-trigger après 3+ carry consécutifs S24/S25/S26).

## LT-6 iroh neighborhood enrichment

- **ID** : LT-6
- **Title** : iroh neighborhood enrichment (enrichir le mécanisme
  de découverte de pairs via le neighborhood iroh pour améliorer la
  résilience réseau).
- **Origine** : S23 audit → carry S24 → S25 → S26 (3 carry
  consécutifs). Item identifié comme amélioration non-bloquante de
  la qualité du réseau P2P. L'enrichissement du neighborhood permet
  de diversifier les pairs découverts et de réduire la surface
  d'attaque Eclipse (B-Eclipse threat).
- **Condition de déclenchement** : **iroh release > 0.97** (avec API
  neighborhood améliorée) OU **tag `v1.0` go-live**. **RESOLVED** :
  iroh 0.98 déployé Sprint 32 Phase A (`90aff27`). Day 0 #3 pin levé.
  Les 4 crates iroh upgradés simultanément (iroh 0.98, iroh-docs 0.98,
  iroh-gossip 0.98, iroh-blobs 0.100).
- **Owner** : `<post-v1.0>`. Sera remplacé par le handle du lead
  réseau au moment de la réactivation.
- **Runbook pointer** : `crates/nexus-core-rs/src/` (code iroh
  existant) + `sprint23_audit_plan.md` (spec originale). Quand
  activé : enrichir les callbacks de découverte dans
  `nexus-shell-daemon-core` pour propager les pairs voisins et
  diversifier la topologie du réseau.
- **Dernière revue** : 2026-04-27 (resolved Sprint 32 Phase A).

## LT-7 Self-hosted build — le reseau compile le reseau

- **ID** : LT-7
- **Title** : Self-hosted build — SBFB compile SBFB via ses propres
  workers (task_type "build", redundancy quorum SHA256, reproducible
  builds). Le reseau qui ne peut pas se compiler lui-meme n'est pas
  un reseau de compute.
- **Origine** : Discussion 2026-05-02 (session S52). Decision
  utilisateur non-negociable : pre-v1.0 obligatoire. Le modele
  archive-zip + workers compute + redundancy_factor + release-attest.sh
  (SOURCE_DATE_EPOCH + SLSA provenance) fournissent deja 80% de
  l'infra. Item net-new (voie 2 preambule).
- **Condition de declenchement** : **PRE-V1.0 OBLIGATOIRE**. Pas de
  tag v1.0 sans self-hosted build operationnel. GHA reste comme
  "stage 0" bootstrap (premier build du premier noeud). Le reseau
  prend le relais pour les builds de routine.
- **Owner** : sprint dedie pre-v1.0 (estimatif S54-S55).
- **Runbook pointer** : a creer au sprint kickoff. Composants
  identifies :
  - `task_type: "build"` dans TaskEntry (extension wire format)
  - Worker build executor (cargo build dans sandbox)
  - Toolchain pinning par hash dans le task descriptor
  - Quorum SHA256 via redundancy_factor existant
  - Fallback GHA si quorum echoue (bootstrap residuel)
  Blockers identifies (research 2026-05-02) :
  - Rust reproducible builds en maturation (rust-lang/rust#129080)
  - Toolchain homogeneite cross-workers (pin Rust+LLVM+linker)
  - Cout reseau binaires 50MB+ × redundancy (acceptable)
  MVP : architecture homogene x86_64-linux d'abord, cross-platform
  apres validation.
- **Derniere revue** : 2026-05-10 (S58 CLOSED). Tier 1 (Woodpecker
  CI pipeline) + Tier 2 (build_executor.rs + quorum SHA256
  validation TaskStatus::AwaitingQuorum + redundancy=3) = **DONE
  S55**. "Operationnel" au sens pre-launch = code path E2E
  fonctionnel, capable de dispatcher build tasks et valider par
  consensus SHA256. Gate pre-v1.0 satisfait par Tier 2.
  **Tier 3 S60** : P2P infra validee (gossip 3 machines Win+VPS+Mac,
  API mutual discovery, task submit signee Ed25519). Worker quorum
  E2E (claim → execute → SHA256 consensus) carry post-tag : workers
  non deployes sur VPS/Mac dans le timebox S60. Gate pre-v1.0
  reste satisfait par Tier 2 (inchange).
  **Tier 3 diversite publique post-launch** : la decentralisation
  organique (N builders independants non-controles) ne peut exister
  qu'avec des nœuds tiers. Premiers users = premiers builders.

## Reservation IDs futurs

- **LT-8+** : IDs alloués dans l'ordre d'entrée au registre
  (reclassification carry ≥3 consecutives OU net-new identifié
  route § préambule voie 2), sans réutilisation des IDs libérés.
