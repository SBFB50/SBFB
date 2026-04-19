# Fairness Vision — Kudos v2

**Date** : 2026-04-19
**Statut** : document vivant de vision produit long-terme
**Horizon** : post-v1.0 (probable v1.3 ou v2.0)
**Nature** : document de direction, pas un plan technique

---

## 1. Préambule

Ce document est un livrable de réflexion produit long-terme. Il ne
décrit pas une implémentation imminente, il ne propose pas de code
pour Sprint 21 (en cours) ni pour Sprint 22 (prévu). Il existe
précisément parce que le Sprint 22 contient une décision de
gouvernance qui, si elle est câblée telle qu'écrite aujourd'hui,
verrouillera définitivement un mode de distribution des droits
incompatible avec la promesse produit du projet. Nous posons ici
le cadre de référence auquel chaque future décision touchant kudos,
routage de tâches, ou droit de parole sur le réseau devra être
comparée.

Ce document est écrit **avant** tout commit de refonte. Conformément
à la discipline `documentation AVANT code` du workflow (§6.7), il
fixe le cap produit pour que les sprints d'ingénierie qui suivront
aient une cible claire.

---

## 2. Le constat produit

Le `README.md` du projet affirme :

> "Decentralized P2P compute network for LLM apps.
> No central server. No admin. Just protocol."

La formule de scoring actuelle, définie à
`packages/nexus-coordinator/src/nexus_coordinator/kudos.py:134`, est :

```
kudos = tokens × quality_factor × trust_multiplier
```

Cette formule est linéaire sur le débit hardware. Un worker équipé
d'une RTX 5090 produit ~200 tokens/seconde. Un Raspberry Pi en
produit ~5. Tout étant égal par ailleurs, le premier accumule
quarante fois plus de kudos que le second pour un volume de
tâches identique. Le dispatcher est, ou sera sous peu, biaisé vers
les workers au kudos élevé — plus de kudos signifie plus de tâches
routées, donc plus de kudos. C'est le Matthew effect : la rente
cumulative s'auto-alimente.

Cette mécanique contredit frontalement le positionnement annoncé.
Un protocole qui récompense linéairement le capital hardware
reproduit les hiérarchies du cloud centralisé qu'il prétend
remplacer. Il n'y a pas d'administrateur qui décide qui mérite des
ressources, certes — mais la formule le fait à sa place, par
construction mathématique. "No admin. Just protocol" devient
alors un slogan, pas une propriété vérifiable.

Le projet est AGPL-3.0. Il se positionne comme le F-Droid des apps
P2P. Le scoring doit être cohérent avec ce positionnement : mesurer
la contribution à un commun, pas la puissance brute louée.

---

## 3. Le Matthew effect, documenté et chiffré

La recherche déposée dans `.planning/research/` établit que la
dérive est systémique dans les réseaux compute P2P existants.

- **Bittensor**, acteur de référence du secteur, présente un Gini de
  stake mesuré à **0.9825** — quasi-monopole parfait. Le top 1 %
  capte 90 % du stake médian sur 64 subnets, 24 % des récompenses
  médianes. Les auteurs concluent à *"a clear misalignment between
  quality and compensation"*. Source :
  [arXiv 2507.02951](https://arxiv.org/html/2507.02951v1).
- **io.net** encode explicitement un `hardware multiplier` par modèle
  GPU, combiné à une réputation qui conditionne l'assignation : plus
  de jobs attribués aux mieux notés, boucle cumulative assumée.
  Source : [io.net block rewards](https://io.net/docs/guides/explorer/block-rewards-page).
- **BOINC** reconnaît sur son wiki que CreditNew *"is basically
  flawed"*. Einstein@Home a abandonné CreditNew pour un crédit fixe
  par tâche après constat que les gros contributeurs avaient
  *"swamped"* la distribution. Source :
  [Einstein@Home credits](https://einsteinathome.org/content/einsteinhome-credits).

À l'inverse, **Render Network** segmente ses charges en trois niveaux
(Trusted Partners, Priority, Economy) pour que le matériel entreprise
et le matériel amateur ne se concurrencent jamais sur la même file.
Source :
[Render tokenomics MTP](https://medium.com/render-token/rndr-tokenomics-update-multi-tier-pricing-mtp-338d5dea1d29).

Nous ne sommes pas condamnés à la trajectoire Bittensor. Des
précédents produit existent. Mais ils nécessitent un choix explicite.

---

## 4. Le cas de nexus-grid à travers les use cases

`docs/VISION_USE_CASES.md` liste quinze scénarios produit. Au moins
cinq sont directement incohérents avec la formule actuelle : le
contributeur local y est irremplaçable mais systématiquement
sous-valorisé face à un datacenter distant qui n'a aucune connexion
au contexte.

- **Use case 1 — Chantier offline**. Le laptop qui tourne sur le
  mesh WiFi du chantier est le **seul** à pouvoir exécuter la tâche
  d'analyse de photos. Un datacenter distant est inaccessible par
  construction. La formule linéaire le paie à la même échelle qu'un
  worker quelconque, alors qu'il porte 100 % de la valeur produite.
- **Use case 2 — Hôpital souverain**. Le GPU du service radiologie
  est irremplaçable : c'est lui, et pas un autre, qui a le droit
  légal de traiter les données patients. Sa rareté contextuelle vaut
  infiniment plus que son throughput mesuré.
- **Use case 5 — École sans bande passante**. Le seul PC GPU de la
  salle sert toute la classe. Il est unique dans son rôle. La formule
  le réduit à son débit en tokens, ignore qu'il est le point de
  passage obligé.
- **Use case 7 — IoT citoyen quartier**. Les Raspberry Pi qui
  mesurent la qualité de l'air **sont** la donnée. Un datacenter
  peut faire tourner le LLM d'alerte, mais ne produira jamais la
  mesure. La formule paie le compute d'inférence et ignore la
  contribution unique de capteur.
- **Use case 13 — Entraide quartier COVID**. Cinquante téléphones
  qui éditent l'inventaire en CRDT sur un mesh 2G. Chacun apporte
  son contexte local. Aucun ne sera jamais compétitif en tokens/s
  face à un H100 distant. La formule les rend invisibles.

Dans chacun de ces cas, la contribution irremplaçable n'est pas du
débit, c'est de la **présence au bon endroit**. La formule actuelle
ne sait pas mesurer ça.

---

## 5. La direction produit pour Kudos v2

Nous fixons quatre principes. Ils ne sont pas des spécifications
techniques. Ils sont des critères d'acceptation produit que toute
proposition d'implémentation devra satisfaire.

**Principe 1 — Mesurer la contribution irremplaçable, pas la
puissance brute.** Le kudos doit répondre à la question "si ce
worker n'avait pas été là, qu'aurait perdu le réseau ?" plutôt
qu'à "combien de tokens a-t-il produits ?". C'est le concept de
contribution marginale moyennée sur toutes les coalitions possibles
— connu en économie coopérative depuis Shapley (1953, prix Nobel
2012). Trop coûteux à calculer en ligne pour chaque tâche, mais
utilisable en audit batch pour recalibrer périodiquement.

**Principe 2 — Segmenter les files de tâches par contexte.** Un
Raspberry Pi et une RTX 5090 ne doivent **jamais se concurrencer
sur la même file**. Les tâches doivent être routées selon la
ressource dominante qu'elles consomment (VRAM, context window,
latence, bande passante, localité géographique). Un worker dominé
sur une dimension peut dominer sur une autre. C'est le pattern
Dominant Resource Fairness, implémenté en production par Apache
Mesos depuis plus d'une décennie.

**Principe 3 — Compresser la queue.** Sans toucher au routage, la
formule de scoring elle-même doit suivre des rendements décroissants.
Doubler le hardware ne doit pas doubler le kudos. À titre
d'illustration produit :

```
kudos_affiche = log(1 + kudos_brut / seuil)
```

Un worker cent fois plus puissant ne reçoit alors qu'environ 4,6 fois
plus de kudos, pas 100 fois plus. L'incentive à contribuer du
matériel performant est préservée (c'est toujours monotone croissant),
mais la queue de distribution est coupée mécaniquement.

**Principe 4 — Faire vieillir le trust.** Un worker qui a contribué
en Sprint 1 et plus depuis ne doit pas conserver sa rente de
pionnier. Le `trust_multiplier` doit être une moyenne mobile avec
demi-vie de l'ordre de quinze jours, pas un cumul perpétuel. Cela
protège des early movers qui capturent le réseau par simple ordre
d'arrivée.

---

## 6. Positionnement concurrentiel

| Acteur | Métaphore courte | Cohérence avec son manifeste |
|---|---|---|
| Bittensor | Wall Street du compute. Stake-first, Gini 0.98. | Cohérent avec sa tokenomics. |
| io.net | Marché boursier du GPU. Hardware multiplier assumé. | Cohérent avec sa thèse DePIN. |
| Akash | AWS en token. Pivot explicite vers Nodekeepers enterprise. | Cohérent avec son modèle. |
| SBFB aujourd'hui | Bittensor gentil. Manifeste "no admin" mais formule linéaire rich-get-richer. | **Incohérent**. |
| SBFB avec Kudos v2 | Service public décentralisé. F-Droid + AGPL + routage équitable. | Cohérent bout-en-bout. |

Le projet ne doit pas devenir la énième version moins bien financée
de Bittensor. Il existe parce qu'il raconte une autre histoire —
celle d'un bien commun protocolaire. Kudos v2 est ce qui rend cette
histoire vraie dans le code.

---

## 7. Le design-conflict du Sprint 22

`docs/security/HARDENING_ROADMAP.md` ligne 250 prévoit pour
Sprint 22 :

> "Kudos-weighted gossip admission (nodes >N kudos full voice,
> others read-only)"

Telle qu'écrite, cette ligne câble le Matthew effect dans la
gouvernance du réseau elle-même. Le raisonnement est simple : si le
droit de parole sur la couche gossip dépend du kudos cumulé, et si
le kudos cumulé dépend du hardware, alors les workers au hardware
haut de gamme obtiennent mécaniquement la parole et les autres sont
muets. La Sybil-resistance est réelle, mais le prix payé est une
ploutocratie protocolaire. Le `No admin. Just protocol` du README
devient alors "no admin, just the biggest GPUs decide".

Cette ligne doit être revisitée avant que Sprint 22 ne l'implémente.
Plusieurs voies alternatives garantissent la Sybil-resistance sans
utiliser le kudos cumulé comme droit de vote :

- **a) Ancienneté du node_id plus proof-of-work individuel.** La
  voix s'obtient par un coût en temps et en calcul dépensé
  personnellement, pas par accumulation de rente. Un Pi patient
  vaut autant qu'un datacenter pressé.
- **b) Multi-signal à la Gitcoin Passport.** Cumul de signaux
  hétérogènes (compte GitHub ancien, `SBFB.json` Keyoxide déjà
  présent depuis Sprint 14, BrightID optionnel). Aucun signal seul
  n'est suffisant, aucun n'est lié au hardware.
- **c) Une voix par projet auquel on a contribué, binaire.** Tu as
  livré au moins une tâche validée pour un projet = tu as une voix
  sur sa gouvernance. Pas plus, pas moins. Découple complètement
  la parole du volume.

Ces trois options ne sont pas exclusives. Elles peuvent se composer.
Aucune ne récompense le capital hardware comme source de légitimité
démocratique.

---

## 8. Horizon temporel

Cette vision ne déclenche aucune refonte dans Sprint 21, en cours
(rate-limit + PII SDK). Elle ne déclenche aucune refonte dans
Sprint 22, dont le focus est Sybil-resistance et détection runtime
de compute theft. Elle vise probablement une version post-v1.0, dans
la fenêtre v1.3 ou v2.0, quand le réseau aura suffisamment d'usage
pour que les choix de fairness soient calibrés sur des données
réelles plutôt que sur des hypothèses.

Entre aujourd'hui et cette refonte, la discipline est de **vigilance**.
Chaque D-décision future qui touche au scoring, au routage, ou à la
gouvernance doit être revue contre les quatre principes ci-dessus
avant validation. Ce document est la référence de ces revues. Si
une décision les viole, elle doit être justifiée explicitement ou
réécrite.

---

## 9. Engagement long-terme enregistré

Cet engagement est tracé dans
`docs/release/ROADMAP_COMMITMENTS.md` sous l'entrée
`LT-1 Kudos-v2 fairness reform`. Ce fichier est le registre unique
des commitments long-terme du projet et sert de garde-fou lors des
audits trimestriels.

---

## 10. Références

**Sources projet :**
- `README.md:3-4` — manifesto "No central server. No admin. Just protocol."
- `docs/VISION_USE_CASES.md:347-348` — pattern commun "Distribution d'apps + état partagé + compute local, sans cloud, sans compte, sans abonnement"
- `.planning/research/S21_research_p2p_compute_scoring_systems.md` — revue de 13 projets P2P compute
- `.planning/research/S21_research_fair_allocation_mechanisms.md` — dix familles de mécanismes fairness
- `docs/security/HARDENING_ROADMAP.md:246-262` — Sprint 22, ligne conflictuelle
- `packages/nexus-coordinator/src/nexus_coordinator/kudos.py:134` — formule actuelle

**Sources académiques :**
- Shapley, L. S. (1953). *A Value for n-person Games*. Contributions to the Theory of Games II. Prix Nobel d'économie 2012.
- Ghodsi, A., Zaharia, M., Hindman, B., Konwinski, A., Shenker, S., Stoica, I. (2011). *Dominant Resource Fairness: Fair Allocation of Multiple Resource Types*. NSDI 2011. [PDF AMPLab Berkeley](https://amplab.cs.berkeley.edu/wp-content/uploads/2011/06/Dominant-Resource-Fairness-Fair-Allocation-of-Multiple-Resource-Types.pdf)
- Mo, J., Walrand, J. (2000). *Fair End-to-End Window-Based Congestion Control*. IEEE/ACM Transactions on Networking, Vol. 8, No. 5.
- Kelly, F. P., Maulloo, A. K., Tan, D. K. H. (1998). *Rate control for communication networks: shadow prices, proportional fairness and stability*. J. Oper. Res. Soc.
- Bianconi, G., Barabási, A.-L. (2001). *Competition and multiscaling in evolving networks*. Europhys. Lett.
- Empirical Analysis of Bittensor Subnet Economy (2025). [arXiv 2507.02951](https://arxiv.org/html/2507.02951v1).
