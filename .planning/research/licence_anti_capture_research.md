# Recherche exhaustive : Licence, vocabulaire anti-capture, et positionnement SBFB

**Date** : 2026-05-18
**Mode** : Ecosystem + Feasibility
**Confiance globale** : HIGH (sources primaires OSI/FSF/SPDX + jurisprudence
recente 2024-2025 + analyse exhaustive du codebase)

---

## Table des matieres

1. [Etat actuel du projet](#1-etat-actuel-du-projet)
2. [Definition OSI et ses limites](#2-definition-osi-et-ses-limites)
3. [Panorama des licences alternatives](#3-panorama-des-licences-alternatives)
4. [Theorie des communs numeriques](#4-theorie-des-communs-numeriques)
5. [AGPL-3.0 : forces et limites reelles](#5-agpl-30--forces-et-limites-reelles)
6. [Analyse des options pour SBFB](#6-analyse-des-options-pour-sbfb)
7. [Analyse du vocabulaire](#7-analyse-du-vocabulaire)
8. [Recommandation](#8-recommandation)
9. [Plan de migration concret](#9-plan-de-migration-concret)
10. [Sources](#10-sources)

---

## 1. Etat actuel du projet

### 1.1 Licence en vigueur

Le fichier `LICENSE` a la racine declare :

```
SPDX-License-Identifier: AGPL-3.0-or-later
Copyright (C) 2026 FlowUP & Contributors
```

La licence est AGPL-3.0 ou version ulterieure. C'est une licence
OSI-approved, FSF-approved, copyleft forte avec clause reseau.

### 1.2 Declaration dans le code

- `Cargo.toml` workspace : `license = "AGPL-3.0-or-later"`
- `deny.toml` : allowlist incluant `AGPL-3.0` et `AGPL-3.0-or-later`
- `Packager.toml` : `copyright = "Copyright 2026 FlowUP — AGPL-3.0-or-later"`
- `BUILDING.md` : `AGPL-3.0-or-later — see LICENSE`
- Chaque fichier source porte l'en-tete SPDX : `// SPDX-License-Identifier: AGPL-3.0-or-later`

La discipline est exemplaire — aucun fichier ne manque l'en-tete SPDX.

### 1.3 Vocabulaire "open source" dans le projet

**Usages dans CLAUDE.md** :
- "App store open source par construction" (ligne 11)
- "AGPL-3.0 maintenue" (decisions gelees, ligne 216)

**Usages dans le Protocol Explorer** (`examples/sbfb-explorer/index.html`) :
- "Open source par construction" (section Philosophie, titre de carte)
- "Le modele F-Droid/Linux applique aux apps web P2P" (corps de carte)
- Footer : "une app open source deployee sur le reseau SBFB"
- Lien vers `LICENSE` — AGPL-3.0

**Usages dans le frontend React** (`web/src/`) :
- `GpuConsentDialog.tsx:67` : "Projets open source verifies"
- `GpuConsentDialog.tsx:70` : "Apps open source verifiees (SLSA L1)"
- `is_open_source` : flag booleen dans `BrowseEntry`, `TaskEntry`, etc.

**Usages dans PUBLISH_MODEL.md** :
- Etat "Verified Release" decrit comme "open source verifie"
- "Le chemin standard pour une publication open source"

**Usages dans la recherche s65 contrat public** :
- Identifie deja les gaps critiques (G1-G3) lies au vocabulaire "verifie"
  et "open source" qui sur-promet par rapport a la garantie reelle

### 1.4 Le flag `is_open_source` dans le code

Ce flag est un booleen dans le protocole wire. Il est set a `true` par
le coordinator au moment du `deploy-from-repo` (deploy verifie depuis
un depot public). Il est set a `false` pour les uploads directs sans
provenance. Il conditionne le consentement GPU L2 des workers.

Le nom `is_open_source` est **techniquement inexact** : il indique que
le code a ete deploye depuis un depot public avec provenance SLSA L1,
pas que le code est "open source" au sens OSI. Un depot public peut
contenir du code sous n'importe quelle licence, y compris proprietaire.

### 1.5 Vision du PO (memory files)

Le fichier `vision_model.md` est explicite :
- Pattern OpenBSD solo maintainer, pas startup
- Pas de funding, pas de fondation, pas de board
- "Durabilite = AGPL + fork rights + code auto-hebergeable"
- "Credibilite = code qui parle + threat model publie + architecture honnete"
- Anti-institutionnalisation, pas anti-adoption

Le fichier `feedback_kudos_non_monetary.md` interdit tout vocabulaire
monetaire. Le fichier `fairness_vision.md` parle de "bien commun
protocolaire".

---

## 2. Definition OSI et ses limites

### 2.1 Les 10 criteres de l'Open Source Definition

L'OSI definit "open source" via 10 criteres. Les deux pertinents pour
le debat anti-capture sont :

**Critere 5 — Non-discrimination envers les personnes ou groupes** :
> "The license must not discriminate against any person or group of
> persons."

**Critere 6 — Non-discrimination envers les champs d'activite** :
> "The license must not restrict anyone from making use of the program
> in a specific field of endeavor. For example, it may not restrict
> the program from being used in a business, or from being used for
> genetic research."

Toute licence qui interdit l'usage commercial, l'usage militaire,
l'usage par des entreprises capitalistes, ou tout autre champ
d'activite **ne peut pas etre "open source" au sens OSI**.

L'AGPL-3.0 est compatible OSI car elle n'interdit aucun usage — elle
exige que le code source soit partage si le logiciel est utilise en
reseau. C'est une obligation de reciprocite, pas une restriction d'usage.

**Confiance : HIGH** — source primaire OSI.

### 2.2 Critiques de la definition OSI (2024-2026)

**Bruce Perens (co-auteur de l'OSD) — "Post-Open Source" (2024)** :

Perens, l'un des auteurs originaux de l'OSD, a publie en mars 2024
un brouillon de "Post-Open Zero Cost License". Sa these :

> "Our licenses aren't working anymore. We've had enough time that
> businesses have found all of the loopholes."

Sa solution proposee : les individus et non-profits utilisent le
logiciel gratuitement, mais les entreprises a >5M$ de CA annuel paient
1% de leur revenu a une entite 501(c)(6) qui redistribue aux
mainteneurs.

**Pertinence SBFB** : la Post-Open License necessite une fondation
administrative — incompatible avec le modele OpenBSD solo maintainer.
Mais la critique de fond (les entreprises contournent le copyleft) est
pertinente.

**Debat AI et OSD (2024-2025)** :

L'OSI a publie une "Open Source AI Definition" (OSAID) tres critiquee
car elle ne requiert pas l'ouverture des donnees d'entrainement. Ce
debat a affaibli la legitimite de l'OSI comme autorite terminologique.
Toutefois, la definition logicielle classique (les 10 criteres) reste
stable depuis 2002.

**Confiance : HIGH** — sources multiples convergentes.

### 2.3 Le vocabulaire "source-available" (2024-2025)

L'industrie a cristallise une distinction nette en 2024-2025 :

| Terme | Signification | OSI-compatible |
|---|---|---|
| **Open source** | Conforme aux 10 criteres OSI | Oui |
| **Source-available** | Code visible mais avec restrictions d'usage | Non |
| **Proprietary** | Code ferme | Non |
| **Fair source** | Source-available avec conversion automatique en open source apres N annees | Non (avant conversion) |

Le terme "open-washing" est devenu pejoratif en 2025 : utiliser le
vocabulaire "open source" pour des licences qui ne le sont pas.

MongoDB, Redis, HashiCorp, Elastic — tous sont passes de "open source"
a "source-available" en 2018-2024, puis certains sont revenus a
l'AGPL (Redis en 2025, Elastic en 2024) apres les forks communautaires
(Valkey, OpenTofu).

**Confiance : HIGH** — consensus industriel documente.

---

## 3. Panorama des licences alternatives

### 3.1 Licences ethiques / anti-capture

| Licence | Auteur | Mecanisme | Pertinence SBFB | Testee juridiquement | OSI |
|---|---|---|---|---|---|
| **Hippocratic License 3.0** | Coraline Ada Ehmke | Interdit les usages qui violent les droits humains (modules configurables) | FAIBLE — trop vague, SBFB n'est pas un projet de droits humains | Non | Non |
| **Anti-996 License** | communaute chinoise | MIT + conformite aux lois du travail ILO | NULLE — concerne le droit du travail, pas l'extraction logicielle | Non | Non |
| **Non-Violent Public License** | thufie | Interdit l'usage militaire et policier | FAIBLE — restriction de champ d'activite, pas anti-capture general | Non | Non |
| **Anti-Capitalist Software License** | v1.4 | Interdit l'usage par les entreprises non-cooperatives | MODERE en theorie — mais inapplicable et politiquement polarisant | Non | Non |
| **Peer Production License** | Kleiner/Bauwens | Usage commercial reserve aux cooperatives et non-profits | MODERE — alignement philosophique mais zero jurisprudence, derive de CC-NC (pas concu pour le logiciel) | Non | Non |
| **Commons Clause** | FOSSA/Heather Meeker | Ajout a une licence existante : interdit la revente du logiciel comme service | FAIBLE — protege contre le cloud hosting, mais SBFB est P2P, pas SaaS | Non | Non |
| **PolyForm Noncommercial** | PolyForm Project | Usage non-commercial uniquement | FAIBLE — trop restrictif pour l'adoption | Non | Non |
| **PolyForm Small Business** | PolyForm Project | Gratuit pour les entreprises <1M$/an et <100 employes | FAIBLE — SBFB n'a pas de modele de revenus, seuil arbitraire | Non | Non |

### 3.2 Licences "source-available" corporate

| Licence | Auteur | Mecanisme | Cas d'usage | Pertinence SBFB |
|---|---|---|---|---|
| **SSPL** (Server Side Public License) | MongoDB | Super-AGPL : qui offre le logiciel comme service doit open-sourcer toute sa stack | MongoDB, Redis | NULLE — concu pour la protection SaaS, SBFB est P2P |
| **BSL 1.1** (Business Source License) | MariaDB/HashiCorp | Source-available avec conversion en open source apres 4 ans | HashiCorp Terraform | NULLE — concu pour proteger un business model, pas un commun |
| **FSL** (Functional Source License) | Sentry | Non-compete + conversion Apache/MIT apres 2 ans | Sentry, Codecov | NULLE — meme logique business que BSL |
| **ELv2** (Elastic License 2.0) | Elastic | Interdit de fournir le logiciel comme service heberge | Elasticsearch | NULLE — meme logique SaaS |
| **RSALv2** (Redis Source Available License) | Redis | Interdit la commercialisation comme service | Redis | NULLE — meme logique |

### 3.3 Lecon des licences alternatives

**Toutes les licences "anti-capture" existantes ont ete concues pour
proteger un business model contre les cloud providers** (Amazon, Google,
Azure). Elles repondent a la question : "Comment empecher AWS de
vendre mon logiciel en SaaS sans contribuer ?"

**SBFB ne pose pas cette question.** SBFB est un protocole P2P
decentralise, sans serveur central, sans modele de revenus, sans
entreprise derriere. La menace n'est pas "Amazon heberge mon SaaS"
mais "une entite fork le protocole et supprime le copyleft".

L'AGPL est **la seule licence standard qui repond a cette menace** :
elle oblige quiconque modifie et deploie le logiciel en reseau a
partager ses modifications sous la meme licence. C'est exactement le
mecanisme anti-capture dont SBFB a besoin.

**Confiance : HIGH** — analyse comparative basee sur les textes des
licences et leurs cas d'application reels.

---

## 4. Theorie des communs numeriques

### 4.1 Elinor Ostrom et les 8 principes de gouvernance des communs

Elinor Ostrom (Nobel 2009) a identifie 8 principes institutionnels
pour la gouvernance durable des communs. Application a SBFB :

| Principe Ostrom | Application SBFB | Etat |
|---|---|---|
| 1. Frontieres clairement definies | Le protocole definit qui est un noeud (Ed25519 identity) | OK |
| 2. Regles d'usage adaptees au contexte local | Consent GPU 4 niveaux, curator lists locales | OK |
| 3. Arrangements de choix collectif | Curator lists Ed25519 = gouvernance distribuee | OK |
| 4. Monitoring | Feed hash-chain verifiable, kudos ledger | OK |
| 5. Sanctions graduees | Trust scoring, quarantine, ban automatique | OK |
| 6. Mecanismes de resolution des conflits | Fork libre (AGPL), curator lists competitives | Partiel |
| 7. Reconnaissance minimale du droit a s'organiser | AGPL garantit le droit de fork et de re-deploiement | OK |
| 8. Gouvernance emboitee (multiple niveaux) | Local (noeud) + reseau (gossip) + communaute (curators) | OK |

SBFB est **structurellement un commun numerique** au sens d'Ostrom.
L'architecture P2P + Ed25519 + curator lists + AGPL satisfait les
8 principes sans institutionnalisation.

### 4.2 Anti-capture vs anti-enclosure

La theorie des communs distingue deux menaces :

**Enclosure** (cloture) : privatiser une ressource qui etait commune.
Exemple : un fork proprietaire de SBFB qui ferme le code.
*Reponse* : le copyleft AGPL empechant l'enclosure — quiconque modifie
doit partager.

**Extraction** (extractivisme) : exploiter la ressource commune sans
contribuer en retour, au-dela de son taux de renouvellement.
Exemple : une entreprise qui utilise le reseau SBFB massivement sans
contribuer aucune ressource (compute, curation, bugs).
*Reponse* : le copyleft seul ne suffit pas. C'est un probleme de
**gouvernance communautaire**, pas de licence.

SBFB a des mecanismes contre l'extraction :
- Kudos per-project (reputation basee sur la contribution)
- Age witness (anciennete pour l'admission)
- PoW Sybil resistance (cout d'entree)
- Curator lists (gouvernance distribuee de la confiance)

Ces mecanismes sont **protocolaires**, pas juridiques. C'est la bonne
approche : le protocole fait respecter les regles, pas un tribunal.

### 4.3 Le concept de "commun logiciel"

Le terme "commun logiciel" (software commons) a une base theorique
solide :
- UNESCO le reconnait dans le cadre des "knowledge commons"
- La P2P Foundation le definit comme une ressource partagee gouvernee
  par une communaute
- Open Future (initiative europeenne) utilise "digital commons" dans
  le contexte des politiques publiques europeennes 2025

Utiliser "commun logiciel" plutot que "open source" est
**theoriquement fonde** et **politiquement coherent** avec la vision
SBFB.

**Confiance : HIGH** — litterature academique et institutionnelle solide.

---

## 5. AGPL-3.0 : forces et limites reelles

### 5.1 Ce que l'AGPL garantit

| Garantie | Mecanisme | Force |
|---|---|---|
| **Reciprocite reseau** | Quiconque deploie le logiciel modifie en reseau doit partager le code source | C'est la raison d'etre de l'AGPL vs GPL |
| **Droit de fork** | N'importe qui peut forker, modifier, et re-deployer | Fondamental pour la perennite |
| **Propagation virale** | Les oeuvres derivees restent sous AGPL | Empeche l'enclosure du code |
| **Non-discrimination** | Tout le monde peut utiliser, y compris commercialement | OSI-compatible |
| **Clause brevets** | Contributeurs accordent licence de brevet non-exclusive | Protection contre le patent trolling |

### 5.2 Ce que l'AGPL ne garantit PAS

| Non-garantie | Explication | Impact SBFB |
|---|---|---|
| **Pas d'anti-commercial** | L'utilisation commerciale est explicitement autorisee | Amazon peut forker et heberger |
| **Pas d'anti-fork** | Forker est un droit fondamental | Un fork proprietaire est interdit, mais un fork AGPL est permis |
| **Pas de governance** | La licence ne dit rien sur qui decide quoi | La gouvernance est dans le protocole, pas la licence |
| **Pas de contribution obligatoire** | On peut utiliser sans contribuer (tant qu'on partage les modifications) | Free riding sans modification est permis |
| **Pas de build reproductible** | La licence concerne le code, pas le build | La provenance SLSA L1 est un ajout protocolaire |

### 5.3 Le "AGPL ban" corporate

Google interdit l'utilisation de code AGPL en interne. Beaucoup de
grandes entreprises suivent. C'est un effet secondaire **desirable**
pour SBFB : l'AGPL agit comme un **deterrent passif** contre
l'integration dans des stacks proprietaires.

Mais le ban n'est pas universel — Meta, par exemple, utilise du code
AGPL dans certains contextes.

### 5.4 Cas Mastodon / Truth Social

Le cas Mastodon/Truth Social (2021) est le seul cas d'enforcement AGPL
mediatise. Truth Social a utilise le code Mastodon sans partager ses
modifications. Apres mise en demeure, Truth Social a publie un zip
barebones — techniquement conforme mais minimalement.

**Lecon pour SBFB** : l'enforcement AGPL est **possible mais difficile**
sans CLA (Contributor License Agreement). Mastodon n'a pas de CLA, donc
l'enforcement necessite l'accord de tous les contributeurs. SBFB est
dans la meme situation (solo maintainer + contributeurs sans CLA).

### 5.5 Cas Redis et Elastic (2024-2025)

Redis et Elastic ont quitte l'open source (BSD/Apache) pour des
licences source-available (SSPL, RSALv2, BSL, ELv2) en 2018-2023.
Les deux ont ete forkes (Valkey pour Redis, OpenSearch pour Elastic).
En 2024-2025, les deux sont **revenus a l'AGPL** comme option de
licence supplementaire.

**Lecon pour SBFB** : l'AGPL est la licence vers laquelle convergent
les projets qui ont essaye les alternatives. Elle est imparfaite
mais c'est le meilleur compromis existant entre protection et adoption.

### 5.6 Contournements connus

| Contournement | Description | Applicable a SBFB ? |
|---|---|---|
| **Dual licensing** | L'auteur vend une licence non-AGPL en parallele | Non — pas de business model |
| **API boundary** | Ne pas linker directement, communiquer via API/protocole | Possible mais le protocole P2P rend ca inutile |
| **Aggregation** | Distribuer le logiciel AGPL a cote d'un logiciel proprio sans lien | Faible risque pour un daemon standalone |
| **Non-modification** | Utiliser le logiciel sans le modifier → pas d'obligation de partage | Acceptable — ils utilisent le protocole standard |
| **Service sans reseau** | Utiliser le logiciel en interne sans l'exposer via reseau | Inapplicable — SBFB est inheremment reseau |

**Confiance : HIGH** — texte de la licence + cas d'usage reels.

---

## 6. Analyse des options pour SBFB

### Option A — AGPL-3.0 pure (statu quo)

**Forces** :
- Reconnue mondialement, juridiquement testee (30+ ans de copyleft GPL)
- Compatible OSI et FSF
- Copyleft reseau — oblige au partage des modifications en SaaS/P2P
- Deterrent corporate (Google ban, etc.)
- SBFB peut **legitimement** utiliser le terme "open source"
- Pas de friction d'adoption — les devs connaissent l'AGPL
- Redis et Elastic y sont revenus apres avoir essaye les alternatives

**Faiblesses** :
- N'empeche pas l'extraction commerciale (utiliser sans contribuer)
- N'empeche pas le fork AGPL concurrent
- Le mot "open source" a une connotation corporate Silicon Valley

**Vocabulaire possible** : "open source", "logiciel libre", "copyleft reseau"

**Verdict** : **RECOMMANDE** comme licence.

### Option B — AGPL-3.0 + Commons Clause

**Forces** :
- Empeche la revente du logiciel comme service
- Code source reste visible

**Faiblesses** :
- Plus "open source" au sens OSI → interdit de se dire open source
- Juridiquement moins testee que l'AGPL seule
- La Commons Clause est concue pour les SaaS, pas les protocoles P2P
- Un fork par la communaute serait inevitable (cf. Redis → Valkey)
- Contradiction avec le modele OpenBSD (fork rights fondamentaux)

**Vocabulaire impose** : "source-available"

**Verdict** : **REJETE** — ajoute de la friction sans resoudre le bon
probleme. SBFB n'est pas un SaaS, la revente comme service n'est pas
la menace.

### Option C — Licence SBFB custom (anti-capture)

**Forces** :
- Controle total du vocabulaire et des restrictions
- Peut exprimer exactement la philosophie du projet

**Faiblesses** :
- **Zero jurisprudence** — aucun tribunal n'a jamais interprete cette licence
- Friction d'adoption extreme — chaque contributeur doit comprendre une nouvelle licence
- Cout legal prohibitif pour le drafter correctement (Perens estime qu'un avocat qualifie est necessaire)
- Le Peer Production License, l'Anti-Capitalist License, et la Hippocratic License existent depuis des annees et **aucune n'a ete testee en justice**
- Contradiction avec le modele OpenBSD (simplicite, standards reconnus)
- Risque d'open-washing inverse : se dire "anti-capture" avec une licence que personne ne comprend

**Vocabulaire possible** : "licence SBFB anti-capture"

**Verdict** : **REJETE** — cout/risque >> benefice. Ecrire une bonne
licence est un travail d'avocat specialise, pas de dev. Meme Perens
n'a pas fini la sienne apres 2 ans.

### Option D — Dual licensing (AGPL + commercial)

**Forces** :
- AGPL pour la communaute, licence commerciale pour qui veut fermer
- Modele bien teste (MySQL, Qt, MongoDB historique)

**Faiblesses** :
- Le mainteneur doit posseder 100% du copyright (CLA obligatoire)
- Les contributeurs doivent signer un CLA cedant leurs droits
- Contradiction totale avec le modele OpenBSD (pas de business model)
- Contradition avec le vision_model.md (pas de funding, pas de revenus)
- Cree une asymetrie mainteneur/contributeurs contraire a l'esprit commun

**Vocabulaire possible** : "open core" ou "dual license"

**Verdict** : **REJETE** — incompatible avec la vision du projet.

### Option E — AGPL-3.0 + convention sociale documentee

**Forces** :
- Licence OSI valide → peut se dire "open source" si souhaite
- Document compagnon exprimant la philosophie anti-capture
- Pas de friction juridique (le document n'est pas contraignant)
- Precedent : la licence Python a un "Zen of Python", l'AGPL a le
  "Preamble" du GNU qui explique la philosophie
- Compatible avec le modele OpenBSD (la licence est standard, la
  philosophie est documentee mais non-contraignante)
- Le protocole lui-meme (kudos, age witness, PoW, curators) est le
  vrai mecanisme anti-capture, pas la licence

**Faiblesses** :
- Le document n'est pas juridiquement contraignant
- Quelqu'un peut l'ignorer tout en respectant la licence
- "Convention sociale" sonne moins fort que "licence anti-capture"

**Vocabulaire possible** : "commun logiciel" + "AGPL-3.0" + "convention
anti-capture"

**Verdict** : **RECOMMANDE** — c'est l'option qui combine le meilleur
de tous les mondes.

### Option F — Peer Production License

**Forces** :
- Explicitement concue pour les communs
- Philosophiquement alignee avec SBFB (cooperatives, pas d'extraction)
- Nom evocateur ("production par les pairs")

**Faiblesses** :
- **Derivee de Creative Commons CC-BY-NC-SA** — pas concue pour le logiciel
- Zero jurisprudence, zero adoption significative dans le logiciel
- Restreint l'usage commercial aux cooperatives → discrimine contre
  les champs d'activite (criteres OSI 5 et 6)
- Contradition avec le droit de fork universel (un individu non-cooperatif
  ne peut pas utiliser commercialement)
- Inapplicable dans la pratique (comment verifier qu'une organisation
  est une cooperative ?)
- N'existe meme pas dans le registre SPDX standard
- Contradiction avec le modele OpenBSD (Theo de Raadt n'est pas une
  cooperative)

**Vocabulaire impose** : "peer production" ou "commun logiciel"

**Verdict** : **REJETE** — philosophiquement seduisant, juridiquement
inutilisable. Une licence concue pour les oeuvres culturelles ne
s'applique pas au logiciel.

---

## 7. Analyse du vocabulaire

### 7.1 "Open source"

| Aspect | Analyse |
|---|---|
| **Definition OSI** | Conforme aux 10 criteres. SBFB sous AGPL-3.0 EST "open source" au sens OSI. |
| **Connotation** | Associe a la Silicon Valley, au corporate OSS, au free-riding par les GAFAM. |
| **Pertinence SBFB** | Techniquement correct. Politiquement connoté. Le PO ressent un malaise légitime. |
| **Risque si abandon** | Perdre la visibilite et la decouverte — "open source" est un terme de recherche majeur. |
| **Risque si maintien** | Sur-promesse potentielle si le vocabulaire evolue vers des restrictions non-OSI. |
| **Verdict** | **Garder dans le contexte technique** (badge "Source", deploiement), **eviter comme identite principale** du projet. |

### 7.2 "Source verifiable"

| Aspect | Analyse |
|---|---|
| **Definition** | Le code source est visible ET sa correspondance avec l'artefact deploye est verifiable cryptographiquement. |
| **Base theorique** | Concept technique — provenance SLSA + signature Ed25519 + hash BLAKE3. |
| **Pertinence SBFB** | **Excellent** — c'est exactement ce que SBFB fait. Mieux que "open source" car ca decrit la **propriete cryptographique**, pas juste la visibilite du code. |
| **Difference avec "source-available"** | "Source-available" = code visible avec restrictions. "Source verifiable" = code visible + preuve cryptographique de correspondance deploiement/source. |
| **Risque** | Terme nouveau, pas encore standard. Necessite explication. |
| **Verdict** | **RECOMMANDE comme terme technique** pour decrire la propriete du deploy verifie. |

### 7.3 "Commun logiciel" / "digital commons"

| Aspect | Analyse |
|---|---|
| **Definition** | Ressource logicielle partagee, gouvernee par une communaute, protegee contre l'enclosure par une licence copyleft. |
| **Base theorique** | Ostrom, P2P Foundation, UNESCO, Open Future. Solide. |
| **Pertinence SBFB** | **Excellent** — SBFB est structurellement un commun (cf. analyse Ostrom section 4.1). |
| **Risque** | Terme academique, peut sembler pretentieux. Moins connu que "open source". |
| **Verdict** | **RECOMMANDE comme identite du projet**. "SBFB est un commun logiciel" est plus exact que "SBFB est un projet open source". |

### 7.4 "Anti-capture"

| Aspect | Analyse |
|---|---|
| **Definition** | Mecanismes qui empechent l'appropriation privee d'une ressource commune. |
| **Base theorique** | Ostrom (enclosure), P2P Foundation (peer production), communs numeriques. |
| **Pertinence SBFB** | **Bonne** — mais le terme est vague. Anti-capture par qui ? Contre quoi exactement ? |
| **Risque** | Trop militant pour certains publics. Pas auto-explicatif. |
| **Verdict** | **Utiliser dans la documentation interne et la philosophie**, pas dans l'UI. Le protocole est le mecanisme anti-capture, pas un label. |

### 7.5 "Anti-extraction"

| Aspect | Analyse |
|---|---|
| **Definition** | Contre l'exploitation d'une ressource commune sans reciprocite, au-dela de son taux de renouvellement. |
| **Base theorique** | Theorie des communs (tragedy of the commons). Lien avec "extractivisme numerique" (Sadin, Morozov). |
| **Pertinence SBFB** | **Moderee** — SBFB a des mecanismes anti-extraction (kudos, PoW, age witness) mais le terme est politise. |
| **Risque** | Terme militant marxisant. Alienation des contributeurs non-politises. |
| **Verdict** | **Eviter dans l'UI et la communication publique.** Utilisable dans la documentation philosophique interne. |

### 7.6 "Copyleft reseau"

| Aspect | Analyse |
|---|---|
| **Definition** | Clause de la licence AGPL qui oblige au partage du code source si le logiciel est utilise via un reseau. |
| **Base theorique** | GNU AGPL-3.0, section 13. Standard juridique. |
| **Pertinence SBFB** | **Exacte** — c'est litteralement ce que fait l'AGPL pour SBFB. |
| **Risque** | Terme technique, peu connu du grand public. |
| **Verdict** | **Utiliser dans la documentation technique et les tooltips.** Pas dans les titres UI. |

### 7.7 "Provenance verifiable"

| Aspect | Analyse |
|---|---|
| **Definition** | Capacite de verifier cryptographiquement la chaine de provenance d'un artefact (source → build → deploiement). |
| **Base theorique** | SLSA, Sigstore, in-toto. Standard industriel. |
| **Pertinence SBFB** | **Exacte** — c'est le coeur du deploy verifie SBFB. |
| **Risque** | Aucun — terme technique standard, factuel. |
| **Verdict** | **RECOMMANDE comme terme technique** pour le systeme de provenance. Deja bien utilise dans la recherche s65 contrat public. |

### 7.8 "Licence libre"

| Aspect | Analyse |
|---|---|
| **Definition FSF** | Les 4 libertes (utiliser, etudier, modifier, distribuer). AGPL-3.0 est une licence libre. |
| **Difference avec "open source"** | "Libre" = accent sur la liberte. "Open source" = accent sur la methode de developpement. En pratique les deux se recouvrent quasi-totalement. |
| **Pertinence SBFB** | Bonne — mais le terme est associe a la FSF/GNU, et le PO n'est pas dans un positionnement FSF. |
| **Verdict** | **Acceptable mais pas prioritaire.** "Commun logiciel" est plus distinctif. |

### 7.9 Synthese du vocabulaire recommande

| Contexte | Terme recommande | Terme a eviter | Raison |
|---|---|---|---|
| **Identite du projet** | "commun logiciel" | "open source" comme identite | Plus exact, plus distinctif, base theorique solide |
| **Licence** | "AGPL-3.0" ou "copyleft reseau" | "licence libre" generique | Precision juridique |
| **Deploy verifie** | "source verifiable" + "provenance verifiable" | "open source verifie" | Evite la sur-promesse, decrit la propriete reelle |
| **Flag `is_open_source`** | garder le nom dans le code (wire compat) | — | Renommage dans le protocole wire serait un break |
| **UI badges** | "Provenance", "Source" | "Verifie" sans qualification | cf. recherche s65 contrat public |
| **Documentation technique** | "copyleft reseau", "anti-capture protocolaire" | "anti-extraction", "anti-capitaliste" | Precision vs militantisme |
| **Communication publique** | "commun logiciel protege par copyleft reseau" | "open source anti-capture" | Coherent, factuel, non-polarisant |
| **Footer/about** | "sous licence AGPL-3.0" | "sous licence open source" | La licence a un nom, l'utiliser |

---

## 8. Recommandation

### 8.1 Decision licence : AGPL-3.0 maintenue (Option A + E)

**La licence ne change pas.** L'AGPL-3.0-or-later reste la licence du
projet. C'est la decision architecturale gelee dans CLAUDE.md, et
l'analyse confirme qu'elle est correcte.

**Ce qui change, c'est le vocabulaire et le positionnement.**

### 8.2 Convention anti-capture (Option E)

Creer un document `COMMONS.md` (ou `ANTI-CAPTURE.md`) a la racine du
projet qui :

1. **Definit SBFB comme un commun logiciel** au sens d'Ostrom
2. **Explique pourquoi l'AGPL-3.0** (copyleft reseau = anti-enclosure)
3. **Documente les mecanismes protocolaires anti-capture** :
   - Kudos non-monetaires (pas d'extraction financiere)
   - Age witness (cout d'entree temporel)
   - PoW Sybil resistance (cout d'entree computationnel)
   - Curator lists Ed25519 (gouvernance distribuee)
   - Fork rights AGPL (pas de capture par un acteur unique)
4. **Declare les normes communautaires** (non-contraignantes) :
   - Les contributions sont bienvenues de tous
   - Le free-riding est tolere (c'est le prix de la liberte)
   - L'enclosure (fork proprietaire) est contraire a l'esprit du projet
   - Le protocole, pas un tribunal, fait respecter les regles
5. **Explique pourquoi SBFB n'utilise pas le terme "open source"
   comme identite** (trop connote corporate) tout en etant
   **juridiquement open source** (OSI-compatible AGPL-3.0)

Ce document est l'equivalent du "Zen of Python" ou du "GNU Manifesto"
pour SBFB : une declaration philosophique qui accompagne la licence
sans la modifier.

### 8.3 Vocabulaire a figer avant S65 Phase A

| Contexte | Ancien terme | Nouveau terme |
|---|---|---|
| CLAUDE.md §Projet | "App store open source par construction" | "App store a source verifiable — commun logiciel sous AGPL-3.0" |
| CLAUDE.md §Decisions gelees | "AGPL-3.0 maintenue" | Inchange (correct tel quel) |
| Protocol Explorer §Philosophie | "Open source par construction" | "Source verifiable par construction" |
| Protocol Explorer §Philosophie | "Le modele F-Droid/Linux" | "Inspire par F-Droid — apps deployees depuis leur source" |
| Protocol Explorer footer | "une app open source deployee" | "une app a source verifiable deployee" |
| GpuConsentDialog L2 | "Projets open source verifies" | "Apps deployees depuis un depot public" |
| GpuConsentDialog L2 threat | "Apps open source verifiees (SLSA L1)" | "Apps avec provenance auto-attestee" |
| Network.tsx L2 label | "L2 — Open source" | "L2 — Depot public" |
| PUBLISH_MODEL.md | "open source verifie" | "release avec provenance" |
| Browse badge | "Verifie" | "Provenance" (cf. recherche s65 contrat public) |

### 8.4 Ce qui ne change PAS

- Le flag `is_open_source` dans le code/wire ne change pas (break de
  protocole pre-launch interdit, et le nom reste acceptable en code)
- Le fichier `LICENSE` ne change pas
- Les en-tetes SPDX ne changent pas
- `deny.toml` ne change pas
- La mention "AGPL-3.0" partout ou elle apparait reste

---

## 9. Plan de migration concret

### 9.1 Phase A du S65 — Document de philosophie + taxonomie

**Delivrables** :
1. `COMMONS.md` a la racine (convention anti-capture, ~150 lignes)
2. Mise a jour CLAUDE.md §Projet (vocabulaire aligne)
3. `docs/protocol/TRUST_TAXONOMY.md` (taxonomie de confiance, cf. recherche s65 contrat public)
4. Mise a jour `docs/architecture/PUBLISH_MODEL.md` (vocabulaire aligne)

**Scope** : ~300 lignes de documentation, 0 ligne de code.

### 9.2 Phase B du S65 — Migration UI

**Delivrables** :
1. Protocol Explorer `index.html` : tous les textes corriges
2. `GpuConsentDialog.tsx` : L2 wording
3. `Network.tsx` : L2 label
4. `Browse.tsx` / `BrowsedProject.tsx` : badges (cf. recherche s65 contrat public)
5. `Curators.tsx` : "de confiance" retire
6. Tests Vitest/Playwright mis a jour

**Scope** : ~10 fichiers touches, ~80 lignes modifiees.

### 9.3 Phase C (si le temps permet) — Badge dynamique + scan

**Delivrables** :
1. Badge dynamique post-verification dans BrowsedProject
2. Script `scan-trust-wording.sh` (cf. recherche s65 contrat public)

### 9.4 Items resolus par la migration

| Carry | Resolution |
|---|---|
| P2-BADGE-WORDING-PREMATURE (pre-existant S14) | Phase B — badge "Provenance" au lieu de "Verifie" |

---

## 10. Sources

### Sources primaires (HIGH confidence)

- [Open Source Definition — OSI](https://opensource.org/osd) — les 10 criteres
- [GNU AGPL-3.0 texte officiel](https://www.gnu.org/licenses/agpl-3.0.en.html)
- [SLSA v1.0 Security Levels](https://slsa.dev/spec/v1.0/levels)
- Code source SBFB : `LICENSE`, `Cargo.toml`, `deny.toml`, `CLAUDE.md`, tous les fichiers listes en section 1

### Sources secondaires (MEDIUM confidence)

- [Bruce Perens — Post-Open License First Draft (mars 2024)](https://perens.com/2024/03/08/post-open-license-first-draft/)
- [The Register — Perens Post-Open Zero Cost License (avril 2024)](https://www.theregister.com/2024/04/30/bruce_perens_post_open_license/)
- [FOSS Force — Is Perens' Post Open License Necessary? (mai 2025)](https://fossforce.com/2025/05/is-bruce-perens-post-open-license-necessary/)
- [Goodwin Law — Moving Away From Open Source: Trends in Source-Available Licensing (sept 2024)](https://www.goodwinlaw.com/en/insights/publications/2024/09/insights-practices-moving-away-from-open-source-trends-in-licensing)
- [Redis — Returns to AGPLv3 (2025)](https://redis.io/blog/agplv3/)
- [Elastic — AGPLv3 as third option (2024)](https://nocturnalknight.co/why-did-elastic-decide-to-go-open-source-again/)
- [OpenTofu Manifesto](https://opentofu.org/manifesto/)
- [Sentry — FSL Introduction](https://blog.sentry.io/introducing-the-functional-source-license-freedom-without-free-riding/)
- [Sentry — Fair Source](https://blog.sentry.io/sentry-is-now-fair-source/)
- [Open Core Ventures — AGPL is a non-starter](https://www.opencoreventures.com/blog/agpl-license-is-a-non-starter-for-most-companies)
- [Google AGPL Policy](https://opensource.google/documentation/reference/using/agpl-policy)
- [Heather Meeker — AGPL In the Light of Day (2023)](https://heathermeeker.com/2023/10/13/agpl-in-the-light-of-day/)
- [Mastodon / Truth Social AGPL enforcement](https://mastodon.social/@Gargron/102316267911697650)
- [Nextcloud — Why AGPL is great for business](https://nextcloud.com/blog/why-the-agpl-is-great-for-business-users/)
- [Hippocratic License 3.0](https://firstdonoharm.dev/)
- [Anti-Capitalist Software License](https://anticapitalist.software/)
- [P2P Foundation — Peer Production License](https://wiki.p2pfoundation.net/Peer_Production_License)
- [P2P Foundation — Critique of the PPL](https://wiki.p2pfoundation.net/Critique_of_the_Peer_Production_License)
- [PolyForm Noncommercial License](https://polyformproject.org/licenses/noncommercial/1.0.0)
- [PolyForm Small Business License](https://polyformproject.org/licenses/small-business/1.0.0/)
- [Mozilla Foundation — Ostrom's Principles for Data Commons](https://www.mozillafoundation.org/en/blog/a-practical-framework-for-applying-ostroms-principles-to-data-commons-governance/)
- [David Bollier — Ostrom and Software Platforms](https://www.bollier.org/blog/applying-ostroms-guidelines-design-software-platforms)
- [Internet Policy Review — Digital Commons](https://policyreview.info/concepts/digital-commons)
- [UNESCO — Knowledge Commons and Enclosures](https://www.unesco.org/en/articles/knowledge-commons-and-enclosures)
- [Open Future — Digital Commons](https://openfuture.eu/tag/digital-commons/)
- [Wikipedia — Source-Available Software](https://en.wikipedia.org/wiki/Source-available_software)
- [F-Droid — Making Reproducible Builds Visible (mai 2025)](https://f-droid.org/2025/05/21/making-reproducible-builds-visible.html)
- [Sigstore Overview](https://docs.sigstore.dev/cosign/signing/overview/)

### Sources tertiaires (LOW confidence — contexte)

- [Ethicalsource.dev — Ethical Source Licenses](https://ethicalsource.dev/licenses/)
- [Nonviolent Public Licenses](https://thufie.lain.haus/NPL.html)
- [Anti-996 License GitHub](https://github.com/kattgu7/Anti-996-License)
- [Bollier — Bauwens Peer Production License](https://www.bollier.org/blog/bauwens-use-peer-production-license-foster-%E2%80%9Copen-cooperativism%E2%80%9D)
- [Frontiers — DAOs and Digital Commons (2025)](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2025.1538227/full)

---

*Recherche licence anti-capture : 2026-05-18*
