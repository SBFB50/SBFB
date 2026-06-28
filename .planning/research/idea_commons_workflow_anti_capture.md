# Atelier des communs - workflow d'idees anti-capture

Date: 2026-06-28
Status: NOTE DE RECHERCHE. Hors sprint. Aucun code, aucun wire, aucun commit
de phase, aucun verdict process.

Cette note recadre `idea_development_workflow_provider_neutral.md` et
`idea_creation_science_rrv_workflow.md`. Le but n'est pas de construire un
pipeline SaaS, un backlog de croissance, ou un Canny decentralise. Le but est
de transformer des besoins situes en communs verifiables, forkables,
maintenables et non extractifs.

## 1. Correction de cadrage

Le nom produit cible n'est pas "Ideas Hub" au sens tableau de votes. Le nom de
cadre est:

```text
Atelier des communs
```

Formule courte:

```text
des graines d'idees signees,
maturees par preuves RRV,
prises en charge par des porteurs responsables,
transformees en communs verifiables via Factory,
sous arbitrage humain et curation locale.
```

Ideas Hub peut rester le nom historique de l'app de capture. Mais l'objet
produit doit etre un atelier de maturation des communs, pas un funnel produit.

## 2. Evidence repo

### 2.1 Commun logiciel et anti-capture

`COMMONS.md` pose deja le cadre:

- le protocole SBFB est sous AGPL-3.0-or-later;
- pas de CLA;
- pas de fondation proprietaire;
- pas de governance token;
- pas de DAO;
- pas de tokenomics;
- pas de pitch investisseur;
- les apps du reseau sont a source verifiable, mais pas automatiquement open
  source.

`licence_anti_capture_research.md` confirme que l'AGPL est le mecanisme
standard adapte a SBFB: elle empeche l'enclosure du code sans fabriquer une
licence custom fragile. La note recommande aussi le vocabulaire "commun
logiciel", "copyleft reseau" et "source verifiable", plutot que "open source"
flou ou "source-available".

### 2.2 Anti-centralisation et curation locale

`README.md` et `POST_CHATONS_FORUM.md` fixent le cap:

- pas d'app store central;
- pas de compte cloud;
- pas d'autorite centrale qui decide ce qui peut exister;
- curators choisis par l'utilisateur;
- les communautes signent des recommandations ou des deconseils;
- un curator ne supprime pas une app du reseau;
- le projet reste en pilote ferme, pas en service public de production.

La consequence pour les idees est directe: aucun vote global, aucun score
global, aucun top des idees du reseau. Chaque noeud ou communaute peut tenir sa
vue locale, signer son appui ou son dissent, et se retirer.

### 2.3 Humanitaire, culture libre et utilite sociale

`sbfb_cross_domain_use_cases.md` definit SBFB comme infrastructure portable de
communs. Les meilleurs domaines ne sont pas les plus monetisables; ce sont les
domaines ou l'echec d'une plateforme centrale cause un dommage social reel:

- crise et secours;
- education offline;
- sante/logistique non sensible au depart;
- archives publiques;
- reparation;
- fabrication locale;
- archives culturelles;
- resilience energetique;
- science reproductible;
- transparence des chaines d'approvisionnement.

`docs/affine-sbfb/04_BABEL_SUR_SBFB.md` pose Babel comme infrastructure libre
de lecture: corpus, bibliotheque, traduction, provenance contributive,
validation humaine benevole, lecture offline, Babel Shelf, liseuses libres ou
reconditionnees. Gutenberg est le corpus de demarrage, pas une exception au
droit: chaque texte doit garder source, droits, attribution, redistribution et
traduction prouvables.

`docs/apps/CATASTROPHE_HUMANITAIRE.md` et `docs/apps/EHPAD_LIEN_FAMILLE.md`
montrent des besoins concrets: continuer a coordonner quand l'infrastructure
tombe; garder le lien humain sans cloud ni abonnement.

### 2.4 Compute volontaire, pas marche du GPU

Le compute SBFB n'est pas un marche de GPU. `POST_CHATONS_FORUM.md` parle de
mutualisme, anti-capture IA et solidarite. `FAIRNESS_VISION.md` avertit qu'un
scoring lineaire sur le hardware reproduit les hierarchies du cloud.

Donc le workflow d'idees doit favoriser:

- traduction;
- OCR;
- indexation;
- accessibilite;
- verification;
- analyse documentaire;
- education;
- preservation culturelle.

Il doit penaliser:

- slop;
- engagement farming;
- publicite;
- rente d'abonnement;
- lock-in cloud;
- pouvoir politique donne au capital hardware.

### 2.5 RRV et Factory restent des outils, pas des autorites

`docs/agent/RRV_FACTORY_CONTRACT.md` fixe:

```text
process > RRV > Factory
```

RRV recherche, classe, cite et expose les limites. Factory produit, valide,
publie et journalise. Aucun des deux ne decide qu'une idee est bonne, morale,
finie ou "PASS".

## 3. Langage product owner corrige

L'ancien langage "produit" doit etre traduit ainsi:

| Langage a eviter | Langage cible |
| --- | --- |
| product pipeline | chemin de maturation d'un commun |
| user/customer | personnes concernees, pairs, collectifs, noeuds locaux |
| value | utilite collective, dommage evite, autonomie gagnee |
| product-first | commun-first, solidarite-first, gouvernance-locale-first |
| adoption | appropriation volontaire |
| growth | capacite de reprise et de maintenance |
| MVP | plus petite intervention utile, reversible, maintenable |
| funnel | registre local de maturation |
| Product Owner | steward d'usage, garant du besoin situe |
| champion | porteur responsable |
| vote/upvote | appui local, attestation locale, watch, dissent |
| shipped apps | communs publies, forkables, maintenus |
| score de confiance | completude de preuve, facteurs de preuve |
| fork-and-compete | droit de reprise, fork de sauvetage |

## 4. Principe de priorisation

Une idee SBFB est prioritaire si elle:

1. augmente un commun;
2. reduit une dependance proprietaire;
3. rend une capacite utile accessible a des gens qui ne l'auraient pas
   autrement;
4. peut etre portee par un humain ou micro-noyau verifiable;
5. garde source, licence, provenance et droit de fork visibles;
6. ne transforme pas le reseau en marche, leaderboard, token ou rente GPU.

Ce n'est pas une priorite si son meilleur argument est:

- viralite;
- abonnement;
- monetaire;
- captation de donnees;
- avantage concurrentiel ferme;
- dependance a une API SaaS;
- pouvoir donne aux plus gros GPU;
- classement social global.

## 5. Artefacts

### 5.1 GraineDeBesoin

Remplace `IdeaSeed`.

Objet brut, non autoritaire:

- intuition;
- personnes concernees;
- contexte local;
- dommage actuel;
- autonomie recherchee;
- urgence;
- source;
- consentement si temoignage humain;
- statut de confidentialite.

### 5.2 BriefDeCommun

Remplace `DevelopmentBrief`.

Questions obligatoires:

1. Quel commun est augmente ou protege?
2. Qui est aide concretement?
3. Quelle dependance proprietaire ou institutionnelle est reduite?
4. Quelle donnee doit rester locale, exportable ou effacable?
5. Quelle licence/source/provenance rendra la reprise possible?
6. Quelle plus petite intervention utile suffit?
7. Qu'est-ce qui ne doit pas etre construit?

### 5.3 PaquetRRV

Evidence bundle phrase par phrase:

- `Lu`: present dans repo, doc, feed, manifest, archive;
- `Deduit`: inference locale explicite;
- `Verifie`: hash, signature, test, build, provenance, quorum;
- `Non verifie`: source web, hypothese LLM, declaration non prouvee.

RRV ne fusionne jamais source web, hypothese LLM et preuve SBFB sous la meme
couleur de confiance.

### 5.4 RevueAntiExtraction

Cette revue cherche ce qui pourrait transformer le commun en capture:

- collecte de donnees inutile;
- dependance fournisseur;
- rente d'abonnement;
- classement global;
- reputation convertible en pouvoir;
- kudos ou GPU comme gate;
- surveillance;
- charge de maintenance externalisee sur les plus faibles;
- open-washing;
- source fermee derriere une UI "communautaire".

### 5.5 CarteDeCooperation

Elle remplace l'idee qu'une foule abstraite "adoptera" le projet.

Elle decrit:

- porteur responsable;
- reviewer sceptique;
- personnes concernees;
- collectif ou noeud local;
- mainteneur possible;
- canaux de decision;
- forks de sauvetage;
- abandon propre.

### 5.6 EngagementDeStewardship

Le seuil utile n'est pas "10 votes". Le seuil utile est:

```text
1 porteur responsable + 1 review sceptique + 1 PaquetRRV minimal
```

Le porteur declare:

- prochaine action;
- delai;
- ressources;
- cout humain;
- cout compute;
- critere d'abandon;
- comment quelqu'un d'autre peut reprendre.

### 5.7 DecisionLocale

Sorties possibles:

- `reject`;
- `keep_research`;
- `needs_more_evidence`;
- `local_pilot`;
- `factory_brief`;
- `sprint_candidate`;
- `fork_or_reuse_existing`;
- `do_not_build`.

Une decision locale n'est jamais un verdict global du reseau.

## 6. Pipeline

```text
0. Graine
   Capturer le besoin sans jugement et sans vote global.

1. Clarification du commun
   Transformer l'intuition en BriefDeCommun.

2. Incubation
   Pause bornee, questions ouvertes, pas de sprint immediat.

3. Repo + RRV
   Produire le PaquetRRV avec labels Lu/Deduit/Verifie/Non verifie.

4. Options
   Reparons l'existant / Commun protocole-native / Solidarite-humanitaire /
   Ne pas construire.

5. Revue anti-extraction
   Chercher capture, rente, dependance, surveillance, charge cachee.

6. Carte de cooperation
   Identifier le porteur, les reviewers, les mainteneurs et les chemins de
   reprise.

7. ResearchPack
   Documenter preuves, limites, options, dissent et decision candidate.

8. Arbitrage humain
   Seul l'humain promeut vers Factory brief, pilote local ou sprint.
```

## 7. Gates non-sprint

Ces gates ne bloquent pas la publication generale sur le reseau. Sinon ils
deviendraient une police centrale. Ils bloquent seulement la promotion en
`factory_brief`, `sprint_candidate`, ou l'usage du label "commun verifiable".

| Gate | Question |
| --- | --- |
| NR0 Scope | Le pack est-il hors sprint et sans verdict PASS? |
| NR1 Separation | Capture, clarification, review et decision sont-elles separees? |
| NR2 Evidence | Chaque claim a-t-il source, label RRV ou `Not evidenced`? |
| NR3 Anti-fixation | Y a-t-il plusieurs options, dont `do_not_build`? |
| NR4 Incubation | L'idee a-t-elle une pause ou une justification documentee? |
| NR5 Human authority | La promotion reste-t-elle humaine? |
| NR6 Sandbox boundary | Aucune app sandboxee ne controle l'Operator? |
| NR7 Anti-ranking | Aucun score global n'est utilise comme gate? |
| NR8 LLM boundary | Le LLM ne sert ni de preuve ni de verdict final? |
| NR9 RRV labels | Les labels Lu/Deduit/Verifie/Non verifie restent distincts? |
| NR10 Commons Purpose | Le besoin social, les beneficiaires et le dommage evite sont explicites? |
| NR11 License & Reuse | Licence/source/provenance/export/fork sont prevus? |
| NR12 Anti-Capture | Pas de leaderboard global, token, stake, kudos gate ou rente GPU? |
| NR13 Maintenance Reality | Porteur, prochaine action, cout et abandon propre sont documentes? |

## 8. Future app Ideas Hub

L'app future doit etre "Atelier des communs" en pratique:

- `Graines`: besoins bruts, pas votes en premier;
- `Clarification`: commun vise, personnes concernees, dommage evite;
- `Incubation`: questions ouvertes et retour borne;
- `Preuves`: PaquetRRV et limites;
- `Options`: reparer, adapter, construire, ne pas construire;
- `Anti-capture`: revue des risques de rente ou dependance;
- `Prises en charge`: porteurs responsables et engagements;
- `Dissent`: objections preservees;
- `Decisions locales`: orientations par noeud/collectif;
- `Reprises`: forks, maintenance, abandon propre.

Les votes existants peuvent rester comme signal social faible. Ils ne doivent
pas piloter la promotion.

## 9. Mesures honnetes

Ne pas mesurer:

- croissance brute;
- conversion idee -> app;
- top votes;
- temps moyen vers sprint;
- volume GPU consomme;
- nombre de "features shipped".

Mesurer plutot:

- besoins clarifies;
- packs RRV complets;
- dissent integre;
- risques d'extraction evites;
- projets rejetes avec raison;
- projets repris ou forkables;
- communs publies avec source/provenance/licence;
- maintenance apres publication;
- contributions non-GPU reconnues;
- accessibilite offline ou low-bandwidth;
- donnees restees locales ou exportables;
- cout compute justifie par utilite sociale.

## 10. Domaines prioritaires

Priorite haute:

- Babel/Gutenberg et lecture/traduction libre;
- education offline;
- repair notebook;
- public document ledger;
- cultural archive pack;
- crisis readonly pack;
- EHPAD/famille sans cloud;
- capteurs citoyens et environnement local;
- open hardware build ledger.

Priorite plus tardive ou stricte:

- sante operationnelle;
- energie critique;
- decisions democratiques liantes;
- allegations supply-chain;
- compute scientifique a grande echelle.

Ces domaines peuvent commencer en read-only, pack de connaissances, simulation,
preuve RRV ou app locale limitee avant toute promesse operationnelle.

## 11. Non-goals

- Pas de promesse humanitaire de production pendant le pilote ferme.
- Pas de morale encodee comme censure centrale.
- Pas de licence custom anti-capitaliste fragile.
- Pas de token, DAO, gouvernance par stake ou hardware.
- Pas de classement global des idees.
- Pas de preuve que "la communaute" maintiendra sans porteur.
- Pas d'app sandboxee qui lance l'Operator.
- Pas de LLM oracle.

## 12. Decision de recherche

`idea_development_workflow_provider_neutral.md` reste utile pour la mecanique
provider/capabilities. `idea_creation_science_rrv_workflow.md` reste utile pour
la psychologie et les risques cognitifs. Mais le cadre produit correct est
celui-ci:

```text
Atelier des communs > workflow provider-neutral > future Ideas Hub app
```

La prochaine etape, si l'humain veut agir, n'est pas un sprint direct. C'est
de promouvoir cette doctrine en `docs/idea-workflow/` ou
`docs/agent/IDEA_WORKFLOW.md`, puis de definir schemas, templates et validator
non-sprint autour des gates NR0-NR13.
