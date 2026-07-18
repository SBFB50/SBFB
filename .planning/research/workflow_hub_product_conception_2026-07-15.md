# Workflow Hub - conception produit dediee

Statut: Product Research Pack hors sprint. Il ne cree aucun sprint actif et
n'accorde aucune autorite a l'app.

Date initiale: 2026-07-15.
Mise a jour du plan: 2026-07-17.

Document amont autoritaire pour l'architecture, la securite, le runtime et le
protocole:

- `.planning/research/workflow_agents_app_conception_ultradeep_2026-07-15.md`

Ce document est le compagnon normatif UX du Research Pack global. Il ne definit
plus de sprint ni de sequence autonome. Le plan canonique unique compte 36
phases A a AJ dans le chapitre 24 du document global et reste a faire verifier
independamment avant activation.

Ce document isole volontairement **Workflow Hub**, l'application visible par
l'utilisateur. Il ne redocumente pas le moteur Rust, les schemas protocolaires,
les workers SBFB ou l'implementation interne des packs metier.

En cas de conflit, `PROCESS.md` et `AGENT_SYSTEM.md` prevalent pour le processus,
puis le Research Pack global prevaut pour scope, architecture, securite,
runtime, protocole, sequencement et closure. Ce document prevaut seulement pour
l'architecture produit, les parcours et les etats de Workflow Hub.

## 1. Definition simple

Workflow Hub est l'application dans laquelle une personne peut:

1. decouvrir un workflow ou partir d'un modele;
2. creer et configurer des agents;
3. construire un workflow LLM/multi-agents;
4. comprendre ses donnees, outils, couts et permissions;
5. le valider, le tester et comparer ses resultats;
6. lancer et suivre des executions durables;
7. intervenir lorsqu'une decision humaine est requise;
8. publier, installer, activer, forker, mettre a jour ou revoquer un pack;
9. reutiliser ses artefacts depuis Ideas, Conception, Factory, Babel ou une
   autre app autorisee.

Workflow Hub est un **outil operationnel**, pas une landing page, un simple
catalogue ou un canvas de demonstration.

## 2. Distinction des noms et responsabilites

### 2.1 Workflow Hub n'est pas Idea Hub

- **Workflow Hub** est l'outil generique de creation, d'execution et de gestion
  des workflows.
- **Idea Hub / Ideas** est un workflow metier d'ideation qui utilise cette base.
- Ideas peut etre fourni comme pack first-party et comme exemple de bout en
  bout, mais il ne determine pas toute l'interface du Hub.

### 2.2 Workflow Hub n'est pas le Workflow Runtime

Workflow Hub affiche, edite et demande des operations. Il ne possede pas
l'autorite d'execution.

| Couche | Responsabilite |
|---|---|
| Workflow Hub | UX, brouillons, projections d'edition, intentions, suivi et inspection |
| Shell hote | identite de l'app, grants, secrets, confirmations privilegiees et quotas |
| Workflow Runtime | compilation autoritaire, journal, scheduler, effets, reprise et provenance |
| Protocole SBFB | publication, distribution, artefacts et calcul terminal borne |
| Packs metier | logique declarative Ideas, Conception, Factory, Babel et extensions |

Le Hub ne contient donc:

- ni credential provider;
- ni cle de signature privee;
- ni base autoritaire des runs;
- ni execution directe d'un outil privilegie;
- ni scheduler distribue;
- ni appel iframe-vers-iframe.

### 2.3 Relation confirmee entre les noms

Decision PO du 2026-07-17: **Workflow Hub** est le nom de l'application visible
de conception et d'exploitation. **Workflow Agents** designe le chantier global
qui livre cette app, le runtime Rust, les contrats portables, les adaptateurs et
l'integration protocole. Il ne s'agit ni de deux produits concurrents ni de deux
sprints. Le kickoff peut ratifier les noms techniques de crates/schemas, mais
ne peut pas recreer un plan Hub autonome.

## 3. Publics et travaux a accomplir

| Public | Besoin principal |
|---|---|
| Debutant | partir d'un modele, comprendre chaque etape et executer sans JSON |
| Auteur de workflow | composer agents, schemas, branches, outils, budgets et evals |
| Operateur | suivre, reprendre, annuler et diagnostiquer les runs |
| Auditeur | inspecter digests, grants, provenance, couts, donnees et preuves |
| Auteur de pack | evaluer, signer, publier, versionner, deprecier et revoquer |
| App metier | invoquer une operation headless et recevoir un artefact type |
| Utilisateur d'AT | accomplir les memes parcours sans canvas, souris, vision ou son |

Un meme utilisateur peut passer d'un role a l'autre. L'interface ne doit pas
creer un produit simplifie qui masque la confiance, ni un produit expert qui
oblige tout le monde a lire des structures brutes.

## 4. Ambition du MVP Workflow Hub

Le MVP est une **premiere version complete pour pilote ferme**, pas une demo.
Un seul sprint peut contenir autant de phases que necessaire. Les phases
ordonnent le travail; elles ne reduisent pas le resultat attendu.

Le MVP Hub n'est complet que si les neuf piliers suivants sont utilisables de
bout en bout.

### Pilier H1 - Bibliotheque de travail

- workflows recents, locaux, installes, publies et forks;
- recherche et filtres;
- modeles first-party;
- versions, statut, auteur, licence et digest visibles;
- reprise rapide d'un brouillon ou d'un run;
- aucun ranking global presente comme une verite.

### Pilier H2 - Auteur complet

- workflow vide, template, duplication ou fork;
- import declaratif borne avec rapport `mapped/approximated/refused`;
- canvas, outline et mode guide lineaire sur le meme modele;
- schemas d'entree/sortie et ports types;
- branches, parallelisme, joins, maps, boucles bornees et sous-workflows;
- undo/redo, autosave, recherche et recovery;
- diff semantique distinct du diff de layout;
- aucune modification JSON obligatoire.

### Pilier H3 - Registre d'agents

- agents reutilisables et versionnes;
- prompt, role, schemas, model policy, outils, contexte et memoire;
- handoff, agent-as-tool et appel simple differencies;
- budgets, timeouts et limites d'iteration visibles;
- dependances et usages de l'agent recherchables;
- comparaison entre deux versions.

### Pilier H4 - Validation et confiance avant run

- erreurs de schema et connexions invalides;
- boucle non bornee ou join incoherent;
- capacites, donnees sortantes et destinations;
- placement local, cloud autorise ou feuille SBFB;
- cout et budget previsibles;
- points de validation humaine;
- explication simple de ce que le workflow va faire.

### Pilier H5 - Runs durables et interventions

- timeline live et historique rejouable;
- statuts precis, tentatives et retries distincts;
- pause, attente, reprise et annulation honnete;
- fermeture de l'iframe sans perte du run;
- reprise apres crash du runtime;
- `outcome_unknown` explique sans faux succes;
- decisions humaines rendues dans le shell hote;
- artefacts, receipts et provenance inspectables.

### Pilier H6 - Evaluations

- scenarios et datasets versionnes;
- resultats attendus et contraintes de trajectoire;
- graders deterministes avant grader LLM;
- comparaison par version, modele, provider, cout et latence;
- revue humaine non remplacee par un score global;
- regression visible avant publication.

### Pilier H7 - Catalogue et cycle du pack

- decouverte par nom, domaine, schema, langue, licence et capacite;
- inspection avant installation;
- installation desactivee;
- activation avec grants explicites;
- publication et indexation;
- fork, update avec nouveau consentement, rollback et depreciation;
- revocation locale sans destruction de l'historique;
- aucune mise a jour silencieuse.

### Pilier H8 - Handoffs inter-apps

- operations headless publiees par schema;
- artefacts types plutot que texte colle;
- child-runs visibles, dedupliques et bornes;
- identite de l'app appelante;
- profondeur, cycles et budgets affiches;
- chaine Ideas -> Conception -> Factory observable;
- operation Babel consommable sans moteur parallele.

### Pilier H9 - Accessibilite maximale et i18n

- aucun parcours central reserve au canvas ou a la souris;
- app et confirmations du shell testees comme un seul processus;
- francais complet au lancement;
- architecture de traduction et RTL;
- preferences fonctionnelles empilables et privees;
- tests automatiques, manuels, multi-AT et avec personnes handicapees.

## 5. Architecture de l'information

La premiere vue est le produit lui-meme, pas une page marketing.

```text
+-----------------------------------------------------------------------+
| Workflow Hub | Rechercher | Nouveau | Notifications | Profil          |
+----------------+------------------------------------------------------+
| Bibliotheque   | contenu de l'espace actif                            |
| Catalogue      |                                                      |
| Agents         | workflow, run, eval ou fiche pack                    |
| Capacites      |                                                      |
| Runs           |                                                      |
| Evaluations    |                                                      |
+----------------+------------------------------------------------------+
```

Navigation de premier niveau:

| Espace | Contenu |
|---|---|
| Bibliotheque | brouillons, workflows, packs installes, templates et favoris locaux |
| Catalogue | packs publies, forks, versions, sources, licences et preuves |
| Agents | definitions reutilisables, versions, dependances et usages |
| Capacites | donnees, destinations, outils, grants demandes/effectifs et couts; gestion privilegiee renvoyee vers le shell |
| Runs | executions, attentes humaines, echecs, reprises et couts |
| Evaluations | datasets, scenarios, comparaisons et regressions |

L'editeur, la publication et l'inspection sont des espaces contextuels ouverts
depuis un workflow ou un pack. Ils ne gonflent pas inutilement la navigation
globale.

### 5.1 Commandes globales

- **Rechercher** couvre workflows, packs, agents, runs et artefacts selon les
  droits locaux;
- **Nouveau** ouvre directement les choix Workflow vide, Modele, Fork et
  Import, sans menu marketing;
- **Notifications** est une file d'actions: attente humaine, run echoue, grant
  expire, update ou revocation; aucun token LLM ou message promotionnel;
- **Profil** montre identite locale, noeud, langue, preferences fonctionnelles
  et sessions; credentials et authentification restent rendus par le shell;
- chaque compteur, menu et changement de contexte est accessible, consultable
  en liste et sans mise a jour qui vole le focus.

## 6. Ecran Bibliotheque

La Bibliotheque doit permettre de travailler rapidement et de comprendre
l'etat de chaque objet.

### Contenu attendu

- recherche plein texte;
- filtres statut, source, domaine, version, langue et licence;
- vues liste et grille compacte;
- sections Recents, Brouillons, Installes, Publies et Modeles;
- version et digest visibles sans ouvrir une fiche;
- actions Nouveau, Ouvrir, Forker, Comparer et Archiver;
- statut d'evaluation et de grants sans badge de qualite trompeur;
- reprise du dernier contexte sans changer de vue de maniere inattendue.

### Etats obligatoires

- chargement avec structure stable;
- bibliotheque vide avec action de creation;
- recherche sans resultat;
- hors ligne avec contenu local identifiable;
- index distant indisponible;
- objet local orphelin ou version retiree;
- conflit entre brouillon local et version publiee.

## 7. Workspace d'edition

### 7.1 Structure

```text
barre: retour | nom/version | autosave | valider | evaluer | lancer | publier
rail: nodes | agents | sous-workflows | recherche
centre: canvas | outline | mode guide
inspecteur: configuration typee | I/O | modele | outils | budgets
tiroir: erreurs | timeline test | artefacts | diff
```

Les panneaux sont denses et previsibles. Il n'y a pas de cartes decoratives
imbriquees ni de redimensionnement qui deplace une action pendant son usage.

### 7.2 Trois projections de premiere classe

**Canvas**

- composition spatiale, groupes, ports et chemins;
- zoom, pan et selection;
- layout separe de la semantique executable;
- alternatives a chaque drag et geste complexe.

**Outline**

- liste structuree des nodes, branches et connexions;
- edition complete par formulaires;
- navigation erreur -> node -> champ;
- raccourcis remappables et focus stable;
- descriptions textuelles du graphe.

**Mode guide**

- une etape ou decision a la fois;
- langage simple et consequences visibles;
- ajout et connexion sans vocabulaire graphique obligatoire;
- reprise de contexte apres interruption;
- meme puissance semantique que le canvas et l'outline.

Les trois projections utilisent les memes commandes typees et produisent le
meme digest semantique. Seules les metadonnees de layout peuvent differer.

### 7.3 Assistance IA a l'auteur

Le Hub peut proposer un brouillon depuis une demande en francais, mais la
sortie reste un **diff declaratif**:

- chaque agent, node, connexion, outil et budget propose est visible;
- l'utilisateur accepte, modifie ou refuse le diff;
- aucun grant n'est accorde automatiquement;
- aucun run ou publication n'est lance automatiquement;
- le modele ne modifie jamais silencieusement le plan compile;
- la provenance de la proposition est conservee.

## 8. Registre d'agents

La fiche d'un agent montre:

- nom, version, digest, auteur et licence;
- role et limites en langage simple;
- prompt et variables;
- schemas d'entree/sortie;
- provider/model policy;
- outils et capacites demandees;
- contexte recu et donnees transmises;
- memoire eventuelle et retention;
- budgets, timeout, retries et iterations;
- workflows qui utilisent cette version;
- evals et comparaison avec une autre version.

Modifier un agent partage cree une nouvelle version ou un fork explicite. Le
Hub ne change jamais silencieusement les runs ou packs epingles.

## 9. Capacites, validation et lancement

### 9.1 Espace Capacites

L'espace Capacites est une vue de confiance et de diagnostic, pas un panneau
qui accorde lui-meme des droits. Il permet de:

- filtrer par workflow, pack, agent, outil, destination et classe de donnees;
- distinguer capacite demandee, declaree, accordee, expiree et refusee;
- afficher placement local/cloud/SBFB, egress, cout et reversibilite;
- retrouver quel node et quelle version demandent chaque droit;
- comparer les capacites entre deux versions;
- ouvrir l'action hote appropriee pour accorder, reduire ou revoquer;
- expliquer une absence de grant sans exposer de secret;
- rester lisible offline a partir du dernier snapshot, marque comme non frais.

Etats obligatoires: aucun droit demande, demandes non accordees, grants
partiels, grant expire, provider indisponible, politique locale plus stricte,
version revoquee et donnees interdites a l'egress.

### 9.2 Preflight et lancement

Avant lancement, le Hub presente un preflight unique:

1. erreurs bloquantes du graphe;
2. avertissements et limites connues;
3. version/digest du workflow et des agents;
4. placement de chaque node: local, cloud autorise ou SBFB;
5. donnees qui quittent le noeud et leur destination;
6. outils, classe d'effet et reversibilite;
7. grants disponibles et grants manquants;
8. cout, budget, timeout et limites;
9. validations humaines prevues;
10. resume simple du resultat attendu.

Le bouton Lancer transmet une intention. Toute autorisation manquante ou
confirmation sensible est rendue par le shell hote et liee au digest exact.

## 10. Centre des Runs

### 10.1 Etats visibles

- `queued`;
- `running`;
- `waiting_human`;
- `waiting_external`;
- `paused`;
- `completed`;
- `failed`;
- `cancel_requested`;
- `cancelled`;
- `outcome_unknown`.

Chaque statut possede un libelle francais, une explication et les actions
reellement possibles. La couleur ou une animation ne sont jamais l'unique
signal.

### 10.2 Inspection

- timeline par sequence et par node;
- retries distincts des nouveaux runs;
- donnees redigees selon la permission;
- handoffs et child-runs;
- appels d'outils et receipts;
- decisions humaines;
- cout, latence et provider;
- artefacts et provenance;
- comparaison avec une execution precedente.

Les evenements LLM sont groupes par transition semantique. Le Hub n'affiche et
n'annonce pas chaque token comme un nouvel evenement.

### 10.3 Reprise et annulation

- fermer le Hub ne termine pas le run;
- une reconnexion reprend la timeline depuis la derniere sequence connue;
- une demande de cancel distingue demande, confirmation et impossibilite;
- un effet ambigu produit `outcome_unknown`, jamais un succes invente;
- un fork depuis checkpoint cree un nouveau run et preserve l'historique.

## 11. Evaluations et comparaison

L'espace Evaluations permet de:

- creer un scenario ou importer un dataset borne;
- choisir les sorties et trajectoires a verifier;
- definir budgets, providers et outils autorises;
- lancer une evaluation sans publication;
- comparer versions, modeles et providers;
- distinguer echec deterministe, jugement humain et score LLM;
- inspecter les cas individuels avant un resume agrege;
- marquer une regression comme bloquante;
- exporter un rapport redige sans secret.

Aucun score unique ne doit etre affiche comme une verite universelle sur un
agent ou un workflow.

## 12. Catalogue et fiche Pack

### 12.1 Recherche et decouverte

- nom, description et domaine;
- schemas exposes;
- langue et licence;
- auteur, publisher, signatures et provenance;
- version, digest, forks et continuations;
- providers et outils necessaires;
- capacites et classes de donnees;
- evals publiees et limites;
- curation locale explicite.

Le catalogue n'impose ni ranking mondial, ni pay-to-rank, ni autorite centrale
obligatoire.

### 12.2 Cycle visible

```text
brouillon -> valide -> evalue -> signe -> publie -> indexe
  -> inspecte -> installe desactive -> active avec grants
  -> fork / update avec re-consentement / rollback / depreciation / revocation
```

Publication, installation et activation sont trois decisions distinctes. Une
version revoquee reste consultable pour audit, mais ne peut pas demarrer un
nouveau run.

### 12.3 Import declaratif

Le Hub accepte seulement un subset de donnees de formats externes, jamais leur
runtime ou leur code:

- fichier traite comme hostile avant lecture;
- taille, profondeur, nombres de nodes et schemas bornes;
- aucun `eval`, script, credential ou plugin importe;
- apercu du brouillon avant enregistrement;
- rapport `mapped`, `approximated`, `unsupported` et `refused`;
- prompts et secrets potentiels rediges avant affichage/export;
- resultat non signe, non accorde et non executable avant validation locale;
- Flowise comme premier adaptateur data-only; n8n/LangGraph restent hors
  compatibilite executable.

## 13. Handoffs entre apps

Le Hub rend les chaines inter-apps inspectables sans appeler une autre iframe.

```text
Ideas -> DecisionPack
DecisionPack -> Conception -> ConceptionPack
ConceptionPack -> Factory -> DiffAndEvidencePack
Babel -> TranslationBatch -> PublishedTranslation
```

Pour chaque appel, l'utilisateur peut voir:

- app et operation appelantes;
- pack, version et schema invoques;
- artefacts d'entree;
- budget et profondeur restants;
- child-run cree ou deduplique;
- resultat, receipt et provenance;
- refus pour cycle, depth, schema ou grant.

## 14. Accessibilite maximale

L'accessibilite est un plancher du MVP Hub et une gate de chaque phase UI.

### 14.1 Referentiels

- WCAG 2.2 AA bloquant sur chaque surface, etat et processus complet;
- chaque critere AAA applicable evalue et archive;
- ATAG 2.0 Part A AA pour l'outil auteur et Part B lorsque applicable;
- COGA pour comprehension, memoire, attention et recuperation d'erreur;
- EN 301 549 applicable figee au kickoff;
- HTML natif avant ARIA et tests reels des widgets riches.

### 14.2 Couverture fonctionnelle

| Besoin | Exigence Hub |
|---|---|
| Cecite/braille | landmarks, noms/roles/etats, outline complet, descriptions du graphe, tableaux et transcripts |
| Basse vision | zoom 400 %, reflow 320 CSS px, texte agrandi, focus fort et magnification |
| Daltonisme/sans couleur | texte, forme et icone; forced colors; aucun signal couleur seul |
| Surdite/malentendance | aucun signal sonore seul, captions et transcripts |
| Moteur/dexterite | clavier, switch, voix, tactile, single-pointer sans drag et grandes cibles |
| Parole | aucune commande vocale obligatoire; labels visibles inclus dans les noms accessibles |
| Cognitif/apprentissage | mode guide, langage simple, aide stable, erreurs reparables et consequences visibles |
| Dyslexie/lecture | fonte, espacement, largeur de ligne et lecture assistable |
| Attention/neurodivergence | densite reduite, mode focus, annonces groupees et interruptions controlables |
| Memoire/fatigue | autosave, historique, reprise, aucune memorisation ou limite de temps bloquante |
| Vestibulaire/photosensibilite | reduced motion, aucun mouvement indispensable et aucun flash dangereux |
| Besoins combines | preferences empilables; la valeur la plus protectrice gagne par axe |

### 14.3 Frontiere Hub/shell

- iframe titree et navigation hote/app explicite;
- focus deplace vers la confirmation hote puis restaure deterministiquement;
- iframe inactive pendant le dialogue hote;
- action, donnees, destination, cout, effet et refus expliques;
- aucun type-to-confirm, calcul, CAPTCHA cognitif ou timer court;
- file d'approbations accessible meme si le Hub est ferme;
- meme profil contraste, echelle, mouvement et densite des deux cotes.

### 14.4 Confidentialite

- preferences transmises comme axes fonctionnels resolus;
- aucun diagnostic ou `needs[]` dans DOM, logs, prompts, runs ou reseau;
- semantique accessible toujours active, sans mode lecteur d'ecran;
- aucune detection d'une technologie d'assistance;
- sync/export des preferences seulement par opt-in explicite et revocable.

### 14.5 Preuve de livraison

- axe et tests composants sur chaque etat;
- Playwright clavier, focus, forced colors, reflow, text spacing et motion;
- parite generee canvas/outline/mode guide;
- NVDA, JAWS, VoiceOver, TalkBack et Orca;
- braille, Voice Access/Dragon, Voice Control, switch et tactile;
- Windows, macOS, iOS/iPadOS, Android et Ubuntu;
- au moins douze participants handicapes remuneres en deux rounds;
- zero barriere P0/P1 sur les parcours critiques;
- rapport versionne des preuves et limites connues.

La promesse est une accessibilite maximale demontree, pas une affirmation
impossible de perfection universelle.

## 15. i18n, responsive et performance

### 15.1 i18n

- interface initiale complete en francais;
- IDs et schemas stables en anglais technique;
- aucun texte utilisateur construit par concatenation fragile;
- erreurs avec code stable et message localise;
- RTL sur bibliotheque, editeur, timeline et formulaires;
- noms de packs traites comme contenu, pas comme chaine UI;
- localisation des contenus dynamiques sans perdre leur provenance.

### 15.2 Responsive

- desktop: toutes les projections et inspecteurs;
- tablette: canvas adapte et modes outline/guide complets;
- mobile: bibliotheque, mode guide, runs, HITL, catalogue et publication
  utilisables sans parcours tronque;
- aucun double scroll iframe/shell;
- panneaux et barres sans chevauchement a 320 CSS px et 400 %.

### 15.3 Performance

Les budgets chiffres sont ratifies au kickoff, puis gates. Le Hub doit au
minimum prevoir:

- bundle initial sans moteur canvas lourd;
- canvas charge en chunk lazy;
- bibliotheques et timelines virtualisees;
- events LLM coalesces;
- artefacts charges a la demande;
- recherche locale indexee;
- grands graphes avec niveaux de detail;
- aucune variation de layout provoquee par un statut live.

## 16. Etats d'erreur et mode degrade

Le Hub doit avoir des parcours explicites pour:

- shell non joignable;
- runtime non disponible;
- bridge refuse ou version incompatible;
- pack invalide ou signature incorrecte;
- provider local absent;
- credential cloud manquant;
- worker SBFB indisponible;
- index catalogue hors ligne;
- run partiellement observable;
- evenement perdu puis replay;
- permission expiree;
- espace de stockage insuffisant;
- profil d'accessibilite impossible a charger.

Chaque erreur indique cause connue, impact, donnees preservees et prochaine
action. Le brouillon reste recuperable autant que possible.

## 17. Contrat fonctionnel avec l'hote

Le Hub a besoin de familles de commandes typees, sans figer ici leurs noms de
wire definitifs.

| Intention Hub | Reponse attendue |
|---|---|
| chercher/lire un pack | resultats, provenance, version et disponibilite |
| enregistrer un brouillon | revision locale et statut de sauvegarde |
| valider/compiler | diagnostics structures et digest du plan |
| lancer un run | run ID ou refus explicite |
| suivre un run | snapshot puis events sequences/replay |
| demander pause/cancel/retry | etat accepte, refuse ou ambigu |
| soumettre une decision humaine | receipt lie au digest exact |
| lire un artefact | metadata puis contenu selon permission |
| evaluer/comparer | run d'eval, cas et rapport |
| publier/installer/activer | intention puis confirmation hote et receipt |
| invoquer une operation | child-run et ArtifactRef/erreur typee |
| appliquer les preferences | profil fonctionnel resolu, versionne et minimal |

Le Hub ne fabrique jamais un succes local si l'hote refuse ou ne confirme pas
l'operation.

## 18. Acceptations bloquantes du MVP Hub

### HUB-T1 - Auteur sans JSON

Creer le meme workflow multi-agents par canvas, outline et mode guide. Les trois
chemins produisent le meme digest semantique, supportent undo/autosave/recovery
et corrigent les memes erreurs.

### HUB-T2 - Confiance avant run

Comprendre agents, providers, outils, donnees, couts, grants et HITL avant de
lancer. L'espace Capacites distingue demandes, grants effectifs, refus et
expiration, puis delegue toute mutation au shell hote. Toute mutation apres
consentement invalide la confirmation liee au digest precedent.

### HUB-T3 - Run durable

Lancer, fermer le Hub, tuer/redemarrer le runtime, reprendre le meme frontier,
intervenir depuis le shell et terminer sans double effet. Afficher
`outcome_unknown` lorsqu'une confirmation externe manque.

### HUB-T4 - Catalogue et cycle complet

Decouvrir, inspecter, installer desactive, activer, executer, forker, mettre a
jour avec re-consentement, rollback, deprecier et revoquer. Aucun nouveau run
ne part sur une version revoquee.

### HUB-T5 - Cross-app

Avec les packs externes reellement executables, ouvrir, comprendre, forker,
reevaluer et invoquer Ideas, Conception, Factory et Babel. Observer Ideas ->
Conception -> Factory avec artefacts types et child-runs, puis une operation
Babel complete. Deduplications, cycle, depth, budget et refus de schema sont
visibles et comprehensibles. Leur logique metier ne migre pas dans le bundle du
Hub.

Pour Factory, le Hub doit montrer le processus reel, pas une simple carte
`implementation -> tests`: audit Phase 0, kickoff/plan/design review, preflight,
driver, trois blocs de verification, review `PASS-PENDING`, Codex brut,
reconciliation/reparation, verdict final, intention de commit confirmee par
l'humain, carries, verification et audit. Les phases globales D, Z, AA, AB, AC
et AJ portent ce contrat et sa migration.

### HUB-T6 - Evaluations et provenance

Comparer deux versions, detecter une regression, inspecter chaque cas, exporter
une preuve redigee et refuser toute mutation d'un digest ou receipt.

### HUB-T7 - Accessibilite complete

Bibliotheque, auteur, validation, run/HITL, eval, catalogue, installation,
activation et publication sont realisables sans souris, avec lecteur d'ecran,
mode guide, zoom 400 %, forced colors, reduced motion, voix, switch et tactile.
Le parcours app + shell passe WCAG 2.2 AA sans echec connu; les matrices AAA
applicable, ATAG Part A/Part B applicable, COGA et EN 301 549 sont archivees
sans ecart P0/P1. Les preferences empilees s'appliquent avant le premier rendu.
La matrice multi-OS/AT et les deux rounds utilisateurs sont archives.

### HUB-T8 - Resilience produit

Les etats offline, bridge refuse, provider absent, run partiel, storage plein et
pack invalide preservent l'orientation, expliquent la situation et n'inventent
ni execution ni sauvegarde. Recherche globale, Nouveau, Notifications et Profil
restent utilisables et ne volent jamais le focus.

### HUB-T9 - Import data-only

Importer une fixture Flowise valide puis un fichier hostile. Le premier produit
un brouillon et un rapport de compatibilite; le second est refuse sans execution,
secret, grant, signature ou effet de bord.

### HUB-T10 - Verite d'integration

Les parcours finaux utilisent le Workflow Runtime, le shell hote et les
fixtures multi-noeuds reels du repo. Les mocks hermetiques restent utiles en CI,
mais ne suffisent pas a fermer le pilote.

## 19. Gates proposes

| Gate | Preuve attendue |
|---|---|
| `HUB-SCOPE` | HUB-T1 a HUB-T10 complets; aucun espace central remplace par un placeholder |
| `HUB-MODEL-PARITY` | commandes typees communes et digest identique canvas/outline/mode guide |
| `HUB-DRAFT` | autosave, undo/redo, recovery et conflits testes |
| `HUB-SHELL` | recherche/nouveau/notifications/profil ont contenus, erreurs, clavier et focus testes |
| `HUB-BRIDGE` | refus/timeout/replay/backpressure affiches sans faux succes |
| `HUB-TRUST` | version, digest, grants, donnees, destination, cout et HITL visibles |
| `HUB-CAPACITIES` | demandes/effectifs/refus/expiration et diffs visibles; mutation seulement par le shell |
| `HUB-RUN` | fermeture/reconnexion/crash/reprise sans perte d'orientation ni double rendu |
| `HUB-EVAL` | comparaison, regression et preuves inspectables |
| `HUB-LIFECYCLE` | publish/install/activate/fork/update/rollback/deprecate/revoke complets |
| `HUB-APPCALL` | quatre packs executables ouverts/forkes/reevalues/invoques; child-runs et refus visibles |
| `HUB-IMPORT` | subset data-only borne; rapport de compatibilite; aucun code/secret execute ou importe |
| `HUB-AI-DIFF` | assistance francaise produit uniquement un diff declaratif source, inspecte et accepte explicitement |
| `HUB-MOBILE-E2E` | mode guide, runs, HITL, catalogue et lifecycle complets sur viewport mobile/tactile |
| `HUB-INTEGRATION` | E2E final avec runtime, shell et fixtures protocole reels; mocks seuls insuffisants |
| `HUB-A11Y-WCAG` | zero echec A/AA connu sur surface, etat ou processus inclus |
| `HUB-A11Y-AAA` | chaque critere applicable possede decision, preuve et exception arbitree |
| `HUB-A11Y-AUTHORING` | outil auteur et sorties applicables evalues contre ATAG Part A/B |
| `HUB-A11Y-COGA` | comprehension, memoire, attention, aide et recovery testes |
| `HUB-A11Y-EN301549` | version applicable figee et matrice sans ecart P0/P1 non resolu |
| `HUB-A11Y-PARITY` | tous les parcours critiques sans canvas, drag ou souris |
| `HUB-A11Y-AT` | matrice multi-OS/AT versionnee et archivee |
| `HUB-A11Y-USERS` | deux rounds remuneres, zero barriere P0/P1 critique |
| `HUB-A11Y-PRIVACY` | aucun diagnostic/need dans DOM, bridge, prompt, run, log ou reseau |
| `HUB-A11Y-PREFS` | preferences empilees, privees et appliquees avant premier rendu |
| `HUB-I18N` | francais complet, extraction/parite et RTL testes |
| `HUB-PERF` | budgets chiffres bibliotheque/canvas/timeline et size-limit passes |

Traceabilite minimale vers le plan global:

| Acceptance Hub | Phases proprietaires principales | Gates globaux associes |
|---|---|---|
| HUB-T1 Auteur | S, T, U | `G-SCOPE-FLOOR`, gates auteur/accessibilite |
| HUB-T2 Confiance | C, L, U | `G-BRIDGE-GRANT`, `G-CAPABILITY`, `G-HITL` |
| HUB-T3 Run durable | E, I, J, K, V | `G-STORE`, `G-UPGRADE-REPLAY`, `G-EFFECT` |
| HUB-T4 Lifecycle | X | `G-PUBLISH-RECOVERY`, `G-RRV-PROJECTION`, `G-LIFECYCLE` |
| HUB-T5 Cross-app | D, Y, Z a AF, AJ | `G-APPCALL`, `G-FACTORY-CONTRACT`, `G-FACTORY-CUTOVER` |
| HUB-T6 Evals | M, W | `G-PROVENANCE`, `G-EVAL` |
| HUB-T7 Accessibilite | B puis chaque phase UI, consolidation AI | gates `G-A11Y-*` et `HUB-A11Y-*` |
| HUB-T8 Resilience | R, V, AI | `G-AUTHORITY`, `G-PERF`, `HUB-BRIDGE` |
| HUB-T9 Import | AG, AH | `HUB-IMPORT`, `G-SUPPLY-CHAIN` |
| HUB-T10 Integration | C a AJ, pre-cutover AI puis fermeture AJ | `G-SCOPE-FLOOR`, `HUB-INTEGRATION` |

## 20. Crosswalk vers le plan global A a AJ

**Sequence autonome superseded.** L'ancienne liste Hub de 32 phases n'est plus
activable. Le seul sequencement est le plan de 36 phases A a AJ du chapitre 24
du Research Pack global. Le tableau ci-dessous indique ou vivent les obligations
Hub; il ne cree pas un second DAG.

| Phases globales | Obligations Workflow Hub |
|---|---|
| A | scope HUB-T1 a HUB-T10, gates, budgets, ressources AT/panel et matrice de preuves |
| B | tokens, preboot, preferences privees, i18n/RTL, architecture responsive et harness accessibilite |
| C | erreurs/refus bridge, grants expires/revoques et aucune apparence de succes local |
| D a Q | contrats runtime/Factory/host visibles et comprehensibles, sans deplacer autorite ou secrets dans l'iframe |
| R | shell app, accueil Bibliotheque, recherche, recents, templates, versions, offline et reconnexion |
| S | registre d'agents, brouillons, autosave, undo/redo, recovery, conflits, versions et diff |
| T | modele auteur partage, canvas, outline et mode guide a parite complete |
| U | inspecteurs, schemas, providers, outils, data/cout, capacites, grants et preflight |
| V | Runs, timeline, artifacts, child-runs, HITL, replay, pause/cancel/retry et unknown |
| W | datasets, cas, graders, comparaison, regressions et export de preuves |
| X | catalogue, Proof Card, RRV, publish/install/activate/update/rollback/deprecate/revoke |
| Y | ouverture, fork, reevaluation et invocation des packs; child-runs et refus visibles |
| Z a AC | Factory complet rendu lisible: kickoff, preflight, implementation, review, Codex, reconciliation, commit humain, carries et audit |
| AD a AF | packs Ideas, Conception et Babel externes mais executables/forkables depuis le Hub |
| AG | assistance auteur par diff declaratif et import Flowise data-only hostile |
| AH | securite UX, redaction, CSP, imports malveillants, usurpation et supply chain |
| AI | fault/offline, scale/performance, mobile, i18n/RTL, multi-AT, deux rounds utilisateurs, HUB-T1 a T9 et pre-cutover HUB-T10 |
| AJ | shadow/cutover Factory, smoke et acceptance post-bascule fermant HUB-T10, aide, preuves, limites et go/no-go pilote ferme |

Accessibilite, securite, i18n et tests ne commencent pas en AI: ils sont gates
dans chaque phase qui ajoute une surface. AI consolide et reteste seulement.

## 21. Hors du bundle Hub, mais pas hors du sprint global

Ces sujets n'appartiennent jamais a l'iframe ou au bundle de l'app. Les cinq
premiers restent cependant dans le scope du sprint global A a AJ et sont des
dependances bloquantes des parcours Hub, pas des carries implicites:

- implementation du reducer/scheduler et du journal Rust;
- format wire definitif de Task/Result/ArtifactRef;
- gestion interne du keystore et des credentials;
- implementation des adaptateurs Ollama/cloud/MCP;
- protocole de publication et index distribue;

Les sujets suivants restent hors du MVP global sauf decision PO contraire:

- scheduler de DAG distribue ou failover multi-leader;
- replication live P2P de l'etat prive des runs;
- co-edition temps reel;
- execution de code importe ou plugins frontend tiers;
- marketplace centrale, ranking mondial ou token governance;
- quatre applications metier completes integrees dans le bundle du Hub;
- commit, push, deploiement ou publication autonomes par un modele;
- promesse de production publique avant les preuves du pilote ferme.

Aucune closure Hub n'est possible avec des mocks si runtime, wire, credentials,
adaptateurs ou publication reels manquent. Leur implementation reste dans les
couches globales et n'est jamais deplacee silencieusement dans l'iframe.

## 22. Decisions confirmees et a ratifier

### Confirme par le PO

- un document separe est dedie a Workflow Hub;
- le MVP doit etre audacieux et complet;
- le sprint peut contenir de nombreuses phases;
- l'accessibilite maximale fait partie du MVP;
- Workflow Hub est l'app visible et Workflow Agents le chantier global;
- un seul sprint global suit les 36 phases A a AJ; il n'existe plus de plan Hub
  autonome;
- Factory est le workflow fondateur et reprend tout le processus sprint Nexus,
  y compris Claude, Codex, reconciliation, gates, commit humain et audit;
- ce travail reste hors sprint tant que le kickoff n'est pas explicite.

### Details d'implementation a ratifier au kickoff

HUB-0 et HUB-1 sont confirmes ci-dessus. Les decisions restantes ne peuvent pas
contredire une decision WA ratifiee dans le Research Pack global.

| ID | Decision | Recommandation |
|---|---|---|
| HUB-2 | Vue par defaut | Bibliotheque operationnelle, jamais landing page |
| HUB-3 | Placement | app SBFB sandboxee plein ecran |
| HUB-4 | Edition | canvas + outline + mode guide a parite |
| HUB-5 | Mobile | mode guide, runs, HITL, catalogue et lifecycle complets |
| HUB-6 | Assistance IA | diff declaratif a accepter, aucun grant/run/publish autonome |
| HUB-7 | Catalogue | decentralise, curation explicite, aucun ranking global |
| HUB-8 | Accessibilite | socle complet du chapitre 14, gates a chaque phase |
| HUB-9 | Preferences | axes fonctionnels resolus, aucun diagnostic partage |
| HUB-10 | Reference packs | Ideas, Conception, Factory et Babel externes, executables, forkables, reevaluables et invocables depuis le Hub |

## 23. Sources internes

Source principale:

- `.planning/research/workflow_agents_app_conception_ultradeep_2026-07-15.md`
  - produit et parcours: chapitre 7;
  - publication/installation: chapitre 16;
  - UX/accessibilite: chapitre 20;
  - app candidate: chapitre 21.5;
  - MVP/gates/phases: chapitres 22 a 24.

Ancres repo a revalider au kickoff:

- `web/src/bridge/protocol.ts`
- `web/src/bridge/useBridge.ts`
- `crates/sbfb-manifest/src/lib.rs`
- `crates/nexus-core-rs/src/csp.rs`
- `crates/nexus-shell-daemon/src/storage_api.rs`
- `tools/factory-operator/src/preferences/accessibility.ts`
- `tools/factory-operator/scripts/check-accessibility-system.sh`
- `docs/factory/OPERATOR_ACCESSIBILITY_DESIGN_SYSTEM.md`

## 24. Sous-checklist Hub du futur kickoff global

Il n'existe pas de kickoff Workflow Hub separe. Le kickoff global A a AJ doit:

1. revalider le HEAD et les contrats bridge disponibles;
2. conserver HUB-0/HUB-1 comme decisions PO et ratifier HUB-2 a HUB-10 sans
   contredire WA-0 a WA-24;
3. transformer HUB-T1 a HUB-T10, `HUB-AI-DIFF` et `HUB-MOBILE-E2E` en E2E executables,
   chacun mappe a une phase, une commande, une fixture et un artefact;
4. reserver la matrice AT, les licences et le panel handicape en Phase A;
5. geler les commandes du modele auteur partage avant canvas/outline/guide;
6. livrer Bibliotheque et mode guide aussi serieusement que le canvas;
7. traiter les confirmations hote et les intentions commit/push comme partie du
   parcours, sans donner l'autorite finale a l'iframe;
8. afficher Factory Z-AC et sa bascule AJ comme processus complet, jamais comme
   un simple generer/tester;
9. ne jamais deplacer runtime, secrets ou autorite dans l'iframe;
10. refuser une closure avec placeholder sur evals, catalogue, lifecycle,
    cross-app, Factory ou accessibilite;
11. conserver le statut pilote ferme jusqu'aux preuves de readiness publique.
